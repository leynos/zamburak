//! Unit tests for authority token lifecycle semantics.

use super::test_helpers::*;
use super::{
    AuthorityLifecycleError, AuthorityToken, DelegationRequest, InvalidAuthorityReason,
    IssuerTrust, RevocationIndex, ScopeResource, TokenTimestamp, revalidate_tokens_on_restore,
    validate_tokens_at_policy_boundary,
};
use rstest::rstest;

#[rstest]
fn mint_rejects_untrusted_issuer() -> Result<(), AuthorityLifecycleError> {
    assert_mint_fails(
        MintRequestBuilder::new(TOKEN_NAME_MINT_UNTRUSTED)?
            .issuer("remote-agent", IssuerTrust::Untrusted)?
            .lifetime(10, 20)
            .build()?,
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::UntrustedMinter { issuer } if issuer.as_str() == "remote-agent"
            )
        },
    );
    Ok(())
}

#[rstest]
fn mint_rejects_non_forward_lifetime() -> Result<(), AuthorityLifecycleError> {
    assert_mint_fails(
        MintRequestBuilder::new(TOKEN_NAME_MINT_INVALID)?
            .lifetime(15, 15)
            .build()?,
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
    Ok(())
}

#[rstest]
fn delegation_accepts_strict_scope_and_lifetime_narrowing(
    token_parent: Result<AuthorityToken, AuthorityLifecycleError>,
    scope_send_email: Result<super::AuthorityScope, AuthorityLifecycleError>,
) -> Result<(), AuthorityLifecycleError> {
    let parent = token_parent?;
    let child_id = token_id(TOKEN_NAME_CHILD)?;
    let timing = DelegationTiming {
        delegated_at: 20,
        expires_at: 120,
    };
    let delegated = AuthorityToken::delegate(
        &parent,
        delegation_request_with_scope(&child_id, &scope_send_email?, &timing)?,
        &RevocationIndex::default(),
    )?;

    assert_eq!(delegated.parent_token_id(), Some(parent.token_id()));
    assert_eq!(delegated.capability(), parent.capability());
    Ok(())
}

#[rstest]
#[case::equal_scope(false)]
#[case::widened_scope(true)]
fn delegation_rejects_non_strict_scope_subset(
    token_parent: Result<AuthorityToken, AuthorityLifecycleError>,
    scope_email_and_draft: Result<super::AuthorityScope, AuthorityLifecycleError>,
    scope_widened: Result<super::AuthorityScope, AuthorityLifecycleError>,
    #[case] use_widened: bool,
) -> Result<(), AuthorityLifecycleError> {
    let delegated_scope = if use_widened {
        scope_widened?
    } else {
        scope_email_and_draft?
    };
    let child_id = token_id(TOKEN_NAME_CHILD)?;
    let timing = DelegationTiming {
        delegated_at: 20,
        expires_at: 120,
    };
    let result = AuthorityToken::delegate(
        &token_parent?,
        DelegationRequest {
            token_id: child_id,
            delegated_by: issuer(TEST_DELEGATED_BY)?,
            subject: subject(TEST_SUBJECT)?,
            scope: delegated_scope,
            delegated_at: TokenTimestamp::new(timing.delegated_at),
            expires_at: TokenTimestamp::new(timing.expires_at),
        },
        &RevocationIndex::default(),
    );

    assert!(matches!(
        result,
        Err(AuthorityLifecycleError::DelegationScopeNotStrictSubset)
    ));
    Ok(())
}

#[rstest]
#[case::non_strict_lifetime(20, 200, |err: &AuthorityLifecycleError| {
    matches!(
        err,
        AuthorityLifecycleError::DelegationLifetimeNotStrictSubset {
            delegated_expires_at: 200,
            parent_expires_at: 200,
        }
    )
})]
#[case::before_parent_issuance(5, 120, |err: &AuthorityLifecycleError| {
    matches!(
        err,
        AuthorityLifecycleError::DelegationBeforeParentIssuance {
            delegated_at: 5,
            parent_issued_at: 10,
        }
    )
})]
fn delegation_rejects_invalid_timing<F>(
    token_parent: Result<AuthorityToken, AuthorityLifecycleError>,
    scope_send_email: Result<super::AuthorityScope, AuthorityLifecycleError>,
    #[case] delegated_at: u64,
    #[case] expires_at: u64,
    #[case] predicate: F,
) -> Result<(), AuthorityLifecycleError>
where
    F: Fn(&AuthorityLifecycleError) -> bool,
{
    let child_id = token_id(TOKEN_NAME_CHILD)?;
    let timing = DelegationTiming {
        delegated_at,
        expires_at,
    };
    assert_delegation_fails(
        &token_parent?,
        delegation_request_with_scope(&child_id, &scope_send_email?, &timing)?,
        &RevocationIndex::default(),
        predicate,
    );
    Ok(())
}

