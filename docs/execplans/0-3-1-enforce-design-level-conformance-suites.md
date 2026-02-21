# Enforce design-level conformance suites before Phase 1 (Task 0.3.1)

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

`PLANS.md` is not present in this repository at draft time, so this document is
the governing execution plan for this task.

## Purpose / big picture

Implement roadmap Task 0.3.1 from `docs/roadmap.md`: enforce design-level
conformance suites before Phase 1 build work begins.

Phase 0 of the Zamburak roadmap requires all four Phase 1 conformance suites to
pass before Phase 1 build work starts. Two of the four suites already exist and
pass: policy-schema-contract (test filter `policy_schema_bdd::`) and
authority-lifecycle (test filter `authority_lifecycle_bdd::`). Two are missing:
LLM sink enforcement (test filter `llm_sink_enforcement::`) and localization
contract (test filter `localization_contract::`). The phase-gate target in
`.github/phase-gate-target.txt` currently reads `phase0`.

After this change a user can observe success by running `make phase-gate` and
seeing all four Phase 1 suites listed, executed, and passing. Running
`make test` confirms the new BDD suites exercise both the LLM sink enforcement
and localization contract types.

This task introduces the *design-contract types* (traits, structs, enums, and
contract functions) that express the design-document API shapes and writes BDD
tests proving those shapes exist and behave correctly. Actual runtime logic
(policy evaluation, Fluent loading, real LLM adapter dispatch) comes later in
Phase 1, Phase 4, and Phase 6.

## Constraints

- Implement to these requirement signposts: `docs/zamburak-design-document.md`
  section "Design-level acceptance criteria before phase 1 build-out",
  `docs/verification-targets.md` rows "Policy schema loader", "LLM sink
  enforcement", and "Authority lifecycle",
  `docs/zamburak-engineering-standards.md` section "Review and
  change-management standards".
- Respect dependency ordering: Tasks 0.1.1, 0.1.3, and 0.2.2 are complete; do
  not regress their coverage paths.
- In scope: schema, sink enforcement, authority lifecycle, and localization
  contract conformance test gating.
- Out of scope: Phase 1 feature implementation. Do not implement runtime policy
  evaluation, real LLM adapter dispatch, or Fluent integration. Only introduce
  *contract types* (traits, structs, enums, and contract functions) that
  express the documented API shapes.
- Phase-gate test filter matching: new test modules must produce fully
  qualified test names containing the substrings `llm_sink_enforcement::` and
  `localization_contract::` to match `src/phase_gate_contract.rs` (lines 109
  and 119). This means module names must be `llm_sink_enforcement` and
  `localization_contract` (without `_bdd` suffix), because the gate uses
  `name.contains(suite.test_filter)` matching.
- Add unit tests and BDD behavioural tests using `rstest-bdd` v0.5.0 covering
  happy and unhappy paths and relevant edge cases.
- Record design decisions in `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` if any library-consumer-visible behaviour or API
  changes; the localization section already documents the planned API shape, so
  no changes are expected unless the contract types deviate from what is
  documented.
- Mark roadmap Task 0.3.1 as done in `docs/roadmap.md` only after all quality
  and documentation gates pass.
- Required completion gates: `make check-fmt`, `make lint`, `make test`.
- Because this task updates Markdown documentation, also run:
  `make markdownlint`, `make nixie`, and `make fmt`.
- Clippy lint restrictions from `Cargo.toml` apply: `unwrap_used`,
  `expect_used`, `indexing_slicing`, `missing_docs`, `needless_pass_by_value`,
  and `cognitive_complexity` are all denied. Production code must use `Result`
  and `Option` combinators. Test code may use `panic!` with descriptive
  messages.
- Functions must not exceed 400 lines per `AGENTS.md`.

## Tolerances (exception triggers)

- Scope tolerance: if implementation requires edits in more than 16 files or
  1200 net changed lines, stop and escalate with a split proposal.
