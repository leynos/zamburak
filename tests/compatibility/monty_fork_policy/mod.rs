//! Behavioural tests for full-monty fork policy guardrails.

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use zamburak::monty_fork_policy_contract::{MontyForkViolation, evaluate_patch_text};

#[derive(Default)]
struct MontyForkPolicyWorld {
    patch_text: String,
    violations: Vec<MontyForkViolation>,
}

#[fixture]
fn world() -> MontyForkPolicyWorld {
    MontyForkPolicyWorld::default()
}

#[given("a generic observer hook patch")]
fn generic_observer_patch(world: &mut MontyForkPolicyWorld) {
    concat!(
        "diff --git a/src/run.rs b/src/run.rs\n",
        "+++ b/src/run.rs\n",
        "+pub enum RuntimeEvent { ValueCreated }\n",
    )
    .clone_into(&mut world.patch_text);
}

#[given("a patch with Zamburak semantics in public API")]
fn zamburak_public_api_patch(world: &mut MontyForkPolicyWorld) {
    concat!(
        "diff --git a/src/run.rs b/src/run.rs\n",
        "+++ b/src/run.rs\n",
        "+pub struct ZamburakObserver;\n",
    )
    .clone_into(&mut world.patch_text);
}

#[given("a patch with forbidden term in non-public code")]
fn forbidden_non_public_patch(world: &mut MontyForkPolicyWorld) {
    concat!(
        "diff --git a/src/run.rs b/src/run.rs\n",
        "+++ b/src/run.rs\n",
        "+let policy_context = 1_u8;\n",
    )
    .clone_into(&mut world.patch_text);
}

#[given("a patch with mixed public and non-public forbidden terms")]
fn mixed_patch(world: &mut MontyForkPolicyWorld) {
    concat!(
        "diff --git a/src/run.rs b/src/run.rs\n",
        "+++ b/src/run.rs\n",
        "+let zamburak_context = 1_u8;\n",
        "+pub enum PolicyEvent { Started }\n",
    )
    .clone_into(&mut world.patch_text);
}

#[when("the fork policy checker evaluates the patch")]
fn evaluate_patch(world: &mut MontyForkPolicyWorld) {
    world.violations = evaluate_patch_text(&world.patch_text);
}

#[then("the violation count is {expected_count:u8}")]
fn assert_violation_count(world: &MontyForkPolicyWorld, expected_count: u8) {
    assert_eq!(world.violations.len(), usize::from(expected_count));
}

#[then("a violation token includes {expected_token}")]
fn assert_violation_token(world: &MontyForkPolicyWorld, expected_token: String) {
    let token = expected_token.trim_matches('"').to_ascii_lowercase();
    let has_token = world
        .violations
        .iter()
        .any(|violation| violation.matched_token.contains(&token));

    assert!(has_token, "expected token `{token}` in one violation");
}

#[scenario(
    path = "tests/compatibility/features/monty_fork_policy.feature",
    name = "Generic observer API additions are accepted"
)]
fn generic_observer_api_is_accepted(world: MontyForkPolicyWorld) {
    assert!(world.violations.is_empty());
}

#[scenario(
    path = "tests/compatibility/features/monty_fork_policy.feature",
    name = "Zamburak semantics in public API are rejected"
)]
fn zamburak_semantics_are_rejected(world: MontyForkPolicyWorld) {
    assert_eq!(world.violations.len(), 1);
}

#[scenario(
    path = "tests/compatibility/features/monty_fork_policy.feature",
    name = "Forbidden terms in non-public additions are ignored"
)]
fn forbidden_terms_in_non_public_are_ignored(world: MontyForkPolicyWorld) {
    assert!(world.violations.is_empty());
}

#[scenario(
    path = "tests/compatibility/features/monty_fork_policy.feature",
    name = "Mixed patch rejects only public API semantic violations"
)]
fn mixed_patch_rejects_only_public_api_violations(world: MontyForkPolicyWorld) {
    assert_eq!(world.violations.len(), 1);
}
