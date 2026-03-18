# Add `zamburak-monty` adapter crate for governed execution (Task 0.6.1)

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`,
`Decision log`, and `Outcomes & retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / big picture

Implement roadmap Task 0.6.1 from `docs/roadmap.md`: add the
`crates/zamburak-monty` adapter crate that provides a governed execution path
around the vendored `full-monty` interpreter substrate.

After this change, a consumer of the Zamburak library can:

- construct a `GovernedRunner` (or equivalent governed run orchestrator) that
  wraps `MontyRun` with a Zamburak `RuntimeObserver` implementation installed
  via `RuntimeObserverHandle`,
- execute a Monty program through a single governed-run entrypoint that
  mediates every external-function call through a deterministic hook,
- supply a policy engine and receive allow, deny, or confirmation decisions at
  each external-call boundary without implementing observer plumbing directly.

This is the first Track B pull request (PR B1) as defined in
`docs/adr-001-monty-ifc-vm-hooks.md` section "Track B staged pull requests". It
bridges the generic Track A observer substrate into Zamburak governance
semantics, keeping the full IFC propagation graph and dependency-summary
machinery out of scope for this task.

## Constraints

- Implement to requirement signposts:
  `docs/adr-001-monty-ifc-vm-hooks.md` section "Track B staged pull requests",
  `docs/zamburak-design-document.md` sections "Architecture overview" and
  "Policy evaluation semantics", `docs/repository-layout.md` section
  `crates/zamburak-monty`.
- Dependency constraint: Tasks 0.5.2 (generic observer events) and 0.1.1
  (canonical policy schema v1) are hard preconditions. Confirm both are marked
  done before code changes.
- In scope: `full-monty` integration, observer installation, and one governed
  run entrypoint with deterministic external-call mediation hooks.
- Out of scope: full IFC propagation semantics (deferred to Task 0.6.2),
  IFC-to-observer wiring (deferred to Task 0.6.3), and full policy-gate
  enforcement at external-call boundaries (deferred to Task 0.6.4).
- Track B guardrail: all governance semantics must live in the
  `zamburak-monty` crate (or other Zamburak-owned crates), never in
  `third_party/full-monty/`.
- No changes to the `full-monty` submodule are expected or permitted for this
  task. The adapter consumes the existing public API only.
- Any new public API must follow existing Zamburak crate conventions:
  module-level `//!` docs, public item `///` docs, `missing_docs = "deny"`.
- File size limit: no single code file may exceed 400 lines.
- Behaviour-Driven Development (BDD) coverage must use `rstest-bdd` v0.5.0
  where applicable, covering happy and unhappy paths and relevant edge cases.
- Record design decisions in `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` for consumer-visible behaviour and API.
- Update `docs/repository-layout.md` if artefact locations differ from the
  planned layout.
- Mark roadmap Task 0.6.1 done in `docs/roadmap.md` only after all gates are
  green.
- Required completion gates: `make check-fmt`, `make lint`, and `make test`.
- Use en-GB-oxendict spelling in documentation and comments.

## Tolerances (exception triggers)

- Scope tolerance: if implementation requires edits in more than 25 files or
  2000 net changed lines, stop and escalate with a split plan.
- Interface tolerance: if the `full-monty` public API does not expose
  sufficient observer or run types to implement the governed entrypoint from
  outside the crate, stop and present alternatives (e.g. thin additive API in
  `full-monty` versus internal workaround).
- Dependency tolerance: if any new third-party dependency is required beyond
  what already exists in the workspace, stop and escalate before adding it.
- Submodule tolerance: if any change to `third_party/full-monty/` is required,
  stop and escalate. This task must consume the existing Track A API as-is.
- Policy-depth tolerance: if implementing the governed entrypoint requires
  full IFC propagation or dependency-summary computation (beyond stub
  placeholders), stop and defer to Tasks 0.6.2 and 0.6.3.
- Iteration tolerance: if required gates fail after three focused fix loops,
  stop and report failures with root-cause hypotheses.

## Risks

- Risk: `full-monty` types (`MontyRun`, `RuntimeObserverHandle`,
  `RuntimeObserverEvent`, `RunProgress`, etc.) may have `pub(crate)` fields or
  constructors that prevent external wrapping from a sibling workspace crate.
  Severity: high. Likelihood: medium. Mitigation: verify API surface
  accessibility early in Stage A. If blocked, evaluate whether a thin re-export
  shim in `full-monty` is justified (escalate per submodule tolerance) or
  whether an integration-test-only approach is preferable.

