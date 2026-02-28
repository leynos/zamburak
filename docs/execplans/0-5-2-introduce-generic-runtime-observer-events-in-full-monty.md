# Introduce generic runtime observer events in `full-monty` (Task 0.5.2)

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & discoveries`, `Decision log`, and
`Outcomes & retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Implement roadmap Task 0.5.2 from `docs/roadmap.md`: add a generic runtime
observer event substrate in `third_party/full-monty/` with the Track A minimum
event set from ADR-001:

- `ValueCreated`,
- `OpResult`,
- `ExternalCallRequested`,
- `ExternalCallReturned`,
- `ControlCondition`.

After this change, host integrations can attach an observer to Monty execution
and receive canonical runtime events without changing interpreter semantics.
Success is observable through unit and behavioural tests proving that events
are emitted in expected scenarios, while hook-disabled and no-op-observer modes
preserve baseline behaviour. As part of this task, behavioural coverage is
required both in `third_party/full-monty/` and in Zamburak repository BDD tests
under `tests/compatibility/` and `tests/security/`.

## Constraints

- Implement to requirement signposts:
  `docs/adr-001-monty-ifc-vm-hooks.md` section "A2. Lightweight event emission
  hooks", `docs/zamburak-design-document.md` section "Two-track execution
  model", and `docs/verification-targets.md` row "IFC propagation".
- Dependency constraint: Task 0.5.1 is a hard precondition for 0.5.2.
  Confirm it is marked done before code changes.
- In scope: canonical observer events `ValueCreated`, `OpResult`,
  `ExternalCallRequested`, `ExternalCallReturned`, and `ControlCondition`.
- Out of scope: policy decision types in observer payloads.
- Track A guardrail: do not introduce Zamburak-specific semantics or naming in
  `third_party/full-monty/` public API.
- No behavioural drift is allowed for existing `MontyRun`, `Snapshot`,
  `MontyRepl`, and `ReplSnapshot` entrypoints when no observer is installed.
- Observer defaults must be effectively no-op and cheap (no avoidable per-event
  heap allocation in hot paths).
- Add tests covering happy paths, unhappy paths, and edge cases.
- Add behavioural tests using `rstest-bdd` v0.5.0 where applicable.
- Add Zamburak-repository BDD tests that exercise the new observer behaviour
  exposed by the pinned `full-monty` submodule.
- Record design decisions in `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` for consumer-visible API or behaviour changes.
- Mark roadmap Task 0.5.2 done in `docs/roadmap.md` only after all gates are
  green.
- Required completion gates: `make check-fmt`, `make lint`, and `make test`.

## Tolerances (exception triggers)

- Scope tolerance: if implementation requires edits in more than 20 files or
  1400 net changed lines, stop and escalate with a split plan.
- Interface tolerance: if preserving existing APIs is impossible and a breaking
  signature change is required, stop and escalate with additive and breaking
  options.
- Dependency tolerance: if any new third-party dependency is required in either
  the superproject or `full-monty`, stop and escalate before adding it.
- Hot-path tolerance: if the event API design requires per-event heap
  allocation in the VM opcode loop, stop and escalate with alternative payload
  designs.
- Iteration tolerance: if required gates fail after three focused fix loops,
  stop, and report failures with root-cause hypotheses.
- Ambiguity tolerance: if event semantics are unclear for a specific opcode or
  control-flow form and materially affect IFC correctness, stop, and request a
  decision with concrete examples.
- Cross-workspace tolerance: if Zamburak BDD coverage cannot exercise
  `full-monty` observer behaviour without unsupported nested-workspace linking,
  stop and escalate with command-driven and contract-driven alternatives.

## Risks

- Risk: opcode-level event insertion can sprawl across VM dispatch and increase
  complexity quickly. Severity: high Likelihood: medium Mitigation: centralize
  emission through helper methods in `bytecode/vm/mod.rs`, and start with
  high-leverage opcode families first (binary, compare, call, jump).

- Risk: observer API can accidentally leak policy semantics if event payloads
  are over-specified. Severity: medium Likelihood: medium Mitigation: keep
  payloads strictly runtime-generic (IDs, call IDs, op kind, branch outcome)
  and validate names via `monty_fork_review`.

- Risk: no-op parity regressions in pause and resume flows (`run` and `repl`).
  Severity: high Likelihood: medium Mitigation: add regression tests that
  compare outputs and suspension shape between no observer and no-op observer
  for identical scripts.

