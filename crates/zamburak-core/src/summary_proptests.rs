//! Property tests for `DependencySummary` join semantics and
//! transitive summary computation invariants.

use proptest::prelude::*;

use super::{DependencySummary, compute_summary};
use crate::AuthorityCapability;
use crate::dependency_graph::{DependencyGraph, GraphBudgets};
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

// ---------------------------------------------------------------------------
// DependencySummary join algebraic laws
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn summary_join_commutativity(
        a in arb_summary(),
        b in arb_summary(),
    ) {
        prop_assert_eq!(a.join(&b), b.join(&a));
    }

    #[test]
    fn summary_join_associativity(
        a in arb_summary(),
        b in arb_summary(),
        c in arb_summary(),
    ) {
        prop_assert_eq!(a.join(&b).join(&c), a.join(&b.join(&c)));
    }

    #[test]
    fn summary_join_truncation_monotonicity(
        a in arb_summary(),
        b in arb_summary(),
    ) {
        let joined = a.join(&b);
        if a.truncated || b.truncated {
            prop_assert!(joined.truncated);
        }
    }

    #[test]
    fn summary_join_integrity_monotonicity(
        a in arb_summary(),
        b in arb_summary(),
    ) {
        let joined = a.join(&b);
        // Result integrity <= min(a.integrity, b.integrity)
        prop_assert!(joined.integrity_join <= a.integrity_join);
        prop_assert!(joined.integrity_join <= b.integrity_join);
    }

    #[test]
    fn summary_join_confidentiality_monotonicity(
        a in arb_summary(),
        b in arb_summary(),
    ) {
        let joined = a.join(&b);
        // Result confidentiality ⊇ a.confidentiality ∪ b.confidentiality
        prop_assert!(a.confidentiality_join.is_subset_of(&joined.confidentiality_join));
        prop_assert!(b.confidentiality_join.is_subset_of(&joined.confidentiality_join));
    }

    #[test]
    fn summary_join_authority_monotonicity(
        a in arb_summary(),
        b in arb_summary(),
    ) {
        let joined = a.join(&b);
        // Result authority ⊆ a.authority ∩ b.authority
        for cap in joined.authority_join.iter() {
            prop_assert!(a.authority_join.contains(cap));
            prop_assert!(b.authority_join.contains(cap));
        }
    }
}

// ---------------------------------------------------------------------------
// Transitive summary invariants
// ---------------------------------------------------------------------------

proptest! {
    /// Single-node summary matches from_node.
    #[test]
    fn single_node_summary_equals_from_node(
        integrity in arb_integrity_label(),
        confidentiality in arb_data_labels(),
        authority in arb_authority_set(),
    ) {
        let mut graph = DependencyGraph::new(GraphBudgets::default());
        graph
            .insert_value(
                ValueId::new(1),
                integrity,
                confidentiality,
                authority,
            )
            .map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;

        let summary = compute_summary(&graph, &ValueId::new(1), graph.budgets())
            .map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;

        let node = graph.get_node(&ValueId::new(1))
            .ok_or_else(|| TestCaseError::Fail("node missing".into()))?;
        let expected = DependencySummary::from_node(node);

        prop_assert_eq!(summary, expected);
    }

    /// Budget overflow conservatism: max_closure_steps=0 yields unknown_top
    /// for any node with parents.
    #[test]
    fn budget_overflow_yields_unknown_top(
        integrity in arb_integrity_label(),
    ) {
        let budgets = GraphBudgets {
            max_closure_steps: 0,
            ..GraphBudgets::default()
        };
        let mut graph = DependencyGraph::new(budgets);

        graph
            .insert_value(
                ValueId::new(1),
                integrity,
                DataLabels::new(),
                AuthoritySet::new(),
            )
            .map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;
        graph
            .insert_value(
                ValueId::new(2),
                IntegrityLabel::Verified,
                DataLabels::new(),
                AuthoritySet::new(),
            )
            .map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;
        graph
            .add_dependency(ValueId::new(2), ValueId::new(1))
            .map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;

        let summary = compute_summary(&graph, &ValueId::new(2), &budgets)
            .map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;

        prop_assert_eq!(summary, DependencySummary::unknown_top());
    }

    /// Transitivity: if A->B->C, then compute_summary(A) includes C's
    /// integrity influence.
    #[test]
    fn transitive_integrity_propagation(
        c_integrity in arb_integrity_label(),
    ) {
        let mut graph = DependencyGraph::new(GraphBudgets::default());

        graph
            .insert_value(
                ValueId::new(1),
                c_integrity,
                DataLabels::new(),
                AuthoritySet::new(),
            )
            .map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;
        graph
            .insert_value(
                ValueId::new(2),
                IntegrityLabel::Verified,
                DataLabels::new(),
                AuthoritySet::new(),
            )
            .map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;
        graph
            .add_dependency(ValueId::new(2), ValueId::new(1))
            .map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;
        graph
            .insert_value(
                ValueId::new(3),
                IntegrityLabel::Verified,
                DataLabels::new(),
                AuthoritySet::new(),
            )
            .map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;
        graph
            .add_dependency(ValueId::new(3), ValueId::new(2))
            .map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;

        let summary = compute_summary(&graph, &ValueId::new(3), graph.budgets())
            .map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;

        // Summary integrity should be <= c_integrity (transitive influence).
        prop_assert!(summary.integrity_join <= c_integrity);
    }
}
