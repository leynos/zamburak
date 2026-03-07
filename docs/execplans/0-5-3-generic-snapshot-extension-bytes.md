# Add generic snapshot extension bytes in `full-monty` (Task 0.5.3)

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`,
`Decision log`, and `Outcomes & retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / big picture

Implement roadmap Task 0.5.3 from `docs/roadmap.md`: add a generic, embedder-
owned snapshot-extension byte payload in `third_party/full-monty/` so snapshot
persistence can carry opaque host state without Monty interpretation.

After this change, hosts can attach optional bytes to `Snapshot`,
`FutureSnapshot`, `ReplSnapshot`, and `ReplFutureSnapshot` values. Those bytes
must round-trip through `dump()` and `load()` for run/repl progress payloads.
Success is observable when unit and behavioural tests prove that extension
bytes persist through suspend, dump, load, and resume flows, while empty or
missing extensions preserve baseline semantics.

## Constraints

- Implement to requirement signposts:
  `docs/adr-001-monty-ifc-vm-hooks.md` section "A3. Snapshot extension seam
  (optional but preferred)", `docs/zamburak-design-document.md` sections
  "Two-track execution model" and "Snapshot and resume semantics", and
  `docs/verification-targets.md` row "Control context".
- Dependency constraint: Task 0.5.2 is a hard precondition for 0.5.3. Confirm
  it is marked done before code changes.
- In scope: embedder-owned opaque bytes attached to snapshot persistence with
  no Monty semantic interpretation.
- Out of scope: Zamburak-specific encoding formats or policy-specific naming in
  `full-monty` APIs.
- Track A guardrail: no `taint`, `policy`, `capabilities`, or `Zamburak`
  naming in `third_party/full-monty/` public APIs.
- Add tests covering happy paths, unhappy paths, and edge cases.
- Add behavioural tests using `rstest-bdd` v0.5.0 where applicable.
- Record design decisions in `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` for consumer-visible API or behaviour changes.
- Mark roadmap Task 0.5.3 done in `docs/roadmap.md` only after all gates are
  green.
- Required completion gates: `make check-fmt`, `make lint`, and `make test`.

## Tolerances (exception triggers)

- Scope tolerance: if implementation requires edits in more than 20 files or
  1400 net changed lines, stop and escalate with a split plan.
- Interface tolerance: if additive APIs are insufficient and a breaking change
  is required, stop and escalate with additive and breaking options.
- Serialization tolerance: if snapshot extension support cannot preserve
  backwards-compatible `dump()`/`load()` behaviour, stop and escalate with a
  versioned format proposal.
- Dependency tolerance: if any new third-party dependency is required in either
  the superproject or `full-monty`, stop and escalate before adding it.
- Iteration tolerance: if required gates fail after three focused fix loops,
  stop and report failures with root-cause hypotheses.
- Ambiguity tolerance: if it is unclear whether extension bytes must propagate
  automatically across multiple snapshots, stop and request a decision with
  trade-offs and examples.

## Risks

- Risk: adding a new serialized field can break existing snapshot payloads if
  `serde` defaults are not configured. Severity: high Likelihood: medium
  Mitigation: use `#[serde(default)]` and add dump/load regression tests to
  confirm backwards-compatible payloads.

- Risk: extension bytes are large and could add unexpected memory pressure.
  Severity: medium Likelihood: medium Mitigation: keep the payload optional,
  document host responsibility for sizing, and avoid implicit copies.

- Risk: REPL and async snapshot paths diverge in behaviour. Severity: medium
  Likelihood: medium Mitigation: add explicit tests for `Snapshot`,
  `FutureSnapshot`, `ReplSnapshot`, and `ReplFutureSnapshot` round-trips.

- Risk: nested checkout tooling friction for `full-monty` tests. Severity:
  medium Likelihood: medium Mitigation: use
  `make -C third_party/full-monty lint-rs-local` and submodule-local test
  commands for focused evidence.

## Progress

- [x] (2026-03-04 00:10Z) Reviewed roadmap Task 0.5.3 requirements and signpost
  documents.
