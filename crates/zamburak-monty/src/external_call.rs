//! External-call mediation traits and built-in mediator implementations.
//!
//! The [`ExternalCallMediator`] trait defines the deterministic hook that the
//! governed runner invokes at each external-call boundary. Implementations
//! receive call metadata and return a [`MediationDecision`] indicating whether
//! the call should be allowed, denied, or held for interactive confirmation.
//!
//! Two built-in mediators are provided for testing and permissive operation:
//! [`AllowAllMediator`] and [`DenyAllMediator`].

use monty::ExternalCallKind;

/// Contextual metadata presented to the mediator at each external-call boundary.
///
/// The governed runner populates this from the `RunProgress` call-yield fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallContext {
    /// Host-visible call identifier.
    pub call_id: u32,
    /// External-call class (function, OS, or method).
    pub kind: ExternalCallKind,
    /// Name of the function or OS operation being called.
    pub function_name: String,
}

/// Context attached to a `RequireConfirmation` decision for host-interactive
/// approval workflows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfirmationContext {
    /// Human-readable description of what the call intends to do.
    pub description: String,
    /// The call context that triggered the confirmation request.
    pub call: CallContext,
}

/// Outcome of mediating an external-call request.
///
/// The governed runner inspects this to decide how to resume (or halt)
/// execution after an external-call yield.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MediationDecision {
    /// Proceed with the external call.
    Allow,
    /// Block the external call with an explanation.
    Deny {
        /// Human-readable reason for denial.
        reason: String,
    },
    /// Yield to the host for interactive approval before proceeding.
    RequireConfirmation {
        /// Context for the host to display and act upon.
        request: ConfirmationContext,
    },
}

/// Trait for external-call mediation at governed execution boundaries.
///
/// Implementations receive a [`CallContext`] describing the pending call and
/// return a [`MediationDecision`]. The governed runner calls this synchronously
/// from the execution loop.
///
/// # Examples
///
/// ```
/// use zamburak_monty::{AllowAllMediator, ExternalCallMediator, MediationDecision};
/// use zamburak_monty::CallContext;
/// use monty::ExternalCallKind;
///
/// let mut mediator = AllowAllMediator;
/// let ctx = CallContext {
///     call_id: 1,
///     kind: ExternalCallKind::Function,
///     function_name: "print".to_owned(),
/// };
/// assert_eq!(mediator.mediate(&ctx), MediationDecision::Allow);
/// ```
pub trait ExternalCallMediator: Send {
    /// Evaluate whether the described external call should proceed.
    fn mediate(&mut self, context: &CallContext) -> MediationDecision;
}

/// Unconditionally allows every external call. Used for testing and
/// permissive-mode operation.
///
/// # Examples
///
/// ```
/// use zamburak_monty::{AllowAllMediator, ExternalCallMediator, MediationDecision};
/// use zamburak_monty::CallContext;
/// use monty::ExternalCallKind;
///
/// let mut m = AllowAllMediator;
/// let ctx = CallContext {
///     call_id: 0,
///     kind: ExternalCallKind::Os,
///     function_name: "open".to_owned(),
/// };
/// assert_eq!(m.mediate(&ctx), MediationDecision::Allow);
/// ```
pub struct AllowAllMediator;

impl ExternalCallMediator for AllowAllMediator {
    fn mediate(&mut self, _context: &CallContext) -> MediationDecision {
        MediationDecision::Allow
    }
}

/// Unconditionally denies every external call. Used for testing deny-path
/// coverage.
///
/// # Examples
///
/// ```
/// use zamburak_monty::{DenyAllMediator, ExternalCallMediator, MediationDecision};
/// use zamburak_monty::CallContext;
/// use monty::ExternalCallKind;
///
/// let mut m = DenyAllMediator;
/// let ctx = CallContext {
///     call_id: 0,
///     kind: ExternalCallKind::Function,
///     function_name: "exit".to_owned(),
/// };
/// assert!(matches!(m.mediate(&ctx), MediationDecision::Deny { .. }));
/// ```
pub struct DenyAllMediator;

impl ExternalCallMediator for DenyAllMediator {
    fn mediate(&mut self, context: &CallContext) -> MediationDecision {
        MediationDecision::Deny {
            reason: format!(
                "all external calls denied by DenyAllMediator (call_id={}, fn={})",
                context.call_id, context.function_name
            ),
        }
    }
}

#[cfg(test)]
#[path = "external_call_tests.rs"]
mod tests;
