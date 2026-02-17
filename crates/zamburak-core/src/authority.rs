//! Authority token lifecycle domain model and validation utilities.

use std::collections::{BTreeSet, HashSet};

use thiserror::Error;

/// Monotonic timestamp used for lifecycle evaluation boundaries.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TokenTimestamp(u64);

impl TokenTimestamp {
    /// Create a timestamp from seconds in the runtime clock domain.
    #[must_use]
    pub const fn new(seconds: u64) -> Self {
        Self(seconds)
    }

    /// Return the wrapped timestamp value.
    #[must_use]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Stable identifier for an authority token.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct AuthorityTokenId(String);

impl AuthorityTokenId {
    /// Return the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AuthorityTokenId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl TryFrom<&str> for AuthorityTokenId {
    type Error = AuthorityLifecycleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        non_empty(value, "token_id")?;
        Ok(Self(value.to_owned()))
    }
}

/// Subject for whom authority is granted.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct AuthoritySubject(String);

impl AuthoritySubject {
    /// Return the subject as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for AuthoritySubject {
    type Error = AuthorityLifecycleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        non_empty(value, "subject")?;
        Ok(Self(value.to_owned()))
    }
}

/// Capability encoded by an authority token.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct AuthorityCapability(String);

impl AuthorityCapability {
    /// Return the capability as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for AuthorityCapability {
    type Error = AuthorityLifecycleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        non_empty(value, "capability")?;
        Ok(Self(value.to_owned()))
    }
}

/// Scope entry that an authority token may permit.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct ScopeResource(String);

impl ScopeResource {
    /// Return the scope resource as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for ScopeResource {
    type Error = AuthorityLifecycleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        non_empty(value, "scope_resource")?;
        Ok(Self(value.to_owned()))
    }
}

/// Set of scope resources permitted by a token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityScope {
    resources: BTreeSet<ScopeResource>,
}

impl AuthorityScope {
    /// Build a scope from resources, rejecting empty sets.
    pub fn new(
        resources: impl IntoIterator<Item = ScopeResource>,
    ) -> Result<Self, AuthorityLifecycleError> {
        let scope = Self {
            resources: resources.into_iter().collect(),
        };

        if scope.resources.is_empty() {
            return Err(AuthorityLifecycleError::EmptyField { field: "scope" });
        }

        Ok(scope)
    }

    /// Return whether this scope strictly narrows another scope.
    #[must_use]
    pub fn is_strict_subset_of(&self, parent: &Self) -> bool {
        self.resources.is_subset(&parent.resources) && self.resources != parent.resources
    }

    /// Return whether the scope includes a specific resource.
    #[must_use]
    pub fn contains(&self, resource: &ScopeResource) -> bool {
        self.resources.contains(resource)
    }
}

/// Trust class of the minting issuer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssuerTrust {
    /// Trusted host-side minting authority.
    HostTrusted,
    /// Untrusted minting source.
    Untrusted,
}

/// Parameters for minting an authority token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MintRequest {
    /// New token identifier.
    pub token_id: AuthorityTokenId,
    /// Minting issuer name for audit provenance.
    pub issuer: String,
    /// Minting issuer trust class.
    pub issuer_trust: IssuerTrust,
    /// Subject receiving authority.
    pub subject: AuthoritySubject,
    /// Capability encoded into the token.
    pub capability: AuthorityCapability,
    /// Scope resources the token permits.
    pub scope: AuthorityScope,
    /// Token issue time.
    pub issued_at: TokenTimestamp,
    /// Token expiry time.
    pub expires_at: TokenTimestamp,
}

/// Parameters for delegating an existing authority token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationRequest {
    /// New delegated token identifier.
    pub token_id: AuthorityTokenId,
    /// Delegating issuer name for lineage.
    pub delegated_by: String,
    /// Delegated subject.
    pub subject: AuthoritySubject,
    /// Delegated scope.
    pub scope: AuthorityScope,
    /// Delegation time.
    pub delegated_at: TokenTimestamp,
    /// Delegated token expiry.
    pub expires_at: TokenTimestamp,
}

/// Host-minted authority token with lineage and lifecycle fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityToken {
    token_id: AuthorityTokenId,
    issuer: String,
    subject: AuthoritySubject,
    capability: AuthorityCapability,
    scope: AuthorityScope,
    issued_at: TokenTimestamp,
    expires_at: TokenTimestamp,
    parent_token_id: Option<AuthorityTokenId>,
}

