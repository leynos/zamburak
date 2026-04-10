//! Test helper functions for external-call policy evaluation tests.

use zamburak_core::control_context::ExecutionContextSummary;
use zamburak_core::trust::{AuthoritySet, IntegrityLabel};
use zamburak_core::{DataLabel, DataLabels, DependencySummary};

use crate::engine::PolicyEngine;
use crate::engine::evaluation::{
    ExternalCallKind, ExternalCallPolicyInput, KeywordArgumentSummary,
};

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

pub(super) struct PolicyInputBuilder<'a> {
    tool_name: &'a str,
    arg_summaries: Vec<DependencySummary>,
    kwarg_summaries: Vec<KeywordArgumentSummary>,
    caller_authority: AuthoritySet,
    control_context: ExecutionContextSummary,
}

impl<'a> PolicyInputBuilder<'a> {
    pub(super) fn new(tool_name: &'a str) -> Self {
        Self {
            tool_name,
            arg_summaries: vec![],
            kwarg_summaries: vec![],
            caller_authority: AuthoritySet::full(),
            control_context: ExecutionContextSummary::new(),
        }
    }

    pub(super) fn arg_summaries(mut self, v: Vec<DependencySummary>) -> Self {
        self.arg_summaries = v;
        self
    }

    pub(super) fn kwarg_summaries(mut self, v: Vec<KeywordArgumentSummary>) -> Self {
        self.kwarg_summaries = v;
        self
    }

    pub(super) fn caller_authority(mut self, a: AuthoritySet) -> Self {
        self.caller_authority = a;
        self
    }

    pub(super) fn control_context(mut self, c: ExecutionContextSummary) -> Self {
        self.control_context = c;
        self
    }

    pub(super) fn build(self) -> ExternalCallPolicyInput {
        ExternalCallPolicyInput {
            tool_name: self.tool_name.to_owned(),
            call_kind: ExternalCallKind::Function,
            aggregate_summary: DependencySummary::unknown_top(),
            arg_summaries: self.arg_summaries,
            kwarg_summaries: self.kwarg_summaries,
            caller_authority: self.caller_authority,
            control_context: self.control_context,
        }
    }
}

pub(super) fn make_input(
    tool_name: &str,
    arg_summaries: Vec<DependencySummary>,
    control_context: ExecutionContextSummary,
) -> ExternalCallPolicyInput {
    PolicyInputBuilder::new(tool_name)
        .arg_summaries(arg_summaries)
        .control_context(control_context)
        .build()
}

pub(super) fn named_kwarg_summary(
    name: &str,
    key_summary: DependencySummary,
    value_summary: DependencySummary,
) -> KeywordArgumentSummary {
    KeywordArgumentSummary {
        name: name.to_owned(),
        key_summary,
        value_summary,
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