- Risk: nested-checkout lint pitfalls (`PYO3_PYTHON` and clippy config).
  Severity: medium Likelihood: medium Mitigation: run
  `make -C third_party/full-monty lint-rs-local` for submodule lint evidence
  and keep required superproject gates authoritative.

- Risk: Zamburak tests cannot directly path-depend on
  `third_party/full-monty/crates/monty` because of nested workspace
  inheritance. Severity: high Likelihood: medium Mitigation: design Zamburak
  BDD tests around a repository-local probe path (command-driven targeted suite
  execution and contract assertions) rather than direct crate linking.

## Progress

- [x] (2026-02-28 02:25Z) Gathered signpost documents, Task 0.5.1 context, and
  current `full-monty` runtime-ID surfaces.
- [x] (2026-02-28 02:25Z) Drafted this ExecPlan with staged delivery, red-green
  test sequencing, and quality gates.
- [x] (2026-02-28 02:42Z) Revised plan to include Zamburak-repository BDD
  suites that exercise `full-monty` observer behaviour.
- [x] (2026-02-28 15:07Z) Confirmed dependency status: roadmap Task 0.5.1 was
  already marked done before implementation edits.
- [x] (2026-02-28 15:41Z) Added observer unit and BDD suites in
  `third_party/full-monty/crates/monty/tests/`.
- [x] (2026-02-28 15:41Z) Implemented observer substrate wiring across VM,
  run/resume, and repl/resume paths with no-op defaults.
- [x] (2026-02-28 15:47Z) Added Zamburak compatibility/security BDD probes for
  observer behaviour exposed by the submodule.
- [x] (2026-02-28 15:54Z) Updated design and user documentation, and marked
  roadmap Task 0.5.2 done.
- [x] (2026-02-28 16:22Z) Ran required quality gates and supporting checks.

## Surprises & discoveries

- Observation: `third_party/full-monty/` is empty until submodules are
  initialized. Evidence: initial `rg` calls failed for
  `third_party/full-monty/crates/monty` before
  `git submodule update --init --recursive`. Impact: include explicit submodule
  initialization in concrete steps.

- Observation: no existing runtime observer substrate exists in
  `crates/monty/src`; event naming currently appears only in ADR and roadmap
  docs. Evidence: repository search returned no observer/event API definitions
  in `full-monty` runtime modules. Impact: this task must introduce both API
  surface and wiring points.
- Observation: strict nested `full-monty` Clippy gates flagged additional
  style constraints (`if_not_else`, redundant clone, assigning-clones, and
  helper argument count) during implementation. Impact: converted one helper to
  a struct-input pattern and tightened event-test code to satisfy existing
  policy without lint suppressions.

## Decision log

- Decision: preserve existing host APIs by adding observer-aware overloads and
  keeping current methods as no-op wrappers. Rationale: satisfies no
  behavioural drift while enabling additive upstream-friendly evolution.
  Date/Author: 2026-02-28 / Codex.

- Decision: treat method calls and OS calls as members of the canonical
  `ExternalCallRequested` and `ExternalCallReturned` event class rather than
  adding Track A-specific policy labels. Rationale: keeps event taxonomy
  generic and aligned with ADR minimum set. Date/Author: 2026-02-28 / Codex.

- Decision: deliver dual-layer behavioural coverage: primary observer event
  semantics in `third_party/full-monty/crates/monty/tests/`, plus
  Zamburak-repository BDD suites that exercise the exposed behaviour through a
  repository-local probe path. Rationale: satisfies roadmap artefact
  expectations for `tests/compatibility/` and `tests/security/` while
  respecting nested workspace constraints. Date/Author: 2026-02-28 / Codex.

- Decision: emit `OpResult` from VM-owned value creation points (core
  unary/binary/compare operations and resume/future resolution conversion) and
  keep `ExternalCallReturned` emission at run/repl boundary wiring. Rationale:
  keeps output IDs true runtime-value IDs and avoids synthetic call-ID payloads
  while preserving generic Track A semantics. Date/Author: 2026-02-28 / Codex.

## Outcomes & retrospective

Task 0.5.2 completed with all requested artefacts and gates.

- Delivered:
  - new observer API substrate in `third_party/full-monty/crates/monty/src/`,
  - run and repl observer-aware entrypoints and snapshot propagation,
  - canonical event emission across value creation, operation results, external
    call request/return, and control conditions,
  - full-monty unit + BDD observer suites,
  - Zamburak compatibility + security BDD probe suites,
  - roadmap/docs updates for consumer and architecture traceability.
