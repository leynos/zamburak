# Wire `full-monty` observer events into IFC updates (Task 0.6.3)

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / big picture

Implement roadmap Task 0.6.3 from
[docs/roadmap.md](/home/user/project/docs/roadmap.md): wire the Track A
`full-monty` runtime observer event stream into the Track B information-flow
control (IFC) state managed by `crates/zamburak-monty`.

After this change, a governed execution path must do more than count observer
events. It must maintain a live `ValueId`-keyed dependency graph from runtime
events, derive effect-time summaries for external calls, and include
strict-mode control-context influence in those summaries. A consumer of the
library must be able to observe that:

- arithmetic and value-producing operations create IFC edges from operands to
  outputs,
- external-call yields carry complete IFC summaries derived from observer state
  rather than ad hoc bookkeeping,
- strict mode includes active control-context influence in effect checks, so a
  constant external call inside an untrusted conditional is still marked as
  control-dependent,
- integration tests prove supported observer event classes are translated into
  complete IFC state updates.

This is Track B PR B3 in
[docs/adr-001-monty-ifc-vm-hooks.md](/home/user/project/docs/adr-001-monty-ifc-vm-hooks.md).
 It builds directly on Task 0.6.2 (IFC core) and consumes Track A event and
snapshot seams from Task 0.5.3 without adding Zamburak semantics to the
vendored interpreter.

## Constraints

- Implement to the requirement signposts in
  [docs/adr-001-monty-ifc-vm-hooks.md](/home/user/project/docs/adr-001-monty-ifc-vm-hooks.md)
   section "Track B staged pull requests",
  [docs/zamburak-design-document.md](/home/user/project/docs/zamburak-design-document.md)
   sections "Component responsibilities" and "Strict-mode effect semantics",
  and
  [docs/verification-targets.md](/home/user/project/docs/verification-targets.md)
   rows "IFC propagation" and "Control context".
- Dependency precondition: Tasks 0.6.2 and 0.5.3 must remain completed in
  [docs/roadmap.md](/home/user/project/docs/roadmap.md) before implementation
  starts.
- In scope: event-to-IFC graph updates, external-call summary construction from
  observer state, strict-mode control dependency tracking, and additive
  Zamburak-owned APIs needed to expose those summaries to governed mediation.
- Out of scope: policy decision presentation user experience, human-facing deny
  or confirmation rendering, and any Track A semantic change in
  `third_party/full-monty/`.
- All governance semantics must remain in Zamburak-owned code under
  `crates/zamburak-monty/` and `crates/zamburak-core/`. Do not encode policy,
  taint, or Zamburak naming into Track A APIs.
- No single Rust source file may exceed 400 lines. If IFC wiring makes
  [crates/zamburak-monty/src/observer.rs](/home/user/project/crates/zamburak-monty/src/observer.rs)
   or
  [crates/zamburak-monty/src/run/flow.rs](/home/user/project/crates/zamburak-monty/src/run/flow.rs)
   too large, extract internal submodules rather than growing them in place.
- Public APIs must keep module-level `//!` docs and `///` item docs. New public
  IFC-facing types must explain their role with examples where appropriate.
- Validation must include unit tests and behavioural tests using
  `rstest-bdd` v0.5.0 where the behaviour is naturally scenario-shaped. Cover
  happy paths, unhappy paths, and edge cases.
- Record design decisions taken during this task in
  [docs/zamburak-design-document.md](/home/user/project/docs/zamburak-design-document.md).
- Update [docs/users-guide.md](/home/user/project/docs/users-guide.md) for any
  library-consumer-visible behaviour or API additions.
- Mark roadmap Task 0.6.3 done only after all required gates are green.
- Final required gates for the implementation turn are
  `make check-fmt`, `make lint`, and `make test`. For this planning turn, the
  applicable documentation gates are `make fmt`, `make markdownlint`, and
  `make nixie`.
- Use en-GB-oxendict spelling in documentation and comments.

## Tolerances (exception triggers)

- Scope tolerance: if implementation requires edits in more than 22 files or
  1800 net changed lines, stop, document the split, and escalate with a
  narrower follow-on plan.
- Submodule tolerance: if observer-driven IFC completeness cannot be achieved
  using the existing Track A event and snapshot-extension surface, stop,
  document the missing signal precisely, and escalate rather than changing
  `third_party/full-monty/` in this task.