- Risk: the submodule may not be initialized in the working tree. Severity:
  low. Likelihood: high. Mitigation: concrete steps include submodule
  initialization as the first action.

- Risk: the strict Clippy lint configuration in the workspace
  (`expect_used = "deny"`, `unwrap_used = "deny"`, `indexing_slicing = "deny"`,
  etc.) may conflict with patterns needed for observer event matching.
  Severity: medium. Likelihood: medium. Mitigation: use exhaustive match arms,
  `.get()` for slice access, and `Result`-returning helpers. Avoid lint
  suppressions except as a last resort with tightly scoped reasons.

- Risk: the governed entrypoint design may be too tightly coupled to the
  current `PolicyEngine` shape, making it difficult for Task 0.6.4 to introduce
  boundary enforcement cleanly. Severity: medium. Likelihood: low. Mitigation:
  define the mediation hook as a trait or callback rather than hard-wiring
  `PolicyEngine` directly. Keep the hook signature minimal and extensible.

## Progress

- [x] Reviewed roadmap Task 0.6.1, governing ADR, engineering standards,
  design document, user guide, repository layout, and neighbouring ExecPlans.
- [x] Initialized `third_party/full-monty/` and verified Track A API
  accessibility from an external crate.
- [x] Drafted this ExecPlan.
- [x] Implemented Stage A: crate scaffold and workspace registration.
- [x] Implemented Stage B: observer bridge and event recording.
- [x] Implemented Stage C: governed run entrypoint and external-call
  mediation hooks.
- [x] Implemented Stage D: unit tests and rstest parameterized cases.
- [x] Implemented Stage E: BDD behavioural tests with rstest-bdd v0.5.0.
- [x] Implemented Stage F: documentation and roadmap sync.
- [x] Final gate run: `make check-fmt`, `make lint`, `make test` all pass.

## Surprises & discoveries

No surprises or discoveries recorded yet. This section will be updated during
implementation.

## Decision log

- Decision: define the external-call mediation hook as a trait
  (`ExternalCallMediator` or equivalent) rather than hard-wiring `PolicyEngine`
  directly. Rationale: Task 0.6.1 scope is "one governed run entrypoint"
  without full IFC propagation; the trait boundary allows Tasks 0.6.3 and 0.6.4
  to provide progressively richer mediator implementations without changing the
  adapter crate's public run API. The initial implementation will ship with an
  `AllowAllMediator` for testing and a `PolicyMediator` stub that delegates to
  `PolicyEngine`. Date/Author: 2026-03-18 / DevBoxer.

- Decision: the adapter crate depends on `monty` (via path to
  `third_party/full-monty/crates/monty`) and on `zamburak-policy` (via
  workspace path). It does not re-export `full-monty` types directly; consumers
  interact through Zamburak-owned wrapper types. Rationale: this keeps the
  public API stable even if `full-monty` internal types change and avoids
  leaking Track A internals into Track B consumers. Date/Author: 2026-03-18 /
  DevBoxer.

- Decision: use `Arc<Mutex<dyn ExternalCallMediator>>` as the shared
  mediator handle, consistent with `full-monty`'s own `SharedRuntimeObserver`
  pattern (`Arc<Mutex<dyn RuntimeObserver>>`). Rationale: the observer callback
  fires synchronously from the VM execution loop and needs interior mutability
  for state accumulation; the mediator follows the same execution-context
  pattern. Date/Author: 2026-03-18 / DevBoxer.

- Decision: the governed run entrypoint accepts a compiled `MontyRun`
  configuration and returns `GovernedRunProgress`, a Zamburak-owned enum that
  wraps or mirrors `RunProgress` but enriches external-call yields with
  mediation metadata (decision outcome, call identifier). Rationale: this
  preserves the pause/resume protocol while giving consumers governance context
  at each yield point. Date/Author: 2026-03-18 / DevBoxer.

## Outcomes & retrospective

No outcomes recorded yet. This section will be completed after implementation.

## Context and orientation

Current repository state relevant to Task 0.6.1:

- `docs/roadmap.md` marks Tasks 0.5.1, 0.5.2, 0.5.3, 0.5.4, and 0.1.1 as
  complete.
