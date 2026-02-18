# Align toolchain and quality-gate baseline with repository configuration (Task 0.2.1)

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: DRAFT

`PLANS.md` is not present in this repository, so this document is the
controlling execution plan for roadmap Task 0.2.1.

## Purpose / big picture

Implement `docs/roadmap.md` Task 0.2.1 by making the toolchain pin and
quality-gate command contract consistent across repository configuration and
baseline documentation.

After this change, maintainers should observe one canonical baseline across
`rust-toolchain.toml`, `Makefile`, and baseline docs, with drift detection
covered by tests and enforced by required gates. Success is observable when the
new baseline consistency tests pass, required gates pass, and Task 0.2.1 is
marked done in `docs/roadmap.md`.

## Constraints

- Implement against these requirement signposts only:
  `docs/tech-baseline.md` sections "Canonical version baseline" and "Baseline
  usage contract", `docs/zamburak-engineering-standards.md` section "Command
  and gateway standards", and `docs/repository-layout.md` section "Root and
  operational files".
- Keep scope to baseline alignment for `rust-toolchain.toml`, `Makefile`, and
  baseline-document consistency.
- Out of scope: introducing additional build systems.
- Preserve Makefile-first command workflow; do not replace gate invocation with
  ad hoc scripts.
- Add validation coverage for happy and unhappy paths plus relevant edge cases.
  Use unit tests and behavioural tests with `rstest-bdd` v0.5.0 where
  behavioural scenario narration is appropriate.
- Record design decisions in `docs/zamburak-design-document.md` if this task
  changes baseline governance semantics.
- Update `docs/users-guide.md` only if library-consumer-visible behaviour or
  API changes; if none, record that conclusion in `Decision Log`.
- Completion requires `make check-fmt`, `make lint`, and `make test` to pass.
- Because Markdown docs will be updated, also run `make markdownlint`,
  `make nixie`, and `make fmt`.
- Mark roadmap Task 0.2.1 as `[x]` in `docs/roadmap.md` only after all
  completion criteria and gates succeed.

## Tolerances (exception triggers)

- Scope tolerance: if implementation exceeds 10 files or 450 net changed lines,
  stop and escalate with a split proposal.
- Interface tolerance: if changes to make-target names (`check-fmt`, `lint`,
  `test`) are required, stop and escalate with compatibility options.
- Dependency tolerance: if any new dependency is required for baseline
  consistency checks, stop and escalate before adding it.
- Test strategy tolerance: if `rstest-bdd` cannot express the behavioural
  baseline checks after two concrete attempts, stop and record why before
  selecting a non-BDD behavioural fallback.
- Ambiguity tolerance: if documentation signposts disagree on canonical command
  semantics in a way that changes gate meaning, stop and escalate with explicit
  interpretations and trade-offs.
- Iteration tolerance: if required gates still fail after three focused fix
  loops, stop and report failures with root-cause hypotheses.

## Risks

- Risk: baseline docs and Makefile currently describe similar but not identical
  command forms (`--workspace` versus `--all`, `cargo doc` in `lint`, extra
  test flags). Severity: high Likelihood: high Mitigation: choose one canonical
  contract and align all artefacts in one change set with explicit acceptance
  tests.

- Risk: behavioural tests for contributor tooling can become brittle if they
  assert full command strings including irrelevant formatting. Severity: medium
  Likelihood: medium Mitigation: assert stable command fragments and semantics,
  not whitespace or argument ordering that does not alter behaviour.

- Risk: aligning to stricter gate commands may expose latent warnings in crates
  not currently exercised by `make lint`. Severity: medium Likelihood: medium
  Mitigation: run targeted and full gates early, then fix surfaced defects
  rather than weakening the contract.

## Progress

- [x] (2026-02-18 12:00Z) Reviewed roadmap Task 0.2.1 and all listed signpost
  documents.
- [x] (2026-02-18 12:05Z) Inspected current `Makefile`, `rust-toolchain.toml`,
  and workspace testing layout.
- [x] (2026-02-18 12:10Z) Drafted this ExecPlan with staged execution,
  tolerances, and validation criteria.
- [ ] Implement baseline alignment updates in configuration and docs.
- [ ] Add/extend baseline consistency unit and behavioural tests.
- [ ] Run required gates and capture evidence logs.
- [ ] Update roadmap status for Task 0.2.1.

## Surprises & Discoveries

- Observation: `docs/tech-baseline.md` pins `nightly-2026-01-30`, and
  `rust-toolchain.toml` already matches this value. Evidence: direct file
  comparison during planning. Impact: toolchain-channel work is likely minimal;
  focus shifts to gate-command and documentation consistency.

