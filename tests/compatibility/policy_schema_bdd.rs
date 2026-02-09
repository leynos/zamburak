use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use zamburak_policy::{PolicyEngine, PolicyLoadError};

#[derive(Default)]
struct LoaderWorld {
    policy_document: String,
    load_result: Option<Result<PolicyEngine, PolicyLoadError>>,
}

#[fixture]
fn world() -> LoaderWorld {
    LoaderWorld::default()
}

#[given("a canonical schema v1 policy document")]
fn canonical_schema_policy(world: &mut LoaderWorld) {
    canonical_policy_yaml().clone_into(&mut world.policy_document);
}

#[given("a policy document with unknown schema version {schema_version:u64}")]
fn unknown_schema_policy(world: &mut LoaderWorld, schema_version: u64) {
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

    assert!(load_result.is_ok());
}

#[then("the runtime rejects the policy as unsupported schema version")]
fn policy_rejected_for_unknown_schema_version(world: &LoaderWorld) {
    let Some(load_result) = world.load_result.as_ref() else {
        panic!("load step must run before assertion");
    };

    assert!(matches!(
        load_result,
        Err(PolicyLoadError::UnsupportedSchemaVersion {
            found: 2,
            expected: 1
        })
    ));
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

const fn canonical_policy_yaml() -> &'static str {
    r"
schema_version: 1
policy_name: personal_assistant_default
default_action: Deny
strict_mode: true
budgets:
  max_values: 100000
  max_parents_per_value: 64
  max_closure_steps: 10000
  max_witness_depth: 32
tools:
  - tool: send_email
    side_effect_class: ExternalWrite
    required_authority: [EmailSendCap]
    arg_rules:
      - arg: body
        forbids_confidentiality: [AUTH_SECRET]
    context_rules:
      deny_if_pc_integrity_contains: [Untrusted]
    default_decision: RequireConfirmation
"
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
