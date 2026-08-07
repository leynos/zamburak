# Add compatibility, security, and snapshot-governance suites (Task 0.6.5)

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: DRAFT

## Purpose / big picture

Implement roadmap Task 0.6.5 from `docs/roadmap.md`: deliver three new test
suites that together close the Track B integration workstream for governed
execution.

After this change, a library consumer or maintainer must be able to run
`make test` and observe that:

- a permissive YAML policy and the `AllowAllMediator` produce the same
  governed outcomes for an identical Monty program — same terminal state,
  same observer event counts, same captured information-flow control (IFC)
  summaries (permissive-policy parity);
- adversarial Monty programs that attempt tool-call occurrence side channels,
  tool-call count side channels, taint laundering, and exception-text
  injection are blocked or downgraded to `RequireConfirmation` under
  strict-mode IFC propagation, while remaining permitted under a
  documented permissive baseline (strict-mode security regressions);
- the governed IFC state can be serialized to a versioned representation,
  round-tripped, and replayed against the same policy, producing identical
  decisions and explanations to a run that was never suspended (snapshot or
  resume governance continuity).

This is Track B PR B5 from `docs/adr-001-monty-ifc-vm-hooks.md` section
"Track B staged pull requests". It builds on Task 0.6.4 (policy-gated
external calls), Task 0.6.3 (observer-driven IFC), and Task 0.1.3
(authority-token lifecycle including `revalidate_tokens_on_restore`).

## Constraints

- Implement per the requirement signposts:
  - `docs/adr-001-monty-ifc-vm-hooks.md` sections "B3. Durable and versioned
    IFC state" and "B4. Auditable decisions",
  - `docs/zamburak-design-document.md` sections
    "Mechanistic correctness requirements" and "Security regression suite",
  - `docs/verification-targets.md` rows "IFC propagation", "Control context",
    and "Audit pipeline".
- Dependency precondition: Task 0.6.4 must remain marked complete in
  `docs/roadmap.md` before implementation starts.
- In scope:
  - permissive-policy parity tests comparing `PolicyMediator` (with a
    permissive YAML) against `AllowAllMediator`,
  - strict-mode security regression tests for tool-call occurrence side
    channels, tool-call count side channels, taint laundering patterns, and
    exception-text injection,
  - snapshot or resume governance continuity tests proving that the IFC
    governance state survives a versioned round-trip with policy-equivalent
    outcomes,
  - any minimal `zamburak-monty` or `zamburak-core` API additions required to
    serialize the IFC governance state in a versioned form (additive only).
- Out of scope:
  - model-in-loop benchmark expansion (deferred to Task 5.2.2),
  - mutable tool documentation injection coverage (depends on tool
    catalogue, deferred to Phase 3),
  - MCP trust-boundary bypass coverage (deferred to Phase 3),
  - audit chain tamper-evidence implementation (deferred to Phase 5),
  - any change to the policy schema (`policies/schema.json`) or to a
    `PolicyAction` variant.
- Track A purity: no Zamburak-specific governance semantics may be added to
  `third_party/full-monty/`. All snapshot serialization for IFC state must
  live in `zamburak-monty` or `zamburak-core`.
- Public-API discipline: existing `GovernedRunner`, `GovernedRunProgress`,
  `CallContext`, `ExternalCallMediator`, `MediationDecision`, and
  `PolicyEngine` surfaces remain backwards-compatible. New types are
  additive.
- File-size invariant: no Rust source file may exceed 400 lines. Plan for
  new submodules (for example
  `crates/zamburak-monty/src/observer/ifc_state/snapshot.rs`) rather than
  growing existing hotspots:
  - `crates/zamburak-monty/src/observer/ifc_state.rs` is currently the
    primary IFC state owner,
  - `tests/security/` and `tests/integration/` test entrypoints must each
    stay below 400 lines per file; split scenarios into focused step
    modules where needed.
- Validation must include unit tests and behavioural tests using
  `rstest-bdd` v0.5.0. Cover happy paths, unhappy paths, and edge cases.
  Use the conventions in `docs/rstest-bdd-users-guide.md` and follow the
  existing fixture-injection style from
  `tests/integration/governed_ifc_bdd.rs` and
  `tests/security/llm_sink_enforcement/mod.rs`.
