//! Unit tests for the [`ZamburakObserver`] bridge.

use monty::{
    ControlConditionEvent, ExternalCallKind, ExternalCallRequestedEvent, ExternalCallReturnKind,
    ExternalCallReturnedEvent, OpInputIds, OpResultEvent, RuntimeObserver, RuntimeObserverEvent,
    RuntimeValueId, ValueCreatedEvent,
};
use rstest::rstest;
use zamburak_core::IntegrityLabel;

use crate::CallIfcContext;
use crate::observer::{CallIfcLookupError, ZamburakObserver};
use crate::observer_test_helpers::{
    allow_all_observer, observer_with_one_pending_call, strict_ifc_observer,
};

fn assert_single_arg_ifc(
    ifc: &CallIfcContext,
    expected_arg_origin: u32,
    expected_agg_origin: u32,
    expected_integrity: IntegrityLabel,
) {
    assert_eq!(
        ifc.arg_summaries.len(),
        1,
        "expected exactly one arg summary"
    );
    assert_eq!(ifc.arg_summaries[0].origin_count, expected_arg_origin);
    assert_eq!(ifc.aggregate_summary.origin_count, expected_agg_origin);
    assert_eq!(ifc.aggregate_summary.integrity_join, expected_integrity);
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
    assert_single_arg_ifc(&ifc, 3, 3, IntegrityLabel::Trusted);
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
    assert_single_arg_ifc(&ifc, 3, 9, IntegrityLabel::Trusted);
    assert_eq!(ifc.kwarg_summaries.len(), 2);
    assert_eq!(ifc.kwarg_summaries[0].0.origin_count, 1);
    assert_eq!(ifc.kwarg_summaries[0].1.origin_count, 3);
    assert_eq!(ifc.kwarg_summaries[1].0.origin_count, 1);
    assert_eq!(ifc.kwarg_summaries[1].1.origin_count, 1);
}

#[rstest]
fn strict_mode_joins_control_context_into_aggregate_summary() {
    let mut obs = strict_ifc_observer();
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

fn build_observer_after_source_call_returns() -> ZamburakObserver {
    let mut obs = strict_ifc_observer();
    let source_args = vec![RuntimeValueId::new(10)];
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
    obs
}

#[rstest]
fn returned_call_provenance_flows_into_next_effect_argument() {
    let mut obs = build_observer_after_source_call_returns();
    let returned_args = vec![RuntimeValueId::new(20)];
    let kwarg_ids: Vec<(RuntimeValueId, RuntimeValueId)> = vec![];
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
fn zero_input_internal_op_does_not_consume_returned_call_provenance() {
    let mut obs = build_observer_after_source_call_returns();
    let internal_args = vec![RuntimeValueId::new(99)];
    let returned_args = vec![RuntimeValueId::new(20)];
    let kwarg_ids: Vec<(RuntimeValueId, RuntimeValueId)> = vec![];

    obs.on_event(RuntimeObserverEvent::ValueCreated(ValueCreatedEvent {
        value_id: RuntimeValueId::new(99),
    }));
    obs.on_event(RuntimeObserverEvent::OpResult(OpResultEvent {
        output_id: RuntimeValueId::new(99),
        inputs: OpInputIds::None,
    }));
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
    obs.on_event(RuntimeObserverEvent::ExternalCallRequested(
        ExternalCallRequestedEvent {
            call_id: 9,
            kind: ExternalCallKind::Function,
            arg_runtime_ids: &internal_args,
            kwarg_runtime_ids: &kwarg_ids,
        },
    ));

    let sink_ifc = obs
        .shared_state()
        .call_ifc_context(8, ExternalCallKind::Function, "sink")
        .expect("returned provenance should remain available");
    let internal_ifc = obs
        .shared_state()
        .call_ifc_context(9, ExternalCallKind::Function, "other")
        .expect("internal zero-input output should remain internal");
    assert_eq!(
        sink_ifc.arg_summaries[0].integrity_join,
        IntegrityLabel::Untrusted
    );
    assert_eq!(sink_ifc.arg_summaries[0].origin_count, 2);
    assert_eq!(
        internal_ifc.arg_summaries[0].integrity_join,
        IntegrityLabel::Trusted
    );
    assert_eq!(internal_ifc.arg_summaries[0].origin_count, 1);
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
