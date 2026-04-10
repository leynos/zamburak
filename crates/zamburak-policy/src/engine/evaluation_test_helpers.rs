//! Test helper functions for external-call policy evaluation tests.

use zamburak_core::control_context::ExecutionContextSummary;
use zamburak_core::trust::{AuthoritySet, IntegrityLabel};
use zamburak_core::{DataLabel, DataLabels, DependencySummary};

use crate::engine::evaluation::{ExternalCallKind, ExternalCallPolicyInput};
use crate::engine::PolicyEngine;

pub(super) fn minimal_policy_with_tools(tools_yaml: &str) -> PolicyEngine {
    let yaml = format!(
        concat!(
            "schema_version: 1\n",
            "policy_name: test_policy\n",
            "default_action: Deny\n",
            "strict_mode: true\n",
            "budgets:\n",
            "  max_values: 100\n",
            "  max_parents_per_value: 10\n",
            "  max_closure_steps: 100\n",
            "  max_witness_depth: 10\n",
            "tools:\n",
            "{}"
        ),
        tools_yaml
    );
    PolicyEngine::from_yaml_str(&yaml).expect("valid test policy")
}

pub(super) fn make_input(
    tool_name: &str,
    arg_summaries: Vec<DependencySummary>,
    control_context: ExecutionContextSummary,
) -> ExternalCallPolicyInput {
    make_input_full(
        tool_name,
        arg_summaries,
        vec![],
        AuthoritySet::full(),
        control_context,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "test helper covers all input fields"
)]
pub(super) fn make_input_full(
    tool_name: &str,
    arg_summaries: Vec<DependencySummary>,
    kwarg_summaries: Vec<(DependencySummary, DependencySummary)>,
    caller_authority: AuthoritySet,
    control_context: ExecutionContextSummary,
) -> ExternalCallPolicyInput {
    ExternalCallPolicyInput {
        tool_name: tool_name.to_owned(),
        call_kind: ExternalCallKind::Function,
        aggregate_summary: DependencySummary::unknown_top(),
        arg_summaries,
        kwarg_summaries,
        caller_authority,
        control_context,
    }
}

pub(super) fn summary_with_integrity(integrity: IntegrityLabel) -> DependencySummary {
    DependencySummary {
        integrity_join: integrity,
        confidentiality_join: DataLabels::new(),
        authority_join: Default::default(),
        origin_count: 1,
        truncated: false,
    }
}

pub(super) fn summary_with_confidentiality(labels: &[DataLabel]) -> DependencySummary {
    DependencySummary {
        integrity_join: IntegrityLabel::Trusted,
        confidentiality_join: DataLabels::from_iter(labels.iter().copied()),
        authority_join: Default::default(),
        origin_count: 1,
        truncated: false,
    }
}

pub(super) fn control_context_with_untrusted_pc() -> ExecutionContextSummary {
    use zamburak_core::value_id::ValueId;
    let mut ctx = ExecutionContextSummary::new();
    ctx.push_condition(
        ValueId::new(1),
        &summary_with_integrity(IntegrityLabel::Untrusted),
    );
    ctx
}