#[rstest]
fn policy_boundary_validation_strips_revoked_and_expired_tokens(
    token_valid: Result<AuthorityToken, AuthorityLifecycleError>,
    token_revoked: Result<AuthorityToken, AuthorityLifecycleError>,
    token_expired: Result<AuthorityToken, AuthorityLifecycleError>,
) -> Result<(), AuthorityLifecycleError> {
    let valid = token_valid?;
    let revoked = token_revoked?;
    let expired = token_expired?;

    let mut revocation_index = RevocationIndex::default();
    revocation_index.revoke(revoked.token_id().clone());

    let revoked_id = revoked.token_id().clone();
    let expired_id = expired.token_id().clone();
    let validation = validate_tokens_at_policy_boundary(
        &[valid.clone(), revoked, expired],
        &revocation_index,
        TokenTimestamp::new(150),
    );

    assert_eq!(validation.effective_tokens(), &[valid]);
    assert_eq!(validation.invalid_tokens().len(), 2);
    assert!(
        validation.invalid_tokens().iter().any(|t| {
            t.token_id() == &revoked_id && t.reason() == InvalidAuthorityReason::Revoked
        })
    );
    assert!(
        validation.invalid_tokens().iter().any(|t| {
            t.token_id() == &expired_id && t.reason() == InvalidAuthorityReason::Expired
        })
    );
    Ok(())
}

#[rstest]
fn policy_boundary_validation_strips_pre_issuance_tokens(
    token_future: Result<AuthorityToken, AuthorityLifecycleError>,
) -> Result<(), AuthorityLifecycleError> {
    let future = token_future?;
    let validation = validate_tokens_at_policy_boundary(
        std::slice::from_ref(&future),
        &RevocationIndex::default(),
        TokenTimestamp::new(100),
    );

    assert!(validation.effective_tokens().is_empty());
    assert_eq!(validation.invalid_tokens().len(), 1);
    assert_eq!(
        validation.invalid_tokens()[0].reason(),
        InvalidAuthorityReason::PreIssuance
    );
    Ok(())
}

#[rstest]
fn restore_revalidation_matches_policy_boundary_validation(
    token_for_restore: Result<AuthorityToken, AuthorityLifecycleError>,
) -> Result<(), AuthorityLifecycleError> {
    let token = token_for_restore?;
    let revocation_index = RevocationIndex::default();
    let boundary = validate_tokens_at_policy_boundary(
        std::slice::from_ref(&token),
        &revocation_index,
        TokenTimestamp::new(120),
    );
    let restored = revalidate_tokens_on_restore(
        std::slice::from_ref(&token),
        &revocation_index,
        TokenTimestamp::new(120),
    );
    assert_eq!(restored, boundary);
    Ok(())
}

#[test]
fn domain_types_reject_empty_fields() {
    use super::{AuthorityCapability, AuthorityIssuer, AuthoritySubject, AuthorityTokenId};
    let cases: Vec<(&str, Result<(), AuthorityLifecycleError>)> = vec![
        ("token_id", AuthorityTokenId::try_from("").map(|_| ())),
        ("issuer", AuthorityIssuer::try_from("").map(|_| ())),
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

#[rstest]
#[case::equal(100, 100)]
#[case::reversed(120, 100)]
fn delegation_rejects_invalid_request_lifetime(
    token_parent: Result<AuthorityToken, AuthorityLifecycleError>,
    scope_send_email: Result<super::AuthorityScope, AuthorityLifecycleError>,
    #[case] delegated_at: u64,
    #[case] expires_at: u64,
) -> Result<(), AuthorityLifecycleError> {
    let child_id = token_id(TOKEN_NAME_CHILD)?;
    let timing = DelegationTiming {
        delegated_at,
        expires_at,
    };
    assert_delegation_fails(
        &token_parent?,
        delegation_request_with_scope(&child_id, &scope_send_email?, &timing)?,
        &RevocationIndex::default(),
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::InvalidTokenLifetime { issued_at, .. }
                    if *issued_at == delegated_at
            )
        },
    );
    Ok(())
}

#[rstest]
#[case::before_expiry(299, false)]
#[case::at_expiry(300, true)]
#[case::after_expiry(301, true)]
fn is_expired_at_boundary_conditions(
    token_valid: Result<AuthorityToken, AuthorityLifecycleError>,
    #[case] time: u64,
    #[case] expected_expired: bool,
) -> Result<(), AuthorityLifecycleError> {
    assert_eq!(
        token_valid?.is_expired_at(TokenTimestamp::new(time)),
        expected_expired
    );
    Ok(())
}

#[rstest]
fn grants_respects_subject_capability_and_scope(
    token_valid: Result<AuthorityToken, AuthorityLifecycleError>,
) -> Result<(), AuthorityLifecycleError> {
    let valid = token_valid?;
    let subj = valid.subject().clone();
    let cap = valid.capability().clone();
    let res = ScopeResource::try_from("send_email")?;
    assert!(valid.grants(&subj, &cap, &res));
    assert!(!valid.grants(&subject("other")?, &cap, &res));
    assert!(!valid.grants(&subj, &capability("OtherCap")?, &res));
    let out = ScopeResource::try_from("calendar_write")?;
    assert!(!valid.grants(&subj, &cap, &out));
    Ok(())
}
