# Architectural decision record: Monty VM hooks for IFC

- Date: 2026-02-09
- Status: Proposed
- Related: `docs/zamburak-design-document.md`

## Context

Zamburak's design requires a Monty VM hook layer with no bypass around
information flow control (IFC), complete opcode propagation coverage, and
strict mode support for control-context summaries at every effect boundary.[^1]

This ADR assesses the current extension surfaces in `pydantic/monty` and
chooses an implementation strategy that minimizes long-term fork maintenance
cost while still meeting the design contract.

Source analysis in this ADR is pinned to Monty commit
`20d9d27bda234336e077c673ca1a2e713f2e787f` (main branch, 2026-02-09).

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
- Verdict: Rejected as default path due project-health risk.

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

[^1]: Zamburak IFC and VM hook requirements:
    `docs/zamburak-design-document.md`.
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
