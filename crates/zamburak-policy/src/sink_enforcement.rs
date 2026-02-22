//! Contract types for LLM sink enforcement architecture.
//!
//! LLM sink checks run at three explicit points described in
//! `docs/zamburak-design-document.md` section "LLM sink enforcement
//! architecture":
//!
//! 1. pre-dispatch policy check in the runtime effect gateway,
//! 2. adapter-level transport guard in the LLM tool adapter,
//! 3. post-dispatch audit emission in the audit pipeline.
//!
//! These contracts define the type shapes and decision vocabulary for
//! design-level conformance testing. The contract functions encode the
//! design-contract minimum: calls without redaction are denied. Full
//! budget and context evaluation belongs to Phase 4 (Task 4.1.2).

/// Newtype for execution identifiers linking calls to the audit chain.
///
/// Prevents accidental mix-up between execution and call identifiers
/// at downstream call sites.
///
/// # Examples
///
/// ```rust
/// use zamburak_policy::sink_enforcement::ExecutionId;
///
/// let id = ExecutionId::new("exec_01");
/// assert_eq!(id.as_str(), "exec_01");
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionId(String);

impl ExecutionId {
    /// Create a new execution identifier.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// View the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Newtype for per-call identifiers linking pre-dispatch to
/// post-dispatch audit records.
///
/// Prevents accidental mix-up between call and execution identifiers
/// at downstream call sites.
///
/// # Examples
///
/// ```rust
/// use zamburak_policy::sink_enforcement::CallId;
///
/// let id = CallId::new("call_01");
/// assert_eq!(id.as_str(), "call_01");
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallId(String);

impl CallId {
    /// Create a new call identifier.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// View the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Discriminant for LLM call path classification.
///
/// P-LLM calls are trusted planner queries; Q-LLM calls are
/// quarantined transformations of untrusted tool outputs.
///
/// # Examples
///
/// ```rust
/// use zamburak_policy::sink_enforcement::LlmCallPath;
///
/// assert_ne!(LlmCallPath::Planner, LlmCallPath::Quarantined);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LlmCallPath {
    /// Planner LLM path: trusted query decomposition and plan
    /// synthesis.
    Planner,
    /// Quarantined LLM path: transformation of untrusted tool outputs.
    Quarantined,
}

/// Pre-dispatch policy check request for an LLM sink call.
///
/// Carries the argument and execution-context summaries that the
/// runtime effect gateway evaluates before any remote prompt emission.
/// Both P-LLM and Q-LLM calls must pass pre-dispatch checks.
///
/// # Examples
///
/// ```rust
/// use zamburak_policy::sink_enforcement::{
///     CallId, ExecutionId, LlmCallPath, SinkPreDispatchRequest,
/// };
///
/// let request = SinkPreDispatchRequest {
///     execution_id: ExecutionId::new("exec_01"),
///     call_id: CallId::new("call_01"),
///     call_path: LlmCallPath::Planner,
///     redaction_applied: true,
/// };
/// assert_eq!(request.call_path, LlmCallPath::Planner);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SinkPreDispatchRequest {
    /// Execution identifier linking this call to the audit chain.
    pub execution_id: ExecutionId,
    /// Per-call identifier for audit linkage.
    pub call_id: CallId,
    /// LLM call path classification.
    pub call_path: LlmCallPath,
    /// Whether required minimization and redaction transforms have been
    /// applied before dispatch.
    pub redaction_applied: bool,
}

/// Decision from the pre-dispatch policy check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SinkPreDispatchDecision {
    /// Call is permitted to proceed.
    Allow,
    /// Call is denied by policy.
    Deny,
}

/// Transport guard check at the adapter level.
///
/// Ensures required redaction and minimization transforms were applied
/// before payload leaves the process boundary. This check is separate
/// from the pre-dispatch check because the design specifies it runs at
/// the LLM tool adapter, not at the runtime effect gateway.
///
/// # Examples
///
/// ```rust
/// use zamburak_policy::sink_enforcement::{
///     CallId, ExecutionId, TransportGuardCheck,
/// };
///
/// let check = TransportGuardCheck {
///     execution_id: ExecutionId::new("exec_01"),
///     call_id: CallId::new("call_01"),
///     redaction_applied: true,
/// };
/// assert!(check.redaction_applied);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportGuardCheck {
    /// Execution identifier for audit linkage.
    pub execution_id: ExecutionId,
    /// Per-call identifier for audit linkage.
    pub call_id: CallId,
    /// Whether required redaction transforms were applied.
    pub redaction_applied: bool,
}

