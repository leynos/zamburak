//! Runtime policy evaluation for governed external-call requests.
//!
//! This module implements deterministic, fail-closed policy decisions for
//! external-function and OS-call boundaries in governed execution. The
//! evaluation engine consumes IFC summaries and control-context snapshots
//! from the runtime and matches them against loaded policy rules.

use zamburak_core::DependencySummary;
use zamburak_core::control_context::ExecutionContextSummary;
use zamburak_core::trust::IntegrityLabel;

use crate::engine::PolicyEngine;

/// External-call classification used for policy diagnostics.
///
/// This is a policy-layer-owned copy of the Monty runtime's call-kind
/// discriminator, allowing the policy crate to remain independent of
/// the interpreter implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalCallKind {
    /// External function call.
    Function,
    /// OS-level system call.
    Os,
    /// Method call on an external object.
    Method,
}

/// Input for external-call policy evaluation.
///
/// This type is policy-layer-owned and does not depend on Monty runtime
/// internals, enabling independent testing and versioning of the evaluation
/// contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalCallPolicyInput {
    /// Tool name used for policy lookup.
    pub tool_name: String,
    /// External-call classification.
    pub call_kind: ExternalCallKind,
    /// Aggregate dependency summary for the whole call.
    pub aggregate_summary: DependencySummary,
    /// Per-positional-argument dependency summaries.
    pub arg_summaries: Vec<DependencySummary>,
    /// Per-keyword `(key, value)` dependency summaries.
    pub kwarg_summaries: Vec<(DependencySummary, DependencySummary)>,
    /// Control-context snapshot at the call boundary.
    pub control_context: ExecutionContextSummary,
}

/// Decision outcome for an evaluated external-call request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExternalCallPolicyDecision {
    /// Allow the external call to proceed.
    Allow(PolicyDecisionExplanation),
    /// Deny the external call before side-effect execution.
    Deny(PolicyDecisionExplanation),
    /// Require interactive confirmation before proceeding.
    RequireConfirmation(PolicyDecisionExplanation),
}

/// Explanation metadata attached to a policy decision.
///
/// This type provides deterministic, safely redacted evidence for why a
/// decision was made. Task 0.6.4 keeps this minimal; richer audit-pipeline
/// metadata can be added additively in later tasks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDecisionExplanation {
    /// Human-readable summary of the decision rationale.
    pub summary: String,
}

impl PolicyEngine {
    /// Evaluate an external-call request against the loaded policy.
    ///
    /// Evaluation follows the documented decision order:
    /// 1. Hard deny constraints (context rules),
    /// 2. Authority token requirements,
    /// 3. Verification requirements,
    /// 4. Context constraints,
    /// 5. Confirmation and draft requirements,
    /// 6. Default action.
    ///
    /// Missing tool policy fails closed. Missing or unavailable information
    /// required by a rule fails closed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use zamburak_policy::{PolicyEngine, PolicyLoadError};
    /// # use zamburak_policy::engine::evaluation::{ExternalCallPolicyInput, ExternalCallPolicyDecision};
    /// # use monty::ExternalCallKind;
    /// # use zamburak_core::{DependencySummary, control_context::ExecutionContextSummary};
    /// #
    /// # let policy_yaml = r#"
    /// # schema_version: 1
    /// # policy_name: test_policy
    /// # default_action: Deny
    /// # strict_mode: true
    /// # budgets:
    /// #   max_values: 1
    /// #   max_parents_per_value: 1
    /// #   max_closure_steps: 1
    /// #   max_witness_depth: 1
    /// # tools: []
    /// # "#;
    /// #
    /// let engine = PolicyEngine::from_yaml_str(policy_yaml)?;
    /// let input = ExternalCallPolicyInput {
    ///     tool_name: "unknown_tool".to_owned(),
    ///     call_kind: ExternalCallKind::Function,
    ///     aggregate_summary: DependencySummary::unknown_top(),
    ///     arg_summaries: vec![],
    ///     kwarg_summaries: vec![],
    ///     control_context: ExecutionContextSummary::new(),
    /// };
    ///
    /// let decision = engine.evaluate_external_call(&input);
    /// assert!(matches!(decision, ExternalCallPolicyDecision::Deny(_)));
    /// # Ok::<(), PolicyLoadError>(())
    /// ```
    #[must_use]
    pub fn evaluate_external_call(
        &self,
        input: &ExternalCallPolicyInput,
    ) -> ExternalCallPolicyDecision {
        evaluate_external_call_impl(self.policy_definition(), input)
    }
}