- Apply guidance from `docs/rust-testing-with-rstest-fixtures.md`,
  `docs/rust-doctest-dry-guide.md`,
  `docs/reliable-testing-in-rust-via-dependency-injection.md`, and
  `docs/complexity-antipatterns-and-refactoring-strategies.md`. Keep step
  fixtures small and reuse `crates/test-utils` for any shared helper that
  must be visible across test crates.
- Record design decisions in `docs/zamburak-design-document.md` (notably the
  IFC governance snapshot schema and round-trip equivalence contract).
- Update `docs/users-guide.md` for any library-consumer-visible API
  additions (governance snapshot, restore, version constants).
- Update `docs/developers-guide.md` (create if absent) with guidance on
  adding new strict-mode security regressions and writing snapshot
  round-trip tests.
- Mark roadmap Task 0.6.5 done only after all gates are green.
- Required gates: `make check-fmt`, `make lint`, and `make test`. Because
  this task also changes Markdown, run `make fmt`, `make markdownlint`, and
  `make nixie` before finishing. Run all commands with `tee` to a log file
  under `/tmp` per `AGENTS.md`.
- Use en-GB-oxendict spelling in documentation and comments.

## Tolerances

- Scope tolerance: if implementation requires edits in more than 28 files or
  2200 net changed lines, stop, document the split, and escalate with a
  narrower follow-on plan.
- Public-API tolerance: additive exports are allowed. If meeting the
  snapshot continuity contract requires breaking changes to
  `GovernedRunner`, `GovernedRunProgress`, `ZamburakObserver`,
  `CallContext`, or `PolicyEngine`, stop and escalate with explicit
  compatibility options.
- Schema tolerance: this task must not introduce a new policy schema
  version. If a security regression cannot be expressed within schema v1,
  stop and escalate before editing `policies/schema.json` or
  `crates/zamburak-policy/src/policy_def.rs`.
- Dependency tolerance: if a new production dependency is required, stop
  and escalate. The IFC governance snapshot should serialize through types
  already available (`serde` is already pulled in by `monty` and
  `zamburak-policy`); confirm before adding any new crate.
- Iteration tolerance: if `make check-fmt`, `make lint`, or `make test`
  still fail after three focused fix loops, stop and report failing
  commands plus root-cause hypotheses.
- Time tolerance: if any single milestone exceeds eight focused hours, stop
  and report the blocker.

## Risks

- Risk: `zamburak-monty` does not expose a `GovernedRunner` snapshot or
  resume API at the interpreter level. The underlying `monty::MontyRun`
  surface used here (`new`, `start_with_observer`) consumes itself, and
  the `monty::Snapshot::with_snapshot_extension` byte channel from Task
  0.5.3 is reachable only via the full-monty subprocess probe in
  `tests/compatibility/full_monty_snapshot_extension_bdd.rs`. Severity:
  high. Likelihood: high. Mitigation: scope "snapshot governance
  continuity" to a versioned IFC governance snapshot owned by Zamburak,
  not a full interpreter dump. Use `SuspendedCall::resume()` as the
  resume boundary, plus a serialize/deserialize round-trip on the IFC
  governance state, plus authority-token revalidation. Document this
  scoping decision and signpost the future full-interpreter-dump work as
  out of scope for 0.6.5.

- Risk: there is no canonical "permissive policy" YAML in `policies/`. The
  current `policies/default.yaml` declares `default_action:
  RequireConfirmation` for `send_email` and is therefore not permissive.
  Severity: medium. Likelihood: high. Mitigation: introduce a
  test-fixture-only policy YAML (under `tests/test_utils/` or
  `tests/integration/fixtures/`) with `default_action: Allow`, no
  context rules, and no arg rules, used solely by parity scenarios.
  Document why it is not shipped as a default in
  `docs/zamburak-design-document.md`.

