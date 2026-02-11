# ADR-002: Adopt Fluent localization via injected localizer interfaces

Date: 2026-02-11

Status: Proposed

## Context and problem statement

Zamburak is designed to be consumed as a library. As the project grows, it will
need to emit user-facing diagnostics for policy denials, verification failures,
capability checks, tool gating, and audit-oriented messages.

Internationalization (i18n) and localization (l10n) in a library have a
different shape from application-localized interfaces:

- the library should not choose the process locale,
- the library should not own global localization state when that state can
  conflict with host application behaviour, and
- the host application should remain free to compose multiple localizable
  libraries into one coherent localization context.

To select an approach for Zamburak, this decision reviews two related codebases
and one design note:

- `../../rstest-bdd/`
- `../../ortho-config/`
- `docs/localizable-rust-libraries-with-fluent.md`

## Review findings

### `rstest-bdd` approach

`rstest-bdd` embeds Fluent assets and provides helper functions for message
lookup (`message`, `message_with_args`), language selection, and global loader
installation.

Strengths:

- embedded resources are straightforward to ship,
- English fallback is explicit and reliable,
- thread-local overrides support test isolation patterns.

Trade-offs for a library-first design:

- global mutable loader state can make composition awkward in applications that
  already own localization state,
- formatting APIs implicitly read ambient state instead of requiring explicit
  context, and
- integration with an application's existing language negotiation pipeline is
  possible, but not the default shape.

### `ortho-config` approach

`ortho-config` uses a `Localizer` trait plus a `FluentLocalizer` implementation
that layers consumer resources over default embedded bundles.

Strengths:

- no global mutable localization state,
- dependency injection keeps locale control in the caller,
- clean fallback behaviour (`lookup` then caller fallback),
- explicit handling of formatting issues while preserving usable output.

Trade-offs:

- slightly more integration work for consumers than ambient-global lookup,
- message rendering requires explicit localizer plumbing through call sites.

### Guidance from localizable library design note

`docs/localizable-rust-libraries-with-fluent.md` recommends that the
application remains the authority for locale selection and loader lifecycle,
and that libraries expose embedded resources plus APIs that accept injected
localization context.

## Decision drivers

- library composability across mixed crates and host applications,
- deterministic behaviour in tests and CI, regardless of host locale settings,
- compatibility with Fluent tooling and resource embedding patterns,
- explicit fallback semantics when translation keys or arguments are missing,
- thread-safe operation without cross-thread locale bleed.

## Considered options

### Option 1: Global loader managed by Zamburak

Adopt a singleton + override model similar to `rstest-bdd`.

Pros:

- minimal API friction for internal call sites,
- easy direct lookup from anywhere in the crate.

Cons:

- weaker library composability,
- harder integration with host-managed localization stacks,
- potential hidden coupling through ambient mutable state.

### Option 2: Pure dependency injection via `Localizer`

Require all localized rendering to receive a localizer explicitly.

Pros:

- maximally explicit control surface,
- easy embedding into host application architecture.

Cons:

- may introduce extra plumbing in low-level error formatting paths.

### Option 3: Injection-first architecture with Fluent adapters (chosen)

Adopt `Localizer` injection as the primary contract, while shipping first-party
Fluent adapters and embedded assets to reduce consumer boilerplate.

Pros:

- preserves host ownership of locale negotiation,
- keeps APIs explicit and testable,
- provides an ergonomic default path for consumers using Fluent.

Cons:

- requires disciplined API design to avoid reintroducing ambient global state.

## Decision outcome

Zamburak will adopt Option 3.

The project will implement an injection-first localization architecture with:

- a core `Localizer` trait used by all localized rendering,
- a `NoOpLocalizer` fallback implementation,
- embedded Fluent assets exposed publicly for host composition,
- helper adapters for Fluent loaders that are created and owned by the host
  application, not by Zamburak.

## Proposed architecture

### Public API shape

- `zamburak::i18n::Localizer`:
  lookup interface for message IDs and optional formatting arguments.
- `zamburak::i18n::NoOpLocalizer`:
  intentionally declines all lookups so caller-provided fallback text is used.
- `zamburak::i18n::Localizations`:
  embedded Fluent assets (`RustEmbed`) for host loading into a shared Fluent
  loader.
- Fluent adapter module:
  wrapper(s) that implement `Localizer` using a host-supplied
  `FluentLanguageLoader`.

### Message rendering contract

- Core domain errors keep stable, deterministic `Display` text in English for
  logs and machine-stable assertions.
- Localized user-facing text is produced through explicit APIs that accept
  `&dyn Localizer` (for example, `render_localized` methods on diagnostic
  structures).
- Rendering must always accept a caller-provided fallback string.

### Fallback order

For Fluent-backed lookups, resolution order is:

1. consumer-provided catalogue entries,
2. Zamburak bundled locale entries (requested locale),
3. Zamburak bundled `en-US` entries,
4. caller-provided fallback text.

If formatting fails due to missing arguments or malformed patterns, the error
is reported through structured diagnostics (for example, tracing events), and
resolution continues to the next fallback stage.

### Locale ownership and negotiation

- Zamburak does not read `LANG`, `LC_ALL`, or `LC_MESSAGES` directly.
- Zamburak does not maintain global locale state.
- Host applications perform language negotiation, select locale preferences, and
  load resources into their own loader.
- Zamburak exposes helpers to load its embedded assets into a caller-supplied
  loader, but these helpers do not store global state.

### API sketch

```rust,no_run
use fluent_bundle::FluentValue;
use std::collections::HashMap;

pub type LocalizationArgs<'a> = HashMap<&'a str, FluentValue<'a>>;

pub trait Localizer: Send + Sync {
    fn lookup(&self, id: &str, args: Option<&LocalizationArgs<'_>>) -> Option<String>;

    fn message(&self, id: &str, args: Option<&LocalizationArgs<'_>>, fallback: &str) -> String {
        self.lookup(id, args).unwrap_or_else(|| fallback.to_owned())
    }
}

pub trait LocalizedDiagnostic {
    fn render_localized(&self, localizer: &dyn Localizer) -> String;
}
```

## Consequences

Positive:

- supports composition with host applications that already localize other
  libraries,
- avoids hidden mutable global state in a security-critical library,
- keeps localization decisions explicit in call sites and tests,
- enables progressive rollout: untranslated strings still render via fallback.

Negative:

- additional API plumbing is required wherever user-facing text is rendered,
- contributors must maintain message IDs and fallback copy discipline.

## Implementation plan

1. Introduce `zamburak::i18n` with `Localizer`, `NoOpLocalizer`,
   `LocalizationArgs`, and `Localizations`.
2. Add optional Fluent adapter module behind a `fluent` Cargo feature.
3. Define message ID conventions (for example, `zamburak-<domain>-<detail>`)
   and add bundled `en-US` catalogues.
4. Refactor user-facing diagnostics to expose localized rendering methods that
   accept `&dyn Localizer`.
5. Add tests for:
   - fallback behaviour,
   - argument interpolation,
   - missing key handling,
   - locale layering precedence.
6. Document consumer integration patterns in the user guide with examples for:
   - host-owned loader setup,
   - loading Zamburak embedded assets,
   - rendering localized diagnostics.

## Acceptance criteria

- No global mutable localization loader exists in Zamburak.
- All localized rendering paths accept explicit localizer context.
- English fallback exists for every public diagnostic message ID.
- Host applications can load Zamburak Fluent assets into their own loader and
  render localized diagnostics without replacing their localization strategy.
