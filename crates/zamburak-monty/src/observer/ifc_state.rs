//! Internal IFC runtime state driven by Track A observer events.
//!
//! This module translates generic `full-monty` runtime observer events into the
//! dependency-graph and control-context state owned by `zamburak-monty`.

use monty::{
    ControlConditionEvent, ExternalCallKind, ExternalCallRequestedEvent, ExternalCallReturnedEvent,
    OpInputIds, OpResultEvent, RuntimeValueId, ValueCreatedEvent,
};
use tracing::warn;
use zamburak_core::control_context::ExecutionContextSummary;
use zamburak_core::propagation::{PropagationMode, propagate_labels};
use zamburak_core::summary::compute_summary;
use zamburak_core::{
    AuthoritySet, DataLabels, DependencyGraph, DependencySummary, GraphBudgets, IfcError,
    IntegrityLabel, ValueId, ValueLabels,
};

use crate::external_call::CallIfcContext;

mod call_ifc_tracker;

use call_ifc_tracker::{CallIfcTracker, RequestedCallIfcState, ReturnedCallIfcState};

/// IFC seed labels for values created inside the governed runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IfcValueSeedConfig {
    /// Labels assigned to ordinary internal values when first observed.
    pub internal_values: ValueLabels,
    /// Labels assigned to values resumed from host external-call returns.
    pub resumed_external_returns: ValueLabels,
}

impl IfcValueSeedConfig {
    /// Returns the canonical boundary seed labels used by governed IFC tests.
    ///
    /// # Examples
    ///
    /// ```
    /// use zamburak_core::IntegrityLabel;
    /// use zamburak_monty::IfcValueSeedConfig;
    ///
    /// let config = IfcValueSeedConfig::boundary_defaults();
    /// assert_eq!(config.internal_values.integrity, IntegrityLabel::Trusted);
    /// assert_eq!(
    ///     config.resumed_external_returns.integrity,
    ///     IntegrityLabel::Untrusted,
    /// );
    /// ```
    #[must_use]
    pub fn boundary_defaults() -> Self {
        Self {
            internal_values: ValueLabels {
                integrity: IntegrityLabel::Trusted,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::full(),
            },
            resumed_external_returns: ValueLabels {
                integrity: IntegrityLabel::Untrusted,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::full(),
            },
        }
    }
}

impl Default for IfcValueSeedConfig {
    fn default() -> Self {
        Self::boundary_defaults()
    }
}

/// Public configuration for observer-driven IFC updates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernedIfcConfig {
    /// Budget limits for the dependency graph.
    pub graph_budgets: GraphBudgets,
    /// Propagation mode for effect summaries.
    pub propagation_mode: PropagationMode,
    /// Seed labels for newly observed values.
    pub value_seeds: IfcValueSeedConfig,
}

impl GovernedIfcConfig {
    /// Returns the shared strict-mode configuration with canonical boundary
    /// seed labels.
    ///
    /// # Examples
    ///
    /// ```
    /// use zamburak_core::propagation::PropagationMode;
    /// use zamburak_monty::GovernedIfcConfig;
    ///
    /// let config = GovernedIfcConfig::strict_with_boundary_seeds();
    /// assert_eq!(config.propagation_mode, PropagationMode::Strict);
    /// ```
    #[must_use]
    pub fn strict_with_boundary_seeds() -> Self {
        Self {
            propagation_mode: PropagationMode::Strict,
            value_seeds: IfcValueSeedConfig::boundary_defaults(),
            ..Self::default()
        }
    }
}

impl Default for GovernedIfcConfig {
    fn default() -> Self {
        Self {
            graph_budgets: GraphBudgets::default(),
            propagation_mode: PropagationMode::Normal,
            value_seeds: IfcValueSeedConfig::default(),
        }
    }
}

/// Observer-owned runtime IFC state updated from Track A events.
pub(crate) struct IfcRuntimeState {
    graph: DependencyGraph,
    propagation_mode: PropagationMode,
    control_context: ExecutionContextSummary,
    calls: CallIfcTracker,
    value_seeds: IfcValueSeedConfig,
    is_conservative: bool,
}

impl IfcRuntimeState {
    /// Create an empty IFC runtime state from the configured budgets and seeds.
    pub(crate) fn new(config: GovernedIfcConfig) -> Self {
        Self {
            graph: DependencyGraph::new(config.graph_budgets),
            propagation_mode: config.propagation_mode,
            control_context: ExecutionContextSummary::new(),
            calls: CallIfcTracker::new(),
            value_seeds: config.value_seeds,
            is_conservative: false,
        }
    }

    /// Record a value-creation event from the VM.
    pub(crate) fn apply_value_created(&mut self, event: ValueCreatedEvent) {
        let Some(value_id) = self.runtime_to_value_id(event.value_id) else {
            return;
        };
        let labels = self.value_seeds.internal_values.clone();
        self.ensure_value(value_id, &labels);
    }

