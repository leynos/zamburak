# Establish automation script baseline for roadmap-delivered scripts (Task 0.2.3)

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: DRAFT

`PLANS.md` is not present in this repository at draft time, so this document is
the governing execution plan for this task.

## Purpose / big picture

Implement roadmap Task 0.2.3 from `docs/roadmap.md`: establish a repository
baseline so every roadmap-delivered automation script follows the scripting
standards for runtime metadata, command invocation, and test coverage.

After this change, contributors can add scripts under `scripts/` with one clear
contract: scripts use the `uv` metadata block and Python 3.13 baseline,
external commands use Cuprum execution patterns, and each script ships with
matching tests (unit and behavioural where meaningful).

Task completion is observable when script baseline checks and script tests are
wired into local and CI execution paths, the design decision record is updated,
and roadmap task `0.2.3` is marked done.

## Constraints

- Implement to these normative signposts:
  `docs/scripting-standards.md` sections "Language and runtime", "Testing
  expectations", and "CI wiring: GitHub Actions (Cyclopts-first)";
  `docs/repository-layout.md` section "Root and operational files"
  (`scripts/`); `docs/tech-baseline.md` section "Required engineering tools and
  rationale".
- Keep scope limited to script runtime metadata, command invocation patterns,
  and script test conventions.
- Dependencies for this roadmap task are none.
- Out of scope: replacing non-script Rust automation with Python.
- Baseline must be forward-looking: future roadmap scripts must be validated
  automatically against the baseline.
- Script tests must include both happy and unhappy paths and relevant edge
  cases.
- `rstest-bdd` v0.5.0 applies only where Rust behavioural tests are introduced.
  For this task, behavioural testing for Python scripts should follow
  `docs/scripting-standards.md` (`pytest-bdd`) unless a Rust-facing change
  makes `rstest-bdd` applicable.
- Record design decisions taken during implementation in
  `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` if and only if this task changes
  library-consumer-visible behaviour or APIs.
- Mark roadmap task `0.2.3` done in `docs/roadmap.md` only after all required
  checks pass.
- Required quality gates before completion:
  `make check-fmt`, `make lint`, and `make test`.
- Because this task updates Markdown docs, docs gates are also required:
  `make markdownlint`, `make nixie`, and `make fmt`.

## Tolerances (exception triggers)

- Scope tolerance:
  if implementation exceeds 14 files or 900 net changed lines, stop and
  escalate with a split plan.
- Interface tolerance:
  if meeting this task requires changing stable Rust public APIs, stop and
  escalate with compatibility options.
- Dependency tolerance:
  if new tooling beyond script-testing essentials (`pytest`, `pytest-mock`,
  `pytest-bdd`, `cmd-mox`) is required, stop and escalate before adding it.
- CI tolerance:
  if script-test wiring requires redesigning the entire CI workflow rather than
  adding targeted steps, stop and escalate with options.
- Behavioural-testing tolerance:
  if no meaningful behavioural scenarios can be expressed after two concrete
  attempts, document why and fall back to unit coverage only with explicit
  justification in `Decision Log`.
- Iteration tolerance:
  if required gates remain failing after three focused fix loops, stop and
  escalate with failing logs and root-cause hypotheses.

## Risks

- Risk: baseline checks may accidentally include helper modules not intended as
  runnable scripts (for example underscore-prefixed helpers), producing false
  failures. Severity: medium Likelihood: high Mitigation: define explicit
  discovery rules (for example, enforce baseline on executable entrypoint
  scripts only) and test the selector.

- Risk: script tests may run locally but be omitted from CI, undermining
  completion criteria. Severity: high Likelihood: medium Mitigation: add
  explicit CI workflow steps and verify failure behaviour by referencing script
  test command output.

- Risk: command invocation standards may remain advisory if no executable
  contract test validates Cuprum usage patterns. Severity: medium Likelihood:
  medium Mitigation: add tests that assert expected invocation-path helpers and
  failure handling conventions in representative scripts.

- Risk: user-guide churn without consumer-visible API changes may create noisy
  docs. Severity: low Likelihood: medium Mitigation: apply a strict
  "consumer-visible change required" gate before editing `docs/users-guide.md`,
  and record the decision either way.

## Progress

- [x] (2026-02-20 17:16Z) Reviewed roadmap task `0.2.3` and signpost docs.
- [x] (2026-02-20 17:16Z) Confirmed repository currently has `scripts/` with
  helper module only and no script-test harness.
