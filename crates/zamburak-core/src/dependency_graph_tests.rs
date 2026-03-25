//! Unit tests for the `ValueId`-keyed dependency graph.

use rstest::*;

use crate::ifc_errors::IfcError;
use crate::trust::{AuthoritySet, DataLabel, DataLabels, IntegrityLabel};
use crate::value_id::ValueId;

use super::{DependencyGraph, GraphBudgets, ValueLabels};

#[fixture]
fn small_budgets() -> GraphBudgets {
    GraphBudgets {
        max_values: 3,
        max_parents_per_value: 2,
        max_closure_steps: 10,
        max_witness_depth: 4,
    }
}

#[fixture]
fn default_budgets() -> GraphBudgets {
    GraphBudgets::default()
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

#[rstest]
fn insert_value_happy_path(small_budgets: GraphBudgets) {
    let mut graph = DependencyGraph::new(small_budgets);
    insert_trusted(&mut graph, 1);

    assert_eq!(graph.node_count(), 1);
    assert!(graph.contains(&ValueId::new(1)));
    assert!(!graph.contains(&ValueId::new(99)));
}

#[rstest]
fn insert_value_preserves_labels(default_budgets: GraphBudgets) {
    let mut graph = DependencyGraph::new(default_budgets);
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

#[rstest]
fn insert_value_duplicate_id_rejected(default_budgets: GraphBudgets) {
    let mut graph = DependencyGraph::new(default_budgets);
    insert_trusted(&mut graph, 1);

    let result = graph.insert_value(
        ValueId::new(1),
        ValueLabels {
            integrity: IntegrityLabel::Untrusted,
            confidentiality: DataLabels::from_iter([DataLabel::Pii]),
            authority: AuthoritySet::new(),
        },
    );

    assert!(matches!(
        result,
        Err(IfcError::DuplicateValueId(id)) if id == ValueId::new(1)
    ));
    assert_eq!(graph.node_count(), 1);
    // Original node should be unchanged.
    let node = graph.get_node(&ValueId::new(1)).expect("node should exist");
    assert_eq!(node.integrity(), IntegrityLabel::Trusted);
}

#[rstest]
fn insert_value_budget_exhaustion(small_budgets: GraphBudgets) {
    let mut graph = DependencyGraph::new(small_budgets);
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

#[rstest]
fn add_dependency_happy_path(small_budgets: GraphBudgets) {
    let mut graph = DependencyGraph::new(small_budgets);
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

#[rstest]
fn add_dependency_self_loop_rejected(small_budgets: GraphBudgets) {
    let mut graph = DependencyGraph::new(small_budgets);
    insert_trusted(&mut graph, 1);

    let result = graph.add_dependency(ValueId::new(1), ValueId::new(1));
    assert!(matches!(
        result,
        Err(IfcError::CycleDetected { from, to })
        if from == ValueId::new(1) && to == ValueId::new(1)
    ));
}

#[rstest]
fn add_dependency_unknown_child(small_budgets: GraphBudgets) {
    let mut graph = DependencyGraph::new(small_budgets);
    insert_trusted(&mut graph, 1);

    let result = graph.add_dependency(ValueId::new(99), ValueId::new(1));
    assert!(matches!(
        result,
        Err(IfcError::UnknownValueId(id)) if id == ValueId::new(99)
    ));
}

#[rstest]
fn add_dependency_unknown_parent(small_budgets: GraphBudgets) {
    let mut graph = DependencyGraph::new(small_budgets);
    insert_trusted(&mut graph, 1);

    let result = graph.add_dependency(ValueId::new(1), ValueId::new(99));
    assert!(matches!(
        result,
        Err(IfcError::UnknownValueId(id)) if id == ValueId::new(99)
    ));
}

#[rstest]
fn add_dependency_parent_budget_exhaustion(small_budgets: GraphBudgets) {
    // Expand value budget but keep parent budget at 2.
    let budgets = GraphBudgets {
        max_values: 10,
        max_parents_per_value: 2,
        ..small_budgets
    };
    let mut graph = DependencyGraph::new(budgets);
    insert_trusted(&mut graph, 1);
    insert_trusted(&mut graph, 2);
    insert_trusted(&mut graph, 3);
    insert_trusted(&mut graph, 4);

    graph
        .add_dependency(ValueId::new(4), ValueId::new(1))
        .expect("first edge ok");
    graph
        .add_dependency(ValueId::new(4), ValueId::new(2))
        .expect("second edge ok");

    let result = graph.add_dependency(ValueId::new(4), ValueId::new(3));
    assert!(matches!(
        result,
        Err(IfcError::ParentBudgetExhausted {
            value_id,
            current: 2,
            limit: 2,
        }) if value_id == ValueId::new(4)
    ));
}

#[rstest]
fn add_dependency_duplicate_edge_rejected(default_budgets: GraphBudgets) {
    let mut graph = DependencyGraph::new(default_budgets);
    insert_trusted(&mut graph, 1);
    insert_trusted(&mut graph, 2);

    graph
        .add_dependency(ValueId::new(2), ValueId::new(1))
        .expect("first edge should succeed");

    let result = graph.add_dependency(ValueId::new(2), ValueId::new(1));
    assert!(matches!(
        result,
        Err(IfcError::DuplicateEdge { child, parent })
        if child == ValueId::new(2) && parent == ValueId::new(1)
    ));

    // Parent list should still have exactly one entry.
    let node = graph
        .get_node(&ValueId::new(2))
        .expect("child should exist");
    assert_eq!(node.parents().len(), 1);
}

struct CycleTestCase {
    nodes: Vec<u64>,
    forward_edges: Vec<(u64, u64)>,
    closing_edge: (u64, u64),
    expected: IfcError,
}

#[rstest]
#[case(CycleTestCase {
    nodes: vec![1u64, 2],
    forward_edges: vec![(2u64, 1u64)],
    closing_edge: (1u64, 2u64),
    expected: IfcError::CycleDetected {
        from: ValueId::new(1),
        to: ValueId::new(2),
    },
})]
#[case(CycleTestCase {
    nodes: vec![1u64, 2, 3],
    forward_edges: vec![(2u64, 1u64), (3u64, 2u64)],
    closing_edge: (1u64, 3u64),
    expected: IfcError::CycleDetected {
        from: ValueId::new(1),
        to: ValueId::new(3),
    },
})]
fn add_dependency_cycle_rejected(default_budgets: GraphBudgets, #[case] tc: CycleTestCase) {
    let mut graph = DependencyGraph::new(default_budgets);
    for id in tc.nodes {
        insert_trusted(&mut graph, id);
    }
    for (child, parent) in &tc.forward_edges {
        graph
            .add_dependency(ValueId::new(*child), ValueId::new(*parent))
            .expect("forward edge should succeed");
    }
    let (child, parent) = tc.closing_edge;
    let result = graph.add_dependency(ValueId::new(child), ValueId::new(parent));
    assert_eq!(result, Err(tc.expected));
}