/// Outcome of the transport guard check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TransportGuardOutcome {
    /// Payload passes the transport guard.
    Passed,
    /// Payload blocked: required transforms were not applied.
    Blocked,
}

/// Post-dispatch audit record for an LLM sink call.
///
/// Emitted after the call completes (or after denial), carrying the
/// decision code and payload-hash witness linkage. Linked by
/// `execution_id` and `call_id` as required by the design document
/// audit record schema.
///
/// # Examples
///
/// ```rust
/// use zamburak_policy::sink_enforcement::{
///     CallId, ExecutionId, LlmCallPath, SinkAuditRecord,
///     SinkPreDispatchDecision,
/// };
///
/// let record = SinkAuditRecord {
///     execution_id: ExecutionId::new("exec_7f2c"),
///     call_id: CallId::new("call_0192"),
///     decision: SinkPreDispatchDecision::Allow,
///     redaction_applied: true,
///     call_path: LlmCallPath::Planner,
/// };
/// assert_eq!(record.execution_id, ExecutionId::new("exec_7f2c"));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SinkAuditRecord {
    /// Execution identifier linking to the broader execution context.
    pub execution_id: ExecutionId,
    /// Per-call identifier linking pre-dispatch to post-dispatch.
    pub call_id: CallId,
    /// Pre-dispatch decision code.
    pub decision: SinkPreDispatchDecision,
    /// Whether redaction was applied before transport.
    pub redaction_applied: bool,
    /// LLM call path classification.
    pub call_path: LlmCallPath,
}

/// Evaluate pre-dispatch policy check for an LLM sink call.
///
/// At the design-contract level, this enforces that calls without
/// required redaction transforms are denied. Full policy evaluation
/// (confidentiality budgets, provider-specific approval, context
/// summaries) belongs to Phase 4 (Task 4.1.2).
///
/// # Examples
///
/// ```rust
/// use zamburak_policy::sink_enforcement::{
///     CallId, ExecutionId, LlmCallPath, SinkPreDispatchDecision,
///     SinkPreDispatchRequest, evaluate_pre_dispatch,
/// };
///
/// let request = SinkPreDispatchRequest {
///     execution_id: ExecutionId::new("exec_01"),
///     call_id: CallId::new("call_01"),
///     call_path: LlmCallPath::Planner,
///     redaction_applied: true,
/// };
/// assert_eq!(evaluate_pre_dispatch(&request), SinkPreDispatchDecision::Allow);
/// ```
#[must_use]
pub fn evaluate_pre_dispatch(request: &SinkPreDispatchRequest) -> SinkPreDispatchDecision {
    if request.redaction_applied {
        SinkPreDispatchDecision::Allow
    } else {
        SinkPreDispatchDecision::Deny
    }
}

/// Evaluate transport guard at the adapter level.
///
/// Blocks payload transmission when required redaction transforms have
/// not been applied.
///
/// # Examples
///
/// ```rust
/// use zamburak_policy::sink_enforcement::{
///     CallId, ExecutionId, TransportGuardCheck,
///     TransportGuardOutcome, evaluate_transport_guard,
/// };
///
/// let check = TransportGuardCheck {
///     execution_id: ExecutionId::new("exec_01"),
///     call_id: CallId::new("call_01"),
///     redaction_applied: true,
/// };
/// assert_eq!(
///     evaluate_transport_guard(&check),
///     TransportGuardOutcome::Passed,
/// );
/// ```
#[must_use]
pub fn evaluate_transport_guard(check: &TransportGuardCheck) -> TransportGuardOutcome {
    if check.redaction_applied {
        TransportGuardOutcome::Passed
    } else {
        TransportGuardOutcome::Blocked
    }
}

