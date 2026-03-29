//! Shared helpers for governed-run tests across workspace crates.

use std::sync::{Arc, Mutex};

use zamburak_monty::{CallContext, ExternalCallMediator, MediationDecision};

/// Shared recording buffer used by governed-run test mediators.
pub type RecordedContexts = Arc<Mutex<Vec<CallContext>>>;

/// Test mediator that records every observed call context and allows the call.
#[derive(Clone)]
pub struct RecordingMediator {
    /// Shared sink for cloned call contexts observed by the mediator.
    pub contexts: RecordedContexts,
}

impl ExternalCallMediator for RecordingMediator {
    fn mediate(&mut self, context: &CallContext) -> MediationDecision {
        self.contexts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(context.clone());
        MediationDecision::Allow
    }
}
