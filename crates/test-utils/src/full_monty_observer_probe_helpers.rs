//! Shared helpers for full-monty observer probe command execution.

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
