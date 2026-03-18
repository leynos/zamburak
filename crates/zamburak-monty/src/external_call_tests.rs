//! Unit tests for external-call mediation traits and built-in mediators.

use monty::ExternalCallKind;
use rstest::rstest;

use crate::external_call::{
    AllowAllMediator, CallContext, DenyAllMediator, ExternalCallMediator, MediationDecision,
};

fn function_call_context(call_id: u32, name: &str) -> CallContext {
    CallContext {
        call_id,
        kind: ExternalCallKind::Function,
        function_name: name.to_owned(),
    }
}

fn os_call_context(call_id: u32, name: &str) -> CallContext {
    CallContext {
        call_id,
        kind: ExternalCallKind::Os,
        function_name: name.to_owned(),
    }
}

#[rstest]
fn allow_all_mediator_returns_allow_for_function_call() {
    let mut mediator = AllowAllMediator;
    let ctx = function_call_context(1, "print");
    assert_eq!(mediator.mediate(&ctx), MediationDecision::Allow);
}

#[rstest]
fn allow_all_mediator_returns_allow_for_os_call() {
    let mut mediator = AllowAllMediator;
    let ctx = os_call_context(2, "open");
    assert_eq!(mediator.mediate(&ctx), MediationDecision::Allow);
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
    };
    assert!(matches!(
        mediator.mediate(&ctx),
        MediationDecision::Deny { .. }
    ));
}
