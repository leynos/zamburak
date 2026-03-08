//! Probe test for full-monty Track A overhead checks.

use test_utils::full_monty_observer_probe_helpers;

#[test]
fn full_monty_track_a_overhead_probe() {
    let output = full_monty_observer_probe_helpers::run_cargo_probe(
        &[
            "test".to_owned(),
            "--manifest-path".to_owned(),
            "third_party/full-monty/Cargo.toml".to_owned(),
            "-p".to_owned(),
            "monty".to_owned(),
            "--test".to_owned(),
            "track_a_invariants".to_owned(),
            "--".to_owned(),
            "--nocapture".to_owned(),
            "track_a_overhead".to_owned(),
        ],
        "track-a overhead probe should execute",
    );

    assert_eq!(output.status_code, Some(0), "stderr:\n{}", output.stderr);

    let combined_output = format!("{}\n{}", output.stdout, output.stderr);
    assert!(
        combined_output.contains("track_a_overhead disabled")
            && combined_output.contains("track_a_overhead noop"),
        "expected overhead markers in probe output"
    );
}