- The Track A observer substrate lives in:
  - `third_party/full-monty/crates/monty/src/observer.rs`
    (`RuntimeObserver`, `RuntimeObserverHandle`, `NoopRuntimeObserver`,
    `SharedRuntimeObserver`),
  - `third_party/full-monty/crates/monty/src/bytecode/vm/observer_hooks.rs`
    (VM-level event emission),
  - `third_party/full-monty/crates/monty/src/run.rs` and `run_progress.rs`
    (`MontyRun`, `RunProgress`, `start_with_observer`),
  - `third_party/full-monty/crates/monty/src/repl.rs`
    (`MontyRepl`, `ReplProgress`).
- The canonical Track A event set is defined in
  `docs/zamburak-design-document.md` section "Track A observer event contract":
  `ValueCreated`, `OpResult`, `ExternalCallRequested`, `ExternalCallReturned`,
  `ControlCondition`.
- The policy engine lives in:
  - `crates/zamburak-policy/src/engine.rs` (`PolicyEngine`),
  - `crates/zamburak-policy/src/policy_def.rs` (`PolicyDefinition`).
- No `crates/zamburak-monty/` directory exists yet. It must be created.
- The workspace `Cargo.toml` currently lists three workspace members:
  `crates/test-utils`, `crates/zamburak-core`, `crates/zamburak-policy`.
- `docs/repository-layout.md` already defines the expected file-purpose
  mapping for `crates/zamburak-monty` (Table 3).
- Existing integration and compatibility test infrastructure under `tests/`
  provides patterns for BDD coverage (`.feature` files, `rstest-bdd` step
  definitions, and `main.rs` test-crate modules).

Files expected to change or be created:

- `crates/zamburak-monty/Cargo.toml` (new)
- `crates/zamburak-monty/src/lib.rs` (new)
- `crates/zamburak-monty/src/observer.rs` (new)
- `crates/zamburak-monty/src/run.rs` (new)
- `crates/zamburak-monty/src/external_call.rs` (new)
- `crates/zamburak-monty/src/error.rs` (new, if error types warrant a
  separate module)
- `Cargo.toml` (workspace member addition)
- `src/lib.rs` (re-export governed run types if appropriate)
- `tests/integration/main.rs` (new or extended, for governed-run integration
  tests)
- `tests/integration/governed_run_bdd.rs` (new)
- `tests/integration/features/governed_run.feature` (new)
- `docs/zamburak-design-document.md` (Track B adapter contract section)
- `docs/users-guide.md` (governed execution section)
- `docs/repository-layout.md` (confirm or update `crates/zamburak-monty`
  entries)
- `docs/roadmap.md` (mark Task 0.6.1 done)

## Plan of work

### Stage A: preflight and crate scaffold

Confirm that roadmap Tasks 0.5.2 and 0.1.1 are marked done. Initialize the
`full-monty` submodule. Verify that `MontyRun`, `RuntimeObserverHandle`,
`RuntimeObserver`, `RuntimeObserverEvent`, `RunProgress`, and snapshot types
are accessible from an external crate within the workspace.

Create the `crates/zamburak-monty/` directory with `Cargo.toml` and
`src/lib.rs`. Register it as a workspace member. Add dependencies on `monty`
(via path to `third_party/full-monty/crates/monty`) and on `zamburak-core` and
`zamburak-policy` (via workspace paths). Add dev-dependencies on `rstest` and
`rstest-bdd`. Verify that the crate compiles with `cargo check`.

Key files:

- `crates/zamburak-monty/Cargo.toml`
- `crates/zamburak-monty/src/lib.rs`
- `Cargo.toml` (workspace members list)

Go or no-go for Stage A: the new crate compiles, imports `monty` types
successfully, and `cargo check --workspace` passes.

### Stage B: observer bridge and event recording

Implement `crates/zamburak-monty/src/observer.rs` with a `ZamburakObserver`
struct that implements the `full-monty` `RuntimeObserver` trait. The observer:

- records `ExternalCallRequested` events so the governed run entrypoint can
  intercept them,
- records `ExternalCallReturned` events for post-call auditing hooks,
- passes through `ValueCreated`, `OpResult`, and `ControlCondition` events
  to an optional downstream event sink (preparing for Task 0.6.3 IFC wiring
  without implementing it now).

The observer must be constructible with a shared mediator handle
(`Arc<Mutex<dyn ExternalCallMediator>>`) so mediation decisions can be queried
synchronously during the call boundary.

Key files:

- `crates/zamburak-monty/src/observer.rs`

Go or no-go for Stage B: the observer compiles, implements `RuntimeObserver`,
and `cargo check` passes.

### Stage C: governed run entrypoint and external-call mediation hooks

