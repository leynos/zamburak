use rstest::{fixture, rstest};

use super::{
    CallId, ExecutionId, LlmCallPath, SinkPreDispatchDecision, SinkPreDispatchRequest,
    TransportGuardCheck, TransportGuardOutcome, emit_audit_record, evaluate_pre_dispatch,
    evaluate_transport_guard,
};

#[fixture]
fn planner_request() -> SinkPreDispatchRequest {
    SinkPreDispatchRequest {
        execution_id: ExecutionId::new("exec_test"),
        call_id: CallId::new("call_test"),
        call_path: LlmCallPath::Planner,
        redaction_applied: true,
    }
}

#[fixture]
fn transport_guard_check() -> TransportGuardCheck {
    TransportGuardCheck {
        execution_id: ExecutionId::new("exec_test"),
        call_id: CallId::new("call_test"),
        redaction_applied: true,
    }
}

#[rstest]
#[case::allows_with_redaction(true, SinkPreDispatchDecision::Allow)]
#[case::denies_without_redaction(false, SinkPreDispatchDecision::Deny)]
fn pre_dispatch_redaction_decides(
    planner_request: SinkPreDispatchRequest,
    #[case] redaction_applied: bool,
    #[case] expected: SinkPreDispatchDecision,
) {
    let request = SinkPreDispatchRequest {
        redaction_applied,
        ..planner_request
    };
    assert_eq!(evaluate_pre_dispatch(&request), expected);
}

#[rstest]
#[case::passes_with_redaction(true, TransportGuardOutcome::Passed)]
#[case::blocks_without_redaction(false, TransportGuardOutcome::Blocked)]
fn transport_guard_redaction_decides(
    transport_guard_check: TransportGuardCheck,
    #[case] redaction_applied: bool,
    #[case] expected: TransportGuardOutcome,
) {
    let check = TransportGuardCheck {
        redaction_applied,
        ..transport_guard_check
    };
    assert_eq!(evaluate_transport_guard(&check), expected);
}

#[rstest]
fn audit_record_preserves_linkage_fields() {
    let request = SinkPreDispatchRequest {
        execution_id: ExecutionId::new("exec_7f2c"),
        call_id: CallId::new("call_0192"),
        call_path: LlmCallPath::Planner,
        redaction_applied: true,
    };
    let decision = evaluate_pre_dispatch(&request);
    let audit = emit_audit_record(&request, decision);

    assert_eq!(audit.execution_id, ExecutionId::new("exec_7f2c"));
    assert_eq!(audit.call_id, CallId::new("call_0192"));
    assert_eq!(audit.decision, SinkPreDispatchDecision::Allow);
    assert_eq!(audit.call_path, LlmCallPath::Planner);
    assert!(audit.redaction_applied);
}

#[rstest]
fn quarantined_path_discrimination() {
    let request = SinkPreDispatchRequest {
        execution_id: ExecutionId::new("exec_q"),
        call_id: CallId::new("call_q"),
        call_path: LlmCallPath::Quarantined,
        redaction_applied: true,
    };
    let decision = evaluate_pre_dispatch(&request);
    let audit = emit_audit_record(&request, decision);

    assert_eq!(audit.call_path, LlmCallPath::Quarantined);
}
