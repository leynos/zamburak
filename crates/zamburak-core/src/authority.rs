//! Authority token lifecycle domain model and validation utilities.

mod errors;
mod types;
mod validation;

use std::collections::HashSet;

pub use errors::AuthorityLifecycleError;
use errors::validate_lifetime;
pub use types::{
    AuthorityCapability, AuthorityIssuer, AuthorityScope, AuthoritySubject, AuthorityTokenId,
    DelegationRequest, InvalidAuthorityReason, IssuerTrust, MintRequest, ScopeResource,
    TokenTimestamp,
};
pub use validation::{
    AuthorityBoundaryValidation, InvalidAuthorityToken, revalidate_tokens_on_restore,
    validate_tokens_at_policy_boundary,
};

/// Host-minted authority token with lineage and lifecycle fields.
///
/// Tokens encode a subject, capability, scope, and time bounds. They are
/// created via [`mint`](Self::mint) or derived from a parent via
/// [`delegate`](Self::delegate), preserving lineage for audit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityToken {
    token_id: AuthorityTokenId,
    issuer: AuthorityIssuer,
    subject: AuthoritySubject,
    capability: AuthorityCapability,
    scope: AuthorityScope,
    issued_at: TokenTimestamp,
    expires_at: TokenTimestamp,
    parent_token_id: Option<AuthorityTokenId>,
}

