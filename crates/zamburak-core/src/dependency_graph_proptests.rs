//! Property tests for dependency graph budget invariants.

use proptest::prelude::*;

use super::{DependencyGraph, GraphBudgets, ValueLabels};
use crate::trust::{AuthoritySet, DataLabels, IntegrityLabel};
use crate::value_id::ValueId;

// ---------------------------------------------------------------------------
// Budget invariants
// ---------------------------------------------------------------------------

proptest! {
    /// Node count never exceeds max_values budget.
    #[test]
    fn node_count_never_exceeds_budget(max_values in 1u64..=20) {
        let budgets = GraphBudgets {
            max_values,
            ..GraphBudgets::default()
        };
        let mut graph = DependencyGraph::new(budgets);

        for i in 0..max_values + 5 {
            let _ = graph.insert_value(
                ValueId::new(i),
                ValueLabels {
                    integrity: IntegrityLabel::Trusted,
                    confidentiality: DataLabels::new(),
                    authority: AuthoritySet::new(),
                },
            );
        }

        let count = u64::try_from(graph.node_count()).unwrap_or(u64::MAX);
        prop_assert!(count <= max_values);
    }

    /// Budget monotonicity: is_truncated never transitions from true to false.
    #[test]
    fn truncated_flag_is_monotone(max_values in 1u64..=10) {
        let budgets = GraphBudgets {
            max_values,
            ..GraphBudgets::default()
        };
        let mut graph = DependencyGraph::new(budgets);

        let mut seen_truncated = false;
        for i in 0..max_values + 5 {
            let _ = graph.insert_value(
                ValueId::new(i),
                ValueLabels {
                    integrity: IntegrityLabel::Trusted,
                    confidentiality: DataLabels::new(),
                    authority: AuthoritySet::new(),
                },
            );
            if seen_truncated {
                prop_assert!(graph.is_truncated(),
                    "truncated flag went from true to false after inserting id={i}");
            }
            if graph.is_truncated() {
                seen_truncated = true;
            }
        }
    }

    /// Parent count never exceeds max_parents_per_value.
    #[test]
    fn parent_count_never_exceeds_budget(max_parents in 1u64..=8) {
        let budgets = GraphBudgets {
            max_parents_per_value: max_parents,
            ..GraphBudgets::default()
        };
        let mut graph = DependencyGraph::new(budgets);

        // Insert parent nodes.
        for i in 0..max_parents + 3 {
            graph.insert_value(
                ValueId::new(i),
                ValueLabels {
                    integrity: IntegrityLabel::Trusted,
                    confidentiality: DataLabels::new(),
                    authority: AuthoritySet::new(),
                },
            ).map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;
        }

        // Insert child and add all parents.
        let child_id = max_parents + 3;
        graph.insert_value(
            ValueId::new(child_id),
            ValueLabels {
                integrity: IntegrityLabel::Trusted,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::new(),
            },
        ).map_err(|e| TestCaseError::Fail(format!("{e}").into()))?;

        for i in 0..max_parents + 3 {
            let _ = graph.add_dependency(ValueId::new(child_id), ValueId::new(i));
        }

        let node = graph
            .get_node(&ValueId::new(child_id))
            .unwrap_or_else(|| panic!("child node {child_id} should exist"));
        let parent_count = u64::try_from(node.parents().len())
            .unwrap_or_else(|_| panic!("parent count for child {child_id} should fit in u64"));
        prop_assert!(
            parent_count <= max_parents,
            "parent count {parent_count} exceeded budget {max_parents}",
        );
    }
}
