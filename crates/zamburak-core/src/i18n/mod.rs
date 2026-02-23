//! Injection-first localization contracts for user-facing diagnostics.
//!
//! All localized rendering goes through an injected [`Localizer`] trait.
//! The runtime provides [`NoOpLocalizer`] for deterministic fallback when
//! no localization backend is configured. Host applications own locale
//! negotiation and loader lifecycle.

mod localizer;

pub use localizer::{LocalizationArgs, LocalizedDiagnostic, Localizer, NoOpLocalizer};
