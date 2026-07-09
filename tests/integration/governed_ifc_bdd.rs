//! Behavioural IFC tests for governed execution.

use std::sync::{Arc, Mutex};

use anyhow::{Context as _, ensure};
use monty::{MontyObject, MontyRun, NoLimitTracker, PrintWriter};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_utils::governed_run_test_helpers::RecordingMediator;
use zamburak_core::propagation::PropagationMode;
use zamburak_core::{DependencySummary, IntegrityLabel};
use zamburak_monty::{
    CallContext, GovernedIfcConfig, GovernedRunProgress, GovernedRunner, IfcValueSeedConfig,
};

#[derive(Default)]
struct GovernedIfcWorld {
    source: String,
    ifc_config: GovernedIfcConfig,
    contexts: Arc<Mutex<Vec<CallContext>>>,
    result: Option<GovernedRunProgress<NoLimitTracker>>,
}

#[fixture]
fn world() -> GovernedIfcWorld {
    GovernedIfcWorld {
        ifc_config: GovernedIfcConfig::default(),
        contexts: Arc::new(Mutex::new(Vec::new())),
        ..GovernedIfcWorld::default()
    }
}

fn take_pending_call(
    world: &mut GovernedIfcWorld,
) -> zamburak_monty::SuspendedCall<NoLimitTracker> {
    let Some(result) = world.result.take() else {
        panic!("expected stored run result");
    };
    match result {
        GovernedRunProgress::ExternalCallPending { suspended, .. } => suspended,
        other => panic!("expected ExternalCallPending, got {other:?}"),
    }
}

/// Fetch the captured call context for `function_name`.
///
/// Fixture-style query: failure to find the context is reported as an error
/// so the calling step can propagate it as the scenario verdict.
fn captured_context(
    world: &GovernedIfcWorld,
    function_name: &str,
) -> Result<CallContext, anyhow::Error> {
    let contexts = world
        .contexts
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    contexts
        .iter()
        .find(|context| context.function_name == function_name)
        .cloned()
        .context("expected named context to be captured")
}

#[given("a program whose constant effect call is control-dependent on an external result")]
fn given_control_program(world: &mut GovernedIfcWorld) {
    concat!(
        "condition = gate()\n",
        "if condition:\n",
        "    effect(\"constant\")\n",
        "else:\n",
        "    effect(\"constant\")\n",
    )
    .clone_into(&mut world.source);
}

#[given("a program whose second effect consumes a resumed external return value")]
fn given_return_flow_program(world: &mut GovernedIfcWorld) {
    "value = source(\"seed\")\nsink(value)".clone_into(&mut world.source);
}

#[given("strict IFC propagation is configured")]
fn given_strict_ifc(world: &mut GovernedIfcWorld) {
    world.ifc_config = GovernedIfcConfig::strict_with_boundary_seeds();
}

#[given("normal IFC propagation is configured")]
fn given_normal_ifc(world: &mut GovernedIfcWorld) {
    world.ifc_config = GovernedIfcConfig {
        propagation_mode: PropagationMode::Normal,
        value_seeds: IfcValueSeedConfig::boundary_defaults(),
        ..GovernedIfcConfig::default()
    };
}

#[when("governed execution starts and the first external call returns string {value}")]
fn when_start_and_resume_first_call(world: &mut GovernedIfcWorld, value: String) {
    let mediator = Arc::new(Mutex::new(RecordingMediator {
        contexts: Arc::clone(&world.contexts),
    }));
    let monty_run = match MontyRun::new(world.source.clone(), "test.py", vec![]) {
        Ok(run) => run,
        Err(error) => panic!("MontyRun should be created: {error}"),
    };
    let runner = GovernedRunner::new(monty_run, mediator).with_ifc_config(world.ifc_config.clone());

    let first_result = match runner.run_no_limits(vec![]) {
        Ok(result) => result,
        Err(error) => panic!("first governed step should succeed: {error}"),
    };
    world.result = Some(first_result);

    let pending = take_pending_call(world);
    let resumed = match pending.resume(
        MontyObject::String(value.trim_matches('"').to_owned()),
        PrintWriter::Stdout,
    ) {
        Ok(result) => result,
        Err(error) => panic!("resume should succeed: {error}"),
    };
    world.result = Some(resumed);
}

/// Wraps an `IntegrityLabel` so `FromStr` can parse test-input strings such as
/// `"Trusted"`, `"Untrusted"`, and `"Verified"` into typed integrity values.
struct IntegrityArg(IntegrityLabel);

