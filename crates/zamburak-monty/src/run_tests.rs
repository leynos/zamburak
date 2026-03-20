//! Unit tests for the governed execution entrypoint.

use std::sync::{Arc, Mutex};

use monty::{MontyObject, MontyRun, NoLimitTracker, PrintWriter};
use rstest::rstest;

use crate::external_call::{AllowAllMediator, DenyAllMediator, ExternalCallMediator};
use crate::run::{GovernedRunError, GovernedRunProgress, GovernedRunner};

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
fn allowed_external_call_yields_pending_call_and_resumes_to_completion() {
    let runner = governed_runner("foo()", shared_mediator(AllowAllMediator));
    let result = runner.run_no_limits(vec![]);
    let suspended = match result {
        Ok(GovernedRunProgress::ExternalCallPending { context, suspended }) => {
            assert_eq!(context.function_name, "foo");
            suspended
        }
        other => panic!("expected ExternalCallPending, got {other:?}"),
    };

    let resumed = suspended.resume(MontyObject::Int(7), PrintWriter::Stdout);
    match resumed {
        Ok(GovernedRunProgress::Complete(value)) => {
            assert_eq!(value, MontyObject::Int(7));
        }
        other => panic!("expected Complete(7), got {other:?}"),
    }
}

#[rstest]
fn custom_print_writer_is_used_after_resuming_pending_external_call() {
    let runner = governed_runner(
        "print(\"before\")\nfoo()\nprint(\"after\")",
        shared_mediator(AllowAllMediator),
    );
    let mut output = String::new();
    let result = runner.run(vec![], NoLimitTracker, PrintWriter::Collect(&mut output));
    let suspended = match result {
        Ok(GovernedRunProgress::ExternalCallPending { suspended, .. }) => suspended,
        other => panic!("expected ExternalCallPending, got {other:?}"),
    };
    assert_eq!(output, "before\n");

    let resumed = suspended.resume(MontyObject::None, PrintWriter::Collect(&mut output));
    match resumed {
        Ok(GovernedRunProgress::Complete(value)) => {
            assert_eq!(value, MontyObject::None);
        }
        other => panic!("expected Complete(None), got {other:?}"),
    }
    assert_eq!(output, "before\nafter\n");
}

#[rstest]
fn program_with_interpreter_error_mapped_to_governed_run_error_interpreter() {
    let runner = governed_runner("1 / 0", shared_mediator(DenyAllMediator));
    let result = runner.run_no_limits(vec![]);
    match result {
        Err(GovernedRunError::Interpreter(_)) => {}
        other => panic!("expected Err(GovernedRunError::Interpreter(_)), got {other:?}"),
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
