//! Shared constants, fixtures, and assertion helpers for authority tests.

use rstest::fixture;

use super::{
    AuthorityCapability, AuthorityIssuer, AuthorityLifecycleError, AuthorityScope,
    AuthoritySubject, AuthorityToken, AuthorityTokenId, DelegationRequest, IssuerTrust,
    MintRequest, RevocationIndex, ScopeResource, TokenTimestamp,
};

pub const TEST_ISSUER: &str = "policy-host";
pub const TEST_SUBJECT: &str = "assistant";
pub const TEST_CAPABILITY: &str = "EmailSendCap";
pub const TEST_DELEGATED_BY: &str = "policy-host";
pub const TOKEN_NAME_PARENT: &str = "parent";
pub const TOKEN_NAME_CHILD: &str = "child";
pub const TOKEN_NAME_VALID: &str = "valid";
pub const TOKEN_NAME_REVOKED: &str = "revoked";
pub const TOKEN_NAME_EXPIRED: &str = "expired";
pub const TOKEN_NAME_TOKEN: &str = "token";
pub const TOKEN_NAME_FUTURE: &str = "future";
pub const TOKEN_NAME_MINT_UNTRUSTED: &str = "mint-untrusted";
pub const TOKEN_NAME_MINT_INVALID: &str = "mint-invalid-lifetime";

// ── Fallible constructors ──────────────────────────────────────────

pub fn token_id(value: &str) -> Result<AuthorityTokenId, AuthorityLifecycleError> {
    AuthorityTokenId::try_from(value)
}

pub fn issuer(value: &str) -> Result<AuthorityIssuer, AuthorityLifecycleError> {
    AuthorityIssuer::try_from(value)
}

pub fn subject(value: &str) -> Result<AuthoritySubject, AuthorityLifecycleError> {
    AuthoritySubject::try_from(value)
}

pub fn capability(value: &str) -> Result<AuthorityCapability, AuthorityLifecycleError> {
    AuthorityCapability::try_from(value)
}

pub fn scope(resources: &[&str]) -> Result<AuthorityScope, AuthorityLifecycleError> {
    let parsed: Result<Vec<_>, _> = resources
        .iter()
        .map(|r| ScopeResource::try_from(*r))
        .collect();
    AuthorityScope::new(parsed?)
}

pub fn mint_token(
    id: AuthorityTokenId,
    scope: AuthorityScope,
    issued_at: u64,
    expires_at: u64,
) -> Result<AuthorityToken, AuthorityLifecycleError> {
    AuthorityToken::mint(MintRequest {
        token_id: id,
        issuer: issuer(TEST_ISSUER)?,
        issuer_trust: IssuerTrust::HostTrusted,
        subject: subject(TEST_SUBJECT)?,
        capability: capability(TEST_CAPABILITY)?,
        scope,
        issued_at: TokenTimestamp::new(issued_at),
        expires_at: TokenTimestamp::new(expires_at),
    })
}

// ── Timing bundle ──────────────────────────────────────────────────

/// Bundled delegation timing parameters.
pub struct DelegationTiming {
    /// Delegation start time.
    pub delegated_at: u64,
    /// Delegated token expiry time.
    pub expires_at: u64,
}

// ── rstest fixtures ────────────────────────────────────────────────

#[fixture]
pub fn scope_send_email() -> Result<AuthorityScope, AuthorityLifecycleError> {
    scope(&["send_email"])
}

#[fixture]
pub fn scope_email_and_draft() -> Result<AuthorityScope, AuthorityLifecycleError> {
    scope(&["send_email", "send_email_draft"])
}

#[fixture]
pub fn scope_widened() -> Result<AuthorityScope, AuthorityLifecycleError> {
    scope(&["send_email", "send_email_draft", "calendar_write"])
}

#[fixture]
pub fn token_parent() -> Result<AuthorityToken, AuthorityLifecycleError> {
    mint_token(
        token_id(TOKEN_NAME_PARENT)?,
        scope_email_and_draft()?,
        10,
        200,
    )
}

#[fixture]
pub fn token_valid() -> Result<AuthorityToken, AuthorityLifecycleError> {
    mint_token(token_id(TOKEN_NAME_VALID)?, scope_send_email()?, 10, 300)
}

#[fixture]
pub fn token_revoked() -> Result<AuthorityToken, AuthorityLifecycleError> {
    mint_token(token_id(TOKEN_NAME_REVOKED)?, scope_send_email()?, 10, 300)
}

