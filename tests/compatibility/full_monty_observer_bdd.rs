//! Behavioural compatibility probe for full-monty observer-event coverage.

use std::sync::{Mutex, OnceLock};

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};

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
    world.command_args = vec![
        "test".to_owned(),
        "--manifest-path".to_owned(),
        "third_party/full-monty/Cargo.toml".to_owned(),
        "-p".to_owned(),
        "monty".to_owned(),
        "--test".to_owned(),
        "runtime_observer_events_bdd".to_owned(),
    ];
}

#[when("the probe command is executed")]
fn when_probe_command_executes(world: &mut FullMontyObserverProbeWorld) {
    static CARGO_PROBE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let cargo_probe_guard = CARGO_PROBE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let output_result = std::process::Command::new("cargo")
        .args(&world.command_args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env(
            "PYO3_PYTHON",
            std::env::var("PYO3_PYTHON").unwrap_or_else(|_| "python3".to_owned()),
        )
        .output();

    let output = match output_result {
        Ok(output) => output,
        Err(error) => panic!("probe command should execute: {error}"),
    };

    drop(cargo_probe_guard);

    world.status_code = output.status.code();
    world.stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    world.stderr = String::from_utf8_lossy(&output.stderr).into_owned();
}

#[then("the probe command succeeds")]
fn then_probe_command_succeeds(world: &FullMontyObserverProbeWorld) {
    assert_eq!(world.status_code, Some(0), "stderr:\n{}", world.stderr);
}

#[then("the probe output mentions runtime observer events")]
fn then_probe_output_mentions_observer_events(world: &FullMontyObserverProbeWorld) {
    let combined_output = format!("{}\n{}", world.stdout, world.stderr).to_lowercase();
    assert!(
        combined_output.contains("runtime_observer_events") || combined_output.contains("observer"),
        "expected observer-test output in probe logs"
    );
}

#[scenario(
    path = "tests/compatibility/features/full_monty_observer.feature",
    name = "Full-monty observer BDD suite succeeds from the superproject"
)]
fn full_monty_observer_probe(world: FullMontyObserverProbeWorld) {
    drop(world);
}
