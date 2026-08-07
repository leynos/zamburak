//! Explicit policy-schema migration transforms and migration audit metadata.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::policy_def::{
    ArgRule, ContextRules, PolicyAction, PolicyBudgets, PolicyDefinition, SchemaVersion,
    SideEffectClass, ToolPolicy,
};

pub(crate) const LEGACY_POLICY_SCHEMA_VERSION: SchemaVersion = SchemaVersion::new(0);
const SCHEMA_VERSION_V1: SchemaVersion = SchemaVersion::new(1);
const SCHEMA_MIGRATION_V0_TO_V1: &str = "policy_schema_v0_to_v1";

/// Auditable migration evidence for a loaded policy document.
///
/// # Examples
///
/// ```rust
/// use zamburak_policy::PolicyDefinition;
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
/// let load_outcome = PolicyDefinition::from_yaml_str_with_migration_audit(policy_yaml)?;
/// let audit = load_outcome.migration_audit();
/// assert_eq!(audit.source_schema_version.as_u64(), 1);
/// assert_eq!(audit.target_schema_version.as_u64(), 1);
/// assert!(!audit.was_migrated());
///
/// Ok::<(), zamburak_policy::PolicyLoadError>(())
/// ```
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MigrationAuditRecord {
    /// Source schema version observed in the input policy document.
    pub source_schema_version: SchemaVersion,
    /// Canonical schema version produced for runtime enforcement.
    pub target_schema_version: SchemaVersion,
    /// Deterministic hash of canonicalized source policy content.
    pub source_document_hash: String,
    /// Deterministic hash of canonicalized canonical-policy content.
    pub target_document_hash: String,
    /// Ordered migration steps that were applied.
    pub migration_steps: Vec<MigrationStepRecord>,
}

impl MigrationAuditRecord {
    /// Return `true` when at least one migration transform was executed.
    #[must_use]
    pub fn was_migrated(&self) -> bool {
        !self.migration_steps.is_empty()
    }
}

/// Auditable evidence for one explicit schema migration transform.
///
/// # Examples
///
/// ```rust
/// use zamburak_policy::PolicyDefinition;
///
/// let legacy_policy_yaml = r#"
/// schema_version: 0
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
/// let load_outcome =
///     PolicyDefinition::from_yaml_str_with_migration_audit(legacy_policy_yaml)?;
/// let migration_step = &load_outcome.migration_audit().migration_steps[0];
///
/// assert_eq!(migration_step.from_schema_version.as_u64(), 0);
/// assert_eq!(migration_step.to_schema_version.as_u64(), 1);
/// assert_eq!(migration_step.transform_name, "policy_schema_v0_to_v1");
///
/// Ok::<(), zamburak_policy::PolicyLoadError>(())
/// ```
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MigrationStepRecord {
    /// Source schema version before this step.
    pub from_schema_version: SchemaVersion,
    /// Target schema version after this step.
    pub to_schema_version: SchemaVersion,
    /// Stable transform identifier.
    pub transform_name: String,
    /// Deterministic hash of canonicalized step input.
    pub input_hash: String,
    /// Deterministic hash of canonicalized step output.
    pub output_hash: String,
}

/// Errors encountered while building migration evidence.
#[derive(Debug, Error)]
pub enum MigrationError {
    /// Migration-evidence hashing could not serialize the policy payload.
    #[error("failed to serialize policy payload during migration hashing: {0}")]
    HashSerializationFailed(#[source] serde_json::Error),
}

/// Result of migration execution before canonical schema validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MigrationOutcome {
    pub(crate) policy_definition: PolicyDefinition,
    pub(crate) migration_audit: MigrationAuditRecord,
}

/// Legacy schema v0 policy format supported for explicit migration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PolicyDefinitionV0 {
    pub(crate) schema_version: SchemaVersion,
    pub(crate) policy_name: String,
    pub(crate) default_action: PolicyAction,
    pub(crate) strict_mode: bool,
    pub(crate) budgets: PolicyBudgets,
    pub(crate) tools: Vec<ToolPolicyV0>,
}

/// Legacy schema v0 per-tool policy format.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ToolPolicyV0 {
    pub(crate) name: String,
    pub(crate) side_effect: SideEffectClass,
    #[serde(default)]
    pub(crate) authority: Vec<String>,
    #[serde(default)]
    pub(crate) args: Vec<ArgRuleV0>,
    #[serde(default)]
    pub(crate) context: Option<ContextRules>,
    pub(crate) default_decision: PolicyAction,
}