- Interface tolerance: if fulfilling the task requires changing existing public
  library API signatures (in `zamburak-core` or `zamburak-policy`), stop and
  escalate with compatibility options.
- Dependency tolerance: if a new third-party dependency is required (for
  example `fluent-bundle` in `zamburak-core`), stop and escalate before adding
  it.
- Behavioural-test tolerance: if `rstest-bdd` cannot express a required
  scenario after two concrete attempts, stop and document why before using a
  non-BDD fallback.
- Iteration tolerance: if required gates still fail after three focused fix
  loops, stop and report failing suites with root-cause hypotheses.
- Ambiguity tolerance: if the design document and ADR-002 conflict on API shape
  (for example `LocalizationArgs` type), stop and present options with
  trade-offs.

## Risks

- Risk: `LocalizationArgs` type coupling. ADR-002 sketches
  `HashMap<&str, FluentValue<'a>>` which would add `fluent-bundle` as a
  dependency to `zamburak-core` at design-contract phase. Severity: medium.
  Likelihood: high (certain if ADR-002 is followed literally). Mitigation: use
  `HashMap<&str, String>` at design-contract phase, deferring Fluent coupling
  to Phase 6 Task 6.1.2 when `FluentLocalizerAdapter` is implemented. Record
  this decision in the design document.

- Risk: phase-gate filter mismatch. The filters `llm_sink_enforcement::` and
  `localization_contract::` from `src/phase_gate_contract.rs` must match test
  names produced by `cargo test -- --list`. If test module names include a
  `_bdd` suffix, the filter will not match. Severity: high. Likelihood: medium.
  Mitigation: name modules `llm_sink_enforcement` and `localization_contract`
  without `_bdd` suffix, and verify match in `cargo test -- --list` output
  before advancing the phase-gate target.

- Risk: Clippy strictness on new contract types. `unwrap_used`, `expect_used`,
  `missing_docs`, and `needless_pass_by_value` are denied. Severity: low.
  Likelihood: medium. Mitigation: contract functions return values directly
  without `Result` unwrapping, all public items have doc comments, and struct
  types are passed by reference.

## Progress

- [x] (2026-02-21) Reviewed requirements and wrote ExecPlan.
- [x] (2026-02-21) Implemented localization contract types in `zamburak-core`.
- [x] (2026-02-21) Implemented LLM sink enforcement contract types in
  `zamburak-policy`.
- [x] (2026-02-21) Created localization contract BDD suite (5 scenarios).
- [x] (2026-02-21) Created LLM sink enforcement BDD suite (6 scenarios).
- [x] (2026-02-21) Ran code quality gates (`check-fmt`, `lint`, `test`).
- [x] (2026-02-21) Advanced phase-gate target to `phase1`; `make phase-gate`
  passes with 4 suite(s) checked.
- [x] (2026-02-21) Recorded `LocalizationArgs` type decision in design
  document. Marked Task 0.3.1 done in roadmap.
- [x] (2026-02-21) Run all documentation quality gates (`markdownlint`,
  `nixie`, `fmt`).

## Surprises & discoveries

- Observation: Clippy `option_option` lint denied `Option<Option<String>>` in
  the `LocalizationWorld` test struct. Evidence: `make lint` failed with
  `error[clippy::option_option]`. Impact: replaced
  `lookup_result: Option<Option<String>>` with a dedicated `LookupOutcome` enum
  (`Absent` variant) in the localization contract BDD step definitions. Pattern
  matches cleanly and avoids the nested `Option`.

## Decision log

- Decision: use `HashMap<&'a str, String>` for `LocalizationArgs` instead of
  `HashMap<&'a str, FluentValue<'a>>`. Rationale: avoids adding `fluent-bundle`
  as a dependency to `zamburak-core` at the design-contract phase, which is
  consistent with the constraint that Phase 1 feature implementation is out of
  scope. The `FluentLocalizerAdapter` in Phase 6 (Task 6.1.2) converts `String`
  values to `FluentValue` internally. Date/Author: 2026-02-21 / ExecPlan draft.

