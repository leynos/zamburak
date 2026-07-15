//! IFC-specific error types for dependency graph operations.
//!
//! Identifier-bearing variants use `ValueId`, while count and limit
//! fields remain primitive to keep the public error type compact.

use thiserror::Error;

use crate::value_id::ValueId;

/// Errors arising from IFC dependency graph operations.
///
/// Budget-enforcement errors indicate that the graph has reached a
/// policy-defined limit. The caller should fall back to conservative
/// unknown-top summaries when these occur.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum IfcError {
    /// The maximum number of tracked values has been reached.
    #[error("value budget exhausted: {current} values tracked, limit is {limit}")]
    ValueBudgetExhausted {
        /// Current number of values in the graph.
        current: u64,
        /// Maximum allowed by the budget.
        limit: u64,
    },

    /// The maximum number of parents for a single value has been reached.
    #[error(
        "parent budget exhausted for value {}: \
         {current} parents, limit is {limit}",
        value_id.inner()
    )]
    ParentBudgetExhausted {
        /// Value whose parent budget is exhausted.
        value_id: ValueId,
        /// Current number of parents for this value.
        current: u64,
        /// Maximum parents allowed by the budget.
        limit: u64,
    },

    /// The closure-step budget has been exhausted during ancestry traversal.
    ///
    /// This error is returned from `DependencyGraph::add_dependency` when the
    /// BFS reachability check (used for cycle detection) exceeds
    /// `max_closure_steps`. The `steps` field is the number of BFS steps taken
    /// before exhaustion, and `limit` is the budget's `max_closure_steps` value.
    ///
    /// Note: `summary::compute_summary` does NOT return this error; instead it
    /// returns `Ok(DependencySummary::unknown_top())` on budget exhaustion as
    /// part of its fail-closed conservative behaviour.
    #[error(
        "closure step budget exhausted: \
         {steps} steps taken, limit is {limit}"
    )]
    ClosureStepBudgetExhausted {
        /// Number of steps taken before exhaustion.
        steps: u64,
        /// Maximum allowed by the budget.
        limit: u64,
    },

    /// A referenced value identifier is not present in the graph.
    #[error("unknown value ID: {}", _0.inner())]
    UnknownValueId(ValueId),

    /// Attempted to insert a value with an identifier that already exists.
    #[error("duplicate value ID: {} already exists in the graph", _0.inner())]
    DuplicateValueId(ValueId),

    /// Adding the requested edge would create a cycle (self-loop or
    /// transitive back-edge).
    #[error(
        "cycle detected: adding edge from {} to {} \
         would create a cycle",
        from.inner(),
        to.inner()
    )]
    CycleDetected {
        /// Source (child) value identifier.
        from: ValueId,
        /// Target (parent) value identifier.
        to: ValueId,
    },

    /// The requested parent edge already exists for this child.
    #[error(
        "duplicate edge: parent {} already exists for child {}",
        parent.inner(),
        child.inner()
    )]
    DuplicateEdge {
        /// Child value identifier.
        child: ValueId,
        /// Parent value identifier that is already present.
        parent: ValueId,
    },
}

#[cfg(test)]
#[path = "ifc_errors_tests.rs"]
mod tests;
