//! `ValueId`-keyed dependency DAG with budget-enforced edge insertion.
//!
//! The dependency graph tracks direct data dependencies between runtime
//! values. Each value node carries integrity, confidentiality, and
//! authority labels. Budget limits constrain graph growth; exceeding a
//! budget marks the graph as truncated, triggering fail-closed
//! conservative summaries downstream.

use std::collections::HashMap;

use crate::ifc_errors::IfcError;
use crate::trust::{AuthoritySet, DataLabels, IntegrityLabel};
use crate::value_id::ValueId;

/// Budget configuration for the IFC dependency graph.
///
/// Fields mirror `PolicyBudgets` in `zamburak-policy` without introducing
/// a dependency on the policy crate. The caller (Task 0.6.3 observer
/// wiring) is responsible for constructing `GraphBudgets` from policy
/// configuration.
///
/// # Examples
///
/// ```
/// use zamburak_core::GraphBudgets;
///
/// let budgets = GraphBudgets::default();
/// assert_eq!(budgets.max_values, 100_000);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GraphBudgets {
    /// Maximum number of tracked values in the graph.
    pub max_values: u64,
    /// Maximum number of direct parent dependencies per value.
    pub max_parents_per_value: u64,
    /// Maximum number of closure traversal steps during summary
    /// computation.
    pub max_closure_steps: u64,
    /// Maximum witness depth in explanations.
    pub max_witness_depth: u64,
}

impl Default for GraphBudgets {
    fn default() -> Self {
        Self {
            max_values: 100_000,
            max_parents_per_value: 64,
            max_closure_steps: 10_000,
            max_witness_depth: 32,
        }
    }
}

/// Per-value metadata stored in the dependency graph.
///
/// Each node records the value's identity, its IFC labels, and its
/// direct parent dependencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueNode {
    id: ValueId,
    integrity: IntegrityLabel,
    confidentiality: DataLabels,
    authority: AuthoritySet,
    parents: Vec<ValueId>,
}

impl ValueNode {
    /// Return the value's identifier.
    #[must_use]
    pub const fn id(&self) -> ValueId {
        self.id
    }

    /// Return the value's integrity label.
    #[must_use]
    pub const fn integrity(&self) -> IntegrityLabel {
        self.integrity
    }

    /// Return the value's confidentiality label set.
    #[must_use]
    pub const fn confidentiality(&self) -> &DataLabels {
        &self.confidentiality
    }

    /// Return the value's authority capability set.
    #[must_use]
    pub const fn authority(&self) -> &AuthoritySet {
        &self.authority
    }

    /// Return the value's direct parent dependencies.
    #[must_use]
    pub fn parents(&self) -> &[ValueId] {
        &self.parents
    }
}

/// IFC labels for a single runtime value.
///
/// Bundles the three label dimensions (integrity, confidentiality,
/// authority) used in value insertion and summary propagation.
///
/// # Examples
///
/// ```
/// use zamburak_core::{ValueLabels, IntegrityLabel, DataLabels, AuthoritySet};
///
/// let labels = ValueLabels {
///     integrity: IntegrityLabel::Trusted,
///     confidentiality: DataLabels::new(),
///     authority: AuthoritySet::new(),
/// };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueLabels {
    /// Integrity level of the value.
    ///
    /// Tracks the trustworthiness of the value's source, where
    /// `Verified` is strongest, `Trusted` is moderate, and
    /// `Untrusted` is weakest.
    pub integrity: IntegrityLabel,

    /// Confidentiality classification set.
    ///
    /// Tracks which confidentiality labels (e.g., PII, auth secrets)
    /// are associated with the value. Labels accumulate through
    /// dependency joins.
    pub confidentiality: DataLabels,

    /// Authority capability set.
    ///
    /// Tracks which capabilities the value possesses. Capabilities
    /// narrow (intersect) through dependency joins.
    pub authority: AuthoritySet,
}

