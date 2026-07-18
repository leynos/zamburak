//! Policy schema value types and the loader failure contract.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::migration::MigrationError;

use super::{BudgetLimit, SchemaVersion};

/// Loader failure contract for policy definitions.
#[derive(Debug, Error)]
pub enum PolicyLoadError {
    /// YAML parser rejected the policy document.
    #[error("policy YAML parsing failed: {0}")]
    InvalidYaml(#[source] serde_yaml::Error),
    /// JSON parser rejected the policy document.
    #[error("policy JSON parsing failed: {0}")]
    InvalidJson(#[source] serde_json::Error),
    /// Runtime rejects policies that do not match the canonical schema version.
    #[error("unsupported policy schema_version `{found}`; only `{expected}` is accepted")]
    UnsupportedSchemaVersion {
        /// Parsed schema version in the input document.
        found: SchemaVersion,
        /// Canonical schema version accepted by the runtime.
        expected: SchemaVersion,
    },
    /// Migration-audit evidence generation failed during policy loading.
    #[error("migration audit generation failed: {0}")]
    MigrationAuditFailed(#[source] MigrationError),
}

/// Policy fallback and per-rule action types.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PolicyAction {
    /// Allow without requiring confirmation.
    Allow,
    /// Deny the action.
    Deny,
    /// Require explicit user confirmation.
    RequireConfirmation,
    /// Require draft generation instead of direct execution.
    RequireDraft,
}

/// Supported side-effect classes for policy tool entries.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SideEffectClass {
    /// Non-mutating external read.
    ExternalRead,
    /// Mutating external write.
    ExternalWrite,
}

/// Budget limits used by dependency summarization and traversal.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyBudgets {
    /// Maximum number of tracked values.
    pub max_values: BudgetLimit,
    /// Maximum number of parents per value.
    pub max_parents_per_value: BudgetLimit,
    /// Maximum number of closure traversal steps.
    pub max_closure_steps: BudgetLimit,
    /// Maximum witness depth in explanations.
    pub max_witness_depth: BudgetLimit,
}

/// Per-tool policy definition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolPolicy {
    /// Tool identifier.
    pub tool: String,
    /// Side-effect class used by policy evaluation.
    pub side_effect_class: SideEffectClass,
    /// Required authority tokens.
    #[serde(default)]
    pub required_authority: Vec<String>,
    /// Argument rules applied to tool call arguments.
    #[serde(default)]
    pub arg_rules: Vec<ArgRule>,
    /// Context rules applied to execution control context.
    #[serde(default)]
    pub context_rules: Option<ContextRules>,
    /// Default decision for the tool.
    pub default_decision: PolicyAction,
}

/// Per-argument policy constraints.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArgRule {
    /// Argument identifier.
    pub arg: String,
    /// Optional integrity requirement.
    #[serde(default)]
    pub requires_integrity: Option<String>,
    /// Optional confidentiality deny-list.
    #[serde(default)]
    pub forbids_confidentiality: Vec<String>,
}

/// Context constraints for a tool policy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextRules {
    /// Deny when program-counter integrity contains any listed labels.
    #[serde(default)]
    pub deny_if_pc_integrity_contains: Vec<String>,
}
