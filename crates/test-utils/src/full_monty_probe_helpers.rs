//! Shared helpers for full-monty probe command execution.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// Captures the output of a cargo probe command executed from the superproject
/// root.
pub struct CargoProbeOutput {
    /// Process exit code, or `None` when terminated by signal.
    pub status_code: Option<i32>,
    /// Collected UTF-8 stdout (lossy-decoded from bytes).
    pub stdout: String,
    /// Collected UTF-8 stderr (lossy-decoded from bytes).
    pub stderr: String,
}

/// Builds a `cargo test` command for a vendored `full-monty` integration test.
pub fn build_full_monty_test_command(test_binary: &str, test_args: &[&str]) -> Vec<String> {
    let mut command_args = vec![
        "test".to_owned(),
        "--manifest-path".to_owned(),
        "third_party/full-monty/Cargo.toml".to_owned(),
        "-p".to_owned(),
        "monty".to_owned(),
        "--test".to_owned(),
        test_binary.to_owned(),
    ];
    command_args.extend(test_args.iter().map(|arg| (*arg).to_owned()));
    command_args
}

/// Returns the number of passed tests from a standard Rust test summary line.
pub fn parse_passed_test_count(combined_output: &str) -> Option<usize> {
    combined_output.lines().find_map(|line| {
        let remainder = line.trim().strip_prefix("test result: ok. ")?;
        let (passed, _) = remainder.split_once(" passed")?;
        passed.parse().ok()
    })
}

/// Returns output lines that start with the supplied prefix.
pub fn prefixed_output_lines<'a>(combined_output: &'a str, prefix: &str) -> Vec<&'a str> {
    combined_output
        .lines()
        .filter(|line| line.starts_with(prefix))
        .collect()
}

/// Runs a cargo probe command under a global lock to avoid concurrent probe
/// interference.
pub fn run_cargo_probe(command_args: &[String], panic_context: &str) -> CargoProbeOutput {
    static CARGO_PROBE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let cargo_probe_guard = CARGO_PROBE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let output_result = std::process::Command::new("cargo")
        .args(command_args)
        .current_dir(superproject_root())
        .env(
            "PYO3_PYTHON",
            std::env::var("PYO3_PYTHON").unwrap_or_else(|_| "python3".to_owned()),
        )
        .output();

    let output = match output_result {
        Ok(output) => output,
        Err(error) => panic!("{panic_context}: {error}"),
    };

    drop(cargo_probe_guard);

    CargoProbeOutput {
        status_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn superproject_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
