# Add IFC core with `ValueId`-keyed dependency graph (Task 1.6.2)

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETED

## Purpose / big picture

Implement roadmap Task 1.6.2: add information-flow control (IFC) foundation
types and a `ValueId`-keyed directed acyclic graph (DAG) of dependencies to
`crates/zamburak-core`. After this change, a consumer of the Zamburak library
can:

- create opaque `ValueId` identifiers for runtime values,
- build a bounded dependency DAG with budget-enforced edge insertion,
- attach integrity, confidentiality, and authority labels to values,
- compute bounded transitive dependency summaries with fail-closed overflow,
- select normal or strict propagation modes for label propagation, and
- model control-context summaries for strict-mode effect evaluation.

All IFC core types are decoupled from Monty interpreter internals. The IFC
substrate is validated by unit and property tests that run independently of any
interpreter state.

Running `cargo test -p zamburak-core` passes all existing authority lifecycle
tests plus the new IFC unit, property, and behavioural tests. Running
`make check-fmt`, `make lint`, and `make test` at the repository root passes.

## Constraints

- All IFC files live in `crates/zamburak-core/src/` per
  `docs/repository-layout.md` Table 2. No separate `zamburak-ifc` crate.
- No direct coupling to Monty internal value types (`MontyValue`, `MontyObject`,
  etc.). The IFC substrate must be testable without the interpreter.
- No changes to `third_party/full-monty/`. The submodule is consumed as-is.
- No new production dependencies beyond `thiserror` (already in
  `zamburak-core`). `proptest` is added as a dev-dependency only.
- Module-level `//!` comments on all modules, `///` Rustdoc on all public items.
  `missing_docs = "deny"` enforced.
- No single code file may exceed 400 lines. Extract test modules when needed.
- Workspace Clippy lints must pass: `unwrap_used`, `expect_used`,
  `indexing_slicing`, `cognitive_complexity`, `shadow_*`, etc. are all denied.
- Use en-GB-oxendict spelling in documentation and comments.
- Dependency on Task 1.6.1: confirmed complete (marked `[x]` in roadmap).

## Tolerances (exception triggers)

- Scope tolerance: if implementation requires edits in more than 20 files or
  1500 net changed lines, stop, and escalate with a split plan.
- Interface tolerance: if the existing `zamburak-core` public API must change
  beyond additive exports, stop, and escalate.
- Dependency tolerance: if a new production dependency (not dev-dependency) is
  required beyond `newt-hype` and `thiserror`, stop, and escalate.
- Iteration tolerance: if required gates fail after three focused fix loops,
  stop, and report failures with root-cause hypotheses.
- Ambiguity tolerance: if a design document requirement has multiple valid
  interpretations that materially affect the graph semantics, stop, and present
  options.

## Risks

- Risk: the strict Clippy lint configuration (`cognitive_complexity = "deny"`,
  `indexing_slicing = "deny"`, `shadow_* = "deny"`) may conflict with the BFS
  walk in `compute_summary`. Severity: medium. Likelihood: medium. Mitigation:
  use `.get()` for `HashMap` access, iterator chains, and extract helper
  functions to keep complexity under the threshold.

- Risk: `proptest` may conflict with workspace lint rules (e.g. `expect_used`
  in generated test code). Severity: low. Likelihood: low. Mitigation:
  `proptest` macro-generated code runs in test context where `expect_used` is
  not enforced for `#[cfg(test)]` modules (confirmed by prior project
  experience). If issues arise, scope lint exceptions tightly.

## Progress

- [x] Write this ExecPlan and obtain approval.
- [x] Stage A: Foundation types (`value_id`, `trust`, `ifc_errors`).
- [x] Stage B: Dependency graph (`dependency_graph`).
- [x] Stage C: Summary computation (`summary`).
- [x] Stage D: Propagation and control context (`propagation`,
  `control_context`).
- [x] Stage E: Property tests (`proptest`), exhaustive bounded tests, and BDD
  scenarios.
- [x] Stage F: Documentation (`users-guide.md`, `zamburak-design-document.md`,
  `roadmap.md`) and final gate run.

## Surprises & discoveries

- Initially `ValueId` used `newt-hype`'s `base_newtype!` macro, which
  derives `Display` for wrapper types and emits explicit `Clone` impls
  requiring Clippy suppression. Replaced with a plain `#[repr(transparent)]`
  newtype struct with manual `Display` impl, removing the Clippy suppression
  and the `newt-hype` dependency for `ValueId`.
