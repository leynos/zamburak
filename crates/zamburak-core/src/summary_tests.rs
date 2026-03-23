//! Unit tests for dependency summary computation.

use crate::AuthorityCapability;
use crate::dependency_graph::{DependencyGraph, GraphBudgets, ValueLabels};
use crate::trust::{AuthoritySet, DataLabel, DataLabels, IntegrityLabel};
use crate::value_id::ValueId;

use super::{DependencySummary, compute_summary};

fn cap(name: &str) -> AuthorityCapability {
    AuthorityCapability::try_from(name).expect("invalid capability name in test")
}

fn default_graph() -> DependencyGraph {
    DependencyGraph::new(GraphBudgets::default())
}

// ---------------------------------------------------------------------------
// DependencySummary::from_node
// ---------------------------------------------------------------------------

#[test]
fn from_node_captures_all_labels() {
    let mut graph = default_graph();
    let labels = DataLabels::from_iter([DataLabel::Pii]);
    let auth = AuthoritySet::from_iter([cap("A")]);

    graph
        .insert_value(
            ValueId::new(1),
            ValueLabels {
                integrity: IntegrityLabel::Verified,
                confidentiality: labels.clone(),
                authority: auth.clone(),
            },
        )
        .expect("insert ok");

    let node = graph.get_node(&ValueId::new(1)).expect("node exists");
    let summary = DependencySummary::from_node(node);

    assert_eq!(summary.integrity_join, IntegrityLabel::Verified);
    assert_eq!(summary.confidentiality_join, labels);
    assert_eq!(summary.authority_join, auth);
    assert_eq!(summary.origin_count, 1);
    assert!(!summary.truncated);
}

// ---------------------------------------------------------------------------
// DependencySummary::join
// ---------------------------------------------------------------------------

#[test]
fn join_integrity_takes_minimum() {
    let a = DependencySummary {
        integrity_join: IntegrityLabel::Verified,
        confidentiality_join: DataLabels::new(),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };
    let b = DependencySummary {
        integrity_join: IntegrityLabel::Untrusted,
        ..a.clone()
    };

    let joined = a.join(&b);
    assert_eq!(joined.integrity_join, IntegrityLabel::Untrusted);
}

#[test]
fn join_confidentiality_is_union() {
    let a = DependencySummary {
        integrity_join: IntegrityLabel::Trusted,
        confidentiality_join: DataLabels::from_iter([DataLabel::Pii]),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };
    let b = DependencySummary {
        confidentiality_join: DataLabels::from_iter([DataLabel::AuthSecret]),
        ..a.clone()
    };

    let joined = a.join(&b);
    assert!(joined.confidentiality_join.contains(&DataLabel::Pii));
    assert!(joined.confidentiality_join.contains(&DataLabel::AuthSecret));
}

#[test]
fn join_authority_is_intersection() {
    let a = DependencySummary {
        integrity_join: IntegrityLabel::Trusted,
        confidentiality_join: DataLabels::new(),
        authority_join: AuthoritySet::from_iter([cap("A"), cap("B")]),
        origin_count: 1,
        truncated: false,
    };
    let b = DependencySummary {
        authority_join: AuthoritySet::from_iter([cap("B"), cap("C")]),
        ..a.clone()
    };

    let joined = a.join(&b);
    assert!(joined.authority_join.contains(&cap("B")));
    assert!(!joined.authority_join.contains(&cap("A")));
    assert!(!joined.authority_join.contains(&cap("C")));
}

#[test]
fn join_origin_count_saturates() {
    let a = DependencySummary {
        integrity_join: IntegrityLabel::Trusted,
        confidentiality_join: DataLabels::new(),
        authority_join: AuthoritySet::new(),
        origin_count: u32::MAX,
        truncated: false,
    };
    let b = DependencySummary {
        origin_count: 1,
        ..a.clone()
    };

    assert_eq!(a.join(&b).origin_count, u32::MAX);
}

#[test]
fn join_truncation_propagates() {
    let normal = DependencySummary {
        integrity_join: IntegrityLabel::Trusted,
        confidentiality_join: DataLabels::new(),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };
    let trunc = DependencySummary {
        truncated: true,
        ..normal.clone()
    };

    assert!(normal.join(&trunc).truncated);
    assert!(trunc.join(&normal).truncated);
    assert!(trunc.join(&trunc).truncated);
    assert!(!normal.join(&normal).truncated);
}

#[test]
fn join_is_commutative() {
    let a = DependencySummary {
        integrity_join: IntegrityLabel::Verified,
        confidentiality_join: DataLabels::from_iter([DataLabel::Pii]),
        authority_join: AuthoritySet::from_iter([cap("A"), cap("B")]),
        origin_count: 3,
        truncated: false,
    };
    let b = DependencySummary {
        integrity_join: IntegrityLabel::Untrusted,
        confidentiality_join: DataLabels::from_iter([DataLabel::AuthSecret]),
        authority_join: AuthoritySet::from_iter([cap("B"), cap("C")]),
        origin_count: 5,
        truncated: true,
    };

    assert_eq!(a.join(&b), b.join(&a));
}

