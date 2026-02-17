//! Helper types and functions shared by authority lifecycle BDD steps.

use zamburak_core::{
    AuthorityBoundaryValidation, AuthorityCapability, AuthorityIssuer, AuthorityLifecycleError,
    AuthorityScope, AuthoritySubject, AuthorityToken, AuthorityTokenId, DelegationRequest,
    InvalidAuthorityReason, IssuerTrust, MintRequest, ScopeResource, TokenTimestamp,
};

use super::LifecycleWorld;

// ── Pure constructors ────────────────────────────────────────────────

pub fn make_token_id(name: &str) -> AuthorityTokenId {
    let Ok(id) = AuthorityTokenId::try_from(name) else {
        panic!("test token id '{name}' is invalid");
    };
    id
}

pub fn make_issuer(name: &str) -> AuthorityIssuer {
    let Ok(issuer) = AuthorityIssuer::try_from(name) else {
        panic!("test issuer '{name}' is invalid");
    };
    issuer
}

pub fn make_subject(name: &str) -> AuthoritySubject {
    let Ok(subject) = AuthoritySubject::try_from(name) else {
        panic!("test subject '{name}' is invalid");
    };
    subject
}

pub fn make_capability(name: &str) -> AuthorityCapability {
    let Ok(capability) = AuthorityCapability::try_from(name) else {
        panic!("test capability '{name}' is invalid");
    };
    capability
}

pub fn make_scope(resources: &[&str]) -> AuthorityScope {
    let parsed: Vec<ScopeResource> = resources
        .iter()
        .map(|r| {
            let Ok(resource) = ScopeResource::try_from(*r) else {
                panic!("test scope resource '{r}' is invalid");
            };
            resource
        })
        .collect();
    let Ok(scope) = AuthorityScope::new(parsed) else {
        panic!("test scope is invalid");
    };
    scope
}

pub fn mint_fixture(
    token_name: &str,
    scope_resources: &[&str],
    issued_at: u64,
    expires_at: u64,
) -> AuthorityToken {
    let Ok(token) = AuthorityToken::mint(MintRequest {
        token_id: make_token_id(token_name),
        issuer: make_issuer("policy-host"),
        issuer_trust: IssuerTrust::HostTrusted,
        subject: make_subject("assistant"),
        capability: make_capability("EmailSendCap"),
        scope: make_scope(scope_resources),
        issued_at: TokenTimestamp::new(issued_at),
        expires_at: TokenTimestamp::new(expires_at),
    }) else {
        panic!("mint fixture '{token_name}' is invalid");
    };
    token
}

// ── Result extractors ────────────────────────────────────────────────

pub fn require_mint_result(
    world: &LifecycleWorld,
) -> &Result<AuthorityToken, AuthorityLifecycleError> {
    let Some(result) = world.mint_result.as_ref() else {
        panic!("mint step must run before assertion");
    };
    result
}

pub fn require_delegation_result(
    world: &LifecycleWorld,
) -> &Result<AuthorityToken, AuthorityLifecycleError> {
    let Some(result) = world.delegation_result.as_ref() else {
        panic!("delegation step must run before assertion");
    };
    result
}

pub fn require_boundary_result(world: &LifecycleWorld) -> &AuthorityBoundaryValidation {
    let Some(result) = world.boundary_result.as_ref() else {
        panic!("boundary validation step must run before assertion");
    };
    result
}

// ── Assertion helpers ────────────────────────────────────────────────

/// Generic assertion for mint result error variants.
pub fn assert_mint_error<F>(world: &LifecycleWorld, predicate: F, error_desc: &str)
where
    F: Fn(&AuthorityLifecycleError) -> bool,
{
    let result = require_mint_result(world);
    match result {
        Err(err) if predicate(err) => {} // success
        _ => panic!("expected {error_desc}, got: {result:?}"),
    }
}

/// Generic assertion for delegation result error variants.
pub fn assert_delegation_error<F>(world: &LifecycleWorld, predicate: F, error_desc: &str)
where
    F: Fn(&AuthorityLifecycleError) -> bool,
{
    let result = require_delegation_result(world);
    match result {
        Err(err) if predicate(err) => {} // success
        _ => panic!("expected {error_desc}, got: {result:?}"),
    }
}

/// Assert that a token with the expected invalid ID is present in the
/// invalid tokens list with the specified reason.
pub fn assert_invalid_token_stripped(
    world: &LifecycleWorld,
    expected_reason: InvalidAuthorityReason,
    reason_desc: &str,
) {
    let validation = require_boundary_result(world);
    let Some(invalid_id) = world.expected_invalid_id.as_ref() else {
        panic!("expected invalid id must be set");
    };

    assert!(
        validation
            .invalid_tokens()
            .iter()
            .any(|t| t.token_id() == invalid_id && t.reason() == expected_reason),
        "expected {reason_desc} token to be stripped"
    );
}

// ── Specific assertion helpers ────────────────────────────────────────

/// Assert that the mint result is `UntrustedMinter`.
pub fn assert_mint_rejected_untrusted(world: &LifecycleWorld) {
    assert_mint_error(
        world,
        |err| matches!(err, AuthorityLifecycleError::UntrustedMinter { .. }),
        "UntrustedMinter",
    );
}

/// Assert that the mint result is `InvalidTokenLifetime`.
pub fn assert_mint_rejected_lifetime(world: &LifecycleWorld) {
    assert_mint_error(
        world,
        |err| matches!(err, AuthorityLifecycleError::InvalidTokenLifetime { .. }),
        "InvalidTokenLifetime",
    );
}