impl std::str::FromStr for IntegrityArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim_matches('"') {
            "Trusted" => Ok(Self(IntegrityLabel::Trusted)),
            "Untrusted" => Ok(Self(IntegrityLabel::Untrusted)),
            "Verified" => Ok(Self(IntegrityLabel::Verified)),
            other => Err(format!("unsupported integrity label: {other}")),
        }
    }
}

/// Fetch the integrity label selected by `extract` from the captured call
/// context for `function_name`, then check it equals the expected label.
fn ensure_context_integrity(
    world: &GovernedIfcWorld,
    function_name: &str,
    expected: (IntegrityLabel, &str),
    extract: impl FnOnce(&CallContext) -> Result<IntegrityLabel, anyhow::Error>,
) -> Result<(), anyhow::Error> {
    let context = captured_context(world, function_name.trim_matches('"'))?;
    let actual = extract(&context)?;
    let (expected_label, what) = expected;
    ensure!(
        actual == expected_label,
        "{what} integrity mismatch: expected {expected_label:?}, got {actual:?}"
    );
    Ok(())
}

/// Fetch the origin count selected by `extract` from the captured call
/// context for `function_name`, then check it meets the minimum.
fn ensure_context_origin_count(
    world: &GovernedIfcWorld,
    function_name: &str,
    expected: (u32, &str),
    extract: impl FnOnce(&CallContext) -> Result<u32, anyhow::Error>,
) -> Result<(), anyhow::Error> {
    let context = captured_context(world, function_name.trim_matches('"'))?;
    let actual = extract(&context)?;
    let (min_count, what) = expected;
    ensure!(
        actual >= min_count,
        "expected {what} origin_count >= {min_count}, got {actual}"
    );
    Ok(())
}

/// Fetch the first argument summary or report its absence as an error.
fn first_arg_summary(context: &CallContext) -> Result<&DependencySummary, anyhow::Error> {
    context
        .ifc
        .arg_summaries
        .first()
        .context("expected at least one argument summary")
}

#[then("the captured call context for {function_name} has aggregate integrity {integrity}")]
fn then_aggregate_integrity(
    world: &GovernedIfcWorld,
    function_name: String,
    integrity: IntegrityArg,
) -> Result<(), anyhow::Error> {
    ensure_context_integrity(world, &function_name, (integrity.0, "aggregate"), |c| {
        Ok(c.ifc.aggregate_summary.integrity_join)
    })
}

#[then("the captured call context for {function_name} has PC integrity {integrity}")]
fn then_pc_integrity(
    world: &GovernedIfcWorld,
    function_name: String,
    integrity: IntegrityArg,
) -> Result<(), anyhow::Error> {
    ensure_context_integrity(world, &function_name, (integrity.0, "PC"), |c| {
        Ok(c.ifc.control_context.pc_integrity())
    })
}

#[then("the captured call context for {function_name} has first argument integrity {integrity}")]
fn then_arg_integrity(
    world: &GovernedIfcWorld,
    function_name: String,
    integrity: IntegrityArg,
) -> Result<(), anyhow::Error> {
    ensure_context_integrity(
        world,
        &function_name,
        (integrity.0, "first argument"),
        |c| Ok(first_arg_summary(c)?.integrity_join),
    )
}

#[then(
    "the captured call context for {function_name} has aggregate origin-count at least {min_count}"
)]
fn then_aggregate_origin_count(
    world: &GovernedIfcWorld,
    function_name: String,
    min_count: u32,
) -> Result<(), anyhow::Error> {
    ensure_context_origin_count(world, &function_name, (min_count, "aggregate"), |c| {
        Ok(c.ifc.aggregate_summary.origin_count)
    })
}

#[then(
    "the captured call context for {function_name} has first argument origin-count at least {min_count}"
)]
fn then_arg_origin_count(
    world: &GovernedIfcWorld,
    function_name: String,
    min_count: u32,
) -> Result<(), anyhow::Error> {
    ensure_context_origin_count(world, &function_name, (min_count, "first-argument"), |c| {
        Ok(first_arg_summary(c)?.origin_count)
    })
}

#[scenario(
    path = "tests/integration/features/governed_ifc.feature",
    name = "Strict mode carries control context into a constant effect call"
)]
fn strict_mode_control_context(world: GovernedIfcWorld) {
    drop(world);
}

#[scenario(
    path = "tests/integration/features/governed_ifc.feature",
    name = "Normal mode leaves the constant effect call data summary unchanged"
)]
fn normal_mode_control_context(world: GovernedIfcWorld) {
    drop(world);
}

#[scenario(
    path = "tests/integration/features/governed_ifc.feature",
    name = "Returned host values retain provenance for a following effect"
)]
fn returned_value_provenance(world: GovernedIfcWorld) {
    drop(world);
}
