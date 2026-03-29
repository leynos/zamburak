//! Unit tests for the governed execution entrypoint.

use std::sync::{Arc, Mutex};

use monty::{MontyObject, MontyRun, NoLimitTracker, PrintWriter};
use rstest::rstest;
use zamburak_core::IntegrityLabel;
use zamburak_core::propagation::PropagationMode;

use crate::external_call::{AllowAllMediator, DenyAllMediator, ExternalCallMediator};
use crate::observer::GovernedIfcConfig;
use crate::run::{GovernedRunError, GovernedRunProgress, GovernedRunner};
use crate::test_helpers::recording_mediator;

type SharedMediator = Arc<Mutex<dyn ExternalCallMediator>>;

/// Helper: build a `GovernedRunner` from source code with the given mediator.
fn governed_runner(code: &str, mediator: SharedMediator) -> GovernedRunner {
    let monty_run =
        MontyRun::new(code.to_owned(), "test.py", vec![]).expect("parse should succeed");
    GovernedRunner::new(monty_run, mediator)
}

/// Helper: wrap a mediator in the shared handle type.
fn shared_mediator(m: impl ExternalCallMediator + 'static) -> SharedMediator {
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

#[rstest]
fn strict_mode_control_context_taints_constant_effect_call() {
    let (mediator, contexts) = recording_mediator();
    let monty_run = MontyRun::new(
        "condition = gate()\nif condition:\n    effect(\"constant\")\nelse:\n    effect(\"constant\")"
            .to_owned(),
        "test.py",
        vec![],
    )
    .expect("parse should succeed");
    let runner = GovernedRunner::new(monty_run, mediator)
        .with_ifc_config(GovernedIfcConfig::strict_with_boundary_seeds());

    let first_progress = runner.run_no_limits(vec![]);
    let first_suspended = match first_progress {
        Ok(GovernedRunProgress::ExternalCallPending { suspended, .. }) => suspended,
        other => panic!("expected first ExternalCallPending, got {other:?}"),
    };

    let second_progress =
        first_suspended.resume(MontyObject::String("yes".to_owned()), PrintWriter::Stdout);
    match second_progress {
        Ok(GovernedRunProgress::ExternalCallPending { .. }) => {}
        other => panic!("expected second ExternalCallPending, got {other:?}"),
    }

    let captured_contexts = contexts
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let effect_context = captured_contexts
        .iter()
        .find(|context| context.function_name == "effect")
        .expect("effect context should be captured");
    assert_eq!(effect_context.ifc.propagation_mode, PropagationMode::Strict);
    assert_eq!(
        effect_context.ifc.aggregate_summary.integrity_join,
        IntegrityLabel::Untrusted
    );
    assert_eq!(
        effect_context.ifc.arg_summaries[0].integrity_join,
        IntegrityLabel::Trusted
    );
    assert_eq!(
        effect_context.ifc.control_context.pc_integrity(),
        IntegrityLabel::Untrusted
    );
}

#[rstest]
fn resumed_external_return_provenance_flows_into_following_effect() {
    let (mediator, contexts) = recording_mediator();
    let monty_run = MontyRun::new(
        "value = source(\"seed\")\nsink(value)".to_owned(),
        "test.py",
        vec![],
    )
    .expect("parse should succeed");
    let runner = GovernedRunner::new(monty_run, mediator)
        .with_ifc_config(GovernedIfcConfig::strict_with_boundary_seeds());

    let first_progress = runner.run_no_limits(vec![]);
    let first_suspended = match first_progress {
        Ok(GovernedRunProgress::ExternalCallPending { suspended, .. }) => suspended,
        other => panic!("expected source call to suspend, got {other:?}"),
    };

    let second_progress = first_suspended.resume(
        MontyObject::String("payload".to_owned()),
        PrintWriter::Stdout,
    );
    match second_progress {
        Ok(GovernedRunProgress::ExternalCallPending { .. }) => {}
        other => panic!("expected sink call to suspend, got {other:?}"),
    }

    let captured_contexts = contexts
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let sink_context = captured_contexts
        .iter()
        .find(|context| context.function_name == "sink")
        .expect("sink context should be captured");
    assert_eq!(
        sink_context.ifc.arg_summaries[0].integrity_join,
        IntegrityLabel::Untrusted
    );
    assert!(sink_context.ifc.arg_summaries[0].origin_count >= 2);
}