- Decision: name test modules `llm_sink_enforcement` and
  `localization_contract` (without `_bdd` suffix). Rationale: the phase-gate
  contract filters `llm_sink_enforcement::` and `localization_contract::` use
  substring matching (`name.contains(suite.test_filter)`). Module names with a
  `_bdd` suffix would produce test names like `llm_sink_enforcement_bdd::`
  which do not contain the substring `llm_sink_enforcement::`. Date/Author:
  2026-02-21 / ExecPlan draft.

## Outcomes & retrospective

All four Phase 1 conformance suites pass and phase-gate target is advanced to
`phase1`. The two new suites (localization contract: 5 BDD scenarios + 4 unit
tests; LLM sink enforcement: 6 BDD scenarios + 6 unit tests) exercise
design-contract types without introducing runtime implementation, consistent
with the "out of scope: Phase 1 feature implementation" constraint.

The `LocalizationArgs` type decision (`HashMap<&str, String>` instead of
`FluentValue`) is recorded in the design document for Phase 6 reference.

Lesson learned: Clippy pedantic lints (`option_option`) apply even to test-only
structs; use dedicated enums for type-safe state tracking in BDD worlds.

## Context and orientation

Zamburak is a capability-governed execution environment for agent-authored
Monty programs with policy-driven security enforcement. This repository is a
Rust workspace with two member crates and one root crate:

- `crates/zamburak-core` — core runtime contracts for authority lifecycle
  validation. Source: `crates/zamburak-core/src/lib.rs` re-exports types from
  `crates/zamburak-core/src/authority.rs`. No `i18n` module exists yet.
- `crates/zamburak-policy` — policy schema loading, migrations, and runtime
  policy engine. Source: `crates/zamburak-policy/src/lib.rs` re-exports from
  `engine.rs`, `policy_def.rs`, `migration.rs`, and `load_outcome.rs`. No
  `sink_enforcement` module exists yet.
- Root `zamburak` crate (`src/lib.rs`) — re-exports from `zamburak-policy` and
  exposes `pub mod phase_gate_contract`. The phase-gate contract in
  `src/phase_gate_contract.rs` defines Phase 1 required suites as
  `PHASE1_SUITES` (lines 100–121) with four verification suites and their test
  filters.

Tests are organised as integration test binaries:

- `tests/compatibility/main.rs` — compatibility tests, currently contains
  `mod phase_gate_bdd;` and `mod policy_schema_bdd;`.
- `tests/security/main.rs` — security-focused tests, currently contains
  `mod authority_lifecycle_bdd;` and `mod migration_security;`.
- `tests/test_utils/` — shared test utility modules included via `#[path]`.

BDD tests use `rstest-bdd` v0.5.0 with compile-time validation. The pattern is:
feature files in `tests/<category>/features/<name>.feature`, step definitions
in Rust modules with `#[given]`, `#[when]`, `#[then]` macros, a `World` struct
with `#[derive(Default)]`, and `#[scenario]` bindings that tie feature
scenarios to Rust test functions.

The phase-gate target is stored in `.github/phase-gate-target.txt` (currently
`phase0`). `make phase-gate` evaluates required suites for that target by
listing all tests via
`cargo test --workspace --all-targets --all-features -- --list`, confirming
mandated test filters appear, and executing mandated suites.

## Plan of work

Stage A: introduce localization contract types and tests. This stage creates
the `Localizer` trait, `NoOpLocalizer`, `LocalizedDiagnostic`, and
`LocalizationArgs` type alias in `crates/zamburak-core/src/i18n/`, wires them
into the crate, and adds a BDD feature file with step definitions in
`tests/compatibility/localization_contract/`. The API shape follows the design
document class diagram (lines 768–796 of `docs/zamburak-design-document.md`)
and ADR-002 API sketch (lines 190–207 of
`docs/adr-002-localization-and-internationalization-with-fluent.md`),
substituting `String` for `FluentValue`.

