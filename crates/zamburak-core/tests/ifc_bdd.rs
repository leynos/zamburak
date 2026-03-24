//! BDD scenarios for IFC dependency graph and propagation.

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use zamburak_core::control_context::ExecutionContextSummary;
use zamburak_core::propagation::{PropagationMode, propagate_labels};
use zamburak_core::summary::{DependencySummary, compute_summary};
use zamburak_core::{
    AuthoritySet, DataLabel, DataLabels, DependencyGraph, GraphBudgets, IntegrityLabel, ValueId,
    ValueLabels,
};

// ---------------------------------------------------------------------------
// World state
// ---------------------------------------------------------------------------

#[derive(Default)]
struct IfcWorld {
    budgets: Option<GraphBudgets>,
    graph: Option<DependencyGraph>,
    summary: Option<DependencySummary>,
    propagation_result: Option<DependencySummary>,
    context: Option<ExecutionContextSummary>,
}

#[fixture]
fn world() -> IfcWorld {
    IfcWorld::default()
}

// ---------------------------------------------------------------------------
// Given steps
// ---------------------------------------------------------------------------

#[given("a default dependency graph")]
fn given_default_graph(world: &mut IfcWorld) {
    let budgets = GraphBudgets::default();
    world.graph = Some(DependencyGraph::new(budgets));
    world.budgets = Some(budgets);
}

#[given("a dependency graph with max_closure_steps 0")]
fn given_zero_budget_graph(world: &mut IfcWorld) {
    // Build the graph with default budgets so edges can be inserted
    // (cycle detection needs adequate closure steps). The tight budget
    // is stored separately and used only during compute_summary.
    world.graph = Some(DependencyGraph::new(GraphBudgets::default()));
    world.budgets = Some(GraphBudgets {
        max_closure_steps: 0,
        ..GraphBudgets::default()
    });
}

#[given("a fresh execution context")]
fn given_fresh_context(world: &mut IfcWorld) {
    world.context = Some(ExecutionContextSummary::new());
}

