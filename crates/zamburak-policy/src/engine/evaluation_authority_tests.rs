//! Authority-focused unit tests for external-call policy evaluation.

use zamburak_core::AuthorityCapability;
use zamburak_core::trust::AuthoritySet;

use super::ExternalCallPolicyDecision;
use super::helpers::{PolicyInputBuilder, minimal_policy_with_tools};
use crate::engine::evaluation::PolicyDecisionReason;

#[test]
fn required_authority_denies_when_caller_lacks_capability() {
    let engine = minimal_policy_with_tools(concat!(
        "  - tool: send_email\n",
        "    side_effect_class: ExternalWrite\n",
        "    required_authority:\n",
        "      - EmailSendCap\n",
        "    default_decision: Allow"
    ));
    let input = PolicyInputBuilder::new("send_email")
        .caller_authority(AuthoritySet::new())
        .build();
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
    let input = PolicyInputBuilder::new("send_email")
        .caller_authority(AuthoritySet::from_iter([cap]))
        .build();
    assert!(matches!(
        engine.evaluate_external_call(&input),
        ExternalCallPolicyDecision::Allow(_)
    ));
}

#[test]
fn invalid_required_authority_in_policy_denies_with_stable_reason() {
    let engine = minimal_policy_with_tools(concat!(
        "  - tool: send_email\n",
        "    side_effect_class: ExternalWrite\n",
        "    required_authority:\n",
        "      - \"\"\n",
        "    default_decision: Allow"
    ));
    let input = PolicyInputBuilder::new("send_email").build();
    let decision = engine.evaluate_external_call(&input);
    match &decision {
        ExternalCallPolicyDecision::Deny(e) => {
            assert_eq!(e.reason, PolicyDecisionReason::InvalidAuthorityInPolicy)
        }
        other => panic!("expected Deny, got {other:?}"),
    }
}
