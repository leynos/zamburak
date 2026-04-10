//! Public types for runtime external-call policy evaluation.
//!
//! These types define the stable request and decision surface consumed by
//! `zamburak-monty` and other policy evaluation clients.

use zamburak_core::DependencySummary;
use zamburak_core::control_context::ExecutionContextSummary;
use zamburak_core::trust::AuthoritySet;

/// External-call classification used for policy diagnostics.
///
/// This is a policy-layer-owned copy of the Monty runtime's call-kind
/// discriminator, allowing the policy crate to remain independent of
/// the interpreter implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalCallKind {
    /// External function call.
    Function,
    /// OS-level system call.
    Os,
    /// Method call on an external object.
    Method,
}

/// Per-keyword dependency summaries aligned to a concrete keyword name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeywordArgumentSummary {
    /// Resolved keyword identifier used to match `arg_rules`.
    pub name: String,
    /// Dependency summary for the keyword key object itself.
    pub key_summary: DependencySummary,
    /// Dependency summary for the keyword value.
    pub value_summary: DependencySummary,
}

/// Input for external-call policy evaluation.
///
/// This type is policy-layer-owned and does not depend on Monty runtime
/// internals, enabling independent testing and versioning of the evaluation
/// contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalCallPolicyInput {
    /// Tool name used for policy lookup.
    pub tool_name: String,
    /// External-call classification.
    pub call_kind: ExternalCallKind,
    /// Aggregate dependency summary for the whole call.
    pub aggregate_summary: DependencySummary,
    /// Per-positional-argument dependency summaries.
    pub arg_summaries: Vec<DependencySummary>,
    /// Per-keyword summaries aligned to resolved keyword identifiers.
    pub kwarg_summaries: Vec<KeywordArgumentSummary>,
    /// Authority capabilities held by the caller at this call boundary.
    pub caller_authority: AuthoritySet,
    /// Control-context snapshot at the call boundary.
    pub control_context: ExecutionContextSummary,
}

/// Decision outcome for an evaluated external-call request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExternalCallPolicyDecision {
    /// Allow the external call to proceed.
    Allow(PolicyDecisionExplanation),
    /// Deny the external call before side-effect execution.
    Deny(PolicyDecisionExplanation),
    /// Require interactive confirmation before proceeding.
    RequireConfirmation(PolicyDecisionExplanation),
}

/// Machine-parseable reason code for policy decisions.
///
/// This enum provides a stable, deterministic classification of decision
/// rationale, enabling structured audit pipelines and programmatic handling
/// of policy outcomes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyDecisionReason {
    /// Tool not found in policy; failed closed.
    MissingToolPolicy,
    /// Denied by a hard constraint in context rules (e.g., PC integrity match).
    ContextRuleDeny,
    /// Caller lacks a required authority capability.
    MissingAuthority,
    /// Invalid authority capability string in policy definition.
    InvalidAuthorityInPolicy,
    /// Argument does not meet required integrity level.
    ArgumentIntegrityRequirement,
    /// Argument contains a forbidden confidentiality label.
    ArgumentConfidentialityForbidden,
    /// Denied by the tool's default policy action.
    DefaultDeny,
    /// Allowed by the tool's default policy action.
    DefaultAllow,
    /// Confirmation required by the tool's default policy action.
    DefaultRequireConfirmation,
    /// Confirmation required (mapped conservatively from RequireDraft).
    RequireDraftMappedToConfirmation,
}

/// Explanation metadata attached to a policy decision.
///
/// This type provides deterministic, safely redacted evidence for why a
/// decision was made, including both a machine-parseable reason code and a
/// human-readable summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDecisionExplanation {
    /// Machine-parseable reason code for this decision.
    pub reason: PolicyDecisionReason,
    /// Human-readable summary of the decision rationale.
    pub summary: String,
}
