//! Unit tests for the governed execution entrypoint.

use std::sync::{Arc, Mutex};

use monty::{MontyObject, MontyRun};
use rstest::rstest;

use crate::external_call::{AllowAllMediator, DenyAllMediator, ExternalCallMediator};
use crate::run::{GovernedRunProgress, GovernedRunner};

/// Helper: build a `GovernedRunner` from source code with the given mediator.
fn governed_runner(code: &str, mediator: Arc<Mutex<dyn ExternalCallMediator>>) -> GovernedRunner {
    let monty_run =
        MontyRun::new(code.to_owned(), "test.py", vec![]).expect("parse should succeed");
    GovernedRunner::new(monty_run, mediator)
}

/// Helper: wrap a mediator in the shared handle type.
fn shared_mediator(m: impl ExternalCallMediator + 'static) -> Arc<Mutex<dyn ExternalCallMediator>> {
    Arc::new(Mutex::new(m))
}

#[rstest]
#[case::simple_program("x = 1 + 2\nx", MontyObject::Int(3))]
#[case::string_operations(
    "\nx = \"hello\"\ny = \" world\"\nx + y\n",
    MontyObject::String("hello world".to_owned())
)]
#[case::boolean_and_none("True", MontyObject::Bool(true))]
#[case::conditional(
    "\nx = 10\nif x > 5:\n    result = \"big\"\nelse:\n    result = \"small\"\nresult\n",
    MontyObject::String("big".to_owned())
)]
fn complete_without_external_calls(#[case] code: &str, #[case] expected: MontyObject) {
    let runner = governed_runner(code, shared_mediator(AllowAllMediator));
    let result = runner.run_no_limits(vec![]);
    match result {
        Ok(GovernedRunProgress::Complete(value)) => {
            assert_eq!(value, expected);
        }
        other => panic!("expected Complete({expected:?}), got {other:?}"),
    }
}

#[rstest]
fn program_with_external_call_denied_by_deny_all_mediator() {
    // This program calls `foo()`, which is an external function.
    // With DenyAllMediator, the governed runner should deny it.
    let runner = governed_runner("foo()", shared_mediator(DenyAllMediator));
    let result = runner.run_no_limits(vec![]);
    match result {
        Ok(GovernedRunProgress::Denied {
            reason,
            function_name,
            ..
        }) => {
            assert!(reason.contains("DenyAllMediator"));
            assert_eq!(function_name, "foo");
        }
        other => panic!("expected Denied, got {other:?}"),
    }
}

#[rstest]
fn program_with_inputs_completes_correctly() {
    let monty_run = MontyRun::new(
        "a + b".to_owned(),
        "test.py",
        vec!["a".to_owned(), "b".to_owned()],
    )
    .expect("parse should succeed");
    let mediator = shared_mediator(AllowAllMediator);
    let runner = GovernedRunner::new(monty_run, mediator);
    let result = runner.run_no_limits(vec![MontyObject::Int(10), MontyObject::Int(32)]);
    match result {
        Ok(GovernedRunProgress::Complete(value)) => {
            assert_eq!(value, MontyObject::Int(42));
        }
        other => panic!("expected Complete(42), got {other:?}"),
    }
}
