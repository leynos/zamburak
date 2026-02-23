//! Core localizer trait and no-op fallback implementation.
//!
//! The `Localizer` trait defines the injection-first localization contract
//! from `docs/zamburak-design-document.md` and ADR-002. At the
//! design-contract phase, `LocalizationArgs` uses `String` values
//! rather than `FluentValue` to avoid coupling `zamburak-core` to
//! `fluent-bundle`. The `FluentLocalizerAdapter` (Phase 6, Task 6.1.2)
//! converts `String` values to `FluentValue` internally.

use std::collections::HashMap;

/// Arguments for message interpolation.
///
/// Keys are argument names; values are formatted string
/// representations. Using `String` rather than `FluentValue` at the
/// design-contract phase to avoid coupling core contracts to the
/// Fluent crate before it is needed.
pub type LocalizationArgs<'a> = HashMap<&'a str, String>;

/// Injection-first localizer contract.
///
/// All user-facing localized text must be rendered through an
/// implementation of this trait. The host application owns the concrete
/// implementation and its lifecycle. Zamburak must not read process
/// locale environment variables directly and must not maintain mutable
/// process-wide localization singletons.
///
/// # Examples
///
/// ```rust
/// use zamburak_core::i18n::{Localizer, NoOpLocalizer};
///
/// let localizer = NoOpLocalizer;
/// assert_eq!(
///     localizer.message("msg-id", None, "fallback"),
///     "fallback",
/// );
/// ```
pub trait Localizer: Send + Sync {
    /// Look up a message by identifier with optional interpolation
    /// arguments.
    ///
    /// Returns `None` when no translation is available for the
    /// requested identifier.
    fn lookup(&self, id: &str, args: Option<&LocalizationArgs<'_>>) -> Option<String>;

    /// Look up a message, falling back to caller-provided text when
    /// lookup returns `None`.
    ///
    /// The default implementation delegates to [`Localizer::lookup`]
    /// and substitutes the fallback when the result is absent.
    fn message(&self, id: &str, args: Option<&LocalizationArgs<'_>>, fallback: &str) -> String {
        self.lookup(id, args).unwrap_or_else(|| fallback.to_owned())
    }
}

/// No-op localizer that always returns caller-provided fallback text.
///
/// Used when no localization backend is configured. Provides
/// deterministic behaviour in tests and minimal deployments.
///
/// # Examples
///
/// ```rust
/// use zamburak_core::i18n::{Localizer, NoOpLocalizer};
///
/// let localizer = NoOpLocalizer;
/// assert!(localizer.lookup("any-key", None).is_none());
/// assert_eq!(
///     localizer.message("key", None, "caller fallback"),
///     "caller fallback",
/// );
/// ```
pub struct NoOpLocalizer;

impl Localizer for NoOpLocalizer {
    fn lookup(&self, _id: &str, _args: Option<&LocalizationArgs<'_>>) -> Option<String> {
        None
    }
}

/// Trait for types that can render a localized user-facing diagnostic.
///
/// Implementations produce localized text through an injected localizer
/// rather than embedding locale decisions in `Display`. Core domain
/// `Display` output remains stable English for machine-stable logs and
/// test assertions.
///
/// # Examples
///
/// ```rust
/// use zamburak_core::i18n::{LocalizedDiagnostic, Localizer, NoOpLocalizer};
///
/// struct DenyDiagnostic;
///
/// impl LocalizedDiagnostic for DenyDiagnostic {
///     fn render_localized(&self, localizer: &dyn Localizer) -> String {
///         localizer.message("zamburak-deny", None, "Access denied")
///     }
/// }
///
/// let diag = DenyDiagnostic;
/// let localizer = NoOpLocalizer;
/// assert_eq!(diag.render_localized(&localizer), "Access denied");
/// ```
pub trait LocalizedDiagnostic {
    /// Render this diagnostic using the provided localizer.
    fn render_localized(&self, localizer: &dyn Localizer) -> String;
}

#[cfg(test)]
mod tests {
    use super::{LocalizedDiagnostic, Localizer, NoOpLocalizer};

    #[test]
    fn noop_localizer_lookup_returns_none() {
        let localizer = NoOpLocalizer;
        assert!(localizer.lookup("any-key", None).is_none());
    }

    #[test]
    fn noop_localizer_message_returns_fallback() {
        let localizer = NoOpLocalizer;
        let result = localizer.message("missing-key", None, "fallback text");
        assert_eq!(result, "fallback text");
    }

    #[test]
    fn localizer_trait_is_object_safe() {
        let localizer: &dyn Localizer = &NoOpLocalizer;
        assert_eq!(localizer.message("key", None, "fb"), "fb");
    }

    /// Compile-time proof that `NoOpLocalizer` is `Send + Sync`.
    const _: () = {
        const fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NoOpLocalizer>();
    };

    struct TestDiag;

    impl LocalizedDiagnostic for TestDiag {
        fn render_localized(&self, localizer: &dyn Localizer) -> String {
            localizer.message("test-id", None, "test-fallback")
        }
    }

    #[test]
    fn localized_diagnostic_renders_through_localizer() {
        let diag = TestDiag;
        let localizer = NoOpLocalizer;
        assert_eq!(diag.render_localized(&localizer), "test-fallback");
    }
}
