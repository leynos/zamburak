//! Security conformance tests for policy-schema migration.

#[path = "../test_utils/policy_yaml.rs"]
mod policy_yaml;

use zamburak_policy::{PolicyEngine, PolicyEngineLoadOutcome, PolicyLoadError};

#[test]
fn migration_hashes_are_stable_for_equivalent_legacy_json_orderings() {
    let first_legacy_json = policy_yaml::legacy_policy_v0_json();
    let second_legacy_json = r#"{
        "tools": [
          {
            "default_decision": "RequireConfirmation",
            "context": { "deny_if_pc_integrity_contains": ["Untrusted"] },
            "args": [
              {
                "forbid_confidentiality": ["AUTH_SECRET"],
                "name": "body"
              }
            ],
            "authority": ["EmailSendCap"],
            "side_effect": "ExternalWrite",
            "name": "send_email"
          }
        ],
        "budgets": {
          "max_witness_depth": 32,
          "max_closure_steps": 10000,
          "max_parents_per_value": 64,
          "max_values": 100000
        },
        "strict_mode": true,
        "default_action": "Deny",
        "policy_name": "personal_assistant_default",
        "schema_version": 0
    }"#;

    let first_load = load_engine_with_audit_from_json(first_legacy_json);
    let second_load = load_engine_with_audit_from_json(second_legacy_json);

    assert_eq!(
        first_load.migration_audit().source_document_hash,
        second_load.migration_audit().source_document_hash,
        "source hashes should be stable for equivalent JSON content"
    );
    assert_eq!(
        first_load.migration_audit().target_document_hash,
        second_load.migration_audit().target_document_hash,
        "target hashes should be stable for equivalent JSON content"
    );
    assert_eq!(
        first_load.migration_audit().migration_steps,
        second_load.migration_audit().migration_steps,
        "step-level migration audit must be deterministic"
    );
}

#[test]
fn migration_hashes_change_when_legacy_input_changes() {
    let baseline_json = policy_yaml::legacy_policy_v0_json();
    let changed_json =
        baseline_json.replace("personal_assistant_default", "personal_assistant_changed");

    let baseline_load = load_engine_with_audit_from_json(baseline_json);
    let changed_load = load_engine_with_audit_from_json(&changed_json);

    assert_ne!(
        baseline_load.migration_audit().source_document_hash,
        changed_load.migration_audit().source_document_hash,
        "source hash should change when legacy policy content changes"
    );
    assert_ne!(
        baseline_load.migration_audit().target_document_hash,
        changed_load.migration_audit().target_document_hash,
        "target hash should change when migrated policy content changes"
    );
}

#[test]
fn unsupported_schema_version_remains_fail_closed() {
    let unsupported_policy = policy_yaml::policy_yaml_with_schema_version(9);

    let Err(load_error) = PolicyEngine::from_yaml_str_with_migration_audit(&unsupported_policy)
    else {
        panic!("unsupported schema should not load");
    };

    assert!(matches!(
        load_error,
        PolicyLoadError::UnsupportedSchemaVersion { found, .. } if found.as_u64() == 9
    ));
}

#[test]
fn migrated_policy_remains_restrictive_equivalent_to_canonical_fixture() {
    let canonical_engine = match PolicyEngine::from_yaml_str(policy_yaml::canonical_policy_yaml()) {
        Ok(policy_engine) => policy_engine,
        Err(load_error) => panic!("canonical fixture should load: {load_error:?}"),
    };
    let migrated_engine = match PolicyEngine::from_yaml_str(policy_yaml::legacy_policy_v0_yaml()) {
        Ok(policy_engine) => policy_engine,
        Err(load_error) => panic!("legacy fixture should migrate and load: {load_error:?}"),
    };

    assert_eq!(
        migrated_engine.policy_definition(),
        canonical_engine.policy_definition(),
        "migrated policy must be restrictive-equivalent to canonical fixture"
    );
}

fn load_engine_with_audit_from_json(policy_json: &str) -> PolicyEngineLoadOutcome {
    match PolicyEngine::from_json_str_with_migration_audit(policy_json) {
        Ok(load_outcome) => load_outcome,
        Err(load_error) => panic!("policy should load with migration audit: {load_error:?}"),
    }
}
