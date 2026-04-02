//! Basic bookkeeping tests for the [`ZamburakObserver`] bridge.

use monty::{
    ControlConditionEvent, ExternalCallKind, ExternalCallRequestedEvent, ExternalCallReturnKind,
    ExternalCallReturnedEvent, OpInputIds, OpResultEvent, RuntimeObserver, RuntimeObserverEvent,
    RuntimeValueId, ValueCreatedEvent,
};
use rstest::rstest;

use crate::observer::EventCounts;
use crate::observer_test_helpers::{allow_all_observer, observer_with_one_pending_call};

fn assert_event_counts_eq(actual: EventCounts, expected: EventCounts) {
    assert_eq!(actual, expected);
}

#[rstest]
fn new_observer_starts_with_empty_state() {
    let obs = allow_all_observer();
    assert!(obs.pending_calls().is_empty());
    assert_eq!(obs.event_counts(), EventCounts::default());
}

#[rstest]
#[case(
    RuntimeObserverEvent::ValueCreated(ValueCreatedEvent {
        value_id: RuntimeValueId::new(1),
    }),
    EventCounts {
        value_created: 1,
        ..EventCounts::default()
    }
)]
#[case(
    RuntimeObserverEvent::OpResult(OpResultEvent {
        output_id: RuntimeValueId::new(2),
        inputs: OpInputIds::None,
    }),
    EventCounts {
        op_result: 1,
        ..EventCounts::default()
    }
)]
#[case(
    RuntimeObserverEvent::ExternalCallReturned(ExternalCallReturnedEvent {
        call_id: 1,
        kind: ExternalCallReturnKind::Return,
    }),
    EventCounts {
        external_call_returned: 1,
        ..EventCounts::default()
    }
)]
#[case(
    RuntimeObserverEvent::ControlCondition(ControlConditionEvent {
        condition_id: RuntimeValueId::new(10),
        branch_taken: true,
    }),
    EventCounts {
        control_condition: 1,
        ..EventCounts::default()
    }
)]
fn event_increments_correct_counter(
    #[case] event: RuntimeObserverEvent<'static>,
    #[case] expected_counts: EventCounts,
) {
    let mut obs = allow_all_observer();
    obs.on_event(event);
    assert_event_counts_eq(obs.event_counts(), expected_counts);
    assert!(obs.pending_calls().is_empty());
}

#[rstest]
fn external_call_requested_records_pending_call() {
    let obs = observer_with_one_pending_call(42, ExternalCallKind::Function);
    assert_eq!(obs.event_counts().external_call_requested, 1);
    let pending_calls = obs.pending_calls();
    assert_eq!(pending_calls.len(), 1);
    let recorded = &pending_calls[0];
    assert_eq!(recorded.call_id, 42);
    assert_eq!(recorded.kind, ExternalCallKind::Function);
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

    assert_event_counts_eq(
        obs.event_counts(),
        EventCounts {
            value_created: 2,
            op_result: 1,
            external_call_requested: 1,
            external_call_returned: 1,
            control_condition: 1,
        },
    );
}
