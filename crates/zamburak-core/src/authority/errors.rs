//! Authority lifecycle error types and validation helpers.

use thiserror::Error;

use super::types::{AuthorityTokenId, InvalidAuthorityReason, TokenTimestamp};

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
    /// Delegation starts before the parent token was issued.
    #[error("delegated_at `{delegated_at}` is before parent issued_at `{parent_issued_at}`")]
    DelegationBeforeParentIssuance {
        /// Delegation start timestamp.
        delegated_at: u64,
        /// Parent issuance timestamp.
        parent_issued_at: u64,
    },
}

pub(crate) fn non_empty(value: &str, field: &'static str) -> Result<(), AuthorityLifecycleError> {
    if value.trim().is_empty() {
        return Err(AuthorityLifecycleError::EmptyField { field });
    }

    Ok(())
}

pub(crate) fn validate_lifetime(
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