- [x] (2026-02-20 17:16Z) Confirmed `PLANS.md` is absent.
- [x] (2026-02-20 17:16Z) Drafted this ExecPlan.
- [ ] Implement script baseline scaffolding and script test harness.
- [ ] Wire script baseline checks into CI and local command flow.
- [ ] Update design decisions, roadmap completion state, and user guide (if
  applicable).
- [ ] Run all required quality gates and archive logs.

## Surprises & Discoveries

- Observation: no Qdrant MCP note tools are exposed in this session.
  Evidence: `list_mcp_resources` and `list_mcp_resource_templates` returned
  empty results. Impact: this planning pass cannot retrieve or store
  project-memory notes via Qdrant; rely on repository docs only.

- Observation: `scripts/` currently contains only `scripts/_cuprum_helpers.py`.
  Evidence: repository tree inspection. Impact: baseline discovery rules must
  distinguish helper modules from roadmap-delivered script entrypoints.

## Decision Log

- Decision: treat this change as baseline-contract establishment rather than
  adding a large script portfolio. Rationale: roadmap scope for Task 0.2.3 is
  standards and conventions, not feature script rollout. Date/Author:
  2026-02-20 / Codex

- Decision: keep `rstest-bdd` behavioural coverage conditional on Rust-facing
  behaviour changes, and use scripting-standards BDD guidance for Python script
  behaviours. Rationale: aligns the user request with the task scope and
  existing scripting-standards contract. Date/Author: 2026-02-20 / Codex

## Outcomes & Retrospective

Pending implementation. Success criteria at completion:

- script baseline rules are codified and executable,
- script unit/behavioural tests exist for baseline enforcement,
- CI and local workflows execute script checks,
- design and roadmap docs are updated and consistent,
- required quality gates pass.

Retrospective notes will be added when execution completes.

## Context and orientation

Current repository state relevant to Task 0.2.3:

- `docs/scripting-standards.md` defines Python 3.13 + `uv` metadata, Cuprum
  command patterns, testing expectations, and Cyclopts-first CI wiring.
- `docs/repository-layout.md` reserves `scripts/` for operational helper
  scripts.
- `docs/tech-baseline.md` requires quality-gate execution via Make targets.
- `.github/workflows/ci.yml` currently runs Rust and docs gates but has no
  script-specific baseline/test stage.
- `scripts/` contains only helper module `scripts/_cuprum_helpers.py`.
- `docs/roadmap.md` marks Task 0.2.3 as not done.

Target state for Task 0.2.3:

- baseline contract enforcement exists for roadmap-delivered scripts,
- script test conventions are implemented and runnable,
- CI wiring includes script checks,
- design-document decisions are recorded,
- roadmap Task 0.2.3 is marked `[x]`,
- `docs/users-guide.md` is updated only if library-consumer behaviour changes.

## Plan of work

Stage A: lock baseline contract and define discoverability boundaries.

- Specify which files count as roadmap-delivered scripts (for example, runnable
  Python entrypoints in `scripts/` that are not underscore-prefixed helpers).
- Define enforceable baseline assertions:
  required `uv` metadata block, Python version floor, command invocation style,
  and required test pairing.
- Capture these decisions in `docs/zamburak-design-document.md` under a
  delivery-baseline section.

Go/no-go for Stage A: baseline assertions are explicit, testable, and mapped to
the roadmap completion criteria.

Stage B: add script baseline validation and tests.

- Create script test scaffolding under `scripts/tests/`, including:
  unit tests for metadata and invocation conventions, and behavioural scenarios
  for script execution flows where scenario narratives add clarity.
- Ensure tests cover happy path, unhappy path, and edge conditions:
  missing metadata, incorrect runtime declaration, disallowed invocation
  patterns, and missing test pairing for new scripts.
- If implementation introduces Rust-facing enforcement utilities, add
  `rstest`/`rstest-bdd` (v0.5.0) coverage for those Rust paths; otherwise keep
  script tests in the Python harness.

Go/no-go for Stage B: baseline tests fail on non-conforming scripts and pass on
conforming fixtures.

Stage C: wire commands and CI integration.

- Add local command path(s) to run script baseline tests predictably
  (preferably via `Makefile` target additions that align with existing command
  conventions).
- Update `.github/workflows/ci.yml` to run script baseline checks in CI,
  following Cyclopts-first env wiring guidance where script invocation is
  parameterized.
- Ensure script test failures are merge-blocking in CI.

Go/no-go for Stage C: CI and local runs both execute script baseline checks and
produce consistent pass/fail outcomes.

Stage D: docs, roadmap closure, and full validation.