    /// Record an operation-result event from the VM.
    pub(crate) fn apply_op_result(&mut self, event: OpResultEvent) {
        let Some(output_id) = self.runtime_to_value_id(event.output_id) else {
            return;
        };
        if let Some(displaced_candidate) = self.calls.take_returned_for_output(
            event.inputs,
            self.graph.get_node(&output_id).is_some(),
            output_id,
        ) {
            if let Some(displaced_candidate) = displaced_candidate {
                self.materialize_internal_value(displaced_candidate);
            }
            return;
        }

        self.materialize_internal_value(output_id);
        for input_id in self.input_value_ids(event.inputs) {
            if input_id != output_id {
                self.add_dependency(output_id, input_id);
            }
        }
    }

    /// Record a control-condition event using the conservative lifetime model.
    pub(crate) fn apply_control_condition(&mut self, event: ControlConditionEvent) {
        let Some(condition_id) = self.runtime_to_value_id(event.condition_id) else {
            return;
        };
        self.materialize_returned_value_if_needed(condition_id);
        let labels = self.value_seeds.internal_values.clone();
        self.ensure_value(condition_id, &labels);
        let summary = self
            .summary_for(condition_id)
            .unwrap_or_else(DependencySummary::unknown_top);
        self.control_context.push_condition(condition_id, &summary);
    }

    /// Snapshot IFC state at an external-call request boundary.
    pub(crate) fn apply_external_call_requested(&mut self, event: ExternalCallRequestedEvent<'_>) {
        let (arg_value_ids, arg_summaries) = self.collect_arg_operands(event.arg_runtime_ids);
        let (kwarg_value_ids, kwarg_summaries) =
            self.collect_kwarg_operands(event.kwarg_runtime_ids);

        let aggregate_operands = aggregate_operands(&arg_summaries, &kwarg_summaries);
        let aggregate_summary = propagate_labels(
            self.propagation_mode,
            &aggregate_operands,
            &self.control_context,
        )
        .unwrap_or_else(empty_summary);

        self.calls.record_requested(RequestedCallIfcState {
            call_id: event.call_id,
            kind: event.kind,
            arg_value_ids,
            kwarg_value_ids,
            ifc: CallIfcContext {
                propagation_mode: self.propagation_mode,
                aggregate_summary,
                control_context: self.control_context.clone(),
                arg_summaries,
                kwarg_summaries,
            },
        });
    }

    /// Reconcile a completed external-call yield.
    pub(crate) fn apply_external_call_returned(&mut self, event: ExternalCallReturnedEvent) {
        if self.calls.record_returned(event).is_none() {
            self.enter_conservative_mode();
        }
    }

    /// Return a call snapshot for mediation while recording the effect boundary.
    pub(crate) fn call_ifc_context(
        &mut self,
        call_id: u32,
        kind: ExternalCallKind,
        function_name: &str,
    ) -> Option<CallIfcContext> {
        let mut call_ifc = self.calls.call_ifc_context(call_id, kind)?;
        self.record_effect_for_call(function_name, &mut call_ifc);
        Some(call_ifc)
    }
}

fn empty_summary() -> DependencySummary {
    DependencySummary {
        integrity_join: IntegrityLabel::Trusted,
        confidentiality_join: DataLabels::new(),
        authority_join: AuthoritySet::full(),
        origin_count: 0,
        truncated: false,
    }
}

fn aggregate_operands(
    arg_summaries: &[DependencySummary],
    kwarg_summaries: &[(DependencySummary, DependencySummary)],
) -> Vec<DependencySummary> {
    arg_summaries
        .iter()
        .cloned()
        .chain(
            kwarg_summaries
                .iter()
                .flat_map(|(key_summary, value_summary)| {
                    [key_summary.clone(), value_summary.clone()]
                }),
        )
        .collect()
}

fn join_labels(seed: &ValueLabels, summary: &DependencySummary) -> ValueLabels {
    ValueLabels {
        integrity: seed.integrity.join(summary.integrity_join),
        confidentiality: seed.confidentiality.join(&summary.confidentiality_join),
        authority: seed.authority.join(&summary.authority_join),
    }
}

type KwargOperandIds = Vec<(ValueId, ValueId)>;
type KwargOperandSummaries = Vec<(DependencySummary, DependencySummary)>;

impl IfcRuntimeState {
    fn input_value_ids(&mut self, inputs: OpInputIds) -> Vec<ValueId> {
        match inputs {
            OpInputIds::None => Vec::new(),
            OpInputIds::One(value_id) => self
                .runtime_to_value_id(value_id)
                .into_iter()
                .inspect(|value_id| self.materialize_returned_value_if_needed(*value_id))
                .collect(),
            OpInputIds::Two(lhs, rhs) => [lhs, rhs]
                .into_iter()
                .filter_map(|value_id| {
                    self.runtime_to_value_id(value_id).inspect(|value_id| {
                        self.materialize_returned_value_if_needed(*value_id);
                    })
                })
                .collect(),
        }
    }

