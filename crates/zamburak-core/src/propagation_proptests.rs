//! Property tests for propagation mode invariants.

use proptest::prelude::*;

use super::{PropagationMode, propagate_labels};
use crate::AuthorityCapability;
use crate::control_context::ExecutionContextSummary;
use crate::summary::DependencySummary;
use crate::trust::{AuthoritySet, DataLabel, DataLabels, IntegrityLabel};
use crate::value_id::ValueId;

// ---------------------------------------------------------------------------
// Arbitrary strategies
// ---------------------------------------------------------------------------

fn arb_integrity_label() -> impl Strategy<Value = IntegrityLabel> {
    prop_oneof![
        Just(IntegrityLabel::Untrusted),
        Just(IntegrityLabel::Trusted),
        Just(IntegrityLabel::Verified),
    ]
}

fn arb_data_label() -> impl Strategy<Value = DataLabel> {
    prop_oneof![
        Just(DataLabel::Pii),
        Just(DataLabel::AuthSecret),
        Just(DataLabel::PrivateEmailBody),
        Just(DataLabel::PaymentInstrument),
        Just(DataLabel::InternalPolicyNote),
    ]
}

fn arb_data_labels() -> impl Strategy<Value = DataLabels> {
    proptest::collection::btree_set(arb_data_label(), 0..=5).prop_map(|set| {
        let labels: Vec<DataLabel> = set.into_iter().collect();
        DataLabels::from_iter(labels)
    })
}

const CAP_NAMES: &[&str] = &["A", "B", "C", "D", "E"];

fn cap_from_index(i: usize) -> Option<AuthorityCapability> {
    let name = CAP_NAMES.get(i)?;
    AuthorityCapability::try_from(*name).ok()
}

fn arb_authority_set() -> impl Strategy<Value = AuthoritySet> {
    proptest::collection::btree_set(0..CAP_NAMES.len(), 0..=CAP_NAMES.len()).prop_map(|indices| {
        let mut set = AuthoritySet::new();
        for cap in indices.into_iter().filter_map(cap_from_index) {
            set.insert(cap);
        }
        set
    })
}

fn arb_summary() -> impl Strategy<Value = DependencySummary> {
    (
        arb_integrity_label(),
        arb_data_labels(),
        arb_authority_set(),
        0..100u32,
        any::<bool>(),
    )
        .prop_map(
            |(integrity, confidentiality, authority, count, trunc)| DependencySummary {
                integrity_join: integrity,
                confidentiality_join: confidentiality,
                authority_join: authority,
                origin_count: count,
                truncated: trunc,
            },
        )
}

fn arb_context() -> impl Strategy<Value = ExecutionContextSummary> {
    (arb_integrity_label(), arb_data_labels()).prop_map(|(integrity, confidentiality)| {
        let mut ctx = ExecutionContextSummary::new();
        let condition = DependencySummary {
            integrity_join: integrity,
            confidentiality_join: confidentiality,
            authority_join: AuthoritySet::new(),
            origin_count: 1,
            truncated: false,
        };
        ctx.push_condition(ValueId::new(0), &condition);
        ctx
    })
}

// ---------------------------------------------------------------------------
// Propagation invariants
// ---------------------------------------------------------------------------

proptest! {
    /// Strict mode is at least as restrictive as Normal: for any set of
    /// operand summaries and control context, the strict-mode result has
    /// integrity <= normal-mode result integrity.
    #[test]
    fn strict_at_least_as_restrictive_as_normal(
        ops in proptest::collection::vec(arb_summary(), 1..=4),
        ctx in arb_context(),
    ) {
        let normal = propagate_labels(PropagationMode::Normal, &ops, &ctx);
        let strict = propagate_labels(PropagationMode::Strict, &ops, &ctx);

        if let (Some(n), Some(s)) = (&normal, &strict) {
            prop_assert!(
                s.integrity_join <= n.integrity_join,
                "strict integrity {:?} > normal integrity {:?}",
                s.integrity_join,
                n.integrity_join,
            );
            // Strict confidentiality is a superset of normal.
            prop_assert!(
                n.confidentiality_join.is_subset_of(&s.confidentiality_join),
                "normal confidentiality not subset of strict",
            );
        }
    }

    /// Adding operands never increases integrity.
    #[test]
    fn adding_operands_never_increases_integrity(
        ops in proptest::collection::vec(arb_summary(), 1..=3),
        extra in arb_summary(),
    ) {
        let ctx = ExecutionContextSummary::new();

        let before = propagate_labels(PropagationMode::Normal, &ops, &ctx);

        let mut extended = ops.clone();
        extended.push(extra);
        let after = propagate_labels(PropagationMode::Normal, &extended, &ctx);

        if let (Some(b), Some(a)) = (&before, &after) {
            prop_assert!(
                a.integrity_join <= b.integrity_join,
                "integrity increased after adding operand: {:?} > {:?}",
                a.integrity_join,
                b.integrity_join,
            );
        }
    }

    /// Adding operands never removes confidentiality labels.
    #[test]
    fn adding_operands_never_removes_confidentiality(
        ops in proptest::collection::vec(arb_summary(), 1..=3),
        extra in arb_summary(),
    ) {
        let ctx = ExecutionContextSummary::new();

        let before = propagate_labels(PropagationMode::Normal, &ops, &ctx);

        let mut extended = ops.clone();
        extended.push(extra);
        let after = propagate_labels(PropagationMode::Normal, &extended, &ctx);

        if let (Some(b), Some(a)) = (&before, &after) {
            prop_assert!(
                b.confidentiality_join.is_subset_of(&a.confidentiality_join),
                "confidentiality labels were removed after adding operand",
            );
        }
    }
}
