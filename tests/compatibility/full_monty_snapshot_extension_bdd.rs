//! Behavioural compatibility probe for full-monty snapshot extension coverage.

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_utils::full_monty_observer_probe_helpers;

#[derive(Default)]
struct FullMontySnapshotExtensionProbeWorld {
    command_args: Vec<String>,
    status_code: Option<i32>,
    stdout: String,
    stderr: String,
}

#[fixture]
fn world() -> FullMontySnapshotExtensionProbeWorld {
    FullMontySnapshotExtensionProbeWorld::default()
}

#[given("a full-monty snapshot extension BDD probe command")]
fn given_snapshot_extension_probe_command(world: &mut FullMontySnapshotExtensionProbeWorld) {
    world.command_args = vec![
        "test".to_owned(),
        "--manifest-path".to_owned(),
        "third_party/full-monty/Cargo.toml".to_owned(),
        "-p".to_owned(),
        "monty".to_owned(),
        "--test".to_owned(),
        "snapshot_extensions_bdd".to_owned(),
    ];
}

#[when("the snapshot extension probe command is executed")]
fn when_snapshot_extension_probe_executes(world: &mut FullMontySnapshotExtensionProbeWorld) {
    let output = full_monty_observer_probe_helpers::run_cargo_probe(
        &world.command_args,
        "probe command should execute",
    );
    world.status_code = output.status_code;
    world.stdout = output.stdout;
    world.stderr = output.stderr;
}

#[then("the snapshot extension probe command succeeds")]
fn then_snapshot_extension_probe_succeeds(world: &FullMontySnapshotExtensionProbeWorld) {
    assert_eq!(world.status_code, Some(0), "stderr:\n{}", world.stderr);
}

#[then("the probe output mentions snapshot extension coverage")]
fn then_probe_output_mentions_snapshot_extensions(world: &FullMontySnapshotExtensionProbeWorld) {
    let combined_output = format!("{}\n{}", world.stdout, world.stderr);
    assert!(
        combined_output.contains("snapshot_extensions_bdd"),
        "expected snapshot_extensions_bdd binary name in probe logs"
    );
}

#[scenario(
    path = "tests/compatibility/features/full_monty_snapshot_extension.feature",
    name = "Full-monty snapshot extension BDD suite succeeds from the superproject"
)]
fn full_monty_snapshot_extension_probe(world: FullMontySnapshotExtensionProbeWorld) {
    drop(world);
}
