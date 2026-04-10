//! Unit tests for external-call policy evaluation.

#[cfg(test)]
#[path = "evaluation_test_helpers.rs"]
mod helpers;

use rstest::rstest;
use zamburak_core::control_context::ExecutionContextSummary;
use zamburak_core::trust::{AuthoritySet, IntegrityLabel};
use zamburak_core::{AuthorityCapability, DataLabel};

use super::ExternalCallPolicyDecision;
use helpers::{
    control_context_with_untrusted_pc, make_input, make_input_full, minimal_policy_with_tools,
    named_kwarg_summary, summary_with_confidentiality, summary_with_integrity,
};

#[test]
fn missing_tool_policy_fails_closed_with_deny() {
    let engine = minimal_policy_with_tools("");
    let input = make_input("unknown_tool", vec![], ExecutionContextSummary::new());
    let decision = engine.evaluate_external_call(&input);
    assert!(matches!(decision, ExternalCallPolicyDecision::Deny(_)));
}

#[rstest]
#[case("Allow", "safe_print", "allow")]
#[case("Deny", "dangerous_exec", "deny")]
#[case("RequireConfirmation", "sensitive_api", "confirm")]
fn tool_default_decision_returns_expected_outcome(
    #[case] default_decision: &str,
    #[case] tool_name: &str,
    #[case] expected: &str,
) {
    let yaml = format!(
        "  - tool: {tool_name}\n    side_effect_class: ExternalWrite\n    default_decision: {default_decision}"
    );
    let engine = minimal_policy_with_tools(&yaml);
    let input = make_input(tool_name, vec![], ExecutionContextSummary::new());
    let decision = engine.evaluate_external_call(&input);
    match expected {
        "allow" => assert!(matches!(decision, ExternalCallPolicyDecision::Allow(_))),
        "deny" => assert!(matches!(decision, ExternalCallPolicyDecision::Deny(_))),
        "confirm" => assert!(matches!(
            decision,
            ExternalCallPolicyDecision::RequireConfirmation(_)
        )),
        _ => panic!("unexpected: {expected}"),
    }
}

#[rstest]
#[case("Untrusted")]
#[case("UNTRUSTED")]
fn context_rule_denies_on_pc_integrity_match(#[case] label: &str) {
    let yaml = format!(
        concat!(
            "  - tool: llm_api_call\n",
            "    side_effect_class: ExternalWrite\n",
            "    context_rules:\n",
            "      deny_if_pc_integrity_contains:\n",
            "        - {}\n",
            "    default_decision: Allow"
        ),
        label
    );
    let engine = minimal_policy_with_tools(&yaml);
    let input = make_input("llm_api_call", vec![], control_context_with_untrusted_pc());
    assert!(matches!(
        engine.evaluate_external_call(&input),
        ExternalCallPolicyDecision::Deny(_)
    ));
}

#[test]
fn arg_rule_requires_integrity_denies_when_missing() {
    let engine = minimal_policy_with_tools(concat!(
        "  - tool: file_write\n",
        "    side_effect_class: ExternalWrite\n",
        "    arg_rules:\n",
        "      - arg: path\n",
        "        requires_integrity: Verified\n",
        "    default_decision: Allow"
    ));
    let input = make_input(
        "file_write",
        vec![summary_with_integrity(IntegrityLabel::Untrusted)],
        ExecutionContextSummary::new(),
    );
    assert!(matches!(
        engine.evaluate_external_call(&input),
        ExternalCallPolicyDecision::Deny(_)
    ));
}

#[test]
fn arg_rules_pass_when_constraints_are_met() {
    let engine = minimal_policy_with_tools(concat!(
        "  - tool: safe_write\n",
        "    side_effect_class: ExternalWrite\n",
        "    arg_rules:\n",
        "      - arg: data\n",
        "        requires_integrity: Trusted\n",
        "    default_decision: Allow"
    ));
    let input = make_input(
        "safe_write",
        vec![summary_with_integrity(IntegrityLabel::Verified)],
        ExecutionContextSummary::new(),
    );
    assert!(matches!(
        engine.evaluate_external_call(&input),
        ExternalCallPolicyDecision::Allow(_)
    ));
}

