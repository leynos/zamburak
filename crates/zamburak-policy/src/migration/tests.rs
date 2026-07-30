//! Unit tests for explicit migration transforms and audit evidence.

use rstest::{fixture, rstest};

use super::{
    LEGACY_POLICY_SCHEMA_VERSION, MigrationAuditRecord, MigrationError, PolicyDefinitionV0,
    SCHEMA_MIGRATION_V0_TO_V1, SCHEMA_VERSION_V1, audit_for_canonical_policy,
    migrate_schema_v0_to_v1, to_lower_hex,
};
use crate::policy_def::{PolicyDefinition, SchemaVersion};

#[fixture]
fn legacy_policy_v0_yaml() -> &'static str {
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/test_utils/policy-v0.yaml"
    ))
}

#[fixture]
fn legacy_policy_v0(legacy_policy_v0_yaml: &str) -> PolicyDefinitionV0 {
    serde_yaml::from_str(legacy_policy_v0_yaml)
        .expect("test fixture should deserialize as schema v0")
}

#[rstest]
fn migrates_schema_v0_to_v1_with_auditable_step_record(legacy_policy_v0: PolicyDefinitionV0) {
    let policy_v0 = legacy_policy_v0;

    let migration_outcome =
        migrate_schema_v0_to_v1(policy_v0).expect("schema v0 fixture must migrate");

    assert_eq!(
        migration_outcome.policy_definition.schema_version,
        SCHEMA_VERSION_V1
    );
    assert_eq!(
        migration_outcome.migration_audit.source_schema_version,
        LEGACY_POLICY_SCHEMA_VERSION
    );
    assert_eq!(
        migration_outcome.migration_audit.target_schema_version,
        SCHEMA_VERSION_V1
    );
    assert_eq!(migration_outcome.migration_audit.migration_steps.len(), 1);

    let step = &migration_outcome.migration_audit.migration_steps[0];
    assert_eq!(step.from_schema_version, LEGACY_POLICY_SCHEMA_VERSION);
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
    let first_document = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/test_utils/ordered-policy-a.json"
    ));
    let second_document = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/test_utils/ordered-policy-b.json"
    ));

    let first_policy = PolicyDefinition::from_json_str(first_document)
        .expect("first canonical policy should parse");
    let second_policy = PolicyDefinition::from_json_str(second_document)
        .expect("second canonical policy should parse");

    let first_audit =
        audit_for_canonical_policy(&first_policy).expect("first canonical audit should succeed");
    let second_audit =
        audit_for_canonical_policy(&second_policy).expect("second canonical audit should succeed");

    assert_eq!(
        first_audit.source_document_hash,
        second_audit.source_document_hash
    );
    assert_eq!(
        first_audit.target_document_hash,
        second_audit.target_document_hash
    );
}

#[rstest]
fn migration_hashes_change_when_policy_input_changes(
    legacy_policy_v0: PolicyDefinitionV0,
    legacy_policy_v0_yaml: &str,
) {
    let baseline_policy = legacy_policy_v0;
    let changed_policy_yaml =
        legacy_policy_v0_yaml.replace("personal_assistant_default", "different_policy_name");
    let changed_policy = serde_yaml::from_str::<PolicyDefinitionV0>(&changed_policy_yaml)
        .expect("test fixture should deserialize as schema v0");

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

#[test]
fn lower_hex_preserves_leading_zeroes_and_byte_order() {
    assert_eq!(to_lower_hex(&[0x00, 0x0f, 0xff, 0xa0]), "000fffa0");
    assert_eq!(to_lower_hex(&[]), "");
}

#[test]
fn every_byte_renders_as_two_lowercase_round_tripping_digits() {
    for byte in u8::MIN..=u8::MAX {
        let rendered = to_lower_hex(&[byte]);
        assert_eq!(rendered.len(), 2, "byte {byte:#04x} must render two digits");
        assert!(
            rendered
                .bytes()
                .all(|digit| digit.is_ascii_digit() || (b'a'..=b'f').contains(&digit)),
            "byte {byte:#04x} rendered invalid output {rendered:?}",
        );
        let parsed =
            u8::from_str_radix(&rendered, 16).expect("two hexadecimal digits must parse as a byte");
        assert_eq!(parsed, byte, "round-trip mismatch for byte {byte:#04x}");
    }
}