- Validation evidence:
  - `make -C third_party/full-monty format-rs`,
  - `make -C third_party/full-monty lint-rs-local`,
  - `cargo test --manifest-path third_party/full-monty/Cargo.toml -p monty`
    `--test runtime_observer_events --test runtime_observer_events_bdd`,
  - `cargo test --test compatibility full_monty_observer`,
  - `cargo test --test security full_monty_observer_error_probe`,
  - `make check-fmt`,
  - `make lint`,
  - `make test`,
  - `make markdownlint`,
  - `make nixie`.

## Context and orientation

Current repository state relevant to Task 0.5.2:

- `docs/roadmap.md` marks Task 0.5.1 done and Task 0.5.2 not done.
- Runtime-ID substrate from Task 0.5.1 is present in:
  - `third_party/full-monty/crates/monty/src/runtime_id.rs`,
  - `third_party/full-monty/crates/monty/src/run.rs`,
  - `third_party/full-monty/crates/monty/src/repl.rs`,
  - `third_party/full-monty/crates/monty/src/args.rs`,
  - `third_party/full-monty/crates/monty/src/progress_runtime_ids.rs`.
- VM suspension boundaries are formed by `FrameExit` variants in
  `third_party/full-monty/crates/monty/src/bytecode/vm/mod.rs` and translated
  to host-facing progress in `run.rs` and `repl.rs`.
- Existing behavioural test scaffolding with `rstest-bdd` v0.5.0 is present in
  `third_party/full-monty/crates/monty/tests/runtime_ids_bdd.rs` and
  `third_party/full-monty/crates/monty/tests/features/runtime_ids.feature`.
- Existing Zamburak BDD scaffolding is present in:
  - `tests/compatibility/` with feature files under
    `tests/compatibility/features/`,
  - `tests/security/` with feature files under `tests/security/features/`.
- Known constraint: superproject tests cannot directly path-depend on
  `third_party/full-monty/crates/monty`; compatibility BDD must use a
  repository-local probe strategy instead of direct crate linking.

Files expected to change:

- `third_party/full-monty/crates/monty/src/lib.rs`
- `third_party/full-monty/crates/monty/src/run.rs`
- `third_party/full-monty/crates/monty/src/repl.rs`
- `third_party/full-monty/crates/monty/src/bytecode/vm/mod.rs`
- `third_party/full-monty/crates/monty/src/args.rs` (if payload helpers are
  needed for call-event IDs)
- `third_party/full-monty/crates/monty/src/observer.rs` (new module)
- `third_party/full-monty/crates/monty/tests/runtime_observer_events.rs` (new)
- `third_party/full-monty/crates/monty/tests/runtime_observer_events_bdd.rs`
  (new)
- `third_party/full-monty/crates/monty/tests/features/runtime_observer_events.feature`
  (new)
- `tests/compatibility/main.rs`
- `tests/compatibility/features/full_monty_observer.feature` (new)
- `tests/compatibility/full_monty_observer_bdd.rs` (new)
- `tests/security/main.rs`
- `tests/security/features/full_monty_observer_security.feature` (new)
- `tests/security/full_monty_observer_security_bdd.rs` (new)
- `docs/zamburak-design-document.md`
- `docs/users-guide.md`
- `docs/roadmap.md`

## Plan of work

Stage A: preflight and baseline (no feature edits).

- Confirm roadmap dependency status (`0.5.1` done).
- Initialize submodule and record baseline green state for required root gates.
- Identify concrete VM opcode and boundary locations where each canonical event
  will be emitted.

Go or no-go for Stage A: dependency gate is satisfied, submodule is available,
and emission points are explicitly mapped.

Stage B: scaffolding and red tests first.

- Add observer-facing tests before implementation:
  - unit tests for event payload correctness and event ordering in simple
    scripts,
  - unhappy-path tests (observer installed while runtime errors occur, and
    unknown `call_id` resume remains fail-closed),
  - no-op parity tests that compare outputs and suspension behaviour with and
    without a no-op observer,
  - BDD scenarios in `runtime_observer_events.feature` for happy and unhappy
    behavioural cases in `full-monty`,
  - BDD scenarios in Zamburak compatibility/security suites that exercise the
    same exposed behaviour through repository-level probes.
- Run targeted tests and confirm they fail for missing observer support.

Go or no-go for Stage B: failing tests clearly encode required behaviour and
fail for the expected reason.

Stage C: implement observer substrate and event emission.

- Introduce a new generic observer API module (`observer.rs`) with canonical
  event types and a default no-op observer implementation.
