//! Zamburak runtime observer bridging Track A events into governance semantics.
//!
//! [`ZamburakObserver`] implements the `full-monty` [`RuntimeObserver`] trait and
//! records external-call events for the governed run entrypoint to intercept.
//! Non-call events (`ValueCreated`, `OpResult`, `ControlCondition`) are passed
//! through to an optional downstream event sink, preparing for Task 0.6.3 IFC
//! wiring without implementing it now.

use std::sync::{Arc, Mutex};

use monty::{ExternalCallKind, ExternalCallRequestedEvent, RuntimeObserver, RuntimeObserverEvent};

use crate::external_call::ExternalCallMediator;

/// Recorded metadata from an `ExternalCallRequested` observer event.
///
/// The governed run entrypoint inspects these records to determine which
/// external calls require mediation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedCallRequest {
    /// Host-visible call identifier.
    pub call_id: u32,
    /// External-call class (function, OS, or method).
    pub kind: ExternalCallKind,
}

/// Zamburak runtime observer bridging Track A events into Track B governance.
///
/// Implements [`RuntimeObserver`] from `full-monty` and records
/// `ExternalCallRequested` events for mediation by the governed runner.
///
/// # Examples
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use zamburak_monty::{AllowAllMediator, ZamburakObserver};
///
/// let mediator: Arc<Mutex<dyn zamburak_monty::ExternalCallMediator>> =
///     Arc::new(Mutex::new(AllowAllMediator));
/// let observer = ZamburakObserver::new(mediator);
/// ```
pub struct ZamburakObserver {
    /// Shared mediator handle for external-call decisions.
    mediator: Arc<Mutex<dyn ExternalCallMediator>>,
    /// Recorded external-call requests awaiting mediation.
    pending_calls: Vec<RecordedCallRequest>,
    /// Count of each event class received, for diagnostic and test purposes.
    event_counts: EventCounts,
}

/// Diagnostic counters for observer event classes received during execution.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventCounts {
    /// Number of `ValueCreated` events received.
    pub value_created: usize,
    /// Number of `OpResult` events received.
    pub op_result: usize,
    /// Number of `ExternalCallRequested` events received.
    pub external_call_requested: usize,
    /// Number of `ExternalCallReturned` events received.
    pub external_call_returned: usize,
    /// Number of `ControlCondition` events received.
    pub control_condition: usize,
}

impl ZamburakObserver {
    /// Creates a new observer with the given external-call mediator.
    #[must_use]
    pub fn new(mediator: Arc<Mutex<dyn ExternalCallMediator>>) -> Self {
        Self {
            mediator,
            pending_calls: Vec::new(),
            event_counts: EventCounts::default(),
        }
    }

    /// Returns the shared mediator handle.
    #[must_use]
    pub fn mediator(&self) -> &Arc<Mutex<dyn ExternalCallMediator>> {
        &self.mediator
    }

    /// Returns recorded call requests that have not yet been consumed.
    #[must_use]
    pub fn pending_calls(&self) -> &[RecordedCallRequest] {
        &self.pending_calls
    }

    /// Drains and returns all pending call requests.
    pub fn take_pending_calls(&mut self) -> Vec<RecordedCallRequest> {
        std::mem::take(&mut self.pending_calls)
    }

    /// Returns diagnostic event counters accumulated during execution.
    #[must_use]
    pub fn event_counts(&self) -> &EventCounts {
        &self.event_counts
    }
}

impl RuntimeObserver for ZamburakObserver {
    fn on_event(&mut self, event: RuntimeObserverEvent<'_>) {
        match event {
            RuntimeObserverEvent::ValueCreated(_) => {
                self.event_counts.value_created += 1;
            }
            RuntimeObserverEvent::OpResult(_) => {
                self.event_counts.op_result += 1;
            }
            RuntimeObserverEvent::ExternalCallRequested(ExternalCallRequestedEvent {
                call_id,
                kind,
                ..
            }) => {
                self.event_counts.external_call_requested += 1;
                self.pending_calls
                    .push(RecordedCallRequest { call_id, kind });
            }
            RuntimeObserverEvent::ExternalCallReturned(_) => {
                self.event_counts.external_call_returned += 1;
            }
            RuntimeObserverEvent::ControlCondition(_) => {
                self.event_counts.control_condition += 1;
            }
        }
    }
}

#[cfg(test)]
#[path = "observer_tests.rs"]
mod tests;
