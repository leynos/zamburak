//! Helper functions for observer-driven IFC state updates.

use monty::RuntimeValueId;
use zamburak_core::ValueId;
use zamburak_core::{DependencySummary, ValueLabels};

pub(super) type KwargOperandIds = Vec<(ValueId, ValueId)>;
pub(super) type KwargOperandSummaries = Vec<(DependencySummary, DependencySummary)>;

pub(super) fn aggregate_operands(
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

pub(super) fn join_labels(seed: &ValueLabels, summary: &DependencySummary) -> ValueLabels {
    ValueLabels {
        integrity: seed.integrity.join(summary.integrity_join),
        confidentiality: seed.confidentiality.join(&summary.confidentiality_join),
        authority: seed.authority.join(&summary.authority_join),
    }
}

pub(super) fn collect_arg_operands<F>(
    runtime_ids: &[RuntimeValueId],
    mut summarize: F,
) -> (Vec<ValueId>, Vec<DependencySummary>)
where
    F: FnMut(RuntimeValueId) -> (Option<ValueId>, DependencySummary),
{
    let mut value_ids = Vec::new();
    let mut summaries = Vec::new();
    for runtime_value_id in runtime_ids.iter().copied() {
        let (value_id, summary) = summarize(runtime_value_id);
        if let Some(value_id) = value_id {
            value_ids.push(value_id);
        }
        summaries.push(summary);
    }
    (value_ids, summaries)
}

pub(super) fn collect_kwarg_operands<F>(
    runtime_ids: &[(RuntimeValueId, RuntimeValueId)],
    mut summarize: F,
) -> (KwargOperandIds, KwargOperandSummaries)
where
    F: FnMut(RuntimeValueId) -> (Option<ValueId>, DependencySummary),
{
    let mut value_ids = Vec::new();
    let mut summaries = Vec::new();
    for (key_runtime_id, value_runtime_id) in runtime_ids.iter().copied() {
        let (key_id, key_summary) = summarize(key_runtime_id);
        let (value_id, value_summary) = summarize(value_runtime_id);
        if let (Some(key_id), Some(value_id)) = (key_id, value_id) {
            value_ids.push((key_id, value_id));
        }
        summaries.push((key_summary, value_summary));
    }
    (value_ids, summaries)
}
