//! Governed run-loop helpers for mediating and resuming suspended calls.

use std::sync::{Arc, Mutex};

use monty::{
    ExtFunctionResult, ExternalCallKind, NameLookupResult, PrintWriter, ResourceTracker,
    RunProgress,
};

use crate::external_call::{CallContext, ExternalCallMediator, MediationDecision};
use crate::observer::SharedObserverState;
use crate::run::{GovernedRunError, GovernedRunProgress};

/// A suspended external call awaiting host input before resume.
pub struct SuspendedCall<T: ResourceTracker> {
    kind: SuspendedCallKind<T>,
    mediator: Arc<Mutex<dyn ExternalCallMediator>>,
    observer_state: SharedObserverState,
}

/// Internal discriminant for the two suspendable call types.
#[derive(Debug)]
enum SuspendedCallKind<T: ResourceTracker> {
    /// A suspended external function call.
    Function(monty::FunctionCall<T>),
    /// A suspended OS call.
    Os(monty::OsCall<T>),
}

impl<T: ResourceTracker> std::fmt::Debug for SuspendedCall<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SuspendedCall")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl<T: ResourceTracker> SuspendedCall<T> {
    /// Resumes the suspended call with a host-provided result.
    ///
    /// # Errors
    ///
    /// Returns `GovernedRunError::Interpreter` if the interpreter raises an
    /// exception on resume.
    pub fn resume(
        self,
        result: impl Into<ExtFunctionResult>,
        print: PrintWriter<'_>,
    ) -> Result<GovernedRunProgress<T>, GovernedRunError> {
        let progress = resume_suspended_call(self.kind, result, print)?;
        step(progress, &self.mediator, &self.observer_state)
    }
}

/// A suspended name lookup that must resume through governed mediation.
pub struct SuspendedNameLookup<T: ResourceTracker> {
    inner: monty::NameLookup<T>,
    mediator: Arc<Mutex<dyn ExternalCallMediator>>,
    observer_state: SharedObserverState,
}

impl<T: ResourceTracker> std::fmt::Debug for SuspendedNameLookup<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SuspendedNameLookup")
            .field("name", &self.inner.name)
            .finish_non_exhaustive()
    }
}

impl<T: ResourceTracker> SuspendedNameLookup<T> {
    /// Returns the unresolved name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Resumes execution after the host resolves the name lookup.
    ///
    /// # Errors
    ///
    /// Returns `GovernedRunError::Interpreter` if the interpreter raises an
    /// exception on resume.
    pub fn resume(
        self,
        result: impl Into<NameLookupResult>,
        print: PrintWriter<'_>,
    ) -> Result<GovernedRunProgress<T>, GovernedRunError> {
        let progress = self.inner.resume(result, print)?;
        step(progress, &self.mediator, &self.observer_state)
    }
}

/// Suspended async work awaiting future resolution through governed mediation.
pub struct SuspendedResolveFutures<T: ResourceTracker> {
    inner: monty::ResolveFutures<T>,
    mediator: Arc<Mutex<dyn ExternalCallMediator>>,
    observer_state: SharedObserverState,
}

impl<T: ResourceTracker> std::fmt::Debug for SuspendedResolveFutures<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SuspendedResolveFutures")
            .field("pending_call_ids", &self.inner.pending_call_ids())
            .finish_non_exhaustive()
    }
}

impl<T: ResourceTracker> SuspendedResolveFutures<T> {
    /// Returns unresolved external-call identifiers.
    #[must_use]
    pub fn pending_call_ids(&self) -> &[u32] {
        self.inner.pending_call_ids()
    }

    /// Resumes execution with results for one or more pending futures.
    ///
    /// # Errors
    ///
    /// Returns `GovernedRunError::Interpreter` if the interpreter raises an
    /// exception on resume.
    pub fn resume(
        self,
        results: Vec<(u32, ExtFunctionResult)>,
        print: PrintWriter<'_>,
    ) -> Result<GovernedRunProgress<T>, GovernedRunError> {
        let progress = self.inner.resume(results, print)?;
        step(progress, &self.mediator, &self.observer_state)
    }
}

struct MediationResources<'a> {
    mediator: &'a Arc<Mutex<dyn ExternalCallMediator>>,
    observer_state: &'a SharedObserverState,
}

/// Processes a single `RunProgress` step, mediating external calls.
pub(crate) fn step<T: ResourceTracker>(
    progress: RunProgress<T>,
    mediator: &Arc<Mutex<dyn ExternalCallMediator>>,
    observer_state: &SharedObserverState,
) -> Result<GovernedRunProgress<T>, GovernedRunError> {
    let resources = MediationResources {
        mediator,
        observer_state,
    };
    match progress {
        RunProgress::Complete(value) => Ok(GovernedRunProgress::Complete(value)),
        RunProgress::FunctionCall(call) => mediate_function_call(call, &resources),
        RunProgress::OsCall(call) => mediate_os_call(call, &resources),
        RunProgress::NameLookup(lookup) => Ok(GovernedRunProgress::NameLookup {
            name: lookup.name.clone(),
            suspended: suspended_name_lookup(lookup, &resources),
        }),
        RunProgress::ResolveFutures(resolve_futures) => Ok(GovernedRunProgress::ResolveFutures(
            suspended_resolve_futures(resolve_futures, &resources),
        )),
    }
}

