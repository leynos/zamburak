//! Behavioural compatibility probe for full-monty observer-event coverage.

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_utils::full_monty_probe_helpers;

#[derive(Default)]
struct FullMontyObserverProbeWorld {
    command_args: Vec<String>,
    status_code: Option<i32>,
    stdout: String,
    stderr: String,
}

#[fixture]
fn world() -> FullMontyObserverProbeWorld {
    FullMontyObserverProbeWorld::default()
}

#[given("a full-monty observer BDD probe command")]
fn given_observer_probe_command(world: &mut FullMontyObserverProbeWorld) {
    world.command_args =
        full_monty_probe_helpers::build_full_monty_test_command("runtime_observer_events_bdd", &[]);
}

#[when("the probe command is executed")]
fn when_probe_command_executes(world: &mut FullMontyObserverProbeWorld) {
    let output = full_monty_probe_helpers::run_cargo_probe(
        &world.command_args,
        "probe command should execute",
    );
    world.status_code = output.status_code;
    world.stdout = output.stdout;
    world.stderr = output.stderr;
}

#[then("the probe command succeeds")]
fn then_probe_command_succeeds(world: &FullMontyObserverProbeWorld) {
    assert_eq!(world.status_code, Some(0), "stderr:\n{}", world.stderr);
}

#[then("the probe output mentions runtime observer events")]
fn then_probe_output_mentions_observer_events(world: &FullMontyObserverProbeWorld) {
    let combined_output = format!("{}\n{}", world.stdout, world.stderr);
    assert!(
        combined_output.contains("runtime_observer_events_bdd.rs")
            && combined_output.contains("test result: ok."),
        "expected runtime_observer_events_bdd.rs test result line in probe logs"
    );
}

#[scenario(
    path = "tests/compatibility/features/full_monty_observer.feature",
    name = "Full-monty observer BDD suite succeeds from the superproject"
)]
fn full_monty_observer_probe(world: FullMontyObserverProbeWorld) {
    drop(world);
}