- Interface tolerance: additive Zamburak-owned public API is allowed. If any
  existing public API must change incompatibly, stop and escalate.
- Dependency tolerance: if a new production dependency is required, stop and
  escalate before adding it. Dev-dependency additions are also unexpected here
  because `rstest`, `rstest-bdd`, and `proptest` are already available in the
  workspace.
- Observer-contract tolerance: if the Track A event stream does not provide an
  unambiguous way to determine either active control-context lifetime or
  external-call return provenance, stop after the prototype milestone and
  present concrete options.
- Iteration tolerance: if `make check-fmt`, `make lint`, or `make test` still
  fail after three focused fix loops, stop and report the failing commands plus
  root-cause hypotheses.

## Risks

- Risk: the current checkout has an uninitialized `third_party/full-monty`
  submodule, so the exact Track A event-emission sites and resume ordering
  cannot be inspected until Stage A. Severity: high. Likelihood: high.
  Mitigation: initialize the submodule before any implementation work and make
  event-order reconnaissance the first milestone.

- Risk: `ControlCondition` may indicate condition evaluation but not the end of
  the condition's dynamic scope. Severity: high. Likelihood: medium.
  Mitigation: prototype nested branch and loop traces before designing the
  control-context state machine. If event scope cannot be derived precisely,
  escalate before coding the final model.

- Risk: external-call return provenance may be under-specified if the observer
  stream does not clearly identify the runtime value created from a resumed
  host result. Severity: high. Likelihood: medium. Mitigation: prototype
  allowed external-call resume traces and confirm how the returned value
  becomes visible to Track B. If there is no stable linkage, the task cannot
  safely claim complete IFC state.

- Risk: `DependencyGraph` budget overflow and observer ordering mismatches could
  produce silently incomplete summaries. Severity: high. Likelihood: medium.
  Mitigation: treat missing IDs, graph truncation, and queue drift as explicit
  fail-closed states surfaced through typed Zamburak errors or conservative
  unknown-top summaries, then cover them with unit and behavioural tests.

- Risk: the existing `CallContext` public surface is too thin for Task 0.6.4,
  so a one-off B3 design could force another public API reshape immediately in
  the next task. Severity: medium. Likelihood: medium. Mitigation: add a nested
  Zamburak-owned IFC payload to call context now rather than scattering
  separate fields that will have to be regrouped later.

## Progress

- [x] Reviewed the roadmap item, ADR, design document, verification targets,
  repository layout, users guide, and previous 0.6.1/0.6.2 ExecPlans.
- [x] Drafted this ExecPlan.
- [x] Stage A: initialize `full-monty` and prototype the observer contract for
  control-context lifetime and external-call return provenance.
- [x] Stage B: introduce internal IFC runtime-state types in
  `zamburak-monty` and wire `ZamburakObserver` event handling into them.
- [x] Stage C: expose observer-derived IFC summaries on governed external-call
  contexts and resume paths.
- [x] Stage D: add unit, integration, and security-style behavioural tests,
  including `rstest-bdd` v0.5.0 coverage.
- [x] Stage E: update the design document, users guide, and roadmap, then run
  final gates.

## Surprises & Discoveries

- Discovery: the current planning checkout contains an empty
  `third_party/full-monty/` directory, so exact Track A event ordering is not
  yet verified locally. Stage A is therefore a required prototype, not optional
  research.
- Discovery: `ExternalCallReturned(Return)` is followed by `VM::resume(...)`
  emitting `OpResult(output_id, OpInputIds::None)` without a matching
  `ValueCreated` event. Returned-value provenance is therefore preserved by
  Zamburak-owned observer state, not by a later Track A identity hook.
- Discovery: `ControlCondition` has no explicit scope-exit signal in Track A.
  The implementation uses a conservative lifetime model that accumulates
  control-context influence for the remainder of the governed execution segment.

## Decision Log

- Decision: this task will be planned with an explicit prototype milestone
  before implementation, because the two highest-risk semantics are dynamic
  control-context lifetime and external-call return provenance. Rationale: both
  are required for the completion criterion, but neither can be inferred with
  enough confidence from the current checkout without first inspecting the
  vendored `full-monty` source and traces. Date/Author: 2026-03-25 / Codex.