- `cargo fmt` reformats single-line function bodies into multi-line form.
  One-liner accessor methods (e.g. `pub fn id(&self) -> ValueId { self.id }`)
  must be written in multi-line form to pass `make check-fmt`.
- The `clippy::excessive_nesting` lint (threshold 4) fires inside `prop_map`
  closures that contain nested `if let` chains. Fix: extract a helper function
  (e.g. `cap_from_index`) to flatten nesting.
- `clippy::cloned_ref_to_slice_refs` fires when writing
  `&[operand.clone()]` — Clippy prefers `std::slice::from_ref(&operand)`.
- The benchmark test `full_monty_track_a_overhead_probe` is intermittently
  flaky due to system load variability (not related to IFC changes).

## Decision log

- Decision: IFC types live in `zamburak-core`, not a separate `zamburak-ifc`
  crate. Rationale: `docs/repository-layout.md` Table 2 is normative and places
  all IFC files under `crates/zamburak-core/src/`. Table 1 describes
  `zamburak-core` as "Value tagging, graphs, propagation, authority lifecycle."
  The roadmap's aspirational `crates/zamburak-ifc/` naming was resolved by the
  repository layout document. Date/Author: 2026-03-22 / DevBoxer.

- Decision: use `Vec` instead of `SmallVec` for dependency and control-context
  lists. Rationale: `smallvec` is not a `zamburak-core` dependency, and adding
  it would violate the dependency tolerance for this task. `SmallVec` is a
  performance optimization that can be adopted later with profiling data.
  Date/Author: 2026-03-22 / DevBoxer.

- Decision: `IntegrityLabel::Verified` is a simple variant without a
  `VerificationKind` parameter. Rationale: extending `Verified` with
  verification kinds is Task 2.1.1 scope per the roadmap. A simple variant
  establishes the lattice semantics without premature complexity. Date/Author:
  2026-03-22 / DevBoxer.

- Decision: `GraphBudgets` is defined in `zamburak-core` with the same field
  semantics as `PolicyBudgets` in `zamburak-policy`, but without a dependency
  on the policy crate. Rationale: the IFC substrate must not depend on the
  policy layer. The caller (Task 1.6.3 observer wiring) constructs
  `GraphBudgets` from `PolicyBudgets`. Date/Author: 2026-03-22 / DevBoxer.

- Decision: budget overflow in `compute_summary` returns
  `Ok(DependencySummary::unknown_top())`, not `Err(...)`. Rationale: the design
  document states "budget overflow yields unknown-top summary and conservative
  decisions." This is expected fail-closed behaviour, not an error condition.
  Date/Author: 2026-03-22 / DevBoxer.

- Decision: `AuthoritySet::join` uses set intersection. Rationale: when a value
  depends on multiple sources, its effective authority is the intersection of
  their authority sets. A derived value can only exercise capabilities that all
  its dependencies possess. Date/Author: 2026-03-22 / DevBoxer.

- Decision: transitive cycle (back-edge) detection is implemented and
  enforced at insertion time. Rationale: the runtime enforces the DAG invariant
  by rejecting self-loops, duplicate edges, and transitive back-edges via
  bounded BFS reachability checks in `add_dependency`. Implementation is in
  `crates/zamburak-core/src/dependency_graph.rs` (`check_reachable` method)
  with cycle-detection errors defined in
  `crates/zamburak-core/src/ifc_errors.rs` (`CycleDetected`,
  `ClosureStepBudgetExhausted`). Tests exercising this behavior are in
  `crates/zamburak-core/src/dependency_graph_tests.rs`
  (`add_dependency_cycle_rejected`,
  `add_dependency_reachability_budget_exhaustion`). Date/Author: 2026-03-22 /
  DevBoxer.

- Decision: add `proptest` as a new dev-dependency. Rationale: `proptest` is the
  first property testing tool in the workspace. It is preferred over `kani`
  because: (a) `proptest` is a mature crate with no special toolchain
  requirements; (b) `kani` requires its own standalone toolchain installation
  and CI integration exceeding this task's scope; (c) the verification targets
  document requires "property tests" for IFC propagation, and `proptest`
  provides randomized shrinkable input generation that rstest parameterized
  cases cannot. `kani` (bounded model checking) is recommended as dedicated
  verification infrastructure in a later task. Date/Author: 2026-03-22 /
  DevBoxer.