/// Emit a post-dispatch audit record linking pre-dispatch decision to
/// execution and call identifiers.
///
/// Both P-LLM and Q-LLM paths emit linked audit records keyed by
/// `execution_id` and `call_id`.
///
/// # Examples
///
/// ```rust
/// use zamburak_policy::sink_enforcement::{
///     CallId, ExecutionId, LlmCallPath, SinkPreDispatchRequest,
///     emit_audit_record, evaluate_pre_dispatch,
/// };
///
/// let request = SinkPreDispatchRequest {
///     execution_id: ExecutionId::new("exec_01"),
///     call_id: CallId::new("call_01"),
///     call_path: LlmCallPath::Planner,
///     redaction_applied: true,
/// };
/// let decision = evaluate_pre_dispatch(&request);
/// let audit = emit_audit_record(&request, decision);
/// assert_eq!(audit.execution_id, ExecutionId::new("exec_01"));
/// assert_eq!(audit.call_id, CallId::new("call_01"));
/// assert_eq!(audit.call_path, LlmCallPath::Planner);
/// ```
#[must_use]
pub fn emit_audit_record(
    request: &SinkPreDispatchRequest,
    decision: SinkPreDispatchDecision,
) -> SinkAuditRecord {
    SinkAuditRecord {
        execution_id: request.execution_id.clone(),
        call_id: request.call_id.clone(),
        decision,
        redaction_applied: request.redaction_applied,
        call_path: request.call_path,
    }
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use super::{
        CallId, ExecutionId, LlmCallPath, SinkPreDispatchDecision, SinkPreDispatchRequest,
        TransportGuardCheck, TransportGuardOutcome, emit_audit_record, evaluate_pre_dispatch,
        evaluate_transport_guard,
    };

    #[fixture]
    fn planner_request() -> SinkPreDispatchRequest {
        SinkPreDispatchRequest {
            execution_id: ExecutionId::new("exec_test"),
            call_id: CallId::new("call_test"),
            call_path: LlmCallPath::Planner,
            redaction_applied: true,
        }
    }

    #[fixture]
    fn transport_guard_check() -> TransportGuardCheck {
        TransportGuardCheck {
            execution_id: ExecutionId::new("exec_test"),
            call_id: CallId::new("call_test"),
            redaction_applied: true,
        }
    }

    #[rstest]
    fn pre_dispatch_allows_with_redaction(planner_request: SinkPreDispatchRequest) {
        let decision = evaluate_pre_dispatch(&planner_request);
        assert_eq!(decision, SinkPreDispatchDecision::Allow);
    }

    #[rstest]
    fn pre_dispatch_denies_without_redaction(mut planner_request: SinkPreDispatchRequest) {
        planner_request.redaction_applied = false;
        let decision = evaluate_pre_dispatch(&planner_request);
        assert_eq!(decision, SinkPreDispatchDecision::Deny);
    }

    #[rstest]
    fn transport_guard_passes_with_redaction(transport_guard_check: TransportGuardCheck) {
        assert_eq!(
            evaluate_transport_guard(&transport_guard_check),
            TransportGuardOutcome::Passed
        );
    }

    #[rstest]
    fn transport_guard_blocks_without_redaction(mut transport_guard_check: TransportGuardCheck) {
        transport_guard_check.redaction_applied = false;
        assert_eq!(
            evaluate_transport_guard(&transport_guard_check),
            TransportGuardOutcome::Blocked
        );
    }

    #[rstest]
    fn audit_record_preserves_linkage_fields() {
        let request = SinkPreDispatchRequest {
            execution_id: ExecutionId::new("exec_7f2c"),
            call_id: CallId::new("call_0192"),
            call_path: LlmCallPath::Planner,
            redaction_applied: true,
        };
        let decision = evaluate_pre_dispatch(&request);
        let audit = emit_audit_record(&request, decision);

        assert_eq!(audit.execution_id, ExecutionId::new("exec_7f2c"));
        assert_eq!(audit.call_id, CallId::new("call_0192"));
        assert_eq!(audit.decision, SinkPreDispatchDecision::Allow);
        assert_eq!(audit.call_path, LlmCallPath::Planner);
        assert!(audit.redaction_applied);
    }

    #[rstest]
    fn quarantined_path_discrimination() {
        let request = SinkPreDispatchRequest {
            execution_id: ExecutionId::new("exec_q"),
            call_id: CallId::new("call_q"),
            call_path: LlmCallPath::Quarantined,
            redaction_applied: true,
        };
        let decision = evaluate_pre_dispatch(&request);
        let audit = emit_audit_record(&request, decision);

        assert_eq!(audit.call_path, LlmCallPath::Quarantined);
    }
}
