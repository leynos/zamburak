# Implement stable runtime IDs in `full-monty` (Task 0.5.1)

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & discoveries`, `Decision log`, and
`Outcomes & retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Implement roadmap Task 0.5.1 from `docs/roadmap.md`: add stable, host-only
runtime IDs in `third_party/full-monty/` with continuity across `start()` or
`resume()` and `dump()` or `load()`.

After this change, host integrations can observe stable runtime IDs for values
crossing Monty pause and resume boundaries, and those IDs remain consistent
after snapshot serialization round-trips. Success is observable through
unit-level and behavioural tests proving:

- uniqueness within an execution,
- continuity across suspend and resume,
- continuity across dump and load,
- fail-closed behaviour for invalid resume paths.

## Constraints

- Implement to requirement signposts:
  `docs/adr-001-monty-ifc-vm-hooks.md` section "A1. Stable, host-only runtime
  IDs", `docs/zamburak-design-document.md` section "Snapshot and resume
  semantics", and `docs/verification-targets.md` row "IFC (information-flow
  control) propagation".
- Dependency constraint: Task 0.4.2 is a hard precondition for 0.5.1.
  If it is not complete, stop and complete 0.4.2 first.
- In scope: unique host-facing IDs with continuity across `start()` or
  `resume()` and `dump()` or `load()`.
- Out of scope: encoding policy meaning into runtime IDs.
- Track A guardrail: do not introduce Zamburak-specific semantics or naming in
  `third_party/full-monty/` public API.
- Runtime IDs must be additive and generic. Existing execution semantics must
  remain unchanged when host code ignores runtime IDs.
- Add tests covering happy, unhappy, and edge paths.
- Add behavioural tests using `rstest-bdd` v0.5.0 in `full-monty` tests where
  applicable.
- Record any design decisions in `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` for consumer-visible API or behaviour changes.
- Mark roadmap Task 0.5.1 as done in `docs/roadmap.md` only after all gates
  are green.
- Required completion gates: `make check-fmt`, `make lint`, and `make test`.

## Tolerances (exception triggers)

- Scope tolerance: if implementation needs edits in more than 18 files or
  1200 net changed lines, stop and escalate with a split proposal.
- Interface tolerance: if stable runtime IDs require a breaking change in
  existing `monty` host API signatures, stop and escalate with additive and
  breaking options.
- Dependency tolerance: if a new third-party dependency is required in either
  the superproject or `full-monty`, stop and escalate before adding it.
- Execution-model tolerance: if proving uniqueness requires invasive VM-wide
  value-type rewrites that exceed this task's scope, stop and escalate with a
  two-step substrate plan.
- Iteration tolerance: if required gates fail after three focused fix loops,
  stop, and report failures and root-cause hypotheses.
- Ambiguity tolerance: if "unique per execution" cannot be implemented without
  clarifying whether identity is per value instance or per object identity,
  stop, and request a decision with trade-offs.

## Risks

- Risk: current Monty internal value identity (`Value::id()`) may not satisfy
  strict uniqueness expectations for all value kinds. Severity: high
  Likelihood: medium Mitigation: run a focused design checkpoint and document
  the accepted identity model in the design document before implementation.

- Risk: snapshot compatibility regressions if runtime-ID state is not included
  in serialized VM snapshot state. Severity: high Likelihood: medium
  Mitigation: add round-trip tests for both `RunProgress` and resumed execution
  paths, including repeated dump or load cycles.

- Risk: host-binding drift (Rust, Python, JavaScript) if new runtime-ID fields
  are not wired consistently. Severity: medium Likelihood: medium Mitigation:
  update wrapper crates and add focused wrapper tests for the new host-facing
  runtime-ID surface.

## Progress

- [x] (2026-02-26 17:23Z) Reviewed roadmap and signpost documents and drafted
  this ExecPlan.
- [x] (2026-02-26 17:22Z) Initialized `third_party/full-monty/` submodule and
  confirmed current code locations relevant to runtime execution snapshots.