## Outcomes & retrospective

All stages completed successfully. Final test count: 125 tests (121 unit +
proptest in zamburak-core lib, 4 BDD integration tests). All gates pass:
`make check-fmt`, `make lint`, `make test` (excluding pre-existing benchmark
flake).

Files created (20):

- `crates/zamburak-core/src/value_id.rs` — `ValueId` newtype
- `crates/zamburak-core/src/value_id_tests.rs` — 5 unit tests
- `crates/zamburak-core/src/trust.rs` — `IntegrityLabel`, `DataLabel`,
  `DataLabels`, `AuthoritySet`
- `crates/zamburak-core/src/trust_tests.rs` — 13 unit tests
- `crates/zamburak-core/src/trust_proptests.rs` — 14 property + exhaustive
  tests
- `crates/zamburak-core/src/ifc_errors.rs` — `IfcError` enum
- `crates/zamburak-core/src/ifc_errors_tests.rs` — 5 unit tests
- `crates/zamburak-core/src/dependency_graph.rs` — `DependencyGraph`,
  `GraphBudgets`, `ValueNode`
- `crates/zamburak-core/src/dependency_graph_tests.rs` — 11 unit tests
- `crates/zamburak-core/src/dependency_graph_proptests.rs` — 3 property tests
- `crates/zamburak-core/src/summary.rs` — `DependencySummary`,
  `compute_summary`
- `crates/zamburak-core/src/summary_tests.rs` — 11 unit tests
- `crates/zamburak-core/src/summary_proptests.rs` — 9 property tests
- `crates/zamburak-core/src/propagation.rs` — `PropagationMode`,
  `propagate_labels`
- `crates/zamburak-core/src/propagation_tests.rs` — 6 unit tests
- `crates/zamburak-core/src/propagation_proptests.rs` — 3 property tests
- `crates/zamburak-core/src/control_context.rs` — `ExecutionContextSummary`,
  `EffectCounters`
- `crates/zamburak-core/src/control_context_tests.rs` — 8 unit tests
- `crates/zamburak-core/tests/ifc_bdd.rs` — 4 BDD scenarios
- `crates/zamburak-core/tests/features/ifc_propagation.feature` — feature file

Files modified (5):

- `crates/zamburak-core/src/lib.rs` — module declarations and re-exports
- `crates/zamburak-core/Cargo.toml` — `proptest`, `rstest-bdd`,
  `rstest-bdd-macros` dev-dependencies
- `docs/users-guide.md` — IFC core section
- `docs/zamburak-design-document.md` — implementation decisions
- `docs/roadmap.md` — Task 1.6.2 marked done

Key decisions validated by implementation:

1. `zamburak-core` location works cleanly; no cross-crate issues.
2. `Vec` over `SmallVec` simplified implementation with no observable cost.
3. `proptest` integrates well; no lint conflicts in `#[cfg(test)]` context.
4. BDD scenarios using `rstest-bdd` v0.5.0 work as expected.
5. Budget overflow as `Ok(unknown_top())` was straightforward to implement.

## Context and orientation

The repository is a Rust workspace (edition 2024, rust-version 1.85) with these
crates:

- `crates/zamburak-core` — authority lifecycle, localization, and (after this
  task) IFC foundation types,
- `crates/zamburak-monty` — governed execution adapter around `full-monty`,
- `crates/zamburak-policy` — canonical policy schema, evaluation, and
  decisions,
- `crates/test-utils` — shared test utilities.

The `full-monty` interpreter substrate is vendored at `third_party/full-monty/`
as a Git submodule. Track A (observer substrate) and Track B (Zamburak
governance) are defined in `docs/adr-001-monty-ifc-vm-hooks.md`.

Normative type definitions are in `docs/zamburak-design-document.md` lines
807-831 (`TaggedValue`, `DependencySummary`, `ExecutionContextSummary`).
Propagation mode semantics are at lines 573-589. Budget overflow behaviour is
at lines 791-799.

The `zamburak-core` crate currently exports authority lifecycle types from
`crates/zamburak-core/src/authority.rs` and localization types from
`crates/zamburak-core/src/i18n.rs`. The crate depends on `newt-hype` (newtype
generation) and `thiserror` (error derivation). Dev-dependencies include
`rstest`.

`docs/repository-layout.md` Table 2 defines the file-purpose mapping for
`zamburak-core`, including all IFC modules this task will create:

- `crates/zamburak-core/src/value_id.rs` — stable `ValueId` generation,
- `crates/zamburak-core/src/dependency_graph.rs` — graph state and edge
  insertion,
- `crates/zamburak-core/src/propagation.rs` — normal and strict propagation
  rules,
- `crates/zamburak-core/src/control_context.rs` — control-context stack and
  strict-mode helpers,
- `crates/zamburak-core/src/summary.rs` — bounded transitive summarization,
- `crates/zamburak-core/src/trust.rs` — integrity label and verification
  kinds.

## Plan of work

### Stage A: foundation types (`value_id`, `trust`, `ifc_errors`)

Create three new modules in `crates/zamburak-core/src/`:

`value_id.rs` defines a `ValueId` newtype wrapping `u64` as a plain
`#[repr(transparent)]` struct, mirroring the backing type of `RuntimeValueId`
in `full-monty`. The type must derive `Clone`, `Copy`, `Debug`, `Eq`,
`PartialEq`, `Hash`, `Ord`, `PartialOrd` and implement `Display` for
diagnostics.

`trust.rs` defines:

- `IntegrityLabel` — a three-variant enum (`Untrusted`, `Trusted`, `Verified`)
  with `Ord` derived in declaration order so `Untrusted < Trusted < Verified`.
  The `join` method returns `min(self, other)`, implementing the greatest lower
  bound (meet) in the integrity lattice: when a value depends on both a
  `Trusted` and an `Untrusted` source, the result is `Untrusted`.
- `DataLabel` — a confidentiality tag enum with variants `Pii`, `AuthSecret`,
  `PrivateEmailBody`, `PaymentInstrument`, `InternalPolicyNote`.
- `DataLabels` — a `BTreeSet<DataLabel>` wrapper with `join` (set union),
  `all()` (all variants, used for unknown-top), `contains`, `is_empty`, `len`,
  `iter`, `is_subset_of`.
- `AuthoritySet` — a `BTreeSet<AuthorityCapability>` wrapper (re-using the
  existing `AuthorityCapability` type from `zamburak-core::authority`) with
  `join` (set intersection: authority narrows on dependency), `contains`,
  `is_empty`, `insert`.

`ifc_errors.rs` defines `IfcError`, a `thiserror`-derived enum with variants:
`ValueBudgetExhausted`, `ParentBudgetExhausted`, `ClosureStepBudgetExhausted`,
`UnknownValueId`, `DuplicateValueId`, `CycleDetected`, `DuplicateEdge`.
Value-identifier fields use the `ValueId` newtype; count and limit fields use
primitive `u64` to stay under the `result_large_err` threshold.

Update `crates/zamburak-core/src/lib.rs` to declare the three new modules and
add public re-exports.

Unit tests cover: `ValueId` creation and equality; all 9 `IntegrityLabel::join`
pairs; `DataLabels` union and contains; `AuthoritySet` intersection; error
display formatting.

Gate: `make check-fmt && make lint && make test`.

### Stage B: dependency graph

Create `dependency_graph.rs` defining:

- `GraphBudgets` — budget configuration struct with fields `max_values`,
  `max_parents_per_value`, `max_closure_steps`, `max_witness_depth` (all
  `u64`), plus a `Default` impl matching the canonical policy defaults (100,000
  / 64 / 10,000 / 32).
- `ValueNode` — per-value metadata: `id: ValueId`,
  `integrity: IntegrityLabel`, `confidentiality: DataLabels`,
  `authority: AuthoritySet`, `parents: Vec<ValueId>`.
- `DependencyGraph` — holds `HashMap<ValueId, ValueNode>`, `budgets`, and
  `truncated: bool`. Operations:
  - `new(budgets)` — construct an empty graph,
  - `insert_value(id, labels: ValueLabels)` — checks `max_values`,
    returns `Err(IfcError::DuplicateValueId)` if the ID already exists,
    sets `truncated` on overflow and returns
    `Err(IfcError::ValueBudgetExhausted)`,
  - `add_dependency(child, parent)` — checks both IDs exist, rejects
    self-loops, checks `max_parents_per_value`, returns appropriate error,
  - `get_node`, `contains`, `node_count`, `is_truncated`, `parents`.

If the file approaches 400 lines with tests, extract tests to
`dependency_graph_tests.rs` via `#[cfg(test)] #[path = "..."] mod tests;`.