#[test]
fn more_arg_rules_than_arguments_does_not_panic() {
    let engine = minimal_policy_with_tools(concat!(
        "  - tool: guarded_write\n",
        "    side_effect_class: ExternalWrite\n",
        "    arg_rules:\n",
        "      - arg: first\n",
        "        requires_integrity: Trusted\n",
        "      - arg: second\n",
        "        requires_integrity: Verified\n",
        "    default_decision: Allow"
    ));
    let input = make_input(
        "guarded_write",
        vec![summary_with_integrity(IntegrityLabel::Verified)],
        ExecutionContextSummary::new(),
    );
    assert!(matches!(
        engine.evaluate_external_call(&input),
        ExternalCallPolicyDecision::Allow(_)
    ));
}

#[test]
fn more_arguments_than_arg_rules_falls_through_to_default() {
    let engine = minimal_policy_with_tools(concat!(
        "  - tool: guarded_write\n",
        "    side_effect_class: ExternalWrite\n",
        "    arg_rules:\n",
        "      - arg: first\n",
        "        requires_integrity: Trusted\n",
        "    default_decision: Allow"
    ));
    let input = make_input(
        "guarded_write",
        vec![
            summary_with_integrity(IntegrityLabel::Verified),
            summary_with_integrity(IntegrityLabel::Untrusted),
        ],
        ExecutionContextSummary::new(),
    );
    assert!(matches!(
        engine.evaluate_external_call(&input),
        ExternalCallPolicyDecision::Allow(_)
    ));
}

#[rstest]
#[case("AuthSecret")]
#[case("AUTH_SECRET")]
fn arg_rule_forbids_confidentiality_denies_when_present(#[case] label: &str) {
    let yaml = format!(
        concat!(
            "  - tool: public_log\n",
            "    side_effect_class: ExternalWrite\n",
            "    arg_rules:\n",
            "      - arg: message\n",
            "        forbids_confidentiality:\n",
            "          - {}\n",
            "    default_decision: Allow"
        ),
        label
    );
    let engine = minimal_policy_with_tools(&yaml);
    let input = make_input(
        "public_log",
        vec![summary_with_confidentiality(&[DataLabel::AuthSecret])],
        ExecutionContextSummary::new(),
    );
    assert!(matches!(
        engine.evaluate_external_call(&input),
        ExternalCallPolicyDecision::Deny(_)
    ));
}

#[test]
fn context_rule_with_unrecognised_integrity_label_fails_closed() {
    let engine = minimal_policy_with_tools(concat!(
        "  - tool: api_call\n",
        "    side_effect_class: ExternalWrite\n",
        "    context_rules:\n",
        "      deny_if_pc_integrity_contains:\n",
        "        - Typoed\n",
        "    default_decision: Allow"
    ));
    let input = make_input("api_call", vec![], ExecutionContextSummary::new());
    let decision = engine.evaluate_external_call(&input);
    match &decision {
        ExternalCallPolicyDecision::Deny(e) => assert!(
            e.summary.contains("unrecognised integrity label"),
            "should mention unrecognised label: {}",
            e.summary
        ),
        other => panic!("expected Deny, got {other:?}"),
    }
}

#[test]
fn arg_rule_with_unrecognised_confidentiality_label_fails_closed() {
    let engine = minimal_policy_with_tools(concat!(
        "  - tool: log_call\n",
        "    side_effect_class: ExternalWrite\n",
        "    arg_rules:\n",
        "      - arg: msg\n",
        "        forbids_confidentiality:\n",
        "          - NonExistentLabel\n",
        "    default_decision: Allow"
    ));
    let input = make_input(
        "log_call",
        vec![summary_with_integrity(IntegrityLabel::Trusted)],
        ExecutionContextSummary::new(),
    );
    assert!(matches!(
        engine.evaluate_external_call(&input),
        ExternalCallPolicyDecision::Deny(_)
    ));
}