#[fixture]
pub fn token_expired() -> Result<AuthorityToken, AuthorityLifecycleError> {
    mint_token(token_id(TOKEN_NAME_EXPIRED)?, scope_send_email()?, 10, 100)
}

#[fixture]
pub fn token_for_restore() -> Result<AuthorityToken, AuthorityLifecycleError> {
    mint_token(token_id(TOKEN_NAME_TOKEN)?, scope_send_email()?, 10, 100)
}

#[fixture]
pub fn token_future() -> Result<AuthorityToken, AuthorityLifecycleError> {
    mint_token(token_id(TOKEN_NAME_FUTURE)?, scope_send_email()?, 500, 900)
}

// ── Builder ────────────────────────────────────────────────────────

/// Builder for `MintRequest` with sensible defaults for test scenarios.
///
/// # Examples
///
/// ```rust,ignore
/// let request = MintRequestBuilder::new("tok-1")?
///     .issuer("remote-agent", IssuerTrust::Untrusted)?
///     .lifetime(10, 20)
///     .build()?;
/// ```
pub struct MintRequestBuilder {
    token_name: String,
    issuer: AuthorityIssuer,
    issuer_trust: IssuerTrust,
    subject_name: String,
    capability_name: String,
    scope_resources: Vec<String>,
    issued_at: u64,
    expires_at: u64,
}

impl MintRequestBuilder {
    /// Create a builder with host-trusted defaults for the given token name.
    pub fn new(token_name: &str) -> Result<Self, AuthorityLifecycleError> {
        Ok(Self {
            token_name: token_name.to_owned(),
            issuer: issuer(TEST_ISSUER)?,
            issuer_trust: IssuerTrust::HostTrusted,
            subject_name: TEST_SUBJECT.to_owned(),
            capability_name: TEST_CAPABILITY.to_owned(),
            scope_resources: vec!["send_email".to_owned()],
            issued_at: 0,
            expires_at: 100,
        })
    }

    /// Override the issuer identity and trust level.
    pub fn issuer(
        mut self,
        issuer_name: &str,
        trust: IssuerTrust,
    ) -> Result<Self, AuthorityLifecycleError> {
        self.issuer = issuer(issuer_name)?;
        self.issuer_trust = trust;
        Ok(self)
    }

    /// Override the token lifetime.
    pub fn lifetime(mut self, issued_at: u64, expires_at: u64) -> Self {
        self.issued_at = issued_at;
        self.expires_at = expires_at;
        self
    }

    /// Consume the builder and produce a `MintRequest`.
    pub fn build(self) -> Result<MintRequest, AuthorityLifecycleError> {
        let scope_refs: Vec<&str> = self.scope_resources.iter().map(String::as_str).collect();
        Ok(MintRequest {
            token_id: token_id(&self.token_name)?,
            issuer: self.issuer,
            issuer_trust: self.issuer_trust,
            subject: subject(&self.subject_name)?,
            capability: capability(&self.capability_name)?,
            scope: scope(&scope_refs)?,
            issued_at: TokenTimestamp::new(self.issued_at),
            expires_at: TokenTimestamp::new(self.expires_at),
        })
    }
}

// ── Assertion helpers ──────────────────────────────────────────────

pub fn assert_mint_fails<F: Fn(&AuthorityLifecycleError) -> bool>(
    request: MintRequest,
    predicate: F,
) {
    let result = AuthorityToken::mint(request);
    match result {
        Err(ref err) if predicate(err) => {}
        _ => panic!("mint request did not fail as expected, got: {result:?}"),
    }
}

pub fn delegation_request_with_scope(
    child_id: &AuthorityTokenId,
    scope: &AuthorityScope,
    timing: &DelegationTiming,
) -> Result<DelegationRequest, AuthorityLifecycleError> {
    Ok(DelegationRequest {
        token_id: child_id.clone(),
        delegated_by: issuer(TEST_DELEGATED_BY)?,
        subject: subject(TEST_SUBJECT)?,
        scope: scope.clone(),
        delegated_at: TokenTimestamp::new(timing.delegated_at),
        expires_at: TokenTimestamp::new(timing.expires_at),
    })
}

pub fn assert_delegation_fails<F: Fn(&AuthorityLifecycleError) -> bool>(
    parent: &AuthorityToken,
    request: DelegationRequest,
    revocation_index: &RevocationIndex,
    predicate: F,
) {
    let result = AuthorityToken::delegate(parent, request, revocation_index);
    match result {
        Err(ref err) if predicate(err) => {}
        _ => panic!("delegation request did not fail as expected, got: {result:?}"),
    }
}