- [x] Finalized runtime-ID substrate design and API shape.
- [x] Implemented runtime-ID state plumbing in `full-monty` runtime and
  snapshot-facing structures.
- [x] Added unit and behavioural tests (including `rstest-bdd` coverage where
  applicable) for uniqueness and continuity.
- [x] Updated `docs/zamburak-design-document.md` and `docs/users-guide.md`.
- [x] Ran required gates and marked Task 0.5.1 done in `docs/roadmap.md`.

## Surprises & discoveries

- Observation: `third_party/full-monty/` was empty in the initial workspace
  checkout until submodules were initialized. Evidence:
  `ls third_party/full-monty` returned no source files before
  `git submodule update --init --recursive`. Impact: implementation
  instructions must include an explicit submodule initialization step.

- Observation: superproject compatibility tests cannot directly depend on
  `third_party/full-monty/crates/monty` because nested workspace manifests with
  `workspace = true` package and dependency keys cannot be resolved from the
  parent workspace. Evidence: `cargo test --test compatibility runtime_ids`
  failed with workspace-root inheritance and nested-workspace errors. Impact:
  behavioural runtime-ID coverage moved into `full-monty` tests using
  `rstest-bdd` v0.5.0.

## Decision log

- Decision: enforce a hard go/no-go dependency gate on Task 0.4.2 before
  implementing 0.5.1 changes. Rationale: roadmap dependency order is normative
  and avoids overlapping unresolved repository-mechanics work with Track A
  substrate work. Date/Author: 2026-02-26 / Codex.

- Decision: keep runtime IDs strictly generic and host-facing, with no policy
  semantics in Track A naming or payload design. Rationale: required by ADR-001
  Track A constraints and fork policy. Date/Author: 2026-02-26 / Codex.

- Decision: validate behaviour in `full-monty` test binaries only
  (`runtime_ids.rs` and `runtime_ids_bdd.rs`) due to nested workspace
  limitations for direct superproject compatibility wiring. Rationale:
  preserves required unit and behavioural coverage while keeping the
  superproject workspace manifest stable. Date/Author: 2026-02-26 / Codex.

## Outcomes & retrospective

Completed.

- Runtime-ID substrate delivered as additive `full-monty` API:
  `RuntimeValueId`, `RunProgress` and `ReplProgress` runtime-ID fields, and
  `runtime_ids()` accessors.
- Runtime IDs now round-trip through `RunProgress::dump()` and
  `RunProgress::load()` with continuity across `resume()`.
- Validation evidence:
  - `cargo test --manifest-path third_party/full-monty/Cargo.toml -p monty`
    `--test runtime_ids --test runtime_ids_bdd`
  - `make check-fmt`
  - `make lint`
  - `make test`
- Documentation and roadmap synchronized:
  - snapshot semantics design decision recorded,
  - user-facing behaviour documented,
  - roadmap task 0.5.1 marked done.

## Context and orientation

Current repository state relevant to Task 0.5.1:

- `third_party/full-monty/` is present as a git submodule at the repository's
  currently pinned revision.
- Runtime pause/resume orchestration is in
  `third_party/full-monty/crates/monty/src/run.rs` and
  `third_party/full-monty/crates/monty/src/repl.rs`.
- VM snapshot state is defined in
  `third_party/full-monty/crates/monty/src/bytecode/vm/mod.rs` (`VMSnapshot`).
- Host argument marshalling for external and OS calls is in
  `third_party/full-monty/crates/monty/src/args.rs` (argument-to-host payload
  conversion paths).
- Current value identity helper logic is in
  `third_party/full-monty/crates/monty/src/value.rs` (`Value::id()`), but this
  is not yet a documented 0.5.1 host-runtime-ID contract.
- Behavioural coverage for this task now lives directly in
  `third_party/full-monty/crates/monty/tests/runtime_ids_bdd.rs`.

Files expected to change:

