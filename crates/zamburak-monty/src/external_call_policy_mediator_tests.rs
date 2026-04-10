//! Policy-backed mediator tests and shared call-context builders.

use monty::ExternalCallKind;
use rstest::rstest;
use zamburak_core::control_context::ExecutionContextSummary;
use zamburak_core::propagation::PropagationMode;
use zamburak_core::trust::IntegrityLabel;
use zamburak_core::value_id::ValueId;
use zamburak_core::{AuthorityCapability, AuthoritySet, DataLabels, DependencySummary};

use crate::PolicyMediator;
use crate::external_call::{CallContext, CallIfcContext, ExternalCallMediator, MediationDecision};

const POLICY_TEST_HEADER: &str = r#"schema_version: 1
policy_name: test_policy
default_action: Deny
strict_mode: true
budgets:
  max_values: 100
  max_parents_per_value: 10
  max_closure_steps: 100
  max_witness_depth: 10"#;

pub(super) fn function_call_context(call_id: u32, name: &str) -> CallContext {
    call_context(call_id, ExternalCallKind::Function, name)
}

pub(super) fn os_call_context(call_id: u32, name: &str) -> CallContext {
    call_context(call_id, ExternalCallKind::Os, name)
}

fn method_call_context(call_id: u32, name: &str) -> CallContext {
    call_context(call_id, ExternalCallKind::Method, name)
}

fn call_context(call_id: u32, kind: ExternalCallKind, name: &str) -> CallContext {
    CallContext {
        call_id,
        kind,
        function_name: name.to_owned(),
        caller_authority: AuthoritySet::full(),
        kwarg_names: vec![],
        ifc: default_ifc_context(),
    }
}

pub(super) fn default_ifc_context() -> CallIfcContext {
    CallIfcContext {
        propagation_mode: PropagationMode::Normal,
        aggregate_summary: DependencySummary::unknown_top(),
        control_context: ExecutionContextSummary::new(),
        arg_summaries: Vec::new(),
        kwarg_summaries: Vec::new(),
    }
}

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
        caller_authority: AuthoritySet::full(),
        kwarg_names: vec![],
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

#[rstest]
fn policy_mediator_matches_kwarg_rules_by_keyword_name() {
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
  - tool: guarded_write
    side_effect_class: ExternalWrite
    arg_rules:
      - arg: path
        requires_integrity: Verified
    default_decision: Allow
"#;
    let engine =
        zamburak_policy::PolicyEngine::from_yaml_str(policy_yaml).expect("valid test policy");
    let mut mediator = PolicyMediator::new(engine);
    let ctx = CallContext {
        call_id: 2,
        kind: ExternalCallKind::Function,
        function_name: "guarded_write".to_owned(),
        caller_authority: AuthoritySet::full(),
        kwarg_names: vec!["path".to_owned()],
        ifc: CallIfcContext {
            propagation_mode: PropagationMode::Normal,
            aggregate_summary: DependencySummary::unknown_top(),
            control_context: ExecutionContextSummary::new(),
            arg_summaries: vec![],
            kwarg_summaries: vec![(
                DependencySummary::unknown_top(),
                DependencySummary {
                    integrity_join: IntegrityLabel::Untrusted,
                    confidentiality_join: DataLabels::new(),
                    authority_join: Default::default(),
                    origin_count: 1,
                    truncated: false,
                },
            )],
        },
    };

    assert!(matches!(
        mediator.mediate(&ctx),
        MediationDecision::Deny { .. }
    ));
}

#[rstest]
fn policy_mediator_denies_malformed_kwarg_context() {
    let mut mediator = make_mediator_for_single_tool("guarded_write", "Allow");
    let ctx = CallContext {
        call_id: 3,
        kind: ExternalCallKind::Function,
        function_name: "guarded_write".to_owned(),
        caller_authority: AuthoritySet::full(),
        kwarg_names: vec!["path".to_owned()],
        ifc: CallIfcContext {
            propagation_mode: PropagationMode::Normal,
            aggregate_summary: DependencySummary::unknown_top(),
            control_context: ExecutionContextSummary::new(),
            arg_summaries: vec![],
            kwarg_summaries: vec![],
        },
    };

    assert!(matches!(
        mediator.mediate(&ctx),
        MediationDecision::Deny { .. }
    ));
}

#[rstest]
fn policy_mediator_uses_explicit_caller_authority() {
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
  - tool: guarded_write
    side_effect_class: ExternalWrite
    required_authority:
      - email.send
    default_decision: Allow
"#;
    let engine =
        zamburak_policy::PolicyEngine::from_yaml_str(policy_yaml).expect("valid test policy");
    let mut mediator = PolicyMediator::new(engine);
    let ctx = CallContext {
        call_id: 4,
        kind: ExternalCallKind::Function,
        function_name: "guarded_write".to_owned(),
        caller_authority: AuthoritySet::from_iter([
            AuthorityCapability::try_from("email.send").expect("valid authority capability")
        ]),
        kwarg_names: vec![],
        ifc: CallIfcContext {
            propagation_mode: PropagationMode::Normal,
            aggregate_summary: DependencySummary::unknown_top(),
            control_context: ExecutionContextSummary::new(),
            arg_summaries: vec![],
            kwarg_summaries: vec![],
        },
    };

    assert_eq!(mediator.mediate(&ctx), MediationDecision::Allow);
}
