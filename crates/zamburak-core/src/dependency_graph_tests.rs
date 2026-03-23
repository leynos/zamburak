//! Unit tests for the `ValueId`-keyed dependency graph.

use crate::ifc_errors::IfcError;
use crate::trust::{AuthoritySet, DataLabel, DataLabels, IntegrityLabel};
use crate::value_id::ValueId;

use super::{DependencyGraph, GraphBudgets, ValueLabels};

fn small_budgets() -> GraphBudgets {
    GraphBudgets {
        max_values: 3,
        max_parents_per_value: 2,
        max_closure_steps: 10,
        max_witness_depth: 4,
    }
}

fn insert_trusted(graph: &mut DependencyGraph, id: u64) {
    graph
        .insert_value(
            ValueId::new(id),
            ValueLabels {
                integrity: IntegrityLabel::Trusted,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert should succeed in test");
}

#[test]
fn insert_value_happy_path() {
    let mut graph = DependencyGraph::new(small_budgets());
    insert_trusted(&mut graph, 1);

    assert_eq!(graph.node_count(), 1);
    assert!(graph.contains(&ValueId::new(1)));
    assert!(!graph.contains(&ValueId::new(99)));
}

#[test]
fn insert_value_preserves_labels() {
    let mut graph = DependencyGraph::new(GraphBudgets::default());
    let confidentiality = DataLabels::from_iter([DataLabel::Pii, DataLabel::AuthSecret]);

    graph
        .insert_value(
            ValueId::new(1),
            ValueLabels {
                integrity: IntegrityLabel::Untrusted,
                confidentiality: confidentiality.clone(),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert should succeed");

    let node = graph.get_node(&ValueId::new(1)).expect("node should exist");
    assert_eq!(node.integrity(), IntegrityLabel::Untrusted);
    assert_eq!(node.confidentiality(), &confidentiality);
}

#[test]
fn insert_value_budget_exhaustion() {
    let mut graph = DependencyGraph::new(small_budgets());
    insert_trusted(&mut graph, 1);
    insert_trusted(&mut graph, 2);
    insert_trusted(&mut graph, 3);

    assert!(!graph.is_truncated());

    let result = graph.insert_value(
        ValueId::new(4),
        ValueLabels {
            integrity: IntegrityLabel::Trusted,
            confidentiality: DataLabels::new(),
            authority: AuthoritySet::new(),
        },
    );

    assert!(matches!(
        result,
        Err(IfcError::ValueBudgetExhausted {
            current: 3,
            limit: 3
        })
    ));
    assert!(graph.is_truncated());
    assert_eq!(graph.node_count(), 3);
}

#[test]
fn add_dependency_happy_path() {
    let mut graph = DependencyGraph::new(small_budgets());
    insert_trusted(&mut graph, 1);
    insert_trusted(&mut graph, 2);

    graph
        .add_dependency(ValueId::new(2), ValueId::new(1))
        .expect("edge should succeed");

    let child = graph
        .get_node(&ValueId::new(2))
        .expect("child should exist");
    assert_eq!(child.parents(), &[ValueId::new(1)]);
}

#[test]
fn add_dependency_self_loop_rejected() {
    let mut graph = DependencyGraph::new(small_budgets());
    insert_trusted(&mut graph, 1);

    let result = graph.add_dependency(ValueId::new(1), ValueId::new(1));
    assert!(matches!(
        result,
        Err(IfcError::CycleDetected { from: 1, to: 1 })
    ),);
}

#[test]
fn add_dependency_unknown_child() {
    let mut graph = DependencyGraph::new(small_budgets());
    insert_trusted(&mut graph, 1);

    let result = graph.add_dependency(ValueId::new(99), ValueId::new(1));
    assert!(matches!(result, Err(IfcError::UnknownValueId(99))));
}

#[test]
fn add_dependency_unknown_parent() {
    let mut graph = DependencyGraph::new(small_budgets());
    insert_trusted(&mut graph, 1);

    let result = graph.add_dependency(ValueId::new(1), ValueId::new(99));
    assert!(matches!(result, Err(IfcError::UnknownValueId(99))));
}

#[test]
fn add_dependency_parent_budget_exhaustion() {
    let mut graph = DependencyGraph::new(small_budgets());
    insert_trusted(&mut graph, 1);
    insert_trusted(&mut graph, 2);
    insert_trusted(&mut graph, 3);

    // max_parents_per_value is 2
    graph
        .add_dependency(ValueId::new(3), ValueId::new(1))
        .expect("first edge should succeed");
    graph
        .add_dependency(ValueId::new(3), ValueId::new(2))
        .expect("second edge should succeed");

    // Insert a fourth node to use as third parent (need to expand budget
    // for values but keep parent budget at 2).
    let mut bigger = DependencyGraph::new(GraphBudgets {
        max_values: 10,
        max_parents_per_value: 2,
        ..small_budgets()
    });
    insert_trusted(&mut bigger, 1);
    insert_trusted(&mut bigger, 2);
    insert_trusted(&mut bigger, 3);
    insert_trusted(&mut bigger, 4);

    bigger
        .add_dependency(ValueId::new(4), ValueId::new(1))
        .expect("first edge ok");
    bigger
        .add_dependency(ValueId::new(4), ValueId::new(2))
        .expect("second edge ok");

    let result = bigger.add_dependency(ValueId::new(4), ValueId::new(3));
    assert!(matches!(
        result,
        Err(IfcError::ParentBudgetExhausted {
            value_id: 4,
            current: 2,
            limit: 2,
        })
    ));
}

#[test]
fn node_count_tracks_insertions() {
    let mut graph = DependencyGraph::new(GraphBudgets::default());
    assert_eq!(graph.node_count(), 0);

    insert_trusted(&mut graph, 1);
    assert_eq!(graph.node_count(), 1);

    insert_trusted(&mut graph, 2);
    assert_eq!(graph.node_count(), 2);
}

#[test]
fn get_node_returns_none_for_missing_id() {
    let graph = DependencyGraph::new(GraphBudgets::default());
    assert!(graph.get_node(&ValueId::new(1)).is_none());
}

#[test]
fn multiple_parents_recorded_in_order() {
    let mut graph = DependencyGraph::new(GraphBudgets::default());
    insert_trusted(&mut graph, 10);
    insert_trusted(&mut graph, 20);
    insert_trusted(&mut graph, 30);

    graph
        .add_dependency(ValueId::new(30), ValueId::new(10))
        .expect("edge ok");
    graph
        .add_dependency(ValueId::new(30), ValueId::new(20))
        .expect("edge ok");

    let node = graph
        .get_node(&ValueId::new(30))
        .expect("node should exist");
    assert_eq!(node.parents(), &[ValueId::new(10), ValueId::new(20)]);
}