    fn collect_arg_operands(
        &mut self,
        runtime_ids: &[RuntimeValueId],
    ) -> (Vec<ValueId>, Vec<DependencySummary>) {
        let mut value_ids = Vec::new();
        let mut summaries = Vec::new();
        for runtime_value_id in runtime_ids.iter().copied() {
            let (value_id, summary) = self.runtime_operand_summary(runtime_value_id);
            if let Some(value_id) = value_id {
                value_ids.push(value_id);
            }
            summaries.push(summary);
        }
        (value_ids, summaries)
    }

    fn collect_kwarg_operands(
        &mut self,
        runtime_ids: &[(RuntimeValueId, RuntimeValueId)],
    ) -> (KwargOperandIds, KwargOperandSummaries) {
        let mut value_ids = Vec::new();
        let mut summaries = Vec::new();
        for (key_runtime_id, value_runtime_id) in runtime_ids.iter().copied() {
            let (key_id, key_summary) = self.runtime_operand_summary(key_runtime_id);
            let (value_id, value_summary) = self.runtime_operand_summary(value_runtime_id);
            if let (Some(key_id), Some(value_id)) = (key_id, value_id) {
                value_ids.push((key_id, value_id));
            }
            summaries.push((key_summary, value_summary));
        }
        (value_ids, summaries)
    }

    fn runtime_operand_summary(
        &mut self,
        runtime_value_id: RuntimeValueId,
    ) -> (Option<ValueId>, DependencySummary) {
        match self.runtime_to_value_id(runtime_value_id) {
            Some(value_id) => (Some(value_id), self.summary_for_seeded(value_id)),
            None => (None, DependencySummary::unknown_top()),
        }
    }

    fn runtime_to_value_id(&mut self, value_id: RuntimeValueId) -> Option<ValueId> {
        let Ok(raw_value_id) = u64::try_from(value_id.raw()) else {
            warn!(
                raw_value_id = ?value_id.raw(),
                "runtime value ID exceeded u64; switching to conservative mode"
            );
            self.enter_conservative_mode();
            return None;
        };
        Some(ValueId::new(raw_value_id))
    }

    fn ensure_value(&mut self, value_id: ValueId, labels: &ValueLabels) {
        if self.graph.get_node(&value_id).is_some() {
            return;
        }

        if let Err(error) = self.graph.insert_value(value_id, labels.clone()) {
            self.record_ifc_error(error);
        }
    }

    fn add_dependency(&mut self, child: ValueId, parent: ValueId) {
        if let Err(error) = self.graph.add_dependency(child, parent)
            && !matches!(error, IfcError::DuplicateEdge { .. })
        {
            self.record_ifc_error(error);
        }
    }

    fn summary_for_seeded(&mut self, value_id: ValueId) -> DependencySummary {
        self.materialize_returned_value_if_needed(value_id);
        self.materialize_internal_value(value_id);
        self.summary_for(value_id)
            .unwrap_or_else(DependencySummary::unknown_top)
    }

    fn materialize_returned_value_if_needed(&mut self, value_id: ValueId) {
        let Some(returned_call) = self.calls.take_returned_for_value(value_id) else {
            return;
        };

        let output_labels = join_labels(
            &self.value_seeds.resumed_external_returns,
            &returned_call.aggregate_summary,
        );
        self.ensure_value(value_id, &output_labels);
        self.add_call_dependencies(value_id, &returned_call);
    }

    fn materialize_internal_value(&mut self, value_id: ValueId) {
        let labels = self.value_seeds.internal_values.clone();
        self.ensure_value(value_id, &labels);
    }

    fn summary_for(&mut self, value_id: ValueId) -> Option<DependencySummary> {
        match compute_summary(&self.graph, &value_id, self.graph.budgets()) {
            Ok(summary) if self.is_conservative => {
                Some(DependencySummary::unknown_top().join(&summary))
            }
            Ok(summary) => Some(summary),
            Err(IfcError::UnknownValueId(_)) => None,
            Err(error) => {
                self.record_ifc_error(error);
                Some(DependencySummary::unknown_top())
            }
        }
    }

    fn add_call_dependencies(&mut self, output_id: ValueId, returned_call: &ReturnedCallIfcState) {
        for input_id in returned_call.arg_value_ids.iter().copied().chain(
            returned_call
                .kwarg_value_ids
                .iter()
                .flat_map(|(key_id, value_id)| [*key_id, *value_id]),
        ) {
            self.materialize_internal_value(input_id);
            if input_id != output_id {
                self.add_dependency(output_id, input_id);
            }
        }
    }

    fn enter_conservative_mode(&mut self) {
        self.is_conservative = true;
    }

    fn record_effect_for_call(&mut self, function_name: &str, call_ifc: &mut CallIfcContext) {
        self.control_context.record_effect(function_name);
        call_ifc.control_context.record_effect(function_name);
    }

    fn record_ifc_error(&mut self, error: IfcError) {
        warn!(
            ?error,
            "IFC error encountered; switching to conservative mode"
        );
        self.enter_conservative_mode();
    }
}
