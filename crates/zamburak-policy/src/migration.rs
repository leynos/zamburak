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

const SCHEMA_VERSION_V0: SchemaVersion = SchemaVersion::new(0);
const SCHEMA_VERSION_V1: SchemaVersion = SchemaVersion::new(1);
const SCHEMA_MIGRATION_V0_TO_V1: &str = "policy_schema_v0_to_v1";

/// Auditable migration evidence for a loaded policy document.
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
    Ok(MigrationAuditRecord {
        source_schema_version: SCHEMA_VERSION_V1,
        target_schema_version: SCHEMA_VERSION_V1,
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
        from_schema_version: SCHEMA_VERSION_V0,
        to_schema_version: SCHEMA_VERSION_V1,
        transform_name: SCHEMA_MIGRATION_V0_TO_V1.to_owned(),
        input_hash: source_hash.clone(),
        output_hash: target_hash.clone(),
    };

    let migration_audit = MigrationAuditRecord {
        source_schema_version: SCHEMA_VERSION_V0,
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
    Ok(format!("{digest:x}"))
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
mod tests {
    //! Unit tests for explicit migration transforms and audit evidence.

    use super::{
        MigrationAuditRecord, MigrationError, PolicyDefinitionV0, SCHEMA_MIGRATION_V0_TO_V1,
        SCHEMA_VERSION_V0, SCHEMA_VERSION_V1, audit_for_canonical_policy, migrate_schema_v0_to_v1,
    };
    use crate::policy_def::{PolicyDefinition, SchemaVersion};

    fn parse_policy_v0(policy_yaml: &str) -> PolicyDefinitionV0 {
        serde_yaml::from_str(policy_yaml).expect("test fixture should deserialize as schema v0")
    }

    #[test]
    fn migrates_schema_v0_to_v1_with_auditable_step_record() {
        let policy_v0 = parse_policy_v0(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/test_utils/policy-v0.yaml"
        )));

        let migration_outcome =
            migrate_schema_v0_to_v1(policy_v0).expect("schema v0 fixture must migrate");

        assert_eq!(
            migration_outcome.policy_definition.schema_version,
            SCHEMA_VERSION_V1
        );
        assert_eq!(
            migration_outcome.migration_audit.source_schema_version,
            SCHEMA_VERSION_V0
        );
        assert_eq!(
            migration_outcome.migration_audit.target_schema_version,
            SCHEMA_VERSION_V1
        );
        assert_eq!(migration_outcome.migration_audit.migration_steps.len(), 1);

        let step = &migration_outcome.migration_audit.migration_steps[0];
        assert_eq!(step.from_schema_version, SCHEMA_VERSION_V0);
        assert_eq!(step.to_schema_version, SCHEMA_VERSION_V1);
        assert_eq!(step.transform_name, SCHEMA_MIGRATION_V0_TO_V1);
        assert_eq!(
            step.input_hash,
            migration_outcome.migration_audit.source_document_hash
        );
        assert_eq!(
            step.output_hash,
            migration_outcome.migration_audit.target_document_hash
        );
    }

    #[test]
    fn canonical_policy_audit_has_no_migration_steps() {
        let policy_definition = PolicyDefinition::from_yaml_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../policies/default.yaml"
        )))
        .expect("default policy should parse");

        let migration_audit = audit_for_canonical_policy(&policy_definition)
            .expect("canonical policy audit should succeed");

        assert!(!migration_audit.was_migrated());
        assert_eq!(migration_audit.source_schema_version, SCHEMA_VERSION_V1);
        assert_eq!(migration_audit.target_schema_version, SCHEMA_VERSION_V1);
        assert_eq!(
            migration_audit.source_document_hash,
            migration_audit.target_document_hash
        );
    }

    #[test]
    fn canonical_audit_hashes_are_stable_for_equivalent_json_orderings() {
        let first_document = r#"{
            "schema_version": 1,
            "policy_name": "ordered",
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
        let second_document = r#"{
            "tools": [],
            "budgets": {
              "max_witness_depth": 1,
              "max_closure_steps": 1,
              "max_parents_per_value": 1,
              "max_values": 1
            },
            "strict_mode": true,
            "default_action": "Deny",
            "policy_name": "ordered",
            "schema_version": 1
        }"#;

        let first_policy = PolicyDefinition::from_json_str(first_document)
            .expect("first canonical policy should parse");
        let second_policy = PolicyDefinition::from_json_str(second_document)
            .expect("second canonical policy should parse");

        let first_audit = audit_for_canonical_policy(&first_policy)
            .expect("first canonical audit should succeed");
        let second_audit = audit_for_canonical_policy(&second_policy)
            .expect("second canonical audit should succeed");

        assert_eq!(
            first_audit.source_document_hash,
            second_audit.source_document_hash
        );
        assert_eq!(
            first_audit.target_document_hash,
            second_audit.target_document_hash
        );
    }

    #[test]
    fn migration_hashes_change_when_policy_input_changes() {
        let baseline_policy = parse_policy_v0(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/test_utils/policy-v0.yaml"
        )));

        let changed_policy_yaml = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/test_utils/policy-v0.yaml"
        ))
        .replace("personal_assistant_default", "different_policy_name");
        let changed_policy = parse_policy_v0(&changed_policy_yaml);

        let baseline_result =
            migrate_schema_v0_to_v1(baseline_policy).expect("baseline migration should succeed");
        let changed_result =
            migrate_schema_v0_to_v1(changed_policy).expect("changed migration should succeed");

        assert_ne!(
            baseline_result.migration_audit.source_document_hash,
            changed_result.migration_audit.source_document_hash
        );
        assert_ne!(
            baseline_result.migration_audit.target_document_hash,
            changed_result.migration_audit.target_document_hash
        );
    }

    #[test]
    fn migration_audit_record_reports_unmigrated_when_steps_are_absent() {
        let migration_audit = MigrationAuditRecord {
            source_schema_version: SchemaVersion::new(0),
            target_schema_version: SchemaVersion::new(1),
            source_document_hash: String::from("a"),
            target_document_hash: String::from("b"),
            migration_steps: vec![],
        };

        assert!(!migration_audit.was_migrated());
    }

    #[test]
    fn migration_error_keeps_serialization_source() {
        let serialization_error = serde_json::from_str::<serde_json::Value>("not json")
            .expect_err("fixture should fail json parsing");
        let error = MigrationError::HashSerializationFailed(serialization_error);
        assert!(error.to_string().contains("migration hashing"));
    }
}
