//! Governed run-loop helpers for mediating and resuming suspended calls.

use std::sync::{Arc, Mutex};

use monty::{ExtFunctionResult, ExternalCallKind, PrintWriter, ResourceTracker, RunProgress};

use crate::external_call::{
    CallContext, ConfirmationContext, ExternalCallMediator, MediationDecision,
};
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
    if is_complete_progress(&progress) {
        return finish_complete_progress(progress);
    }
    if is_function_call_progress(&progress) {
        let call = progress
            .into_function_call()
            .expect("function-call progress should contain FunctionCall");
        return mediate_function_call(call, &resources);
    }
    if is_os_call_progress(&progress) {
        let call = progress
            .into_os_call()
            .expect("OS-call progress should contain OsCall");
        return mediate_os_call(call, &resources);
    }
    if is_name_lookup_progress(&progress) {
        return suspend_name_lookup(progress);
    }
    suspend_resolve_futures(progress)
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
    if is_allow_decision(&decision) {
        return Ok(GovernedRunProgress::ExternalCallPending {
            context,
            suspended: suspended_function_call(call, resources),
        });
    }
    if is_deny_decision(&decision) {
        let reason = deny_reason(decision).expect("deny decision should include reason");
        return Ok(GovernedRunProgress::Denied {
            reason,
            function_name: context.function_name,
            call_id: context.call_id,
        });
    }
    let request =
        confirmation_request(decision).expect("confirmation decision should include request");
    Ok(GovernedRunProgress::AwaitConfirmation {
        context: request,
        suspended: suspended_function_call(call, resources),
    })
}

fn resolve_os_call_decision<T: ResourceTracker>(
    call: monty::OsCall<T>,
    context: CallContext,
    decision: MediationDecision,
    resources: &MediationResources<'_>,
) -> Result<GovernedRunProgress<T>, GovernedRunError> {
    if is_allow_decision(&decision) {
        return Ok(GovernedRunProgress::ExternalCallPending {
            context,
            suspended: suspended_os_call(call, resources),
        });
    }
    if is_deny_decision(&decision) {
        let reason = deny_reason(decision).expect("deny decision should include reason");
        return Ok(GovernedRunProgress::Denied {
            reason,
            function_name: context.function_name,
            call_id: context.call_id,
        });
    }
    let request =
        confirmation_request(decision).expect("confirmation decision should include request");
    Ok(GovernedRunProgress::AwaitConfirmation {
        context: request,
        suspended: suspended_os_call(call, resources),
    })
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

fn finish_complete_progress<T: ResourceTracker>(
    progress: RunProgress<T>,
) -> Result<GovernedRunProgress<T>, GovernedRunError> {
    Ok(GovernedRunProgress::Complete(
        progress
            .into_complete()
            .expect("complete progress should contain a final value"),
    ))
}

fn suspend_name_lookup<T: ResourceTracker>(
    progress: RunProgress<T>,
) -> Result<GovernedRunProgress<T>, GovernedRunError> {
    let lookup = progress
        .into_name_lookup()
        .expect("name-lookup progress should contain NameLookup");
    Ok(GovernedRunProgress::NameLookup {
        name: lookup.name.clone(),
        inner: lookup,
    })
}

fn suspend_resolve_futures<T: ResourceTracker>(
    progress: RunProgress<T>,
) -> Result<GovernedRunProgress<T>, GovernedRunError> {
    Ok(GovernedRunProgress::ResolveFutures(
        progress
            .into_resolve_futures()
            .expect("resolve-futures progress should contain ResolveFutures"),
    ))
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

fn is_complete_progress<T: ResourceTracker>(progress: &RunProgress<T>) -> bool {
    matches!(progress, RunProgress::Complete(_))
}

fn is_function_call_progress<T: ResourceTracker>(progress: &RunProgress<T>) -> bool {
    matches!(progress, RunProgress::FunctionCall(_))
}

fn is_os_call_progress<T: ResourceTracker>(progress: &RunProgress<T>) -> bool {
    matches!(progress, RunProgress::OsCall(_))
}

fn is_name_lookup_progress<T: ResourceTracker>(progress: &RunProgress<T>) -> bool {
    matches!(progress, RunProgress::NameLookup(_))
}

fn is_allow_decision(decision: &MediationDecision) -> bool {
    matches!(decision, MediationDecision::Allow)
}

fn is_deny_decision(decision: &MediationDecision) -> bool {
    matches!(decision, MediationDecision::Deny { .. })
}

fn deny_reason(decision: MediationDecision) -> Option<String> {
    if let MediationDecision::Deny { reason } = decision {
        return Some(reason);
    }
    None
}

fn confirmation_request(decision: MediationDecision) -> Option<ConfirmationContext> {
    if let MediationDecision::RequireConfirmation { request } = decision {
        return Some(request);
    }
    None
}
