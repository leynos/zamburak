//! Security regression probe for full-monty observer error-return behaviour.

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_utils::full_monty_probe_helpers;

#[derive(Default)]
struct FullMontyObserverSecurityWorld {
    command_args: Vec<String>,
    status_code: Option<i32>,
    stdout: String,
    stderr: String,
}

#[fixture]
fn world() -> FullMontyObserverSecurityWorld {
    FullMontyObserverSecurityWorld::default()
}

#[given("a full-monty observer error-path probe command")]
fn given_error_path_probe_command(world: &mut FullMontyObserverSecurityWorld) {
    world.command_args = full_monty_probe_helpers::build_full_monty_test_command(
        "runtime_observer_events",
        &[
            "runtime_observer_emits_external_return_kinds::case_2_error",
            "--",
            "--exact",
        ],
    );
}

#[when("the security probe command is executed")]
fn when_security_probe_executes(world: &mut FullMontyObserverSecurityWorld) {
    let output = full_monty_probe_helpers::run_cargo_probe(
        &world.command_args,
        "security probe command should execute",
    );
    world.status_code = output.status_code;
    world.stdout = output.stdout;
    world.stderr = output.stderr;
}

#[then("the security probe command succeeds")]
fn then_security_probe_succeeds(world: &FullMontyObserverSecurityWorld) {
    assert_eq!(world.status_code, Some(0), "stderr:\n{}", world.stderr);
}

#[then("the security probe output mentions error return coverage")]
fn then_security_probe_mentions_error_return(world: &FullMontyObserverSecurityWorld) {
    let combined_output = format!("{}\n{}", world.stdout, world.stderr);
    assert!(
        combined_output.contains("running 1 test")
            && combined_output
                .contains("test runtime_observer_emits_external_return_kinds::case_2_error ... ok"),
        "expected exact error-return probe test result line"
    );
}

#[scenario(
    path = "tests/security/features/full_monty_observer_security.feature",
    name = "Full-monty observer error-path regression succeeds from the superproject"
)]
fn full_monty_observer_error_probe(world: FullMontyObserverSecurityWorld) {
    drop(world);
}
