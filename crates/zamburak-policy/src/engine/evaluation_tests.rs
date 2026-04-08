//! Unit tests for external-call policy evaluation.

use rstest::rstest;
use zamburak_core::control_context::ExecutionContextSummary;
use zamburak_core::trust::IntegrityLabel;
use zamburak_core::{DataLabel, DataLabels, DependencySummary};

use super::{ExternalCallKind, ExternalCallPolicyDecision, ExternalCallPolicyInput};
use crate::engine::PolicyEngine;

/// Helper to build a minimal test policy with custom tools configuration.
fn minimal_policy_with_tools(tools_yaml: &str) -> PolicyEngine {
    let policy_yaml = format!(
        r#"
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
{tools_yaml}
"#
    );
    PolicyEngine::from_yaml_str(&policy_yaml).expect("valid test policy")
}

/// Helper to build an ExternalCallPolicyInput for testing.
fn make_input(
    tool_name: &str,
    arg_summaries: Vec<DependencySummary>,
    control_context: ExecutionContextSummary,
) -> ExternalCallPolicyInput {
    ExternalCallPolicyInput {
        tool_name: tool_name.to_owned(),
        call_kind: ExternalCallKind::Function,
        aggregate_summary: DependencySummary::unknown_top(),
        arg_summaries,
        kwarg_summaries: vec![],
        control_context,
    }
}

/// Helper to build a DependencySummary with specified integrity and default other fields.
fn summary_with_integrity(integrity: IntegrityLabel) -> DependencySummary {
    DependencySummary {
        integrity_join: integrity,
        confidentiality_join: DataLabels::new(),
        authority_join: Default::default(),
        origin_count: 1,
        truncated: false,
    }
}

/// Helper to build a DependencySummary with specified confidentiality labels and default other fields.
fn summary_with_confidentiality(labels: &[DataLabel]) -> DependencySummary {
    DependencySummary {
        integrity_join: IntegrityLabel::Trusted,
        confidentiality_join: DataLabels::from_iter(labels.iter().copied()),
        authority_join: Default::default(),
        origin_count: 1,
        truncated: false,
    }
}

/// Helper to build a control context with an untrusted condition pushed.
fn control_context_with_untrusted_pc() -> ExecutionContextSummary {
    use zamburak_core::value_id::ValueId;
    let mut context = ExecutionContextSummary::new();
    let untrusted_condition = summary_with_integrity(IntegrityLabel::Untrusted);
    context.push_condition(ValueId::new(1), &untrusted_condition);
    context
}

#[test]
fn missing_tool_policy_fails_closed_with_deny() {
    let engine = minimal_policy_with_tools("");
    let input = ExternalCallPolicyInput {
        tool_name: "unknown_tool".to_owned(),
        call_kind: ExternalCallKind::Function,
        aggregate_summary: DependencySummary::unknown_top(),
        arg_summaries: vec![],
        kwarg_summaries: vec![],
        control_context: ExecutionContextSummary::new(),
    };

    let decision = engine.evaluate_external_call(&input);

    assert!(
        matches!(decision, ExternalCallPolicyDecision::Deny(_)),
        "missing tool policy must fail closed with deny"
    );
}

#[rstest]
#[case("Allow", "safe_print", "allow")]
#[case("Deny", "dangerous_exec", "deny")]
#[case("RequireConfirmation", "sensitive_api_call", "require confirmation")]
fn tool_default_decision_returns_expected_outcome(
    #[case] default_decision: &str,
    #[case] tool_name: &str,
    #[case] expected_outcome: &str,
) {
    let tools_yaml = format!(
        r#"
  - tool: {tool_name}
    side_effect_class: ExternalWrite
    default_decision: {default_decision}
"#
    );
    let engine = minimal_policy_with_tools(&tools_yaml);
    let input = make_input(tool_name, vec![], ExecutionContextSummary::new());

    let decision = engine.evaluate_external_call(&input);

    match expected_outcome {
        "allow" => assert!(
            matches!(decision, ExternalCallPolicyDecision::Allow(_)),
            "tool with Allow default_decision should allow when no rules fire"
        ),
        "deny" => assert!(
            matches!(decision, ExternalCallPolicyDecision::Deny(_)),
            "tool with Deny default_decision should deny when no rules fire"
        ),
        "require confirmation" => assert!(
            matches!(decision, ExternalCallPolicyDecision::RequireConfirmation(_)),
            "tool with RequireConfirmation default_decision should require confirmation"
        ),
        _ => panic!("unexpected outcome: {expected_outcome}"),
    }
}

