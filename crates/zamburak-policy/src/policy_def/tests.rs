//! Unit tests for policy schema loading, migration, and fail-closed behaviour.

mod policy_yaml {
    //! Shared policy fixtures and schema-version helpers for tests.

    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/test_utils/policy_yaml.rs"
    ));
}

use super::{
    CANONICAL_POLICY_SCHEMA_VERSION, PolicyDefinition, PolicyLoadError,
    PolicyLoadError::UnsupportedSchemaVersion, SchemaVersion,
};
use rstest::rstest;

#[test]
fn accepts_schema_version_one_yaml() {
    let policy = PolicyDefinition::from_yaml_str(policy_yaml::canonical_policy_yaml())
        .expect("valid schema v1");

    assert_eq!(
        policy.schema_version,
        SchemaVersion::new(CANONICAL_POLICY_SCHEMA_VERSION.as_u64())
    );
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

    let policy = PolicyDefinition::from_json_str(canonical_policy_json).expect("valid schema v1");

    assert_eq!(
        policy.schema_version,
        SchemaVersion::new(CANONICAL_POLICY_SCHEMA_VERSION.as_u64())
    );
}

#[test]
fn migrates_legacy_schema_version_zero_yaml_to_canonical_schema() {
    let load_outcome =
        PolicyDefinition::from_yaml_str_with_migration_audit(policy_yaml::legacy_policy_v0_yaml())
            .expect("legacy schema v0 should migrate");

    assert_eq!(
        load_outcome.policy_definition().schema_version,
        SchemaVersion::new(CANONICAL_POLICY_SCHEMA_VERSION.as_u64())
    );
    assert!(load_outcome.migration_audit().was_migrated());
    assert_eq!(load_outcome.migration_audit().migration_steps.len(), 1);
    assert_eq!(
        load_outcome
            .migration_audit()
            .source_schema_version
            .as_u64(),
        0
    );
    assert_eq!(
        load_outcome
            .migration_audit()
            .target_schema_version
            .as_u64(),
        1
    );
}

#[test]
fn migrates_legacy_schema_version_zero_json_to_canonical_schema() {
    let load_outcome =
        PolicyDefinition::from_json_str_with_migration_audit(&policy_yaml::legacy_policy_v0_json())
            .expect("legacy schema v0 json should migrate");

    assert!(load_outcome.migration_audit().was_migrated());
    assert_eq!(
        load_outcome.policy_definition().schema_version,
        SchemaVersion::new(CANONICAL_POLICY_SCHEMA_VERSION.as_u64())
    );
}

#[test]
fn canonical_load_with_migration_audit_reports_no_migration_steps() {
    let load_outcome =
        PolicyDefinition::from_yaml_str_with_migration_audit(policy_yaml::canonical_policy_yaml())
            .expect("canonical schema v1 should load");

    assert!(!load_outcome.migration_audit().was_migrated());
    assert!(load_outcome.migration_audit().migration_steps.is_empty());
    assert_eq!(
        load_outcome.migration_audit().source_document_hash,
        load_outcome.migration_audit().target_document_hash
    );
}

#[rstest]
#[case(2_u64)]
#[case(u64::MAX)]
fn rejects_unknown_schema_versions(#[case] schema_version: u64) {
    let unknown_schema_policy = policy_yaml::policy_yaml_with_schema_version(schema_version);

    let error =
        PolicyDefinition::from_yaml_str(&unknown_schema_policy).expect_err("must fail closed");

    assert!(matches!(
        error,
        UnsupportedSchemaVersion {
            found,
            expected
        } if found.as_u64() == schema_version
            && expected == CANONICAL_POLICY_SCHEMA_VERSION
    ));
}

#[rstest]
#[case("", "", "", "must fail closed on missing schema version")]
#[case(
    "schema_version: \"1\"\n",
    "",
    "",
    "must fail closed on non-numeric schema version"
)]
#[case(
    "schema_version: 1\n",
    "unexpected_field: true\n",
    "",
    "must fail closed on unknown top-level field"
)]
#[case(
    "schema_version: 1\n",
    "",
    "  unknown_budget_field: 1\n",
    "must fail closed on unknown nested field"
)]
fn rejects_invalid_policy_shapes(
    #[case] schema_version_line: &str,
    #[case] top_level_extra: &str,
    #[case] budget_extra: &str,
    #[case] expectation_message: &str,
) {
    let invalid_policy_yaml = format!(
        concat!(
            "{schema_version_line}",
            "policy_name: personal_assistant_default\n",
            "default_action: Deny\n",
            "strict_mode: true\n",
            "budgets:\n",
            "  max_values: 100000\n",
            "  max_parents_per_value: 64\n",
            "  max_closure_steps: 10000\n",
            "  max_witness_depth: 32\n",
            "{budget_extra}",
            "tools: []\n",
            "{top_level_extra}"
        ),
        schema_version_line = schema_version_line,
        budget_extra = budget_extra,
        top_level_extra = top_level_extra,
    );

    let error =
        PolicyDefinition::from_yaml_str(&invalid_policy_yaml).expect_err(expectation_message);

    assert!(matches!(error, PolicyLoadError::InvalidYaml(_)));
}
