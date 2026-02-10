# Architectural decision record: Monty VM hooks for information flow control (IFC)

- Date: 2026-02-09
- Status: Proposed
- Related: `docs/zamburak-design-document.md`

## Context

Zamburak's design requires a Monty VM hook layer with no bypass around
information flow control (IFC), complete opcode propagation coverage, and
strict mode support for control-context summaries at every effect boundary.[^1]

This architectural decision record (ADR) assesses the current extension
surfaces in `pydantic/monty` and chooses an implementation strategy that
minimizes long-term fork maintenance cost while still meeting the design
contract.

Source analysis in this ADR is pinned to Monty commit
`20d9d27bda234336e077c673ca1a2e713f2e787f` (main branch, 2026-02-09).

### Requirement traceability to the system design

To make requirement lineage explicit, key statements in this ADR map to these
specific sections in `docs/zamburak-design-document.md`:

- no-bypass VM hook expectation:
  [component responsibilities](docs/zamburak-design-document.md#component-responsibilities),
   especially the `Monty VM hook layer` invariants,
- complete opcode propagation coverage and defect posture for missing rules:
  [supported language subset](docs/zamburak-design-document.md#supported-language-subset),
- strict-mode control-context requirements for effect decisions:
  [strict-mode effect semantics](docs/zamburak-design-document.md#strict-mode-effect-semantics),
- snapshot and resume behavioural equivalence requirements:
  [snapshot and resume semantics](docs/zamburak-design-document.md#snapshot-and-resume-semantics),
- conservative handling when summary information is incomplete:
  [dependency representation](docs/zamburak-design-document.md#dependency-representation)
   and [fail-closed rules](docs/zamburak-design-document.md#fail-closed-rules).

## Findings from source exploration

### 1. Existing hook and instrumentation mechanisms in Monty

Monty currently exposes host boundary hooks, but not opcode-level VM
instrumentation:

- `FrameExit::{ExternalCall, OsCall, ResolveFutures}` lets the host intercept
  external calls and OS operations via pause/resume flow.[^2]
- `RunProgress` and `Snapshot` expose these boundary pauses in the public run
  API.[^3]
- `ResourceTracker` and `PrintWriter` allow resource metering and output capture
  callbacks.[^4]

Monty does not currently expose:

- a public bytecode observer hook,
- a per-opcode callback interface,
- a public extension surface around `VM::run` dispatch,
- or a public VM module API that downstream crates can instrument directly.[^5]

### 2. Can this be done without forking or vendoring Monty?

- Partial capability (effect-boundary checks only): yes, without forking.
- Full IFC semantics required by Zamburak design: not with the current public
  Monty API.

Reason: host-boundary callbacks cannot provide complete value dependency
tracking, control-context propagation, or opcode coverage guarantees. Those
require VM-internal event points.

### 3. Best way to augment execution-flow and data-model information

The most effective path is an upstream-first, minimal hook API in Monty, then
an out-of-tree IFC engine in Zamburak:

- keep hook substrate small and generic in Monty core,
- keep policy and IFC semantics in Zamburak-owned crates,
- avoid a large behavioural fork of the interpreter.

This aligns with Monty being explicitly experimental and fast-moving.[^6]

## Options considered

### Option A: Boundary-only enforcement with current Monty APIs

- Description: use `RunProgress::FunctionCall` and `RunProgress::OsCall` only.
- Pros: no Monty changes, no fork.
- Cons: cannot satisfy complete IFC propagation and no-bypass requirements.
- Verdict: Rejected as primary architecture; acceptable only as a temporary
  bootstrap for effect-policy prototyping.

### Option B: Long-lived Zamburak fork with deep IFC integration in VM

- Description: embed IFC metadata directly into Monty runtime internals and
  maintain a private branch indefinitely.
- Pros: immediate full control.
- Cons: high sync overhead, fragile against upstream internal refactors, larger
  trusted computing base churn.
- Verdict: Rejected as default path due to project-health risk.

### Option C: Upstream-first hook substrate plus Zamburak IFC adapter

- Description: propose and upstream a compact VM hook interface; implement IFC,
  summaries, and policy in Zamburak crates using that interface.
- Pros: low long-term divergence, preserves upstream velocity, supports full IFC
  requirements.
- Cons: needs upstream collaboration and an interim patch branch.
- Verdict: Accepted.

### Option D: Bytecode/source rewriting outside Monty

- Description: rewrite Monty code or bytecode to inject IFC logic without VM
  changes.
- Pros: avoids modifying Monty core directly.
- Cons: brittle, incomplete against dynamic semantics and builtins, difficult to
  prove no bypass.
- Verdict: Rejected.

## Decision

Adopt Option C:

- Build a minimal VM hook API upstream in `pydantic/monty`.
- Keep IFC engine and policy logic in Zamburak-owned code.
- Use a short-lived patch branch until upstream merge; do not vendor.

## Temporary fork constraints

The short-lived fork exists only to bridge the time between local integration
and upstream acceptance. The fork must be managed under strict constraints to
maximize mergeability.

### Scope constraints

- Fork changes are limited to hook substrate and tests needed to prove its
  correctness.
- No Zamburak policy semantics, label models, or product-specific behaviour may
  be introduced in the fork.
- No broad refactors, style churn, renames, or opportunistic clean-ups may be
  included in fork pull requests.
- If a required change is not directly about hook API enablement, it must be
  proposed as an independent upstream contribution first.

### Drift constraints

- Fork `main` must remain within 7 calendar days of upstream `main`.
- Fork pull requests must be rebased to current upstream `main` before merge.
- If drift exceeds 7 days, feature work pauses until rebase and regression
  checks are completed.
- If drift exceeds 14 days, the fork is treated as off-track and requires an
  explicit maintainer review before any new feature merge.

### Standards and compatibility constraints

- Fork code must follow upstream Monty coding, lint, formatting, test, and CI
  standards exactly.
- Public API changes must be additive where possible, with defaults that
  preserve existing behaviour when hooks are not configured.
- Hook event payloads must avoid exposing private interpreter internals as API
  commitments.
- Performance impact must be measured, and the default no-hook path must remain
  near-zero overhead.

### Merge readiness constraints

- Every fork change must map to a corresponding upstream PR or draft PR.
- Fork commits should be logically small and reviewable in isolation.
- Each commit message must state why the change is required for generic Monty
  capability rather than Zamburak-specific behaviour.
- Before removing the fork, all required patches must be merged upstream or
  explicitly superseded by upstream alternatives.

### Fork retirement and upstream-only cutover criteria

The temporary fork may be deleted only when all criteria below are met:

- All fork-required upstream PRs are merged or closed as superseded by accepted
  upstream alternatives.
- A stable Monty release (non-prerelease) includes those merged changes.
- Zamburak runs against that release with all relevant verification suites
  passing, including snapshot and resume equivalence checks and policy gate
  regression tests.
- No local compatibility shims remain that depend on fork-only APIs.
- Dependency references in build and release configuration point only to
  upstream Monty sources.
- Documentation and operational runbooks are updated to declare upstream-only
  support and remove fork procedures.

Minimum stability window before deleting fork infrastructure:

- At least one full Zamburak release cycle on upstream-only Monty, with no
  critical regressions attributed to hook substrate changes.

## Proposed Monty extension surface

Expose an optional hook trait passed into VM execution:

```rust
pub trait VmHooks {
    fn on_opcode_start(&mut self, event: OpcodeStartEvent) {}
    fn on_opcode_end(&mut self, event: OpcodeEndEvent) {}
    fn on_branch(&mut self, event: BranchEvent) {}
    fn on_effect_boundary(&mut self, event: EffectBoundaryEvent) {}
    fn on_snapshot(&mut self, event: SnapshotEvent) {}
}
```

Expose stable, low-coupling event payloads:

- `frame_id`, `task_id`, `code_object_id`, `ip`, `opcode`
- opaque `value_id` handles (not raw `Value` internals)
- operation role metadata (`read`, `write`, `mutate`, `call`, `return`)
- effect boundary linkage (`call_id`, external function id or OS function)

Design rule:

- event payloads must be sufficient for IFC dependency construction and strict
  control-context tracking, but avoid exposing Monty private internals as API.

### Concrete opcode event examples

The examples below are illustrative event sequences showing that the proposed
fields are sufficient for IFC while remaining generic.

Example A: branch opcode (`JumpIfFalse`)

```json
{
  "hook": "on_opcode_start",
  "frame_id": "f3",
  "task_id": "t0",
  "code_object_id": "co_main",
  "ip": 418,
  "opcode": "JumpIfFalse",
  "reads": ["v_predicate"]
}
{
  "hook": "on_branch",
  "frame_id": "f3",
  "task_id": "t0",
  "ip": 418,
  "opcode": "JumpIfFalse",
  "predicate_value_id": "v_predicate",
  "taken": true,
  "target_ip": 442,
  "fallthrough_ip": 421
}
{
  "hook": "on_opcode_end",
  "frame_id": "f3",
  "task_id": "t0",
  "ip": 418,
  "opcode": "JumpIfFalse",
  "result": "ok"
}
```

IFC consequence:

- `predicate_value_id` is added to the active control-context dependencies for
  subsequent effect checks in strict mode.

Example B: call opcode yielding an external effect (`CallFunction`)

```json
{
  "hook": "on_opcode_start",
  "frame_id": "f3",
  "task_id": "t0",
  "code_object_id": "co_main",
  "ip": 512,
  "opcode": "CallFunction",
  "callable_value_id": "v_callable_send_email",
  "arg_value_ids": ["v_to", "v_body"]
}
{
  "hook": "on_effect_boundary",
  "frame_id": "f3",
  "task_id": "t0",
  "ip": 512,
  "opcode": "CallFunction",
  "effect_kind": "external_call",
  "call_id": 17,
  "external_function_id": "send_email",
  "arg_value_ids": ["v_to", "v_body"],
  "control_context_value_ids": ["v_predicate"]
}
{
  "hook": "on_opcode_end",
  "frame_id": "f3",
  "task_id": "t0",
  "ip": 512,
  "opcode": "CallFunction",
  "result": "yield_external",
  "call_id": 17
}
{
  "hook": "on_snapshot",
  "reason": "external_call",
  "call_id": 17
}
```

IFC consequence:

- argument and control-context summaries can be computed using only opaque value
  identifiers and emitted effect metadata, without exposing private VM layout.

## Consequences

Positive:

- meets Zamburak IFC requirements with auditable coverage,
- minimizes long-term fork maintenance burden,
- keeps interpreter correctness and performance work in upstream Monty.

Negative:

- requires initial upstream API negotiation,
- requires maintaining a temporary patch branch until merge.

## Risk management

- Keep initial Monty patch small and mechanical: hook plumbing only.
- Keep IFC logic entirely outside Monty core.
- Add coverage tests proving every supported opcode emits required hook events.
- Add snapshot/restore invariance tests for hook state continuity.
- Gate upgrades on a compatibility test suite pinned to Monty versions.

## Implementation phases

1. Prototype phase: use current boundary hooks for policy gateway integration
   and audit schema plumbing.
2. Hook substrate phase: upstream `VmHooks` API and event types.
3. IFC phase: implement propagation engine in Zamburak over hook events.
4. Hardening phase: opcode coverage matrix, differential tests, and performance
   budgets from the design document.[^1]

## References

[^1]: Zamburak IFC and VM-hook requirement anchors:
    [component responsibilities](docs/zamburak-design-document.md#component-responsibilities),
    [supported language subset](docs/zamburak-design-document.md#supported-language-subset),
    [strict-mode effect semantics](docs/zamburak-design-document.md#strict-mode-effect-semantics),
    and
    [snapshot and resume semantics](docs/zamburak-design-document.md#snapshot-and-resume-semantics).
[^2]: Monty VM boundary exits and dispatch loop:
    <https://github.com/pydantic/monty/blob/20d9d27bda234336e077c673ca1a2e713f2e787f/crates/monty/src/bytecode/vm/mod.rs#L195-L234>
    and
    <https://github.com/pydantic/monty/blob/20d9d27bda234336e077c673ca1a2e713f2e787f/crates/monty/src/bytecode/vm/mod.rs#L660-L760>.
[^3]: Monty public pause/resume API:
    <https://github.com/pydantic/monty/blob/20d9d27bda234336e077c673ca1a2e713f2e787f/crates/monty/src/run.rs#L146-L232>
    and
    <https://github.com/pydantic/monty/blob/20d9d27bda234336e077c673ca1a2e713f2e787f/crates/monty/src/run.rs#L613-L688>.
[^4]: Extensible callbacks currently exposed:
    <https://github.com/pydantic/monty/blob/20d9d27bda234336e077c673ca1a2e713f2e787f/crates/monty/src/resource.rs#L104-L155>
    and
    <https://github.com/pydantic/monty/blob/20d9d27bda234336e077c673ca1a2e713f2e787f/crates/monty/src/io.rs#L5-L28>.
[^5]: Monty public exports and non-export of VM internals:
    <https://github.com/pydantic/monty/blob/20d9d27bda234336e077c673ca1a2e713f2e787f/crates/monty/src/lib.rs#L3-L40>.
[^6]: Monty project maturity statement:
    <https://github.com/pydantic/monty/blob/20d9d27bda234336e077c673ca1a2e713f2e787f/README.md#L19-L33>.
