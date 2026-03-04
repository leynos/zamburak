# Zamburak user's guide

## Policy loader contract

The runtime policy loader enforces canonical policy schema v1 and supports an
explicit legacy migration path.

- accepted without migration: `schema_version: 1`,
- accepted with explicit migration: `schema_version: 0` (migrated to v1),
- rejected fail-closed: unknown schema versions and unknown schema families.

Unknown schema versions are never defaulted, partially loaded, or heuristically
migrated.

## Runtime API

Use `PolicyEngine` or `PolicyDefinition` from `zamburak-policy`:

- `PolicyEngine::from_yaml_str(...)`,
- `PolicyEngine::from_json_str(...)`,
- `PolicyDefinition::from_yaml_str(...)`,
- `PolicyDefinition::from_json_str(...)`.

These entrypoints return canonical policy objects and hide migration evidence.

For auditable migration evidence, use:

- `PolicyEngine::from_yaml_str_with_migration_audit(...)`,
- `PolicyEngine::from_json_str_with_migration_audit(...)`,
- `PolicyDefinition::from_yaml_str_with_migration_audit(...)`,
- `PolicyDefinition::from_json_str_with_migration_audit(...)`.

Audit outcomes include:

- source and target schema versions,
- source and target canonicalized SHA-256 document hashes,
- ordered per-step transform records (`policy_schema_v0_to_v1`).

## Source checkout requirement for `full-monty`

This repository vendors the Track A runtime substrate as a Git submodule at
`third_party/full-monty/`.

Consumers building Zamburak from source must initialize and update submodules
before building:

```sh
git submodule update --init --recursive
```

Maintainers syncing Track A fork state can use the repository-local workflow:

```sh
make monty-sync
```

Track A changes are constrained by [Monty fork policy](monty-fork-policy.md).
Zamburak semantics are prohibited in fork API surface, and pull requests that
violate that policy are rejected by automated review checks.

## Track A runtime IDs

`full-monty` exposes stable, host-only runtime IDs for suspendable execution
payloads.

- `RunProgress::FunctionCall` and `RunProgress::OsCall` include
  `arg_runtime_ids: Vec<RuntimeValueId>` and
  `kwarg_runtime_ids: Vec<(RuntimeValueId, RuntimeValueId)>`.
- `ReplProgress::FunctionCall` and `ReplProgress::OsCall` include the same
  runtime-ID field types.
- Both progress enums expose `runtime_ids()` for read-only access to the ID
  slices without destructuring the enum payload, returning
  `(&[RuntimeValueId], &[(RuntimeValueId, RuntimeValueId)])`.

Runtime IDs are opaque host metadata and carry no policy meaning. They remain
stable across `start()` or `resume()` boundaries and survive `dump()` or
`load()` round trips for run-progress payloads.

## Track A runtime observer events

`full-monty` exposes a generic runtime observer surface for host-side
instrumentation.

- install an observer with `RuntimeObserverHandle::new(...)`,
- start run execution with `MontyRun::start_with_observer(...)`,
- start REPL snippet execution with `MontyRepl::start_with_observer(...)` or
  `MontyRepl::start_no_print_with_observer(...)`,
- inspect canonical event classes:
  - `ValueCreated`,
  - `OpResult`,
  - `ExternalCallRequested`,
  - `ExternalCallReturned`,
  - `ControlCondition`.

Observer payloads are runtime-generic and ID-centric. They intentionally do not
carry Zamburak policy decisions or governance semantics.

Baseline semantics are preserved in both of these modes:

- no observer installed (`start(...)` and existing entrypoints),
- explicit no-op observer (`RuntimeObserverHandle::new(NoopRuntimeObserver)`).

This allows hosts to adopt instrumentation incrementally without changing
execution outcomes.

## Example: canonical policy (schema v1)

```yaml
schema_version: 1
policy_name: personal_assistant_default
default_action: Deny
strict_mode: true
budgets:
  max_values: 100000
  max_parents_per_value: 64
  max_closure_steps: 10000
  max_witness_depth: 32
tools: []
```

## Example: migrated legacy policy (schema v0)

```yaml
schema_version: 0
policy_name: personal_assistant_default
default_action: Deny
strict_mode: true
budgets:
  max_values: 100000
  max_parents_per_value: 64
  max_closure_steps: 10000
  max_witness_depth: 32
tools:
  - name: send_email
    side_effect: ExternalWrite
    authority: [EmailSendCap]
    args:
      - name: body
        forbid_confidentiality: [AUTH_SECRET]
    context:
      deny_if_pc_integrity_contains: [Untrusted]
    default_decision: RequireConfirmation
```

The loader migrates this policy to canonical v1 and exposes migration audit
metadata through the audit-bearing API variants.

## Example: rejected policy

A policy document using an unsupported schema version such as
`schema_version: 2` is rejected with
`PolicyLoadError::UnsupportedSchemaVersion`.

This fail-closed behaviour is intentional and required by the security
contracts.

## Authority token lifecycle

Authority tokens are stateful security objects managed through `zamburak-core`.
Lifecycle operations are:

### Minting

Only host-trusted issuers may mint tokens. Each minted token encodes a subject,
capability, scope, and expiry. Minting from untrusted issuers is rejected with
`AuthorityLifecycleError::UntrustedMinter`.

```rust
use zamburak_core::{
    AuthorityToken, AuthorityIssuer, IssuerTrust, MintRequest,
    AuthorityTokenId, AuthoritySubject, AuthorityCapability,
    AuthorityScope, ScopeResource, TokenTimestamp,
};

let token = AuthorityToken::mint(MintRequest {
    token_id: AuthorityTokenId::try_from("tok-1")?,
    issuer: AuthorityIssuer::try_from("policy-host")?,
    issuer_trust: IssuerTrust::HostTrusted,
    subject: AuthoritySubject::try_from("assistant")?,
    capability: AuthorityCapability::try_from("EmailSendCap")?,
    scope: AuthorityScope::new(vec![
        ScopeResource::try_from("send_email")?,
    ])?,
    issued_at: TokenTimestamp::new(100),
    expires_at: TokenTimestamp::new(500),
})?;
# Ok::<(), zamburak_core::AuthorityLifecycleError>(())
```

