//! Unit tests for authority token lifecycle semantics.

use super::test_helpers::*;
use super::{
    AuthorityLifecycleError, AuthorityToken, DelegationRequest, InvalidAuthorityReason,
    IssuerTrust, RevocationIndex, ScopeResource, TokenTimestamp, revalidate_tokens_on_restore,
    validate_tokens_at_policy_boundary,
};
use rstest::rstest;

#[test]
fn mint_rejects_untrusted_issuer() {
    assert_mint_fails(
        MintRequestBuilder::new(TOKEN_NAME_MINT_UNTRUSTED)
            .issuer("remote-agent", IssuerTrust::Untrusted)
            .lifetime(10, 20)
            .build(),
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::UntrustedMinter { issuer } if issuer == "remote-agent"
            )
        },
    );
}

#[test]
fn mint_rejects_non_forward_lifetime() {
    assert_mint_fails(
        MintRequestBuilder::new(TOKEN_NAME_MINT_INVALID)
            .lifetime(15, 15)
            .build(),
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::InvalidTokenLifetime {
                    issued_at: 15,
                    expires_at: 15,
                }
            )
        },
    );
}

#[test]
fn delegation_accepts_strict_scope_and_lifetime_narrowing() {
    let delegated = AuthorityToken::delegate(
        &TOKEN_PARENT,
        delegation_request_with_scope(&TOKEN_ID_CHILD, &SCOPE_SEND_EMAIL, 20, 120),
        &RevocationIndex::default(),
    )
    .expect("strictly narrowed delegations should succeed");

    assert_eq!(delegated.parent_token_id(), Some(TOKEN_PARENT.token_id()));
    assert_eq!(delegated.capability(), TOKEN_PARENT.capability());
}

#[rstest]
#[case::equal_scope(SCOPE_EMAIL_AND_DRAFT.clone())]
#[case::widened_scope(SCOPE_WIDENED.clone())]
fn delegation_rejects_non_strict_scope_subset(#[case] delegated_scope: super::AuthorityScope) {
    let result = AuthorityToken::delegate(
        &TOKEN_PARENT,
        DelegationRequest {
            token_id: TOKEN_ID_CHILD.clone(),
            delegated_by: TEST_DELEGATED_BY.to_owned(),
            subject: SUBJECT_ASSISTANT.clone(),
            scope: delegated_scope,
            delegated_at: TokenTimestamp::new(20),
            expires_at: TokenTimestamp::new(120),
        },
        &RevocationIndex::default(),
    );

    assert!(matches!(
        result,
        Err(AuthorityLifecycleError::DelegationScopeNotStrictSubset)
    ));
}

#[test]
fn delegation_rejects_non_strict_lifetime_subset() {
    assert_delegation_fails(
        &TOKEN_PARENT,
        delegation_request_with_scope(&TOKEN_ID_CHILD, &SCOPE_SEND_EMAIL, 20, 200),
        &RevocationIndex::default(),
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::DelegationLifetimeNotStrictSubset {
                    delegated_expires_at: 200,
                    parent_expires_at: 200,
                }
            )
        },
    );
}

#[test]
fn delegation_rejects_before_parent_issuance() {
    assert_delegation_fails(
        &TOKEN_PARENT,
        delegation_request_with_scope(&TOKEN_ID_CHILD, &SCOPE_SEND_EMAIL, 5, 120),
        &RevocationIndex::default(),
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::DelegationBeforeParentIssuance {
                    delegated_at: 5,
                    parent_issued_at: 10,
                }
            )
        },
    );
}

#[test]
fn policy_boundary_validation_strips_revoked_and_expired_tokens() {
    let mut revocation_index = RevocationIndex::default();
    revocation_index.revoke(TOKEN_ID_REVOKED.clone());

    let validation = validate_tokens_at_policy_boundary(
        &[
            TOKEN_VALID.clone(),
            TOKEN_REVOKED.clone(),
            TOKEN_EXPIRED.clone(),
        ],
        &revocation_index,
        TokenTimestamp::new(150),
    );

    assert_eq!(validation.effective_tokens(), &[TOKEN_VALID.clone()]);
    assert_eq!(validation.invalid_tokens().len(), 2);
    assert!(validation.invalid_tokens().iter().any(|token| {
        token.token_id() == &*TOKEN_ID_REVOKED && token.reason() == InvalidAuthorityReason::Revoked
    }));
    assert!(validation.invalid_tokens().iter().any(|token| {
        token.token_id() == &*TOKEN_ID_EXPIRED && token.reason() == InvalidAuthorityReason::Expired
    }));
}

