//! Unit tests for authority token lifecycle semantics.

use super::{
    AuthorityCapability, AuthorityLifecycleError, AuthorityScope, AuthoritySubject, AuthorityToken,
    AuthorityTokenId, DelegationRequest, InvalidAuthorityReason, IssuerTrust, MintRequest,
    RevocationIndex, ScopeResource, TokenTimestamp, revalidate_tokens_on_restore,
    validate_tokens_at_policy_boundary,
};
use rstest::rstest;

fn token_id(value: &str) -> AuthorityTokenId {
    AuthorityTokenId::try_from(value).expect("token ids used in tests are valid")
}

fn subject(value: &str) -> AuthoritySubject {
    AuthoritySubject::try_from(value).expect("subjects used in tests are valid")
}

fn capability(value: &str) -> AuthorityCapability {
    AuthorityCapability::try_from(value).expect("capabilities used in tests are valid")
}

fn scope(resources: &[&str]) -> AuthorityScope {
    let parsed_resources = resources
        .iter()
        .map(|resource| ScopeResource::try_from(*resource).expect("scope resources are valid"));
    AuthorityScope::new(parsed_resources).expect("scope fixtures are valid")
}

fn mint_authority_token(
    token_name: &str,
    scope_resources: &[&str],
    issued_at: u64,
    expires_at: u64,
) -> AuthorityToken {
    AuthorityToken::mint(MintRequest {
        token_id: token_id(token_name),
        issuer: "policy-host".to_owned(),
        issuer_trust: IssuerTrust::HostTrusted,
        subject: subject("assistant"),
        capability: capability("EmailSendCap"),
        scope: scope(scope_resources),
        issued_at: TokenTimestamp::new(issued_at),
        expires_at: TokenTimestamp::new(expires_at),
    })
    .expect("mint fixtures are valid")
}

/// Helper to assert that a mint request fails with a specific error predicate.
fn assert_mint_fails<F>(request: MintRequest, predicate: F, error_desc: &str)
where
    F: Fn(&AuthorityLifecycleError) -> bool,
{
    let result = AuthorityToken::mint(request);
    match result {
        Err(ref err) if predicate(err) => {} // success
        _ => panic!("expected mint to fail with {error_desc}, got: {result:?}"),
    }
}

/// Helper to create a delegation request for testing.
fn delegation_request(
    child_name: &str,
    scope_resources: &[&str],
    delegated_at: u64,
    expires_at: u64,
) -> DelegationRequest {
    DelegationRequest {
        token_id: token_id(child_name),
        delegated_by: "policy-host".to_owned(),
        subject: subject("assistant"),
        scope: scope(scope_resources),
        delegated_at: TokenTimestamp::new(delegated_at),
        expires_at: TokenTimestamp::new(expires_at),
    }
}

/// Helper to assert that a delegation request fails with a specific error predicate.
fn assert_delegation_fails<F>(
    parent: &AuthorityToken,
    request: DelegationRequest,
    revocation_index: &RevocationIndex,
    predicate: F,
    error_desc: &str,
) where
    F: Fn(&AuthorityLifecycleError) -> bool,
{
    let result = AuthorityToken::delegate(parent, request, revocation_index);
    match result {
        Err(ref err) if predicate(err) => {} // success
        _ => panic!("expected delegation to fail with {error_desc}, got: {result:?}"),
    }
}

#[test]
fn mint_rejects_untrusted_issuer() {
    assert_mint_fails(
        MintRequest {
            token_id: token_id("mint-untrusted"),
            issuer: "remote-agent".to_owned(),
            issuer_trust: IssuerTrust::Untrusted,
            subject: subject("assistant"),
            capability: capability("EmailSendCap"),
            scope: scope(&["send_email"]),
            issued_at: TokenTimestamp::new(10),
            expires_at: TokenTimestamp::new(20),
        },
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::UntrustedMinter { issuer } if issuer == "remote-agent"
            )
        },
        "UntrustedMinter",
    );
}

#[test]
fn mint_rejects_non_forward_lifetime() {
    assert_mint_fails(
        MintRequest {
            token_id: token_id("mint-invalid-lifetime"),
            issuer: "policy-host".to_owned(),
            issuer_trust: IssuerTrust::HostTrusted,
            subject: subject("assistant"),
            capability: capability("EmailSendCap"),
            scope: scope(&["send_email"]),
            issued_at: TokenTimestamp::new(15),
            expires_at: TokenTimestamp::new(15),
        },
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::InvalidTokenLifetime {
                    issued_at: 15,
                    expires_at: 15,
                }
            )
        },
        "InvalidTokenLifetime",
    );
}

#[test]
fn delegation_accepts_strict_scope_and_lifetime_narrowing() {
    let parent = mint_authority_token("parent", &["send_email", "send_email_draft"], 10, 200);

    let delegated = AuthorityToken::delegate(
        &parent,
        delegation_request("child", &["send_email"], 20, 120),
        &RevocationIndex::default(),
    )
    .expect("strictly narrowed delegations should succeed");

    assert_eq!(delegated.parent_token_id(), Some(parent.token_id()));
    assert_eq!(delegated.capability(), parent.capability());
}

#[rstest]
#[case::equal_scope(scope(&["send_email", "send_email_draft"]))]
#[case::widened_scope(scope(&["send_email", "send_email_draft", "calendar_write"]))]
fn delegation_rejects_non_strict_scope_subset(#[case] delegated_scope: AuthorityScope) {
    let parent = mint_authority_token("parent", &["send_email", "send_email_draft"], 10, 200);

    let result = AuthorityToken::delegate(
        &parent,
        DelegationRequest {
            token_id: token_id("child"),
            delegated_by: "policy-host".to_owned(),
            subject: subject("assistant"),
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
    let parent = mint_authority_token("parent", &["send_email", "send_email_draft"], 10, 200);

    assert_delegation_fails(
        &parent,
        delegation_request("child", &["send_email"], 20, 200),
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
        "DelegationLifetimeNotStrictSubset",
    );
}

#[test]
fn policy_boundary_validation_strips_revoked_and_expired_tokens() {
    let valid = mint_authority_token("valid", &["send_email"], 10, 300);
    let revoked = mint_authority_token("revoked", &["send_email"], 10, 300);
    let expired = mint_authority_token("expired", &["send_email"], 10, 100);

    let mut revocation_index = RevocationIndex::default();
    revocation_index.revoke(revoked.token_id().clone());

    let validation = validate_tokens_at_policy_boundary(
        &[valid.clone(), revoked.clone(), expired.clone()],
        &revocation_index,
        TokenTimestamp::new(150),
    );

    assert_eq!(validation.effective_tokens(), &[valid]);
    assert_eq!(validation.invalid_tokens().len(), 2);
    assert!(validation.invalid_tokens().iter().any(|token| {
        token.token_id() == revoked.token_id() && token.reason() == InvalidAuthorityReason::Revoked
    }));
    assert!(validation.invalid_tokens().iter().any(|token| {
        token.token_id() == expired.token_id() && token.reason() == InvalidAuthorityReason::Expired
    }));
}

#[test]
fn restore_revalidation_matches_policy_boundary_validation() {
    let token = mint_authority_token("token", &["send_email"], 10, 100);
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
}
