//! Internal IFC runtime state driven by Track A observer events.
//!
//! This module translates generic `full-monty` runtime observer events into the
//! dependency-graph and control-context state owned by `zamburak-monty`.

use std::collections::{BTreeMap, VecDeque};

use monty::{
    ControlConditionEvent, ExternalCallKind, ExternalCallRequestedEvent, ExternalCallReturnKind,
    ExternalCallReturnedEvent, OpInputIds, OpResultEvent, RuntimeValueId, ValueCreatedEvent,
};
use zamburak_core::control_context::ExecutionContextSummary;
use zamburak_core::propagation::{PropagationMode, propagate_labels};
use zamburak_core::summary::compute_summary;
use zamburak_core::{
    AuthoritySet, DataLabels, DependencyGraph, DependencySummary, GraphBudgets, IfcError,
    IntegrityLabel, ValueId, ValueLabels,
};

use crate::external_call::CallIfcContext;

/// IFC seed labels for values created inside the governed runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IfcValueSeedConfig {
    /// Labels assigned to ordinary internal values when first observed.
    pub internal_values: ValueLabels,
    /// Labels assigned to values resumed from host external-call returns.
    pub resumed_external_returns: ValueLabels,
}

impl Default for IfcValueSeedConfig {
    fn default() -> Self {
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

impl Default for GovernedIfcConfig {
    fn default() -> Self {
        Self {
            graph_budgets: GraphBudgets::default(),
            propagation_mode: PropagationMode::Normal,
            value_seeds: IfcValueSeedConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PendingCallIfcState {
    kind: ExternalCallKind,
    arg_value_ids: Vec<ValueId>,
    kwarg_value_ids: Vec<(ValueId, ValueId)>,
    ifc: CallIfcContext,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReturnedCallIfcState {
    arg_value_ids: Vec<ValueId>,
    kwarg_value_ids: Vec<(ValueId, ValueId)>,
    aggregate_summary: DependencySummary,
}

/// Observer-owned runtime IFC state updated from Track A events.
pub(crate) struct IfcRuntimeState {
    graph: DependencyGraph,
    propagation_mode: PropagationMode,
    control_context: ExecutionContextSummary,
    pending_calls: BTreeMap<u32, PendingCallIfcState>,
    returned_calls: VecDeque<ReturnedCallIfcState>,
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
            pending_calls: BTreeMap::new(),
            returned_calls: VecDeque::new(),
            value_seeds: config.value_seeds,
            is_conservative: false,
        }
    }

    /// Record a value-creation event from the VM.
    pub(crate) fn apply_value_created(&mut self, event: ValueCreatedEvent) {
        let value_id = runtime_to_value_id(event.value_id);
        let labels = self.value_seeds.internal_values.clone();
        self.ensure_value(value_id, &labels);
    }

    /// Record an operation-result event from the VM.
    pub(crate) fn apply_op_result(&mut self, event: OpResultEvent) {
        let output_id = runtime_to_value_id(event.output_id);
        if let Some(returned_call) = self.take_returned_call_for_output(event.inputs) {
            let output_labels = join_labels(
                &self.value_seeds.resumed_external_returns,
                &returned_call.aggregate_summary,
            );
            self.ensure_value(output_id, &output_labels);
            self.add_call_dependencies(output_id, &returned_call);
            return;
        }

        let internal_labels = self.value_seeds.internal_values.clone();
        self.ensure_value(output_id, &internal_labels);
        for input_id in input_value_ids(event.inputs) {
            self.ensure_value(input_id, &internal_labels);
            if input_id != output_id {
                self.add_dependency(output_id, input_id);
            }
        }
    }

    /// Record a control-condition event using the conservative lifetime model.
    pub(crate) fn apply_control_condition(&mut self, event: ControlConditionEvent) {
        let condition_id = runtime_to_value_id(event.condition_id);
        let labels = self.value_seeds.internal_values.clone();
        self.ensure_value(condition_id, &labels);
        let summary = self
            .summary_for(condition_id)
            .unwrap_or_else(DependencySummary::unknown_top);
        self.control_context.push_condition(condition_id, &summary);
    }

    /// Snapshot IFC state at an external-call request boundary.
    pub(crate) fn apply_external_call_requested(&mut self, event: ExternalCallRequestedEvent<'_>) {
        let arg_value_ids: Vec<ValueId> = event
            .arg_runtime_ids
            .iter()
            .copied()
            .map(runtime_to_value_id)
            .collect();
        let kwarg_value_ids: Vec<(ValueId, ValueId)> = event
            .kwarg_runtime_ids
            .iter()
            .copied()
            .map(|(key_id, value_id)| (runtime_to_value_id(key_id), runtime_to_value_id(value_id)))
            .collect();

        let arg_summaries = arg_value_ids
            .iter()
            .copied()
            .map(|value_id| self.summary_for_seeded(value_id))
            .collect::<Vec<_>>();
        let kwarg_summaries = kwarg_value_ids
            .iter()
            .copied()
            .map(|(key_id, value_id)| {
                (
                    self.summary_for_seeded(key_id),
                    self.summary_for_seeded(value_id),
                )
            })
            .collect::<Vec<_>>();

        let aggregate_operands = aggregate_operands(&arg_summaries, &kwarg_summaries);
        let aggregate_summary = propagate_labels(
            self.propagation_mode,
            &aggregate_operands,
            &self.control_context,
        )
        .unwrap_or_else(empty_summary);

        self.pending_calls.insert(
            event.call_id,
            PendingCallIfcState {
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
            },
        );
    }

    /// Reconcile a completed external-call yield.
    pub(crate) fn apply_external_call_returned(&mut self, event: ExternalCallReturnedEvent) {
        let Some(pending_call) = self.pending_calls.remove(&event.call_id) else {
            self.is_conservative = true;
            return;
        };

        if matches!(event.kind, ExternalCallReturnKind::Return) {
            self.returned_calls.push_back(ReturnedCallIfcState {
                arg_value_ids: pending_call.arg_value_ids,
                kwarg_value_ids: pending_call.kwarg_value_ids,
                aggregate_summary: pending_call.ifc.aggregate_summary,
            });
        }
    }

    /// Return a call snapshot for mediation while recording the effect boundary.
    pub(crate) fn call_ifc_context(
        &mut self,
        call_id: u32,
        kind: ExternalCallKind,
        function_name: &str,
    ) -> Option<CallIfcContext> {
        let pending_call = self.pending_calls.get(&call_id)?;
        if pending_call.kind != kind {
            self.is_conservative = true;
            return None;
        }

        self.control_context.record_effect(function_name);
        let mut call_ifc = pending_call.ifc.clone();
        call_ifc.control_context.record_effect(function_name);
        Some(call_ifc)
    }
}

fn runtime_to_value_id(value_id: RuntimeValueId) -> ValueId {
    ValueId::new(u64::try_from(value_id.raw()).unwrap_or(u64::MAX))
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

fn input_value_ids(inputs: OpInputIds) -> Vec<ValueId> {
    match inputs {
        OpInputIds::None => Vec::new(),
        OpInputIds::One(value_id) => vec![runtime_to_value_id(value_id)],
        OpInputIds::Two(lhs, rhs) => vec![runtime_to_value_id(lhs), runtime_to_value_id(rhs)],
    }
}

impl IfcRuntimeState {
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
        let labels = self.value_seeds.internal_values.clone();
        self.ensure_value(value_id, &labels);
        self.summary_for(value_id)
            .unwrap_or_else(DependencySummary::unknown_top)
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

    fn take_returned_call_for_output(
        &mut self,
        inputs: OpInputIds,
    ) -> Option<ReturnedCallIfcState> {
        if !matches!(inputs, OpInputIds::None) {
            return None;
        }
        self.returned_calls.pop_front()
    }

    fn add_call_dependencies(&mut self, output_id: ValueId, returned_call: &ReturnedCallIfcState) {
        for input_id in returned_call.arg_value_ids.iter().copied().chain(
            returned_call
                .kwarg_value_ids
                .iter()
                .flat_map(|(key_id, value_id)| [*key_id, *value_id]),
        ) {
            let labels = self.value_seeds.internal_values.clone();
            self.ensure_value(input_id, &labels);
            if input_id != output_id {
                self.add_dependency(output_id, input_id);
            }
        }
    }

    fn record_ifc_error(&mut self, error: IfcError) {
        let _ = error;
        self.is_conservative = true;
    }
}
