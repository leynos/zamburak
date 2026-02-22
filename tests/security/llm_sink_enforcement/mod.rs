//! Behavioural tests validating LLM sink enforcement contract conformance.

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use zamburak_policy::sink_enforcement::{
    CallId, ExecutionId, LlmCallPath, SinkAuditRecord, SinkPreDispatchDecision,
    SinkPreDispatchRequest, TransportGuardCheck, TransportGuardOutcome, emit_audit_record,
    evaluate_pre_dispatch, evaluate_transport_guard,
};

#[derive(Default)]
struct SinkEnforcementWorld {
    request: Option<SinkPreDispatchRequest>,
    pre_dispatch_decision: Option<SinkPreDispatchDecision>,
    transport_check: Option<TransportGuardCheck>,
    transport_outcome: Option<TransportGuardOutcome>,
    audit_record: Option<SinkAuditRecord>,
}

#[fixture]
fn world() -> SinkEnforcementWorld {
    SinkEnforcementWorld::default()
}

// ── Given steps ─────────────────────────────────────────────────────

#[given("a planner LLM sink call request with redaction applied")]
fn planner_request_with_redaction(world: &mut SinkEnforcementWorld) {
    world.request = Some(SinkPreDispatchRequest {
        execution_id: ExecutionId::new("exec_default"),
        call_id: CallId::new("call_default"),
        call_path: LlmCallPath::Planner,
        redaction_applied: true,
    });
}

#[given("a planner LLM sink call request without redaction applied")]
fn planner_request_without_redaction(world: &mut SinkEnforcementWorld) {
    world.request = Some(SinkPreDispatchRequest {
        execution_id: ExecutionId::new("exec_default"),
        call_id: CallId::new("call_default"),
        call_path: LlmCallPath::Planner,
        redaction_applied: false,
    });
}

#[given("a planner LLM sink call request with execution id {exec_id} and call id {call_id}")]
fn planner_request_with_ids(world: &mut SinkEnforcementWorld, exec_id: String, call_id: String) {
    world.request = Some(SinkPreDispatchRequest {
        execution_id: ExecutionId::new(exec_id.trim_matches('"')),
        call_id: CallId::new(call_id.trim_matches('"')),
        call_path: LlmCallPath::Planner,
        redaction_applied: false,
    });
}

#[given("a quarantined LLM sink call request with execution id {exec_id} and call id {call_id}")]
fn quarantined_request_with_ids(
    world: &mut SinkEnforcementWorld,
    exec_id: String,
    call_id: String,
) {
    world.request = Some(SinkPreDispatchRequest {
        execution_id: ExecutionId::new(exec_id.trim_matches('"')),
        call_id: CallId::new(call_id.trim_matches('"')),
        call_path: LlmCallPath::Quarantined,
        redaction_applied: false,
    });
}

fn build_transport_check(redaction_applied: bool) -> TransportGuardCheck {
    TransportGuardCheck {
        execution_id: ExecutionId::new("exec_default"),
        call_id: CallId::new("call_default"),
        redaction_applied,
    }
}

#[given("a transport guard check with redaction applied")]
fn transport_check_with_redaction(world: &mut SinkEnforcementWorld) {
    world.transport_check = Some(build_transport_check(true));
}

#[given("a transport guard check without redaction applied")]
fn transport_check_without_redaction(world: &mut SinkEnforcementWorld) {
    world.transport_check = Some(build_transport_check(false));
}

#[given("redaction is applied")]
fn set_redaction_applied(world: &mut SinkEnforcementWorld) {
    if let Some(request) = world.request.as_mut() {
        request.redaction_applied = true;
    }
}

// ── When steps ──────────────────────────────────────────────────────

#[when("the pre-dispatch policy check is evaluated")]
fn when_pre_dispatch_evaluated(world: &mut SinkEnforcementWorld) {
    let Some(request) = world.request.as_ref() else {
        panic!("request must be set before pre-dispatch evaluation");
    };
    world.pre_dispatch_decision = Some(evaluate_pre_dispatch(request));
}

#[when("the transport guard is evaluated")]
fn when_transport_guard_evaluated(world: &mut SinkEnforcementWorld) {
    let Some(check) = world.transport_check.as_ref() else {
        panic!("transport check must be set before evaluation");
    };
    world.transport_outcome = Some(evaluate_transport_guard(check));
}

#[when("an audit record is emitted")]
fn when_audit_record_emitted(world: &mut SinkEnforcementWorld) {
    let Some(request) = world.request.as_ref() else {
        panic!("request must be set before emitting audit record");
    };
    let Some(decision) = world.pre_dispatch_decision else {
        panic!("pre-dispatch decision must exist before emitting audit");
    };
    world.audit_record = Some(emit_audit_record(request, decision));
}

// ── Then steps ──────────────────────────────────────────────────────