Implement `crates/zamburak-monty/src/run.rs` with a `GovernedRunner` struct and
`GovernedRunProgress` enum. The governed run entrypoint:

1. accepts compiled Monty inputs and a mediator,
2. constructs a `RuntimeObserverHandle` wrapping the `ZamburakObserver`,
3. calls `MontyRun::start_with_observer(...)`,
4. on each `RunProgress::FunctionCall` or `RunProgress::OsCall` yield:
   a. extracts call metadata (call ID, runtime IDs, call kind), b. invokes the
   mediator's `mediate(...)` method with call context, c. based on the
   `MediationDecision` (Allow, Deny, or
      RequireConfirmation):
      - Allow: resumes execution with the host-provided external result,
      - Deny: resumes execution with an external error (or returns a
        governed denial progress state),
      - RequireConfirmation: yields a `GovernedRunProgress::AwaitConfirmation`
        state to the host for interactive approval,
5. on `RunProgress::Complete`: returns `GovernedRunProgress::Complete`.

Implement `crates/zamburak-monty/src/external_call.rs` with the
`ExternalCallMediator` trait and basic implementations:

- `AllowAllMediator`: unconditionally returns `MediationDecision::Allow`.
  Used for testing and permissive-mode operation.
- `DenyAllMediator`: unconditionally returns `MediationDecision::Deny`.
  Used for testing deny-path coverage.

The `MediationDecision` enum must include at minimum:

- `Allow` — proceed with execution,
- `Deny { reason: String }` — block the call with an explanation,
- `RequireConfirmation { request: ConfirmationContext }` — yield to
  host for interactive approval.

Key files:

- `crates/zamburak-monty/src/run.rs`
- `crates/zamburak-monty/src/external_call.rs`

Go or no-go for Stage C: the governed runner compiles, the `AllowAllMediator`
produces `GovernedRunProgress::Complete` for a simple program, and
`cargo check --workspace` passes.

### Stage D: unit tests and rstest parameterized cases

Add unit tests within the `zamburak-monty` crate covering:

1. **Observer event recording**: verify that `ZamburakObserver` correctly
   captures `ExternalCallRequested` and `ExternalCallReturned` events.
2. **Governed run happy path**: a simple Monty program that completes
   without external calls produces `GovernedRunProgress::Complete`.
3. **Governed run with external call (allow)**: a program that triggers an
   external function call, mediated by `AllowAllMediator`, completes after
   resume.
4. **Governed run with external call (deny)**: a program that triggers an
   external function call, mediated by `DenyAllMediator`, returns a governed
   denial state or error propagation.
5. **Governed run with multiple external calls**: verify sequential
   mediation of multiple call yields.
6. **Observer receives all event types**: verify that `ValueCreated`,
   `OpResult`, `ExternalCallRequested`, `ExternalCallReturned`, and
   `ControlCondition` events are all received during a representative governed
   execution.

Use `rstest` fixtures for shared setup (compiled Monty programs, mediator
construction). Use `rstest` parameterized `#[case]` where test logic is
identical across mediator types or program inputs.

If inline unit tests exceed the 400-line file limit, extract test modules to
sibling files using the `#[path = "..."]` attribute pattern established in the
codebase.

Key files:

- `crates/zamburak-monty/src/run.rs` (inline `#[cfg(test)]` module or
  extracted `run_tests.rs`)
- `crates/zamburak-monty/src/observer.rs` (inline `#[cfg(test)]` module
  or extracted `observer_tests.rs`)

Go or no-go for Stage D: all new unit tests pass with
`cargo test -p zamburak-monty`.

### Stage E: BDD behavioural tests with rstest-bdd v0.5.0

Create BDD scenarios covering the governed execution contract from a consumer
perspective. Use `rstest-bdd` v0.5.0 with Gherkin `.feature` files.

Feature file: `tests/integration/features/governed_run.feature`

Scenarios (minimum):

1. **Happy path: governed execution completes without external calls.**
   Given a simple Monty program with no external calls and an
   `AllowAllMediator`, when the governed runner executes the program, then the
   result is `GovernedRunProgress::Complete` with the expected return value.

2. **Happy path: governed execution mediates an external function call.**
   Given a Monty program that calls an external function and an
   `AllowAllMediator`, when the governed runner executes the program, then the
   external call is intercepted, the mediator receives call metadata, the host
   provides a return value, and execution completes.

