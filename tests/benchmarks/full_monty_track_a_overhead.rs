//! Probe test for full-monty Track A overhead checks.

use std::{thread, time::Duration};

use test_utils::full_monty_probe_helpers;

/// Retries the noisy benchmark probe a small number of times before failing.
const TRACK_A_OVERHEAD_MAX_ATTEMPTS: usize = 3;
/// Sleeps briefly between failed attempts to reduce immediate CI contention.
const TRACK_A_OVERHEAD_RETRY_BACKOFF: Duration = Duration::from_millis(200);

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

#[test]
fn full_monty_track_a_overhead_probe() {
    let mut last_output = None;

    for attempt in 0..TRACK_A_OVERHEAD_MAX_ATTEMPTS {
        let output = run_track_a_overhead_probe_once();
        let combined_output = format!("{}\n{}", output.stdout, output.stderr);
        let overhead_lines =
            full_monty_probe_helpers::prefixed_output_lines(&combined_output, "track_a_overhead ");
        let markers_present = has_overhead_markers(&overhead_lines);

        if output.status_code == Some(0) && markers_present {
            return;
        }

        last_output = Some((output, combined_output));
        if attempt + 1 < TRACK_A_OVERHEAD_MAX_ATTEMPTS {
            thread::sleep(TRACK_A_OVERHEAD_RETRY_BACKOFF);
        }
    }

    let (output, combined_output) = last_output.expect("at least one benchmark attempt should run");
    assert_eq!(
        output.status_code,
        Some(0),
        "stderr:\n{}\nstdout:\n{}",
        output.stderr,
        output.stdout
    );
    let overhead_lines =
        full_monty_probe_helpers::prefixed_output_lines(&combined_output, "track_a_overhead ");
    assert!(
        has_overhead_markers(&overhead_lines),
        "expected Track A overhead markers in probe output:\n{combined_output}"
    );
}
