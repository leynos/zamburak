# Implement explicit schema migration transforms and conformance evidence (Task 0.1.2)

This execution plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

This repository does not include `PLANS.md`; therefore this document is the
authoritative execution plan for this task.

## Purpose / big picture

Implement roadmap Task 0.1.2 from `docs/roadmap.md`: policy loading must
support explicit version-to-version migration transforms and produce audit
metadata proving what changed during migration.

After this change, a library consumer can load a known legacy policy schema,
obtain a canonical schema v1 policy, and inspect auditable migration evidence
(including deterministic input/output hashes and applied transform steps).
Unknown schema families remain rejected fail-closed.

The task is complete only when tests prove restrictive-equivalent outcomes for
migrated policies, auditable migration records are emitted for migrated inputs,
and all required quality gates pass.

## Constraints

- Implement to contracts in:
  `docs/zamburak-design-document.md` ("Schema compatibility and migration
  semantics"), `docs/verification-targets.md` (row "Policy schema loader"), and
  `docs/repository-layout.md` (`crates/zamburak-policy`,
  `tests/compatibility/`, and roadmap traceability row `0.1.2` including
  `tests/security/`).
- Respect roadmap scope boundaries:
  in scope is version-to-version migration execution and migration audit
  evidence; out of scope is compatibility with unknown major schema families.
- Existing fail-closed behaviour for unknown versions must be preserved.
  Migration must only run for explicitly supported source versions.
- Migration transforms must be explicit and ordered (no implicit best-effort or
  heuristic upgrades).
- Migration outcomes must preserve or tighten policy restrictiveness.
  For this task, "restrictive-equivalent" means the canonical output policy is
  not less restrictive than the source policy semantics and is equal to the
  expected canonical fixture for covered migration cases.
- Add unit and behavioural test coverage for happy and unhappy paths plus edge
  cases. Behavioural tests use `rstest-bdd` v0.5.0 where applicable.
- Record migration design decisions in `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` with consumer-visible migration behaviour and
  audit APIs.
- Mark Task 0.1.2 done in `docs/roadmap.md` once implementation is complete.
- Required quality gates for completion:
  `make check-fmt`, `make lint`, and `make test`.
- Because this task updates Markdown documentation, run docs gates as well:
  `make markdownlint`, `make nixie`, and `make fmt`.

## Tolerances (exception triggers)

- Scope tolerance:
  if the implementation exceeds 20 files or 1,000 net changed lines, stop and
  escalate with a decomposition proposal.
- Interface tolerance:
  if existing stable loader methods must change signature
  (`PolicyDefinition::from_yaml_str`, `PolicyDefinition::from_json_str`,
  `PolicyEngine::from_yaml_str`, `PolicyEngine::from_json_str`), stop and
  escalate with compatibility options.
- Dependency tolerance:
  if migration evidence cannot be implemented with existing crates plus at most
  one digest crate (`sha2`), stop and escalate before adding dependencies.
- Test tolerance:
  if behavioural coverage cannot be represented with `rstest-bdd` after two
  concrete implementation attempts, stop and document why before selecting an
  alternative.
- Semantics tolerance:
  if source-version restrictiveness semantics cannot be defined objectively
  from current docs and fixtures, stop and escalate with candidate
  interpretations.
- Iteration tolerance:
  if quality gates still fail after three focused fix loops, stop and report
  failing gates with root-cause hypotheses.

## Risks

- Risk: migration semantics are underspecified for concrete legacy versions.
  Severity: high Likelihood: medium Mitigation: define and document explicit
  supported source versions in this task, including a deterministic transform
  path to canonical v1.

- Risk: audit hash computation may be non-deterministic if map key ordering is
  not canonicalized before hashing. Severity: high Likelihood: medium
  Mitigation: hash canonicalized JSON bytes with stable key ordering and cover
  determinism with unit tests.

- Risk: behavioural tests could verify parser internals rather than observable
  contract outcomes. Severity: medium Likelihood: medium Mitigation: write
  Gherkin scenarios around load outcomes, migration-step counts, and surfaced
  audit metadata only.

- Risk: adding migration paths may accidentally weaken fail-closed behaviour for
  unknown schema families. Severity: high Likelihood: medium Mitigation: keep
  unknown-version rejection explicit and add negative tests for unsupported
  versions in both unit and behavioural suites.

## Progress

- [x] (2026-02-10 19:32Z) Reviewed roadmap, design, verification, repository
  layout, and testing guidance documents for Task 0.1.2.
- [x] (2026-02-10 19:32Z) Inspected current loader and compatibility test
  implementation baseline in `crates/zamburak-policy` and `tests/`.
- [x] (2026-02-10 19:32Z) Drafted this ExecPlan with explicit tolerances,
  staged implementation, and validation criteria.
- [x] (2026-02-10 20:17Z) Implemented migration module, legacy schema-v0
  transform, deterministic hashing, and audit-bearing loader APIs.
- [x] (2026-02-10 20:17Z) Added migration conformance suites for unit,
  compatibility (`rstest-bdd`), and security coverage.
- [x] (2026-02-10 20:17Z) Updated design and user documentation and marked
  roadmap Task 0.1.2 as done.
- [x] (2026-02-10 20:20Z) Ran full quality and documentation gates with logs
  and fixed one Clippy regression (`manual_let_else`) discovered during lint.

## Surprises & Discoveries

- Observation: the current policy loader enforces `schema_version: 1` only and
  has no migration module. Evidence: `crates/zamburak-policy/src/policy_def.rs`
  currently rejects any non-1 schema version via `UnsupportedSchemaVersion`.
  Impact: migration support must be introduced as a new explicit path while
  preserving existing fail-closed behaviour for unsupported versions.

- Observation: `tests/security/` does not yet exist in the repository.
  Evidence: `find tests -maxdepth 3 -type f` lists compatibility fixtures only.
  Impact: Task 0.1.2 should introduce the initial `tests/security/` harness for
  migration-audit evidence and failure-path checks.

- Observation: roadmap traceability names
  `crates/zamburak-policy/src/migration.rs` as a primary artefact for Task
  0.1.2. Evidence: `docs/roadmap.md` table row `0.1.2`. Impact: plan must
  include a dedicated migration module rather than embedding transform logic
  only inside `policy_def.rs`.

- Observation: sharing one fixture helper module between compatibility and
  security suites caused a `dead_code` warning in compatibility builds.
  Evidence: `cargo test --test compatibility` warned that
  `legacy_policy_v0_json` was unused. Impact: compatibility tests now reference
  the helper directly so `RUSTFLAGS="-D warnings"` gate runs remain clean.

## Decision Log

- Decision: keep canonical runtime target as schema v1 and treat migration as a
  pre-validation step into canonical v1, not an alternative runtime schema.
  Rationale: preserves Task 0.1.1 guarantees while adding explicit migration
  behaviour required by Task 0.1.2. Date/Author: 2026-02-10 / Codex

- Decision: add audit-carrying loader entrypoints while preserving existing
  loader method signatures for compatibility. Rationale: migration evidence
  must be inspectable by consumers, but existing APIs should continue working
  unchanged. Date/Author: 2026-02-10 / Codex

- Decision: define restrictive-equivalent conformance using deterministic
  canonical-policy comparisons for covered migration fixtures, plus
  non-weakening assertions for key decision fields. Rationale: this yields
  objective, automatable proof with the current policy engine surface and
  avoids speculative semantics. Date/Author: 2026-02-10 / Codex

- Decision: support one explicit migration path in this task,
  `schema_version: 0` to `schema_version: 1`, and keep all other unknown schema
  versions fail-closed. Rationale: this satisfies roadmap scope for explicit
  transforms without introducing cross-family compatibility risk. Date/Author:
  2026-02-10 / Codex

## Outcomes & Retrospective

Delivered outcomes:

- Added explicit migration module:
  `crates/zamburak-policy/src/migration.rs`.
- Implemented explicit transform chain for supported legacy input:
  `schema_version: 0` to canonical `schema_version: 1`.
- Added migration audit evidence with deterministic canonicalized SHA-256
  hashes and per-step transform records.
- Added audit-bearing API surfaces:
  `PolicyDefinition::from_yaml_str_with_migration_audit`,
  `PolicyDefinition::from_json_str_with_migration_audit`,
  `PolicyEngine::from_yaml_str_with_migration_audit`, and
  `PolicyEngine::from_json_str_with_migration_audit`.
- Added conformance tests:
  unit tests in `crates/zamburak-policy/src/migration.rs` and
  `crates/zamburak-policy/src/policy_def/tests.rs`, compatibility BDD scenarios
  in `tests/compatibility/features/policy_schema.feature`, and security tests
  in `tests/security/migration_security.rs`.
- Added migration fixtures:
  `tests/test_utils/policy-v0.yaml` and `tests/test_utils/policy-v0.json`.
- Updated documentation:
  `docs/zamburak-design-document.md`, `docs/users-guide.md`, and
  `docs/roadmap.md` (Task 0.1.2 marked done).

Gate results:

- passed: `make fmt`,
- passed: `make check-fmt`,
- passed: `make lint`,
- passed: `make test`,
- passed: `make markdownlint`,
- passed: `make nixie`.

Notable lesson:

- Shared test helpers used across multiple integration test crates should be
  referenced from each crate to avoid warning drift under
  `RUSTFLAGS="-D warnings"`.

## Context and orientation

Current state after Task 0.1.1:

- `crates/zamburak-policy/src/policy_def.rs` parses YAML/JSON directly into
  `PolicyDefinition` and accepts only `schema_version: 1`.
- `crates/zamburak-policy/src/engine.rs` builds `PolicyEngine` from validated
  `PolicyDefinition`.
- Behavioural coverage exists in
  `tests/compatibility/features/policy_schema.feature` and
  `tests/compatibility/policy_schema_bdd.rs` for accept-v1 and reject-unknown.
- Shared test fixtures exist in `tests/test_utils/policy_yaml.rs`.
- `docs/users-guide.md` currently states unknown versions are rejected with no
  migration.

Target state for Task 0.1.2:

- explicit migration logic in `crates/zamburak-policy/src/migration.rs`,
- loader paths that can return migration audit records for supported transforms,
- conformance tests in unit, compatibility, and security suites,
- updated design and user docs documenting migration semantics and APIs,
- roadmap task checkbox updated to done.

## Plan of work

Stage A: design-lock and test-first scaffolding.

- Define concrete supported migration path(s) in
  `docs/zamburak-design-document.md` for this task (for example, legacy
  `schema_version: 0` to canonical `schema_version: 1`) and define the audit
  record shape (step list plus deterministic hashes).
- Extend shared fixtures in `tests/test_utils/policy_yaml.rs` with canonical and
  legacy policy documents needed for migration tests.
- Add failing tests first:
  unit tests in `crates/zamburak-policy/src/migration.rs` (or `policy_def.rs`
  test module initially), BDD scenarios under `tests/compatibility/`, and
  security tests under `tests/security/`.

Go/no-go for Stage A: newly added migration tests fail for expected
"unimplemented migration" reasons while existing schema-v1 tests remain stable.

Stage B: implement migration primitives and loader integration.

- Create `crates/zamburak-policy/src/migration.rs` with:
  explicit transform registry/pathing, migration audit record types,
  deterministic hashing helpers, migration-specific error variants.
- Update `crates/zamburak-policy/src/policy_def.rs` to route parsed documents
  through migration when source schema is explicitly supported, then validate
  canonical schema v1.
- Preserve existing methods and add audit-enabled methods that surface migration
  metadata to callers.
- Update `crates/zamburak-policy/src/engine.rs` and
  `crates/zamburak-policy/src/lib.rs` exports for new migration/audit types.

Go/no-go for Stage B: all new and existing unit tests pass locally, including
unknown-version fail-closed behaviour.

Stage C: conformance and behavioural proof.

- Extend compatibility feature files and step definitions to cover:
  successful migration of supported legacy schema, rejection of unsupported
  schema versions, surfaced migration audit evidence for migrated inputs, no
  migration steps for already-canonical inputs.
- Add/extend security-focused tests under `tests/security/` to verify:
  deterministic hash generation, hash changes when migration input/output
  changes, no migration record on rejected unsupported schemas.
- Ensure migration conformance tests prove restrictive-equivalent outcomes by
  comparing migrated canonical policy objects to expected canonical fixtures
  and asserting non-weakening of key decision defaults.

Go/no-go for Stage C: compatibility and security suites pass and provide clear
evidence artifacts for migration correctness and auditability.

Stage D: documentation, roadmap closure, and quality gates.

- Update `docs/zamburak-design-document.md` with concrete migration-transform
  decisions taken during implementation.
- Update `docs/users-guide.md` with migration behaviour, supported source
  versions, and how consumers can access migration audit metadata.
- Mark Task 0.1.2 as done in `docs/roadmap.md`.
- Run all required gates and capture logs for traceability.

Go/no-go for Stage D: all required gates pass and documentation matches shipped
behaviour.

## Concrete steps

Run all commands from repository root: `/home/user/project`.

1. Baseline check and repository orientation.

       git status --short
       rg --files crates tests docs policies | sort

2. Add migration tests first (unit, compatibility, security).

       mkdir -p tests/security tests/compatibility/features
       # Edit migration-focused tests and fixtures in:
       # - crates/zamburak-policy/src/migration.rs (new or test module)
       # - tests/compatibility/features/
       # - tests/compatibility/
       # - tests/security/
       # - tests/test_utils/policy_yaml.rs

3. Run targeted suites during implementation loops.

       set -o pipefail && cargo test -p zamburak-policy migration | tee /tmp/test-0-1-2-migration-unit.out
       set -o pipefail && cargo test --test compatibility migration | tee /tmp/test-0-1-2-migration-compat.out
       set -o pipefail && cargo test --test security migration | tee /tmp/test-0-1-2-migration-security.out

4. Implement migration module and loader integration.

       # Edit:
       # - crates/zamburak-policy/src/migration.rs
       # - crates/zamburak-policy/src/policy_def.rs
       # - crates/zamburak-policy/src/engine.rs
       # - crates/zamburak-policy/src/lib.rs
       # - crates/zamburak-policy/Cargo.toml (if digest dependency is required)

5. Update docs and roadmap completion state.

       # Edit:
       # - docs/zamburak-design-document.md
       # - docs/users-guide.md
       # - docs/roadmap.md

6. Run mandatory Rust quality gates with logs.

       set -o pipefail && make check-fmt | tee /tmp/check-fmt-0-1-2.out
       set -o pipefail && make lint | tee /tmp/lint-0-1-2.out
       set -o pipefail && make test | tee /tmp/test-0-1-2.out

7. Run documentation gates because Markdown is modified.

       set -o pipefail && make markdownlint | tee /tmp/markdownlint-0-1-2.out
       set -o pipefail && make nixie | tee /tmp/nixie-0-1-2.out
       set -o pipefail && make fmt | tee /tmp/fmt-0-1-2.out

## Validation and acceptance

Acceptance criteria for Task 0.1.2:

- Behaviour:
  supported legacy policy schema input is migrated explicitly to canonical v1;
  unsupported schema families are rejected fail-closed.
- Migration evidence:
  migration-capable loader APIs return auditable records with transform-step
  and deterministic hash evidence.
- Unit tests:
  migration transform logic covers happy, unhappy, and edge cases.
- Behavioural tests:
  compatibility scenarios using `rstest-bdd` v0.5.0 cover load success, schema
  rejection, and migration-audit visibility.
- Security tests:
  migration hash determinism and fail-closed unsupported-version behaviour are
  validated in `tests/security/`.
- Documentation:
  `docs/zamburak-design-document.md` and `docs/users-guide.md` reflect final
  migration semantics and APIs.
- Roadmap state:
  Task 0.1.2 in `docs/roadmap.md` is marked `[x]`.
- Quality gates:
  `make check-fmt`, `make lint`, and `make test` succeed.

## Idempotence and recovery

- File creation and edits in this plan are idempotent when rerun.
- If a targeted test fails, fix only the failing scope, rerun that targeted
  suite, then rerun full quality gates.
- If a quality gate fails because of unrelated pre-existing issues, record the
  exact failure in `Decision Log` with evidence before proceeding.
- Do not remove migration evidence paths to pass tests; failures in migration
  audit assertions require implementation correction, not test relaxation.

## Artefacts and notes

Keep the following artefacts for review:

- `git diff` for the final change set,
- targeted logs:
  `/tmp/test-0-1-2-migration-unit.out`, `/tmp/test-0-1-2-migration-compat.out`,
  `/tmp/test-0-1-2-migration-security.out`,
- quality-gate logs:
  `/tmp/check-fmt-0-1-2.out`, `/tmp/lint-0-1-2.out`, `/tmp/test-0-1-2.out`,
  `/tmp/markdownlint-0-1-2.out`, `/tmp/nixie-0-1-2.out`, `/tmp/fmt-0-1-2.out`,
- criterion-to-evidence mapping that shows which tests and docs prove each
  acceptance requirement.

## Interfaces and dependencies

Implementation should leave explicit migration interfaces in
`crates/zamburak-policy`:

- a migration module at `crates/zamburak-policy/src/migration.rs` defining
  explicit transform execution from supported source versions to canonical v1,
- audit record types that expose:
  source schema version, target schema version, ordered applied transform
  steps, deterministic input/output hash strings for each step,
- loader APIs that preserve existing method signatures and add opt-in
  audit-bearing variants for consumers that need migration evidence.

Planned dependency posture:

- Prefer existing dependencies first.
- If cryptographic digest support is not already available, add only one digest
  crate (`sha2`) with a caret requirement and document why.

## Revision note

Initial draft created for roadmap Task 0.1.2 with explicit migration strategy,
test plan (unit + compatibility + security), documentation obligations, and
quality-gate requirements.

Revision (2026-02-10): completed implementation and validation for Task 0.1.2,
updated status to `COMPLETE`, recorded delivery outcomes and gate evidence, and
captured the fixture-warning discovery plus final mitigation.