/// Legacy schema v0 argument-rule format.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArgRuleV0 {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) requires_integrity: Option<String>,
    #[serde(default)]
    pub(crate) forbid_confidentiality: Vec<String>,
}

/// Build an audit record for canonical input that required no migration steps.
pub(crate) fn audit_for_canonical_policy(
    policy_definition: &PolicyDefinition,
) -> Result<MigrationAuditRecord, MigrationError> {
    let canonical_hash = stable_policy_hash(policy_definition)?;
    let schema_version = policy_definition.schema_version;
    Ok(MigrationAuditRecord {
        source_schema_version: schema_version,
        target_schema_version: schema_version,
        source_document_hash: canonical_hash.clone(),
        target_document_hash: canonical_hash,
        migration_steps: Vec::new(),
    })
}

/// Execute the explicit v0-to-v1 schema migration transform.
pub(crate) fn migrate_schema_v0_to_v1(
    source_policy: PolicyDefinitionV0,
) -> Result<MigrationOutcome, MigrationError> {
    let source_hash = stable_policy_hash(&source_policy)?;
    let migrated_policy = PolicyDefinition {
        schema_version: SCHEMA_VERSION_V1,
        policy_name: source_policy.policy_name,
        default_action: source_policy.default_action,
        strict_mode: source_policy.strict_mode,
        budgets: source_policy.budgets,
        tools: source_policy
            .tools
            .into_iter()
            .map(map_tool_policy_v0_to_v1)
            .collect(),
    };
    let target_hash = stable_policy_hash(&migrated_policy)?;

    let migration_step = MigrationStepRecord {
        from_schema_version: LEGACY_POLICY_SCHEMA_VERSION,
        to_schema_version: SCHEMA_VERSION_V1,
        transform_name: SCHEMA_MIGRATION_V0_TO_V1.to_owned(),
        input_hash: source_hash.clone(),
        output_hash: target_hash.clone(),
    };

    let migration_audit = MigrationAuditRecord {
        source_schema_version: LEGACY_POLICY_SCHEMA_VERSION,
        target_schema_version: SCHEMA_VERSION_V1,
        source_document_hash: source_hash,
        target_document_hash: target_hash,
        migration_steps: vec![migration_step],
    };

    Ok(MigrationOutcome {
        policy_definition: migrated_policy,
        migration_audit,
    })
}

fn map_tool_policy_v0_to_v1(tool_policy_v0: ToolPolicyV0) -> ToolPolicy {
    ToolPolicy {
        tool: tool_policy_v0.name,
        side_effect_class: tool_policy_v0.side_effect,
        required_authority: tool_policy_v0.authority,
        arg_rules: tool_policy_v0
            .args
            .into_iter()
            .map(|arg_rule| ArgRule {
                arg: arg_rule.name,
                requires_integrity: arg_rule.requires_integrity,
                forbids_confidentiality: arg_rule.forbid_confidentiality,
            })
            .collect(),
        context_rules: tool_policy_v0.context,
        default_decision: tool_policy_v0.default_decision,
    }
}

fn stable_policy_hash<T>(policy: &T) -> Result<String, MigrationError>
where
    T: Serialize,
{
    let json_value =
        serde_json::to_value(policy).map_err(MigrationError::HashSerializationFailed)?;
    let canonicalized_json = canonicalize_json_value(&json_value);
    let canonical_json_bytes =
        serde_json::to_vec(&canonicalized_json).map_err(MigrationError::HashSerializationFailed)?;
    let digest = Sha256::digest(canonical_json_bytes);
    Ok(to_lower_hex(&digest))
}

/// Encode `bytes` as a lowercase hexadecimal string.
///
/// Every byte renders as exactly two digits, including leading zeroes, so the
/// output is always twice the input length.
///
/// # Examples
///
/// `to_lower_hex(&[0x00, 0xaf])` returns `"00af"`.
#[must_use]
fn to_lower_hex(bytes: &[u8]) -> String {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut hex = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        hex.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
        hex.push(char::from(HEX_DIGITS[usize::from(byte & 0x0f)]));
    }
    hex
}

fn canonicalize_json_value(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let sorted_entries = object
                .iter()
                .fold(BTreeMap::new(), |mut acc, (key, value)| {
                    acc.insert(key.clone(), canonicalize_json_value(value));
                    acc
                });
            let canonicalized_object =
                sorted_entries
                    .into_iter()
                    .fold(Map::new(), |mut canonicalized, (key, value)| {
                        canonicalized.insert(key, value);
                        canonicalized
                    });
            Value::Object(canonicalized_object)
        }
        Value::Array(values) => Value::Array(values.iter().map(canonicalize_json_value).collect()),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests;
