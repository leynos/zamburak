//! Governed execution entrypoint wrapping `full-monty` with Zamburak mediation.
//!
//! [`GovernedRunner`] orchestrates a governed execution session: it installs a
//! [`ZamburakObserver`] on a compiled [`MontyRun`], mediates every external-call
//! yield through an [`ExternalCallMediator`], and exposes the result as a
//! [`GovernedRunProgress`] enum that enriches standard `RunProgress` states with
//! governance metadata.

use std::sync::{Arc, Mutex};

use monty::{
    ExternalCallKind, MontyException, MontyObject, MontyRun, NoLimitTracker, PrintWriter,
    ResourceTracker, RuntimeObserverHandle,
};
use thiserror::Error;

use crate::external_call::{CallContext, ConfirmationContext, ExternalCallMediator};
use crate::observer::{EventCounts, ZamburakObserver};

mod flow;

use flow::step;
pub use flow::{SuspendedCall, SuspendedNameLookup, SuspendedResolveFutures};

/// Errors arising from governed execution.
#[derive(Debug, Error)]
pub enum GovernedRunError {
    /// The underlying Monty interpreter raised an exception.
    #[error("interpreter exception: {0}")]
    Interpreter(MontyException),

    /// The mediator's mutex was poisoned.
    #[error("mediator lock poisoned")]
    MediatorPoisoned,

    /// Observer bookkeeping diverged from yielded external-call progress.
    #[error("observer mismatch for call_id {call_id} with kind {kind:?}")]
    ObserverMismatch {
        /// The yielded call identifier whose observer bookkeeping was missing.
        call_id: u32,
        /// The yielded external-call kind.
        kind: ExternalCallKind,
    },
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
        /// Suspended execution state resumable through governed mediation.
        suspended: SuspendedNameLookup<T>,
    },

    /// Execution paused waiting for async futures to resolve.
    ResolveFutures(SuspendedResolveFutures<T>),
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

    /// Executes the program with no resource limits and returns observer event
    /// counts captured by the governed path.
    ///
    /// # Errors
    ///
    /// Returns `GovernedRunError` if the interpreter raises an exception, the
    /// mediator lock is poisoned, or observer bookkeeping diverges from a
    /// yielded external call.
    pub fn run_no_limits_with_event_counts(
        self,
        inputs: Vec<MontyObject>,
    ) -> Result<(GovernedRunProgress<NoLimitTracker>, EventCounts), GovernedRunError> {
        self.run_with_event_counts(inputs, NoLimitTracker, PrintWriter::Stdout)
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
        let (progress, _) = self.run_with_event_counts(inputs, resource_tracker, print)?;
        Ok(progress)
    }

    fn run_with_event_counts<T: ResourceTracker>(
        self,
        inputs: Vec<MontyObject>,
        resource_tracker: T,
        print: PrintWriter<'_>,
    ) -> Result<(GovernedRunProgress<T>, EventCounts), GovernedRunError> {
        let mediator = self.mediator;
        let observer = ZamburakObserver::new();
        let observer_state = observer.shared_state();
        let handle = RuntimeObserverHandle::new(observer);
        let progress =
            self.monty_run
                .start_with_observer(inputs, resource_tracker, print, handle)?;
        let progress = step(progress, &mediator, &observer_state)?;
        let counts = observer_state.event_counts();
        Ok((progress, counts))
    }
}

#[cfg(test)]
#[path = "run_tests.rs"]
mod tests;
