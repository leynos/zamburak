//! Probe test for full-monty Track A overhead checks.

use std::time::Duration;

use backon::{BlockingRetryable, ExponentialBuilder};
use test_utils::full_monty_probe_helpers;

const TRACK_A_OVERHEAD_CASE_NAMES: [&str; 2] = [
    "track_a_overhead_within_budget::case_1",
    "track_a_overhead_within_budget::case_2",
];

fn run_track_a_overhead_probe_once() -> full_monty_probe_helpers::CargoProbeOutput {
    let mut combined_output = full_monty_probe_helpers::CargoProbeOutput {
        status_code: Some(0),
        stdout: String::new(),
        stderr: String::new(),
    };

    for case_name in TRACK_A_OVERHEAD_CASE_NAMES {
        let output = full_monty_probe_helpers::run_cargo_probe(
            &full_monty_probe_helpers::build_full_monty_test_command(
                "track_a_benchmarks",
                &["--", "--nocapture", "--exact", case_name],
            ),
            "track-a overhead probe should execute",
        );

        if output.status_code != Some(0) {
            combined_output.status_code = output.status_code;
        }

        if !combined_output.stdout.is_empty() {
            combined_output.stdout.push('\n');
        }
        combined_output.stdout.push_str(&output.stdout);

        if !combined_output.stderr.is_empty() {
            combined_output.stderr.push('\n');
        }
        combined_output.stderr.push_str(&output.stderr);
    }

    combined_output
}

fn has_overhead_markers(overhead_lines: &[&str]) -> bool {
    overhead_lines
        .iter()
        .any(|line| line.contains("DisabledHandle"))
        && overhead_lines
            .iter()
            .any(|line| line.contains("NoopObserver"))
}

struct TrackAOverheadProbeFailure {
    output: full_monty_probe_helpers::CargoProbeOutput,
    combined_output: String,
}

#[test]
fn full_monty_track_a_overhead_probe() {
    let retry_result = (|| {
        let output = run_track_a_overhead_probe_once();
        let combined_output = format!("{}\n{}", output.stdout, output.stderr);
        let overhead_lines =
            full_monty_probe_helpers::prefixed_output_lines(&combined_output, "track_a_overhead ");
        let markers_present = has_overhead_markers(&overhead_lines);

        if output.status_code == Some(0) && markers_present {
            Ok(combined_output)
        } else {
            Err(TrackAOverheadProbeFailure {
                output,
                combined_output,
            })
        }
    })
    .retry(
        ExponentialBuilder::default()
            .with_min_delay(Duration::from_millis(200))
            .with_max_times(3)
            .with_factor(2.0)
            .with_jitter(),
    )
    .call();

    if let Err(error) = retry_result {
        let overhead_lines = full_monty_probe_helpers::prefixed_output_lines(
            &error.combined_output,
            "track_a_overhead ",
        );
        assert_eq!(
            error.output.status_code,
            Some(0),
            "stderr:\n{}\nstdout:\n{}",
            error.output.stderr,
            error.output.stdout
        );
        assert!(
            has_overhead_markers(&overhead_lines),
            "expected Track A overhead markers in probe output:\n{}",
            error.combined_output
        );
    }
}
