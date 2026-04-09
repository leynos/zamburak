//! Unit tests for external-call mediation traits and built-in mediators.

use monty::ExternalCallKind;
use rstest::rstest;
use zamburak_core::DataLabels;
use zamburak_core::DependencySummary;
use zamburak_core::control_context::ExecutionContextSummary;
use zamburak_core::propagation::PropagationMode;
use zamburak_core::trust::IntegrityLabel;
use zamburak_core::value_id::ValueId;

use crate::PolicyMediator;
use crate::external_call::{
    AllowAllMediator, CallContext, CallIfcContext, ConfirmationContext, DenyAllMediator,
    ExternalCallMediator, MediationDecision,
};

const POLICY_TEST_HEADER: &str = r#"schema_version: 1
policy_name: test_policy
default_action: Deny
strict_mode: true
budgets:
  max_values: 100
  max_parents_per_value: 10
  max_closure_steps: 100
  max_witness_depth: 10"#;

struct RequireConfirmationMediator;

impl ExternalCallMediator for RequireConfirmationMediator {
    fn mediate(&mut self, context: &CallContext) -> MediationDecision {
        MediationDecision::RequireConfirmation {
            request: ConfirmationContext {
                description: format!("confirm {}", context.function_name),
                call: context.clone(),
            },
        }
    }
}

fn function_call_context(call_id: u32, name: &str) -> CallContext {
    CallContext {
        call_id,
        kind: ExternalCallKind::Function,
        function_name: name.to_owned(),
        ifc: default_ifc_context(),
    }
}

fn os_call_context(call_id: u32, name: &str) -> CallContext {
    CallContext {
        call_id,
        kind: ExternalCallKind::Os,
        function_name: name.to_owned(),
        ifc: default_ifc_context(),
    }
}

fn method_call_context(call_id: u32, name: &str) -> CallContext {
    CallContext {
        call_id,
        kind: ExternalCallKind::Method,
        function_name: name.to_owned(),
        ifc: default_ifc_context(),
    }
}

fn default_ifc_context() -> CallIfcContext {
    CallIfcContext {
        propagation_mode: PropagationMode::Normal,
        aggregate_summary: DependencySummary::unknown_top(),
        control_context: ExecutionContextSummary::new(),
        arg_summaries: Vec::new(),
        kwarg_summaries: Vec::new(),
    }
}

#[rstest]
fn deny_all_mediator_returns_deny_for_function_call() {
    let mut mediator = DenyAllMediator;
    let ctx = function_call_context(1, "exit");
    let decision = mediator.mediate(&ctx);
    match decision {
        MediationDecision::Deny { reason } => {
            assert!(
                reason.contains("DenyAllMediator"),
                "reason should mention DenyAllMediator: {reason}"
            );
            assert!(
                reason.contains("exit"),
                "reason should mention function name: {reason}"
            );
        }
        other => panic!("expected Deny, got {other:?}"),
    }
}

#[rstest]
fn deny_all_mediator_returns_deny_for_os_call() {
    let mut mediator = DenyAllMediator;
    let ctx = os_call_context(5, "stat");
    let decision = mediator.mediate(&ctx);
    match decision {
        MediationDecision::Deny { reason } => {
            assert!(reason.contains("stat"));
        }
        other => panic!("expected Deny, got {other:?}"),
    }
}

#[rstest]
#[case(ExternalCallKind::Function)]
#[case(ExternalCallKind::Os)]
#[case(ExternalCallKind::Method)]
fn allow_all_mediator_allows_all_call_kinds(#[case] kind: ExternalCallKind) {
    let mut mediator = AllowAllMediator;
    let ctx = CallContext {
        call_id: 0,
        kind,
        function_name: "test_fn".to_owned(),
        ifc: default_ifc_context(),
    };
    assert_eq!(mediator.mediate(&ctx), MediationDecision::Allow);
}

#[rstest]
#[case(ExternalCallKind::Function)]
#[case(ExternalCallKind::Os)]
#[case(ExternalCallKind::Method)]
fn deny_all_mediator_denies_all_call_kinds(#[case] kind: ExternalCallKind) {
    let mut mediator = DenyAllMediator;
    let ctx = CallContext {
        call_id: 0,
        kind,
        function_name: "test_fn".to_owned(),
        ifc: default_ifc_context(),
    };
    assert!(matches!(
        mediator.mediate(&ctx),
        MediationDecision::Deny { .. }
    ));
}

#[rstest]
fn require_confirmation_round_trips_confirmation_context() {
    let mut mediator = RequireConfirmationMediator;
    let ctx = function_call_context(42, "test_require_confirmation_roundtrip");

    let decision = mediator.mediate(&ctx);

    match decision {
        MediationDecision::RequireConfirmation { request } => {
            assert_eq!(request.call, ctx);
            assert_eq!(
                request.description,
                "confirm test_require_confirmation_roundtrip"
            );
        }
        other => panic!("expected RequireConfirmation, got {other:?}"),
    }
}

