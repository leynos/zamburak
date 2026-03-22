//! Control-context summary for strict-mode effect evaluation.
//!
//! An `ExecutionContextSummary` tracks the integrity and confidentiality
//! labels accumulated from control-flow conditions (e.g. `if`/`while`
//! guards). In strict mode, this summary is joined into every effectful
//! call's dependency summary so that policy decisions account for
//! implicit information flows through control dependence.

use std::collections::HashMap;

use crate::summary::DependencySummary;
use crate::trust::{AuthoritySet, DataLabels, IntegrityLabel};
use crate::value_id::ValueId;

/// Effect invocation counters for audit and rate limiting.
///
/// Tracks the total number of effectful calls made under a given
/// control context and per-tool breakdowns.
///
/// # Examples
///
/// ```
/// use zamburak_core::control_context::EffectCounters;
///
/// let mut counters = EffectCounters::new();
/// counters.record("file_write");
/// assert_eq!(counters.total_effects(), 1);
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EffectCounters {
    /// Total number of effectful calls recorded.
    total_effects: u64,
    /// Per-tool effect counts.
    effects_by_tool: HashMap<String, u64>,
}

impl EffectCounters {
    /// Create a zero-initialised counter set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an effect invocation for the named tool.
    pub fn record(&mut self, tool: &str) {
        self.total_effects = self.total_effects.saturating_add(1);
        let count = self.effects_by_tool.entry(tool.to_owned()).or_insert(0);
        *count = count.saturating_add(1);
    }

    /// Return the total number of effects recorded.
    #[must_use]
    pub fn total_effects(&self) -> u64 {
        self.total_effects
    }

    /// Return the number of effects recorded for a specific tool.
    #[must_use]
    pub fn tool_effects(&self, tool: &str) -> u64 {
        self.effects_by_tool.get(tool).copied().unwrap_or(0)
    }
}

/// Accumulated control-context state for strict-mode evaluation.
///
/// Tracks the program-counter (PC) integrity and confidentiality
/// labels from control-flow conditions, the set of control
/// dependencies (condition `ValueId`s), and effect counters.
///
/// # Examples
///
/// ```
/// use zamburak_core::control_context::ExecutionContextSummary;
/// use zamburak_core::trust::IntegrityLabel;
///
/// let ctx = ExecutionContextSummary::new();
/// assert_eq!(ctx.pc_integrity(), IntegrityLabel::Verified);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionContextSummary {
    /// Program-counter integrity label (accumulated from conditions).
    pc_integrity: IntegrityLabel,
    /// Program-counter confidentiality labels (accumulated from conditions).
    pc_confidentiality: DataLabels,
    /// Value IDs of control-flow conditions contributing to this context.
    control_dependencies: Vec<ValueId>,
    /// Effect invocation counters.
    effect_counters: EffectCounters,
}

impl ExecutionContextSummary {
    /// Create a fresh execution context with verified integrity,
    /// no confidentiality labels, no control dependencies, and zero
    /// effect counters.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pc_integrity: IntegrityLabel::Verified,
            pc_confidentiality: DataLabels::new(),
            control_dependencies: Vec::new(),
            effect_counters: EffectCounters::new(),
        }
    }

    /// Push a condition's dependency summary into this context.
    ///
    /// Joins the condition's integrity and confidentiality labels into
    /// the PC labels and records the condition's value ID as a control
    /// dependency.
    pub fn push_condition(&mut self, condition_id: ValueId, condition_summary: &DependencySummary) {
        self.pc_integrity = self.pc_integrity.join(condition_summary.integrity_join);
        self.pc_confidentiality = self
            .pc_confidentiality
            .join(&condition_summary.confidentiality_join);
        self.control_dependencies.push(condition_id);
    }

    /// Convert this context into a `DependencySummary` for strict-mode
    /// joins.
    ///
    /// The resulting summary has:
    /// - integrity from the PC label,
    /// - confidentiality from the PC label,
    /// - empty authority (control context does not grant capabilities),
    /// - origin count from the number of control dependencies,
    /// - not truncated.
    #[must_use]
    pub fn as_summary(&self) -> DependencySummary {
        let origin_count = u32::try_from(self.control_dependencies.len()).unwrap_or(u32::MAX);
        DependencySummary {
            integrity_join: self.pc_integrity,
            confidentiality_join: self.pc_confidentiality.clone(),
            authority_join: AuthoritySet::new(),
            origin_count,
            truncated: false,
        }
    }

    /// Record an effectful call for the named tool.
    pub fn record_effect(&mut self, tool: &str) {
        self.effect_counters.record(tool);
    }

    /// Return the current PC integrity label.
    #[must_use]
    pub fn pc_integrity(&self) -> IntegrityLabel {
        self.pc_integrity
    }

    /// Return the current PC confidentiality labels.
    #[must_use]
    pub fn pc_confidentiality(&self) -> &DataLabels {
        &self.pc_confidentiality
    }

    /// Return the control dependency value IDs.
    #[must_use]
    pub fn control_dependencies(&self) -> &[ValueId] {
        &self.control_dependencies
    }

    /// Return the effect counters.
    #[must_use]
    pub fn effect_counters(&self) -> &EffectCounters {
        &self.effect_counters
    }
}

impl Default for ExecutionContextSummary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "control_context_tests.rs"]
mod tests;