- Extend `MontyRun` and `MontyRepl` execution paths with additive observer-aware
  entrypoints, while preserving existing methods as no-op wrappers.
- Wire event emission at these points:
  - value creation and operation-result points in VM execution,
  - control-condition evaluation points for branch-driving predicates,
  - external-call request points at `FrameExit::{ExternalCall, OsCall,
    MethodCall}` conversion,
  - external-call return points in resume flows (`Snapshot::run`,
    `FutureSnapshot::resume`, `ReplSnapshot::run`, `ReplFutureSnapshot::resume`).
- Keep payloads generic and ID-centric; do not attach policy decision values.

Go or no-go for Stage C: new APIs compile, red tests from Stage B turn green,
and existing runtime-ID tests still pass.

Stage D: hardening and behavioural coverage.

- Add/expand `rstest-bdd` scenarios covering:
  - event emission on external call request and return,
  - `ValueCreated` and `OpResult` for representative opcode paths,
  - `ControlCondition` for branch conditions,
  - no-op observer parity,
  - unhappy scenario where execution raises and observer receives only
    pre-failure events.
- Add Zamburak BDD scenarios in `tests/compatibility/` and `tests/security/`
  that confirm:
  - canonical observer events are exposed by the pinned `full-monty` revision,
  - no-op observer parity checks are exercised via repository-level probes,
  - fail-closed paths (for example invalid resume/call_id handling) remain
    regression-covered from the superproject perspective.
- Ensure tests stay deterministic and avoid global mutable state.

Go or no-go for Stage D: unit + behavioural suites pass with stable assertions.

Stage E: documentation, roadmap closure, and gates.

- Update `docs/zamburak-design-document.md` with Track A observer design
  decisions and event contract rationale.
- Update `docs/users-guide.md` with consumer-facing observer API usage and
  no-op default semantics.
- Mark Task 0.5.2 as done in `docs/roadmap.md` after all validation succeeds.
- Run required gates and include evidence paths.

Go or no-go for Stage E: `make check-fmt`, `make lint`, and `make test` are all
green, docs are synchronized, and roadmap state is updated.

## Concrete steps

Run all commands from repository root (`/home/user/project`). Use
`set -o pipefail` and `tee` so failures are visible and exit codes are not
masked.

1. Preflight dependency and submodule readiness.

```sh
git submodule update --init --recursive
git submodule status
rg -n "Task 0.5.1|Task 0.5.2" docs/roadmap.md
```

Expected evidence:

```plaintext
... [x] Task 0.5.1 ...
... [ ] Task 0.5.2 ...
<sha> third_party/full-monty (...)
```

1. Add observer-event test files first (unit + BDD in `full-monty` and
   Zamburak compatibility/security suites), then run focused tests to confirm
   red state.

```sh
set -o pipefail
cargo test --manifest-path third_party/full-monty/Cargo.toml -p monty \
  --test runtime_observer_events --test runtime_observer_events_bdd \
  | tee /tmp/full-monty-runtime-observer-red.out
set -o pipefail
cargo test --test compatibility full_monty_observer \
  | tee /tmp/zamburak-compat-observer-red.out
set -o pipefail
cargo test --test security full_monty_observer_security \
  | tee /tmp/zamburak-security-observer-red.out
```

Expected evidence before implementation:

```plaintext
... error[E0432]: unresolved import ... observer ...
... test result: FAILED ...
```

1. Implement observer substrate and event wiring, then rerun focused tests.

```sh
set -o pipefail
cargo test --manifest-path third_party/full-monty/Cargo.toml -p monty \
  --test runtime_observer_events --test runtime_observer_events_bdd \
  --test runtime_ids --test runtime_ids_bdd --test repl \
  | tee /tmp/full-monty-runtime-observer-green.out
set -o pipefail
cargo test --test compatibility full_monty_observer \
  | tee /tmp/zamburak-compat-observer-green.out
set -o pipefail
cargo test --test security full_monty_observer_security \
  | tee /tmp/zamburak-security-observer-green.out
```

1. Run submodule Rust lint in nested-checkout-safe mode.

```sh
set -o pipefail
make -C third_party/full-monty lint-rs-local \
  | tee /tmp/full-monty-lint-rs-local-observer.out
```

1. Run required superproject gates.

```sh
set -o pipefail
make check-fmt | tee /tmp/check-fmt-zamburak-observer-events.out
set -o pipefail
make lint | tee /tmp/lint-zamburak-observer-events.out
set -o pipefail
make test | tee /tmp/test-zamburak-observer-events.out
```

1. If docs changed, run documentation gates.

