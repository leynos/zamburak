# Freeze canonical policy schema v1 in runtime loading paths (Task 0.1.1)

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

This repository does not include `PLANS.md`; therefore this document is the
authoritative execution plan for this task.

## Purpose / big picture

Implement the Phase 0 Task 0.1.1 contract from `docs/roadmap.md`: runtime
policy loading must accept only canonical policy schema version 1 and reject
any unknown schema version fail-closed.

After this change, a library consumer loading policy definitions can observe
two clear outcomes:

- a policy with `schema_version: 1` loads successfully,
- a policy with any other schema version is rejected with a deterministic
  failure outcome that does not default to permissive behaviour.

The implementation must include unit tests and behavioural tests, and must
update user-facing documentation plus roadmap state when complete.

## Constraints

- Implement to the contracts in:
  `docs/zamburak-design-document.md` ("Canonical policy schema v1"),
  `docs/verification-targets.md` (row "Policy schema loader"),
  `docs/zamburak-engineering-standards.md` ("Fail-closed standards"), and
  `docs/repository-layout.md` (`policies/` and `crates/zamburak-policy`).
- Scope is limited to schema-version acceptance and unknown-version rejection
  behaviour in runtime loading paths.
- Future-version authoring and migration logic are out of scope for this task.
- Unknown schema versions must not fall back to defaults, auto-migrate, or
  partially load.
- Add both unit and behavioural test coverage for happy and unhappy paths.
- Behavioural tests should use `rstest-bdd` v0.5.0 where applicable.
- Record design decisions in `docs/zamburak-design-document.md` in the same
  change set.
- Update `docs/users-guide.md` with consumer-visible loader behaviour. The file
  does not currently exist and should be created if still absent at execution
  time.
- On feature completion, mark Task 0.1.1 as done in `docs/roadmap.md`.
- Quality gates must pass before completion:
  `make check-fmt`, `make lint`, and `make test`.

## Tolerances (exception triggers)

- Scope tolerance: if delivery requires changes to more than 14 files or more
  than 700 net lines, stop and escalate.
- Interface tolerance: if freezing schema v1 requires changing unrelated public
  APIs outside the loader surface, stop and escalate with options.
- Dependency tolerance: if implementation needs crates beyond expected parser
  and test dependencies (`serde`, `serde_yaml`, `serde_json`, `thiserror`,
  `rstest`, `rstest-bdd`, `rstest-bdd-macros`), stop and escalate.
- Test tolerance: if behavioural coverage cannot be expressed with
  `rstest-bdd` v0.5.0 after two implementation attempts, stop and document why
  it is not applicable before choosing an alternative.
- Ambiguity tolerance: if `schema_version` type semantics (integer width or
  string encoding) remain ambiguous after reviewing signpost docs, stop and
  request clarification.

## Risks

- Risk: repository runtime structure is currently a minimal single crate while
  roadmap targets `crates/zamburak-policy`. Severity: high Likelihood: high
  Mitigation: introduce only the minimal workspace and policy-crate scaffolding
  needed for Task 0.1.1, keeping behaviour tightly scoped to loader contracts.

- Risk: behavioural test harness could overfit internal details and reduce
  consumer-level coverage value. Severity: medium Likelihood: medium
  Mitigation: ensure Gherkin scenarios assert externally observable loader
  outcomes and explicit fail-closed behaviour.

- Risk: missing `docs/users-guide.md` could cause inconsistent user messaging.
  Severity: medium Likelihood: high Mitigation: create `docs/users-guide.md`
  with a dedicated section describing accepted schema version and rejection
  semantics, then add it to `docs/contents.md`.

## Progress

- [x] (2026-02-09 00:32Z) Reviewed required signposts and current repository
  state.
- [x] (2026-02-09 00:32Z) Confirmed no `PLANS.md` exists; this ExecPlan is the
  controlling plan document.
- [x] (2026-02-09 00:32Z) Confirmed `docs/users-guide.md` is absent and must be
  created during implementation.
- [x] (2026-02-09 00:52Z) Implemented Stage A by creating
  `crates/zamburak-policy` scaffolding and policy schema artefacts.
- [x] (2026-02-09 01:05Z) Implemented Stage B with unit tests and
  `rstest-bdd` compatibility scenarios under `tests/compatibility/`.
- [x] (2026-02-09 01:06Z) Implemented Stage C loader and engine enforcement for
  schema v1 only.
- [x] (2026-02-09 01:11Z) Implemented Stage D docs updates, roadmap completion,
  and quality-gate validation.

## Surprises & Discoveries

- Observation: The repository does not yet contain `crates/` or `policies/`
  paths referenced by roadmap traceability. Evidence: `rg --files` output from
  repository root. Impact: Task 0.1.1 execution must include minimal path
  scaffolding to align implementation with documented repository contracts.