#[test]
fn require_draft_maps_to_require_confirmation_conservatively() {
    let engine = minimal_policy_with_tools(concat!(
        "  - tool: draft_email\n",
        "    side_effect_class: ExternalWrite\n",
        "    default_decision: RequireDraft"
    ));
    let input = make_input("draft_email", vec![], ExecutionContextSummary::new());
    assert!(matches!(
        engine.evaluate_external_call(&input),
        ExternalCallPolicyDecision::RequireConfirmation(_)
    ));
}

#[test]
fn required_authority_denies_when_caller_lacks_capability() {
    let engine = minimal_policy_with_tools(concat!(
        "  - tool: send_email\n",
        "    side_effect_class: ExternalWrite\n",
        "    required_authority:\n",
        "      - EmailSendCap\n",
        "    default_decision: Allow"
    ));
    let input = make_input_full(
        "send_email",
        vec![],
        vec![],
        AuthoritySet::new(),
        ExecutionContextSummary::new(),
    );
    let decision = engine.evaluate_external_call(&input);
    match &decision {
        ExternalCallPolicyDecision::Deny(e) => assert!(
            e.summary.contains("lacks required authority"),
            "should mention missing authority: {}",
            e.summary
        ),
        other => panic!("expected Deny, got {other:?}"),
    }
}

#[test]
fn required_authority_allows_when_caller_holds_capability() {
    let engine = minimal_policy_with_tools(concat!(
        "  - tool: send_email\n",
        "    side_effect_class: ExternalWrite\n",
        "    required_authority:\n",
        "      - EmailSendCap\n",
        "    default_decision: Allow"
    ));
    let cap = AuthorityCapability::try_from("EmailSendCap").expect("valid capability");
    let input = make_input_full(
        "send_email",
        vec![],
        vec![],
        AuthoritySet::from_iter([cap]),
        ExecutionContextSummary::new(),
    );
    assert!(matches!(
        engine.evaluate_external_call(&input),
        ExternalCallPolicyDecision::Allow(_)
    ));
}

#[test]
fn kwarg_rule_forbids_confidentiality_denies_when_present() {
    let engine = minimal_policy_with_tools(concat!(
        "  - tool: public_log\n",
        "    side_effect_class: ExternalWrite\n",
        "    arg_rules:\n",
        "      - arg: message\n",
        "        forbids_confidentiality:\n",
        "          - AuthSecret\n",
        "    default_decision: Allow"
    ));
    let key = summary_with_integrity(IntegrityLabel::Trusted);
    let val = summary_with_confidentiality(&[DataLabel::AuthSecret]);
    let input = make_input_full(
        "public_log",
        vec![],
        vec![named_kwarg_summary("message", key, val)],
        AuthoritySet::full(),
        ExecutionContextSummary::new(),
    );
    assert!(matches!(
        engine.evaluate_external_call(&input),
        ExternalCallPolicyDecision::Deny(_)
    ));
}

#[test]
fn kwarg_rules_ignore_non_matching_keyword_names() {
    let engine = minimal_policy_with_tools(concat!(
        "  - tool: guarded_write\n",
        "    side_effect_class: ExternalWrite\n",
        "    arg_rules:\n",
        "      - arg: path\n",
        "        requires_integrity: Verified\n",
        "    default_decision: Allow"
    ));
    let input = make_input_full(
        "guarded_write",
        vec![],
        vec![named_kwarg_summary(
            "message",
            summary_with_integrity(IntegrityLabel::Trusted),
            summary_with_integrity(IntegrityLabel::Untrusted),
        )],
        AuthoritySet::full(),
        ExecutionContextSummary::new(),
    );
    assert!(matches!(
        engine.evaluate_external_call(&input),
        ExternalCallPolicyDecision::Allow(_)
    ));
}