#[rstest]
fn add_dependency_reachability_budget_exhaustion() {
    // Use a closure-step budget of 2, allowing edge 2→1 (1 BFS step)
    // and edge 3→2 (2 BFS steps: visit 2, visit 1). Then adding
    // edge 5→3 requires walking 3→2→1 (3 BFS steps), exceeding the
    // budget of 2. Since no cycle actually exists, this is a pure
    // budget-exhaustion conservative rejection.
    let budgets = GraphBudgets {
        max_values: 100,
        max_parents_per_value: 64,
        max_closure_steps: 2,
        max_witness_depth: 32,
    };
    let mut graph = DependencyGraph::new(budgets);

    // Build a chain: 1 ← 2 ← 3 and an unconnected node 5.
    for i in [1, 2, 3, 5] {
        insert_trusted(&mut graph, i);
    }
    graph
        .add_dependency(ValueId::new(2), ValueId::new(1))
        .expect("edge 2→1");
    graph
        .add_dependency(ValueId::new(3), ValueId::new(2))
        .expect("edge 3→2");

    assert!(!graph.is_truncated());

    // Adding 5→3 would walk 3→2→1 (3 steps > budget 2). The graph
    // should be marked truncated and the edge conservatively rejected.
    let result = graph.add_dependency(ValueId::new(5), ValueId::new(3));
    assert!(matches!(
        result,
        Err(IfcError::ClosureStepBudgetExhausted { .. })
    ));
    assert!(graph.is_truncated());
}

#[rstest]
fn node_count_tracks_insertions(default_budgets: GraphBudgets) {
    let mut graph = DependencyGraph::new(default_budgets);
    assert_eq!(graph.node_count(), 0);

    insert_trusted(&mut graph, 1);
    assert_eq!(graph.node_count(), 1);

    insert_trusted(&mut graph, 2);
    assert_eq!(graph.node_count(), 2);
}

#[rstest]
fn get_node_returns_none_for_missing_id(default_budgets: GraphBudgets) {
    let graph = DependencyGraph::new(default_budgets);
    assert!(graph.get_node(&ValueId::new(1)).is_none());
}

#[rstest]
fn multiple_parents_recorded_in_order(default_budgets: GraphBudgets) {
    let mut graph = DependencyGraph::new(default_budgets);
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