// Policy-backed mediator tests.
// These tests verify that the policy-backed mediator correctly translates
// CallContext into policy engine input and converts policy decisions back
// into MediationDecision.

fn make_mediator_with_no_tools() -> crate::PolicyMediator {
    let yaml = format!("{}\ntools: []", POLICY_TEST_HEADER);
    let engine = zamburak_policy::PolicyEngine::from_yaml_str(&yaml).expect("valid test policy");
    crate::PolicyMediator::new(engine)
}

fn make_mediator_for_single_tool(tool: &str, default_decision: &str) -> crate::PolicyMediator {
    let yaml = format!(
        "{}\ntools:\n  - tool: {}\n    side_effect_class: ExternalWrite\n    default_decision: {}",
        POLICY_TEST_HEADER, tool, default_decision
    );
    let engine = zamburak_policy::PolicyEngine::from_yaml_str(&yaml).expect("valid test policy");
    crate::PolicyMediator::new(engine)
}

#[rstest]
fn policy_mediator_denies_missing_tool() {
    let mut mediator = make_mediator_with_no_tools();
    let ctx = function_call_context(1, "unknown_tool");
    let decision = mediator.mediate(&ctx);
    match &decision {
        MediationDecision::Deny { reason } => {
            assert!(
                reason.contains("no policy defined for tool"),
                "deny reason should mention missing tool: {reason}"
            );
        }
        other => panic!("expected Deny, got {other:?}"),
    }
}

#[rstest]
fn policy_mediator_allows_tool_with_allow_default() {
    let mut mediator = make_mediator_for_single_tool("safe_print", "Allow");
    let ctx = function_call_context(1, "safe_print");
    let decision = mediator.mediate(&ctx);
    assert!(
        matches!(decision, MediationDecision::Allow),
        "tool with Allow default_decision should allow"
    );
}

#[rstest]
fn policy_mediator_allows_os_call_with_allow_default() {
    let mut mediator = make_mediator_for_single_tool("safe_print", "Allow");
    let ctx = os_call_context(1, "safe_print");
    let decision = mediator.mediate(&ctx);
    assert!(
        matches!(decision, MediationDecision::Allow),
        "OS call for tool with Allow default should be allowed"
    );
}

#[rstest]
fn policy_mediator_allows_method_call_with_allow_default() {
    let mut mediator = make_mediator_for_single_tool("safe_print", "Allow");
    let ctx = method_call_context(1, "safe_print");
    let decision = mediator.mediate(&ctx);
    assert!(
        matches!(decision, MediationDecision::Allow),
        "Method call for tool with Allow default should be allowed"
    );
}

#[rstest]
fn policy_mediator_denies_on_context_rule_violation() {
    let policy_yaml = r#"
schema_version: 1
policy_name: test_policy
default_action: Deny
strict_mode: true
budgets:
  max_values: 100
  max_parents_per_value: 10
  max_closure_steps: 100
  max_witness_depth: 10
tools:
  - tool: llm_api_call
    side_effect_class: ExternalWrite
    context_rules:
      deny_if_pc_integrity_contains:
        - Untrusted
    default_decision: Allow
"#;

    let engine =
        zamburak_policy::PolicyEngine::from_yaml_str(policy_yaml).expect("valid test policy");
    let mut mediator = PolicyMediator::new(engine);

    // Create a context with Untrusted PC integrity.
    let mut control_context = ExecutionContextSummary::new();
    let untrusted_condition = DependencySummary {
        integrity_join: IntegrityLabel::Untrusted,
        confidentiality_join: DataLabels::new(),
        authority_join: Default::default(),
        origin_count: 1,
        truncated: false,
    };
    control_context.push_condition(ValueId::new(1), &untrusted_condition);

    let ifc = CallIfcContext {
        propagation_mode: PropagationMode::Strict,
        aggregate_summary: DependencySummary::unknown_top(),
        control_context,
        arg_summaries: vec![],
        kwarg_summaries: vec![],
    };
    let ctx = CallContext {
        call_id: 1,
        kind: ExternalCallKind::Function,
        function_name: "llm_api_call".to_owned(),
        ifc,
    };

    let decision = mediator.mediate(&ctx);

    match &decision {
        MediationDecision::Deny { reason } => {
            assert!(
                reason.contains("context rule"),
                "deny reason should mention context rule: {reason}"
            );
        }
        other => panic!("expected Deny, got {other:?}"),
    }
}

#[rstest]
fn policy_mediator_requires_confirmation_when_configured() {
    let mut mediator = make_mediator_for_single_tool("sensitive_api", "RequireConfirmation");
    let ctx = function_call_context(1, "sensitive_api");
    let decision = mediator.mediate(&ctx);
    match &decision {
        MediationDecision::RequireConfirmation { request } => {
            assert!(
                request.description.contains("confirmation required"),
                "confirmation description should contain policy explanation: {}",
                request.description
            );
        }
        other => panic!("expected RequireConfirmation, got {other:?}"),
    }
}
