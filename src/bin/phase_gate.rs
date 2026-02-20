//! CI phase-gate command for verification-target enforcement.

use std::collections::BTreeSet;
use std::env;
use std::io::{self, Write};
use std::process::{Command, ExitCode};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8};
use zamburak::phase_gate_contract::{
    ESCALATION_STEPS, PhaseGateReport, PhaseGateStatus, PhaseGateTarget, RELEASE_BLOCKING_CAUSES,
    VerificationSuite, evaluate_phase_gate, parse_phase_gate_target, required_suites_for_target,
    suite_by_id,
};

const DEFAULT_TARGET_FILE: &str = ".github/phase-gate-target.txt";
const CARGO_TEST_BASE_ARGS: [&str; 4] = ["test", "--workspace", "--all-targets", "--all-features"];
type SuiteId = &'static str;

#[derive(Debug)]
enum PhaseGateCliError {
    InvalidArgument(Box<str>),
    MissingArgumentValue(Box<str>),
    InvalidTargetValue(Box<str>),
    Io {
        path: Utf8PathBuf,
        source: io::Error,
    },
    CargoListFailed,
    GateBlocked,
}

impl std::fmt::Display for PhaseGateCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArgument(flag) => write!(f, "unsupported argument `{flag}`"),
            Self::MissingArgumentValue(flag) => {
                write!(f, "missing value for argument `{flag}`")
            }
            Self::InvalidTargetValue(value) => {
                write!(
                    f,
                    "invalid phase-gate target `{value}`; expected one of: phase0, phase1, phase2, phase3, phase4, phase5, completion"
                )
            }
            Self::Io { path, source } => {
                write!(f, "I/O error for `{path}`: {source}")
            }
            Self::CargoListFailed => {
                write!(f, "failed to enumerate tests from `cargo test -- --list`")
            }
            Self::GateBlocked => write!(f, "phase-gate checks failed"),
        }
    }
}

impl std::error::Error for PhaseGateCliError {}

#[derive(Debug)]
struct CliArgs {
    target_file: Utf8PathBuf,
    explicit_target: Option<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let mut stderr = io::stderr().lock();
            discard_write_result(writeln!(stderr, "phase-gate command error: {error}"));
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), PhaseGateCliError> {
    let cli_args = parse_cli_args(env::args().skip(1).collect())?;
    let target = resolve_target(&cli_args)?;
    let test_catalog = list_available_tests()?;
    let mut failing_suite_ids = BTreeSet::new();

    let initial_report = evaluate_phase_gate(target, &test_catalog, &failing_suite_ids);
    if initial_report.status == PhaseGateStatus::MissingSuites {
        emit_failure_report(&initial_report);
        return Err(PhaseGateCliError::GateBlocked);
    }

    for suite in required_suites_for_target(target) {
        if run_suite_filter(suite.test_filter).is_err() {
            failing_suite_ids.insert(suite.id);
        }
    }

    let final_report = evaluate_phase_gate(target, &test_catalog, &failing_suite_ids);
    emit_report(&final_report);
    if final_report.status == PhaseGateStatus::Passed {
        Ok(())
    } else {
        Err(PhaseGateCliError::GateBlocked)
    }
}

fn parse_cli_args(raw_args: Vec<String>) -> Result<CliArgs, PhaseGateCliError> {
    let mut target_file = Utf8PathBuf::from(DEFAULT_TARGET_FILE);
    let mut explicit_target = None::<String>;
    let mut args = raw_args.into_iter();

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--target-file" => {
                let Some(raw_path) = args.next() else {
                    return Err(PhaseGateCliError::MissingArgumentValue(
                        flag.into_boxed_str(),
                    ));
                };
                target_file = Utf8PathBuf::from(raw_path);
            }
            "--target" => {
                let Some(raw_target) = args.next() else {
                    return Err(PhaseGateCliError::MissingArgumentValue(
                        flag.into_boxed_str(),
                    ));
                };
                explicit_target = Some(raw_target);
            }
            _ => return Err(PhaseGateCliError::InvalidArgument(flag.into_boxed_str())),
        }
    }

    Ok(CliArgs {
        target_file,
        explicit_target,
    })
}