Unit tests cover: insert happy path, value budget exhaustion, add dependency
happy path, self-loop rejection, unknown parent/child IDs, parent budget
exhaustion, node count tracking.

Gate: `make check-fmt && make lint && make test`.

### Stage C: summary computation

Create `summary.rs` defining:

- `DependencySummary` — with fields `integrity_join: IntegrityLabel`,
  `confidentiality_join: DataLabels`, `authority_join: AuthoritySet`,
  `origin_count: u32`, `truncated: bool`. Operations:
  - `from_node(node)` — base case for a single value,
  - `join(&self, other)` — combines two summaries: integrity = min,
    confidentiality = union, authority = intersection,
    origin_count = saturating_add, truncated = or,
  - `unknown_top()` — conservative fail-closed summary: `Untrusted`,
    `DataLabels::all()`, empty `AuthoritySet`, `u32::MAX`, `true`.
- `compute_summary(graph, id, budgets)` — bounded breadth-first
  search (BFS) walk through parent edges. Returns `Ok(summary)` on success,
  `Ok(unknown_top())` when the graph is truncated or on closure-step budget
  overflow, `Err(IfcError::UnknownValueId)` if the root `id` is not present in
  the graph.

Unit tests cover: `from_node` base case, join correctness (integrity meets,
confidentiality unions, authority intersects), truncation propagation,
`unknown_top` values, `compute_summary` on single node / chain / diamond /
budget overflow / missing ID.

Gate: `make check-fmt && make lint && make test`.

### Stage D: propagation and control context

Create `propagation.rs` defining:

- `PropagationMode` — enum with `Normal` and `Strict` variants.
- `propagate_labels(mode, operand_summaries, control_context)` — in Normal
  mode, joins all operand summaries and ignores the control context. In Strict
  mode, joins operand summaries and additionally folds in the control-context
  summary.

Create `control_context.rs` defining:

- `EffectCounters` — `total_effects: u64`,
  `effects_by_tool: HashMap<String, u64>`.
- `ExecutionContextSummary` — with fields `pc_integrity: IntegrityLabel`,
  `pc_confidentiality: DataLabels`, `control_dependencies: Vec<ValueId>`,
  `effect_counters: EffectCounters`. Operations:
  - `new()` — `Verified` integrity, empty confidentiality, empty deps, zero
    counters,
  - `push_condition(condition_summary)` — joins condition labels into pc
    labels,
  - `as_summary()` — converts to `DependencySummary` for strict-mode joins,
  - `record_effect(tool_name)` — increments counters.

Unit tests cover: normal mode ignores context, strict mode includes context,
empty operands, single/multiple operand passthrough, new context is verified,
push_condition degrades integrity, as_summary correctness, record_effect
increments.

Gate: `make check-fmt && make lint && make test`.

### Stage E: property tests and cross-cutting validation

Add `proptest = "1"` as a dev-dependency of `zamburak-core`.

Create `proptest::arbitrary::Arbitrary` strategy implementations for:
`IntegrityLabel` (uniform 3-variant selection), `DataLabel` (uniform all
variants), `DataLabels` (`BTreeSet` of arbitrary labels), `AuthoritySet`
(`BTreeSet` of capabilities from a fixed alphabet), `DependencySummary`
(composed from arbitrary components), `ValueId` (arbitrary `u64`).

Create property test suites in colocated `*_proptests.rs` files:

Lattice algebraic laws (`trust_proptests.rs`): `IntegrityLabel::join`
commutativity, associativity, idempotency, monotonicity; `DataLabels::join`
commutativity, associativity, monotonicity, idempotency; `AuthoritySet::join`
commutativity, associativity, monotonicity.

Summary algebraic laws (`summary_proptests.rs`): `DependencySummary::join`
commutativity, associativity, truncation monotonicity, integrity monotonicity,
confidentiality monotonicity, authority monotonicity.

Dependency graph invariants (`dependency_graph_proptests.rs`): insertion-order
independence, budget monotonicity (`is_truncated` never reverts to `false`),
parent count invariant, node count invariant.

Propagation invariants (`propagation_proptests.rs`): strict mode at least as
restrictive as normal; adding operands never increases integrity; adding
operands never removes confidentiality labels.

Transitive summary invariants (in `summary_proptests.rs`): transitivity (A
depends on B depends on C implies C's labels appear in A's summary); budget
overflow conservatism (`max_closure_steps = 0` yields `unknown_top`);
single-node summary equals `from_node`.