### Delegation

Delegated tokens must narrow both scope (strict subset) and lifetime (strict
subset). Parent lineage is retained for audit. Delegation from revoked or
expired parents is rejected before scope checks run. The delegation start time
must also be on or after the parent issuance time.

```rust
use zamburak_core::{
    AuthorityToken, AuthorityIssuer, AuthorityTokenId, AuthoritySubject,
    AuthorityCapability, AuthorityScope, ScopeResource, IssuerTrust,
    MintRequest, DelegationRequest, RevocationIndex, TokenTimestamp,
};

// Mint a parent token with two scope resources.
let parent_token = AuthorityToken::mint(MintRequest {
    token_id: AuthorityTokenId::try_from("tok-parent")?,
    issuer: AuthorityIssuer::try_from("policy-host")?,
    issuer_trust: IssuerTrust::HostTrusted,
    subject: AuthoritySubject::try_from("assistant")?,
    capability: AuthorityCapability::try_from("EmailSendCap")?,
    scope: AuthorityScope::new(vec![
        ScopeResource::try_from("send_email")?,
        ScopeResource::try_from("draft_email")?,
    ])?,
    issued_at: TokenTimestamp::new(100),
    expires_at: TokenTimestamp::new(500),
})?;

// Delegate with strictly narrowed scope and lifetime.
let revocation_index = RevocationIndex::default();
let child = AuthorityToken::delegate(
    &parent_token,
    DelegationRequest {
        token_id: AuthorityTokenId::try_from("tok-child")?,
        delegated_by: AuthorityIssuer::try_from("policy-host")?,
        subject: AuthoritySubject::try_from("assistant")?,
        scope: AuthorityScope::new(vec![
            ScopeResource::try_from("send_email")?,
        ])?,
        delegated_at: TokenTimestamp::new(200),
        expires_at: TokenTimestamp::new(400),
    },
    &revocation_index,
)?;
# Ok::<(), zamburak_core::AuthorityLifecycleError>(())
```

### Revocation

The host manages a `RevocationIndex`. Revoked tokens are stripped at
policy-evaluation boundaries.

```rust
let mut revocation_index = RevocationIndex::default();
revocation_index.revoke(token.token_id().clone());
```

### Policy boundary validation

`PolicyEngine::validate_authority_tokens` partitions tokens into effective and
invalid sets at a given evaluation time. Revoked, expired, and pre-issuance
tokens (evaluation time before `issued_at`) are stripped from the effective set.

```rust
let validation = engine.validate_authority_tokens(
    &tokens,
    &revocation_index,
    TokenTimestamp::new(now),
);
let effective = validation.effective_tokens();
let invalid = validation.invalid_tokens();
```

### Snapshot restore

`revalidate_tokens_on_restore` applies the same validation as policy-boundary
checks. On restore, any previously valid tokens that have since been revoked or
expired are conservatively stripped.

### Error handling

All lifecycle operations return `Result<_, AuthorityLifecycleError>`:

- `EmptyField` — a required text field was empty,
- `InvalidTokenLifetime` — issued_at is not before expires_at,
- `UntrustedMinter` — issuer trust level is not `HostTrusted`,
- `DelegationScopeNotStrictSubset` — delegated scope is not a proper subset,
- `DelegationLifetimeNotStrictSubset` — delegated expiry is not before parent
  expiry,
- `InvalidParentToken` — parent is revoked or expired at delegation time,
- `DelegationBeforeParentIssuance` — delegation start is before parent
  issuance.

All timestamps are injected via `TokenTimestamp` to ensure deterministic
evaluation without wall-clock dependencies.

## Consumer integration: localized diagnostics

Zamburak uses injection-first localization. The host application owns locale
negotiation and loader lifecycle; Zamburak never reads process locale
environment variables or maintains mutable global state.

### Host-owned loader setup

Create a `FluentLanguageLoader` in the host application and pass it through a
`FluentLocalizerAdapter` that implements the `Localizer` trait:

```rust
use zamburak_core::i18n::{FluentLocalizerAdapter, Localizer};
use i18n_embed::fluent::FluentLanguageLoader;

let loader: FluentLanguageLoader = /* host-configured loader */;
let localizer: Box<dyn Localizer> = Box::new(
    FluentLocalizerAdapter::new(loader),
);
```

When no localization backend is configured, use the deterministic fallback:

```rust
use zamburak_core::i18n::NoOpLocalizer;

let localizer = NoOpLocalizer;
```

### Loading Zamburak embedded assets

Zamburak publishes embedded `.ftl` translation assets via `Localizations`. Load
them into the host-owned loader so Zamburak messages are available:

```rust
use zamburak_core::i18n::Localizations;

loader.load_assets(&Localizations, &requested_locales);
```

Resolution order is:

1. host application catalogue entries,
2. Zamburak bundled entries for the requested locale,
3. Zamburak bundled `en-US` entries,
4. caller-provided fallback text.

### Rendering localized diagnostics

Zamburak diagnostics expose a `render_localized` method that accepts an
injected `&dyn Localizer` plus caller fallback copy:

```rust
let message = diagnostic.render_localized(&localizer, "fallback text");
```

Formatting failures and missing translations fall through the resolution chain
and always produce deterministic output. See
`adr-002-localization-and-internationalization-with-fluent.md` for the full
design rationale.