- Observation: `docs/users-guide.md` does not exist.
  Evidence: path lookup during signpost review returned "No such file". Impact:
  user-guide update requires file creation and index linking.

- Observation: integration test crate compilation failed with
  `-D missing-docs` until the compatibility test crate had a crate-level doc
  comment. Evidence: `cargo test --test compatibility` error from
  `tests/compatibility/main.rs`. Impact: all new integration test crates in
  this repository must include a leading `//!` crate comment.

## Decision Log

- Decision: Treat Task 0.1.1 as requiring minimal creation of
  `crates/zamburak-policy` and `policies/schema.json` artefacts if absent.
  Rationale: `docs/roadmap.md` traceability for Task 0.1.1 explicitly maps to
  those paths; implementation should match documented ownership. Date/Author:
  2026-02-09 / Codex

- Decision: Include `rstest-bdd` v0.5.0 behavioural tests for policy loader
  acceptance/rejection scenarios. Rationale: behavioural coverage is applicable
  for consumer-observable parsing outcomes and is explicitly requested for this
  task. Date/Author: 2026-02-09 / Codex

- Decision: Create `docs/users-guide.md` during implementation if still absent.
  Rationale: user request requires user-facing documentation updates and there
  is no existing guide file to edit. Date/Author: 2026-02-09 / Codex

- Decision: Keep policy parsing strict on `schema_version` type and value:
  non-numeric values fail parsing and numeric values other than `1` fail with
  `UnsupportedSchemaVersion`. Rationale: this keeps fail-closed behaviour
  explicit and deterministic for loader callers. Date/Author: 2026-02-09 / Codex

## Outcomes & Retrospective

Completed implementation outcomes:

- Added runtime loading paths in `crates/zamburak-policy`:
  `PolicyDefinition::from_yaml_str`, `PolicyDefinition::from_json_str`,
  `PolicyEngine::from_yaml_str`, and `PolicyEngine::from_json_str`.
- Enforced canonical schema freeze with
  `CANONICAL_POLICY_SCHEMA_VERSION = 1` and fail-closed
  `UnsupportedSchemaVersion` rejection.
- Added unit tests for happy path and unhappy/edge cases:
  unknown versions (`0`, `2`, `u64::MAX`), missing version, and malformed
  version type.
- Added behavioural tests using `rstest-bdd` v0.5.0 in
  `tests/compatibility/features/policy_schema.feature` with matching step
  bindings in `tests/compatibility/policy_schema_bdd.rs`.
- Added policy schema artefacts in `policies/schema.json` and
  `policies/default.yaml`.
- Updated consumer and design docs plus roadmap completion state:
  `docs/users-guide.md`, `docs/zamburak-design-document.md`,
  `docs/contents.md`, and `docs/roadmap.md`.

Quality gate outcomes:

- passed: `make check-fmt`,
- passed: `make lint`,
- passed: `make test`,
- passed: `make fmt`,
- passed: `make markdownlint`,
- passed: `make nixie`,
- passed: targeted suites
  (`cargo test -p zamburak-policy`, `cargo test --test compatibility`).

## Context and orientation

Current repository state is an early scaffold (`src/lib.rs`) plus detailed
design and roadmap documentation. Task 0.1.1 is the first contract-freeze code
delivery and must anchor future policy and compatibility work.

Key documents and expectations:

- `docs/zamburak-design-document.md` defines schema v1 and fail-closed unknown
  version handling.
- `docs/verification-targets.md` requires unit parser tests and compatibility
  tests for policy schema loading.
- `docs/repository-layout.md` allocates loader models to
  `crates/zamburak-policy/src/policy_def.rs` and policy artefacts to
  `policies/schema.json`.
- `docs/roadmap.md` maps Task 0.1.1 completion to:
  `policies/schema.json`, `crates/zamburak-policy/src/policy_def.rs`,
  `crates/zamburak-policy/src/engine.rs`.

## Plan of work

Stage A: align structure and lock contracts before implementation.

- Introduce minimal workspace structure to host `crates/zamburak-policy` while
  preserving existing root crate behaviour.
- Define loader-facing types and error contract in
  `crates/zamburak-policy/src/policy_def.rs`.
- Add or update `policies/schema.json` so its version contract explicitly states
  schema version 1.

Go/no-go gate for Stage A: build graph resolves and types compile with no
loader logic yet.

Stage B: write tests first for desired behaviour.

- Unit tests in `crates/zamburak-policy/src/policy_def.rs` (or colocated test
  module) covering: accepted `schema_version: 1`, rejected unknown versions
  (for example `0`, `2`, and large integers), missing or malformed schema
  version.
- Behavioural tests in `tests/compatibility/` using `rstest-bdd` v0.5.0 with a
  feature file that expresses: valid canonical policy loads, unknown schema
  version is rejected fail-closed with deterministic reason.
- Use `rstest` fixtures for shared policy input setup.