/// Implementation of the external-call policy evaluation algorithm.
fn evaluate_external_call_impl(
    policy: &crate::policy_def::PolicyDefinition,
    input: &ExternalCallPolicyInput,
) -> ExternalCallPolicyDecision {
    use crate::policy_def::PolicyAction;

    // Step 1: Find the matching tool policy.
    let Some(tool_policy) = policy.tools.iter().find(|t| t.tool == input.tool_name) else {
        // Missing tool policy fails closed.
        return ExternalCallPolicyDecision::Deny(PolicyDecisionExplanation {
            summary: format!("no policy defined for tool '{}'", input.tool_name),
        });
    };

    // Step 2: Check hard deny constraints (context rules).
    if let Some(context_rules) = &tool_policy.context_rules {
        for integrity_label_str in &context_rules.deny_if_pc_integrity_contains {
            if matches_integrity_label(input.control_context.pc_integrity(), integrity_label_str) {
                return ExternalCallPolicyDecision::Deny(PolicyDecisionExplanation {
                    summary: format!(
                        "denied by context rule: PC integrity '{}' matched",
                        integrity_label_str
                    ),
                });
            }
        }
    }

    // Step 3: Check authority token requirements (placeholder for now).
    // Task 0.6.4 does not yet expose authority validation in the external-call input.

    // Step 4: Check argument rules.
    for (arg_index, arg_summary) in input.arg_summaries.iter().enumerate() {
        let Some(arg_rule) = tool_policy.arg_rules.get(arg_index) else {
            continue;
        };

        // Check integrity requirement.
        if let Some(required_integrity_str) = &arg_rule.requires_integrity
            && !satisfies_integrity_requirement(
                arg_summary.integrity_join,
                required_integrity_str,
            )
        {
            return ExternalCallPolicyDecision::Deny(PolicyDecisionExplanation {
                summary: format!(
                    "argument '{}' requires integrity '{}', found '{:?}'",
                    arg_rule.arg, required_integrity_str, arg_summary.integrity_join
                ),
            });
        }

        // Check confidentiality deny-list.
        for forbidden_label_str in &arg_rule.forbids_confidentiality {
            if contains_confidentiality_label(
                &arg_summary.confidentiality_join,
                forbidden_label_str,
            ) {
                return ExternalCallPolicyDecision::Deny(PolicyDecisionExplanation {
                    summary: format!(
                        "argument '{}' contains forbidden confidentiality label '{}'",
                        arg_rule.arg, forbidden_label_str
                    ),
                });
            }
        }
    }

    // Step 5: Return the tool's default decision.
    match tool_policy.default_decision {
        PolicyAction::Allow => ExternalCallPolicyDecision::Allow(PolicyDecisionExplanation {
            summary: format!("allowed by default decision for tool '{}'", input.tool_name),
        }),
        PolicyAction::Deny => ExternalCallPolicyDecision::Deny(PolicyDecisionExplanation {
            summary: format!("denied by default decision for tool '{}'", input.tool_name),
        }),
        PolicyAction::RequireConfirmation => {
            ExternalCallPolicyDecision::RequireConfirmation(PolicyDecisionExplanation {
                summary: format!(
                    "confirmation required by default decision for tool '{}'",
                    input.tool_name
                ),
            })
        }
        PolicyAction::RequireDraft => {
            // Task 0.6.4 maps RequireDraft conservatively to RequireConfirmation.
            ExternalCallPolicyDecision::RequireConfirmation(PolicyDecisionExplanation {
                summary: format!(
                    "confirmation required (mapped from RequireDraft) for tool '{}'",
                    input.tool_name
                ),
            })
        }
    }
}

/// Check if the given integrity label matches the policy string.
fn matches_integrity_label(integrity: IntegrityLabel, label_str: &str) -> bool {
    match label_str {
        "Untrusted" => integrity == IntegrityLabel::Untrusted,
        "Trusted" => integrity == IntegrityLabel::Trusted,
        "Verified" => integrity == IntegrityLabel::Verified,
        _ => false,
    }
}

/// Check if the given integrity level satisfies the requirement.
fn satisfies_integrity_requirement(integrity: IntegrityLabel, required_str: &str) -> bool {
    let required = match required_str {
        "Untrusted" => IntegrityLabel::Untrusted,
        "Trusted" => IntegrityLabel::Trusted,
        "Verified" => IntegrityLabel::Verified,
        _ => return false,
    };
    integrity >= required
}

/// Check if the confidentiality label set contains the specified label.
fn contains_confidentiality_label(labels: &zamburak_core::DataLabels, label_str: &str) -> bool {
    use zamburak_core::DataLabel;
    let target_label = match label_str {
        "Pii" => DataLabel::Pii,
        "AuthSecret" => DataLabel::AuthSecret,
        "PrivateEmailBody" => DataLabel::PrivateEmailBody,
        "PaymentInstrument" => DataLabel::PaymentInstrument,
        "InternalPolicyNote" => DataLabel::InternalPolicyNote,
        _ => return false,
    };
    labels.contains(&target_label)
}

#[cfg(test)]
#[path = "evaluation_tests.rs"]
mod tests;
