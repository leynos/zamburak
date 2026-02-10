//! Canonical policy schema models and schema-version validation.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Canonical schema version accepted by runtime policy loaders.
pub const CANONICAL_POLICY_SCHEMA_VERSION: u64 = 1;

/// A validated policy definition that can be used by runtime loaders.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyDefinition {
    /// Policy schema contract version.
    pub schema_version: u64,
    /// Stable policy profile identifier.
    pub policy_name: String,
    /// Baseline action for non-matching tool rules.
    pub default_action: PolicyAction,
    /// Strict-mode policy switch.
    pub strict_mode: bool,
    /// Resource limits for provenance analysis.
    pub budgets: PolicyBudgets,
    /// Per-tool policy definitions.
    pub tools: Vec<ToolPolicy>,
}

impl PolicyDefinition {
    /// Parse and validate a policy document from YAML.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zamburak_policy::{PolicyDefinition, PolicyLoadError};
    ///
    /// let policy_yaml = r#"
    /// schema_version: 1
    /// policy_name: minimal_policy
    /// default_action: Deny
    /// strict_mode: true
    /// budgets:
    ///   max_values: 1
    ///   max_parents_per_value: 1
    ///   max_closure_steps: 1
    ///   max_witness_depth: 1
    /// tools: []
    /// "#;
    ///
    /// let policy = PolicyDefinition::from_yaml_str(policy_yaml)?;
    /// assert_eq!(policy.schema_version, 1);
    ///
    /// Ok::<(), PolicyLoadError>(())
    /// ```
    pub fn from_yaml_str(policy_yaml: &str) -> Result<Self, PolicyLoadError> {
        let policy =
            serde_yaml::from_str::<Self>(policy_yaml).map_err(PolicyLoadError::InvalidYaml)?;
        policy.ensure_canonical_schema_version()
    }

    /// Parse and validate a policy document from JSON.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zamburak_policy::{PolicyDefinition, PolicyLoadError};
    ///
    /// let policy_json = r#"
    /// {
    ///   "schema_version": 1,
    ///   "policy_name": "minimal_policy",
    ///   "default_action": "Deny",
    ///   "strict_mode": true,
    ///   "budgets": {
    ///     "max_values": 1,
    ///     "max_parents_per_value": 1,
    ///     "max_closure_steps": 1,
    ///     "max_witness_depth": 1
    ///   },
    ///   "tools": []
    /// }
    /// "#;
    ///
    /// let policy = PolicyDefinition::from_json_str(policy_json)?;
    /// assert_eq!(policy.schema_version, 1);
    ///
    /// Ok::<(), PolicyLoadError>(())
    /// ```
    pub fn from_json_str(policy_json: &str) -> Result<Self, PolicyLoadError> {
        let policy =
            serde_json::from_str::<Self>(policy_json).map_err(PolicyLoadError::InvalidJson)?;
        policy.ensure_canonical_schema_version()
    }

    fn ensure_canonical_schema_version(self) -> Result<Self, PolicyLoadError> {
        if self.schema_version == CANONICAL_POLICY_SCHEMA_VERSION {
            Ok(self)
        } else {
            Err(PolicyLoadError::UnsupportedSchemaVersion {
                found: self.schema_version,
                expected: CANONICAL_POLICY_SCHEMA_VERSION,
            })
        }
    }
}

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
        found: u64,
        /// Canonical schema version accepted by the runtime.
        expected: u64,
    },
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
    pub max_values: u64,
    /// Maximum number of parents per value.
    pub max_parents_per_value: u64,
    /// Maximum number of closure traversal steps.
    pub max_closure_steps: u64,
    /// Maximum witness depth in explanations.
    pub max_witness_depth: u64,
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

#[cfg(test)]
mod tests {
    use super::{
        CANONICAL_POLICY_SCHEMA_VERSION, PolicyDefinition, PolicyLoadError,
        PolicyLoadError::UnsupportedSchemaVersion,
    };
    use rstest::rstest;

    const CANONICAL_POLICY_YAML: &str = include_str!("../../../policies/default.yaml");

    #[test]
    fn accepts_schema_version_one_yaml() {
        let policy =
            PolicyDefinition::from_yaml_str(CANONICAL_POLICY_YAML).expect("valid schema v1");

        assert_eq!(policy.schema_version, CANONICAL_POLICY_SCHEMA_VERSION);
    }