Go/no-go for Stage A: `make test` passes with new localization contract
scenarios visible in `cargo test -- --list` output containing
`localization_contract::`.

Stage B: introduce LLM sink enforcement contract types and tests. This stage
creates the three-point enforcement contract types (`LlmCallPath`,
`SinkPreDispatchRequest`, `SinkPreDispatchDecision`, `TransportGuardCheck`,
`TransportGuardOutcome`, `SinkAuditRecord`) and contract functions
(`evaluate_pre_dispatch`, `evaluate_transport_guard`, `emit_audit_record`) in
`crates/zamburak-policy/src/sink_enforcement.rs`. The contract functions encode
the design-contract minimum: calls without redaction are denied; full budget
and context evaluation is Phase 4. BDD tests go in
`tests/security/llm_sink_enforcement/`.

Go/no-go for Stage B: `make test` passes with new sink enforcement scenarios
visible in `cargo test -- --list` output containing `llm_sink_enforcement::`.

Stage C: advance phase-gate target and verify. Change
`.github/phase-gate-target.txt` from `phase0` to `phase1`. Run
`make phase-gate` and confirm all four suites pass.

Go/no-go for Stage C: `make phase-gate` exits zero and reports all four suites
present and passing.

Stage D: documentation, roadmap closure, and full validation. Record the
`LocalizationArgs` type decision in `docs/zamburak-design-document.md`. Mark
Task 0.3.1 as done in `docs/roadmap.md`. Run all code and documentation gates.

Go/no-go for Stage D: all required gates pass, documentation is synchronised,
and roadmap status is updated.

## Concrete steps

Run all commands from repository root (`/home/user/project`).

Stage A: localization contract.

1. Create `crates/zamburak-core/src/i18n/mod.rs` with module-level doc comment
   and re-exports of `LocalizationArgs`, `LocalizedDiagnostic`, `Localizer`,
   and `NoOpLocalizer`.

2. Create `crates/zamburak-core/src/i18n/localizer.rs` containing:

   - `LocalizationArgs<'a>` type alias: `HashMap<&'a str, String>`.
   - `Localizer` trait (`Send + Sync`) with `lookup(&self, id: &str, args:
     Option<&LocalizationArgs<'_>>) -> Option<String>` and a default
     `message()` method that falls back to caller-provided text.
   - `NoOpLocalizer` struct implementing `Localizer` by always returning
     `None` from `lookup`.
   - `LocalizedDiagnostic` trait with `render_localized(&self, localizer:
     &dyn Localizer) -> String`.
   - Doctests and `#[cfg(test)] mod tests` with unit tests for `NoOpLocalizer`
     fallback, object safety, and `Send + Sync` bounds.

3. Edit `crates/zamburak-core/src/lib.rs` to add `pub mod i18n;`.

4. Create `tests/compatibility/features/localization_contract.feature` with
   five scenarios covering explicit injection fallback, lookup returning
   `None`, object-safe dynamic dispatch, `LocalizedDiagnostic` rendering, and
   no-global-state proof.

5. Create `tests/compatibility/localization_contract/mod.rs` with
   `LocalizationWorld` struct, `TestDiagnostic` implementing
   `LocalizedDiagnostic`, step definitions, and `#[scenario]` bindings. The
   first scenario function is named `explicit_localizer` to produce test name
   `localization_contract::explicit_localizer`.

6. Edit `tests/compatibility/main.rs` to add `mod localization_contract;`.

7. Run quality gates:

       set -o pipefail; make check-fmt 2>&1 | tee /tmp/check-fmt-zamburak-$(git branch --show-current).out
       set -o pipefail; make lint 2>&1 | tee /tmp/lint-zamburak-$(git branch --show-current).out
       set -o pipefail; make test 2>&1 | tee /tmp/test-zamburak-$(git branch --show-current).out

   Verify `localization_contract::` appears in test list:

       cargo test --workspace --all-targets --all-features -- --list 2>&1 | grep localization_contract

