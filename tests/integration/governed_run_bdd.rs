//! Behavioural tests for governed execution via `zamburak-monty`.

use std::sync::{Arc, Mutex};

use monty::{
    MontyObject, MontyRun, NoLimitTracker, PrintWriter, RuntimeObserverEvent, RuntimeObserverHandle,
};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use zamburak_monty::{
    AllowAllMediator, DenyAllMediator, ExternalCallMediator, GovernedRunProgress, GovernedRunner,
};

/// Mutable world accumulating state across BDD steps.
#[derive(Default)]
struct GovernedRunWorld {
    source: String,
    input_names: Vec<String>,
    input_values: Vec<MontyObject>,
    mediator: Option<Arc<Mutex<dyn ExternalCallMediator>>>,
    result: Option<Result<GovernedRunProgress<NoLimitTracker>, zamburak_monty::GovernedRunError>>,
}

/// Snapshot of observer event counts captured via a counting observer.
#[derive(Clone, Debug, Default)]
struct EventSnapshot {
    value_created: usize,
    op_result: usize,
    control_condition: usize,
}

#[fixture]
fn world() -> GovernedRunWorld {
    GovernedRunWorld::default()
}

// ── Given steps ──────────────────────────────────────────────────────

#[given("a simple arithmetic Monty program")]
fn given_arithmetic_program(world: &mut GovernedRunWorld) {
    world.source = "x = 1 + 2\nx".to_owned();
}

#[given("a Monty program that calls an external function {name}")]
fn given_external_call_program(world: &mut GovernedRunWorld, name: String) {
    let name = name.trim_matches('"');
    world.source = format!("{name}()");
}

#[given("a Monty program with conditional branching")]
fn given_branching_program(world: &mut GovernedRunWorld) {
    world.source = concat!(
        "x = 10\n",
        "if x > 5:\n",
        "    result = \"big\"\n",
        "else:\n",
        "    result = \"small\"\n",
        "result",
    )
    .to_owned();
}

#[given("a Monty program with two numeric inputs")]
fn given_program_with_inputs(world: &mut GovernedRunWorld) {
    world.source = "a + b".to_owned();
    world.input_names = vec!["a".to_owned(), "b".to_owned()];
}

#[given("an AllowAll mediator")]
fn given_allow_all(world: &mut GovernedRunWorld) {
    world.mediator = Some(Arc::new(Mutex::new(AllowAllMediator)));
}

#[given("a DenyAll mediator")]
fn given_deny_all(world: &mut GovernedRunWorld) {
    world.mediator = Some(Arc::new(Mutex::new(DenyAllMediator)));
}

// ── When steps ───────────────────────────────────────────────────────

#[when("the governed runner executes the program")]
fn when_run(world: &mut GovernedRunWorld) {
    let mediator = world
        .mediator
        .clone()
        .unwrap_or_else(|| Arc::new(Mutex::new(AllowAllMediator)));
    let monty_run = MontyRun::new(world.source.clone(), "test.py", world.input_names.clone())
        .expect("parse should succeed");
    let runner = GovernedRunner::new(monty_run, mediator);
    world.result = Some(runner.run_no_limits(world.input_values.clone()));
}

#[when("the governed runner executes the program with integer inputs {a:i64} and {b:i64}")]
fn when_run_with_inputs(world: &mut GovernedRunWorld, a: i64, b: i64) {
    world.input_values = vec![MontyObject::Int(a), MontyObject::Int(b)];
    when_run(world);
}

// ── Then steps ───────────────────────────────────────────────────────

#[then("the result is Complete with integer value {expected:i64}")]
fn then_complete_int(world: &GovernedRunWorld, expected: i64) {
    let result = world.result.as_ref().expect("run step must execute first");
    match result {
        Ok(GovernedRunProgress::Complete(value)) => {
            assert_eq!(
                *value,
                MontyObject::Int(expected),
                "expected integer {expected}"
            );
        }
        other => panic!("expected Complete({expected}), got {other:?}"),
    }
}

#[then("the result is Complete with string value {expected}")]
fn then_complete_string(world: &GovernedRunWorld, expected: String) {
    let expected_str = expected.trim_matches('"');
    let result = world.result.as_ref().expect("run step must execute first");
    match result {
        Ok(GovernedRunProgress::Complete(value)) => {
            assert_eq!(
                *value,
                MontyObject::String(expected_str.to_owned()),
                "expected string \"{expected_str}\""
            );
        }
        other => panic!("expected Complete(\"{expected_str}\"), got {other:?}"),
    }
}

#[then("the result is Complete")]
fn then_complete(world: &GovernedRunWorld) {
    let result = world.result.as_ref().expect("run step must execute first");
    assert!(
        matches!(result, Ok(GovernedRunProgress::Complete(_))),
        "expected Complete, got {result:?}"
    );
}