// ---------------------------------------------------------------------------
// DependencySummary::unknown_top
// ---------------------------------------------------------------------------

#[test]
fn unknown_top_is_conservative() {
    let top = DependencySummary::unknown_top();

    assert_eq!(top.integrity_join, IntegrityLabel::Untrusted);
    assert_eq!(top.confidentiality_join, DataLabels::all());
    assert!(top.authority_join.is_empty());
    assert_eq!(top.origin_count, u32::MAX);
    assert!(top.truncated);
}

// ---------------------------------------------------------------------------
// compute_summary
// ---------------------------------------------------------------------------

#[test]
fn compute_summary_single_node() {
    let mut graph = default_graph();
    graph
        .insert_value(
            ValueId::new(1),
            ValueLabels {
                integrity: IntegrityLabel::Verified,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert ok");

    let summary = compute_summary(&graph, &ValueId::new(1), graph.budgets()).expect("summary ok");

    assert_eq!(summary.integrity_join, IntegrityLabel::Verified);
    assert_eq!(summary.origin_count, 1);
    assert!(!summary.truncated);
}

#[test]
fn compute_summary_chain_propagates_labels() {
    let mut graph = default_graph();

    // C -> B -> A
    graph
        .insert_value(
            ValueId::new(1),
            ValueLabels {
                integrity: IntegrityLabel::Untrusted,
                confidentiality: DataLabels::from_iter([DataLabel::Pii]),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert A");

    graph
        .insert_value(
            ValueId::new(2),
            ValueLabels {
                integrity: IntegrityLabel::Trusted,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert B");
    graph
        .add_dependency(ValueId::new(2), ValueId::new(1))
        .expect("B->A");

    graph
        .insert_value(
            ValueId::new(3),
            ValueLabels {
                integrity: IntegrityLabel::Verified,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert C");
    graph
        .add_dependency(ValueId::new(3), ValueId::new(2))
        .expect("C->B");

    let summary = compute_summary(&graph, &ValueId::new(3), graph.budgets()).expect("summary ok");

    // Integrity should be Untrusted (min across chain).
    assert_eq!(summary.integrity_join, IntegrityLabel::Untrusted);
    // Confidentiality should include A's Pii label.
    assert!(summary.confidentiality_join.contains(&DataLabel::Pii));
    assert_eq!(summary.origin_count, 3);
}

#[test]
fn compute_summary_diamond_deduplicates() {
    let mut graph = default_graph();

    // D depends on B and C; both B and C depend on A.
    graph
        .insert_value(
            ValueId::new(1),
            ValueLabels {
                integrity: IntegrityLabel::Untrusted,
                confidentiality: DataLabels::from_iter([DataLabel::AuthSecret]),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert A");

    graph
        .insert_value(
            ValueId::new(2),
            ValueLabels {
                integrity: IntegrityLabel::Trusted,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert B");
    graph
        .add_dependency(ValueId::new(2), ValueId::new(1))
        .expect("B->A");

    graph
        .insert_value(
            ValueId::new(3),
            ValueLabels {
                integrity: IntegrityLabel::Trusted,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert C");
    graph
        .add_dependency(ValueId::new(3), ValueId::new(1))
        .expect("C->A");

    graph
        .insert_value(
            ValueId::new(4),
            ValueLabels {
                integrity: IntegrityLabel::Verified,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert D");
    graph
        .add_dependency(ValueId::new(4), ValueId::new(2))
        .expect("D->B");
    graph
        .add_dependency(ValueId::new(4), ValueId::new(3))
        .expect("D->C");

    let summary = compute_summary(&graph, &ValueId::new(4), graph.budgets()).expect("summary ok");

    assert_eq!(summary.integrity_join, IntegrityLabel::Untrusted);
    assert!(
        summary
            .confidentiality_join
            .contains(&DataLabel::AuthSecret)
    );
    // A is visited once (deduplication), so origin_count = 4 (D, B, C, A).
    assert_eq!(summary.origin_count, 4);
}

#[test]
fn compute_summary_budget_overflow_yields_unknown_top() {
    let budgets = GraphBudgets {
        max_closure_steps: 0,
        ..GraphBudgets::default()
    };
    let mut graph = DependencyGraph::new(budgets);

    graph
        .insert_value(
            ValueId::new(1),
            ValueLabels {
                integrity: IntegrityLabel::Verified,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert A");
    graph
        .insert_value(
            ValueId::new(2),
            ValueLabels {
                integrity: IntegrityLabel::Verified,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert B");
    graph
        .add_dependency(ValueId::new(2), ValueId::new(1))
        .expect("B->A");

    let summary = compute_summary(&graph, &ValueId::new(2), &budgets)
        .expect("summary should succeed with unknown_top");

    assert_eq!(summary, DependencySummary::unknown_top());
}

#[test]
fn compute_summary_missing_id_returns_error() {
    let graph = default_graph();
    let result = compute_summary(&graph, &ValueId::new(99), graph.budgets());

    assert!(matches!(result, Err(crate::IfcError::UnknownValueId(99)),));
}
