//! Shared mediation helpers for governed run-loop flow control.

use std::sync::{Arc, Mutex};

use monty::{ExternalCallKind, MontyObject, OsFunction, ResourceTracker, RunProgress};
use zamburak_core::trust::AuthoritySet;

use crate::external_call::{CallContext, ExternalCallMediator};
use crate::observer::SharedObserverState;
use crate::run::{GovernedRunError, GovernedRunProgress};

use super::{map_call_ifc_lookup_error, step};

pub(crate) struct MediationResources<'a> {
    pub(crate) mediator: &'a Arc<Mutex<dyn ExternalCallMediator>>,
    pub(crate) observer_state: &'a SharedObserverState,
    pub(crate) caller_authority: &'a AuthoritySet,
}

struct CallContextRequest<'a> {
    call_id: u32,
    kind: ExternalCallKind,
    function_name: String,
    kwargs: &'a [(MontyObject, MontyObject)],
}

pub(crate) fn build_function_call_context<T: ResourceTracker>(
    call: &monty::FunctionCall<T>,
    observer_state: &SharedObserverState,
    caller_authority: &AuthoritySet,
) -> Result<CallContext, GovernedRunError> {
    let kind = if call.method_call {
        ExternalCallKind::Method
    } else {
        ExternalCallKind::Function
    };
    build_call_context(
        CallContextRequest {
            call_id: call.call_id,
            kind,
            function_name: call.function_name.clone(),
            kwargs: &call.kwargs,
        },
        observer_state,
        caller_authority,
    )
}

pub(crate) fn build_os_call_context<T: ResourceTracker>(
    call: &monty::OsCall<T>,
    observer_state: &SharedObserverState,
    caller_authority: &AuthoritySet,
) -> Result<CallContext, GovernedRunError> {
    build_call_context(
        CallContextRequest {
            call_id: call.call_id,
            kind: ExternalCallKind::Os,
            function_name: map_os_function_to_policy_name(call.function).to_owned(),
            kwargs: &call.kwargs,
        },
        observer_state,
        caller_authority,
    )
}

fn build_call_context(
    request: CallContextRequest<'_>,
    observer_state: &SharedObserverState,
    caller_authority: &AuthoritySet,
) -> Result<CallContext, GovernedRunError> {
    let ifc = observer_state
        .call_ifc_context(request.call_id, request.kind, &request.function_name)
        .map_err(map_call_ifc_lookup_error)?;
    Ok(CallContext {
        call_id: request.call_id,
        kind: request.kind,
        function_name: request.function_name,
        caller_authority: caller_authority.clone(),
        kwarg_names: extract_kwarg_names(
            request.kwargs,
            ifc.kwarg_summaries.len(),
            request.call_id,
            request.kind,
        )?,
        ifc,
    })
}

fn extract_kwarg_names(
    kwargs: &[(MontyObject, MontyObject)],
    expected_summary_count: usize,
    call_id: u32,
    kind: ExternalCallKind,
) -> Result<Vec<String>, GovernedRunError> {
    if kwargs.len() != expected_summary_count {
        return Err(GovernedRunError::ObserverMismatch { call_id, kind });
    }

    kwargs
        .iter()
        .map(|(key, _value)| match key {
            MontyObject::String(name) => Ok(name.clone()),
            _ => Err(GovernedRunError::ObserverMismatch { call_id, kind }),
        })
        .collect()
}

pub(crate) fn map_os_function_to_policy_name(function: OsFunction) -> &'static str {
    match function {
        OsFunction::Exists => "Path.exists",
        OsFunction::IsFile => "Path.is_file",
        OsFunction::IsDir => "Path.is_dir",
        OsFunction::IsSymlink => "Path.is_symlink",
        OsFunction::ReadText => "Path.read_text",
        OsFunction::ReadBytes => "Path.read_bytes",
        OsFunction::WriteText => "Path.write_text",
        OsFunction::WriteBytes => "Path.write_bytes",
        OsFunction::Mkdir => "Path.mkdir",
        OsFunction::Unlink => "Path.unlink",
        OsFunction::Rmdir => "Path.rmdir",
        OsFunction::Iterdir => "Path.iterdir",
        OsFunction::Stat => "Path.stat",
        OsFunction::Rename => "Path.rename",
        OsFunction::Resolve => "Path.resolve",
        OsFunction::Absolute => "Path.absolute",
        OsFunction::Getenv => "os.getenv",
        OsFunction::GetEnviron => "os.environ",
    }
}

pub(crate) fn resume_and_step<T, E, F>(
    resume: F,
    mediator: &Arc<Mutex<dyn ExternalCallMediator>>,
    observer_state: &SharedObserverState,
    caller_authority: &AuthoritySet,
) -> Result<GovernedRunProgress<T>, GovernedRunError>
where
    T: ResourceTracker,
    E: Into<GovernedRunError>,
    F: FnOnce() -> Result<RunProgress<T>, E>,
{
    let progress = resume().map_err(Into::into)?;
    step(progress, mediator, observer_state, caller_authority)
}
