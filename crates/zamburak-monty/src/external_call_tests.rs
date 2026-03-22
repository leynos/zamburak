//! Unit tests for external-call mediation traits and built-in mediators.

use monty::ExternalCallKind;
use rstest::rstest;

use crate::external_call::{
    AllowAllMediator, CallContext, ConfirmationContext, DenyAllMediator, ExternalCallMediator,
    MediationDecision,
};

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
