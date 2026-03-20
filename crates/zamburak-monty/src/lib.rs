//! Governed execution adapter bridging the `full-monty` interpreter substrate
//! into Zamburak governance semantics.
//!
//! This crate provides a governed execution path around the vendored `full-monty`
//! interpreter. A [`GovernedRunner`] wraps a compiled [`monty::MontyRun`] with a
//! Zamburak [`ZamburakObserver`] and mediates every external-function call through
//! a deterministic [`ExternalCallMediator`] hook.
//!
//! This is the Track B adapter layer: all governance semantics live here, never in
//! the `full-monty` submodule itself.

mod external_call;
mod observer;
mod run;

pub use external_call::{
    AllowAllMediator, CallContext, ConfirmationContext, DenyAllMediator, ExternalCallMediator,
    MediationDecision,
};
pub use observer::ZamburakObserver;
pub use run::{GovernedRunError, GovernedRunProgress, GovernedRunner, SuspendedCall};