Stage B: LLM sink enforcement contract.

1. Create `crates/zamburak-policy/src/sink_enforcement.rs` containing:

   - `LlmCallPath` enum with `Planner` and `Quarantined` variants.
   - `SinkPreDispatchRequest` struct with `execution_id: String`,
     `call_id: String`, `call_path: LlmCallPath`, and
     `redaction_applied: bool`.
   - `SinkPreDispatchDecision` enum with `Allow` and `Deny` variants.
   - `TransportGuardCheck` struct with `execution_id`, `call_id`, and
     `redaction_applied`.
   - `TransportGuardOutcome` enum with `Passed` and `Blocked` variants.
   - `SinkAuditRecord` struct with `execution_id`, `call_id`, `decision`,
     `redaction_applied`, and `call_path`.
   - `evaluate_pre_dispatch(&SinkPreDispatchRequest) ->
     SinkPreDispatchDecision` — allows when `redaction_applied` is `true`,
     denies otherwise.
   - `evaluate_transport_guard(&TransportGuardCheck) ->
     TransportGuardOutcome` — passes when `redaction_applied` is `true`,
     blocks otherwise.
   - `emit_audit_record(&SinkPreDispatchRequest, SinkPreDispatchDecision) ->
     SinkAuditRecord` — constructs record preserving linkage fields.
   - Doctests and `#[cfg(test)] mod tests`.

2. Edit `crates/zamburak-policy/src/lib.rs` to add
   `pub mod sink_enforcement;`.

3. Create `tests/security/features/llm_sink_enforcement.feature` with six
   scenarios covering pre-dispatch allow/deny, transport guard pass/block,
   P-LLM audit linkage, and Q-LLM path tagging.

4. Create `tests/security/llm_sink_enforcement/mod.rs` with
   `SinkEnforcementWorld` struct, step definitions, and `#[scenario]` bindings.
   The first scenario function is named `pre_dispatch` to produce test name
   `llm_sink_enforcement::pre_dispatch`.

5. Edit `tests/security/main.rs` to add `mod llm_sink_enforcement;`.

6. Run quality gates:

        set -o pipefail; make check-fmt 2>&1 | tee /tmp/check-fmt-zamburak-$(git branch --show-current).out
        set -o pipefail; make lint 2>&1 | tee /tmp/lint-zamburak-$(git branch --show-current).out
        set -o pipefail; make test 2>&1 | tee /tmp/test-zamburak-$(git branch --show-current).out

   Verify `llm_sink_enforcement::` appears in test list:

        cargo test --workspace --all-targets --all-features -- --list 2>&1 | grep llm_sink_enforcement

Stage C: advance phase-gate target.

1. Edit `.github/phase-gate-target.txt` to read `phase1` instead of `phase0`.

2. Run phase-gate:

        set -o pipefail; make phase-gate 2>&1 | tee /tmp/phase-gate-zamburak-$(git branch --show-current).out

    Expected output includes all four suites listed, executed, and passing.

Stage D: documentation and roadmap closure.

1. Edit `docs/zamburak-design-document.md` to record the `LocalizationArgs`
   type decision in the localization section.

2. Edit `docs/roadmap.md` line 129 to change `- [ ] Task 0.3.1:` to
   `- [x] Task 0.3.1:`.

3. Run all documentation and code quality gates:

        set -o pipefail; make check-fmt 2>&1 | tee /tmp/check-fmt-zamburak-$(git branch --show-current).out
        set -o pipefail; make lint 2>&1 | tee /tmp/lint-zamburak-$(git branch --show-current).out
        set -o pipefail; make test 2>&1 | tee /tmp/test-zamburak-$(git branch --show-current).out
        set -o pipefail; make markdownlint 2>&1 | tee /tmp/markdownlint-zamburak-$(git branch --show-current).out
        set -o pipefail; make nixie 2>&1 | tee /tmp/nixie-zamburak-$(git branch --show-current).out
        set -o pipefail; make fmt 2>&1 | tee /tmp/fmt-zamburak-$(git branch --show-current).out