- `third_party/full-monty/crates/monty/src/run.rs`
- `third_party/full-monty/crates/monty/src/repl.rs`
- `third_party/full-monty/crates/monty/src/bytecode/vm/mod.rs`
- `third_party/full-monty/crates/monty/src/args.rs`
- `third_party/full-monty/crates/monty/src/value.rs` (or a new sibling module
  for runtime-ID types and allocator state)
- `third_party/full-monty/crates/monty/src/lib.rs` (exports if needed)
- `third_party/full-monty/crates/monty/tests/` (new runtime-ID unit/integration
  tests)
- `third_party/full-monty/crates/monty/Cargo.toml` (test-only
  `rstest-bdd` dependencies)
- `third_party/full-monty/crates/monty/tests/features/runtime_ids.feature`
- `third_party/full-monty/crates/monty/tests/runtime_ids_bdd.rs`
- `docs/zamburak-design-document.md`
- `docs/users-guide.md`
- `docs/roadmap.md`

## Plan of work

Stage A: dependency and baseline preflight.

- Verify Task 0.4.2 completion and ensure `make monty-sync` plus sync policy
  are present and green. If not, stop this task and complete 0.4.2 first.
- Initialize and sync `third_party/full-monty/` to the pinned revision.
- Capture baseline test evidence that currently does not prove the 0.5.1
  acceptance criteria.

Go/no-go for Stage A: dependency gate is satisfied and submodule is ready.

Stage B: runtime-ID substrate design checkpoint (short, explicit, and frozen).

- Define the host-facing runtime-ID surface in `full-monty` as additive API.
  This includes naming, type (`u64`-class newtype), and where IDs appear in run
  progress payloads.
- Define serialization requirements so runtime-ID state survives
  `dump()`/`load()` and remains stable across `resume()`.
- Record the chosen identity model in `docs/zamburak-design-document.md` with
  explicit rationale for uniqueness semantics.

Go/no-go for Stage B: one concrete identity model is documented and accepted.

Stage C: implement runtime-ID plumbing in `full-monty`.

- Add or update runtime-ID state in VM execution context and snapshot state
  (`VMSnapshot`, and run/repl snapshot holders).
- Update host boundary conversion paths so host-visible values carry runtime IDs
  while preserving existing behaviour when callers ignore IDs.
- Ensure state restoration during `resume()` and `load()` reconstructs the
  runtime-ID allocator or mapping without ID drift.
- Keep Track A API generic; avoid policy-specific naming or semantics.

Go/no-go for Stage C: compile passes and targeted runtime-ID tests fail before
and pass after implementation.

Stage D: verification suites (unit + behavioural).

- Add `full-monty` tests in `third_party/full-monty/crates/monty/tests/`
  validating:
  - uniqueness of runtime IDs for distinct values in one execution,
  - continuity across `start()` to `resume()`,
  - continuity across `dump()` to `load()` for both runner and progress state,
  - unhappy path handling for invalid resume inputs without silent ID reuse.
- Add `full-monty` behavioural tests using `rstest-bdd` v0.5.0 in
  `crates/monty/tests/runtime_ids_bdd.rs` and
  `crates/monty/tests/features/runtime_ids.feature` for happy and unhappy paths.

Go/no-go for Stage D: runtime unit and behavioural BDD tests pass and prove the
task completion criteria.

Stage E: documentation sync, roadmap closure, and quality gates.

- Update `docs/users-guide.md` with any new host-visible runtime-ID behaviour or
  API usage.
- Update `docs/zamburak-design-document.md` with final runtime-ID design
  decisions and continuity invariants.
- Mark Task 0.5.1 done in `docs/roadmap.md`.
- Run all required gates and archive logs as implementation evidence.

Go/no-go for Stage E: all gates are green and docs plus roadmap are aligned.

## Concrete steps

Run from repository root (`/home/user/project`). Use `set -o pipefail` and
`tee` for gate outputs.

- Step 1: Preflight dependency and submodule readiness.