Go/no-go gate for Stage B: tests fail for the expected reasons before loader
logic is added.

Stage C: implement loader enforcement with minimal surface.

- Implement parsing and validation path in
  `crates/zamburak-policy/src/policy_def.rs` to accept only schema version 1.
- Ensure all runtime loading paths route through that validation.
- Ensure unknown version branch returns denial/error without fallback.
- Keep `crates/zamburak-policy/src/engine.rs` aligned so no code path can bypass
  loader validation when consuming policies.

Go/no-go gate for Stage C: new unit and behavioural suites pass locally.

Stage D: document and finalize.

- Update `docs/zamburak-design-document.md` with any concrete design decisions
  made during implementation (for example: exact rejection semantics).
- Create or update `docs/users-guide.md` with consumer-facing loader contract
  and examples.
- Update `docs/contents.md` to include `docs/users-guide.md` if created.
- Mark Task 0.1.1 as done in `docs/roadmap.md`.
- Run all required quality gates and archive logs.

## Concrete steps

Run commands from repository root: `/home/user/project`.

1. Baseline and branch state.

       git status --short
       rg --files

2. Create/adjust workspace and policy crate scaffolding required by Task 0.1.1.

       mkdir -p crates/zamburak-policy/src policies tests/compatibility/features
       # Then edit Cargo manifests and source files as defined in Plan of work.

3. Add tests before implementation.

       # Add unit tests in crates/zamburak-policy/src/policy_def.rs
       # Add behavioural feature and step definitions under tests/compatibility/
       # using rstest-bdd v0.5.0 and rstest fixtures.

4. Run targeted tests during iteration.

       set -o pipefail && cargo test --workspace policy_schema | tee /tmp/test-policy-schema-<branch>.out

5. Apply implementation edits and rerun targeted tests until green.

6. Update docs and roadmap completion marker.

7. Run full quality gates with logs (required).

       set -o pipefail && make check-fmt | tee /tmp/check-fmt-zamburak-<branch>.out
       set -o pipefail && make lint | tee /tmp/lint-zamburak-<branch>.out
       set -o pipefail && make test | tee /tmp/test-zamburak-<branch>.out

8. Run documentation gates because docs are changed.

       set -o pipefail && make markdownlint | tee /tmp/markdownlint-zamburak-<branch>.out
       set -o pipefail && make nixie | tee /tmp/nixie-zamburak-<branch>.out
       set -o pipefail && make fmt | tee /tmp/fmt-zamburak-<branch>.out

## Validation and acceptance

Acceptance is met only when all checks below pass:

- Behaviour:
  policy loader accepts canonical schema v1 and rejects unknown schema versions
  fail-closed in every runtime loading path.
- Unit tests:
  parser/validator tests cover happy and unhappy paths, including edge cases.
- Behavioural tests:
  `tests/compatibility/` scenarios (implemented with `rstest-bdd` v0.5.0 where
  applicable) cover positive and negative contract outcomes.
- Documentation:
  design decision updates appear in `docs/zamburak-design-document.md` and
  consumer-facing behaviour appears in `docs/users-guide.md`.
- Roadmap state:
  Task 0.1.1 checkbox in `docs/roadmap.md` is marked `[x]`.
- Quality gates:
  `make check-fmt`, `make lint`, and `make test` succeed.

## Idempotence and recovery

- All file-creation steps are idempotent when rerun; existing files should be
  edited in place.
- If a gate fails, fix only the failing cause, rerun that gate, then rerun the
  full required gate sequence.
- If behavioural test integration blocks progress, stop at tolerance boundary
  and record alternatives in `Decision Log` before proceeding.

## Artifacts and notes

Capture and keep these artefacts while executing:

- `git diff` for the final change set,
- gate logs in `/tmp/*-zamburak-<branch>.out`,
- targeted test log:
  `/tmp/test-policy-schema-<branch>.out`,
- a short mapping from each acceptance criterion to the test or document that
  proves it.

## Interfaces and dependencies

The implementation must leave a clear loader interface in
`crates/zamburak-policy`:

- a parse/validate entrypoint that accepts serialized policy input and returns
  a typed policy model or typed loader error,
- explicit schema-version rejection error variant for unsupported versions,
- runtime call paths that consume only validated policy definitions.

Expected dependency additions (if not already present):

- parsing/validation: `serde`, `serde_yaml`, `serde_json`, `thiserror`,
- tests: `rstest`, `rstest-bdd = "0.5.0"`,
  `rstest-bdd-macros = "0.5.0"` with `compile-time-validation` enabled.

## Revision note

Initial draft created for roadmap Task 0.1.1 with explicit execution stages,
test strategy, documentation obligations, and quality-gate requirements.

Revision (2026-02-09): completed implementation, updated status to `COMPLETE`,
checked all stage progress items, added test/discovery decisions, and recorded
final outcomes and gate evidence.
