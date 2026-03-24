//! Unit tests for IFC error display formatting.

use rstest::*;

use super::IfcError;

#[rstest]
#[case(
    IfcError::ValueBudgetExhausted { current: 100, limit: 100 },
    &["100", "limit"]
)]
#[case(
    IfcError::ParentBudgetExhausted { value_id: 42, current: 64, limit: 64 },
    &["42"]
)]
#[case(IfcError::UnknownValueId(999), &["999"])]
#[case(IfcError::DuplicateValueId(42), &["42", "duplicate"])]
#[case(IfcError::CycleDetected { from: 1, to: 1 }, &["cycle"])]
#[case(
    IfcError::ClosureStepBudgetExhausted { steps: 10_000, limit: 10_000 },
    &["10000"]
)]
fn error_display_contains_expected_strings(
    #[case] err: IfcError,
    #[case] expected_fragments: &[&str],
) {
    let msg = format!("{err}");
    for fragment in expected_fragments {
        assert!(
            msg.contains(fragment),
            "message should contain '{fragment}': {msg}"
        );
    }
}
