//! Core runtime contracts for authority lifecycle, localization, and
//! information-flow control.

pub mod authority;
pub mod control_context;
pub mod dependency_graph;
pub mod i18n;
pub mod ifc_errors;
pub mod propagation;
pub mod summary;
pub mod trust;
pub mod value_id;

pub use authority::{
    AuthorityBoundaryValidation, AuthorityCapability, AuthorityIssuer, AuthorityLifecycleError,
    AuthorityScope, AuthoritySubject, AuthorityToken, AuthorityTokenId, DelegationRequest,
    InvalidAuthorityReason, InvalidAuthorityToken, IssuerTrust, MintRequest, RevocationIndex,
    ScopeResource, TokenTimestamp, revalidate_tokens_on_restore,
    validate_tokens_at_policy_boundary,
};
pub use dependency_graph::{DependencyGraph, GraphBudgets, ValueLabels, ValueNode};
pub use ifc_errors::IfcError;
pub use summary::DependencySummary;
pub use trust::{AuthoritySet, DataLabel, DataLabels, IntegrityLabel};
pub use value_id::ValueId;
