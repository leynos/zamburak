//! Unit tests for IFC error display formatting.

use super::IfcError;

#[test]
fn value_budget_exhausted_display() {
    let err = IfcError::ValueBudgetExhausted {
        current: 100,
        limit: 100,
    };
    let msg = format!("{err}");
    assert!(msg.contains("100"), "message should contain count: {msg}");
    assert!(msg.contains("limit"), "message should mention limit: {msg}");
}

#[test]
fn parent_budget_exhausted_display() {
    let err = IfcError::ParentBudgetExhausted {
        value_id: 42,
        current: 64,
        limit: 64,
    };
    let msg = format!("{err}");
    assert!(msg.contains("42"), "message should contain value ID: {msg}");
}

#[test]
fn unknown_value_id_display() {
    let err = IfcError::UnknownValueId(999);
    let msg = format!("{err}");
    assert!(msg.contains("999"), "message should contain ID: {msg}");
}

#[test]
fn duplicate_value_id_display() {
    let err = IfcError::DuplicateValueId(42);
    let msg = format!("{err}");
    assert!(msg.contains("42"), "message should contain ID: {msg}");
    assert!(
        msg.contains("duplicate"),
        "message should mention duplicate: {msg}"
    );
}

#[test]
fn cycle_detected_display() {
    let err = IfcError::CycleDetected { from: 1, to: 1 };
    let msg = format!("{err}");
    assert!(msg.contains("cycle"), "message should mention cycle: {msg}");
}

#[test]
fn closure_step_budget_exhausted_display() {
    let err = IfcError::ClosureStepBudgetExhausted {
        steps: 10_000,
        limit: 10_000,
    };
    let msg = format!("{err}");
    assert!(
        msg.contains("10000"),
        "message should contain step count: {msg}",
    );
}