3. **Unhappy path: governed execution denies an external function call.**
   Given a Monty program that calls an external function and a
   `DenyAllMediator`, when the governed runner executes the program, then the
   external call is denied with a reason and execution terminates or propagates
   a denial error.

4. **Edge case: governed execution handles multiple sequential external
   calls.** Given a Monty program with two sequential external calls and an
   `AllowAllMediator`, when the governed runner executes, then both calls are
   mediated in order and execution completes.

5. **Edge case: observer receives the full Track A event set during
   governed execution.** Given a Monty program with branching, value creation,
   and an external call, when the governed runner executes with an
   `AllowAllMediator`, then the observer records `ValueCreated`, `OpResult`,
   `ControlCondition`, `ExternalCallRequested`, and `ExternalCallReturned`
   events.

Create `tests/integration/main.rs` (if it does not exist) and
`tests/integration/governed_run_bdd.rs` with step definitions.

Key files:

- `tests/integration/main.rs` (new)
- `tests/integration/governed_run_bdd.rs` (new)
- `tests/integration/features/governed_run.feature` (new)

Go or no-go for Stage E: all BDD scenarios pass with
`cargo test --test integration governed_run`.

### Stage F: documentation and roadmap sync

Update:

- `docs/zamburak-design-document.md`:
  Add or update the Track B adapter contract subsection under "Architecture
  overview" to describe the `zamburak-monty` crate's role, the
  `ExternalCallMediator` trait boundary, and the `GovernedRunner` orchestration
  pattern. Record any implementation decisions made during Stages B–E.

- `docs/users-guide.md`:
  Add a "Governed execution with `zamburak-monty`" section describing:
  - how to construct a `GovernedRunner`,
  - how to implement or select an `ExternalCallMediator`,
  - the `GovernedRunProgress` yield states,
  - a minimal code example showing a governed run with `AllowAllMediator`.

- `docs/repository-layout.md`:
  Confirm or update the `crates/zamburak-monty` file-purpose mapping (Table 3)
  to match the realized file layout.

- `docs/roadmap.md`:
  Mark Task 0.6.1 done only after all gates pass.

Go or no-go for Stage F: documentation is accurate and internally consistent.

## Concrete steps

<!-- markdownlint-disable MD029 -->
1. Initialize the submodule if needed.

   ```plaintext
   git submodule update --init --recursive
   ```

   Expected outcome: `third_party/full-monty/` is populated and ready for build
   and test.

2. Confirm dependency gates.

   ```plaintext
   rg -n "Task 0.5.2|Task 0.1.1" docs/roadmap.md
   ```

   Expected outcome: both tasks are marked `[x]` (complete).

3. Verify `full-monty` API accessibility.

   ```plaintext
   rg -n "pub struct MontyRun|pub fn start_with_observer|pub trait RuntimeObserver" \
     third_party/full-monty/crates/monty/src/
   ```

   Expected outcome: `MontyRun`, `start_with_observer`, and `RuntimeObserver`
   are `pub` and accessible from an external crate.

4. Create crate scaffold.

   - Create `crates/zamburak-monty/Cargo.toml` with dependencies on
     `monty`, `zamburak-core`, and `zamburak-policy`.
   - Create `crates/zamburak-monty/src/lib.rs` with module declarations.
   - Add `"crates/zamburak-monty"` to workspace members in root
     `Cargo.toml`.
   - Run `cargo check --workspace` to verify compilation.

5. Implement observer bridge.

   - Create `crates/zamburak-monty/src/observer.rs`.
   - Implement `ZamburakObserver` implementing `RuntimeObserver`.
   - Run `cargo check -p zamburak-monty`.

6. Implement governed run entrypoint and mediation traits.

   - Create `crates/zamburak-monty/src/run.rs`.
   - Create `crates/zamburak-monty/src/external_call.rs`.
   - Implement `GovernedRunner`, `GovernedRunProgress`,
     `ExternalCallMediator`, `AllowAllMediator`, `DenyAllMediator`.
   - Run `cargo check --workspace`.

7. Implement unit tests.

   - Add `#[cfg(test)]` modules or extracted test files.
   - Run `cargo test -p zamburak-monty`.

8. Implement BDD behavioural tests.

   - Create `tests/integration/main.rs`,
     `tests/integration/governed_run_bdd.rs`, and
     `tests/integration/features/governed_run.feature`.
   - Run `cargo test --test integration governed_run`.

9. Update documentation.

   - Update `docs/zamburak-design-document.md`,
     `docs/users-guide.md`, `docs/repository-layout.md`.
   - Mark Task 0.6.1 done in `docs/roadmap.md`.