impl AuthorityToken {
    /// Mint a new authority token.
    pub fn mint(request: MintRequest) -> Result<Self, AuthorityLifecycleError> {
        non_empty(&request.issuer, "issuer")?;
        validate_lifetime(request.issued_at, request.expires_at)?;

        if request.issuer_trust != IssuerTrust::HostTrusted {
            return Err(AuthorityLifecycleError::UntrustedMinter {
                issuer: request.issuer,
            });
        }

        Ok(Self {
            token_id: request.token_id,
            issuer: request.issuer,
            subject: request.subject,
            capability: request.capability,
            scope: request.scope,
            issued_at: request.issued_at,
            expires_at: request.expires_at,
            parent_token_id: None,
        })
    }

    /// Delegate an authority token with strict scope and lifetime narrowing.
    pub fn delegate(
        parent: &Self,
        request: DelegationRequest,
        revocation_index: &RevocationIndex,
    ) -> Result<Self, AuthorityLifecycleError> {
        non_empty(&request.delegated_by, "delegated_by")?;

        // Reject invalid parent tokens before inspecting the delegation request
        // itself so that revoked or expired parents fail closed regardless of
        // whether the request carries well-formed timestamps or scope.
        if revocation_index.is_revoked(parent.token_id()) {
            return Err(AuthorityLifecycleError::InvalidParentToken {
                token_id: parent.token_id().clone(),
                reason: InvalidAuthorityReason::Revoked,
            });
        }

        if parent.is_expired_at(request.delegated_at) {
            return Err(AuthorityLifecycleError::InvalidParentToken {
                token_id: parent.token_id().clone(),
                reason: InvalidAuthorityReason::Expired,
            });
        }

        validate_lifetime(request.delegated_at, request.expires_at)?;

        if !request.scope.is_strict_subset_of(parent.scope()) {
            return Err(AuthorityLifecycleError::DelegationScopeNotStrictSubset);
        }

        if request.expires_at >= parent.expires_at() {
            return Err(AuthorityLifecycleError::DelegationLifetimeNotStrictSubset {
                delegated_expires_at: request.expires_at.as_u64(),
                parent_expires_at: parent.expires_at().as_u64(),
            });
        }

        Ok(Self {
            token_id: request.token_id,
            issuer: request.delegated_by,
            subject: request.subject,
            capability: parent.capability().clone(),
            scope: request.scope,
            issued_at: request.delegated_at,
            expires_at: request.expires_at,
            parent_token_id: Some(parent.token_id().clone()),
        })
    }

    /// Return whether this token grants authority for a tool resource.
    #[must_use]
    pub fn grants(
        &self,
        subject: &AuthoritySubject,
        capability: &AuthorityCapability,
        tool_resource: &ScopeResource,
    ) -> bool {
        self.subject() == subject
            && self.capability() == capability
            && self.scope().contains(tool_resource)
    }

    /// Return whether the token is expired at an evaluation time.
    #[must_use]
    pub fn is_expired_at(&self, evaluation_time: TokenTimestamp) -> bool {
        evaluation_time >= self.expires_at
    }

    /// Return the token identifier.
    #[must_use]
    pub const fn token_id(&self) -> &AuthorityTokenId {
        &self.token_id
    }

    /// Return the token issuer.
    #[must_use]
    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    /// Return the token subject.
    #[must_use]
    pub const fn subject(&self) -> &AuthoritySubject {
        &self.subject
    }

    /// Return the token capability.
    #[must_use]
    pub const fn capability(&self) -> &AuthorityCapability {
        &self.capability
    }

    /// Return the token scope.
    #[must_use]
    pub const fn scope(&self) -> &AuthorityScope {
        &self.scope
    }

    /// Return token issuance time.
    #[must_use]
    pub const fn issued_at(&self) -> TokenTimestamp {
        self.issued_at
    }

    /// Return token expiry time.
    #[must_use]
    pub const fn expires_at(&self) -> TokenTimestamp {
        self.expires_at
    }

    /// Return parent token identifier when this token was delegated.
    #[must_use]
    pub const fn parent_token_id(&self) -> Option<&AuthorityTokenId> {
        self.parent_token_id.as_ref()
    }
}

/// Host-managed revocation index for authority tokens.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RevocationIndex {
    revoked_tokens: HashSet<AuthorityTokenId>,
}

impl RevocationIndex {
    /// Revoke a token identifier.
    pub fn revoke(&mut self, token_id: AuthorityTokenId) {
        self.revoked_tokens.insert(token_id);
    }

    /// Return whether a token identifier is revoked.
    #[must_use]
    pub fn is_revoked(&self, token_id: &AuthorityTokenId) -> bool {
        self.revoked_tokens.contains(token_id)
    }
}

/// Invalidity reason recorded for stripped authority tokens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidAuthorityReason {
    /// Token was revoked in the revocation index.
    Revoked,
    /// Token expired at evaluation time.
    Expired,
}

impl std::fmt::Display for InvalidAuthorityReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reason = match self {
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        };
        formatter.write_str(reason)
    }
}