#[then("the pre-dispatch decision is Allow")]
fn then_decision_allow(world: &SinkEnforcementWorld) {
    let Some(decision) = world.pre_dispatch_decision else {
        panic!("pre-dispatch evaluation must run before assertion");
    };
    assert_eq!(decision, SinkPreDispatchDecision::Allow);
}

#[then("the pre-dispatch decision is Deny")]
fn then_decision_deny(world: &SinkEnforcementWorld) {
    let Some(decision) = world.pre_dispatch_decision else {
        panic!("pre-dispatch evaluation must run before assertion");
    };
    assert_eq!(decision, SinkPreDispatchDecision::Deny);
}

#[then("the transport guard outcome is Passed")]
fn then_transport_passed(world: &SinkEnforcementWorld) {
    let Some(outcome) = world.transport_outcome else {
        panic!("transport guard must be evaluated before assertion");
    };
    assert_eq!(outcome, TransportGuardOutcome::Passed);
}

#[then("the transport guard outcome is Blocked")]
fn then_transport_blocked(world: &SinkEnforcementWorld) {
    let Some(outcome) = world.transport_outcome else {
        panic!("transport guard must be evaluated before assertion");
    };
    assert_eq!(outcome, TransportGuardOutcome::Blocked);
}

#[then("the audit record execution id is {expected}")]
fn then_audit_execution_id(world: &SinkEnforcementWorld, expected: String) {
    let Some(record) = world.audit_record.as_ref() else {
        panic!("audit record must be emitted before assertion");
    };
    assert_eq!(record.execution_id.as_str(), expected.trim_matches('"'));
}

#[then("the audit record call id is {expected}")]
fn then_audit_call_id(world: &SinkEnforcementWorld, expected: String) {
    let Some(record) = world.audit_record.as_ref() else {
        panic!("audit record must be emitted before assertion");
    };
    assert_eq!(record.call_id.as_str(), expected.trim_matches('"'));
}

#[then("the audit record decision is Allow")]
fn then_audit_decision_allow(world: &SinkEnforcementWorld) {
    let Some(record) = world.audit_record.as_ref() else {
        panic!("audit record must be emitted before assertion");
    };
    assert_eq!(record.decision, SinkPreDispatchDecision::Allow);
}

#[then("the audit record decision is Deny")]
fn then_audit_decision_deny(world: &SinkEnforcementWorld) {
    let Some(record) = world.audit_record.as_ref() else {
        panic!("audit record must be emitted before assertion");
    };
    assert_eq!(record.decision, SinkPreDispatchDecision::Deny);
}

#[then("the audit record redaction applied flag is false")]
fn then_audit_redaction_false(world: &SinkEnforcementWorld) {
    let Some(record) = world.audit_record.as_ref() else {
        panic!("audit record must be emitted before assertion");
    };
    assert!(!record.redaction_applied);
}

#[then("the audit record call path is Quarantined")]
fn then_audit_call_path_quarantined(world: &SinkEnforcementWorld) {
    let Some(record) = world.audit_record.as_ref() else {
        panic!("audit record must be emitted before assertion");
    };
    assert_eq!(record.call_path, LlmCallPath::Quarantined);
}

// ── Scenario bindings ───────────────────────────────────────────────

#[scenario(
    path = "tests/security/features/llm_sink_enforcement.feature",
    name = "Pre-dispatch allows planner LLM call with redaction applied"
)]
fn pre_dispatch(world: SinkEnforcementWorld) {
    assert!(world.pre_dispatch_decision.is_some());
}

#[scenario(
    path = "tests/security/features/llm_sink_enforcement.feature",
    name = "Pre-dispatch denies planner LLM call without redaction"
)]
fn pre_dispatch_deny(world: SinkEnforcementWorld) {
    assert!(world.pre_dispatch_decision.is_some());
}

#[scenario(
    path = "tests/security/features/llm_sink_enforcement.feature",
    name = "Transport guard passes when redaction is applied"
)]
fn transport_guard_passes(world: SinkEnforcementWorld) {
    assert!(world.transport_outcome.is_some());
}

#[scenario(
    path = "tests/security/features/llm_sink_enforcement.feature",
    name = "Transport guard blocks when redaction is missing"
)]
fn transport_guard_blocks(world: SinkEnforcementWorld) {
    assert!(world.transport_outcome.is_some());
}

#[scenario(
    path = "tests/security/features/llm_sink_enforcement.feature",
    name = "Post-dispatch audit record links execution and call identifiers"
)]
fn audit_record_linkage(world: SinkEnforcementWorld) {
    assert!(world.audit_record.is_some());
}

#[scenario(
    path = "tests/security/features/llm_sink_enforcement.feature",
    name = "Denied pre-dispatch decision emits audit record with Deny"
)]
fn denied_audit_record(world: SinkEnforcementWorld) {
    assert!(world.audit_record.is_some());
}

#[scenario(
    path = "tests/security/features/llm_sink_enforcement.feature",
    name = "Quarantined LLM call emits linked audit record"
)]
fn quarantined_audit_record(world: SinkEnforcementWorld) {
    assert!(world.audit_record.is_some());
}