impl AuthorityToken {
    /// Mint a new root authority token from a host-trusted issuer.
    ///
    /// Untrusted issuers are rejected fail-closed. The minted token has no
    /// parent lineage.
    ///
    /// # Examples
    ///
    /// ```
    /// use zamburak_core::{
    ///     AuthorityToken, AuthorityTokenId, AuthorityIssuer, AuthoritySubject,
    ///     AuthorityCapability, AuthorityScope, ScopeResource, IssuerTrust,
    ///     MintRequest, TokenTimestamp,
    /// };
    ///
    /// let token = AuthorityToken::mint(MintRequest {
    ///     token_id: AuthorityTokenId::try_from("tok-1")?,
    ///     issuer: AuthorityIssuer::try_from("policy-host")?,
    ///     issuer_trust: IssuerTrust::HostTrusted,
    ///     subject: AuthoritySubject::try_from("assistant")?,
    ///     capability: AuthorityCapability::try_from("EmailSendCap")?,
    ///     scope: AuthorityScope::new(vec![ScopeResource::try_from("send_email")?])?,
    ///     issued_at: TokenTimestamp::new(100),
    ///     expires_at: TokenTimestamp::new(500),
    /// })?;
    ///
    /// assert!(token.parent_token_id().is_none());
    /// # Ok::<(), zamburak_core::AuthorityLifecycleError>(())
    /// ```
    pub fn mint(request: MintRequest) -> Result<Self, AuthorityLifecycleError> {
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
    ///
    /// Delegation enforces fail-closed ordering: revoked or expired parents
    /// are rejected before scope or lifetime checks run. The child token
    /// inherits the parent's capability and records parent lineage.
    ///
    /// # Examples
    ///
    /// ```
    /// use zamburak_core::{
    ///     AuthorityToken, AuthorityTokenId, AuthorityIssuer, AuthoritySubject,
    ///     AuthorityCapability, AuthorityScope, ScopeResource, IssuerTrust,
    ///     MintRequest, DelegationRequest, RevocationIndex, TokenTimestamp,
    /// };
    ///
    /// let parent = AuthorityToken::mint(MintRequest {
    ///     token_id: AuthorityTokenId::try_from("parent")?,
    ///     issuer: AuthorityIssuer::try_from("policy-host")?,
    ///     issuer_trust: IssuerTrust::HostTrusted,
    ///     subject: AuthoritySubject::try_from("assistant")?,
    ///     capability: AuthorityCapability::try_from("EmailSendCap")?,
    ///     scope: AuthorityScope::new(vec![
    ///         ScopeResource::try_from("send_email")?,
    ///         ScopeResource::try_from("draft_email")?,
    ///     ])?,
    ///     issued_at: TokenTimestamp::new(10),
    ///     expires_at: TokenTimestamp::new(200),
    /// })?;
    ///
    /// let child = AuthorityToken::delegate(
    ///     &parent,
    ///     DelegationRequest {
    ///         token_id: AuthorityTokenId::try_from("child")?,
    ///         delegated_by: AuthorityIssuer::try_from("policy-host")?,
    ///         subject: AuthoritySubject::try_from("assistant")?,
    ///         scope: AuthorityScope::new(vec![
    ///             ScopeResource::try_from("send_email")?,
    ///         ])?,
    ///         delegated_at: TokenTimestamp::new(20),
    ///         expires_at: TokenTimestamp::new(120),
    ///     },
    ///     &RevocationIndex::default(),
    /// )?;
    ///
    /// assert_eq!(child.parent_token_id(), Some(parent.token_id()));
    /// # Ok::<(), zamburak_core::AuthorityLifecycleError>(())
    /// ```
    pub fn delegate(
        parent: &Self,
        request: DelegationRequest,
        revocation_index: &RevocationIndex,
    ) -> Result<Self, AuthorityLifecycleError> {
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

        if request.delegated_at < parent.issued_at() {
            return Err(AuthorityLifecycleError::DelegationBeforeParentIssuance {
                delegated_at: request.delegated_at.as_u64(),
                parent_issued_at: parent.issued_at().as_u64(),
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

    /// Check whether this token authorizes a specific tool-resource action.
    ///
    /// Returns `true` only when subject, capability, and resource all match.
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

    /// Check whether the token has expired at a given evaluation time.
    ///
    /// Expiry is inclusive: a token whose `expires_at` equals `evaluation_time`
    /// is considered expired.
    #[must_use]
    pub fn is_expired_at(&self, evaluation_time: TokenTimestamp) -> bool {
        evaluation_time >= self.expires_at
    }

    /// Check whether the evaluation time precedes this token's issuance.
    ///
    /// Pre-issuance tokens are stripped at policy boundaries.
    #[must_use]
    pub fn is_pre_issuance_at(&self, evaluation_time: TokenTimestamp) -> bool {
        evaluation_time < self.issued_at
    }

    /// Stable identifier used for revocation lookups and lineage tracking.
    #[must_use]
    pub const fn token_id(&self) -> &AuthorityTokenId {
        &self.token_id
    }

    /// Issuer that minted or delegated this token, used for audit provenance.
    #[must_use]
    pub const fn issuer(&self) -> &AuthorityIssuer {
        &self.issuer
    }

    /// Principal to whom this token grants authority.
    #[must_use]
    pub const fn subject(&self) -> &AuthoritySubject {
        &self.subject
    }

    /// Capability this token encodes (inherited from parent on delegation).
    #[must_use]
    pub const fn capability(&self) -> &AuthorityCapability {
        &self.capability
    }

    /// Set of scope resources this token permits.
    #[must_use]
    pub const fn scope(&self) -> &AuthorityScope {
        &self.scope
    }

    /// Timestamp from which the token is considered valid.
    #[must_use]
    pub const fn issued_at(&self) -> TokenTimestamp {
        self.issued_at
    }

    /// Timestamp at which the token expires (inclusive boundary).
    #[must_use]
    pub const fn expires_at(&self) -> TokenTimestamp {
        self.expires_at
    }

    /// Parent token identifier, present only for delegated tokens.
    #[must_use]
    pub const fn parent_token_id(&self) -> Option<&AuthorityTokenId> {
        self.parent_token_id.as_ref()
    }
}

/// Host-managed index tracking revoked authority tokens.
///
/// Revoked tokens are stripped at policy-evaluation boundaries and rejected
/// as delegation parents.
///
/// # Examples
///
/// ```
/// use zamburak_core::{RevocationIndex, AuthorityTokenId};
///
/// let mut index = RevocationIndex::default();
/// let id = AuthorityTokenId::try_from("tok-1").unwrap();
/// assert!(!index.is_revoked(&id));
///
/// index.revoke(id.clone());
/// assert!(index.is_revoked(&id));
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RevocationIndex {
    revoked_tokens: HashSet<AuthorityTokenId>,
}

impl RevocationIndex {
    /// Mark a token as revoked so it is stripped at the next boundary check.
    pub fn revoke(&mut self, token_id: AuthorityTokenId) {
        self.revoked_tokens.insert(token_id);
    }

    /// Check whether a token identifier has been revoked.
    #[must_use]
    pub fn is_revoked(&self, token_id: &AuthorityTokenId) -> bool {
        self.revoked_tokens.contains(token_id)
    }
}

#[cfg(test)]
mod test_helpers;
#[cfg(test)]
mod tests;
