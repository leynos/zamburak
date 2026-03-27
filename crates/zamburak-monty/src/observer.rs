//! Zamburak runtime observer bridging Track A events into governance semantics.
//!
//! [`ZamburakObserver`] implements the `full-monty` [`RuntimeObserver`] trait and
//! translates Track A observer events into governed pending-call snapshots and
//! live IFC state for the run loop to inspect.

use std::sync::{Arc, Mutex};

use monty::{ExternalCallKind, RuntimeObserver, RuntimeObserverEvent};

mod ifc_state;

pub use ifc_state::{GovernedIfcConfig, IfcValueSeedConfig};

use crate::external_call::CallIfcContext;
use ifc_state::IfcRuntimeState;

/// Recorded metadata from an `ExternalCallRequested` observer event.
///
/// The governed run entrypoint consumes these records as bookkeeping while it
/// mediates external-call `RunProgress` yields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedCallRequest {
    /// Host-visible call identifier.
    pub call_id: u32,
    /// External-call class (function, OS, or method).
    pub kind: ExternalCallKind,
}

struct ObserverState {
    pending_calls: Vec<RecordedCallRequest>,
    event_counts: EventCounts,
    ifc_state: IfcRuntimeState,
}

/// Cloneable shared observer state used by the governed run loop.
#[derive(Clone)]
pub(crate) struct SharedObserverState {
    inner: Arc<Mutex<ObserverState>>,
}

/// Zamburak runtime observer bridging Track A events into Track B governance.
///
/// Implements [`RuntimeObserver`] from `full-monty` and records
/// `ExternalCallRequested` events for mediation by the governed runner.
///
/// # Examples
///
/// ```
/// use zamburak_monty::ZamburakObserver;
///
/// let observer = ZamburakObserver::new();
/// ```
pub struct ZamburakObserver {
    state: SharedObserverState,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallIfcLookupError {
    ObserverMismatch {
        call_id: u32,
        kind: ExternalCallKind,
    },
    MissingIfcSnapshot {
        call_id: u32,
        kind: ExternalCallKind,
    },
}

impl ZamburakObserver {
    /// Creates a new observer with empty state.
    #[must_use]
    pub fn new() -> Self {
        Self::with_ifc_config(GovernedIfcConfig::default())
    }

    /// Creates a new observer with explicit IFC configuration.
    #[must_use]
    pub fn with_ifc_config(config: GovernedIfcConfig) -> Self {
        Self {
            state: SharedObserverState::new(config),
        }
    }

    /// Returns a clone of the shared observer state.
    #[must_use]
    pub(crate) fn shared_state(&self) -> SharedObserverState {
        self.state.clone()
    }

    /// Returns recorded call requests that have not yet been consumed.
    #[must_use]
    pub fn pending_calls(&self) -> Vec<RecordedCallRequest> {
        self.state.pending_calls()
    }

    /// Drains and returns all pending call requests.
    pub fn take_pending_calls(&mut self) -> Vec<RecordedCallRequest> {
        self.state.take_pending_calls()
    }

    /// Returns diagnostic event counters accumulated during execution.
    #[must_use]
    pub fn event_counts(&self) -> EventCounts {
        self.state.event_counts()
    }
}

impl Default for ZamburakObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeObserver for ZamburakObserver {
    fn on_event(&mut self, event: RuntimeObserverEvent<'_>) {
        self.state.record_event(event);
    }
}

impl SharedObserverState {
    fn new(config: GovernedIfcConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ObserverState {
                pending_calls: Vec::new(),
                event_counts: EventCounts::default(),
                ifc_state: IfcRuntimeState::new(config),
            })),
        }
    }

    pub(crate) fn call_ifc_context(
        &self,
        call_id: u32,
        kind: ExternalCallKind,
        function_name: &str,
    ) -> Result<CallIfcContext, CallIfcLookupError> {
        let mut state = lock_state(&self.inner);
        let Some(index) = state
            .pending_calls
            .iter()
            .position(|call| call.call_id == call_id && call.kind == kind)
        else {
            return Err(CallIfcLookupError::ObserverMismatch { call_id, kind });
        };
        state.pending_calls.remove(index);
        state
            .ifc_state
            .call_ifc_context(call_id, kind, function_name)
            .ok_or(CallIfcLookupError::MissingIfcSnapshot { call_id, kind })
    }

    fn pending_calls(&self) -> Vec<RecordedCallRequest> {
        let state = lock_state(&self.inner);
        state.pending_calls.clone()
    }

    fn take_pending_calls(&self) -> Vec<RecordedCallRequest> {
        let mut state = lock_state(&self.inner);
        std::mem::take(&mut state.pending_calls)
    }

    pub(crate) fn event_counts(&self) -> EventCounts {
        let state = lock_state(&self.inner);
        state.event_counts.clone()
    }

    fn record_event(&self, event: RuntimeObserverEvent<'_>) {
        let mut state = lock_state(&self.inner);
        dispatch_event(&mut state, event);
    }
}

fn dispatch_event(state: &mut ObserverState, event: RuntimeObserverEvent<'_>) {
    match event {
        RuntimeObserverEvent::ValueCreated(event) => {
            state.event_counts.value_created += 1;
            state.ifc_state.apply_value_created(event);
        }
        RuntimeObserverEvent::OpResult(event) => {
            state.event_counts.op_result += 1;
            state.ifc_state.apply_op_result(event);
        }
        RuntimeObserverEvent::ExternalCallRequested(event) => {
            state.event_counts.external_call_requested += 1;
            state.pending_calls.push(RecordedCallRequest {
                call_id: event.call_id,
                kind: event.kind,
            });
            state.ifc_state.apply_external_call_requested(event);
        }
        RuntimeObserverEvent::ExternalCallReturned(event) => {
            state.event_counts.external_call_returned += 1;
            state.ifc_state.apply_external_call_returned(event);
        }
        RuntimeObserverEvent::ControlCondition(event) => {
            state.event_counts.control_condition += 1;
            state.ifc_state.apply_control_condition(event);
        }
    }
}

fn lock_state(state: &Arc<Mutex<ObserverState>>) -> std::sync::MutexGuard<'_, ObserverState> {
    match state.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
#[path = "observer_tests.rs"]
mod tests;
