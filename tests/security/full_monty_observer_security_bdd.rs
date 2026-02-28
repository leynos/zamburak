//! Security regression probe for full-monty observer error-return behaviour.

use std::sync::{Mutex, OnceLock};

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};

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
    world.command_args = vec![
        "test".to_owned(),
        "--manifest-path".to_owned(),
        "third_party/full-monty/Cargo.toml".to_owned(),
        "-p".to_owned(),
        "monty".to_owned(),
        "--test".to_owned(),
        "runtime_observer_events".to_owned(),
        "runtime_observer_emits_error_return_for_failed_external_call".to_owned(),
        "--".to_owned(),
        "--exact".to_owned(),
    ];
}

#[when("the security probe command is executed")]
fn when_security_probe_executes(world: &mut FullMontyObserverSecurityWorld) {
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
        Err(error) => panic!("security probe command should execute: {error}"),
    };

    drop(cargo_probe_guard);

    world.status_code = output.status.code();
    world.stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    world.stderr = String::from_utf8_lossy(&output.stderr).into_owned();
}

#[then("the security probe command succeeds")]
fn then_security_probe_succeeds(world: &FullMontyObserverSecurityWorld) {
    assert_eq!(world.status_code, Some(0), "stderr:\n{}", world.stderr);
}

#[then("the security probe output mentions error return coverage")]
fn then_security_probe_mentions_error_return(world: &FullMontyObserverSecurityWorld) {
    let combined_output = format!("{}\n{}", world.stdout, world.stderr).to_lowercase();
    assert!(
        combined_output.contains("runtime_observer_emits_error_return_for_failed_external_call")
            || combined_output.contains("error_return"),
        "expected error-path observer coverage output"
    );
}

#[scenario(
    path = "tests/security/features/full_monty_observer_security.feature",
    name = "Full-monty observer error-path regression succeeds from the superproject"
)]
fn full_monty_observer_error_probe(world: FullMontyObserverSecurityWorld) {
    drop(world);
}