10. Run documentation gates.

    ```plaintext
    set -o pipefail; make fmt | tee /tmp/make-fmt-0-6-1.log
    ```

    ```plaintext
    set -o pipefail; make markdownlint | tee /tmp/make-markdownlint-0-6-1.log
    ```

    ```plaintext
    set -o pipefail; make nixie | tee /tmp/make-nixie-0-6-1.log
    ```

11. Run the required root gates.

    ```plaintext
    set -o pipefail; make check-fmt | tee /tmp/make-check-fmt-0-6-1.log
    ```

    ```plaintext
    set -o pipefail; make lint | tee /tmp/make-lint-0-6-1.log
    ```

    ```plaintext
    set -o pipefail; make test | tee /tmp/make-test-0-6-1.log
    ```

    Expected outcome: all required repository gates are green.
<!-- markdownlint-enable MD029 -->

## Acceptance criteria

- Governed execution path:
  `GovernedRunner` wraps `MontyRun::start_with_observer(...)` with a
  `ZamburakObserver` and mediates every external-call yield through an
  `ExternalCallMediator`.

- Deterministic external-call mediation:
  every `RunProgress::FunctionCall` and `RunProgress::OsCall` yield passes
  through the mediator before resume, with deterministic `MediationDecision`
  outcomes.

- Observer event completeness:
  `ZamburakObserver` receives all five Track A event classes during governed
  execution.

- Unit test coverage:
  happy paths (allow), unhappy paths (deny), multiple sequential calls, and
  observer event completeness are covered by unit tests using `rstest`.

- BDD coverage:
  `rstest-bdd` v0.5.0 scenarios cover the governed execution contract from a
  consumer perspective, including happy paths, deny paths, and edge cases.

- Documentation:
  design document, user guide, repository layout, and roadmap reflect the
  delivered adapter crate and governed execution contract.

- Gates:
  `make check-fmt`, `make lint`, and `make test` pass at the repository root.

## Evidence to capture

- `cargo test -p zamburak-monty` log showing all adapter-crate unit tests
  passing.
- `cargo test --test integration governed_run` log showing BDD scenarios
  passing.
- Final root gate logs from `make check-fmt`, `make lint`, and `make test`.
- Short decision-note entries in this document for any design choices made
  during implementation.

## Interfaces and dependencies

### `full-monty` types consumed (Track A public API)

- `monty::MontyRun` — run-mode execution context.
- `monty::RunProgress` — execution yield enum (FunctionCall, OsCall,
  Complete, Error).
- `monty::RuntimeObserver` — observer trait for event callbacks.
- `monty::RuntimeObserverHandle` — handle wrapping a shared observer.
- `monty::RuntimeObserverEvent` — event enum with five canonical variants.
- `monty::ExternalCallRequestedEvent`,
  `monty::ExternalCallReturnedEvent` — event payloads.
- `monty::ExternalResult` — host-provided call result for resume.
- `monty::RuntimeValueId` — opaque runtime value identifier.
- `monty::ExternalCallKind` — function, OS, or method call kind.
- `monty::SnapshotExtension` — optional embedder-owned snapshot bytes
  (consumed but not interpreted by this task; prepared for Task 0.6.5
  snapshot-governance suites).

### Zamburak types consumed

- `zamburak_policy::PolicyEngine` — policy evaluation engine (used in
  future `PolicyMediator` stub, not fully wired for this task).
- `zamburak_core` types as needed for error modelling.

### Types introduced by this task

- `zamburak_monty::GovernedRunner` — orchestrates governed execution.
- `zamburak_monty::GovernedRunProgress` — governed execution yield enum.
- `zamburak_monty::ZamburakObserver` — `RuntimeObserver` implementation
  that bridges Track A events to Track B governance.
- `zamburak_monty::ExternalCallMediator` — trait for external-call
  mediation decisions.
- `zamburak_monty::MediationDecision` — enum of allow, deny, and
  confirmation outcomes.
- `zamburak_monty::AllowAllMediator` — permissive mediator for testing.
- `zamburak_monty::DenyAllMediator` — restrictive mediator for testing.
- `zamburak_monty::GovernedRunError` — error type for governed execution
  failures.

## Revision note

- 2026-03-18: Initial plan drafted from roadmap Task 0.6.1 requirements,
  ADR-001 Track B PR B1 specification, design document architecture overview,
  and repository layout expectations.
