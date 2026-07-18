//! Canonical policy schema models, migrations, and schema-version validation.

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::load_outcome::LoadOutcome;
use crate::migration::{MigrationAuditRecord, PolicyDefinitionV0};

/// Canonical schema version accepted by runtime policy loaders.
pub const CANONICAL_POLICY_SCHEMA_VERSION: SchemaVersion = SchemaVersion::new(1);

/// Canonical policy schema version wrapper to avoid integer soup in APIs.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct SchemaVersion(u64);

impl SchemaVersion {
    /// Build a schema version from a primitive value.
    #[must_use]
    pub const fn new(version: u64) -> Self {
        Self(version)
    }

    /// Return the wrapped primitive schema version.
    #[must_use]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Resource-budget limit wrapper used across policy definitions.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct BudgetLimit(u64);

impl BudgetLimit {
    /// Build a budget limit from a primitive value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Return the wrapped primitive budget value.
    #[must_use]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Outcome of loading a policy document with migration evidence.
///
/// # Examples
///
/// ```rust
/// use zamburak_policy::{PolicyDefinition, PolicyLoadError};
///
/// let policy_json = r#"{
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
/// }"#;
///
/// let load_outcome = PolicyDefinition::from_json_str_with_migration_audit(policy_json)?;
/// assert_eq!(load_outcome.policy_definition().schema_version.as_u64(), 1);
/// assert!(!load_outcome.migration_audit().was_migrated());
///
/// Ok::<(), PolicyLoadError>(())
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyLoadOutcome {
    load_outcome: LoadOutcome<PolicyDefinition>,
}

impl PolicyLoadOutcome {
    /// Return the validated canonical policy definition.
    #[must_use]
    pub const fn policy_definition(&self) -> &PolicyDefinition {
        self.load_outcome.value()
    }

    /// Return migration audit evidence for this load operation.
    #[must_use]
    pub const fn migration_audit(&self) -> &MigrationAuditRecord {
        self.load_outcome.migration_audit()
    }

    /// Consume this outcome and return both the policy and migration audit.
    #[must_use]
    pub fn into_parts(self) -> (PolicyDefinition, MigrationAuditRecord) {
        self.load_outcome.into_parts()
    }
}

/// A validated policy definition that can be used by runtime loaders.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyDefinition {
    /// Policy schema contract version.
    pub schema_version: SchemaVersion,
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

/// Generate a loader function with migration audit for a given serialization format.
macro_rules! define_loader_with_migration_audit {
    (
        fn_name: $fn_name:ident,
        param_name: $param_name:ident,
        format: $format:literal,
        parser_module: $parser:ident,
        canonical_parser: $canonical_parser:ident,
        error_variant: $error_variant:ident,
        example: $example:literal
    ) => {
        #[doc = concat!("Parse and validate a policy document from ", $format, " with migration evidence.")]
        #[doc = concat!("\n\n# Examples\n\n```rust\n", $example, "\n```")]
        pub fn $fn_name(
            $param_name: &str,
        ) -> Result<PolicyLoadOutcome, PolicyLoadError> {
            load_with_migration_audit(
                $param_name,
                MigrationLoadParsers {
                    parse_schema_version: (|value| {
                        $parser::from_str::<SchemaVersionProbe>(value)
                    }) as fn(&str) -> Result<SchemaVersionProbe, $parser::Error>,
                    parse_canonical_policy: $canonical_parser,
                    parse_legacy_policy: (|value| {
                        $parser::from_str::<PolicyDefinitionV0>(value)
                    }) as fn(&str) -> Result<PolicyDefinitionV0, $parser::Error>,
                    map_parse_error: PolicyLoadError::$error_variant,
                    _phantom: std::marker::PhantomData,
                },
            )
        }
    };
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
    /// assert_eq!(policy.schema_version.as_u64(), 1);
    ///
    /// Ok::<(), PolicyLoadError>(())
    /// ```
    pub fn from_yaml_str(policy_yaml: &str) -> Result<Self, PolicyLoadError> {
        let load_outcome = Self::from_yaml_str_with_migration_audit(policy_yaml)?;
        let (policy_definition, _migration_audit) = load_outcome.into_parts();
        Ok(policy_definition)
    }

    define_loader_with_migration_audit! {
        fn_name: from_yaml_str_with_migration_audit,
        param_name: policy_yaml,
        format: "YAML",
        parser_module: serde_yaml,
        canonical_parser: parse_canonical_yaml_policy,
        error_variant: InvalidYaml,
        example: r###"use zamburak_policy::{PolicyDefinition, PolicyLoadError};

let policy_yaml = r#"
schema_version: 1
policy_name: minimal_policy
default_action: Deny
strict_mode: true
budgets:
  max_values: 1
  max_parents_per_value: 1
  max_closure_steps: 1
  max_witness_depth: 1
tools: []
"#;

let load_outcome = PolicyDefinition::from_yaml_str_with_migration_audit(policy_yaml)?;
assert!(!load_outcome.migration_audit().was_migrated());

let (policy_definition, migration_audit) = load_outcome.into_parts();
assert_eq!(policy_definition.schema_version.as_u64(), 1);
assert_eq!(migration_audit.target_schema_version.as_u64(), 1);

Ok::<(), PolicyLoadError>(())"###
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
    /// assert_eq!(policy.schema_version.as_u64(), 1);
    ///
    /// Ok::<(), PolicyLoadError>(())
    /// ```
    pub fn from_json_str(policy_json: &str) -> Result<Self, PolicyLoadError> {
        let load_outcome = Self::from_json_str_with_migration_audit(policy_json)?;
        let (policy_definition, _migration_audit) = load_outcome.into_parts();
        Ok(policy_definition)
    }

    define_loader_with_migration_audit! {
        fn_name: from_json_str_with_migration_audit,
        param_name: policy_json,
        format: "JSON",
        parser_module: serde_json,
        canonical_parser: parse_canonical_json_policy,
        error_variant: InvalidJson,
        example: r###"use zamburak_policy::{PolicyDefinition, PolicyLoadError};

let policy_json = r#"{
  "schema_version": 1,
  "policy_name": "minimal_policy",
  "default_action": "Deny",
  "strict_mode": true,
  "budgets": {
    "max_values": 1,
    "max_parents_per_value": 1,
    "max_closure_steps": 1,
    "max_witness_depth": 1
  },
  "tools": []
}"#;

let load_outcome = PolicyDefinition::from_json_str_with_migration_audit(policy_json)?;
assert!(!load_outcome.migration_audit().was_migrated());

let (policy_definition, migration_audit) = load_outcome.into_parts();
assert_eq!(policy_definition.schema_version.as_u64(), 1);
assert_eq!(migration_audit.target_schema_version.as_u64(), 1);

Ok::<(), PolicyLoadError>(())"###
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

mod loading;
mod schema;

pub use schema::{
    ArgRule, ContextRules, PolicyAction, PolicyBudgets, PolicyLoadError, SideEffectClass,
    ToolPolicy,
};

use loading::{
    MigrationLoadParsers, SchemaVersionProbe, load_with_migration_audit,
    parse_canonical_json_policy, parse_canonical_yaml_policy,
};

#[cfg(test)]
mod tests;