#[given("an untrusted PII-labelled value 1")]
fn given_untrusted_pii_value(world: &mut IfcWorld) {
    let graph = world.graph.as_mut().expect("graph must exist");
    graph
        .insert_value(
            ValueId::new(1),
            ValueLabels {
                integrity: IntegrityLabel::Untrusted,
                confidentiality: DataLabels::from_iter([DataLabel::Pii]),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert value 1");
}

#[given("a verified clean value 2 depending on value 1")]
fn given_verified_value_depending_on_untrusted(world: &mut IfcWorld) {
    let graph = world.graph.as_mut().expect("graph must exist");
    graph
        .insert_value(
            ValueId::new(2),
            ValueLabels {
                integrity: IntegrityLabel::Verified,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert value 2");
    graph
        .add_dependency(ValueId::new(2), ValueId::new(1))
        .expect("add dependency 2->1");
}

#[given("a trusted clean value 3")]
fn given_trusted_clean_value(world: &mut IfcWorld) {
    let graph = world.graph.as_mut().expect("graph must exist");
    graph
        .insert_value(
            ValueId::new(3),
            ValueLabels {
                integrity: IntegrityLabel::Trusted,
                confidentiality: DataLabels::new(),
                authority: AuthoritySet::new(),
            },
        )
        .expect("insert value 3");
}

#[given("an untrusted condition pushed into the context")]
fn given_untrusted_condition_in_context(world: &mut IfcWorld) {
    let ctx = world.context.as_mut().expect("context must exist");
    let condition = DependencySummary {
        integrity_join: IntegrityLabel::Untrusted,
        confidentiality_join: DataLabels::from_iter([DataLabel::Pii]),
        authority_join: AuthoritySet::new(),
        origin_count: 1,
        truncated: false,
    };
    ctx.push_condition(ValueId::new(99), &condition);
}

// ---------------------------------------------------------------------------
// When steps
// ---------------------------------------------------------------------------

#[when("the summary is computed for value 2")]
fn when_compute_summary_for_value_2(world: &mut IfcWorld) {
    let graph = world.graph.as_ref().expect("graph must exist");
    let budgets = world.budgets.as_ref().expect("budgets must exist");
    let summary =
        compute_summary(graph, &ValueId::new(2), budgets).expect("compute_summary should succeed");
    world.summary = Some(summary);
}

#[when("labels are propagated in Normal mode for value 3")]
fn when_propagate_normal(world: &mut IfcWorld) {
    let graph = world.graph.as_ref().expect("graph must exist");
    let budgets = world.budgets.as_ref().expect("budgets must exist");
    let summary =
        compute_summary(graph, &ValueId::new(3), budgets).expect("compute_summary should succeed");
    let ctx = world.context.as_ref().expect("context must exist");
    let result = propagate_labels(PropagationMode::Normal, &[summary], ctx);
    world.propagation_result = result;
}

#[when("labels are propagated in Strict mode for value 3")]
fn when_propagate_strict(world: &mut IfcWorld) {
    let graph = world.graph.as_ref().expect("graph must exist");
    let budgets = world.budgets.as_ref().expect("budgets must exist");
    let summary =
        compute_summary(graph, &ValueId::new(3), budgets).expect("compute_summary should succeed");
    let ctx = world.context.as_ref().expect("context must exist");
    let result = propagate_labels(PropagationMode::Strict, &[summary], ctx);
    world.propagation_result = result;
}

// ---------------------------------------------------------------------------
// Then steps
// ---------------------------------------------------------------------------

#[then("the summary integrity is Untrusted")]
fn then_summary_integrity_untrusted(world: &mut IfcWorld) {
    let summary = world.summary.as_ref().expect("summary must exist");
    assert_eq!(summary.integrity_join, IntegrityLabel::Untrusted);
}

#[then("the summary confidentiality includes PII")]
fn then_summary_includes_pii(world: &mut IfcWorld) {
    let summary = world.summary.as_ref().expect("summary must exist");
    assert!(summary.confidentiality_join.contains(&DataLabel::Pii));
}

#[then("the summary is not truncated")]
fn then_summary_not_truncated(world: &mut IfcWorld) {
    let summary = world.summary.as_ref().expect("summary must exist");
    assert!(!summary.truncated);
}

#[then("the summary equals unknown_top")]
fn then_summary_is_unknown_top(world: &mut IfcWorld) {
    let summary = world.summary.as_ref().expect("summary must exist");
    assert_eq!(*summary, DependencySummary::unknown_top());
}

#[then("the propagation result integrity is Trusted")]
fn then_propagation_integrity_trusted(world: &mut IfcWorld) {
    let result = world
        .propagation_result
        .as_ref()
        .expect("result must exist");
    assert_eq!(result.integrity_join, IntegrityLabel::Trusted);
}

#[then("the propagation result does not include PII")]
fn then_propagation_no_pii(world: &mut IfcWorld) {
    let result = world
        .propagation_result
        .as_ref()
        .expect("result must exist");
    assert!(!result.confidentiality_join.contains(&DataLabel::Pii));
}

#[then("the propagation result integrity is Untrusted")]
fn then_propagation_integrity_untrusted(world: &mut IfcWorld) {
    let result = world
        .propagation_result
        .as_ref()
        .expect("result must exist");
    assert_eq!(result.integrity_join, IntegrityLabel::Untrusted);
}

#[then("the propagation result includes PII")]
fn then_propagation_includes_pii(world: &mut IfcWorld) {
    let result = world
        .propagation_result
        .as_ref()
        .expect("result must exist");
    assert!(result.confidentiality_join.contains(&DataLabel::Pii));
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

#[scenario(
    path = "tests/features/ifc_propagation.feature",
    name = "Transitive label propagation through dependency chain"
)]
fn happy_path_transitive_label_propagation(world: IfcWorld) {
    drop(world);
}

#[scenario(
    path = "tests/features/ifc_propagation.feature",
    name = "Budget exhaustion yields conservative summary"
)]
fn budget_exhaustion_yields_conservative_summary(world: IfcWorld) {
    drop(world);
}

#[scenario(
    path = "tests/features/ifc_propagation.feature",
    name = "Normal mode excludes control context"
)]
fn normal_mode_excludes_control_context(world: IfcWorld) {
    drop(world);
}

#[scenario(
    path = "tests/features/ifc_propagation.feature",
    name = "Strict mode includes control context"
)]
fn strict_mode_includes_control_context(world: IfcWorld) {
    drop(world);
}