#[then("the result is Denied for function {name}")]
fn then_denied(world: &GovernedRunWorld, name: String) {
    let name = name.trim_matches('"');
    let result = world.result.as_ref().expect("run step must execute first");
    match result {
        Ok(GovernedRunProgress::Denied { function_name, .. }) => {
            assert_eq!(function_name, name, "denied function name mismatch");
        }
        other => panic!("expected Denied for \"{name}\", got {other:?}"),
    }
}

#[then("the denial reason mentions {fragment}")]
fn then_denial_reason_contains(world: &GovernedRunWorld, fragment: String) {
    let fragment = fragment.trim_matches('"');
    let result = world.result.as_ref().expect("run step must execute first");
    match result {
        Ok(GovernedRunProgress::Denied { reason, .. }) => {
            assert!(
                reason.contains(fragment),
                "expected reason to contain \"{fragment}\", got: {reason}"
            );
        }
        other => panic!("expected Denied, got {other:?}"),
    }
}

#[then("the observer recorded value_created events")]
fn then_observer_value_created(world: &GovernedRunWorld) {
    let snapshot = count_events_for(&world.source);
    assert!(
        snapshot.value_created > 0,
        "expected at least one ValueCreated event, got 0"
    );
}

#[then("the observer recorded op_result events")]
fn then_observer_op_result(world: &GovernedRunWorld) {
    let snapshot = count_events_for(&world.source);
    assert!(
        snapshot.op_result > 0,
        "expected at least one OpResult event, got 0"
    );
}

#[then("the observer recorded control_condition events")]
fn then_observer_control_condition(world: &GovernedRunWorld) {
    let snapshot = count_events_for(&world.source);
    assert!(
        snapshot.control_condition > 0,
        "expected at least one ControlCondition event, got 0"
    );
}

// ── Observer event counting helper ───────────────────────────────────

/// Re-runs the program with a shared-state observer to capture event counts.
///
/// The governed runner owns its observer internally, so event counts are not
/// directly accessible from the consumer API. This helper uses a
/// `SharedCountingObserver` that writes counts into a shared `EventSnapshot`
/// so we can inspect them after execution completes.
fn count_events_for(source: &str) -> EventSnapshot {
    let snapshot = Arc::new(Mutex::new(EventSnapshot::default()));
    let observer = SharedCountingObserver {
        snapshot: Arc::clone(&snapshot),
    };
    let handle = RuntimeObserverHandle::new(observer);

    let monty_run =
        MontyRun::new(source.to_owned(), "test.py", vec![]).expect("parse should succeed");
    let _progress = monty_run
        .start_with_observer(vec![], NoLimitTracker, PrintWriter::Stdout, handle)
        .expect("execution should not raise");

    let guard = snapshot
        .lock()
        .expect("snapshot lock should not be poisoned");
    guard.clone()
}

/// Observer that writes event counts into a shared `EventSnapshot`.
struct SharedCountingObserver {
    snapshot: Arc<Mutex<EventSnapshot>>,
}

impl monty::RuntimeObserver for SharedCountingObserver {
    fn on_event(&mut self, event: RuntimeObserverEvent<'_>) {
        let mut snap = self
            .snapshot
            .lock()
            .expect("snapshot lock should not be poisoned");
        match event {
            RuntimeObserverEvent::ValueCreated(_) => snap.value_created += 1,
            RuntimeObserverEvent::OpResult(_) => snap.op_result += 1,
            RuntimeObserverEvent::ControlCondition(_) => {
                snap.control_condition += 1;
            }
            _ => {}
        }
    }
}

// ── Scenario bindings ────────────────────────────────────────────────

#[scenario(
    path = "tests/integration/features/governed_run.feature",
    name = "Simple program completes without external calls"
)]
fn simple_program_completes(world: GovernedRunWorld) {
    assert!(world.result.is_some());
}

#[scenario(
    path = "tests/integration/features/governed_run.feature",
    name = "External function call is denied by DenyAll mediator"
)]
fn external_call_denied(world: GovernedRunWorld) {
    assert!(world.result.is_some());
}

#[scenario(
    path = "tests/integration/features/governed_run.feature",
    name = "Conditional execution completes under governance"
)]
fn conditional_execution_completes(world: GovernedRunWorld) {
    assert!(world.result.is_some());
}

#[scenario(
    path = "tests/integration/features/governed_run.feature",
    name = "Program with inputs completes correctly"
)]
fn program_with_inputs(world: GovernedRunWorld) {
    assert!(world.result.is_some());
}

#[scenario(
    path = "tests/integration/features/governed_run.feature",
    name = "Observer receives Track A events during governed execution"
)]
fn observer_receives_track_a_events(world: GovernedRunWorld) {
    assert!(world.result.is_some());
}