    #[test]
    fn accepts_schema_version_one_json() {
        let canonical_policy_json = r#"
            {
              "schema_version": 1,
              "policy_name": "personal_assistant_default",
              "default_action": "Deny",
              "strict_mode": true,
              "budgets": {
                "max_values": 100000,
                "max_parents_per_value": 64,
                "max_closure_steps": 10000,
                "max_witness_depth": 32
              },
              "tools": [
                {
                  "tool": "send_email",
                  "side_effect_class": "ExternalWrite",
                  "default_decision": "RequireConfirmation"
                }
              ]
            }
        "#;

        let policy =
            PolicyDefinition::from_json_str(canonical_policy_json).expect("valid schema v1");

        assert_eq!(policy.schema_version, CANONICAL_POLICY_SCHEMA_VERSION);
    }

    #[rstest]
    #[case(0_u64)]
    #[case(2_u64)]
    #[case(u64::MAX)]
    fn rejects_unknown_schema_versions(#[case] schema_version: u64) {
        let unknown_schema_policy = format!(
            concat!(
                "schema_version: {schema_version}\n",
                "policy_name: personal_assistant_default\n",
                "default_action: Deny\n",
                "strict_mode: true\n",
                "budgets:\n",
                "  max_values: 100000\n",
                "  max_parents_per_value: 64\n",
                "  max_closure_steps: 10000\n",
                "  max_witness_depth: 32\n",
                "tools: []\n"
            ),
            schema_version = schema_version,
        );

        let error =
            PolicyDefinition::from_yaml_str(&unknown_schema_policy).expect_err("must fail closed");

        assert!(matches!(
            error,
            UnsupportedSchemaVersion {
                found,
                expected
            } if found == schema_version && expected == CANONICAL_POLICY_SCHEMA_VERSION
        ));
    }

    #[test]
    fn rejects_missing_schema_version() {
        let missing_version_policy = r#"
            policy_name: personal_assistant_default
            default_action: Deny
            strict_mode: true
            budgets:
              max_values: 100000
              max_parents_per_value: 64
              max_closure_steps: 10000
              max_witness_depth: 32
            tools: []
        "#;

        let error =
            PolicyDefinition::from_yaml_str(missing_version_policy).expect_err("must fail closed");

        assert!(matches!(error, PolicyLoadError::InvalidYaml(_)));
    }

    #[test]
    fn rejects_non_numeric_schema_version() {
        let malformed_version_policy = r#"
            schema_version: "1"
            policy_name: personal_assistant_default
            default_action: Deny
            strict_mode: true
            budgets:
              max_values: 100000
              max_parents_per_value: 64
              max_closure_steps: 10000
              max_witness_depth: 32
            tools: []
        "#;

        let error = PolicyDefinition::from_yaml_str(malformed_version_policy)
            .expect_err("must fail closed");

        assert!(matches!(error, PolicyLoadError::InvalidYaml(_)));
    }

    #[test]
    fn rejects_unknown_top_level_field() {
        let policy_with_unknown_field = r#"
            schema_version: 1
            policy_name: personal_assistant_default
            default_action: Deny
            strict_mode: true
            budgets:
              max_values: 100000
              max_parents_per_value: 64
              max_closure_steps: 10000
              max_witness_depth: 32
            tools: []
            unexpected_field: true
        "#;

        let error = PolicyDefinition::from_yaml_str(policy_with_unknown_field)
            .expect_err("must fail closed on unknown top-level field");

        assert!(matches!(error, PolicyLoadError::InvalidYaml(_)));
    }

    #[test]
    fn rejects_unknown_nested_field() {
        let policy_with_unknown_nested_field = r#"
            schema_version: 1
            policy_name: personal_assistant_default
            default_action: Deny
            strict_mode: true
            budgets:
              max_values: 100000
              max_parents_per_value: 64
              max_closure_steps: 10000
              max_witness_depth: 32
              unknown_budget_field: 1
            tools: []
        "#;

        let error = PolicyDefinition::from_yaml_str(policy_with_unknown_nested_field)
            .expect_err("must fail closed on unknown nested field");

        assert!(matches!(error, PolicyLoadError::InvalidYaml(_)));
    }
}