- Decision: the expected public-shape outcome is an additive Zamburak-owned IFC
  payload on governed call contexts rather than a Track A API change.
  Rationale: Task 0.6.3 is Track B integration work, and Task 0.6.4 should be
  able to consume the same governed context without another public API reshape.
  Date/Author: 2026-03-25 / Codex.

- Decision: internal values are seeded as `Trusted`, while resumed external
  returns are seeded as `Untrusted` before joining in call provenance.
  Rationale: this gives Track B an explicit host-boundary trust downgrade
  without adding policy semantics to Track A. Date/Author: 2026-03-26 / Codex.

## Outcomes & Retrospective

Implemented in `crates/zamburak-monty` with an internal `observer/ifc_state.rs`
runtime-state module, additive public `CallIfcContext`/`GovernedIfcConfig`
APIs, new observer and governed-run unit tests, new integration BDD coverage
for strict versus normal mode and returned-value provenance, and a security
regression proving strict-mode PC taint can deny a constant effect call.

Prototype outcome:

- Track A provides enough signal to build complete call-boundary IFC summaries
  for the supported event classes.
- Control-context lifetime must remain conservative until Track A exposes an
  explicit scope-exit signal.
- Returned external values require Zamburak-owned provenance carry-over between
  `ExternalCallReturned` and the following `OpResult(None)`.

## Context and orientation

The current relevant code and documentation are:

- [crates/zamburak-core/src/dependency_graph.rs](/home/user/project/crates/zamburak-core/src/dependency_graph.rs)
  defines `DependencyGraph`, `GraphBudgets`, `ValueNode`, and `ValueLabels`.
- [crates/zamburak-core/src/summary.rs](/home/user/project/crates/zamburak-core/src/summary.rs)
  computes bounded transitive `DependencySummary` values.
- [crates/zamburak-core/src/control_context.rs](/home/user/project/crates/zamburak-core/src/control_context.rs)
  defines `ExecutionContextSummary` and `EffectCounters`.
- [crates/zamburak-core/src/propagation.rs](/home/user/project/crates/zamburak-core/src/propagation.rs)
  defines `PropagationMode` and `propagate_labels(...)`.
- [crates/zamburak-monty/src/observer.rs](/home/user/project/crates/zamburak-monty/src/observer.rs)
  currently records pending external calls and event counters only.
- [crates/zamburak-monty/src/external_call.rs](/home/user/project/crates/zamburak-monty/src/external_call.rs)
  currently exposes `CallContext { call_id, kind, function_name }` with no IFC
  payload.
- [crates/zamburak-monty/src/run.rs](/home/user/project/crates/zamburak-monty/src/run.rs)
  and
  [crates/zamburak-monty/src/run/flow.rs](/home/user/project/crates/zamburak-monty/src/run/flow.rs)
   mediate external calls but do not derive summaries from observer state.
- [tests/integration/governed_run_bdd.rs](/home/user/project/tests/integration/governed_run_bdd.rs)
  and
  [tests/integration/features/governed_run.feature](/home/user/project/tests/integration/features/governed_run.feature)
   already exercise the governed path and can be extended for IFC-aware
  scenarios.
- [docs/zamburak-design-document.md](/home/user/project/docs/zamburak-design-document.md)
  states that strict mode must include control-context summary in every effect
  check and that `full-monty` observer events are the canonical Track A signal
  surface.

The design document already establishes the canonical event classes:

1. `ValueCreated`
2. `OpResult`
3. `ExternalCallRequested`
4. `ExternalCallReturned`
5. `ControlCondition`

Task 0.6.3 must translate those generic events into Track B state updates. That
means this plan cannot stop at event counting. It must name the runtime state
that is updated, the governed API that exposes the resulting summaries, and the
tests that prove no supported event class is ignored.

### Expected file set

The implementation is expected to touch these existing files and may add a
small number of internal helper modules beneath them:

- [crates/zamburak-monty/src/observer.rs](/home/user/project/crates/zamburak-monty/src/observer.rs)
- [crates/zamburak-monty/src/external_call.rs](/home/user/project/crates/zamburak-monty/src/external_call.rs)
- [crates/zamburak-monty/src/run.rs](/home/user/project/crates/zamburak-monty/src/run.rs)
- [crates/zamburak-monty/src/run/flow.rs](/home/user/project/crates/zamburak-monty/src/run/flow.rs)
- [crates/zamburak-monty/src/lib.rs](/home/user/project/crates/zamburak-monty/src/lib.rs)
- [crates/zamburak-monty/src/observer_tests.rs](/home/user/project/crates/zamburak-monty/src/observer_tests.rs)
- [crates/zamburak-monty/src/run_tests.rs](/home/user/project/crates/zamburak-monty/src/run_tests.rs)
- [tests/integration/governed_run_bdd.rs](/home/user/project/tests/integration/governed_run_bdd.rs)
- [tests/integration/features/governed_run.feature](/home/user/project/tests/integration/features/governed_run.feature)
- [tests/security/main.rs](/home/user/project/tests/security/main.rs)
- [docs/zamburak-design-document.md](/home/user/project/docs/zamburak-design-document.md)
- [docs/users-guide.md](/home/user/project/docs/users-guide.md)
- [docs/roadmap.md](/home/user/project/docs/roadmap.md)

If the internal implementation becomes too large for the existing source files,
extract internal submodules such as:

- `crates/zamburak-monty/src/observer/ifc_state.rs`
- `crates/zamburak-monty/src/observer/control_context.rs`
- `crates/zamburak-monty/src/run/ifc_context.rs`

These are implementation suggestions, not mandatory filenames.

## Design shape to implement

The implementation should aim for one Zamburak-owned runtime state object that
the observer mutates and the governed runner reads. The concrete name can
change, but the structure should be equivalent to:

```rust
pub struct IfcRuntimeState {
    graph: DependencyGraph,
    propagation_mode: PropagationMode,
    control_context: ExecutionContextSummary,
    pending_calls: BTreeMap<u32, PendingCallIfcState>,
    value_labels: BTreeMap<ValueId, ValueLabels>,
    truncated: bool,
}
```

`PendingCallIfcState` should capture the information needed to build an
observer-derived call context:

```rust
pub struct PendingCallIfcState {
    call_id: u32,
    kind: ExternalCallKind,
    function_name: String,
    arg_value_ids: Vec<ValueId>,
    kwarg_value_ids: Vec<(ValueId, ValueId)>,
    arg_summaries: Vec<DependencySummary>,
    kwarg_summaries: Vec<(DependencySummary, DependencySummary)>,
    aggregate_summary: DependencySummary,
    control_context: ExecutionContextSummary,
}
```

The exact field names may differ. The important constraints are:

1. The observer owns the mutable runtime state.
2. `run/flow.rs` consumes read-only snapshots of that state when constructing
   `CallContext`.
3. The API exposed to consumers is Zamburak-owned and additive.
4. Strict-mode behaviour must be controlled by `PropagationMode`, not by
   duplicating a parallel boolean in multiple modules.

### Event-to-state mapping

The implementation target is this mapping:

1. `ValueCreated(value_id)`
   Insert a graph node if absent. Seed it with the current default labels for a
   new internal value. The default must be documented explicitly in the design
   document. If implementation discovers that a different seed is required for
   host inputs or resumed external results, add a Zamburak-owned seeding seam
   rather than hard-coding policy logic.

2. `OpResult(output_id, inputs)`
   Ensure the output node exists, convert all input runtime IDs to `ValueId`,
   and add graph edges from `output_id` to each operand ID represented by
   `OpInputIds`. Unsupported or unknown IDs must fail closed.

3. `ControlCondition(condition_id, branch_taken)`
   Compute the condition summary from the graph and update the active
   `ExecutionContextSummary`. The prototype milestone must first establish how
   the event stream defines dynamic scope. The final implementation must be
   explicit about whether the context is stack-based, frame-based, or
   conservatively segment-based.

4. `ExternalCallRequested(call_id, kind, arg_runtime_ids, kwarg_runtime_ids)`
   Build per-argument and aggregate summaries from the graph. In strict mode,
   join the active control-context summary into the aggregate effect summary.
   Store the result as pending call IFC state keyed by `call_id`.

5. `ExternalCallReturned(call_id, kind)`
   Reconcile the pending call entry and preserve enough provenance to connect
   the resumed result value to the call inputs once the returned value becomes
   visible. The prototype milestone must verify the precise event ordering and
   identifier availability for this step.

## Plan of work

### Stage A: prototype the observer contract before implementation

Initialize the vendored submodule:

```plaintext
git submodule update --init --recursive third_party/full-monty
```

Then inspect the exact event-emission sites for:

- `ControlCondition`
- `ExternalCallRequested`
- `ExternalCallReturned`
- any `ValueCreated` emission associated with resumed external results

Use a small temporary probe or targeted tests to record event order for three
program shapes:

1. pure arithmetic with one `OpResult`,
2. a conditional branch followed by an external call,
3. an external call that resumes with a host result and then flows into another
   operation or effect.

The output of this prototype must answer three go or no-go questions:

1. Does the event stream expose enough information to model active
   control-context lifetime?
2. Does the resumed external result receive a stable value ID that Track B can
   connect back to the original call?
3. Are all value IDs referenced by `OpResult`, `ControlCondition`, and
   external-call argument payloads created before first use?

If any answer is "no", stop here, update this ExecPlan, and escalate with the
precise missing Track A signal. Do not guess.

### Stage B: add IFC runtime state and observer application logic

Refactor
[crates/zamburak-monty/src/observer.rs](/home/user/project/crates/zamburak-monty/src/observer.rs)
 so it owns both the existing queue bookkeeping and the new IFC runtime state.
Avoid making `observer.rs` a god object. Extract helper types or internal
submodules early.

Implementation goals:

1. Add an internal `IfcRuntimeState` type that owns:
   - `DependencyGraph`
   - `GraphBudgets`
   - `PropagationMode`
   - `ExecutionContextSummary`
   - pending call IFC snapshots
   - any required value-label seeding map
2. Convert Track A `RuntimeValueId` to Track B `ValueId` at the boundary.
   Keep the conversion tiny and explicit.
3. Replace the current counter-only `dispatch_event(...)` path with
   event-specific handlers such as:
   - `apply_value_created(...)`
   - `apply_op_result(...)`
   - `apply_control_condition(...)`
   - `apply_external_call_requested(...)`
   - `apply_external_call_returned(...)`
4. Preserve the existing `EventCounts` diagnostics. They are still useful for
   compatibility probes and should remain available.

Do not entangle event application with call mediation. `observer.rs` should
only update state; `run/flow.rs` should read that state later.

### Stage C: expose IFC-aware governed call contexts

Extend
[crates/zamburak-monty/src/external_call.rs](/home/user/project/crates/zamburak-monty/src/external_call.rs)
 with a nested public IFC payload, for example:

```rust
pub struct CallIfcContext {
    pub propagation_mode: PropagationMode,
    pub aggregate_summary: DependencySummary,
    pub control_context: ExecutionContextSummary,
    pub arg_summaries: Vec<DependencySummary>,
    pub kwarg_summaries: Vec<(DependencySummary, DependencySummary)>,
}
```

Then extend `CallContext` to carry it:

```rust
pub struct CallContext {
    pub call_id: u32,
    pub kind: ExternalCallKind,
    pub function_name: String,
    pub ifc: CallIfcContext,
}
```

This gives Task 0.6.4 a stable input surface for policy evaluation and gives
Task 0.6.3 tests a direct way to assert correctness.

Update
[crates/zamburak-monty/src/run/flow.rs](/home/user/project/crates/zamburak-monty/src/run/flow.rs)
 to pull the pending IFC snapshot from observer state when building
`CallContext`. Keep the existing observer-mismatch error, but add explicit
fail-closed handling for:

- missing IFC snapshot for a yielded call,
- unknown runtime IDs referenced by the observer stream,
- graph truncation that forces `unknown_top()` summaries,
- control-context update failures.

If the final result needs a read-only inspection API outside an external-call
yield, add an additive helper such as
`GovernedRunner::run_no_limits_with_ifc_snapshot(...)` rather than changing the
existing `run_no_limits(...)` contract.

### Stage D: make strict mode explicit and testable

Task 0.6.3 is not complete if strict mode exists only in documentation. The
governed path must choose a propagation mode and make it observable in tests.

Implementation steps:

1. Decide where propagation mode is configured for governed execution.
   The expected direction is:
   - derive `GraphBudgets` and `PropagationMode` from policy configuration when
     available,
   - provide a small additive constructor or builder for tests so strict and
     normal mode can both be exercised before Task 0.6.4.