```sh
set -o pipefail
make markdownlint | tee /tmp/markdownlint-zamburak-observer-events.out
set -o pipefail
make nixie | tee /tmp/nixie-zamburak-observer-events.out
set -o pipefail
make fmt | tee /tmp/fmt-zamburak-observer-events.out
```

1. Mark roadmap completion only after all gates pass.

```plaintext
Edit docs/roadmap.md: change Task 0.5.2 checkbox from [ ] to [x].
```

## Validation and acceptance

Acceptance behaviours:

- Installing an observer emits canonical Track A events:
  `ValueCreated`, `OpResult`, `ExternalCallRequested`, `ExternalCallReturned`,
  and `ControlCondition`.
- Event payloads remain generic runtime instrumentation data and include no
  policy decision types.
- No observer installed: existing API behaviour and outputs remain unchanged.
- No-op observer installed: behaviour remains unchanged relative to baseline.
- Unhappy paths remain fail-closed (for example invalid `call_id` resume still
  errors) and do not silently alter event invariants.
- Behavioural tests using `rstest-bdd` v0.5.0 cover happy, unhappy, and edge
  scenarios in both `third_party/full-monty/` and Zamburak compatibility or
  security suites.
- Required gates pass:
  - `make check-fmt`,
  - `make lint`,
  - `make test`.
- Task 0.5.2 is marked done in `docs/roadmap.md` only after all evidence is
  green.

Quality criteria:

- Tests: new unit + behavioural observer coverage plus runtime-ID regression
  coverage, including BDD checks in `tests/compatibility/` and
  `tests/security/`.
- Lint and type safety: no warnings in required gates.
- Formatting: formatter checks and markdown validators pass for changed docs.
- Governance: `full-monty` delta remains Track A generic and passes
  `monty_fork_review` policy expectations.

## Idempotence and recovery

- Submodule initialization and baseline checks are safe to rerun.
- Test commands are deterministic and safe to rerun.
- If observer wiring introduces drift, isolate and revert only observer-emitter
  call sites while preserving red tests.
- If a gate fails, fix one failure class at a time and rerun focused suites
  before rerunning full gates.

## Artifacts and notes

Expected evidence snippets at completion:

```plaintext
$ cargo test --manifest-path third_party/full-monty/Cargo.toml -p monty \
    --test runtime_observer_events --test runtime_observer_events_bdd
...
test result: ok.

$ cargo test --test compatibility full_monty_observer
...
test result: ok.

$ cargo test --test security full_monty_observer_security
...
test result: ok.

$ make test
...
test result: ok.

$ rg -n "Task 0.5.2" docs/roadmap.md
... [x] Task 0.5.2: Introduce generic runtime observer events in `full-monty`.
```

## Interfaces and dependencies

Target interfaces to exist after implementation:

```rust
pub trait RuntimeObserver {
    fn on_event(&mut self, event: RuntimeObserverEvent<'_>) {}
}

pub enum RuntimeObserverEvent<'a> {
    ValueCreated(ValueCreatedEvent),
    OpResult(OpResultEvent<'a>),
    ExternalCallRequested(ExternalCallRequestedEvent<'a>),
    ExternalCallReturned(ExternalCallReturnedEvent),
    ControlCondition(ControlConditionEvent),
}
```

Representative event payload contracts (names may vary, semantics must not):

- `ValueCreated`: emitted when a new runtime value identity is materialized.
- `OpResult`: emitted with output value ID and contributing input IDs.
- `ExternalCallRequested`: emitted before yielding to host, including call ID
  and argument runtime IDs.
- `ExternalCallReturned`: emitted when host result is resumed into VM, with call
  ID and return-shape metadata.
- `ControlCondition`: emitted when a branch-driving condition is evaluated,
  carrying condition value ID and branch outcome.

Implementation shape constraints:

- Existing APIs remain available unchanged.
- Observer-aware APIs are additive wrappers or overload-like methods.
- Event emission should use borrowed slices and stack-local data where possible
  to avoid unnecessary allocations.
- Do not add external dependencies for this task.
- Reuse existing test stack in `full-monty`:
  `rstest`, `rstest-bdd = "0.5.0"`, `rstest-bdd-macros`.

## Revision note

- Initial draft created for roadmap Task 0.5.2 with dependency gate,
  additive API strategy, red-green test sequencing, and completion gates.
- Revised to include explicit BDD coverage in Zamburak
  `tests/compatibility/` and `tests/security/` for behaviour exposed by
  `full-monty`, including concrete test file targets and commands.
