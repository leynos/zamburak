//! Domain value types for authority token lifecycle modelling.

use std::collections::BTreeSet;

use super::errors::{AuthorityLifecycleError, non_empty};

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

#[expect(
    clippy::expl_impl_clone_on_copy,
    reason = "newt-hype macro expansion emits explicit Clone for Copy wrappers"
)]
mod authority_newtypes {
    use newt_hype::{base_newtype, newtype};

    base_newtype!(AuthorityStringNewtype);
    newtype!(AuthorityTokenId, AuthorityStringNewtype, String);
}

/// Stable identifier for an authority token.
pub type AuthorityTokenId = authority_newtypes::AuthorityTokenId;

impl TryFrom<&str> for AuthorityTokenId {
    type Error = AuthorityLifecycleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        non_empty(value, "token_id")?;
        Ok(Self::new(value.to_owned()))
    }
}

/// Validated issuer identity for minting and delegation provenance.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct AuthorityIssuer(String);

impl AuthorityIssuer {
    /// Return the issuer as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AuthorityIssuer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl TryFrom<&str> for AuthorityIssuer {
    type Error = AuthorityLifecycleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        non_empty(value, "issuer")?;
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
    /// Minting issuer identity for audit provenance.
    pub issuer: AuthorityIssuer,
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
    /// Delegating issuer identity for lineage.
    pub delegated_by: AuthorityIssuer,
    /// Delegated subject.
    pub subject: AuthoritySubject,
    /// Delegated scope.
    pub scope: AuthorityScope,
    /// Delegation time.
    pub delegated_at: TokenTimestamp,
    /// Delegated token expiry.
    pub expires_at: TokenTimestamp,
}

/// Invalidity reason recorded for stripped authority tokens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidAuthorityReason {
    /// Token was revoked in the revocation index.
    Revoked,
    /// Token expired at evaluation time.
    Expired,
    /// Token has not yet reached its issuance time.
    PreIssuance,
}

impl std::fmt::Display for InvalidAuthorityReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reason = match self {
            Self::Revoked => "revoked",
            Self::Expired => "expired",
            Self::PreIssuance => "pre-issuance",
        };
        formatter.write_str(reason)
    }
}
