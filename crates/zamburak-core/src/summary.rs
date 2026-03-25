//! Bounded transitive dependency summary computation.
//!
//! A `DependencySummary` captures the joined integrity, confidentiality,
//! and authority labels of a value's transitive dependency closure.
//! `compute_summary` performs a bounded BFS walk through the dependency
//! graph, returning `unknown_top()` when the closure-step budget is
//! exceeded (fail-closed conservative behaviour).

use std::collections::{HashSet, VecDeque};

use crate::dependency_graph::{DependencyGraph, GraphBudgets, ValueNode};
use crate::ifc_errors::IfcError;
use crate::trust::{AuthoritySet, DataLabels, IntegrityLabel};
use crate::value_id::ValueId;

/// Transitive dependency summary for a runtime value.
///
/// The summary captures the joined (meet/union/intersection) labels
/// across a value's entire dependency closure. When the graph has been
/// truncated or the closure-step budget exceeded, the `truncated` flag
/// is set and labels are conservatively widened.
///
/// # Examples
///
/// ```
/// use zamburak_core::{
///     DependencySummary, IntegrityLabel, DataLabels, AuthoritySet,
/// };
///
/// let top = DependencySummary::unknown_top();
/// assert_eq!(top.integrity_join, IntegrityLabel::Untrusted);
/// assert!(top.truncated);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencySummary {
    /// Joined integrity label (greatest lower bound across closure).
    pub integrity_join: IntegrityLabel,
    /// Joined confidentiality labels (union across closure).
    pub confidentiality_join: DataLabels,
    /// Joined authority capabilities (intersection across closure).
    pub authority_join: AuthoritySet,
    /// Number of origin values contributing to this summary.
    pub origin_count: u32,
    /// Whether the summary is incomplete due to budget overflow.
    pub truncated: bool,
}

impl DependencySummary {
    /// Create a base summary from a single value node.
    #[must_use]
    pub fn from_node(node: &ValueNode) -> Self {
        Self {
            integrity_join: node.integrity(),
            confidentiality_join: node.confidentiality().clone(),
            authority_join: node.authority().clone(),
            origin_count: 1,
            truncated: false,
        }
    }

    /// Combine two summaries using IFC join semantics.
    ///
    /// - Integrity: greatest lower bound (minimum).
    /// - Confidentiality: set union (additive).
    /// - Authority: set intersection (narrowing).
    /// - Origin count: saturating addition.
    /// - Truncated: logical OR.
    #[must_use]
    pub fn join(&self, other: &Self) -> Self {
        Self {
            integrity_join: self.integrity_join.join(other.integrity_join),
            confidentiality_join: self.confidentiality_join.join(&other.confidentiality_join),
            authority_join: self.authority_join.join(&other.authority_join),
            origin_count: self.origin_count.saturating_add(other.origin_count),
            truncated: self.truncated || other.truncated,
        }
    }

    /// Create the conservative fail-closed summary for budget overflow.
    ///
    /// This represents the "worst case" from a security perspective:
    /// `Untrusted` integrity, all confidentiality labels present, no
    /// authority capabilities, and the truncated flag set.
    #[must_use]
    pub fn unknown_top() -> Self {
        Self {
            integrity_join: IntegrityLabel::Untrusted,
            confidentiality_join: DataLabels::all(),
            authority_join: AuthoritySet::new(),
            origin_count: u32::MAX,
            truncated: true,
        }
    }
}

/// Compute the bounded transitive dependency summary for a value.
///
/// Performs a BFS walk through the dependency graph starting from `id`,
/// joining the labels of all reachable ancestors. The computation
/// returns a conservative `Ok(DependencySummary::unknown_top())` summary
/// (fail-closed, not an error) in two cases:
///
/// - The underlying `graph.is_truncated()` is `true` (checked
///   immediately before traversal), indicating a prior budget overflow.
/// - The BFS walk exceeds `budgets.max_closure_steps` during traversal.
///
/// Returns `Err(IfcError::UnknownValueId)` only if the root `id` does
/// not exist in the graph.
///
/// # Examples
///
/// ```
/// use zamburak_core::{
///     DependencyGraph, GraphBudgets, ValueId, ValueLabels,
///     IntegrityLabel, DataLabels, AuthoritySet, DependencySummary,
///     summary::compute_summary,
/// };
///
/// let mut graph = DependencyGraph::new(GraphBudgets::default());
/// graph.insert_value(
///     ValueId::new(1),
///     ValueLabels {
///         integrity: IntegrityLabel::Trusted,
///         confidentiality: DataLabels::new(),
///         authority: AuthoritySet::new(),
///     },
/// ).expect("insert failed");
///
/// let summary = compute_summary(
///     &graph, &ValueId::new(1), graph.budgets(),
/// ).expect("summary failed");
///
/// assert_eq!(summary.integrity_join, IntegrityLabel::Trusted);
/// assert!(!summary.truncated);
/// ```
pub fn compute_summary(
    graph: &DependencyGraph,
    id: &ValueId,
    budgets: &GraphBudgets,
) -> Result<DependencySummary, IfcError> {
    let root_node = graph.get_node(id).ok_or(IfcError::UnknownValueId(*id))?;

    if graph.is_truncated() {
        return Ok(DependencySummary::unknown_top());
    }

    let mut summary = DependencySummary::from_node(root_node);

    let mut visited = HashSet::new();
    visited.insert(*id);

    let mut queue = VecDeque::new();
    enqueue_parents(root_node, &mut queue);

    let mut steps: u64 = 0;

    while let Some(parent_id) = queue.pop_front() {
        if !visited.insert(parent_id) {
            continue;
        }

        steps = steps.saturating_add(1);
        if steps > budgets.max_closure_steps {
            return Ok(DependencySummary::unknown_top());
        }

        if let Some(parent_node) = graph.get_node(&parent_id) {
            let parent_summary = DependencySummary::from_node(parent_node);
            summary = summary.join(&parent_summary);
            enqueue_parents(parent_node, &mut queue);
        }
    }

    Ok(summary)
}

/// Push all parent IDs of a node into the BFS queue.
fn enqueue_parents(node: &ValueNode, queue: &mut VecDeque<ValueId>) {
    for parent_id in node.parents() {
        queue.push_back(*parent_id);
    }
}

#[cfg(test)]
#[path = "summary_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "summary_proptests.rs"]
mod proptests;
