//! Call-lifecycle tracking for observer-driven IFC snapshots.

use std::collections::{BTreeMap, VecDeque};
use std::mem;

use monty::{ExternalCallKind, ExternalCallReturnKind, ExternalCallReturnedEvent, OpInputIds};
use zamburak_core::{DependencySummary, ValueId};

use crate::external_call::CallIfcContext;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PendingCallIfcState {
    kind: ExternalCallKind,
    arg_value_ids: Vec<ValueId>,
    kwarg_value_ids: Vec<(ValueId, ValueId)>,
    ifc: CallIfcContext,
}

pub(super) struct RequestedCallIfcState {
    pub(super) call_id: u32,
    pub(super) kind: ExternalCallKind,
    pub(super) arg_value_ids: Vec<ValueId>,
    pub(super) kwarg_value_ids: Vec<(ValueId, ValueId)>,
    pub(super) ifc: CallIfcContext,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ReturnedCallIfcState {
    pub(super) arg_value_ids: Vec<ValueId>,
    pub(super) kwarg_value_ids: Vec<(ValueId, ValueId)>,
    pub(super) aggregate_summary: DependencySummary,
}

/// Call-specific IFC snapshots and returned-call provenance.
#[derive(Default)]
pub(super) struct CallIfcTracker {
    pending_calls: BTreeMap<u32, PendingCallIfcState>,
    returned_calls: VecDeque<ReturnedCallIfcState>,
    candidate_output_id: Option<ValueId>,
}

impl CallIfcTracker {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn record_requested(&mut self, requested: RequestedCallIfcState) {
        self.pending_calls.insert(
            requested.call_id,
            PendingCallIfcState {
                kind: requested.kind,
                arg_value_ids: requested.arg_value_ids,
                kwarg_value_ids: requested.kwarg_value_ids,
                ifc: requested.ifc,
            },
        );
    }

    /// Converts an [`ExternalCallReturnedEvent`] into queued returned-call
    /// provenance only for [`ExternalCallReturnKind::Return`]. Error and future
    /// completions are intentionally ignored because only successful returns
    /// produce tracked return values, although the matching
    /// [`PendingCallIfcState`] is still removed and returned to the caller.
    pub(super) fn record_returned(
        &mut self,
        event: ExternalCallReturnedEvent,
    ) -> Option<PendingCallIfcState> {
        let mut pending_call = self.pending_calls.remove(&event.call_id)?;
        if matches!(event.kind, ExternalCallReturnKind::Return) {
            self.returned_calls.push_back(ReturnedCallIfcState {
                arg_value_ids: mem::take(&mut pending_call.arg_value_ids),
                kwarg_value_ids: mem::take(&mut pending_call.kwarg_value_ids),
                aggregate_summary: pending_call.ifc.aggregate_summary.clone(),
            });
        }
        Some(pending_call)
    }

    /// Returns `true` when the conditions required to gate an `OpResult` as a
    /// resumed external-call return are not met, and the output should be
    /// treated as a plain internal value.
    fn returned_call_gating_unavailable(
        &self,
        inputs: OpInputIds,
        output_was_observed: bool,
    ) -> bool {
        !matches!(inputs, OpInputIds::None) || output_was_observed || self.returned_calls.is_empty()
    }

    /// `take_returned_for_output` returns `None` when `inputs !=
    /// OpInputIds::None`, when `output_was_observed` is true, or when
    /// `returned_calls` is empty. Otherwise it updates `candidate_output_id`
    /// with `output_id` and returns `Some(None)` if nothing was displaced or
    /// `Some(Some(displaced_value))` when a previous candidate was replaced.
    pub(super) fn take_returned_for_output(
        &mut self,
        inputs: OpInputIds,
        output_was_observed: bool,
        output_id: ValueId,
    ) -> Option<Option<ValueId>> {
        if self.returned_call_gating_unavailable(inputs, output_was_observed) {
            return None;
        }
        Some(self.candidate_output_id.replace(output_id))
    }

    /// `take_returned_for_value` only succeeds when `candidate_output_id ==
    /// Some(value_id)`. In that case it clears `candidate_output_id` and
    /// consumes the next returned call from `returned_calls`.
    pub(super) fn take_returned_for_value(
        &mut self,
        value_id: ValueId,
    ) -> Option<ReturnedCallIfcState> {
        if self.candidate_output_id != Some(value_id) {
            return None;
        }

        self.candidate_output_id = None;
        self.returned_calls.pop_front()
    }

    pub(super) fn call_ifc_context(
        &self,
        call_id: u32,
        kind: ExternalCallKind,
    ) -> Option<CallIfcContext> {
        let pending_call = self.pending_calls.get(&call_id)?;
        (pending_call.kind == kind).then(|| pending_call.ifc.clone())
    }
}