```sh
git submodule update --init --recursive
git submodule status
rg -n "Task 0.4.2|monty-sync" docs/roadmap.md Makefile
```

Expected evidence:

```plaintext
<sha> third_party/full-monty (...)
```

- Step 2: Implement runtime-ID substrate in `third_party/full-monty/` files
  listed in this plan.

- Step 3: Run focused `full-monty` runtime-ID tests first.

```sh
set -o pipefail
make -C third_party/full-monty test \
  | tee /tmp/full-monty-test-runtime-ids.out
```

- Step 4: Run required superproject gates.

```sh
set -o pipefail
make check-fmt | tee /tmp/check-fmt-zamburak-runtime-ids.out
set -o pipefail
make lint | tee /tmp/lint-zamburak-runtime-ids.out
set -o pipefail
make test | tee /tmp/test-zamburak-runtime-ids.out
```

- Step 5: Run documentation gates when docs are changed.

```sh
set -o pipefail
make markdownlint | tee /tmp/markdownlint-zamburak-runtime-ids.out
set -o pipefail
make nixie | tee /tmp/nixie-zamburak-runtime-ids.out
set -o pipefail
make fmt | tee /tmp/fmt-zamburak-runtime-ids.out
```

- Step 6: Mark roadmap completion after all gates pass.

```sh
# edit docs/roadmap.md: change Task 0.5.1 checkbox from [ ] to [x]
```

## Validation and acceptance

Acceptance behaviours:

- Host-facing runtime IDs are present for relevant runtime values and are
  unique within a run.
- IDs remain stable when execution pauses and resumes.
- IDs remain stable after dumping and loading runner or progress snapshots.
- Invalid resume paths fail closed and do not silently corrupt runtime-ID
  continuity.
- Behavioural tests in `crates/monty/tests/runtime_ids_bdd.rs` cover happy and
  unhappy scenarios using `rstest-bdd` v0.5.0.
- `make check-fmt`, `make lint`, and `make test` pass in the superproject.
- Roadmap Task 0.5.1 is marked done only after all evidence is green.

Quality criteria:

- Tests: new unit and behavioural coverage for uniqueness and continuity.
- Lint/typecheck: no warnings permitted in required gates.
- Formatting: formatter and markdown validators pass after updates.
- Governance: Track A API remains generic and upstream-friendly.

## Idempotence and recovery

- Submodule initialization is safe to rerun.
- Runtime-ID tests are deterministic and safe to rerun.
- If serialization changes break snapshot loading unexpectedly, keep a small
  migration-compatible adapter or version gate in `full-monty` rather than
  silently resetting IDs.
- If a gate fails, fix one failure class at a time and rerun only the affected
  focused tests before rerunning full gates.

## Artifacts and notes

Expected evidence snippets at completion:

```plaintext
$ make -C third_party/full-monty test
...
test result: ok.

$ make test
...
test result: ok.

$ rg -n "Task 0.5.1" docs/roadmap.md
... [x] Task 0.5.1: Implement stable runtime IDs in `full-monty`.
```

## Interfaces and dependencies

Target interfaces to exist after implementation:

- A host-facing runtime-ID type in `full-monty` (`RuntimeValueId`-style newtype
  over an unsigned integer) with stable serialization.
- Additive run-progress accessors or payload fields exposing runtime IDs for
  host-observed values at pause boundaries.
- Snapshot state persistence of runtime-ID allocator or mapping so IDs are
  continuous across `dump()` or `load()`.
- Compatibility-layer test fixtures in this repository that can execute and
  assert runtime-ID continuity behaviour against `full-monty`.

Dependencies:

- Do not add new third-party dependencies without escalation.
- Reuse existing test stack:
  `rstest`, `rstest-bdd = "0.5.0"`, `rstest-bdd-macros`.

## Revision note

- Initial draft created for roadmap Task 0.5.1 with explicit dependency gate,
  implementation stages, test strategy, and completion gates.
