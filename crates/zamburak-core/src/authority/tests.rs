//! Unit tests for authority token lifecycle semantics.

use super::{
    AuthorityCapability, AuthorityLifecycleError, AuthorityScope, AuthoritySubject, AuthorityToken,
    AuthorityTokenId, DelegationRequest, InvalidAuthorityReason, IssuerTrust, MintRequest,
    RevocationIndex, ScopeResource, TokenTimestamp, revalidate_tokens_on_restore,
    validate_tokens_at_policy_boundary,
};
use rstest::rstest;

// Test fixture constants to reduce string parameter repetition.
const TEST_ISSUER: &str = "policy-host";
const TEST_SUBJECT: &str = "assistant";
const TEST_CAPABILITY: &str = "EmailSendCap";
const TEST_DELEGATED_BY: &str = "policy-host";

// Token name constants to reduce string literal usage.
const TOKEN_NAME_PARENT: &str = "parent";
const TOKEN_NAME_CHILD: &str = "child";
const TOKEN_NAME_VALID: &str = "valid";
const TOKEN_NAME_REVOKED: &str = "revoked";
const TOKEN_NAME_EXPIRED: &str = "expired";
const TOKEN_NAME_TOKEN: &str = "token";
const TOKEN_NAME_MINT_UNTRUSTED: &str = "mint-untrusted";
const TOKEN_NAME_MINT_INVALID: &str = "mint-invalid-lifetime";

// Pre-built domain type constants to reduce string parameter usage.
lazy_static::lazy_static! {
    static ref TOKEN_ID_PARENT: AuthorityTokenId = token_id(TOKEN_NAME_PARENT);
    static ref TOKEN_ID_CHILD: AuthorityTokenId = token_id(TOKEN_NAME_CHILD);
    static ref TOKEN_ID_VALID: AuthorityTokenId = token_id(TOKEN_NAME_VALID);
    static ref TOKEN_ID_REVOKED: AuthorityTokenId = token_id(TOKEN_NAME_REVOKED);
    static ref TOKEN_ID_EXPIRED: AuthorityTokenId = token_id(TOKEN_NAME_EXPIRED);
    static ref TOKEN_ID_TOKEN: AuthorityTokenId = token_id(TOKEN_NAME_TOKEN);
    static ref SCOPE_SEND_EMAIL: AuthorityScope = scope(&["send_email"]);
    static ref SCOPE_EMAIL_AND_DRAFT: AuthorityScope = scope(&["send_email", "send_email_draft"]);
    static ref SUBJECT_ASSISTANT: AuthoritySubject = subject(TEST_SUBJECT);
    static ref SCOPE_WIDENED: AuthorityScope =
        scope(&["send_email", "send_email_draft", "calendar_write"]);

    // Pre-built test tokens using typed helper to reduce string parameters.
    static ref TOKEN_PARENT: AuthorityToken =
        mint_authority_token_with_types(TOKEN_ID_PARENT.clone(), SCOPE_EMAIL_AND_DRAFT.clone(), 10, 200);
    static ref TOKEN_VALID: AuthorityToken =
        mint_authority_token_with_types(TOKEN_ID_VALID.clone(), SCOPE_SEND_EMAIL.clone(), 10, 300);
    static ref TOKEN_REVOKED: AuthorityToken =
        mint_authority_token_with_types(TOKEN_ID_REVOKED.clone(), SCOPE_SEND_EMAIL.clone(), 10, 300);
    static ref TOKEN_EXPIRED: AuthorityToken =
        mint_authority_token_with_types(TOKEN_ID_EXPIRED.clone(), SCOPE_SEND_EMAIL.clone(), 10, 100);
    static ref TOKEN_FOR_RESTORE: AuthorityToken =
        mint_authority_token_with_types(TOKEN_ID_TOKEN.clone(), SCOPE_SEND_EMAIL.clone(), 10, 100);
}

/// Builder for mint requests with sensible test defaults.
struct MintRequestBuilder {
    token_name: String,
    issuer: String,
    issuer_trust: IssuerTrust,
    subject_name: String,
    capability_name: String,
    scope_resources: Vec<String>,
    issued_at: u64,
    expires_at: u64,
}

impl MintRequestBuilder {
    fn new(token_name: &str) -> Self {
        Self {
            token_name: token_name.to_owned(),
            issuer: TEST_ISSUER.to_owned(),
            issuer_trust: IssuerTrust::HostTrusted,
            subject_name: TEST_SUBJECT.to_owned(),
            capability_name: TEST_CAPABILITY.to_owned(),
            scope_resources: vec!["send_email".to_owned()],
            issued_at: 0,
            expires_at: 100,
        }
    }

    fn issuer(mut self, issuer: &str, trust: IssuerTrust) -> Self {
        self.issuer = issuer.to_owned();
        self.issuer_trust = trust;
        self
    }

    fn lifetime(mut self, issued_at: u64, expires_at: u64) -> Self {
        self.issued_at = issued_at;
        self.expires_at = expires_at;
        self
    }

    fn build(self) -> MintRequest {
        let scope_refs: Vec<&str> = self.scope_resources.iter().map(String::as_str).collect();
        MintRequest {
            token_id: token_id(&self.token_name),
            issuer: self.issuer,
            issuer_trust: self.issuer_trust,
            subject: subject(&self.subject_name),
            capability: capability(&self.capability_name),
            scope: scope(&scope_refs),
            issued_at: TokenTimestamp::new(self.issued_at),
            expires_at: TokenTimestamp::new(self.expires_at),
        }
    }
}

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

/// Helper to mint an authority token using pre-built domain types.
fn mint_authority_token_with_types(
    token_id: AuthorityTokenId,
    scope: AuthorityScope,
    issued_at: u64,
    expires_at: u64,
) -> AuthorityToken {
    AuthorityToken::mint(MintRequest {
        token_id,
        issuer: TEST_ISSUER.to_owned(),
        issuer_trust: IssuerTrust::HostTrusted,
        subject: subject(TEST_SUBJECT),
        capability: capability(TEST_CAPABILITY),
        scope,
        issued_at: TokenTimestamp::new(issued_at),
        expires_at: TokenTimestamp::new(expires_at),
    })
    .expect("mint fixtures are valid")
}

/// Helper to assert that a mint request fails with a specific error predicate.
fn assert_mint_fails<F>(request: MintRequest, predicate: F)
where
    F: Fn(&AuthorityLifecycleError) -> bool,
{
    let result = AuthorityToken::mint(request);
    match result {
        Err(ref err) if predicate(err) => {} // success
        _ => panic!("mint request did not fail as expected, got: {result:?}"),
    }
}

/// Helper to create a delegation request for testing with pre-built domain types.
fn delegation_request_with_scope(
    child_id: &AuthorityTokenId,
    scope: &AuthorityScope,
    delegated_at: u64,
    expires_at: u64,
) -> DelegationRequest {
    DelegationRequest {
        token_id: child_id.clone(),
        delegated_by: TEST_DELEGATED_BY.to_owned(),
        subject: SUBJECT_ASSISTANT.clone(),
        scope: scope.clone(),
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
) where
    F: Fn(&AuthorityLifecycleError) -> bool,
{
    let result = AuthorityToken::delegate(parent, request, revocation_index);
    match result {
        Err(ref err) if predicate(err) => {} // success
        _ => panic!("delegation request did not fail as expected, got: {result:?}"),
    }
}

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
fn delegation_rejects_non_strict_scope_subset(#[case] delegated_scope: AuthorityScope) {
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