#[test]
fn policy_boundary_validation_strips_pre_issuance_tokens() {
    let validation = validate_tokens_at_policy_boundary(
        std::slice::from_ref(&*TOKEN_FUTURE),
        &RevocationIndex::default(),
        TokenTimestamp::new(100),
    );

    assert!(validation.effective_tokens().is_empty());
    assert_eq!(validation.invalid_tokens().len(), 1);
    assert_eq!(
        validation.invalid_tokens()[0].reason(),
        InvalidAuthorityReason::PreIssuance
    );
}

#[test]
fn restore_revalidation_matches_policy_boundary_validation() {
    let revocation_index = RevocationIndex::default();
    let boundary = validate_tokens_at_policy_boundary(
        std::slice::from_ref(&*TOKEN_FOR_RESTORE),
        &revocation_index,
        TokenTimestamp::new(120),
    );
    let restored = revalidate_tokens_on_restore(
        std::slice::from_ref(&*TOKEN_FOR_RESTORE),
        &revocation_index,
        TokenTimestamp::new(120),
    );
    assert_eq!(restored, boundary);
}

#[test]
fn domain_types_reject_empty_fields() {
    use super::{AuthorityCapability, AuthoritySubject, AuthorityTokenId};
    let cases: Vec<(&str, Result<(), AuthorityLifecycleError>)> = vec![
        ("token_id", AuthorityTokenId::try_from("").map(|_| ())),
        ("subject", AuthoritySubject::try_from("").map(|_| ())),
        ("capability", AuthorityCapability::try_from("").map(|_| ())),
        ("scope_resource", ScopeResource::try_from("").map(|_| ())),
    ];
    for (expected_field, result) in cases {
        assert!(
            matches!(&result, Err(AuthorityLifecycleError::EmptyField { field }) if *field == expected_field),
            "expected EmptyField({expected_field}), got: {result:?}"
        );
    }
}

#[test]
fn mint_rejects_empty_issuer() {
    assert_mint_fails(
        MintRequestBuilder::new(TOKEN_NAME_MINT_UNTRUSTED)
            .issuer("", IssuerTrust::HostTrusted)
            .lifetime(10, 20)
            .build(),
        |err| matches!(err, AuthorityLifecycleError::EmptyField { field } if *field == "issuer"),
    );
}

#[test]
fn delegation_rejects_empty_delegated_by() {
    let mut req = delegation_request_with_scope(&TOKEN_ID_CHILD, &SCOPE_SEND_EMAIL, 20, 120);
    req.delegated_by = String::new();
    let result = AuthorityToken::delegate(&TOKEN_PARENT, req, &RevocationIndex::default());
    assert!(matches!(
        result,
        Err(AuthorityLifecycleError::EmptyField { field }) if field == "delegated_by"
    ));
}

#[rstest]
#[case::equal(100, 100)]
#[case::reversed(120, 100)]
fn delegation_rejects_invalid_request_lifetime(#[case] delegated_at: u64, #[case] expires_at: u64) {
    assert_delegation_fails(
        &TOKEN_PARENT,
        delegation_request_with_scope(&TOKEN_ID_CHILD, &SCOPE_SEND_EMAIL, delegated_at, expires_at),
        &RevocationIndex::default(),
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::InvalidTokenLifetime { issued_at, .. }
                    if *issued_at == delegated_at
            )
        },
    );
}

#[rstest]
#[case::before_expiry(299, false)]
#[case::at_expiry(300, true)]
#[case::after_expiry(301, true)]
fn is_expired_at_boundary_conditions(#[case] time: u64, #[case] expected_expired: bool) {
    assert_eq!(
        TOKEN_VALID.is_expired_at(TokenTimestamp::new(time)),
        expected_expired
    );
}

#[test]
fn grants_respects_subject_capability_and_scope() {
    let subj = TOKEN_VALID.subject().clone();
    let cap = TOKEN_VALID.capability().clone();
    let res = ScopeResource::try_from("send_email").expect("valid resource");
    assert!(TOKEN_VALID.grants(&subj, &cap, &res));
    assert!(!TOKEN_VALID.grants(&subject("other"), &cap, &res));
    assert!(!TOKEN_VALID.grants(&subj, &capability("OtherCap"), &res));
    let out = ScopeResource::try_from("calendar_write").expect("valid resource");
    assert!(!TOKEN_VALID.grants(&subj, &cap, &out));
}