- Update `docs/zamburak-design-document.md` with final implementation decisions
  taken in Stages A-C.
- Update `docs/users-guide.md` only if any library-consumer-visible behaviour
  or API changed; otherwise record explicit no-change rationale in
  `Decision Log`.
- Mark Task 0.2.3 done in `docs/roadmap.md`.
- Run full required quality gates and capture logs per `AGENTS.md`.

Go/no-go for Stage D: all required gates pass and documentation remains
consistent with implementation.

## Concrete steps

Run all commands from repository root: `/home/user/project`.

1. Baseline orientation and branch capture.

       git status --short
       git branch --show-current
       tree -a -L 3 scripts .github/workflows docs

2. Implement Stage A and Stage B edits.

       # Edit baseline docs and add script baseline tests:
       # - docs/zamburak-design-document.md
       # - scripts/tests/…
       # - optional Makefile target(s) for script checks

3. Execute script-focused tests while iterating.

       set -o pipefail && \
       uv run --with pytest --with pytest-mock --with pytest-bdd --with cmd-mox \
       pytest scripts/tests | tee /tmp/script-tests-zamburak-$(git branch --show-current).out

4. Implement Stage C CI wiring.

       # Edit .github/workflows/ci.yml and related command wiring.

5. Run required repository quality gates with logs.

       set -o pipefail && make check-fmt | tee /tmp/check-fmt-zamburak-$(git branch --show-current).out
       set -o pipefail && make lint | tee /tmp/lint-zamburak-$(git branch --show-current).out
       set -o pipefail && make test | tee /tmp/test-zamburak-$(git branch --show-current).out

6. Run documentation gates because docs are modified.

       set -o pipefail && make markdownlint | tee /tmp/markdownlint-zamburak-$(git branch --show-current).out
       set -o pipefail && make nixie | tee /tmp/nixie-zamburak-$(git branch --show-current).out
       set -o pipefail && make fmt | tee /tmp/fmt-zamburak-$(git branch --show-current).out

7. Final closure edits.

       # Update docs/roadmap.md task checkbox for 0.2.3 to [x]
       # Update docs/users-guide.md only if consumer-visible behaviour changed

## Validation and acceptance

Acceptance is met only when all criteria below are true.

- Behaviour:
  any new roadmap-delivered script follows runtime metadata and command
  invocation standards, with paired script tests.
- Tests:
  script baseline unit and behavioural suites pass for conforming scripts and
  fail when baseline rules are violated.
- Rust behavioural tests:
  if this task adds Rust-facing baseline behaviour, `rstest-bdd` v0.5.0
  scenarios are added and pass; if not applicable, rationale is recorded.
- CI wiring:
  CI executes script baseline checks and fails on violations.
- Documentation:
  `docs/zamburak-design-document.md` records baseline design decisions;
  `docs/users-guide.md` is updated when consumer-visible changes exist.
- Roadmap state:
  `docs/roadmap.md` marks Task 0.2.3 as done.
- Quality gates:
  `make check-fmt`, `make lint`, and `make test` pass; docs gates pass for this
  docs-heavy change.

## Idempotence and recovery

- Script baseline checks and tests must be rerunnable without mutating
  repository state.
- If a gate fails, fix only the failing cause, rerun that gate, then rerun the
  full required sequence.
- If CI wiring changes break unrelated jobs, isolate and revert only the CI
  hunk, then reapply in smaller increments.

## Artifacts and notes

Keep these artefacts while executing:

- `/tmp/script-tests-zamburak-<branch>.out`,
- `/tmp/check-fmt-zamburak-<branch>.out`,
- `/tmp/lint-zamburak-<branch>.out`,
- `/tmp/test-zamburak-<branch>.out`,
- `/tmp/markdownlint-zamburak-<branch>.out`,
- `/tmp/nixie-zamburak-<branch>.out`,
- `/tmp/fmt-zamburak-<branch>.out`,
- final `git diff` showing baseline enforcement, tests, CI wiring, and docs
  updates.

## Interfaces and dependencies

Prescriptive interfaces expected by the end of implementation:

- Script baseline enforcement entrypoint(s) under `scripts/` and/or `Makefile`
  must provide a single command path for local and CI usage.
- Script tests live under `scripts/tests/` and mirror script names and
  behaviours.
- CI references the same command path used locally to avoid drift.

Expected dependency surface for script testing:

- `pytest`,
- `pytest-mock`,
- `pytest-bdd`,
- `cmd-mox`.

No additional dependency families should be introduced unless a tolerance
escalation is recorded first.