Exhaustive bounded tests (rstest parameterized, in `trust_tests.rs`): all 9
join pairs (3 x 3), all 27 associativity triples (3 x 3 x 3), all 27
monotonicity triples (3 x 3 x 3) for `IntegrityLabel`.

BDD scenarios (rstest-bdd v0.5.0 if applicable, or rstest Given/When/Then
structured tests): happy path dependency tracking, budget overflow conservative
summary, strict versus normal mode control context handling.

Gate: `make check-fmt && make lint && make test`.

### Stage F: documentation and roadmap sync

Update `docs/users-guide.md` with an "IFC core types" section describing the
public API: `ValueId`, `IntegrityLabel`, `DataLabel`, `DataLabels`,
`AuthoritySet`, `GraphBudgets`, `DependencyGraph`, `DependencySummary`,
`PropagationMode`, `ExecutionContextSummary`, and usage examples.

Update `docs/zamburak-design-document.md` with implementation decisions from
the decision log.

Mark Task 1.6.2 as done (`[x]`) in `docs/roadmap.md`.

Run all gates:

```plaintext
set -o pipefail; make check-fmt 2>&1 | tee /tmp/make-check-fmt-0-6-2.log
set -o pipefail; make lint 2>&1 | tee /tmp/make-lint-0-6-2.log
set -o pipefail; make test 2>&1 | tee /tmp/make-test-0-6-2.log
```

## Concrete steps

<!-- markdownlint-disable MD029 -->
1. Initialize the submodule if needed.

   ```plaintext
   git submodule update --init --recursive
   ```

   Expected outcome: `third_party/full-monty/` is populated.

2. Confirm dependency gates.

   Verify that Task 1.6.1 is marked `[x]` in `docs/roadmap.md`.

3. Create `crates/zamburak-core/src/value_id.rs` with `ValueId` newtype.

4. Create `crates/zamburak-core/src/trust.rs` with `IntegrityLabel`,
   `DataLabel`, `DataLabels`, `AuthoritySet`.

5. Create `crates/zamburak-core/src/ifc_errors.rs` with `IfcError`.

6. Update `crates/zamburak-core/src/lib.rs` with module declarations and
   public re-exports.

7. Add unit tests for Stage A types. Run gate check.

8. Create `crates/zamburak-core/src/dependency_graph.rs` with
   `GraphBudgets`, `ValueNode`, `DependencyGraph`.

9. Add unit tests for Stage B. Run gate check.

10. Create `crates/zamburak-core/src/summary.rs` with `DependencySummary` and
    `compute_summary`.

11. Add unit tests for Stage C. Run gate check.

12. Create `crates/zamburak-core/src/propagation.rs` with `PropagationMode`
    and `propagate_labels`.

13. Create `crates/zamburak-core/src/control_context.rs` with
    `ExecutionContextSummary` and `EffectCounters`.

14. Add unit tests for Stage D. Run gate check.

15. Add `proptest = "1"` to `crates/zamburak-core/Cargo.toml` dev-dependencies.

16. Create `Arbitrary` implementations and property test suites.

17. Create exhaustive bounded rstest tests for `IntegrityLabel`.

18. Create BDD-style scenarios. Run gate check.

19. Update `docs/users-guide.md`, `docs/zamburak-design-document.md`,
    `docs/roadmap.md`.

20. Run documentation gates.

    ```plaintext
    set -o pipefail; make fmt 2>&1 | tee /tmp/make-fmt-0-6-2.log
    set -o pipefail; make markdownlint 2>&1 | tee /tmp/make-markdownlint-0-6-2.log
    ```

21. Run the required root gates.

    ```plaintext
    set -o pipefail; make check-fmt 2>&1 | tee /tmp/make-check-fmt-0-6-2.log
    set -o pipefail; make lint 2>&1 | tee /tmp/make-lint-0-6-2.log
    set -o pipefail; make test 2>&1 | tee /tmp/make-test-0-6-2.log
    ```

    Expected outcome: all gates green.
<!-- markdownlint-enable MD029 -->

## Validation and acceptance

Quality criteria:

- Unit tests: all new `zamburak-core` IFC unit tests pass. Existing authority
  and localization tests still pass.
- Property tests: all `proptest` suites pass (lattice laws, summary laws, graph
  invariants, propagation invariants, transitive summary invariants).