- Observation: `Makefile` currently defines `lint` as `cargo doc --no-deps`
  plus `cargo clippy`, while standards docs describe lint as clippy-only.
  Evidence: `Makefile` target body versus
  `docs/zamburak-engineering-standards.md` command block. Impact: Task 0.2.1
  needs an explicit canonical decision for `lint` semantics.

- Observation: `Makefile` uses `cargo fmt --all` and additional test flags,
  while baseline docs describe `--workspace` and simpler test invocation.
  Evidence: `Makefile` compared against `docs/tech-baseline.md` baseline table.
  Impact: alignment changes are required in either code or docs, with tests to
  prevent future drift.

## Decision Log

- Decision: treat Task 0.2.1 as a contract-alignment task with tests that guard
  against future drift, rather than a one-time textual sync. Rationale: roadmap
  completion criteria require ongoing consistency, not merely a snapshot edit.
  Date/Author: 2026-02-18 / Codex

- Decision: include behavioural checks with `rstest-bdd` only for externally
  observable command-contract behaviour (for example, make-target semantics),
  and keep low-level parsing checks as unit tests. Rationale: this satisfies
  the requested unit plus behavioural coverage while keeping test intent clear
  and maintainable. Date/Author: 2026-02-18 / Codex

- Decision: require explicit evidence logs for both code and docs gates using
  `set -o pipefail` and `tee`. Rationale: this matches `AGENTS.md` and the
  engineering standards command logging convention. Date/Author: 2026-02-18 /
  Codex

## Outcomes & retrospective

Execution has not started. Expected outcomes at completion:

- `rust-toolchain.toml`, `Makefile`, and baseline docs define one canonical
  toolchain and gate-command contract with no contradictions.
- Baseline consistency checks cover happy and unhappy paths and guard against
  regressions.
- Required gates pass with captured evidence logs.
- `docs/roadmap.md` Task 0.2.1 is marked `[x]`.

Retrospective notes will be added after implementation.

## Context and orientation

Current repository context relevant to this task:

- Toolchain pin lives in `rust-toolchain.toml`.
- Quality-gate entrypoints live in `Makefile` targets `check-fmt`, `lint`, and
  `test`.
- Baseline policy is documented in `docs/tech-baseline.md`.
- Command and logging standards are documented in
  `docs/zamburak-engineering-standards.md` and `AGENTS.md`.
- Root-operational-file expectations are documented in
  `docs/repository-layout.md`.
- Existing BDD harnesses reside in `tests/compatibility/` and `tests/security/`
  and already use `rstest-bdd` v0.5.0.

No prior context is required beyond this plan and the repository tree.

## Plan of work

Stage A: lock the canonical baseline contract (no edits yet).

Reconcile differences between `Makefile` commands and baseline docs. Decide the
single canonical command semantics for `check-fmt`, `lint`, and `test`, then
list exact edits needed in config and docs.

Go/no-go for Stage A: every mismatch is identified and mapped to a concrete
edit; no implementation starts with unresolved contract ambiguity.

Stage B: add tests first for baseline consistency.

Add unit tests for deterministic checks (for example, parsing pinned toolchain
channel and required command fragments from source files). Add behavioural
tests using `rstest-bdd` v0.5.0 for user-observable baseline behaviour (happy
path: baseline aligned; unhappy path: representative mismatch fixture fails;
edge cases: whitespace/order variance that should not fail).

Go/no-go for Stage B: new tests fail before baseline-alignment edits and prove
that the intended contract is being enforced.

Stage C: align configuration and documentation.

Apply minimal edits to `rust-toolchain.toml`, `Makefile`, and baseline docs so
all canonical values and gate commands match. Keep target names stable and
maintain Makefile-first invocation patterns. If command semantics are
clarified, update `docs/tech-baseline.md` and
`docs/zamburak-engineering-standards.md` together.

Go/no-go for Stage C: all baseline consistency tests pass and no contradictory
baseline statements remain.

Stage D: documentation closure, gates, and roadmap update.

Update `docs/users-guide.md` only if there is consumer-visible behaviour/API
impact; otherwise record explicit no-change rationale. Record design decisions
in `docs/zamburak-design-document.md` if governance semantics changed. Run all
required gates with logs, then mark Task 0.2.1 done in `docs/roadmap.md`.

Go/no-go for Stage D: every completion criterion in roadmap Task 0.2.1 is
satisfied and evidenced.

## Concrete steps

Run commands from `/home/user/project`.