/// Record describing a token stripped during lifecycle validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidAuthorityToken {
    token_id: AuthorityTokenId,
    reason: InvalidAuthorityReason,
}

impl InvalidAuthorityToken {
    /// Return the stripped token identifier.
    #[must_use]
    pub const fn token_id(&self) -> &AuthorityTokenId {
        &self.token_id
    }

    /// Return the invalidation reason.
    #[must_use]
    pub const fn reason(&self) -> InvalidAuthorityReason {
        self.reason
    }
}

/// Result of authority lifecycle validation at a policy boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityBoundaryValidation {
    effective_tokens: Vec<AuthorityToken>,
    invalid_tokens: Vec<InvalidAuthorityToken>,
}

impl AuthorityBoundaryValidation {
    /// Return tokens that remain valid after lifecycle checks.
    #[must_use]
    pub fn effective_tokens(&self) -> &[AuthorityToken] {
        &self.effective_tokens
    }

    /// Return tokens stripped during lifecycle checks.
    #[must_use]
    pub fn invalid_tokens(&self) -> &[InvalidAuthorityToken] {
        &self.invalid_tokens
    }
}

/// Validate authority tokens at a policy-evaluation boundary.
#[must_use]
pub fn validate_tokens_at_policy_boundary(
    tokens: &[AuthorityToken],
    revocation_index: &RevocationIndex,
    evaluation_time: TokenTimestamp,
) -> AuthorityBoundaryValidation {
    let mut effective_tokens = Vec::new();
    let mut invalid_tokens = Vec::new();

    for token in tokens {
        if revocation_index.is_revoked(token.token_id()) {
            invalid_tokens.push(InvalidAuthorityToken {
                token_id: token.token_id().clone(),
                reason: InvalidAuthorityReason::Revoked,
            });
            continue;
        }

        if token.is_expired_at(evaluation_time) {
            invalid_tokens.push(InvalidAuthorityToken {
                token_id: token.token_id().clone(),
                reason: InvalidAuthorityReason::Expired,
            });
            continue;
        }

        effective_tokens.push(token.clone());
    }

    AuthorityBoundaryValidation {
        effective_tokens,
        invalid_tokens,
    }
}

/// Revalidate authority tokens on snapshot restore.
#[must_use]
pub fn revalidate_tokens_on_restore(
    tokens: &[AuthorityToken],
    revocation_index: &RevocationIndex,
    restore_time: TokenTimestamp,
) -> AuthorityBoundaryValidation {
    validate_tokens_at_policy_boundary(tokens, revocation_index, restore_time)
}

/// Authority lifecycle validation errors.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AuthorityLifecycleError {
    /// Required text field is empty.
    #[error("authority field `{field}` cannot be empty")]
    EmptyField {
        /// Empty field name.
        field: &'static str,
    },
    /// Token lifetime does not progress forward in time.
    #[error(
        "token lifetime is invalid: issued_at `{issued_at}` must be before expires_at `{expires_at}`"
    )]
    InvalidTokenLifetime {
        /// Token issuance timestamp.
        issued_at: u64,
        /// Token expiry timestamp.
        expires_at: u64,
    },
    /// Issuer is not trusted for minting.
    #[error("issuer `{issuer}` is not trusted to mint authority tokens")]
    UntrustedMinter {
        /// Untrusted issuer name.
        issuer: String,
    },
    /// Delegated scope does not narrow the parent scope.
    #[error("delegated scope must be a strict subset of parent scope")]
    DelegationScopeNotStrictSubset,
    /// Delegated lifetime does not narrow the parent lifetime.
    #[error(
        "delegated expiry `{delegated_expires_at}` must be before parent expiry `{parent_expires_at}`"
    )]
    DelegationLifetimeNotStrictSubset {
        /// Delegated token expiry timestamp.
        delegated_expires_at: u64,
        /// Parent token expiry timestamp.
        parent_expires_at: u64,
    },
    /// Parent token cannot be delegated due to lifecycle invalidity.
    #[error("parent token `{token_id}` cannot be delegated because it is {reason}")]
    InvalidParentToken {
        /// Parent token identifier.
        token_id: AuthorityTokenId,
        /// Parent invalidity reason.
        reason: InvalidAuthorityReason,
    },
}

fn non_empty(value: &str, field: &'static str) -> Result<(), AuthorityLifecycleError> {
    if value.trim().is_empty() {
        return Err(AuthorityLifecycleError::EmptyField { field });
    }

    Ok(())
}

fn validate_lifetime(
    issued_at: TokenTimestamp,
    expires_at: TokenTimestamp,
) -> Result<(), AuthorityLifecycleError> {
    if expires_at <= issued_at {
        return Err(AuthorityLifecycleError::InvalidTokenLifetime {
            issued_at: issued_at.as_u64(),
            expires_at: expires_at.as_u64(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests;