/// Assert that the delegation result is `DelegationScopeNotStrictSubset`.
pub fn assert_delegation_rejected_scope(world: &LifecycleWorld) {
    assert_delegation_error(
        world,
        |err| matches!(err, AuthorityLifecycleError::DelegationScopeNotStrictSubset),
        "DelegationScopeNotStrictSubset",
    );
}

/// Assert that the delegation result is `DelegationLifetimeNotStrictSubset`.
pub fn assert_delegation_rejected_lifetime(world: &LifecycleWorld) {
    assert_delegation_error(
        world,
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::DelegationLifetimeNotStrictSubset { .. }
            )
        },
        "DelegationLifetimeNotStrictSubset",
    );
}

/// Assert that the delegation result is `InvalidParentToken` with `Revoked`.
pub fn assert_delegation_rejected_revoked_parent(world: &LifecycleWorld) {
    assert_delegation_error(
        world,
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::InvalidParentToken {
                    reason: InvalidAuthorityReason::Revoked,
                    ..
                }
            )
        },
        "InvalidParentToken(Revoked)",
    );
}

/// Assert that the delegation result is `InvalidParentToken` with `Expired`.
pub fn assert_delegation_rejected_expired_parent(world: &LifecycleWorld) {
    assert_delegation_error(
        world,
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::InvalidParentToken {
                    reason: InvalidAuthorityReason::Expired,
                    ..
                }
            )
        },
        "InvalidParentToken(Expired)",
    );
}

/// Assert that the expected valid token remains in the effective set.
pub fn assert_valid_token_remains(world: &LifecycleWorld) {
    let validation = require_boundary_result(world);
    let Some(valid_id) = world.expected_valid_id.as_ref() else {
        panic!("expected valid id must be set");
    };
    assert!(
        validation
            .effective_tokens()
            .iter()
            .any(|t| t.token_id() == valid_id),
        "expected valid token to remain in effective set"
    );
}

/// Assert that no tokens remain in the effective set.
pub fn assert_no_effective_tokens(world: &LifecycleWorld) {
    let validation = require_boundary_result(world);
    assert!(
        validation.effective_tokens().is_empty(),
        "expected no effective tokens, found: {:?}",
        validation.effective_tokens()
    );
}

// ── Issuer configuration ─────────────────────────────────────────────

/// Configuration for the issuer of a mint request.
#[derive(Clone, Copy)]
pub enum IssuerConfig {
    /// Host-trusted issuer (policy-host).
    HostTrusted,
    /// Untrusted issuer (remote-agent).
    Untrusted,
}

impl IssuerConfig {
    /// Returns the issuer identity for this configuration.
    pub fn issuer(self) -> AuthorityIssuer {
        make_issuer(match self {
            Self::HostTrusted => "policy-host",
            Self::Untrusted => "remote-agent",
        })
    }

    /// Returns the issuer trust level for this configuration.
    pub const fn issuer_trust(self) -> IssuerTrust {
        match self {
            Self::HostTrusted => IssuerTrust::HostTrusted,
            Self::Untrusted => IssuerTrust::Untrusted,
        }
    }
}

// ── World-mutating helpers ───────────────────────────────────────────

/// Create a mint request with configurable issuer trust.
pub fn create_mint_request(
    world: &mut LifecycleWorld,
    subject: &str,
    capability: &str,
    issuer_config: IssuerConfig,
) {
    world.mint_request = Some(MintRequest {
        token_id: make_token_id("mint-token"),
        issuer: issuer_config.issuer(),
        issuer_trust: issuer_config.issuer_trust(),
        subject: make_subject(subject),
        capability: make_capability(capability),
        scope: make_scope(&["placeholder"]),
        issued_at: TokenTimestamp::new(0),
        expires_at: TokenTimestamp::new(1),
    });
}

/// Create a token set with one valid and one revoked token.
pub fn create_token_set_with_revoked(world: &mut LifecycleWorld) {
    let valid = mint_fixture("valid-tok", &["send_email"], 10, 1000);
    let revoked = mint_fixture("revoked-tok", &["send_email"], 10, 1000);

    world.revocation_index.revoke(revoked.token_id().clone());

    world.expected_valid_id = Some(valid.token_id().clone());
    world.expected_invalid_id = Some(revoked.token_id().clone());
    world.token_set = vec![valid, revoked];
}

/// Create a token set with one valid and one expired token.
pub fn create_token_set_with_expired(world: &mut LifecycleWorld) {
    let valid = mint_fixture("valid-tok", &["send_email"], 10, 1000);
    let expired = mint_fixture("expired-tok", &["send_email"], 10, 100);

    world.expected_valid_id = Some(valid.token_id().clone());
    world.expected_invalid_id = Some(expired.token_id().clone());
    world.token_set = vec![valid, expired];
}

/// Create a delegation request with given scope resources.
pub fn create_delegation_request(world: &mut LifecycleWorld, resources: &[&str], expires_at: u64) {
    create_delegation_request_at(world, resources, 20, expires_at);
}

/// Create a delegation request with custom delegation time.
pub fn create_delegation_request_at(
    world: &mut LifecycleWorld,
    resources: &[&str],
    delegated_at: u64,
    expires_at: u64,
) {
    world.delegation_request = Some(DelegationRequest {
        token_id: make_token_id("child"),
        delegated_by: make_issuer("policy-host"),
        subject: make_subject("assistant"),
        scope: make_scope(resources),
        delegated_at: TokenTimestamp::new(delegated_at),
        expires_at: TokenTimestamp::new(expires_at),
    });
}
