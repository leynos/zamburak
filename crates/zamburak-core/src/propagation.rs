//! Propagation mode types and label propagation rules.
//!
//! `PropagationMode` selects whether only direct data dependencies
//! (`Normal`) or both data and control-context dependencies (`Strict`)
//! are included when computing the effective dependency summary for a
//! derived value or effect check.

use crate::control_context::ExecutionContextSummary;
use crate::summary::DependencySummary;

/// Mode controlling which dependency sources are included in label
/// propagation.
///
/// # Examples
///
/// ```
/// use zamburak_core::propagation::PropagationMode;
///
/// let mode = PropagationMode::Normal;
/// assert!(!matches!(mode, PropagationMode::Strict));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropagationMode {
    /// Propagate direct data dependencies only.
    Normal,
    /// Propagate direct data dependencies plus the active
    /// control-context summary.
    Strict,
}

/// Compute the effective dependency summary for a set of operand
/// summaries under the given propagation mode.
///
/// In `Normal` mode, the result is the join of all operand summaries.
/// In `Strict` mode, the control-context summary is additionally
/// joined into the result.
///
/// Returns `None` when `operands` is empty and mode is `Normal`
/// (there is nothing to propagate). In `Strict` mode with empty
/// operands, the control-context summary is returned as a base.
///
/// # Examples
///
/// ```
/// use zamburak_core::{
///     DependencySummary, IntegrityLabel, DataLabels, AuthoritySet,
/// };
/// use zamburak_core::propagation::{PropagationMode, propagate_labels};
/// use zamburak_core::control_context::ExecutionContextSummary;
///
/// let operand = DependencySummary {
///     integrity_join: IntegrityLabel::Trusted,
///     confidentiality_join: DataLabels::new(),
///     authority_join: AuthoritySet::new(),
///     origin_count: 1,
///     truncated: false,
/// };
///
/// let ctx = ExecutionContextSummary::new();
/// let result = propagate_labels(
///     PropagationMode::Normal, &[operand], &ctx,
/// );
/// assert!(result.is_some());
/// ```
#[must_use]
pub fn propagate_labels(
    mode: PropagationMode,
    operands: &[DependencySummary],
    context: &ExecutionContextSummary,
) -> Option<DependencySummary> {
    let data_summary = operands.iter().cloned().reduce(|acc, op| acc.join(&op));

    match mode {
        PropagationMode::Normal => data_summary,
        PropagationMode::Strict => {
            let ctx_summary = context.as_summary();
            Some(match data_summary {
                Some(ds) => ds.join(&ctx_summary),
                None => ctx_summary,
            })
        }
    }
}

#[cfg(test)]
#[path = "propagation_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "propagation_proptests.rs"]
mod proptests;