- [x] (2026-03-04 00:12Z) Inspected current snapshot and progress structures in
  `third_party/full-monty/crates/monty/src/run.rs` and
  `third_party/full-monty/crates/monty/src/repl.rs`.
- [x] (2026-03-04 00:15Z) Drafted this ExecPlan with staged delivery, test-first
  sequencing, and completion gates.
- [x] (2026-03-04 01:02Z) Began implementation: updated plan status and kicked
  off Stage A preflight checks.
- [x] (2026-03-04 02:08Z) Added snapshot extension bytes to run/repl snapshot
  structs with additive accessors and serde defaults, plus resume propagation
  for future snapshots.
- [x] (2026-03-04 02:21Z) Added run/repl unit tests, `rstest-bdd` scenarios, and
  a superproject compatibility probe for extension byte round-trips.
- [x] (2026-03-04 02:42Z) Updated `docs/zamburak-design-document.md` and
  `docs/users-guide.md` with the new extension seam.
- [x] (2026-03-04 03:12Z) Completed submodule gates (`make format-rs`,
  `make lint-rs`, `make test`) and queued superproject gates.
- [x] (2026-03-04 04:30Z) Ran superproject gates (`make fmt`,
  `make markdownlint`, `make nixie`, `make check-fmt`, `make lint`,
  `make test`) and marked roadmap Task 0.5.3 done.

## Surprises & discoveries

- Observation: `third_party/full-monty/` may be empty until submodules are
  initialized. Evidence: `rg` fails until
  `git submodule update --init --recursive` has been run. Impact: concrete
  steps must include submodule initialization for a new checkout.
- Observation: `postcard` rejects optional fields that are skipped during
  serialization; using `skip_serializing_if = "Option::is_none"` caused
  `DeserializeUnexpectedEnd` on `RunProgress::load()` for `None` extension
  values. Impact: omit `skip_serializing_if` and rely on `#[serde(default)]` so
  `None` still serializes as an empty optional marker.

## Decision log

- Decision: model snapshot extensions as `Option<Vec<u8>>` on snapshot structs
  with `serde` defaults. Rationale: optional bytes keep the API generic, avoid
  new dependencies, and permit backwards-compatible deserialization.
  Date/Author: 2026-03-04 / Codex.
- Decision: name the internal field `extension_bytes` and set
  `#[serde(rename = "snapshot_extension")]` to preserve the API signal while
  avoiding `clippy::struct_field_names` warnings. Rationale: keep public
  semantics stable without clippy suppressions. Date/Author: 2026-03-04 / Codex.
- Decision: remove `skip_serializing_if` for optional extension bytes to avoid
  `postcard` binary format deserialization failures when the field is absent in
  `RunProgress::dump()` output. Rationale: ensure round-trip stability for
  `None` values across snapshot persistence. Date/Author: 2026-03-04 / Codex.

## Outcomes & retrospective

Delivered additive snapshot-extension bytes on all run/repl snapshot types,
plus unit, Behaviour-Driven Development (BDD), and compatibility probes
covering round-trips and corruption handling. Documentation now calls out the
extension seam and usage patterns for library consumers. Validation gates
completed in the submodule and superproject (format, lint, and full test
suites).

## Context and orientation

Snapshot persistence today is handled by `RunProgress::dump()` and
`RunProgress::load()` in `third_party/full-monty/crates/monty/src/run.rs`, plus
`ReplProgress::dump()` and `ReplProgress::load()` in
`third_party/full-monty/crates/monty/src/repl.rs`. The snapshot state itself is
stored in public structs `Snapshot<T>`, `FutureSnapshot<T>`, `ReplSnapshot<T>`,
and `ReplFutureSnapshot<T>`. These structs are `serde`-serializable and are
carried inside `RunProgress` and `ReplProgress` variants.

There is no current extension seam for embedder-owned bytes. Any new field must
remain generic and avoid policy naming. Existing behavioural coverage lives in
`third_party/full-monty/crates/monty/tests/runtime_ids_bdd.rs`,
`third_party/full-monty/crates/monty/tests/binary_serde.rs`, and
`third_party/full-monty/crates/monty/tests/repl.rs`. Zamburak-level
compatibility probes for `full-monty` live under `tests/compatibility/` and
reuse the `test_utils::full_monty_observer_probe_helpers` helper to execute
submodule BDD suites from the superproject.

