//! Behavioural tests validating policy schema loader compatibility contracts.

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use zamburak_policy::{CANONICAL_POLICY_SCHEMA_VERSION, PolicyEngine, PolicyLoadError};

#[derive(Default)]
struct LoaderWorld {
    policy_document: String,
    unknown_schema_version: Option<u64>,
    load_result: Option<Result<PolicyEngine, PolicyLoadError>>,
}

const CANONICAL_POLICY_YAML: &str = include_str!("../../policies/default.yaml");

#[fixture]
fn world() -> LoaderWorld {
    LoaderWorld::default()
}

#[given("a canonical schema v1 policy document")]
fn canonical_schema_policy(world: &mut LoaderWorld) {
    CANONICAL_POLICY_YAML.clone_into(&mut world.policy_document);
}

#[given("a policy document with unknown schema version {schema_version:u64}")]
fn unknown_schema_policy(world: &mut LoaderWorld, schema_version: u64) {
    world.unknown_schema_version = Some(schema_version);
    world.policy_document = unknown_schema_policy_yaml(schema_version);
}

#[when("the runtime loads the policy")]
fn load_policy(world: &mut LoaderWorld) {
    world.load_result = Some(PolicyEngine::from_yaml_str(&world.policy_document));
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
            "expected unsupported schema rejection for version \
             {expected_found_version}, but load succeeded"
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

fn unknown_schema_policy_yaml(schema_version: u64) -> String {
    format!(
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
    )
}
