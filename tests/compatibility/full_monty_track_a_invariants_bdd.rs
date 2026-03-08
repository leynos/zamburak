//! Behavioural compatibility probe for full-monty Track A invariants.

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_utils::full_monty_observer_probe_helpers;

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
    world.command_args = vec![
        "test".to_owned(),
        "--manifest-path".to_owned(),
        "third_party/full-monty/Cargo.toml".to_owned(),
        "-p".to_owned(),
        "monty".to_owned(),
        "--test".to_owned(),
        "track_a_invariants_bdd".to_owned(),
    ];
}

#[when("the Track A invariants probe command is executed")]
fn when_track_a_probe_executes(world: &mut FullMontyTrackAInvariantsWorld) {
    let output = full_monty_observer_probe_helpers::run_cargo_probe(
        &world.command_args,
        "track-a invariants probe should execute",
    );
    world.status_code = output.status_code;
    world.stdout = output.stdout;
    world.stderr = output.stderr;
}

#[then("the Track A invariants probe command succeeds")]
fn then_track_a_probe_succeeds(world: &FullMontyTrackAInvariantsWorld) {
    assert_eq!(world.status_code, Some(0), "stderr:\n{}", world.stderr);
}

#[then("the probe output mentions Track A invariants coverage")]
fn then_track_a_probe_mentions_coverage(world: &FullMontyTrackAInvariantsWorld) {
    let combined_output = format!("{}\n{}", world.stdout, world.stderr);
    assert!(
        combined_output.contains("track_a_invariants_bdd")
            && combined_output.contains("test result: ok."),
        "expected Track A invariants BDD output in probe logs"
    );
}

#[scenario(
    path = "tests/compatibility/features/full_monty_track_a_invariants.feature",
    name = "Full-monty Track A invariants BDD suite succeeds from the superproject"
)]
fn full_monty_track_a_invariants_probe(world: FullMontyTrackAInvariantsWorld) {
    drop(world);
}