#[test]
fn strict_mode_context_rule_denies_on_pc_integrity_match() {
    let engine = minimal_policy_with_tools(
        r#"
  - tool: llm_api_call
    side_effect_class: ExternalWrite
    context_rules:
      deny_if_pc_integrity_contains:
        - Untrusted
    default_decision: Allow
"#,
    );

    let input = make_input("llm_api_call", vec![], control_context_with_untrusted_pc());
    let decision = engine.evaluate_external_call(&input);

    assert!(
        matches!(decision, ExternalCallPolicyDecision::Deny(_)),
        "deny_if_pc_integrity_contains should deny when PC integrity matches"
    );
}

#[test]
fn arg_rule_requires_integrity_denies_when_missing() {
    let engine = minimal_policy_with_tools(
        r#"
  - tool: file_write
    side_effect_class: ExternalWrite
    arg_rules:
      - arg: path
        requires_integrity: Verified
    default_decision: Allow
"#,
    );

    let path_summary = summary_with_integrity(IntegrityLabel::Untrusted);
    let input = make_input(
        "file_write",
        vec![path_summary],
        ExecutionContextSummary::new(),
    );
    let decision = engine.evaluate_external_call(&input);

    assert!(
        matches!(decision, ExternalCallPolicyDecision::Deny(_)),
        "requires_integrity should deny when argument integrity is insufficient"
    );
}

#[test]
fn arg_rules_pass_when_constraints_are_met() {
    let engine = minimal_policy_with_tools(
        r#"
  - tool: safe_write
    side_effect_class: ExternalWrite
    arg_rules:
      - arg: data
        requires_integrity: Trusted
    default_decision: Allow
"#,
    );

    let data_summary = summary_with_integrity(IntegrityLabel::Verified);
    let input = make_input(
        "safe_write",
        vec![data_summary],
        ExecutionContextSummary::new(),
    );
    let decision = engine.evaluate_external_call(&input);

    assert!(
        matches!(decision, ExternalCallPolicyDecision::Allow(_)),
        "arg rules should allow when all constraints are satisfied"
    );
}

#[test]
fn arg_rule_forbids_confidentiality_denies_when_present() {
    let engine = minimal_policy_with_tools(
        r#"
  - tool: public_log
    side_effect_class: ExternalWrite
    arg_rules:
      - arg: message
        forbids_confidentiality:
          - AuthSecret
    default_decision: Allow
"#,
    );

    let message_summary = summary_with_confidentiality(&[DataLabel::AuthSecret]);
    let input = make_input(
        "public_log",
        vec![message_summary],
        ExecutionContextSummary::new(),
    );
    let decision = engine.evaluate_external_call(&input);

    assert!(
        matches!(decision, ExternalCallPolicyDecision::Deny(_)),
        "forbids_confidentiality should deny when forbidden label is present"
    );
}

#[test]
fn require_draft_maps_to_require_confirmation_conservatively() {
    let engine = minimal_policy_with_tools(
        r#"
  - tool: draft_email
    side_effect_class: ExternalWrite
    default_decision: RequireDraft
"#,
    );

    let input = make_input("draft_email", vec![], ExecutionContextSummary::new());
    let decision = engine.evaluate_external_call(&input);

    assert!(
        matches!(decision, ExternalCallPolicyDecision::RequireConfirmation(_)),
        "RequireDraft should map to RequireConfirmation for Task 0.6.4"
    );
}