1. Baseline inventory and mismatch capture.

    set -o pipefail && cat rust-toolchain.toml \
      | tee /tmp/toolchain-task-0-2-1-$(git branch --show-current).out |
    set -o pipefail && make -n check-fmt \
      | tee /tmp/check-fmt-dry-task-0-2-1-$(git branch --show-current).out |
    set -o pipefail && make -n lint \
      | tee /tmp/lint-dry-task-0-2-1-$(git branch --show-current).out |
    set -o pipefail && make -n test \
      | tee /tmp/test-dry-task-0-2-1-$(git branch --show-current).out |

2. Add tests before alignment edits.

    Edit/add test artefacts (exact paths to be finalized in Stage A), likely:
    - `tests/compatibility/main.rs`
    - `tests/compatibility/toolchain_baseline_contract.rs`
    - `tests/compatibility/features/tooling_baseline.feature`
    - `tests/compatibility/tooling_baseline_bdd.rs`

3. Align baseline artefacts.

    Edit:
    - `rust-toolchain.toml` (only if mismatch exists)
    - `Makefile`
    - `docs/tech-baseline.md`
    - `docs/zamburak-engineering-standards.md`
    - `docs/repository-layout.md` (only if root-operational-file contract text
      needs correction)

4. Update design/user docs where required.

    Edit conditionally:
    - `docs/zamburak-design-document.md`
    - `docs/users-guide.md`

5. Run required quality gates with evidence logs.

    set -o pipefail && make check-fmt \
      | tee /tmp/check-fmt-zamburak-$(git branch --show-current).out |
    set -o pipefail && make lint \
      | tee /tmp/lint-zamburak-$(git branch --show-current).out |
    set -o pipefail && make test \
      | tee /tmp/test-zamburak-$(git branch --show-current).out |
    set -o pipefail && make markdownlint \
      | tee /tmp/markdownlint-zamburak-$(git branch --show-current).out |
    set -o pipefail && make nixie \
      | tee /tmp/nixie-zamburak-$(git branch --show-current).out |
    set -o pipefail && make fmt \
      | tee /tmp/fmt-zamburak-$(git branch --show-current).out |

6. Mark roadmap completion.

    Update `docs/roadmap.md` Task 0.2.1 checkbox from `[ ]` to `[x]` only after
    Step 5 succeeds and acceptance evidence is complete.

## Validation and acceptance

Task 0.2.1 is complete only when all conditions are true:

- Baseline consistency: toolchain pin and gate commands are identical in
  `rust-toolchain.toml`, `Makefile`, and baseline docs.
- Test coverage: unit and behavioural tests (using `rstest-bdd` v0.5.0 where
  applicable) validate happy path, unhappy path, and edge-case baseline checks.
- Required gates: `make check-fmt`, `make lint`, and `make test` pass.
- Documentation gates: `make markdownlint`, `make nixie`, and `make fmt` pass.
- Documentation obligations: design and user docs are updated where required,
  with explicit rationale when no user-guide update is needed.
- Roadmap closure: Task 0.2.1 is marked done in `docs/roadmap.md`.

## Idempotence and recovery

- All commands in this plan are safe to rerun.
- If a consistency test fails, update only the conflicting contract artefact,
  rerun targeted tests, then rerun full gates.
- If gate failures expose unrelated pre-existing defects, isolate and document
  them in `Surprises & Discoveries` before continuing.
- Do not weaken checks to force green status; repair source-of-truth drift.

## Artifacts and notes

Retain these artefacts as implementation evidence:

- Dry-run command captures:
  `/tmp/check-fmt-dry-task-0-2-1-<branch>.out`,
  `/tmp/lint-dry-task-0-2-1-<branch>.out`,
  `/tmp/test-dry-task-0-2-1-<branch>.out`.
- Gate logs:
  `/tmp/check-fmt-zamburak-<branch>.out`, `/tmp/lint-zamburak-<branch>.out`,
  `/tmp/test-zamburak-<branch>.out`, `/tmp/markdownlint-zamburak-<branch>.out`,
  `/tmp/nixie-zamburak-<branch>.out`, `/tmp/fmt-zamburak-<branch>.out`.
- Criterion-to-evidence mapping from roadmap completion criteria to concrete
  tests and logs.

## Interfaces and dependencies

- No new production-library interfaces are expected.
- Test interfaces may be added under `tests/compatibility/` to codify baseline
  contracts.
- Dependency posture: use existing workspace dependencies; do not add new
  dependencies without escalation per tolerance rules.

## Revision note

This file was revised from a completed Task 0.1.3 plan to a new draft ExecPlan
for roadmap Task 0.2.1 at user request. The remaining work now focuses on
baseline-tooling and documentation-contract alignment instead of authority
lifecycle behaviour.