/// `ValueId`-keyed dependency DAG with budget enforcement.
///
/// The graph is a directed acyclic graph by construction: values are
/// created in temporal order, and dependencies can only point from newer
/// values to pre-existing values. Self-loops are explicitly rejected.
///
/// # Examples
///
/// ```
/// use zamburak_core::{
///     DependencyGraph, GraphBudgets, ValueId, ValueLabels,
///     IntegrityLabel, DataLabels, AuthoritySet,
/// };
///
/// let mut graph = DependencyGraph::new(GraphBudgets::default());
/// let parent_id = ValueId::new(1);
/// let child_id = ValueId::new(2);
///
/// let labels = ValueLabels {
///     integrity: IntegrityLabel::Trusted,
///     confidentiality: DataLabels::new(),
///     authority: AuthoritySet::new(),
/// };
///
/// graph.insert_value(parent_id, labels.clone()).expect("insert failed");
/// graph.insert_value(child_id, labels).expect("insert failed");
///
/// graph.add_dependency(child_id, parent_id).expect("edge failed");
/// assert_eq!(graph.node_count(), 2);
/// ```
pub struct DependencyGraph {
    nodes: HashMap<ValueId, ValueNode>,
    budgets: GraphBudgets,
    truncated: bool,
}

impl DependencyGraph {
    /// Create an empty dependency graph with the given budget limits.
    #[must_use]
    pub fn new(budgets: GraphBudgets) -> Self {
        Self {
            nodes: HashMap::new(),
            budgets,
            truncated: false,
        }
    }

    /// Insert a new value node into the graph.
    ///
    /// Returns `Err(IfcError::DuplicateValueId)` if a node with the given
    /// `id` already exists in the graph. Returns
    /// `Err(IfcError::ValueBudgetExhausted)` and marks the graph as
    /// truncated when the value count would exceed `max_values`.
    pub fn insert_value(&mut self, id: ValueId, labels: ValueLabels) -> Result<(), IfcError> {
        if self.nodes.contains_key(&id) {
            return Err(IfcError::DuplicateValueId(*id.inner()));
        }

        let current = u64::try_from(self.nodes.len()).unwrap_or(u64::MAX);
        if current >= self.budgets.max_values {
            self.truncated = true;
            return Err(IfcError::ValueBudgetExhausted {
                current,
                limit: self.budgets.max_values,
            });
        }

        self.nodes.insert(
            id,
            ValueNode {
                id,
                integrity: labels.integrity,
                confidentiality: labels.confidentiality,
                authority: labels.authority,
                parents: Vec::new(),
            },
        );
        Ok(())
    }

    /// Add a direct dependency edge from `child` to `parent`.
    ///
    /// Both value IDs must already exist in the graph. Self-loops
    /// (`child == parent`) are rejected with `IfcError::CycleDetected`.
    /// Exceeding the per-value parent budget returns
    /// `IfcError::ParentBudgetExhausted`.
    pub fn add_dependency(&mut self, child: ValueId, parent: ValueId) -> Result<(), IfcError> {
        if child == parent {
            return Err(IfcError::CycleDetected {
                from: *child.inner(),
                to: *parent.inner(),
            });
        }

        if !self.nodes.contains_key(&parent) {
            return Err(IfcError::UnknownValueId(*parent.inner()));
        }

        let child_node = self
            .nodes
            .get_mut(&child)
            .ok_or(IfcError::UnknownValueId(*child.inner()))?;

        let current = u64::try_from(child_node.parents.len()).unwrap_or(u64::MAX);
        if current >= self.budgets.max_parents_per_value {
            self.truncated = true;
            return Err(IfcError::ParentBudgetExhausted {
                value_id: *child.inner(),
                current,
                limit: self.budgets.max_parents_per_value,
            });
        }

        child_node.parents.push(parent);
        Ok(())
    }

    /// Look up a value node by its identifier.
    #[must_use]
    pub fn get_node(&self, id: &ValueId) -> Option<&ValueNode> {
        self.nodes.get(id)
    }

    /// Check whether a value identifier exists in the graph.
    #[must_use]
    pub fn contains(&self, id: &ValueId) -> bool {
        self.nodes.contains_key(id)
    }

    /// Return the number of value nodes in the graph.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Return whether the graph has been truncated due to budget overflow.
    #[must_use]
    pub const fn is_truncated(&self) -> bool {
        self.truncated
    }

    /// Return the budget configuration for this graph.
    #[must_use]
    pub const fn budgets(&self) -> &GraphBudgets {
        &self.budgets
    }
}

#[cfg(test)]
#[path = "dependency_graph_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "dependency_graph_proptests.rs"]
mod proptests;
