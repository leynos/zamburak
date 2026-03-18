//! Behavioural compatibility probe for full-monty Track A invariants.

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_utils::full_monty_probe_helpers;

#[derive(Default)]
struct FullMontyTrackAInvariantsWorld {
    command_args: Vec<String>,
    status_code: Option<i32>,
    stdout: String,
    stderr: String,
}

#[fixture]
fn world() -> FullMontyTrackAInvariantsWorld {
    FullMontyTrackAInvariantsWorld::default()
}

#[given("a full-monty Track A invariants BDD probe command")]
fn given_track_a_probe_command(world: &mut FullMontyTrackAInvariantsWorld) {
    world.command_args =
        full_monty_probe_helpers::build_full_monty_test_command("track_a_invariants_bdd", &[]);
}

#[when("the Track A invariants probe command is executed")]
fn when_track_a_probe_executes(world: &mut FullMontyTrackAInvariantsWorld) {
    let output = full_monty_probe_helpers::run_cargo_probe(
        &world.command_args,
        "track-a invariants probe should execute",
    );
    world.status_code = output.status_code;
    world.stdout = output.stdout;
    world.stderr = output.stderr;
}

#[then("the Track A invariants probe command succeeds")]
fn then_track_a_probe_succeeds(world: &FullMontyTrackAInvariantsWorld) {
    assert_eq!(
        world.status_code,
        Some(0),
        "stderr:\n{}\nstdout:\n{}",
        world.stderr,
        world.stdout
    );
}

#[then("the probe output mentions Track A invariants coverage")]
fn then_track_a_probe_mentions_coverage(world: &FullMontyTrackAInvariantsWorld) {
    let combined_output = format!("{}\n{}", world.stdout, world.stderr);
    let passed_tests = full_monty_probe_helpers::parse_passed_test_count(&combined_output);
    assert!(
        combined_output.contains("track_a_invariants_bdd")
            && matches!(passed_tests, Some(count) if count > 0),
        "expected Track A invariants BDD output with executed tests:\n{}",
        combined_output
    );
}

#[scenario(
    path = "tests/compatibility/features/full_monty_track_a_invariants.feature",
    name = "Full-monty Track A invariants BDD suite succeeds from the superproject"
)]
fn full_monty_track_a_invariants_probe(world: FullMontyTrackAInvariantsWorld) {
    drop(world);
}
