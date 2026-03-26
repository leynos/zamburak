//! Behavioural IFC tests for governed execution.

use std::sync::{Arc, Mutex};

use monty::{MontyObject, MontyRun, NoLimitTracker, PrintWriter};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use zamburak_core::propagation::PropagationMode;
use zamburak_core::{AuthoritySet, DataLabels, IntegrityLabel, ValueLabels};
use zamburak_monty::{
    CallContext, ExternalCallMediator, GovernedIfcConfig, GovernedRunProgress, GovernedRunner,
    IfcValueSeedConfig, MediationDecision,
};

#[derive(Clone)]
struct RecordingMediator {
    contexts: Arc<Mutex<Vec<CallContext>>>,
}

impl ExternalCallMediator for RecordingMediator {
    fn mediate(&mut self, context: &CallContext) -> MediationDecision {
        self.contexts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(context.clone());
        MediationDecision::Allow
    }
}

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

fn boundary_seed_config() -> IfcValueSeedConfig {
    IfcValueSeedConfig {
        internal_values: ValueLabels {
            integrity: IntegrityLabel::Trusted,
            confidentiality: DataLabels::new(),
            authority: AuthoritySet::full(),
        },
        resumed_external_returns: ValueLabels {
            integrity: IntegrityLabel::Untrusted,
            confidentiality: DataLabels::new(),
            authority: AuthoritySet::full(),
        },
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

fn captured_context(world: &GovernedIfcWorld, function_name: &str) -> CallContext {
    let contexts = world
        .contexts
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    contexts
        .iter()
        .find(|context| context.function_name == function_name)
        .cloned()
        .unwrap_or_else(|| panic!("expected named context to be captured"))
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
    world.ifc_config = GovernedIfcConfig {
        propagation_mode: PropagationMode::Strict,
        value_seeds: boundary_seed_config(),
        ..GovernedIfcConfig::default()
    };
}

#[given("normal IFC propagation is configured")]
fn given_normal_ifc(world: &mut GovernedIfcWorld) {
    world.ifc_config = GovernedIfcConfig {
        propagation_mode: PropagationMode::Normal,
        value_seeds: boundary_seed_config(),
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

#[then("the captured call context for {function_name} has aggregate integrity {integrity}")]
fn then_aggregate_integrity(world: &GovernedIfcWorld, function_name: String, integrity: String) {
    let context = captured_context(world, function_name.trim_matches('"'));
    let expected = match integrity.trim_matches('"') {
        "Trusted" => IntegrityLabel::Trusted,
        "Untrusted" => IntegrityLabel::Untrusted,
        other => panic!("unsupported integrity label: {other}"),
    };
    assert_eq!(context.ifc.aggregate_summary.integrity_join, expected);
}

#[then("the captured call context for {function_name} has PC integrity {integrity}")]
fn then_pc_integrity(world: &GovernedIfcWorld, function_name: String, integrity: String) {
    let context = captured_context(world, function_name.trim_matches('"'));
    let expected = match integrity.trim_matches('"') {
        "Trusted" => IntegrityLabel::Trusted,
        "Untrusted" => IntegrityLabel::Untrusted,
        other => panic!("unsupported integrity label: {other}"),
    };
    assert_eq!(context.ifc.control_context.pc_integrity(), expected);
}

#[then("the captured call context for {function_name} has first argument integrity {integrity}")]
fn then_arg_integrity(world: &GovernedIfcWorld, function_name: String, integrity: String) {
    let context = captured_context(world, function_name.trim_matches('"'));
    let expected = match integrity.trim_matches('"') {
        "Trusted" => IntegrityLabel::Trusted,
        "Untrusted" => IntegrityLabel::Untrusted,
        other => panic!("unsupported integrity label: {other}"),
    };
    match context.ifc.arg_summaries.first() {
        Some(summary) => assert_eq!(summary.integrity_join, expected),
        None => panic!("expected at least one argument summary"),
    }
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
