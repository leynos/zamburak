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
    "x = 1 + 2\nx".clone_into(&mut world.source);
}

#[given("a Monty program that calls an external function {name}")]
fn given_external_call_program(world: &mut GovernedRunWorld, name: String) {
    let function_name = name.trim_matches('"');
    world.source = format!("{function_name}()");
}

#[given("a Monty program with conditional branching")]
fn given_branching_program(world: &mut GovernedRunWorld) {
    concat!(
        "x = 10\n",
        "if x > 5:\n",
        "    result = \"big\"\n",
        "else:\n",
        "    result = \"small\"\n",
        "result",
    )
    .clone_into(&mut world.source);
}

#[given("a Monty program with two numeric inputs")]
fn given_program_with_inputs(world: &mut GovernedRunWorld) {
    "a + b".clone_into(&mut world.source);
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
    let monty_run = build_monty_run(&world.source, world.input_names.clone());
    let runner = GovernedRunner::new(monty_run, mediator);
    world.result = Some(runner.run_no_limits(world.input_values.clone()));
}

#[when("the governed runner executes the program with integer inputs {a:i64} and {b:i64}")]
fn when_run_with_inputs(world: &mut GovernedRunWorld, a: i64, b: i64) {
    world.input_values = vec![MontyObject::Int(a), MontyObject::Int(b)];
    when_run(world);
}

#[when("the host resumes the pending external call with integer result {value:i64}")]
fn when_resume_pending_call(world: &mut GovernedRunWorld, value: i64) {
    let suspended = take_pending_call(world);
    world.result = Some(suspended.resume(MontyObject::Int(value), PrintWriter::Stdout));
}

// ── Then steps ───────────────────────────────────────────────────────

/// Helper: assert the run completed with a specific [`MontyObject`] value.
fn assert_complete_with_value(world: &GovernedRunWorld, expected: MontyObject) {
    let result = require_result(world);
    match result {
        Ok(GovernedRunProgress::Complete(value)) => {
            assert_eq!(*value, expected, "expected Complete({expected:?})");
        }
        other => panic!("expected Complete({expected:?}), got {other:?}"),
    }
}

/// Helper: assert that the run produced a progress variant exposing a
/// `function_name` equal to `expected_fn_name`.
///
/// `extract` returns `Some(&str)` when the progress is the expected variant,
/// and `None` otherwise. `variant_label` appears only in panic messages.
fn assert_progress_function_name<'a, F>(
    world: &'a GovernedRunWorld,
    expected_fn_name: &str,
    variant_label: &str,
    extract: F,
) where
    F: FnOnce(&'a GovernedRunProgress<NoLimitTracker>) -> Option<&'a str>,
{
    let result = require_result(world);
    let progress = match result {
        Ok(p) => p,
        other => panic!("expected {variant_label} for \"{expected_fn_name}\", got {other:?}"),
    };
    match extract(progress) {
        Some(fn_name) => assert_eq!(
            fn_name, expected_fn_name,
            "{variant_label} function name mismatch"
        ),
        None => panic!("expected {variant_label} for \"{expected_fn_name}\", got {progress:?}"),
    }
}

#[then("the result is Complete with integer value {expected:i64}")]
fn then_complete_int(world: &GovernedRunWorld, expected: i64) {
    assert_complete_with_value(world, MontyObject::Int(expected));
}

#[then("the result is Complete with string value {expected}")]
fn then_complete_string(world: &GovernedRunWorld, expected: String) {
    let expected_str = expected.trim_matches('"').to_owned();
    assert_complete_with_value(world, MontyObject::String(expected_str));
}

#[then("the result is Complete")]
fn then_complete(world: &GovernedRunWorld) {
    let result = require_result(world);
    assert!(
        matches!(result, Ok(GovernedRunProgress::Complete(_))),
        "expected Complete, got {result:?}"
    );
}

#[then("the result is Denied for function {name}")]
fn then_denied(world: &GovernedRunWorld, name: String) {
    let expected = name.trim_matches('"').to_owned();
    assert_progress_function_name(world, &expected, "Denied", |p| {
        if let GovernedRunProgress::Denied { function_name, .. } = p {
            Some(function_name.as_str())
        } else {
            None
        }
    });
}

#[then("the result is ExternalCallPending for function {name}")]
fn then_external_call_pending(world: &GovernedRunWorld, name: String) {
    let expected = name.trim_matches('"').to_owned();
    assert_progress_function_name(world, &expected, "ExternalCallPending", |p| {
        if let GovernedRunProgress::ExternalCallPending { context, .. } = p {
            Some(context.function_name.as_str())
        } else {
            None
        }
    });
}

#[then("the denial reason mentions {fragment}")]
fn then_denial_reason_contains(world: &GovernedRunWorld, fragment: String) {
    let expected_fragment = fragment.trim_matches('"');
    let result = require_result(world);
    match result {
        Ok(GovernedRunProgress::Denied { reason, .. }) => {
            assert!(
                reason.contains(expected_fragment),
                "expected reason to contain \"{expected_fragment}\", got: {reason}"
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

    let monty_run = build_monty_run(source, vec![]);
    match monty_run.start_with_observer(vec![], NoLimitTracker, PrintWriter::Stdout, handle) {
        Ok(_) => {}
        Err(error) => panic!("execution should not raise: {error}"),
    }

    let guard = lock_snapshot(&snapshot);
    guard.clone()
}

/// Observer that writes event counts into a shared `EventSnapshot`.
struct SharedCountingObserver {
    snapshot: Arc<Mutex<EventSnapshot>>,
}

impl monty::RuntimeObserver for SharedCountingObserver {
    fn on_event(&mut self, event: RuntimeObserverEvent<'_>) {
        let mut snap = lock_snapshot(&self.snapshot);
        match event {
            RuntimeObserverEvent::ValueCreated(_) => snap.value_created += 1,
            RuntimeObserverEvent::OpResult(_) => snap.op_result += 1,
            RuntimeObserverEvent::ControlCondition(_) => {
                snap.control_condition += 1;
            }
            RuntimeObserverEvent::ExternalCallRequested(_)
            | RuntimeObserverEvent::ExternalCallReturned(_) => {}
        }
    }
}

fn build_monty_run(source: &str, input_names: Vec<String>) -> MontyRun {
    MontyRun::new(source.to_owned(), "test.py", input_names)
        .expect("failed to create MontyRun for test source")
}

fn require_result(
    world: &GovernedRunWorld,
) -> &Result<GovernedRunProgress<NoLimitTracker>, zamburak_monty::GovernedRunError> {
    world
        .result
        .as_ref()
        .unwrap_or_else(|| panic!("run step must execute first"))
}

fn take_pending_call(
    world: &mut GovernedRunWorld,
) -> zamburak_monty::SuspendedCall<NoLimitTracker> {
    let result = world
        .result
        .take()
        .unwrap_or_else(|| panic!("run step must execute first"));
    match result {
        Ok(GovernedRunProgress::ExternalCallPending { suspended, .. }) => suspended,
        other => panic!("expected ExternalCallPending, got {other:?}"),
    }
}

fn lock_snapshot(snapshot: &Arc<Mutex<EventSnapshot>>) -> std::sync::MutexGuard<'_, EventSnapshot> {
    match snapshot.lock() {
        Ok(guard) => guard,
        Err(error) => panic!("snapshot lock should not be poisoned: {error}"),
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
    name = "External function call allowed by mediator yields pending host resume"
)]
fn external_call_pending_then_resumed(world: GovernedRunWorld) {
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