In this plan, "snapshot extension bytes" means an opaque byte vector supplied
by the embedder, persisted alongside snapshot state, and returned untouched
when a snapshot is loaded. Monty must not interpret, version, or validate these
bytes beyond basic serialization.

## Plan of work

1. Stage A: confirm prerequisites and shape the API.
   Review Task 0.5.2 completion state in `docs/roadmap.md`. Read the ADR and
   design doc sections to clarify the expected seam, then inspect `Snapshot`,
   `FutureSnapshot`, `ReplSnapshot`, and `ReplFutureSnapshot` definitions to
   decide on field names and method signatures. Capture the decision in
   `docs/zamburak-design-document.md` before coding.

2. Stage B: add failing tests for snapshot-extension behaviour.
   Add unit tests in `third_party/full-monty/crates/monty/tests/` for snapshot
   extension round-trips on run and repl paths. Add Behaviour-Driven
   Development (BDD) coverage with `rstest-bdd` v0.5.0 in a new
   `snapshot_extensions_bdd.rs` test file plus a
   `tests/features/snapshot_extensions.feature` scenario. Add a superproject
   compatibility BDD probe under `tests/compatibility/` that executes the new
   `full-monty` BDD suite via
   `cargo test --manifest-path third_party/full-monty/Cargo.toml -p monty`
   `--test snapshot_extensions_bdd`. Ensure the tests fail or do not compile
   before the implementation changes.

3. Stage C: implement extension bytes in `full-monty` snapshot structs.
   Add an optional `snapshot_extension` field to each snapshot struct (use
   `extension_bytes` internally with
   `#[serde(default, rename = "snapshot_extension")]`). Provide
   `with_snapshot_extension` and `snapshot_extension` accessors on `Snapshot`,
   `FutureSnapshot`, `ReplSnapshot`, and `ReplFutureSnapshot`. Update snapshot
   constructors to initialize `snapshot_extension` as `None`. Do not
   reintroduce `skip_serializing_if`; it was intentionally removed per the
   Decision log. Ensure `dump()`/`load()` behaviour remains unchanged aside
   from carrying extension bytes, and keep existing `run.rs`/`repl.rs` doc
   comments unchanged.

4. Stage D: documentation, roadmap update, and validation.
   Update `docs/zamburak-design-document.md` with a dated implementation
   decision in the snapshot/resume section describing the generic extension
   seam. Update `docs/users-guide.md` to explain how consumers attach and read
   extension bytes when persisting `RunProgress` or `ReplProgress`. Mark Task
   0.5.3 done in `docs/roadmap.md` only after all gates pass. Run required
   format, lint, and test gates and capture evidence.

## Concrete steps

1. Initialize submodules if needed.

```plaintext
git submodule update --init --recursive
```

Expected outcome: `third_party/full-monty/` is populated and ready for search.

1. Add test scaffolding for snapshot extension bytes.

- Create `third_party/full-monty/crates/monty/tests/snapshot_extensions.rs`
  with unit tests for run and repl round-trips.
- Create `third_party/full-monty/crates/monty/tests/snapshot_extensions_bdd.rs`
  and `third_party/full-monty/crates/monty/tests/features/`
  `snapshot_extensions.feature` for BDD coverage.
- Create `tests/compatibility/full_monty_snapshot_extension_bdd.rs` and
  `tests/compatibility/features/full_monty_snapshot_extension.feature` to run
  the submodule BDD suite from the superproject.

1. Implement snapshot extension fields and accessors.

- Edit `third_party/full-monty/crates/monty/src/run.rs`:
  add `snapshot_extension` to `Snapshot<T>` and `FutureSnapshot<T>`, add
  accessors, and initialize fields in snapshot constructors.
- Edit `third_party/full-monty/crates/monty/src/repl.rs`:
  add `snapshot_extension` to `ReplSnapshot<T>` and `ReplFutureSnapshot<T>`,
  add accessors, and initialize fields in snapshot constructors.

