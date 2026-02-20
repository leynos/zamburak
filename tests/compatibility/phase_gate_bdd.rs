//! Behavioural tests for phase-gate CI enforcement contracts.

use std::collections::BTreeSet;

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use zamburak::phase_gate_contract::{
    self, ESCALATION_STEPS, PhaseGateReport, PhaseGateStatus, RELEASE_BLOCKING_CAUSES,
    evaluate_phase_gate, parse_phase_gate_target, suite_by_id,
};

#[derive(Default)]
struct PhaseGateWorld {
    target_input: String,
    available_tests: Vec<String>,
    failing_suite_ids: BTreeSet<&'static str>,
    parse_result: Option<phase_gate_contract::PhaseGateTarget>,
    parse_attempted: bool,
    report: Option<PhaseGateReport>,
}

#[fixture]
fn world() -> PhaseGateWorld {
    PhaseGateWorld::default()
}

#[given("a phase-gate target input {target}")]
fn phase_gate_target_input(world: &mut PhaseGateWorld, target: String) {
    target.trim_matches('"').clone_into(&mut world.target_input);
}

#[given("an empty verification catalogue")]
fn empty_verification_catalogue(world: &mut PhaseGateWorld) {
    world.available_tests.clear();
}

#[given("the verification catalogue has schema and authority suites only")]
fn schema_and_authority_only_catalogue(world: &mut PhaseGateWorld) {
    world.available_tests = vec![
        "policy_schema_bdd::load_canonical_schema_policy".to_owned(),
        "authority_lifecycle_bdd::mint_host_trusted".to_owned(),
    ];
}

#[given("the verification catalogue has all phase-one suites")]
fn full_phase_one_catalogue(world: &mut PhaseGateWorld) {
    world.available_tests = vec![
        "policy_schema_bdd::load_canonical_schema_policy".to_owned(),
        "authority_lifecycle_bdd::mint_host_trusted".to_owned(),
        "llm_sink_enforcement::pre_dispatch".to_owned(),
        "localization_contract::explicit_localizer".to_owned(),
    ];
}

#[given("the suite {suite_id} is marked failing")]
fn marked_failing_suite(world: &mut PhaseGateWorld, suite_id: String) {
    let normalized_suite_id = suite_id.trim_matches('"');
    let Some(suite) = suite_by_id(normalized_suite_id) else {
        panic!("unsupported suite id in scenario: {suite_id}");
    };

    world.failing_suite_ids.insert(suite.id);
}

#[when("the phase gate is evaluated")]
fn evaluate_gate(world: &mut PhaseGateWorld) {
    let Some(target) = parse_phase_gate_target(&world.target_input) else {
        panic!("target must parse before gate evaluation");
    };

    world.report = Some(evaluate_phase_gate(
        target,
        &world.available_tests,
        &world.failing_suite_ids,
    ));
}

#[when("the phase-gate target is parsed")]
fn parse_target(world: &mut PhaseGateWorld) {
    world.parse_attempted = true;
    world.parse_result = parse_phase_gate_target(&world.target_input);
}

#[then("the phase gate status is {status}")]
fn assert_gate_status(world: &PhaseGateWorld, status: String) {
    let Some(report) = world.report.as_ref() else {
        panic!("phase gate must be evaluated before asserting status");
    };

    let expected_status = match status.trim_matches('"') {
        "Passed" => PhaseGateStatus::Passed,
        "MissingSuites" => PhaseGateStatus::MissingSuites,
        "FailingSuites" => PhaseGateStatus::FailingSuites,
        _ => panic!("unsupported status in scenario: {status}"),
    };

    assert_eq!(report.status, expected_status);
}

#[then("missing suites include {suite_ids}")]
fn assert_missing_suites(world: &PhaseGateWorld, suite_ids: String) {
    let Some(report) = world.report.as_ref() else {
        panic!("phase gate must be evaluated before asserting missing suites");
    };

    let expected = suite_ids.trim_matches('"').split(',').collect::<Vec<_>>();
    assert_eq!(report.missing_suite_ids, expected);
}

#[then("failing suites include {suite_ids}")]
fn assert_failing_suites(world: &PhaseGateWorld, suite_ids: String) {
    let Some(report) = world.report.as_ref() else {
        panic!("phase gate must be evaluated before asserting failing suites");
    };

    let expected = suite_ids.trim_matches('"').split(',').collect::<Vec<_>>();
    assert_eq!(report.failing_suite_ids, expected);
}

#[then("the phase-gate target parse result is invalid")]
fn assert_invalid_target_parse(world: &PhaseGateWorld) {
    assert!(
        world.parse_attempted,
        "parse step must run before parse assertions"
    );
    assert!(world.parse_result.is_none());
}

#[scenario(
    path = "tests/compatibility/features/phase_gate.feature",
    name = "Phase zero target passes with no mandated suites"
)]
fn phase_zero_target_passes(world: PhaseGateWorld) {
    assert!(world.report.is_some());
}

#[scenario(
    path = "tests/compatibility/features/phase_gate.feature",
    name = "Phase one target blocks when mandated suites are missing"
)]
fn phase_one_missing_suites_blocks(world: PhaseGateWorld) {
    assert!(world.report.is_some());
}

#[scenario(
    path = "tests/compatibility/features/phase_gate.feature",
    name = "Phase one target blocks when a mandated suite fails"
)]
fn phase_one_failing_suite_blocks(world: PhaseGateWorld) {
    assert!(world.report.is_some());
}

#[scenario(
    path = "tests/compatibility/features/phase_gate.feature",
    name = "Unsupported phase target input is rejected"
)]
fn unsupported_phase_target_rejected(world: PhaseGateWorld) {
    assert!(world.parse_attempted);
}

#[test]
fn policy_constants_and_suite_lookup_are_exposed_for_ci_output() {
    assert_eq!(RELEASE_BLOCKING_CAUSES.len(), 4);
    assert_eq!(ESCALATION_STEPS.len(), 3);

    let suite = suite_by_id("authority-lifecycle").expect("authority lifecycle suite should exist");
    assert_eq!(suite.id, "authority-lifecycle");

    let target = parse_phase_gate_target("phase1").expect("phase1 target should parse");
    assert_eq!(target.as_str(), "phase1");
}