- Risk: strict-mode regression scenarios easily over-fit to today's
  control-context behaviour. Severity: medium. Likelihood: medium.
  Mitigation: phrase Then-steps in terms of decision class
  (`Allow`/`Deny`/`RequireConfirmation`) and observable IFC summary
  fields (PC integrity, integrity_join, origin_count), not in terms of
  internal IDs. Provide both a strict-mode and a normal-mode scenario
  for each regression so a refactor that drops control-context
  influence fails loudly.

- Risk: the `RecordingMediator` in `crates/test-utils` only captures
  contexts; it does not let scenarios drive policy outcomes. Severity:
  low. Likelihood: high. Mitigation: extend `crates/test-utils` with a
  small `ScriptedMediator` that pairs context capture with explicit
  decisions, or compose `PolicyMediator` with a YAML fixture per
  scenario. Prefer the latter so parity tests exercise the real policy
  engine path.

- Risk: BDD step ambiguity across the new feature files. Severity: low.
  Likelihood: medium. Mitigation: namespace step phrases (for example
  start them with the suite name: "permissive parity:", "snapshot:",
  "regression:") and keep step modules per-feature so step lookup is
  unambiguous.

- Risk: phase-gate contract drift. The `phase_gate` binary in
  `src/bin/phase_gate.rs` and `src/phase_gate_contract.rs` enumerate
  required suites by id. New 0.6.5 suites are not Phase 1 entry gates,
  so they should not be added to the phase-gate contract — but reviewers
  may expect them to be. Severity: low. Likelihood: low. Mitigation:
  call this out in the Decision Log and in `docs/users-guide.md`.

- Risk: the `full-monty` submodule may be uninitialized in the working
  tree (the current worktree shows the gitlink only). Severity: medium.
  Likelihood: medium. Mitigation: gate any test that requires the
  submodule on its presence and document `git submodule update --init
  --recursive` (or `make monty-sync`) as a prerequisite. Avoid creating
  any test that hard-requires upstream Monty source for the in-scope
  suites.

## Progress

- [ ] (TBD) Reviewed roadmap, ADR, design document, verification targets,
  and current test scaffolding (covered while drafting this plan).
- [ ] (TBD) Drafted this ExecPlan.
- [ ] (TBD) Stage A complete: design contract for IFC governance snapshot
  recorded in `docs/zamburak-design-document.md`; permissive-policy fixture
  drafted; failing scenarios drafted across all three suites.
- [ ] (TBD) Stage B complete: `IfcGovernanceSnapshot` (versioned) and
  `restore_into` round-trip implemented; `ZamburakObserver` exposes a
  governance-snapshot accessor.
- [ ] (TBD) Stage C complete: permissive-policy parity scenarios pass.
- [ ] (TBD) Stage D complete: strict-mode security regressions pass and
  fail under normal mode where applicable.
- [ ] (TBD) Stage E complete: snapshot governance continuity scenarios
  pass, including authority revalidation on restore.
- [ ] (TBD) Stage F complete: documentation updates landed
  (`users-guide.md`, `developers-guide.md`, `zamburak-design-document.md`).
- [ ] (TBD) Stage G complete: roadmap entry marked done; `make fmt`,
  `make check-fmt`, `make lint`, `make test`, `make markdownlint`, and
  `make nixie` all green.

## Surprises & Discoveries

- (To be filled as work proceeds.)

## Decision Log

