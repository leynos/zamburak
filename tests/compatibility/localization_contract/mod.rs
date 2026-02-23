//! Behavioural tests validating localization contract conformance.

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use zamburak_core::i18n::{LocalizationArgs, LocalizedDiagnostic, Localizer, NoOpLocalizer};

/// Tracks the result of a localizer lookup attempt.
enum LookupOutcome {
    /// Lookup returned a translated string.
    #[expect(
        dead_code,
        reason = "variant exists for future localizer impls that return translations"
    )]
    Present(String),
    /// Lookup returned `None`.
    Absent,
}

#[derive(Default)]
struct LocalizationWorld {
    localizer: Option<NoOpLocalizer>,
    trait_object: Option<Box<dyn Localizer>>,
    second_localizer: Option<NoOpLocalizer>,
    lookup_outcome: Option<LookupOutcome>,
    message_result: Option<String>,
    diagnostic_result: Option<String>,
    second_message_result: Option<String>,
    diagnostic: Option<TestDiagnostic>,
}

/// Test diagnostic fixture implementing `LocalizedDiagnostic`.
struct TestDiagnostic {
    message_id: String,
    fallback: String,
}

impl LocalizedDiagnostic for TestDiagnostic {
    fn render_localized(&self, localizer: &dyn Localizer) -> String {
        localizer.message(&self.message_id, None, &self.fallback)
    }
}

#[fixture]
fn world() -> LocalizationWorld {
    LocalizationWorld::default()
}

// ── Given steps ─────────────────────────────────────────────────────

#[given("a no-op localizer")]
fn given_noop_localizer(world: &mut LocalizationWorld) {
    world.localizer = Some(NoOpLocalizer);
}

#[given("a no-op localizer behind a trait object")]
fn given_noop_trait_object(world: &mut LocalizationWorld) {
    world.trait_object = Some(Box::new(NoOpLocalizer));
}

#[given("a diagnostic with message id {message_id} and fallback {fallback}")]
fn given_diagnostic(world: &mut LocalizationWorld, message_id: String, fallback: String) {
    world.diagnostic = Some(TestDiagnostic {
        message_id: message_id.trim_matches('"').to_owned(),
        fallback: fallback.trim_matches('"').to_owned(),
    });
}

#[given("two independent no-op localizer instances")]
fn given_two_localizers(world: &mut LocalizationWorld) {
    world.localizer = Some(NoOpLocalizer);
    world.second_localizer = Some(NoOpLocalizer);
}

// ── When steps ──────────────────────────────────────────────────────

#[when("a message is requested with id {id} and fallback {fallback}")]
fn when_message_requested(world: &mut LocalizationWorld, id: String, fallback: String) {
    let Some(localizer) = world.localizer.as_ref() else {
        panic!("localizer must be set before requesting a message");
    };
    let result = localizer.message(id.trim_matches('"'), None, fallback.trim_matches('"'));
    world.message_result = Some(result);
}

#[when("a message with interpolation arguments is requested for id {id} and fallback {fallback}")]
fn when_message_requested_with_args(world: &mut LocalizationWorld, id: String, fallback: String) {
    let Some(localizer) = world.localizer.as_ref() else {
        panic!("localizer must be set before requesting a message");
    };
    let mut args = LocalizationArgs::new();
    args.insert("name", "World".to_owned());
    args.insert("count", "42".to_owned());
    let result = localizer.message(
        id.trim_matches('"'),
        Some(&args),
        fallback.trim_matches('"'),
    );
    world.message_result = Some(result);
}

#[when("a lookup is performed for id {id}")]
fn when_lookup_performed(world: &mut LocalizationWorld, id: String) {
    let Some(localizer) = world.localizer.as_ref() else {
        panic!("localizer must be set before performing lookup");
    };
    let result = localizer.lookup(id.trim_matches('"'), None);
    world.lookup_outcome = Some(result.map_or(LookupOutcome::Absent, LookupOutcome::Present));
}

