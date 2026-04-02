//! Shared helper utilities for governed-run tests.

use std::sync::{Arc, Mutex};

use crate::{CallContext, ExternalCallMediator, MediationDecision};

/// Shared recording buffer used by the governed-run test mediators.
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

/// Returns a recording mediator paired with its shared captured-context buffer.
#[must_use]
pub fn recording_mediator() -> (Arc<Mutex<dyn ExternalCallMediator>>, RecordedContexts) {
    let contexts = Arc::new(Mutex::new(Vec::new()));
    (
        Arc::new(Mutex::new(RecordingMediator {
            contexts: Arc::clone(&contexts),
        })),
        contexts,
    )
}
