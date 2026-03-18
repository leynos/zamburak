//! Governed execution entrypoint wrapping `full-monty` with Zamburak mediation.
//!
//! [`GovernedRunner`] orchestrates a governed execution session: it installs a
//! [`ZamburakObserver`] on a compiled [`MontyRun`], mediates every external-call
//! yield through an [`ExternalCallMediator`], and exposes the result as a
//! [`GovernedRunProgress`] enum that enriches standard `RunProgress` states with
//! governance metadata.

use std::sync::{Arc, Mutex};

use monty::{
    ExtFunctionResult, ExternalCallKind, MontyException, MontyObject, MontyRun, NoLimitTracker,
    PrintWriter, ResourceTracker, RunProgress, RuntimeObserverHandle,
};
use thiserror::Error;

use crate::external_call::{
    CallContext, ConfirmationContext, ExternalCallMediator, MediationDecision,
};
use crate::observer::ZamburakObserver;

/// Errors arising from governed execution.
#[derive(Debug, Error)]
pub enum GovernedRunError {
    /// The underlying Monty interpreter raised an exception.
    #[error("interpreter exception: {0}")]
    Interpreter(MontyException),

    /// The mediator's mutex was poisoned.
    #[error("mediator lock poisoned")]
    MediatorPoisoned,
}

impl From<MontyException> for GovernedRunError {
    fn from(exc: MontyException) -> Self {
        Self::Interpreter(exc)
    }
}

/// Outcome of a single governed execution step.
///
/// Mirrors `RunProgress` but enriches external-call yields with mediation
/// metadata and adds governance-specific states.
#[derive(Debug)]
#[non_exhaustive]
pub enum GovernedRunProgress<T: ResourceTracker> {
    /// Execution completed with a final value.
    Complete(MontyObject),

    /// Execution was denied at an external-call boundary.
    Denied {
        /// Human-readable reason the call was denied.
        reason: String,
        /// The function name that was denied.
        function_name: String,
        /// The call identifier for the denied call.
        call_id: u32,
    },

    /// Execution paused pending host confirmation for an external call.
    AwaitConfirmation {
        /// Confirmation context for the host to display.
        context: ConfirmationContext,
        /// The suspended execution state, resumable after confirmation.
        suspended: SuspendedCall<T>,
    },

    /// Execution paused for a name lookup the governed runner cannot resolve.
    NameLookup {
        /// The name being looked up.
        name: String,
        /// The underlying Monty `NameLookup` state for resume.
        inner: monty::NameLookup<T>,
    },

    /// Execution paused waiting for async futures to resolve.
    ResolveFutures(monty::ResolveFutures<T>),
}

/// A suspended external call awaiting host confirmation before resume.
#[derive(Debug)]
pub struct SuspendedCall<T: ResourceTracker> {
    /// The kind of suspension (function or OS call).
    kind: SuspendedCallKind<T>,
}

/// Internal discriminant for the two suspendable call types.
#[derive(Debug)]
enum SuspendedCallKind<T: ResourceTracker> {
    /// A suspended external function call.
    Function(monty::FunctionCall<T>),
    /// A suspended OS call.
    Os(monty::OsCall<T>),
}

impl<T: ResourceTracker> SuspendedCall<T> {
    /// Resumes the suspended call with a host-provided result.
    ///
    /// # Errors
    ///
    /// Returns `GovernedRunError::Interpreter` if the interpreter raises an
    /// exception on resume.
    pub fn resume(
        self,
        result: impl Into<ExtFunctionResult>,
        print: PrintWriter<'_>,
    ) -> Result<RunProgress<T>, GovernedRunError> {
        match self.kind {
            SuspendedCallKind::Function(call) => {
                call.resume(result, print).map_err(GovernedRunError::from)
            }
            SuspendedCallKind::Os(call) => {
                call.resume(result, print).map_err(GovernedRunError::from)
            }
        }
    }
}

/// Orchestrates governed execution of a Monty program.
///
/// Wraps a compiled [`MontyRun`] with a [`ZamburakObserver`] and mediates every
/// external-call yield through an [`ExternalCallMediator`]. Consumes itself on
/// execution because `MontyRun::start_with_observer` takes ownership of the
/// compiled program.
///
/// # Examples
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use zamburak_monty::{AllowAllMediator, ExternalCallMediator, GovernedRunner};
///
/// let runner = monty::MontyRun::new(
///     "x = 1 + 2\nx".to_owned(), "test.py", vec![],
/// ).expect("parse failed");
///
/// let mediator: Arc<Mutex<dyn ExternalCallMediator>> =
///     Arc::new(Mutex::new(AllowAllMediator));
/// let governed = GovernedRunner::new(runner, mediator);
/// ```
pub struct GovernedRunner {
    /// The compiled Monty program to execute.
    monty_run: MontyRun,
    /// Shared mediator for external-call decisions.
    mediator: Arc<Mutex<dyn ExternalCallMediator>>,
}

impl GovernedRunner {
    /// Creates a new governed runner from a compiled program and mediator.
    #[must_use]
    pub fn new(monty_run: MontyRun, mediator: Arc<Mutex<dyn ExternalCallMediator>>) -> Self {
        Self {
            monty_run,
            mediator,
        }
    }

