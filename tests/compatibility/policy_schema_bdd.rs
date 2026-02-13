//! Behavioural tests validating policy schema loader compatibility contracts.

#[path = "../test_utils/policy_yaml.rs"]
mod policy_yaml;

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use zamburak_policy::{
    CANONICAL_POLICY_SCHEMA_VERSION, PolicyEngine, PolicyEngineLoadOutcome, PolicyLoadError,
};

#[derive(Default)]
struct LoaderWorld {
    policy_document: String,
    unknown_schema_version: Option<u64>,
    load_result: Option<Result<PolicyEngineLoadOutcome, PolicyLoadError>>,
}

#[fixture]
fn world() -> LoaderWorld {
    LoaderWorld::default()
}

#[given("a canonical schema v1 policy document")]
fn canonical_schema_policy(world: &mut LoaderWorld) {
    policy_yaml::canonical_policy_yaml().clone_into(&mut world.policy_document);
}

#[given("a legacy schema v0 policy document")]
fn legacy_schema_policy(world: &mut LoaderWorld) {
    let _legacy_json_fixture = policy_yaml::legacy_policy_v0_json();
    policy_yaml::legacy_policy_v0_yaml().clone_into(&mut world.policy_document);
}

#[given("a policy document with unknown schema version {schema_version:u64}")]
fn unknown_schema_policy(world: &mut LoaderWorld, schema_version: u64) {
    world.unknown_schema_version = Some(schema_version);
    world.policy_document = policy_yaml::policy_yaml_with_schema_version(schema_version);
}

#[when("the runtime loads the policy")]
fn load_policy(world: &mut LoaderWorld) {
    world.load_result = Some(PolicyEngine::from_yaml_str_with_migration_audit(
        &world.policy_document,
    ));
}

#[then("the policy loads successfully")]
fn policy_loads(world: &LoaderWorld) {
    let Some(load_result) = world.load_result.as_ref() else {
        panic!("load step must run before assertion");
    };

    assert!(
        load_result.is_ok(),
        "expected policy to load successfully, got: {load_result:?}"
    );
}

#[then("the runtime rejects the policy as unsupported schema version")]
fn policy_rejected_for_unknown_schema_version(world: &LoaderWorld) {
    let Some(expected_found_version) = world.unknown_schema_version else {
        panic!("schema_version input must be set before assertion");
    };

    let Some(load_result) = world.load_result.as_ref() else {
        panic!("load step must run before assertion");
    };

    match load_result {
        Ok(_) => panic!(
            "expected unsupported schema rejection for version {expected_found_version}, but load succeeded"
        ),
        Err(PolicyLoadError::UnsupportedSchemaVersion { found, expected }) => {
            assert_eq!(
                found.as_u64(),
                expected_found_version,
                "found schema version did not match scenario input"
            );
            assert_eq!(
                *expected, CANONICAL_POLICY_SCHEMA_VERSION,
                "expected schema version mismatch"
            );
        }
        Err(other_error) => panic!("unexpected error variant: {other_error:?}"),
    }
}

#[then(
    "migration audit records source schema version {source:u64} and target schema version {target:u64}"
)]
fn migration_audit_records_schema_versions(world: &LoaderWorld, source: u64, target: u64) {
    let load_outcome = successful_load_outcome(world);
    let migration_audit = load_outcome.migration_audit();

    assert_eq!(
        migration_audit.source_schema_version.as_u64(),
        source,
        "unexpected source schema version in migration audit"
    );
    assert_eq!(
        migration_audit.target_schema_version.as_u64(),
        target,
        "unexpected target schema version in migration audit"
    );
}

#[then("migration audit records {steps:usize} applied migration step")]
fn migration_audit_records_step_count(world: &LoaderWorld, steps: usize) {
    let load_outcome = successful_load_outcome(world);
    let migration_audit = load_outcome.migration_audit();

    assert_eq!(
        migration_audit.migration_steps.len(),
        steps,
        "unexpected migration step count"
    );
}

#[expect(
    clippy::expect_used,
    reason = "Requested by review feedback to use chained expect calls in this helper"
)]
fn successful_load_outcome(world: &LoaderWorld) -> &PolicyEngineLoadOutcome {
    let load_result = world
        .load_result
        .as_ref()
        .expect("load step must run before assertion");

    load_result
        .as_ref()
        .expect("expected successful load result")
}

#[scenario(
    path = "tests/compatibility/features/policy_schema.feature",
    name = "Load canonical schema version 1 policy"
)]
fn load_canonical_schema_policy(world: LoaderWorld) {
    assert!(world.load_result.is_some());
}

#[scenario(
    path = "tests/compatibility/features/policy_schema.feature",
    name = "Reject unknown policy schema version"
)]
fn reject_unknown_schema_policy(world: LoaderWorld) {
    assert!(world.load_result.is_some());
}

#[scenario(
    path = "tests/compatibility/features/policy_schema.feature",
    name = "Migrate supported legacy policy schema version"
)]
fn migrate_legacy_schema_policy(world: LoaderWorld) {
    assert!(world.load_result.is_some());
}

#[scenario(
    path = "tests/compatibility/features/policy_schema.feature",
    name = "Keep canonical policy load unmigrated in migration audit"
)]
fn keep_canonical_schema_unmigrated_in_audit(world: LoaderWorld) {
    assert!(world.load_result.is_some());
}
