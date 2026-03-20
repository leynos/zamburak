//! Governed execution entrypoint wrapping `full-monty` with Zamburak mediation.
//!
//! [`GovernedRunner`] orchestrates a governed execution session: it installs a
//! [`ZamburakObserver`] on a compiled [`MontyRun`], mediates every external-call
//! yield through an [`ExternalCallMediator`], and exposes the result as a
//! [`GovernedRunProgress`] enum that enriches standard `RunProgress` states with
//! governance metadata.

use std::sync::{Arc, Mutex};

use monty::{
    MontyException, MontyObject, MontyRun, NoLimitTracker, PrintWriter, ResourceTracker,
    RuntimeObserverHandle,
};
use thiserror::Error;

use crate::external_call::{CallContext, ConfirmationContext, ExternalCallMediator};
use crate::observer::ZamburakObserver;

mod flow;

pub use flow::SuspendedCall;
use flow::step;

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

    /// Execution paused so the host can provide an external-call result.
    ExternalCallPending {
        /// Mediation context for the paused call.
        context: CallContext,
        /// Suspended execution state resumable with a host-provided result.
        suspended: SuspendedCall<T>,
    },

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

    /// Execution paused for an unresolved name lookup.
    NameLookup {
        /// The name being looked up.
        name: String,
        /// The underlying Monty `NameLookup` state for resume.
        inner: monty::NameLookup<T>,
    },

    /// Execution paused waiting for async futures to resolve.
    ResolveFutures(monty::ResolveFutures<T>),
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
        let observer = ZamburakObserver::new();
        let observer_state = observer.shared_state();
        let handle = RuntimeObserverHandle::new(observer);
        let progress =
            self.monty_run
                .start_with_observer(inputs, resource_tracker, print, handle)?;
        step(progress, &mediator, &observer_state)
    }
}

#[cfg(test)]
#[path = "run_tests.rs"]
mod tests;
