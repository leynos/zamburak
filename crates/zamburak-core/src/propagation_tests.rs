//! Unit tests for propagation mode types and label propagation rules.

use crate::control_context::ExecutionContextSummary;
use crate::summary::DependencySummary;
use crate::trust::{AuthoritySet, DataLabel, DataLabels, IntegrityLabel};
use crate::value_id::ValueId;

use super::{PropagationMode, propagate_labels};

fn trusted_summary() -> DependencySummary {
    DependencySummary {
        integrity_join: IntegrityLabel::Trusted,
        confidentiality_join: DataLabels::new(),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    }
}

// ---------------------------------------------------------------------------
// Normal mode
// ---------------------------------------------------------------------------

#[test]
fn normal_empty_operands_returns_none() {
    let ctx = ExecutionContextSummary::new();
    let result = propagate_labels(PropagationMode::Normal, &[], &ctx);
    assert!(result.is_none());
}

#[test]
fn normal_single_operand_returns_operand() {
    let ctx = ExecutionContextSummary::new();
    let operand = trusted_summary();

    let result = propagate_labels(
        PropagationMode::Normal,
        std::slice::from_ref(&operand),
        &ctx,
    );
    assert_eq!(result, Some(operand));
}

#[test]
fn normal_multiple_operands_joins_data_only() {
    let mut ctx = ExecutionContextSummary::new();
    // Degrade context integrity — should NOT affect Normal mode result.
    let condition = DependencySummary {
        integrity_join: IntegrityLabel::Untrusted,
        confidentiality_join: DataLabels::from_iter([DataLabel::Pii]),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };
    ctx.push_condition(ValueId::new(99), &condition);

    let a = DependencySummary {
        integrity_join: IntegrityLabel::Verified,
        confidentiality_join: DataLabels::new(),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };
    let b = DependencySummary {
        integrity_join: IntegrityLabel::Trusted,
        confidentiality_join: DataLabels::from_iter([DataLabel::AuthSecret]),
        authority_join: AuthoritySet::new(),
        origin_count: 2,
        truncated: false,
    };

    let result = propagate_labels(PropagationMode::Normal, &[a, b], &ctx);
    let summary = result.expect("should produce summary");

    // Integrity = min(Verified, Trusted) = Trusted
    assert_eq!(summary.integrity_join, IntegrityLabel::Trusted);
    // Confidentiality = union = {AuthSecret}
    assert!(
        summary
            .confidentiality_join
            .contains(&DataLabel::AuthSecret)
    );
    // Context's Pii label should NOT be present (Normal mode).
    assert!(!summary.confidentiality_join.contains(&DataLabel::Pii));
    assert_eq!(summary.origin_count, 3);
}

// ---------------------------------------------------------------------------
// Strict mode
// ---------------------------------------------------------------------------

#[test]
fn strict_empty_operands_returns_context_summary() {
    let mut ctx = ExecutionContextSummary::new();
    let condition = DependencySummary {
        integrity_join: IntegrityLabel::Untrusted,
        confidentiality_join: DataLabels::from_iter([DataLabel::Pii]),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };
    ctx.push_condition(ValueId::new(1), &condition);

    let result = propagate_labels(PropagationMode::Strict, &[], &ctx);
    let summary = result.expect("strict mode should return context summary");

    assert_eq!(summary.integrity_join, IntegrityLabel::Untrusted);
    assert!(summary.confidentiality_join.contains(&DataLabel::Pii));
}

#[test]
fn strict_includes_context_in_join() {
    let mut ctx = ExecutionContextSummary::new();
    let condition = DependencySummary {
        integrity_join: IntegrityLabel::Untrusted,
        confidentiality_join: DataLabels::from_iter([DataLabel::Pii]),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };
    ctx.push_condition(ValueId::new(1), &condition);

    let operand = DependencySummary {
        integrity_join: IntegrityLabel::Verified,
        confidentiality_join: DataLabels::from_iter([DataLabel::AuthSecret]),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };

    let result = propagate_labels(PropagationMode::Strict, &[operand], &ctx);
    let summary = result.expect("should produce summary");

    // Integrity = min(Verified, Untrusted) = Untrusted
    assert_eq!(summary.integrity_join, IntegrityLabel::Untrusted);
    // Confidentiality = union(AuthSecret, Pii)
    assert!(
        summary
            .confidentiality_join
            .contains(&DataLabel::AuthSecret)
    );
    assert!(summary.confidentiality_join.contains(&DataLabel::Pii));
}

#[test]
fn strict_is_at_least_as_restrictive_as_normal() {
    let mut ctx = ExecutionContextSummary::new();
    let condition = DependencySummary {
        integrity_join: IntegrityLabel::Untrusted,
        confidentiality_join: DataLabels::from_iter([DataLabel::Pii]),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };
    ctx.push_condition(ValueId::new(1), &condition);

    let operand = trusted_summary();
    let operands = [operand];

    let normal = propagate_labels(PropagationMode::Normal, &operands, &ctx)
        .expect("normal should produce summary");
    let strict = propagate_labels(PropagationMode::Strict, &operands, &ctx)
        .expect("strict should produce summary");

    // Strict integrity should be <= Normal integrity.
    assert!(strict.integrity_join <= normal.integrity_join);
    // Strict confidentiality should be a superset of Normal confidentiality.
    assert!(
        normal
            .confidentiality_join
            .is_subset_of(&strict.confidentiality_join)
    );
}
