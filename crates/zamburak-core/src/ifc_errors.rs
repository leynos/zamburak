//! IFC-specific error types for dependency graph operations.
//!
//! All error variants use primitive fields to stay under the
//! `result_large_err` Clippy threshold.

use thiserror::Error;

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
        "parent budget exhausted for value {value_id}: \
         {current} parents, limit is {limit}"
    )]
    ParentBudgetExhausted {
        /// Value whose parent budget is exhausted.
        value_id: u64,
        /// Current number of parents for this value.
        current: u64,
        /// Maximum parents allowed by the budget.
        limit: u64,
    },

    /// The closure-step budget has been exhausted during summary traversal.
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
    #[error("unknown value ID: {0}")]
    UnknownValueId(u64),

    /// Attempted to insert a value with an identifier that already exists.
    #[error("duplicate value ID: {0} already exists in the graph")]
    DuplicateValueId(u64),

    /// Adding the requested edge would create a cycle (self-loop or
    /// transitive back-edge).
    #[error(
        "cycle detected: adding edge from {from} to {to} \
         would create a cycle"
    )]
    CycleDetected {
        /// Source (child) value identifier.
        from: u64,
        /// Target (parent) value identifier.
        to: u64,
    },

    /// The requested parent edge already exists for this child.
    #[error("duplicate edge: parent {parent} already exists for child {child}")]
    DuplicateEdge {
        /// Child value identifier.
        child: u64,
        /// Parent value identifier that is already present.
        parent: u64,
    },
}

#[cfg(test)]
#[path = "ifc_errors_tests.rs"]
mod tests;
