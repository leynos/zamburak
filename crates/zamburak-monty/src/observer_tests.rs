//! Unit tests for the [`ZamburakObserver`] bridge.

use monty::{
    ControlConditionEvent, ExternalCallKind, ExternalCallRequestedEvent, ExternalCallReturnKind,
    ExternalCallReturnedEvent, OpInputIds, OpResultEvent, RuntimeObserver, RuntimeObserverEvent,
    RuntimeValueId, ValueCreatedEvent,
};
use rstest::rstest;
use zamburak_core::IntegrityLabel;

use crate::observer::{CallIfcLookupError, EventCounts, GovernedIfcConfig, ZamburakObserver};

/// Helper: build a `ZamburakObserver`.
fn allow_all_observer() -> ZamburakObserver {
    ZamburakObserver::new()
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
    assert_eq!(obs.event_counts(), EventCounts::default());
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
    let pending_calls = obs.pending_calls();
    assert_eq!(pending_calls.len(), 1);
    let recorded = &pending_calls[0];
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

#[rstest]
fn op_result_dependencies_flow_into_external_call_ifc_context() {
    let mut obs = allow_all_observer();
    let arg_ids = vec![RuntimeValueId::new(3)];
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
            call_id: 9,
            kind: ExternalCallKind::Function,
            arg_runtime_ids: &arg_ids,
            kwarg_runtime_ids: &kwarg_ids,
        },
    ));

    let ifc = obs
        .shared_state()
        .call_ifc_context(9, ExternalCallKind::Function, "effect")
        .expect("IFC context should exist");
    assert_eq!(ifc.arg_summaries.len(), 1);
    assert_eq!(ifc.arg_summaries[0].origin_count, 3);
    assert_eq!(ifc.aggregate_summary.origin_count, 3);
    assert_eq!(
        ifc.aggregate_summary.integrity_join,
        IntegrityLabel::Trusted
    );
}

#[rstest]
fn kwarg_dependencies_flow_into_external_call_ifc_context() {
    let mut obs = allow_all_observer();
    let arg_ids = vec![RuntimeValueId::new(3)];
    let kwarg_ids = vec![
        (RuntimeValueId::new(10), RuntimeValueId::new(3)),
        (RuntimeValueId::new(11), RuntimeValueId::new(4)),
    ];

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
    obs.on_event(RuntimeObserverEvent::ValueCreated(ValueCreatedEvent {
        value_id: RuntimeValueId::new(4),
    }));
    obs.on_event(RuntimeObserverEvent::ExternalCallRequested(
        ExternalCallRequestedEvent {
            call_id: 10,
            kind: ExternalCallKind::Function,
            arg_runtime_ids: &arg_ids,
            kwarg_runtime_ids: &kwarg_ids,
        },
    ));

    let ifc = obs
        .shared_state()
        .call_ifc_context(10, ExternalCallKind::Function, "effect")
        .expect("IFC context with kwargs should exist");
    assert_eq!(ifc.arg_summaries.len(), 1);
    assert_eq!(ifc.kwarg_summaries.len(), 2);
    assert_eq!(ifc.kwarg_summaries[0].0.origin_count, 1);
    assert_eq!(ifc.kwarg_summaries[0].1.origin_count, 3);
    assert_eq!(ifc.kwarg_summaries[1].0.origin_count, 1);
    assert_eq!(ifc.kwarg_summaries[1].1.origin_count, 1);
    assert_eq!(ifc.aggregate_summary.origin_count, 9);
    assert_eq!(
        ifc.aggregate_summary.integrity_join,
        IntegrityLabel::Trusted
    );
}

