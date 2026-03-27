//! Call-lifecycle tracking for observer-driven IFC snapshots.

use std::collections::{BTreeMap, VecDeque};

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
pub(super) struct CallIfcTracker {
    pending_calls: BTreeMap<u32, PendingCallIfcState>,
    returned_calls: VecDeque<ReturnedCallIfcState>,
}

impl CallIfcTracker {
    pub(super) fn new() -> Self {
        Self {
            pending_calls: BTreeMap::new(),
            returned_calls: VecDeque::new(),
        }
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

    pub(super) fn record_returned(
        &mut self,
        event: ExternalCallReturnedEvent,
    ) -> Option<PendingCallIfcState> {
        let pending_call = self.pending_calls.remove(&event.call_id)?;
        if matches!(event.kind, ExternalCallReturnKind::Return) {
            self.returned_calls.push_back(ReturnedCallIfcState {
                arg_value_ids: pending_call.arg_value_ids.clone(),
                kwarg_value_ids: pending_call.kwarg_value_ids.clone(),
                aggregate_summary: pending_call.ifc.aggregate_summary.clone(),
            });
        }
        Some(pending_call)
    }

    pub(super) fn take_returned_for_output(
        &mut self,
        inputs: OpInputIds,
    ) -> Option<ReturnedCallIfcState> {
        if !matches!(inputs, OpInputIds::None) {
            return None;
        }
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
