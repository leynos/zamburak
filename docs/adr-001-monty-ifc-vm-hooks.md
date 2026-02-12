# ADR: `full-monty` information flow control (IFC) hooks

- Date: 2026-02-11
- Status: Proposed
- Related: `docs/zamburak-design-document.md`
- Related: `docs/roadmap.md`

## Context

Zamburak needs hard runtime control over capability use at the
external-function boundary. Monty already exposes that boundary as an explicit
pause/resume protocol:

- `start()` can pause on an external function request and return a resumable
  snapshot,
- `resume()` continues execution after the host returns an external result,
- both compiled runners and mid-flight snapshots support `dump()` and `load()`
  byte serialization.[^1]

Monty therefore already models the control-plane seam Zamburak needs. What it
lacks for Zamburak's full IFC design is stable value identity and generic
runtime instrumentation that can survive suspension and restore.

Zamburak also already has an opinionated policy core in this repository,
including explicit policy schema evolution and migration audit records.[^2]

The architectural question is not whether to separate policy from interpreter
internals. The question is how to separate them so that:

- the Monty delta stays small and upstream-PR-able,
- Zamburak policy and IFC can evolve quickly without a large fork burden,
- snapshot durability does not break provenance continuity.

## Decision

Adopt a two-track architecture with an explicitly constrained Monty fork named
`full-monty`.

- `Track A` is `full-monty` as a generic interpreter substrate.
  - Scope: stable runtime IDs, lightweight observer hooks, and an optional
    generic snapshot-extension mechanism.
  - Constraint: no Zamburak semantics or nomenclature.
- `Track B` is Zamburak runtime governance.
  - Scope: IFC dependency graph, policy evaluation, allow/deny/confirmation
    semantics, audit explanations, and adapter behaviour.
  - Constraint: evolves independently of Monty internals.

`full-monty` will be integrated as a Git submodule in the Zamburak repository
and maintained with a strict "keep the diff PR-able" policy.

## Track A requirements (`full-monty`, upstream-friendly)

### A1. Stable, host-only runtime IDs

`full-monty` must provide stable runtime identity (for example `ValueId`) that
is:

- unique per execution,
- stable across `start()` and `resume()`,
- stable across snapshot `dump()` and `load()`.

IDs are host-facing instrumentation data, not guest-Python writable state.

### A2. Lightweight event emission hooks

`full-monty` must expose a generic observer interface with zero-cost defaults.

Requirements:

- when no observer is installed, behaviour remains unchanged with negligible
  overhead,
- no per-event heap allocation on hot paths,
- events are generic and tooling-focused, not policy-focused.

Minimum event set:

- `ExternalCallRequested`,
- `ExternalCallReturned`,
- `ValueCreated`,
- `OpResult` (output ID plus input IDs),
- `ControlCondition` (needed for strict-mode control influence).

These event classes are the canonical Track A event surface and are reused by
`docs/roadmap.md` Task `0.5.2` to avoid drift between planning and ADR
requirements.

### A3. Snapshot extension seam (optional but preferred)

If Zamburak IFC state is not serialized directly in Monty runtime state,
`full-monty` should expose a generic snapshot-extension byte blob owned by the
embedder.

Monty must not interpret this blob. Zamburak owns its versioning and decoding.

### A4. Compatibility and upstreamability invariants

- hook-disabled mode preserves upstream Monty semantics,
- no-op observer mode preserves semantics with understood overhead bounds,
- any new public API is additive and generic,
- no `taint`, `policy`, `capabilities`, or `Zamburak` naming in `Track A`
  surface APIs.

## Track B requirements (Zamburak-owned governance)

### B1. IFC substrate

Implement `ValueId`-keyed dependency tracking with:

- direct dependency DAG,
- bounded transitive summary path,
- `Normal` and `Strict` propagation modes.

Strict mode adds control-context influence to effect decisions.

### B2. Boundary enforcement at external calls

Treat every external function call as a mandatory policy sink decision.

At each external boundary:

- compute argument dependency summaries and context summaries,
- evaluate against `zamburak-policy`,
- return one of the supported outcomes (allow, deny, confirmation, or other
  policy-defined gated path).

### B3. Durable and versioned IFC state

IFC state serialization is versioned, explicit, and round-trip tested. Snapshot
restore must preserve governance semantics, not only interpreter state.

### B4. Auditable decisions

Every deny or confirmation path must include:

- matched rule identifier,
- implicated value and origin identities,
- human-readable, redacted explanation.

## Implementation plan

### Step 0: Repository mechanics and guardrails

- add `full-monty` as a Git submodule at `third_party/full-monty/`,
- add `docs/monty-fork-policy.md` with allowed fork-change categories,
- add `make monty-sync` to fetch upstream Monty, sync fork branch, and run
  Monty plus Zamburak integration checks.

### Track A staged pull requests

- PR A1: stable runtime IDs with snapshot continuity tests,
- PR A2: observer/event trait with no-op parity tests,
- PR A3: snapshot extension seam (if needed for external IFC state).

### Track B staged pull requests

- PR B1: add `zamburak-monty` adapter crate and governed run API,
- PR B2: add IFC core crate (`zamburak-ifc` or equivalent) with pure ID-based
  propagation logic,
- PR B3: connect observer events to IFC updates,
- PR B4: enforce policy at external-call boundary,
- PR B5: add compatibility, security, and snapshot-governance regression suites.

## Process requirements to keep the fork PR-able

### Patch budget

Allow only:

- stable IDs,
- generic hook substrate,
- optional generic snapshot extension,
- narrowly necessary refactors that directly enable the above.

Reject all Zamburak-specific semantics in `full-monty`.

### Upstream-shaped commits

- small, test-complete commits,
- each commit must provide generic Monty value independent of Zamburak,
- each fork PR should be reviewable as a standalone upstream candidate.

### Continuous range-diff control

On each upstream sync:

- run `git range-diff` between previous and current fork deltas,
- investigate any sudden delta growth before continuing feature work.

## Consequences

Positive:

- preserves a clean separation between interpreter substrate and governance
  semantics,
- reduces long-lived fork risk,
- supports fast policy and IFC iteration in Zamburak,
- keeps snapshot durability as a first-class security requirement.

Negative:

- introduces an additional repository-management surface (submodule plus sync
  procedure),
- requires discipline to keep `full-monty` generic and upstream-shaped.

## References

[^1]: Pydantic Monty API references pinned to immutable revision
    `20d9d27bda234336e077c673ca1a2e713f2e787f`:
    <https://github.com/pydantic/monty/blob/20d9d27bda234336e077c673ca1a2e713f2e787f/crates/monty/src/run.rs#L146-L232>
    and
    <https://github.com/pydantic/monty/blob/20d9d27bda234336e077c673ca1a2e713f2e787f/crates/monty/src/run.rs#L613-L688>.
[^2]: Zamburak policy migration work:
    <https://github.com/leynos/zamburak/pull/8>
    and
    <https://github.com/leynos/zamburak/pull/8/files>.