## Validation and acceptance

Acceptance criteria for Task 0.3.1:

- LLM sink enforcement conformance suite exists and passes, with BDD scenarios
  covering pre-dispatch allow/deny, transport guard pass/block, and audit
  linkage for both P-LLM and Q-LLM paths.
- Localization contract conformance suite exists and passes, with BDD scenarios
  covering explicit localiser injection, `NoOpLocalizer` fallback, object
  safety, `LocalizedDiagnostic` rendering, and no-global-state proof.
- Phase-gate target reads `phase1` and `make phase-gate` passes with all four
  mandated suites present and green.
- Phase 1 implementation is blocked until all required conformance suites pass.
- Roadmap Task 0.3.1 is marked done.

Required gates:

- Code gates: `make check-fmt`, `make lint`, `make test`.
- Phase gate: `make phase-gate`.
- Docs gates: `make markdownlint`, `make nixie`, `make fmt`.

## Idempotence and recovery

- All new source files and test modules are additive; re-running any stage
  overwrites to the same state.
- Phase-gate evaluation is read-only and safe to rerun.
- If a gate fails, fix the failing condition, rerun that gate, then rerun the
  full gate sequence.
- If advancing the phase-gate target causes unexpected broad merge blocking,
  revert `.github/phase-gate-target.txt` to `phase0` and keep the test assets
  for diagnosis.

## Artefacts and notes

Evidence to capture during implementation:

- test list output from `cargo test --workspace --all-targets --all-features
  -- --list` showing `llm_sink_enforcement::` and `localization_contract::`.
- gate logs in `/tmp/*-zamburak-<branch>.out`.
- `make phase-gate` output showing all four Phase 1 suites passing.

## Interfaces and dependencies

Prescriptive interface definitions for this task:

In `crates/zamburak-core/src/i18n/localizer.rs`:

    pub type LocalizationArgs<'a> = HashMap<&'a str, String>;

    pub trait Localizer: Send + Sync {
        fn lookup(
            &self,
            id: &str,
            args: Option<&LocalizationArgs<'_>>,
        ) -> Option<String>;

        fn message(
            &self,
            id: &str,
            args: Option<&LocalizationArgs<'_>>,
            fallback: &str,
        ) -> String { … }
    }

    pub struct NoOpLocalizer;

    pub trait LocalizedDiagnostic {
        fn render_localized(&self, localizer: &dyn Localizer) -> String;
    }

In `crates/zamburak-policy/src/sink_enforcement.rs`:

    pub enum LlmCallPath { Planner, Quarantined }

    pub struct SinkPreDispatchRequest {
        pub execution_id: String,
        pub call_id: String,
        pub call_path: LlmCallPath,
        pub redaction_applied: bool,
    }

    pub enum SinkPreDispatchDecision { Allow, Deny }

    pub struct TransportGuardCheck {
        pub execution_id: String,
        pub call_id: String,
        pub redaction_applied: bool,
    }

    pub enum TransportGuardOutcome { Passed, Blocked }

    pub struct SinkAuditRecord {
        pub execution_id: String,
        pub call_id: String,
        pub decision: SinkPreDispatchDecision,
        pub redaction_applied: bool,
        pub call_path: LlmCallPath,
    }

    pub fn evaluate_pre_dispatch(request: &SinkPreDispatchRequest)
        -> SinkPreDispatchDecision;
    pub fn evaluate_transport_guard(check: &TransportGuardCheck)
        -> TransportGuardOutcome;
    pub fn emit_audit_record(
        request: &SinkPreDispatchRequest,
        decision: SinkPreDispatchDecision,
    ) -> SinkAuditRecord;

No new external dependencies. Continued use of `rstest` 0.26.1 and `rstest-bdd`
v0.5.0 for tests.
