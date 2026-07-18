//! Unit tests for policy schema loading, migration, and fail-closed behaviour.

#[path = "../../../../tests/test_utils/policy_yaml.rs"]
mod policy_yaml;

use super::{
    BudgetLimit, CANONICAL_POLICY_SCHEMA_VERSION, PolicyAction, PolicyBudgets, PolicyDefinition,
    PolicyLoadError, PolicyLoadError::UnsupportedSchemaVersion, PolicyLoadOutcome, SchemaVersion,
};
use rstest::rstest;

const CANONICAL_POLICY_JSON: &str = r#"
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

#[rstest]
#[case::yaml(
    policy_yaml::canonical_policy_yaml(),
    PolicyDefinition::from_yaml_str,
    "valid schema v1 yaml"
)]
#[case::json(
    CANONICAL_POLICY_JSON,
    PolicyDefinition::from_json_str,
    "valid schema v1 json"
)]
fn accepts_schema_version_one(
    #[case] policy_document: &str,
    #[case] loader: fn(&str) -> Result<PolicyDefinition, PolicyLoadError>,
    #[case] expectation_message: &str,
) {
    let policy = loader(policy_document).expect(expectation_message);

    assert_eq!(
        policy.schema_version,
        SchemaVersion::new(CANONICAL_POLICY_SCHEMA_VERSION.as_u64())
    );
}

#[rstest]
#[case::yaml(
    policy_yaml::legacy_policy_v0_yaml(),
    PolicyDefinition::from_yaml_str_with_migration_audit,
    "legacy schema v0 yaml should migrate"
)]
#[case::json(
    policy_yaml::legacy_policy_v0_json(),
    PolicyDefinition::from_json_str_with_migration_audit,
    "legacy schema v0 json should migrate"
)]
fn migrates_legacy_schema_version_zero_to_canonical_schema(
    #[case] policy_document: &str,
    #[case] loader: fn(&str) -> Result<PolicyLoadOutcome, PolicyLoadError>,
    #[case] expectation_message: &str,
) {
    let load_outcome = loader(policy_document).expect(expectation_message);

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
fn canonical_load_with_migration_audit_reports_no_migration_steps() {
    let load_outcome =
        PolicyDefinition::from_yaml_str_with_migration_audit(policy_yaml::canonical_policy_yaml())
            .expect("canonical schema v1 should load");

    assert_eq!(
        load_outcome.migration_audit().source_document_hash,
        load_outcome.migration_audit().target_document_hash
    );
    assert!(!load_outcome.migration_audit().was_migrated());
    assert!(load_outcome.migration_audit().migration_steps.is_empty());
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

/// Budget value used throughout [`ensure_canonical_schema_version_rejects_mismatched_version`].
const SINGLE_UNIT_BUDGET: BudgetLimit = BudgetLimit::new(1);

#[test]
fn ensure_canonical_schema_version_rejects_mismatched_version() {
    let mismatched_schema_version =
        SchemaVersion::new(CANONICAL_POLICY_SCHEMA_VERSION.as_u64() + 1);
    let policy = PolicyDefinition {
        schema_version: mismatched_schema_version,
        policy_name: "mismatched".to_owned(),
        default_action: PolicyAction::Deny,
        strict_mode: true,
        budgets: PolicyBudgets {
            max_values: SINGLE_UNIT_BUDGET,
            max_parents_per_value: SINGLE_UNIT_BUDGET,
            max_closure_steps: SINGLE_UNIT_BUDGET,
            max_witness_depth: SINGLE_UNIT_BUDGET,
        },
        tools: Vec::new(),
    };

    // Exercise `BudgetLimit::as_u64` directly before `policy` is consumed below.
    assert_eq!(policy.budgets.max_values.as_u64(), 1);

    let error = policy
        .ensure_canonical_schema_version()
        .expect_err("mismatched schema version must fail closed");

    assert!(matches!(
        error,
        UnsupportedSchemaVersion { found, expected }
            if found == mismatched_schema_version && expected == CANONICAL_POLICY_SCHEMA_VERSION
    ));
    assert_eq!(
        error.to_string(),
        format!(
            "unsupported policy schema_version `{mismatched_schema_version}`; only \
             `{CANONICAL_POLICY_SCHEMA_VERSION}` is accepted"
        )
    );
}