1. Update documentation.

- Edit `docs/zamburak-design-document.md` with a dated implementation decision
  in "Snapshot and resume semantics".
- Edit `docs/users-guide.md` to describe the new snapshot extension bytes API.
- Edit `docs/roadmap.md` to mark Task 0.5.3 done after gates pass.

1. Run formatting and lint gates (superproject).

```plaintext
set -o pipefail
make fmt | tee /tmp/make-fmt.log
```

```plaintext
set -o pipefail
make markdownlint | tee /tmp/make-markdownlint.log
```

```plaintext
set -o pipefail
make nixie | tee /tmp/make-nixie.log
```

```plaintext
set -o pipefail
make check-fmt | tee /tmp/make-check-fmt.log
```

```plaintext
set -o pipefail
make lint | tee /tmp/make-lint.log
```

```plaintext
set -o pipefail
make test | tee /tmp/make-test.log
```

1. Run focused `full-monty` gates for submodule evidence.

```plaintext
set -o pipefail
make -C third_party/full-monty format-rs | tee /tmp/full-monty-format-rs.log
```

```plaintext
set -o pipefail
make -C third_party/full-monty lint-rs-local | tee /tmp/full-monty-lint-rs-local.log
```

```plaintext
set -o pipefail
make -C third_party/full-monty test | tee /tmp/full-monty-test.log
```

## Validation and acceptance

Quality criteria (what "done" means):

- Tests: new unit and BDD tests in `third_party/full-monty` pass, including
  `snapshot_extensions.rs` and `snapshot_extensions_bdd.rs`, and the
  superproject compatibility probe passes.
- Lint/typecheck: `make lint` and `make -C third_party/full-monty lint-rs-local`
  pass with no new warnings.
- Formatting: `make check-fmt` and `make fmt` produce no diffs.

Quality method (how we check):

- Run the commands in the Concrete steps section and confirm each reports
  success.
- Observe that `RunProgress::dump()`/`load()` and `ReplProgress::dump()`/
  `load()` preserve snapshot extension bytes across round-trips, including
  empty and missing bytes.

## Idempotence and recovery

All steps are safe to rerun. If tests fail after adding extension bytes, revert
only the new snapshot fields and re-run the Stage B tests to re-establish the
red state before proceeding. If the submodule is uninitialized, rerun
`git submodule update --init --recursive` before searching for files.

## Artifacts and notes

Expected BDD probe output includes the new suite name and a green result. For
example:

```plaintext
running 1 test
...
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interfaces and dependencies

Do not add new third-party dependencies. Reuse existing `rstest` and
`rstest-bdd = "0.5.0"` crates in `third_party/full-monty/crates/monty`.

Target public API shape (names may vary, semantics must not):

```rust
impl<T: ResourceTracker> Snapshot<T> {
    pub fn with_snapshot_extension(self, bytes: Vec<u8>) -> Self;
    pub fn snapshot_extension(&self) -> Option<&[u8]>;
}

impl<T: ResourceTracker> FutureSnapshot<T> {
    pub fn with_snapshot_extension(self, bytes: Vec<u8>) -> Self;
    pub fn snapshot_extension(&self) -> Option<&[u8]>;
}

impl<T: ResourceTracker> ReplSnapshot<T> {
    pub fn with_snapshot_extension(self, bytes: Vec<u8>) -> Self;
    pub fn snapshot_extension(&self) -> Option<&[u8]>;
}

impl<T: ResourceTracker> ReplFutureSnapshot<T> {
    pub fn with_snapshot_extension(self, bytes: Vec<u8>) -> Self;
    pub fn snapshot_extension(&self) -> Option<&[u8]>;
}
```

The underlying structs should include:

```rust
#[serde(default, rename = "snapshot_extension")]
extension_bytes: Option<Vec<u8>>,
```

## Revision note

- Initial draft created for roadmap Task 0.5.3 with dependency gate, test-first
  sequencing, and completion gates.
- Updated after implementation to record the `postcard` serialization constraint
  (no `skip_serializing_if`) and the `extension_bytes` internal field name with
  `serde` rename for stable API semantics.