#[when("a message is requested through the trait object with fallback {fallback}")]
fn when_trait_object_message(world: &mut LocalizationWorld, fallback: String) {
    let Some(localizer) = world.trait_object.as_ref() else {
        panic!("trait object must be set before requesting a message");
    };
    let result = localizer.message("any-key", None, fallback.trim_matches('"'));
    world.message_result = Some(result);
}

#[when("the diagnostic is rendered with the localizer")]
fn when_diagnostic_rendered(world: &mut LocalizationWorld) {
    let Some(localizer) = world.localizer.as_ref() else {
        panic!("localizer must be set before rendering diagnostic");
    };
    let Some(diagnostic) = world.diagnostic.as_ref() else {
        panic!("diagnostic must be set before rendering");
    };
    world.diagnostic_result = Some(diagnostic.render_localized(localizer));
}

#[when("messages are requested from both localizers independently")]
fn when_both_localizers_used(world: &mut LocalizationWorld) {
    let Some(first) = world.localizer.as_ref() else {
        panic!("first localizer must be set");
    };
    let Some(second) = world.second_localizer.as_ref() else {
        panic!("second localizer must be set");
    };
    world.message_result = Some(first.message("key-a", None, "fallback-a"));
    world.second_message_result = Some(second.message("key-b", None, "fallback-b"));
}

// ── Then steps ──────────────────────────────────────────────────────

#[then("the rendered message is {expected}")]
fn then_message_is(world: &LocalizationWorld, expected: String) {
    let Some(result) = world.message_result.as_ref() else {
        panic!("message step must run before assertion");
    };
    assert_eq!(result, expected.trim_matches('"'));
}

#[then("the lookup result is absent")]
fn then_lookup_absent(world: &LocalizationWorld) {
    let Some(LookupOutcome::Absent) = world.lookup_outcome.as_ref() else {
        panic!("lookup step must run before assertion and must return absent");
    };
}

#[then("the rendered diagnostic text is {expected}")]
fn then_diagnostic_text_is(world: &LocalizationWorld, expected: String) {
    let Some(result) = world.diagnostic_result.as_ref() else {
        panic!("diagnostic rendering step must run before assertion");
    };
    assert_eq!(result, expected.trim_matches('"'));
}

#[then("both produce deterministic fallback results without shared state")]
fn then_both_deterministic(world: &LocalizationWorld) {
    let Some(first) = world.message_result.as_ref() else {
        panic!("first message result must be set");
    };
    let Some(second) = world.second_message_result.as_ref() else {
        panic!("second message result must be set");
    };
    assert_eq!(first, "fallback-a");
    assert_eq!(second, "fallback-b");
}

// ── Scenario bindings ───────────────────────────────────────────────

#[scenario(
    path = "tests/compatibility/features/localization_contract.feature",
    name = "Explicit localizer injection with no-op fallback"
)]
fn explicit_localizer(world: LocalizationWorld) {
    assert!(world.message_result.is_some());
}

#[scenario(
    path = "tests/compatibility/features/localization_contract.feature",
    name = "Localizer lookup returns None for no-op localizer"
)]
fn lookup_returns_none(world: LocalizationWorld) {
    assert!(world.lookup_outcome.is_some());
}

#[scenario(
    path = "tests/compatibility/features/localization_contract.feature",
    name = "Localizer trait is object-safe for dynamic dispatch"
)]
fn trait_object_safe(world: LocalizationWorld) {
    assert!(world.message_result.is_some());
}

#[scenario(
    path = "tests/compatibility/features/localization_contract.feature",
    name = "Localized diagnostic renders through injected localizer"
)]
fn diagnostic_renders(world: LocalizationWorld) {
    assert!(world.diagnostic_result.is_some());
}

#[scenario(
    path = "tests/compatibility/features/localization_contract.feature",
    name = "Message request with non-empty interpolation arguments"
)]
fn interpolation_args(world: LocalizationWorld) {
    assert!(world.message_result.is_some());
}

#[scenario(
    path = "tests/compatibility/features/localization_contract.feature",
    name = "No global mutable localizer state exists"
)]
fn no_global_state(world: LocalizationWorld) {
    assert!(world.message_result.is_some());
    assert!(world.second_message_result.is_some());
}