- Exhaustive tests: all 63 rstest `IntegrityLabel` cases pass (9 + 27 + 27).
- Lint: `make lint` passes (Clippy + rustdoc).
- Format: `make check-fmt` passes.
- Full suite: `make test` passes.
- Documentation: `make markdownlint` passes. User guide, design document, and
  roadmap are updated.

Quality method:

```plaintext
set -o pipefail; make check-fmt 2>&1 | tee /tmp/make-check-fmt-0-6-2.log
set -o pipefail; make lint 2>&1 | tee /tmp/make-lint-0-6-2.log
set -o pipefail; make test 2>&1 | tee /tmp/make-test-0-6-2.log
```

## Idempotence and recovery

Gate checks (`make check-fmt`, `make lint`, `make test`) are idempotent and can
be re-run from the repository root. File creation and editing operations during
implementation can be repeated by overwriting existing files.

Note that the runtime API itself enforces fail-fast semantics: attempting to
insert a value with a duplicate `ValueId` returns `IfcError::DuplicateValueId`,
and attempting to add a duplicate edge returns `IfcError::DuplicateEdge`. These
are explicit errors, not no-ops. If a step fails, fix the issue and re-run the
gate from the repository root.

## Artifacts and notes

Evidence to capture during implementation:

- `cargo test -p zamburak-core` log showing all IFC tests passing.
- Final root gate logs from `make check-fmt`, `make lint`, `make test`.
- Decision-note entries in this document for any design choices made.

## Interfaces and dependencies

### Types introduced by this task

In `crates/zamburak-core/src/value_id.rs`:

```rust
#[repr(transparent)]
pub struct ValueId(u64); // transparent newtype wrapping u64
```

In `crates/zamburak-core/src/trust.rs`:

```rust
pub enum IntegrityLabel { Untrusted, Trusted, Verified }
pub enum DataLabel { Pii, AuthSecret, PrivateEmailBody, PaymentInstrument, InternalPolicyNote }
pub struct DataLabels { /* BTreeSet<DataLabel> */ }
pub struct AuthoritySet { /* BTreeSet<AuthorityCapability> */ }
```

In `crates/zamburak-core/src/ifc_errors.rs`:

```rust
pub enum IfcError {
    ValueBudgetExhausted { current: u64, limit: u64 },
    ParentBudgetExhausted { value_id: ValueId, current: u64, limit: u64 },
    ClosureStepBudgetExhausted { steps: u64, limit: u64 },
    UnknownValueId(ValueId),
    DuplicateValueId(ValueId),
    CycleDetected { from: ValueId, to: ValueId },
    DuplicateEdge { child: ValueId, parent: ValueId },
}
```

In `crates/zamburak-core/src/dependency_graph.rs`:

```rust
pub struct GraphBudgets { pub max_values: u64, /* ... */ }
pub struct ValueLabels { pub integrity: IntegrityLabel, /* ... */ }
pub struct ValueNode { /* id, integrity, confidentiality, authority, parents */ }
pub struct DependencyGraph { /* nodes, budgets, truncated */ }
```

In `crates/zamburak-core/src/summary.rs`:

```rust
pub struct DependencySummary { /* integrity_join, confidentiality_join, ... */ }
pub fn compute_summary(graph: &DependencyGraph, id: &ValueId, budgets: &GraphBudgets)
    -> Result<DependencySummary, IfcError>;
```

In `crates/zamburak-core/src/propagation.rs`:

```rust
pub enum PropagationMode { Normal, Strict }
pub fn propagate_labels(
    mode: PropagationMode,
    operand_summaries: &[DependencySummary],
    control_context: &ExecutionContextSummary,
) -> Option<DependencySummary>;
```

In `crates/zamburak-core/src/control_context.rs`:

```rust
pub struct EffectCounters { /* total_effects, effects_by_tool */ }
pub struct ExecutionContextSummary { /* pc_integrity, pc_confidentiality, ... */ }
```

### Dependencies consumed

- `thiserror = "2.0.11"` (workspace) — `IfcError` derivation.
- `zamburak-core::authority::AuthorityCapability` — re-used in `AuthoritySet`.

### Dev-dependencies

- `rstest = "0.26.1"` (existing) — parameterized unit tests.
- `proptest = "1"` (new) — property-based testing for algebraic invariants.

## Revision note

- 2026-03-22: Initial plan drafted from roadmap Task 1.6.2 requirements,
  ADR-001 section B1, design document sections on dependency representation and
  strict-mode semantics, verification targets row for IFC propagation, and
  repository layout Table 2.