- Decision: scope "snapshot governance continuity" to a Zamburak-owned
  versioned IFC governance snapshot rather than a full interpreter dump.
  Rationale: `zamburak-monty` does not expose a `MontyRun` dump or
  resume API; a Zamburak-owned snapshot is sufficient to satisfy ADR B3
  ("IFC state serialization is versioned, explicit, and round-trip
  tested") and aligns with the Track A patch budget which forbids
  Zamburak semantics inside `full-monty`. Date/Author: 2026-05-04 /
  Codex.

- Decision: introduce a test-only permissive policy YAML rather than
  shipping a permissive `policies/default.yaml`. Rationale: the
  shipped default must remain conservative; parity tests need only a
  fixture. Date/Author: 2026-05-04 / Codex.

- Decision: do not register the new 0.6.5 suites in
  `src/phase_gate_contract.rs`. Rationale: that contract gates phase
  entry; 0.6.5 closes Track B integration but does not gate Phase 1
  entry. Date/Author: 2026-05-04 / Codex.

## Outcomes & Retrospective

To be completed at task close.

## Context and orientation

A novice should treat this section as the only briefing they need.

### Glossary

- **Track A**: the upstream-friendly substrate inside
  `third_party/full-monty/`. Generic, observable, no Zamburak semantics.
- **Track B**: Zamburak-owned governance built on Track A
  (`crates/zamburak-monty`, `crates/zamburak-core`,
  `crates/zamburak-policy`).
- **Information-flow control (IFC)**: dependency tracking that records
  how each value's data label (integrity, confidentiality) and authority
  flows through the program.
- **Strict mode**: an IFC propagation mode that pulls the program
  counter's integrity (the **control context**) into every effect-call
  decision. Normal mode looks at data only.
- **Permissive policy**: a YAML policy whose `default_action` is `Allow`
  and which has no context or argument restrictions. Used as a baseline
  to prove that `PolicyMediator` and `AllowAllMediator` agree when no
  rule fires.
- **Resume boundary**: the point at which a `SuspendedCall<T>` is
  resumed via `.resume(host_value, print)` after the host supplies the
  external-call result.
- **Governance snapshot**: a versioned, opaque-to-Monty record of the
  Zamburak IFC graph, control context, observer event counts, and
  authority-token registry, sufficient to replay a policy decision and
  produce identical outcomes after deserialization.

### Repository state at the start of this task

Crates touched or read by this task:

- `crates/zamburak-monty/src/lib.rs` exports `GovernedRunner`,
  `GovernedRunProgress`, `SuspendedCall`, `SuspendedNameLookup`,
  `SuspendedResolveFutures`, `CallContext`, `CallIfcContext`,
  `ConfirmationContext`, `ExternalCallMediator`, `AllowAllMediator`,
  `DenyAllMediator`, `PolicyMediator`, `MediationDecision`,
  `EventCounts`, `GovernedIfcConfig`, `IfcValueSeedConfig`, and
  `ZamburakObserver`.
- `crates/zamburak-monty/src/observer/ifc_state.rs` owns
  `IfcRuntimeState` and the `CallIfcTracker` used by snapshots.
- `crates/zamburak-monty/src/external_call/policy_mediator.rs` bridges
  `CallContext` into `PolicyEngine::evaluate_external_call`.
- `crates/zamburak-monty/src/run/flow.rs` and `flow/mediation.rs` route
  `RunProgress::FunctionCall` and `RunProgress::OsCall` through one
  shared mediation gateway.
- `crates/zamburak-core/src/lib.rs` exports `DependencyGraph`,
  `ValueId`, `DataLabels`, `IntegrityLabel`, `AuthoritySet`,
  `GraphBudgets`, `DependencySummary`, propagation modes, control
  context summaries, and `authority::revalidate_tokens_on_restore`.
- `crates/zamburak-policy/src/lib.rs` exports `PolicyEngine`,
  `ExternalCallPolicyInput`, `ExternalCallPolicyDecision`,
  `PolicyDecisionExplanation`, and the canonical schema constants.
- `crates/test-utils/src/governed_run_test_helpers.rs` provides
  `RecordingMediator`. There is no `ScriptedMediator` yet.

Existing tests (suite layout this task must extend, not replace):

- `tests/compatibility/`:
  - `policy_schema_bdd.rs` (and `features/policy_schema.feature`),
  - `full_monty_observer_bdd.rs` (probe wrapper),
  - `full_monty_snapshot_extension_bdd.rs` (probe wrapper),
  - `full_monty_track_a_invariants_bdd.rs`,
  - `phase_gate_bdd.rs`,
  - `localization_contract/`,
  - `monty_fork_policy/`.
- `tests/security/`:
  - `authority_lifecycle_bdd/`,
  - `full_monty_observer_security_bdd.rs`,
  - `ifc_control_context.rs`,
  - `llm_sink_enforcement/`,
  - `migration_security.rs`.
- `tests/integration/`:
  - `governed_run_bdd.rs` (and `features/governed_run.feature`),
  - `governed_ifc_bdd.rs` (and `features/governed_ifc.feature`).

Each suite is wired in its respective `tests/<suite>/main.rs` via `mod`
declarations, and BDD scenarios are bound via `#[scenario(path = ...,
name = ...)]` macros that point at the colocated `.feature` files.

The phase-gate target file `.github/phase-gate-target.txt` is currently
`phase1`. Suites added by this task are not part of the Phase 1 entry
gate.

### What is missing

1. A permissive YAML policy fixture and a parity scenario set that runs
   the same Monty program through `PolicyMediator(permissive.yaml)` and
   `AllowAllMediator` and asserts identical observable behaviour.
2. Adversarial Monty programs and accompanying `rstest-bdd` scenarios
   that target tool-call occurrence side channels, tool-call count side
   channels, taint laundering, and exception-text injection — each in
   both strict and normal mode, with appropriate Then-step contrasts.
3. A versioned IFC governance snapshot type, a serialize and
   deserialize round-trip, an `IfcRuntimeState::restore_from` (or
   equivalent) entrypoint, and metamorphic tests proving that decisions
   and explanations are preserved across the round-trip.
4. Authority-token revalidation hooked through the snapshot path
   (re-using `revalidate_tokens_on_restore`).

## Plan of work

### Stage A — design contract and failing scenarios (no behaviour change)

A.1 Add a permissive policy YAML fixture at
`tests/test_utils/policy-permissive.yaml` (caret-conformant schema v1)
and a small Rust loader helper in `crates/test-utils/src/policy_yaml.rs`
that returns a `PolicyEngine` and a `PolicyMediator`. Reuse the existing
`tests/test_utils/policy_yaml.rs` if appropriate; otherwise extend it.

A.2 In `docs/zamburak-design-document.md`, add a new subsection under
"Verification and evaluation strategy" called
"Snapshot or resume governance continuity". Define the
`IfcGovernanceSnapshot` schema (versioned, additive), the round-trip
contract, and the policy-equivalent decision contract.

A.3 Add the new `.feature` files (failing because steps are not yet
implemented):

- `tests/compatibility/features/permissive_parity.feature` — three
  scenarios: arithmetic happy path, single external call resumed, two
  external calls resumed in sequence. Each Then-step asserts:
  - identical `GovernedRunProgress` terminal class,
  - identical `EventCounts`,
  - identical captured `CallContext.ifc.aggregate_summary` for each
    function name.
- `tests/security/features/strict_mode_regressions.feature` — eight
  scenarios in four pairs (strict-deny, normal-allow) for:
  - tool-call occurrence side channel (effect call inside an
    `Untrusted`-conditioned branch),
  - tool-call count side channel (loop bound derived from an
    `Untrusted` value),
  - taint laundering (a verifier that does not strip integrity),
  - exception-text injection (an exception message that carries
    `Untrusted` data into an effect handler).
- `tests/integration/features/snapshot_governance.feature` — five
  scenarios:
  - serialize then restore an empty governed run; baseline equivalence;
  - snapshot mid-run before resume; restore; resume; assert decision
    parity;
  - snapshot after a denied call; restore; assert denial reason and
    explanation parity;
  - snapshot with an authority token whose revocation status changes
    on restore; assert restore-time revalidation surfaces a deny;
  - snapshot version mismatch is rejected fail-closed.

A.4 Add scaffolding step modules under
`tests/{compatibility,security,integration}/` matching the suite
layout. Each module compiles with `unimplemented!()` Then-steps so the
suites fail loudly until Stage C–E land.

A.5 Validation: `cargo test --workspace --all-targets --all-features
--no-run` builds; new `*_bdd::*` test names appear in `cargo test
--workspace --all-targets --all-features -- --list`.

### Stage B — minimal API additions (zamburak-monty and zamburak-core)

B.1 In `crates/zamburak-monty/src/observer/ifc_state/`, add a new
submodule `snapshot.rs` defining:

```rust
pub const IFC_GOVERNANCE_SNAPSHOT_VERSION: u32 = 1;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct IfcGovernanceSnapshot { /* version, graph, control context,
    counters, requested-call snapshots */ }

impl IfcRuntimeState {
    pub fn governance_snapshot(&self) -> IfcGovernanceSnapshot;
    pub fn restore_governance_snapshot(
        snapshot: IfcGovernanceSnapshot,
        config: GovernedIfcConfig,
    ) -> Result<Self, IfcSnapshotError>;
}
```

`IfcSnapshotError` is a small `thiserror` enum covering version
mismatch and graph budget overflow. Keep the file under 400 lines by
splitting helpers into `snapshot/codec.rs` if needed.

B.2 In `crates/zamburak-monty/src/observer.rs`, expose:

```rust
impl ZamburakObserver {
    pub fn governance_snapshot(&self) -> IfcGovernanceSnapshot;
    pub fn restore_from_snapshot(
        snapshot: IfcGovernanceSnapshot,
        config: GovernedIfcConfig,
    ) -> Result<Self, IfcSnapshotError>;
}
```

Re-export `IfcGovernanceSnapshot`, `IfcSnapshotError`, and
`IFC_GOVERNANCE_SNAPSHOT_VERSION` from
`crates/zamburak-monty/src/lib.rs`.

B.3 In `crates/zamburak-core/src/authority/validation.rs`, add (or
expose if already private) a `restore_with_revalidation(...)` helper
that wraps `revalidate_tokens_on_restore` and returns a typed result
suitable for the snapshot path. Add unit tests covering valid restore,
revoked-on-restore, and expired-on-restore.

B.4 In `crates/test-utils/src/governed_run_test_helpers.rs`, add a
`ScriptedMediator` that returns a deterministic sequence of
`MediationDecision`s (for parity tests where we need to compare two
mediators on identical inputs).

B.5 Validation: `make check-fmt`, `make lint`, `make test` pass for
all unit tests in the changed crates. New types are documented with
rustdoc and have at least one doctest each.

### Stage C — permissive-policy parity suite

C.1 Implement step bindings for
`tests/compatibility/features/permissive_parity.feature` in a new
`tests/compatibility/permissive_parity_bdd.rs` (and a step module
`tests/compatibility/permissive_parity/mod.rs` if scenarios exceed
file-size budget). Wire it via `tests/compatibility/main.rs`.

C.2 The Given step builds two `GovernedRunner`s for the same source:
one with `AllowAllMediator`, one with `PolicyMediator(permissive)`.
Resume any pending external calls with identical host values via a
shared `Vec<(String, MontyObject)>` script.

C.3 The Then steps assert equality of:

- terminal `GovernedRunProgress` discriminant,
- `EventCounts`,
- captured `CallContext.ifc.aggregate_summary` per `function_name`,
- `IfcGovernanceSnapshot` byte-for-byte (after serializing through the
  round-trip helper).

C.4 Validation: parity scenarios pass; failing parity (introduced by
intentionally tightening the permissive policy in a doc example) is
visible in the negative unit test added in `crates/test-utils`.

### Stage D — strict-mode security regression suite

D.1 Implement step bindings for
`tests/security/features/strict_mode_regressions.feature` in
`tests/security/strict_mode_regressions/mod.rs`. Wire via
`tests/security/main.rs`.

D.2 Each adversarial Monty program lives in
`tests/security/strict_mode_regressions/programs/`:

- `occurrence_side_channel.py` — `if untrusted: effect("constant")`.
- `count_side_channel.py` — `for _ in range(untrusted_count):
  effect("constant")`.
- `taint_laundering.py` — value passes through a no-op verifier.
- `exception_text_injection.py` — `try: f(untrusted) except Exception
  as e: effect(str(e))`.

D.3 Then-steps assert:

- under strict mode, the effect call yields `Denied` or
  `RequireConfirmation`, with `PolicyDecisionExplanation` mentioning
  the implicated value's origin count and PC integrity;
- under normal mode (with the same permissive YAML), the effect call
  yields `Allow` (documenting the bypass class so a future strict-mode
  regression makes the contrast obvious).

D.4 Validation: each scenario passes; deliberately weakening
strict-mode propagation in a local branch must cause the strict-mode
side of each pair to fail.

### Stage E — snapshot or resume governance continuity suite

E.1 Implement step bindings for
`tests/integration/features/snapshot_governance.feature` in
`tests/integration/snapshot_governance_bdd.rs` (or a `mod.rs` if
scenarios exceed budget). Wire via `tests/integration/main.rs`.

E.2 The Given step constructs a `GovernedRunner`, runs to a chosen
boundary (start, after first external call, after denied call), then
calls `observer.governance_snapshot()` and serializes via `serde_json`
(reuse the `serde_json` already pulled by `zamburak-policy`).

E.3 The When step deserializes the bytes and calls
`ZamburakObserver::restore_from_snapshot(...)`. For runs that left a
`SuspendedCall`, resume the call on the restored runner via a fresh
`MontyRun` parsed from the same source plus the restored observer.

E.4 The Then steps assert:

- `IfcGovernanceSnapshot` round-trip equality (serialize → deserialize
  → re-serialize → byte-equal),
- post-restore mediator decision equals pre-snapshot decision for the
  same `CallContext` shape,
- `PolicyDecisionExplanation` strings are identical,
- a snapshot with `version != IFC_GOVERNANCE_SNAPSHOT_VERSION` is
  rejected with `IfcSnapshotError::VersionMismatch`,
- authority-token revocation introduced between snapshot and restore
  surfaces as a deny on the next policy evaluation.

E.5 Validation: scenarios pass; the version-mismatch scenario fails
loudly when the constant is reverted.

### Stage F — documentation

F.1 Update `docs/zamburak-design-document.md` (the new subsection from
A.2 plus a paragraph in "Mechanistic correctness requirements" linking
to the new suites and the snapshot equivalence contract).

F.2 Update `docs/users-guide.md` with a new "Governance snapshots"
subsection covering the public types, the version constant, the
serialize/deserialize round-trip, and the authority-revalidation
behaviour. Note that these APIs are additive and do not change
existing `GovernedRunner` flows.

F.3 Create or extend `docs/developers-guide.md` with:

- where to add a new strict-mode security regression (programs
  directory, scenario pair pattern, Then-step decision class assertions),
- how the permissive-policy fixture must remain test-only,
- the snapshot version-bump policy (additive fields preferred; bump
  `IFC_GOVERNANCE_SNAPSHOT_VERSION` only on a non-additive change).

F.4 Mark Task 0.6.5 done in `docs/roadmap.md`.

### Stage G — gates and close

G.1 Run, in this order, capturing each to
`/tmp/$ACTION-zamburak-feat-compat-security-snapshot-suites.out`:
`make fmt`, `make check-fmt`, `make lint`, `make test`,
`make markdownlint`, `make nixie`. Then `mbake validate Makefile` if
the Makefile changed.

G.2 Update `Progress`, `Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` in this ExecPlan.

G.3 Append the revision note (see "Revision note" at the bottom of this
plan).

## Concrete steps

Run all commands from the worktree root
`/home/leynos/.lody/repos/github---leynos---zamburak/worktrees/4b6d11ce-9dd5-4c86-9e5a-5c9577d1b97d`.

Build only:

```sh
cargo test --workspace --all-targets --all-features --no-run \
  | tee /tmp/build-zamburak-feat-compat-security-snapshot-suites.out
```

List discoverable tests (used to confirm new suites are registered):

```sh
cargo test --workspace --all-targets --all-features -- --list \
  | tee /tmp/list-zamburak-feat-compat-security-snapshot-suites.out
```

Expected lines once Stage A scaffolding lands (substring-match):

```plaintext
permissive_parity_bdd::
strict_mode_regressions::
snapshot_governance_bdd::
```

Run gates (sequential, never in parallel — see `AGENTS.md`):

```sh
make check-fmt | tee /tmp/check-fmt-zamburak-feat-compat-security-snapshot-suites.out
make lint      | tee /tmp/lint-zamburak-feat-compat-security-snapshot-suites.out
make test      | tee /tmp/test-zamburak-feat-compat-security-snapshot-suites.out
make markdownlint | tee /tmp/mdlint-zamburak-feat-compat-security-snapshot-suites.out
make nixie     | tee /tmp/nixie-zamburak-feat-compat-security-snapshot-suites.out
```

Each must end with success. `make test` must report no test failures.

## Validation and acceptance

Quality criteria (what "done" means):

- Tests:
  - permissive-policy parity scenarios pass (3 scenarios),
  - strict-mode security regression scenarios pass (8 scenarios across 4
    pairs), with each strict scenario yielding `Denied` or
    `RequireConfirmation` and each normal-mode scenario yielding `Allow`,
  - snapshot governance continuity scenarios pass (5 scenarios),
  - all existing workspace tests still pass.
- Lint and typecheck: `make lint` and `cargo doc --workspace --no-deps`
  succeed with zero warnings.
- Format: `make check-fmt` reports no diffs.
- Markdown: `make markdownlint` and `make nixie` pass.
- Snapshot version: a deliberate edit to
  `IFC_GOVERNANCE_SNAPSHOT_VERSION` causes the version-mismatch scenario
  to fail; restoring it returns the suite to green (manual sanity check).

Quality method (how we check):

- `make check-fmt`, `make lint`, `make test`, `make markdownlint`,
  `make nixie` — all merge-blocking via existing CI workflows.
- Phase-gate behaviour is unchanged; the new suites do not alter
  `.github/phase-gate-target.txt` or `src/phase_gate_contract.rs`.

## Idempotence and recovery

All steps are repeatable. If `make test` partially fails:

- re-run only the failing crate's tests with `cargo test -p <crate>
  --all-targets --all-features` to triage,
- once green, re-run the full `make test` to confirm.

If a snapshot serialization regression locks the bytes to an old layout,
revert the offending edit; the round-trip property test will guide
root-cause analysis.

If the `full-monty` submodule is uninitialized and a probe wrapper test
fails, run `make monty-sync` (or
`git -c url."https://github.com/".insteadOf=git@github.com: submodule
update --init --recursive`) before retrying.

## Artifacts and notes

Anticipated transcripts will be appended to `Outcomes & Retrospective`
once each Stage closes:

- abridged `cargo test --list` output proving each new suite is
  registered,
- final `make test` summary line,
- one-line summary of the new file count, lines added/removed, and
  crates touched.

## Interfaces and dependencies

New public types and functions to be added (additive only):

- `crates/zamburak-monty/src/observer/ifc_state/snapshot.rs`:

```rust
pub const IFC_GOVERNANCE_SNAPSHOT_VERSION: u32 = 1;

#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IfcGovernanceSnapshot {
    pub version: u32,
    /* graph, control context, event counts, requested-call snapshots */
}

