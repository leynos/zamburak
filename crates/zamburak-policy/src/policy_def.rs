//! Canonical policy schema models, migrations, and schema-version validation.

use core::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::load_outcome::LoadOutcome;
use crate::migration::{
    LEGACY_POLICY_SCHEMA_VERSION, MigrationAuditRecord, MigrationError, PolicyDefinitionV0,
    audit_for_canonical_policy, migrate_schema_v0_to_v1,
};

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

/// Generate a loader function with migration audit for a given serialisation format.
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

fn parse_canonical_yaml_policy(policy_yaml: &str) -> Result<PolicyDefinition, PolicyLoadError> {
    serde_yaml::from_str::<PolicyDefinition>(policy_yaml).map_err(PolicyLoadError::InvalidYaml)
}

fn parse_canonical_json_policy(policy_json: &str) -> Result<PolicyDefinition, PolicyLoadError> {
    serde_json::from_str::<PolicyDefinition>(policy_json).map_err(PolicyLoadError::InvalidJson)
}

struct MigrationLoadParsers<ParseError, VersionParser, CanonicalParser, LegacyParser, ErrorMapper> {
    parse_schema_version: VersionParser,
    parse_canonical_policy: CanonicalParser,
    parse_legacy_policy: LegacyParser,
    map_parse_error: ErrorMapper,
    _phantom: std::marker::PhantomData<ParseError>,
}

fn load_with_migration_audit<
    ParseError,
    VersionParser,
    CanonicalParser,
    LegacyParser,
    ErrorMapper,
>(
    serialized_policy: &str,
    parsers: MigrationLoadParsers<
        ParseError,
        VersionParser,
        CanonicalParser,
        LegacyParser,
        ErrorMapper,
    >,
) -> Result<PolicyLoadOutcome, PolicyLoadError>
where
    VersionParser: for<'a> Fn(&'a str) -> Result<SchemaVersionProbe, ParseError>,
    CanonicalParser: for<'a> Fn(&'a str) -> Result<PolicyDefinition, PolicyLoadError>,
    LegacyParser: for<'a> Fn(&'a str) -> Result<PolicyDefinitionV0, ParseError>,
    ErrorMapper: Fn(ParseError) -> PolicyLoadError + Copy,
{
    let version_probe =
        (parsers.parse_schema_version)(serialized_policy).map_err(parsers.map_parse_error)?;

    match version_probe.schema_version {
        Some(schema_version) if schema_version == CANONICAL_POLICY_SCHEMA_VERSION => {
            let policy_definition = (parsers.parse_canonical_policy)(serialized_policy)?;
            canonical_load_outcome(policy_definition)
        }
        Some(schema_version) if schema_version == LEGACY_POLICY_SCHEMA_VERSION => {
            let legacy_policy = (parsers.parse_legacy_policy)(serialized_policy)
                .map_err(parsers.map_parse_error)?;
            let migration_outcome = migrate_schema_v0_to_v1(legacy_policy)
                .map_err(PolicyLoadError::MigrationAuditFailed)?;
            let policy_definition = migration_outcome
                .policy_definition
                .ensure_canonical_schema_version()?;
            Ok(PolicyLoadOutcome {
                load_outcome: LoadOutcome::new(
                    policy_definition,
                    migration_outcome.migration_audit,
                ),
            })
        }
        Some(schema_version) => Err(PolicyLoadError::UnsupportedSchemaVersion {
            found: schema_version,
            expected: CANONICAL_POLICY_SCHEMA_VERSION,
        }),
        None => {
            let policy_definition = (parsers.parse_canonical_policy)(serialized_policy)?;
            canonical_load_outcome(policy_definition)
        }
    }
}

fn canonical_load_outcome(
    policy_definition: PolicyDefinition,
) -> Result<PolicyLoadOutcome, PolicyLoadError> {
    let policy_definition = policy_definition.ensure_canonical_schema_version()?;
    let migration_audit = audit_for_canonical_policy(&policy_definition)
        .map_err(PolicyLoadError::MigrationAuditFailed)?;
    Ok(PolicyLoadOutcome {
        load_outcome: LoadOutcome::new(policy_definition, migration_audit),
    })
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
struct SchemaVersionProbe {
    schema_version: Option<SchemaVersion>,
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

#[cfg(test)]
mod tests;