    /// Executes the program to completion with no resource limits, mediating
    /// all external calls and printing to stdout.
    ///
    /// This is the simplest governed entrypoint. It loops through execution
    /// yields, mediating each external call, and returns the final result or
    /// the first denial encountered.
    ///
    /// # Errors
    ///
    /// Returns `GovernedRunError` if the interpreter raises an exception or
    /// the mediator lock is poisoned.
    pub fn run_no_limits(
        self,
        inputs: Vec<MontyObject>,
    ) -> Result<GovernedRunProgress<NoLimitTracker>, GovernedRunError> {
        self.run(inputs, NoLimitTracker, PrintWriter::Stdout)
    }

    /// Executes the program with a custom resource tracker and print writer,
    /// mediating all external calls.
    ///
    /// # Errors
    ///
    /// Returns `GovernedRunError` if the interpreter raises an exception or
    /// the mediator lock is poisoned.
    pub fn run<T: ResourceTracker>(
        self,
        inputs: Vec<MontyObject>,
        resource_tracker: T,
        print: PrintWriter<'_>,
    ) -> Result<GovernedRunProgress<T>, GovernedRunError> {
        let mediator = self.mediator;
        let observer = ZamburakObserver::new(Arc::clone(&mediator));
        let handle = RuntimeObserverHandle::new(observer);

        let progress =
            self.monty_run
                .start_with_observer(inputs, resource_tracker, print, handle)?;

        step(progress, &mediator)
    }
}

/// Processes a single `RunProgress` step, mediating external calls.
fn step<T: ResourceTracker>(
    progress: RunProgress<T>,
    mediator: &Arc<Mutex<dyn ExternalCallMediator>>,
) -> Result<GovernedRunProgress<T>, GovernedRunError> {
    match progress {
        RunProgress::Complete(value) => Ok(GovernedRunProgress::Complete(value)),
        RunProgress::FunctionCall(call) => mediate_function_call(call, mediator),
        RunProgress::OsCall(call) => mediate_os_call(call, mediator),
        RunProgress::NameLookup(lookup) => Ok(GovernedRunProgress::NameLookup {
            name: lookup.name.clone(),
            inner: lookup,
        }),
        RunProgress::ResolveFutures(futures) => Ok(GovernedRunProgress::ResolveFutures(futures)),
    }
}

/// Mediates an external function call yield.
fn mediate_function_call<T: ResourceTracker>(
    call: monty::FunctionCall<T>,
    mediator: &Arc<Mutex<dyn ExternalCallMediator>>,
) -> Result<GovernedRunProgress<T>, GovernedRunError> {
    let context = CallContext {
        call_id: call.call_id,
        kind: if call.method_call {
            ExternalCallKind::Method
        } else {
            ExternalCallKind::Function
        },
        function_name: call.function_name.clone(),
    };

    let decision = query_mediator(mediator, &context)?;

    match decision {
        MediationDecision::Allow => {
            // The governed runner does not natively implement external
            // functions. When a call is allowed, it resumes with NotFound
            // so the interpreter raises NameError — the host is expected
            // to catch the FunctionCall yield and provide the actual
            // return value externally.
            let result = ExtFunctionResult::NotFound(call.function_name.clone());
            let next = call.resume(result, PrintWriter::Stdout)?;
            step(next, mediator)
        }
        MediationDecision::Deny { reason } => Ok(GovernedRunProgress::Denied {
            reason,
            function_name: context.function_name,
            call_id: context.call_id,
        }),
        MediationDecision::RequireConfirmation { request } => {
            Ok(GovernedRunProgress::AwaitConfirmation {
                context: request,
                suspended: SuspendedCall {
                    kind: SuspendedCallKind::Function(call),
                },
            })
        }
    }
}

/// Mediates an OS call yield.
fn mediate_os_call<T: ResourceTracker>(
    call: monty::OsCall<T>,
    mediator: &Arc<Mutex<dyn ExternalCallMediator>>,
) -> Result<GovernedRunProgress<T>, GovernedRunError> {
    let context = CallContext {
        call_id: call.call_id,
        kind: ExternalCallKind::Os,
        function_name: format!("{:?}", call.function),
    };

    let decision = query_mediator(mediator, &context)?;

    match decision {
        MediationDecision::Allow => {
            let result = ExtFunctionResult::NotFound(context.function_name);
            let next = call.resume(result, PrintWriter::Stdout)?;
            step(next, mediator)
        }
        MediationDecision::Deny { reason } => Ok(GovernedRunProgress::Denied {
            reason,
            function_name: context.function_name,
            call_id: context.call_id,
        }),
        MediationDecision::RequireConfirmation { request } => {
            Ok(GovernedRunProgress::AwaitConfirmation {
                context: request,
                suspended: SuspendedCall {
                    kind: SuspendedCallKind::Os(call),
                },
            })
        }
    }
}

/// Queries the mediator for a decision, handling mutex poisoning.
fn query_mediator(
    mediator: &Arc<Mutex<dyn ExternalCallMediator>>,
    context: &CallContext,
) -> Result<MediationDecision, GovernedRunError> {
    let mut guard = mediator
        .lock()
        .map_err(|_| GovernedRunError::MediatorPoisoned)?;
    Ok(guard.mediate(context))
}

#[cfg(test)]
#[path = "run_tests.rs"]
mod tests;
