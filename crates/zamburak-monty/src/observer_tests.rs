//! Unit tests for the [`ZamburakObserver`] bridge.

use std::sync::{Arc, Mutex};

use monty::{
    ControlConditionEvent, ExternalCallKind, ExternalCallRequestedEvent, ExternalCallReturnKind,
    ExternalCallReturnedEvent, OpInputIds, OpResultEvent, RuntimeObserver, RuntimeObserverEvent,
    RuntimeValueId, ValueCreatedEvent,
};
use rstest::rstest;

use crate::external_call::{AllowAllMediator, ExternalCallMediator};
use crate::observer::{EventCounts, ZamburakObserver};

/// Helper: build a `ZamburakObserver` with an `AllowAllMediator`.
fn allow_all_observer() -> ZamburakObserver {
    let mediator: Arc<Mutex<dyn ExternalCallMediator>> = Arc::new(Mutex::new(AllowAllMediator));
    ZamburakObserver::new(mediator)
}

/// Helper: build a `ZamburakObserver` that has already recorded one pending
/// external-call event with the given `call_id` and `kind`.
fn observer_with_one_pending_call(call_id: u64, kind: ExternalCallKind) -> ZamburakObserver {
    let mut obs = allow_all_observer();
    let arg_ids: Vec<RuntimeValueId> = vec![];
    let kwarg_ids: Vec<(RuntimeValueId, RuntimeValueId)> = vec![];
    obs.on_event(RuntimeObserverEvent::ExternalCallRequested(
        ExternalCallRequestedEvent {
            call_id: u32::try_from(call_id).expect("helper call_id should fit in u32"),
            kind,
            arg_runtime_ids: &arg_ids,
            kwarg_runtime_ids: &kwarg_ids,
        },
    ));
    obs
}

#[rstest]
fn new_observer_starts_with_empty_state() {
    let obs = allow_all_observer();
    assert!(obs.pending_calls().is_empty());
    assert_eq!(*obs.event_counts(), EventCounts::default());
}

#[rstest]
fn value_created_event_increments_counter() {
    let mut obs = allow_all_observer();
    obs.on_event(RuntimeObserverEvent::ValueCreated(ValueCreatedEvent {
        value_id: RuntimeValueId::new(1),
    }));
    assert_eq!(obs.event_counts().value_created, 1);
    assert!(obs.pending_calls().is_empty());
}

#[rstest]
fn op_result_event_increments_counter() {
    let mut obs = allow_all_observer();
    obs.on_event(RuntimeObserverEvent::OpResult(OpResultEvent {
        output_id: RuntimeValueId::new(2),
        inputs: OpInputIds::None,
    }));
    assert_eq!(obs.event_counts().op_result, 1);
}

#[rstest]
fn external_call_requested_records_pending_call() {
    let obs = observer_with_one_pending_call(42, ExternalCallKind::Function);
    assert_eq!(obs.event_counts().external_call_requested, 1);
    assert_eq!(obs.pending_calls().len(), 1);

    let recorded = &obs.pending_calls()[0];
    assert_eq!(recorded.call_id, 42);
    assert_eq!(recorded.kind, ExternalCallKind::Function);
}

#[rstest]
fn external_call_returned_increments_counter() {
    let mut obs = allow_all_observer();
    obs.on_event(RuntimeObserverEvent::ExternalCallReturned(
        ExternalCallReturnedEvent {
            call_id: 1,
            kind: ExternalCallReturnKind::Return,
        },
    ));
    assert_eq!(obs.event_counts().external_call_returned, 1);
}

#[rstest]
fn control_condition_increments_counter() {
    let mut obs = allow_all_observer();
    obs.on_event(RuntimeObserverEvent::ControlCondition(
        ControlConditionEvent {
            condition_id: RuntimeValueId::new(10),
            branch_taken: true,
        },
    ));
    assert_eq!(obs.event_counts().control_condition, 1);
}

#[rstest]
fn take_pending_calls_drains_list() {
    let mut obs = observer_with_one_pending_call(1, ExternalCallKind::Os);
    assert_eq!(obs.pending_calls().len(), 1);
    let taken = obs.take_pending_calls();
    assert_eq!(taken.len(), 1);
    assert!(obs.pending_calls().is_empty());
}

#[rstest]
fn multiple_events_accumulate_correctly() {
    let mut obs = allow_all_observer();
    let arg_ids: Vec<RuntimeValueId> = vec![];
    let kwarg_ids: Vec<(RuntimeValueId, RuntimeValueId)> = vec![];

    obs.on_event(RuntimeObserverEvent::ValueCreated(ValueCreatedEvent {
        value_id: RuntimeValueId::new(1),
    }));
    obs.on_event(RuntimeObserverEvent::ValueCreated(ValueCreatedEvent {
        value_id: RuntimeValueId::new(2),
    }));
    obs.on_event(RuntimeObserverEvent::OpResult(OpResultEvent {
        output_id: RuntimeValueId::new(3),
        inputs: OpInputIds::Two(RuntimeValueId::new(1), RuntimeValueId::new(2)),
    }));
    obs.on_event(RuntimeObserverEvent::ExternalCallRequested(
        ExternalCallRequestedEvent {
            call_id: 0,
            kind: ExternalCallKind::Function,
            arg_runtime_ids: &arg_ids,
            kwarg_runtime_ids: &kwarg_ids,
        },
    ));
    obs.on_event(RuntimeObserverEvent::ExternalCallReturned(
        ExternalCallReturnedEvent {
            call_id: 0,
            kind: ExternalCallReturnKind::Return,
        },
    ));
    obs.on_event(RuntimeObserverEvent::ControlCondition(
        ControlConditionEvent {
            condition_id: RuntimeValueId::new(4),
            branch_taken: false,
        },
    ));

    let counts = obs.event_counts();
    assert_eq!(counts.value_created, 2);
    assert_eq!(counts.op_result, 1);
    assert_eq!(counts.external_call_requested, 1);
    assert_eq!(counts.external_call_returned, 1);
    assert_eq!(counts.control_condition, 1);
}