fn resolve_target(cli_args: &CliArgs) -> Result<PhaseGateTarget, PhaseGateCliError> {
    let raw_target = if let Some(explicit_target) = &cli_args.explicit_target {
        explicit_target.clone()
    } else {
        load_target_from_file(&cli_args.target_file)?
    };

    parse_phase_gate_target(&raw_target)
        .ok_or_else(|| PhaseGateCliError::InvalidTargetValue(raw_target.into_boxed_str()))
}

fn load_target_from_file(target_file: &Utf8Path) -> Result<String, PhaseGateCliError> {
    let ambient_dir =
        fs_utf8::Dir::open_ambient_dir(".", ambient_authority()).map_err(|source| {
            PhaseGateCliError::Io {
                path: Utf8PathBuf::from("."),
                source,
            }
        })?;

    let raw = ambient_dir
        .read_to_string(target_file)
        .map_err(|source| PhaseGateCliError::Io {
            path: target_file.to_path_buf(),
            source,
        })?;

    Ok(raw.trim().to_owned())
}

fn list_available_tests() -> Result<Vec<String>, PhaseGateCliError> {
    let output = Command::new("cargo")
        .env("RUSTFLAGS", "-D warnings")
        .args(CARGO_TEST_BASE_ARGS)
        .args(["--", "--list"])
        .output()
        .map_err(|source| PhaseGateCliError::Io {
            path: Utf8PathBuf::from("cargo"),
            source,
        })?;

    if !output.status.success() {
        return Err(PhaseGateCliError::CargoListFailed);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let listed_tests = stdout
        .lines()
        .filter_map(|line| line.strip_suffix(": test"))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    Ok(listed_tests)
}

fn run_suite_filter(test_filter: &str) -> io::Result<()> {
    let status = Command::new("cargo")
        .env("RUSTFLAGS", "-D warnings")
        .args(CARGO_TEST_BASE_ARGS)
        .arg(test_filter)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other("suite command failed"))
    }
}

fn emit_report(report: &PhaseGateReport) {
    match report.status {
        PhaseGateStatus::Passed => emit_success_report(report),
        PhaseGateStatus::MissingSuites | PhaseGateStatus::FailingSuites => {
            emit_failure_report(report);
        }
    }
}

fn emit_success_report(report: &PhaseGateReport) {
    let mut stdout = io::stdout().lock();
    discard_write_result(writeln!(
        stdout,
        "phase-gate target `{}` passed ({} suite(s) checked)",
        report.target.as_str(),
        report.required_suite_ids.len()
    ));
}

fn emit_suite_list(stderr: &mut io::StderrLock<'_>, header: &str, suite_ids: &[SuiteId]) {
    if suite_ids.is_empty() {
        return;
    }

    discard_write_result(writeln!(stderr, "{header}"));
    for suite_id in suite_ids {
        write_suite_line(stderr, suite_id);
    }
}

fn emit_failure_report(report: &PhaseGateReport) {
    let mut stderr = io::stderr().lock();
    discard_write_result(writeln!(
        stderr,
        "phase-gate target `{}` failed with status `{:?}`",
        report.target.as_str(),
        report.status
    ));

    emit_suite_list(
        &mut stderr,
        "missing mandated suites:",
        &report.missing_suite_ids,
    );
    emit_suite_list(
        &mut stderr,
        "failing mandated suites:",
        &report.failing_suite_ids,
    );

    discard_write_result(writeln!(stderr, "release-blocking when failures affect:"));
    for cause in RELEASE_BLOCKING_CAUSES {
        discard_write_result(writeln!(stderr, "- {cause}"));
    }

    discard_write_result(writeln!(stderr, "required escalation actions:"));
    for (index, step) in ESCALATION_STEPS.iter().enumerate() {
        discard_write_result(writeln!(stderr, "{}. {step}", index + 1));
    }
}

fn write_suite_line(stderr: &mut dyn Write, suite_id: &str) {
    let Some(suite) = suite_by_id(suite_id) else {
        discard_write_result(writeln!(stderr, "- {suite_id}"));
        return;
    };

    render_suite(stderr, suite);
}

fn render_suite(stderr: &mut dyn Write, suite: &VerificationSuite) {
    discard_write_result(writeln!(
        stderr,
        "- {} (subsystem: {}, filter: `{}`)",
        suite.id, suite.subsystem, suite.test_filter
    ));
}

fn discard_write_result(write_result: io::Result<()>) {
    drop(write_result);
}
