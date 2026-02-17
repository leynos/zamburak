//! Authority token lifecycle domain model and validation utilities.

mod errors;
mod types;
mod validation;

use std::collections::HashSet;

pub use errors::AuthorityLifecycleError;
use errors::{non_empty, validate_lifetime};
pub use types::{
    AuthorityCapability, AuthorityScope, AuthoritySubject, AuthorityTokenId, DelegationRequest,
    InvalidAuthorityReason, IssuerTrust, MintRequest, ScopeResource, TokenTimestamp,
};
pub use validation::{
    AuthorityBoundaryValidation, InvalidAuthorityToken, revalidate_tokens_on_restore,
    validate_tokens_at_policy_boundary,
};

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

    /// Return whether the token has not yet reached its issuance time.
    #[must_use]
    pub fn is_pre_issuance_at(&self, evaluation_time: TokenTimestamp) -> bool {
        evaluation_time < self.issued_at
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

#[cfg(test)]
mod test_helpers;
#[cfg(test)]
mod tests;
