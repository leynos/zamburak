//! Policy-boundary and snapshot-restore authority validation.

use super::types::{AuthorityTokenId, InvalidAuthorityReason, TokenTimestamp};
use super::{AuthorityToken, RevocationIndex};

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

        if token.is_pre_issuance_at(evaluation_time) {
            invalid_tokens.push(InvalidAuthorityToken {
                token_id: token.token_id().clone(),
                reason: InvalidAuthorityReason::PreIssuance,
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
