//! Unit tests for external-call policy evaluation.

use zamburak_core::control_context::ExecutionContextSummary;
use zamburak_core::trust::IntegrityLabel;
use zamburak_core::{DataLabel, DependencySummary};

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

#[test]
fn tool_with_allow_default_decision_returns_allow() {
    let engine = minimal_policy_with_tools(
        r#"
  - tool: safe_print
    side_effect_class: ExternalWrite
    default_decision: Allow
"#,
    );
    let input = ExternalCallPolicyInput {
        tool_name: "safe_print".to_owned(),
        call_kind: ExternalCallKind::Function,
        aggregate_summary: DependencySummary::unknown_top(),
        arg_summaries: vec![],
        kwarg_summaries: vec![],
        control_context: ExecutionContextSummary::new(),
    };

    let decision = engine.evaluate_external_call(&input);

    assert!(
        matches!(decision, ExternalCallPolicyDecision::Allow(_)),
        "tool with Allow default_decision should allow when no rules fire"
    );
}

#[test]
fn tool_with_deny_default_decision_returns_deny() {
    let engine = minimal_policy_with_tools(
        r#"
  - tool: dangerous_exec
    side_effect_class: ExternalWrite
    default_decision: Deny
"#,
    );
    let input = ExternalCallPolicyInput {
        tool_name: "dangerous_exec".to_owned(),
        call_kind: ExternalCallKind::Function,
        aggregate_summary: DependencySummary::unknown_top(),
        arg_summaries: vec![],
        kwarg_summaries: vec![],
        control_context: ExecutionContextSummary::new(),
    };

    let decision = engine.evaluate_external_call(&input);

    assert!(
        matches!(decision, ExternalCallPolicyDecision::Deny(_)),
        "tool with Deny default_decision should deny when no rules fire"
    );
}

#[test]
fn tool_with_require_confirmation_default_decision_returns_confirmation() {
    let engine = minimal_policy_with_tools(
        r#"
  - tool: sensitive_api_call
    side_effect_class: ExternalWrite
    default_decision: RequireConfirmation
"#,
    );
    let input = ExternalCallPolicyInput {
        tool_name: "sensitive_api_call".to_owned(),
        call_kind: ExternalCallKind::Function,
        aggregate_summary: DependencySummary::unknown_top(),
        arg_summaries: vec![],
        kwarg_summaries: vec![],
        control_context: ExecutionContextSummary::new(),
    };

    let decision = engine.evaluate_external_call(&input);

    assert!(
        matches!(decision, ExternalCallPolicyDecision::RequireConfirmation(_)),
        "tool with RequireConfirmation default_decision should require confirmation"
    );
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

    // Push an untrusted condition to set PC integrity to Untrusted.
    use zamburak_core::DataLabels;
    use zamburak_core::value_id::ValueId;
    let mut control_context = ExecutionContextSummary::new();
    let untrusted_condition = DependencySummary {
        integrity_join: IntegrityLabel::Untrusted,
        confidentiality_join: DataLabels::new(),
        authority_join: Default::default(),
        origin_count: 1,
        truncated: false,
    };
    control_context.push_condition(ValueId::new(1), &untrusted_condition);

    let input = ExternalCallPolicyInput {
        tool_name: "llm_api_call".to_owned(),
        call_kind: ExternalCallKind::Function,
        aggregate_summary: DependencySummary::unknown_top(),
        arg_summaries: vec![],
        kwarg_summaries: vec![],
        control_context,
    };

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

    // Argument summary with Untrusted integrity instead of Verified.
    let path_summary = DependencySummary {
        integrity_join: IntegrityLabel::Untrusted,
        confidentiality_join: Default::default(),
        authority_join: Default::default(),
        origin_count: 1,
        truncated: false,
    };

    let input = ExternalCallPolicyInput {
        tool_name: "file_write".to_owned(),
        call_kind: ExternalCallKind::Function,
        aggregate_summary: DependencySummary::unknown_top(),
        arg_summaries: vec![path_summary],
        kwarg_summaries: vec![],
        control_context: ExecutionContextSummary::new(),
    };

    let decision = engine.evaluate_external_call(&input);

    assert!(
        matches!(decision, ExternalCallPolicyDecision::Deny(_)),
        "requires_integrity should deny when argument integrity is insufficient"
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

    // Argument summary with AuthSecret confidentiality label.
    use zamburak_core::DataLabels;
    let message_summary = DependencySummary {
        integrity_join: IntegrityLabel::Trusted,
        confidentiality_join: DataLabels::from_iter([DataLabel::AuthSecret]),
        authority_join: Default::default(),
        origin_count: 1,
        truncated: false,
    };

    let input = ExternalCallPolicyInput {
        tool_name: "public_log".to_owned(),
        call_kind: ExternalCallKind::Function,
        aggregate_summary: DependencySummary::unknown_top(),
        arg_summaries: vec![message_summary],
        kwarg_summaries: vec![],
        control_context: ExecutionContextSummary::new(),
    };

    let decision = engine.evaluate_external_call(&input);

    assert!(
        matches!(decision, ExternalCallPolicyDecision::Deny(_)),
        "forbids_confidentiality should deny when forbidden label is present"
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

    // Argument with sufficient integrity.
    let data_summary = DependencySummary {
        integrity_join: IntegrityLabel::Verified,
        confidentiality_join: Default::default(),
        authority_join: Default::default(),
        origin_count: 1,
        truncated: false,
    };

    let input = ExternalCallPolicyInput {
        tool_name: "safe_write".to_owned(),
        call_kind: ExternalCallKind::Function,
        aggregate_summary: DependencySummary::unknown_top(),
        arg_summaries: vec![data_summary],
        kwarg_summaries: vec![],
        control_context: ExecutionContextSummary::new(),
    };

    let decision = engine.evaluate_external_call(&input);

    assert!(
        matches!(decision, ExternalCallPolicyDecision::Allow(_)),
        "arg rules should allow when all constraints are satisfied"
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

    let input = ExternalCallPolicyInput {
        tool_name: "draft_email".to_owned(),
        call_kind: ExternalCallKind::Function,
        aggregate_summary: DependencySummary::unknown_top(),
        arg_summaries: vec![],
        kwarg_summaries: vec![],
        control_context: ExecutionContextSummary::new(),
    };

    let decision = engine.evaluate_external_call(&input);

    // Task 0.6.4 maps RequireDraft conservatively to RequireConfirmation.
    assert!(
        matches!(decision, ExternalCallPolicyDecision::RequireConfirmation(_)),
        "RequireDraft should map to RequireConfirmation for Task 0.6.4"
    );
}