#[derive(Debug, thiserror::Error)]
pub enum IfcSnapshotError {
    #[error("snapshot version {found} does not match supported version {expected}")]
    VersionMismatch { expected: u32, found: u32 },
    #[error("snapshot graph exceeds budgets")]
    BudgetOverflow,
}
```

- `crates/zamburak-monty/src/observer.rs`:

```rust
impl ZamburakObserver {
    pub fn governance_snapshot(&self) -> IfcGovernanceSnapshot;
    pub fn restore_from_snapshot(
        snapshot: IfcGovernanceSnapshot,
        config: GovernedIfcConfig,
    ) -> Result<Self, IfcSnapshotError>;
}
```

- `crates/zamburak-core/src/authority/validation.rs` (if not already
  exposed): a small wrapper around `revalidate_tokens_on_restore`
  returning a typed result usable by the snapshot path.

- `crates/test-utils/src/governed_run_test_helpers.rs`:

```rust
pub struct ScriptedMediator {
    pub script: std::sync::Mutex<std::collections::VecDeque<MediationDecision>>,
}

impl ExternalCallMediator for ScriptedMediator { /* pop and return */ }
```

Crates and modules consumed (no new external dependencies expected):

- `monty` (already used by `zamburak-monty`),
- `zamburak-core` (graph, labels, authority),
- `zamburak-policy` (engine, decision types),
- `serde`, `serde_json`, `thiserror`, `tracing` (already in workspace),
- `rstest`, `rstest-bdd`, `rstest-bdd-macros` (already in dev-deps).

If any of the above is not already available transitively to the test
crates, add it to the relevant `Cargo.toml` `[dev-dependencies]` only —
never to production dependencies — and stop and escalate before adding
anything else.

---

Revision note: initial draft. Awaiting approval before implementation.
