//! Probe test for full-monty Track A overhead checks.

use std::time::Duration;

use backon::{BlockingRetryable, ExponentialBuilder};
use test_utils::full_monty_probe_helpers;

fn run_track_a_overhead_probe_once() -> full_monty_probe_helpers::CargoProbeOutput {
    full_monty_probe_helpers::run_cargo_probe(
        &full_monty_probe_helpers::build_full_monty_test_command(
            "track_a_benchmarks",
            &["--", "--nocapture", "track_a_overhead_within_budget"],
        ),
        "track-a overhead probe should execute",
    )
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
