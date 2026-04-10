//! Policy-backed external-call mediator.
//!
//! `PolicyMediator` implements `ExternalCallMediator` by delegating to a
//! `PolicyEngine` from `zamburak-policy`. It translates `CallContext` into
//! policy-engine input and converts policy decisions back into
//! `MediationDecision`.

use monty::ExternalCallKind;
use zamburak_policy::PolicyEngine;
use zamburak_policy::{
    ExternalCallPolicyDecision, ExternalCallPolicyInput, KeywordArgumentSummary,
};

use crate::external_call::{
    CallContext, ConfirmationContext, ExternalCallMediator, MediationDecision,
};

/// Policy-backed mediator for governed external-call boundaries.
///
/// Consumes a `PolicyEngine` at construction and evaluates each external-call
/// request against the loaded policy rules.
///
/// # Examples
///
/// ```
/// use zamburak_monty::{PolicyMediator, ExternalCallMediator, MediationDecision};
/// use zamburak_monty::CallContext;
/// use monty::ExternalCallKind;
/// use zamburak_core::control_context::ExecutionContextSummary;
/// use zamburak_core::propagation::PropagationMode;
/// use zamburak_core::DependencySummary;
/// use zamburak_policy::PolicyEngine;
///
/// let policy_yaml = r#"
/// schema_version: 1
/// policy_name: test_policy
/// default_action: Deny
/// strict_mode: true
/// budgets:
///   max_values: 100
///   max_parents_per_value: 10
///   max_closure_steps: 100
///   max_witness_depth: 10
/// tools: []
/// "#;
///
/// let engine = PolicyEngine::from_yaml_str(policy_yaml).expect("valid policy");
/// let mut mediator = PolicyMediator::new(engine);
///
/// let ctx = CallContext {
///     call_id: 1,
///     kind: ExternalCallKind::Function,
///     function_name: "unknown_tool".to_owned(),
///     caller_authority: zamburak_core::AuthoritySet::full(),
///     kwarg_names: vec![],
///     ifc: zamburak_monty::CallIfcContext {
///         propagation_mode: PropagationMode::Normal,
///         aggregate_summary: DependencySummary::unknown_top(),
///         control_context: ExecutionContextSummary::new(),
///         arg_summaries: vec![],
///         kwarg_summaries: vec![],
///     },
/// };
///
/// // Unknown tool should be denied (fail-closed).
/// assert!(matches!(mediator.mediate(&ctx), MediationDecision::Deny { .. }));
/// ```
pub struct PolicyMediator {
    engine: PolicyEngine,
}

impl PolicyMediator {
    /// Construct a new policy-backed mediator from a policy engine.
    #[must_use]
    pub fn new(engine: PolicyEngine) -> Self {
        Self { engine }
    }
}

impl ExternalCallMediator for PolicyMediator {
    fn mediate(&mut self, context: &CallContext) -> MediationDecision {
        if context.kwarg_names.len() != context.ifc.kwarg_summaries.len() {
            return MediationDecision::Deny {
                reason:
                    "invalid call context: kwarg_names length does not match kwarg_summaries length"
                        .to_owned(),
            };
        }
        let policy_input = translate_call_context_to_policy_input(context);
        let policy_decision = self.engine.evaluate_external_call(&policy_input);
        translate_policy_decision_to_mediation_decision(policy_decision, context)
    }
}

/// Translate `CallContext` into policy-engine input.
fn translate_call_context_to_policy_input(context: &CallContext) -> ExternalCallPolicyInput {
    ExternalCallPolicyInput {
        tool_name: context.function_name.clone(),
        call_kind: translate_external_call_kind(context.kind),
        aggregate_summary: context.ifc.aggregate_summary.clone(),
        arg_summaries: context.ifc.arg_summaries.clone(),
        kwarg_summaries: context
            .kwarg_names
            .iter()
            .zip(&context.ifc.kwarg_summaries)
            .map(
                |(name, (key_summary, value_summary))| KeywordArgumentSummary {
                    name: name.clone(),
                    key_summary: key_summary.clone(),
                    value_summary: value_summary.clone(),
                },
            )
            .collect(),
        caller_authority: context.caller_authority.clone(),
        control_context: context.ifc.control_context.clone(),
    }
}

/// Translate Monty's `ExternalCallKind` to the policy crate's copy.
fn translate_external_call_kind(kind: ExternalCallKind) -> zamburak_policy::ExternalCallKind {
    use zamburak_policy::ExternalCallKind as PolicyCallKind;
    match kind {
        ExternalCallKind::Function => PolicyCallKind::Function,
        ExternalCallKind::Os => PolicyCallKind::Os,
        ExternalCallKind::Method => PolicyCallKind::Method,
    }
}

/// Translate policy-engine decision into `MediationDecision`.
fn translate_policy_decision_to_mediation_decision(
    decision: ExternalCallPolicyDecision,
    context: &CallContext,
) -> MediationDecision {
    match decision {
        ExternalCallPolicyDecision::Allow(_explanation) => MediationDecision::Allow,
        ExternalCallPolicyDecision::Deny(explanation) => MediationDecision::Deny {
            reason: explanation.summary,
        },
        ExternalCallPolicyDecision::RequireConfirmation(explanation) => {
            MediationDecision::RequireConfirmation {
                request: ConfirmationContext {
                    description: explanation.summary,
                    call: context.clone(),
                },
            }
        }
    }
}