#[rstest]
fn strict_mode_joins_control_context_into_aggregate_summary() {
    let mut obs =
        ZamburakObserver::with_ifc_config(GovernedIfcConfig::strict_with_boundary_seeds());
    let empty_args: Vec<RuntimeValueId> = vec![];
    let effect_args = vec![RuntimeValueId::new(300)];
    let kwarg_ids: Vec<(RuntimeValueId, RuntimeValueId)> = vec![];

    obs.on_event(RuntimeObserverEvent::ExternalCallRequested(
        ExternalCallRequestedEvent {
            call_id: 1,
            kind: ExternalCallKind::Function,
            arg_runtime_ids: &empty_args,
            kwarg_runtime_ids: &kwarg_ids,
        },
    ));
    obs.on_event(RuntimeObserverEvent::ExternalCallReturned(
        ExternalCallReturnedEvent {
            call_id: 1,
            kind: ExternalCallReturnKind::Return,
        },
    ));
    obs.on_event(RuntimeObserverEvent::OpResult(OpResultEvent {
        output_id: RuntimeValueId::new(200),
        inputs: OpInputIds::None,
    }));
    obs.on_event(RuntimeObserverEvent::ControlCondition(
        ControlConditionEvent {
            condition_id: RuntimeValueId::new(200),
            branch_taken: true,
        },
    ));
    obs.on_event(RuntimeObserverEvent::ValueCreated(ValueCreatedEvent {
        value_id: RuntimeValueId::new(300),
    }));
    obs.on_event(RuntimeObserverEvent::ExternalCallRequested(
        ExternalCallRequestedEvent {
            call_id: 2,
            kind: ExternalCallKind::Function,
            arg_runtime_ids: &effect_args,
            kwarg_runtime_ids: &kwarg_ids,
        },
    ));

    let ifc = obs
        .shared_state()
        .call_ifc_context(2, ExternalCallKind::Function, "effect")
        .expect("strict IFC context should exist");
    assert_eq!(
        ifc.aggregate_summary.integrity_join,
        IntegrityLabel::Untrusted
    );
    assert_eq!(
        ifc.control_context.pc_integrity(),
        IntegrityLabel::Untrusted
    );
}

#[rstest]
fn returned_call_provenance_flows_into_next_effect_argument() {
    let mut obs =
        ZamburakObserver::with_ifc_config(GovernedIfcConfig::strict_with_boundary_seeds());
    let source_args = vec![RuntimeValueId::new(10)];
    let returned_args = vec![RuntimeValueId::new(20)];
    let kwarg_ids: Vec<(RuntimeValueId, RuntimeValueId)> = vec![];

    obs.on_event(RuntimeObserverEvent::ValueCreated(ValueCreatedEvent {
        value_id: RuntimeValueId::new(10),
    }));
    obs.on_event(RuntimeObserverEvent::ExternalCallRequested(
        ExternalCallRequestedEvent {
            call_id: 7,
            kind: ExternalCallKind::Function,
            arg_runtime_ids: &source_args,
            kwarg_runtime_ids: &kwarg_ids,
        },
    ));
    obs.on_event(RuntimeObserverEvent::ExternalCallReturned(
        ExternalCallReturnedEvent {
            call_id: 7,
            kind: ExternalCallReturnKind::Return,
        },
    ));
    obs.on_event(RuntimeObserverEvent::OpResult(OpResultEvent {
        output_id: RuntimeValueId::new(20),
        inputs: OpInputIds::None,
    }));
    obs.on_event(RuntimeObserverEvent::ExternalCallRequested(
        ExternalCallRequestedEvent {
            call_id: 8,
            kind: ExternalCallKind::Function,
            arg_runtime_ids: &returned_args,
            kwarg_runtime_ids: &kwarg_ids,
        },
    ));

    let ifc = obs
        .shared_state()
        .call_ifc_context(8, ExternalCallKind::Function, "sink")
        .expect("returned provenance should be available");
    assert_eq!(
        ifc.arg_summaries[0].integrity_join,
        IntegrityLabel::Untrusted
    );
    assert_eq!(ifc.arg_summaries[0].origin_count, 2);
}

#[rstest]
fn missing_pending_call_bookkeeping_is_reported_before_ifc_lookup() {
    let mut obs = observer_with_one_pending_call(11, ExternalCallKind::Function);
    let _ = obs.take_pending_calls();

    let error = obs
        .shared_state()
        .call_ifc_context(11, ExternalCallKind::Function, "effect")
        .expect_err("pending queue drift should surface as an error");
    assert_eq!(
        error,
        CallIfcLookupError::ObserverMismatch {
            call_id: 11,
            kind: ExternalCallKind::Function,
        }
    );
}

#[rstest]
fn missing_ifc_snapshot_is_reported_when_runtime_state_has_already_dropped_call() {
    let mut obs = observer_with_one_pending_call(12, ExternalCallKind::Function);
    obs.on_event(RuntimeObserverEvent::ExternalCallReturned(
        ExternalCallReturnedEvent {
            call_id: 12,
            kind: ExternalCallReturnKind::Error,
        },
    ));

    let error = obs
        .shared_state()
        .call_ifc_context(12, ExternalCallKind::Function, "effect")
        .expect_err("missing IFC snapshot should surface as an error");
    assert_eq!(
        error,
        CallIfcLookupError::MissingIfcSnapshot {
            call_id: 12,
            kind: ExternalCallKind::Function,
        }
    );
}