fn mediate_function_call<T: ResourceTracker>(
    call: monty::FunctionCall<T>,
    resources: &MediationResources<'_>,
) -> Result<GovernedRunProgress<T>, GovernedRunError> {
    let call_context = build_function_call_context(&call, resources.observer_state);
    let decision = query_mediator(resources.mediator, &call_context)?;
    resolve_function_call_decision(call, call_context, decision, resources)
}

fn mediate_os_call<T: ResourceTracker>(
    call: monty::OsCall<T>,
    resources: &MediationResources<'_>,
) -> Result<GovernedRunProgress<T>, GovernedRunError> {
    let call_context = build_os_call_context(&call, resources.observer_state);
    let decision = query_mediator(resources.mediator, &call_context)?;
    resolve_os_call_decision(call, call_context, decision, resources)
}

fn build_function_call_context<T: ResourceTracker>(
    call: &monty::FunctionCall<T>,
    observer_state: &SharedObserverState,
) -> CallContext {
    let kind = if call.method_call {
        ExternalCallKind::Method
    } else {
        ExternalCallKind::Function
    };
    observer_state.consume_pending_call(call.call_id, kind);
    CallContext {
        call_id: call.call_id,
        kind,
        function_name: call.function_name.clone(),
    }
}

fn build_os_call_context<T: ResourceTracker>(
    call: &monty::OsCall<T>,
    observer_state: &SharedObserverState,
) -> CallContext {
    observer_state.consume_pending_call(call.call_id, ExternalCallKind::Os);
    CallContext {
        call_id: call.call_id,
        kind: ExternalCallKind::Os,
        function_name: format!("{:?}", call.function),
    }
}

fn resolve_function_call_decision<T: ResourceTracker>(
    call: monty::FunctionCall<T>,
    context: CallContext,
    decision: MediationDecision,
    resources: &MediationResources<'_>,
) -> Result<GovernedRunProgress<T>, GovernedRunError> {
    match decision {
        MediationDecision::Allow => Ok(GovernedRunProgress::ExternalCallPending {
            context,
            suspended: suspended_function_call(call, resources),
        }),
        MediationDecision::Deny { reason } => Ok(GovernedRunProgress::Denied {
            reason,
            function_name: context.function_name,
            call_id: context.call_id,
        }),
        MediationDecision::RequireConfirmation { request } => {
            Ok(GovernedRunProgress::AwaitConfirmation {
                context: request,
                suspended: suspended_function_call(call, resources),
            })
        }
    }
}

fn resolve_os_call_decision<T: ResourceTracker>(
    call: monty::OsCall<T>,
    context: CallContext,
    decision: MediationDecision,
    resources: &MediationResources<'_>,
) -> Result<GovernedRunProgress<T>, GovernedRunError> {
    match decision {
        MediationDecision::Allow => Ok(GovernedRunProgress::ExternalCallPending {
            context,
            suspended: suspended_os_call(call, resources),
        }),
        MediationDecision::Deny { reason } => Ok(GovernedRunProgress::Denied {
            reason,
            function_name: context.function_name,
            call_id: context.call_id,
        }),
        MediationDecision::RequireConfirmation { request } => {
            Ok(GovernedRunProgress::AwaitConfirmation {
                context: request,
                suspended: suspended_os_call(call, resources),
            })
        }
    }
}

fn suspended_function_call<T: ResourceTracker>(
    call: monty::FunctionCall<T>,
    resources: &MediationResources<'_>,
) -> SuspendedCall<T> {
    SuspendedCall {
        kind: SuspendedCallKind::Function(call),
        mediator: Arc::clone(resources.mediator),
        observer_state: resources.observer_state.clone(),
    }
}

fn suspended_os_call<T: ResourceTracker>(
    call: monty::OsCall<T>,
    resources: &MediationResources<'_>,
) -> SuspendedCall<T> {
    SuspendedCall {
        kind: SuspendedCallKind::Os(call),
        mediator: Arc::clone(resources.mediator),
        observer_state: resources.observer_state.clone(),
    }
}

fn suspended_name_lookup<T: ResourceTracker>(
    lookup: monty::NameLookup<T>,
    resources: &MediationResources<'_>,
) -> SuspendedNameLookup<T> {
    SuspendedNameLookup {
        inner: lookup,
        mediator: Arc::clone(resources.mediator),
        observer_state: resources.observer_state.clone(),
    }
}

fn suspended_resolve_futures<T: ResourceTracker>(
    resolve_futures: monty::ResolveFutures<T>,
    resources: &MediationResources<'_>,
) -> SuspendedResolveFutures<T> {
    SuspendedResolveFutures {
        inner: resolve_futures,
        mediator: Arc::clone(resources.mediator),
        observer_state: resources.observer_state.clone(),
    }
}

fn resume_suspended_call<T: ResourceTracker>(
    suspended: SuspendedCallKind<T>,
    result: impl Into<ExtFunctionResult>,
    print: PrintWriter<'_>,
) -> Result<RunProgress<T>, GovernedRunError> {
    match suspended {
        SuspendedCallKind::Function(call) => {
            call.resume(result, print).map_err(GovernedRunError::from)
        }
        SuspendedCallKind::Os(call) => call.resume(result, print).map_err(GovernedRunError::from),
    }
}

fn query_mediator(
    mediator: &Arc<Mutex<dyn ExternalCallMediator>>,
    context: &CallContext,
) -> Result<MediationDecision, GovernedRunError> {
    let mut guard = mediator
        .lock()
        .map_err(|_| GovernedRunError::MediatorPoisoned)?;
    Ok(guard.mediate(context))
}
