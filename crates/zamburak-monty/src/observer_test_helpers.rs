//! Shared helpers for `ZamburakObserver` unit tests.

use monty::{
    ExternalCallKind, ExternalCallRequestedEvent, RuntimeObserver, RuntimeObserverEvent,
    RuntimeValueId,
};

use crate::observer::{GovernedIfcConfig, ZamburakObserver};

/// Helper: build a `ZamburakObserver`.
pub(crate) fn allow_all_observer() -> ZamburakObserver {
    ZamburakObserver::new()
}

/// Helper: build a `ZamburakObserver` that has already recorded one pending
/// external-call event with the given `call_id` and `kind`.
pub(crate) fn observer_with_one_pending_call(
    call_id: u64,
    kind: ExternalCallKind,
) -> ZamburakObserver {
    let mut obs = allow_all_observer();
    let arg_ids: Vec<RuntimeValueId> = vec![];
    let kwarg_ids: Vec<(RuntimeValueId, RuntimeValueId)> = vec![];
    obs.on_event(RuntimeObserverEvent::ExternalCallRequested(
        ExternalCallRequestedEvent {
            call_id: u32::try_from(call_id).expect("helper call_id should fit in u32"),
            kind,
            arg_runtime_ids: &arg_ids,
            kwarg_runtime_ids: &kwarg_ids,
        },
    ));
    obs
}

/// Helper: build a strict IFC observer.
pub(crate) fn strict_ifc_observer() -> ZamburakObserver {
    ZamburakObserver::with_ifc_config(GovernedIfcConfig::strict_with_boundary_seeds())
}