2. When constructing external-call summaries, use
   `zamburak_core::propagation::propagate_labels(...)` so strict-mode control
   behaviour is centralized in the IFC core.
3. Increment `EffectCounters` at the effect boundary once per governed external
   call so context snapshots reflect actual effect history.
4. Document the chosen strict-mode control-context model in
   [docs/zamburak-design-document.md](/home/user/project/docs/zamburak-design-document.md),
    including any conservative limitations discovered during implementation.

### Stage E: build the test matrix before claiming completion

Follow red, green, refactor. Add the failing tests first.

#### Unit tests

Extend or add unit tests under `crates/zamburak-monty/src/` to cover:

1. `ValueCreated` inserts a node exactly once and preserves default labels.
2. `OpResult` creates parent edges for `OpInputIds::{None, One, Two}` and any
   supported multi-input representation discovered in Stage A.
3. `ExternalCallRequested` stores per-argument and aggregate summaries.
4. `ExternalCallReturned` clears or transforms pending call state correctly.
5. Strict mode joins control-context summaries; normal mode does not.
6. Graph truncation or missing IDs yield conservative fail-closed results.
7. Observer drift still surfaces `GovernedRunError::ObserverMismatch` or a more
   precise successor error.

Prefer small `rstest` fixtures and helper structs over large parameter lists.

#### Behavioural integration tests

Extend or add `rstest-bdd` v0.5.0 scenarios under
[tests/integration/](/home/user/project/tests/integration/) for:

1. A pure arithmetic program where a later external call sees operand-derived
   provenance in `CallContext.ifc.aggregate_summary`.
2. A program with an untrusted controlling value where strict mode taints an
   otherwise constant external call through control context.
3. The same program under normal mode, proving the control-context influence is
   absent there.
4. An allowed external call that resumes with a host result and then feeds a
   second effect, proving returned-value provenance is retained.
5. A fail-closed unhappy path where an intentionally incomplete or overflowed
   IFC state surfaces `unknown_top()` or an explicit governed error rather than
   silently producing a clean summary.

The existing governed-run BDD suite is a good home if the scenarios remain
coherent. If not, add a sibling file such as
`tests/integration/governed_ifc_bdd.rs` and a dedicated feature file.

#### Security-style regression

Add at least one security regression under
[tests/security/](/home/user/project/tests/security/) that demonstrates the
control-context requirement from
[docs/verification-targets.md](/home/user/project/docs/verification-targets.md):

- branch on an untrusted condition,
- issue an external call with constant arguments,
- assert that the mediated call context contains untrusted PC influence in
  strict mode.

This can use a test-only mediator that inspects `CallContext.ifc` and returns a
deterministic denial when the context is missing expected taint.

### Stage F: documentation, roadmap sync, and final gates

Update these documents in the same change:

1. [docs/zamburak-design-document.md](/home/user/project/docs/zamburak-design-document.md)
   with the implementation decision for:
   - the runtime-state type used for observer-driven IFC wiring,
   - the default label-seeding rule,
   - the strict-mode control-context lifetime model,
   - any additive governed API introduced for IFC snapshots or context.
2. [docs/users-guide.md](/home/user/project/docs/users-guide.md) with the new
   governed-call IFC context shape and any new constructor or runner helper
   used by library consumers.
3. [docs/roadmap.md](/home/user/project/docs/roadmap.md) by marking Task 0.6.3
   done only after all gates pass.

Run the final commands exactly this way so failures are preserved through `tee`:

```plaintext
set -o pipefail; make fmt | tee /tmp/make-fmt.log
set -o pipefail; make markdownlint | tee /tmp/make-markdownlint.log
set -o pipefail; make nixie | tee /tmp/make-nixie.log
set -o pipefail; make check-fmt | tee /tmp/make-check-fmt.log
set -o pipefail; make lint | tee /tmp/make-lint.log
set -o pipefail; make test | tee /tmp/make-test.log
```

Minimum success evidence:

- the new unit tests pass,
- the new `rstest-bdd` scenarios pass,
- the security-style regression passes,
- root `make check-fmt`, `make lint`, and `make test` pass,
- Task 0.6.3 is marked `[x]` in the roadmap,
- the users guide explains the new governed IFC surface.

## Approval gate

This plan is ready for review but not for execution. Per the `execplans` skill,
implementation should not begin until the user explicitly approves this
ExecPlan or requests revisions.
