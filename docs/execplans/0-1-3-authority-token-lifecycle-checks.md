# Wire phase-gate verification checks into CI (Task 0.2.2)

This execution plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: DRAFT

This repository does not include `PLANS.md`; therefore this document is the
authoritative execution plan for this task.

## Purpose / big picture

Implement roadmap Task 0.2.2 from `docs/roadmap.md`: wire phase-gate checks
from `docs/verification-targets.md` into merge-blocking CI so phase advancement
is blocked when mandated suites are missing or failing.

After this change, maintainers can observe phase-gate status directly in CI:

- when required suites exist and pass, phase-gate jobs are green;
- when a required suite is missing or failing, CI fails with explicit
  escalation guidance from the failure policy.

Completion is observable when CI enforces the gate set for the selected phase,
tests cover happy and unhappy paths for gate resolution and failure behaviour,
and the roadmap entry for Task 0.2.2 is marked done.

## Constraints

- Implement to these normative signposts:
  `docs/verification-targets.md` sections "Acceptance gates for implementation
  phases" and "Failure and escalation policy",
  `docs/zamburak-engineering-standards.md` section "Testing and verification
  evidence standards", and `docs/repository-layout.md` section
  `.github/workflows/`.
- Respect roadmap scope boundaries:
  in scope is merge-blocking phase-gate wiring and gate-failure escalation
  behaviour; out of scope is release-train orchestration outside repository CI.
- Dependencies required by roadmap are already satisfied:
  Task 0.1.1 and Task 0.1.3 are complete.
- Keep CI checks deterministic and repository-local. Do not depend on mutable
  external services for pass/fail logic.
- Add unit tests and behavioural tests for gate wiring and escalation outcomes,
  covering happy and unhappy paths plus edge cases.
- Use `rstest-bdd` v0.5.0 for behavioural scenarios where it improves
  contract-level clarity.
- Record implementation design decisions in
  `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` if there is any library-consumer-visible
  behaviour or API change. If none, keep this explicit in the change notes.
- Mark Task 0.2.2 as done in `docs/roadmap.md` only after all completion
  criteria and quality gates are satisfied.
- Required completion gates: `make check-fmt`, `make lint`, and `make test`.
- Because Markdown docs will be updated, also run:
  `make markdownlint`, `make nixie`, and `make fmt`.

## Tolerances (exception triggers)

- Scope tolerance:
  if implementation requires edits in more than 12 files or more than 900 net
  changed lines, stop and escalate with a split proposal.
- Interface tolerance:
  if this task requires public library API signature changes, stop and escalate
  with compatibility options before proceeding.
- Dependency tolerance:
  if implementation requires a new third-party dependency for gate execution,
  stop and escalate before adding it.
- CI ownership tolerance:
  if branch-protection or required-check configuration outside this repository
  is required to satisfy completion criteria, stop and escalate with an ops
  handoff note.
- Behavioural-test tolerance:
  if `rstest-bdd` cannot represent phase-gate scenarios after two concrete
  attempts, stop and document why before switching to non-BDD coverage.
- Iteration tolerance:
  if required gates still fail after three focused fix loops, stop and report
  failing commands with root-cause hypotheses.

## Risks

- Risk: phase advancement is not currently represented by a machine-readable
  repository artefact. Severity: high Likelihood: high Mitigation: introduce an
  explicit in-repo phase-gate manifest and phase-target selector used by both
  CI and local verification commands.

- Risk: required suites for later contract areas (for example LLM sink and
  localization) may not exist yet, causing premature CI failures. Severity:
  high Likelihood: medium Mitigation: enforce gates based on an explicit target
  phase and make phase advancement itself a gated change.

- Risk: shell-only wiring in workflows can become brittle and hard to test.
  Severity: medium Likelihood: medium Mitigation: isolate gate-selection logic
  in a testable module and keep CI step scripts thin.

- Risk: escalation guidance may be omitted or unclear at failure time.
  Severity: medium Likelihood: medium Mitigation: emit standard failure summary
  text that mirrors `docs/verification-targets.md` "Failure and escalation
  policy".

## Progress

- [x] (2026-02-18 11:35Z) Reviewed roadmap Task 0.2.2 scope, dependencies, and
  completion criteria.
- [x] (2026-02-18 11:35Z) Reviewed verification-target acceptance gates and
  failure escalation policy.
- [x] (2026-02-18 11:35Z) Reviewed current CI wiring in
  `.github/workflows/ci.yml` and current `Makefile` targets.
- [x] (2026-02-18 11:35Z) Drafted this ExecPlan for Task 0.2.2.
- [ ] Implement phase-gate manifest, execution wiring, and merge-blocking CI
  jobs.
- [ ] Add unit and behavioural tests (including `rstest-bdd` where applicable)
  for happy/unhappy/edge phase-gate outcomes.
- [ ] Update design and user documentation where required.
- [ ] Run required quality gates with log capture and mark roadmap Task 0.2.2
  done.

## Surprises & Discoveries

- Observation: the target plan path
  `docs/execplans/0-1-3-authority-token-lifecycle-checks.md` previously
  contained the completed Task 0.1.3 execution record. Evidence: file contents
  before this draft were `Status: DONE` and scoped to authority lifecycle
  checks. Impact: this plan explicitly documents Task 0.2.2 despite legacy
  filename mismatch.

- Observation: repository CI currently has one broad job in
  `.github/workflows/ci.yml` and no explicit phase-gate matrix. Evidence:
  current workflow runs format/lint/tests/coverage but does not define
  phase-targeted verification gate checks. Impact: Task 0.2.2 requires
  introducing phase-gate specific wiring.

- Observation: MCP project-memory resources are not exposed in this session.
  Evidence: `list_mcp_resources` and `list_mcp_resource_templates` returned
  empty results. Impact: this draft relies on repository-local documentation
  only.

## Decision Log

- Decision: draft and maintain Task 0.2.2 in the user-requested file path even
  though the filename references Task 0.1.3. Rationale: follow explicit user
  instruction while preserving task identity in the document title and content.
  Date/Author: 2026-02-18 / Codex

- Decision: use a machine-readable phase-gate contract in-repo instead of
  parsing human prose from `docs/verification-targets.md` at runtime.
  Rationale: reduces CI fragility and enables deterministic unit testing.
  Date/Author: 2026-02-18 / Codex

- Decision: require CI to fail with explicit escalation instructions that map
  to the documented failure policy. Rationale: the task scope includes
  gate-failure escalation behaviour, not only pass/fail wiring. Date/Author:
  2026-02-18 / Codex

## Outcomes & Retrospective

Not started for implementation. This section will be updated after delivery
with:

- phase-gate wiring outcomes,
- test and CI evidence,
- gate-failure escalation evidence,
- lessons learned and follow-up actions.

## Context and orientation

Current relevant repository state:

- CI workflow:
  `.github/workflows/ci.yml` runs `make check-fmt`, Markdown lint, `make lint`,
  and a coverage action, but has no phase-targeted gate matrix.
- Build and test gateway surface:
  `Makefile` currently includes `check-fmt`, `lint`, `test`, `markdownlint`,
  `nixie`, and `fmt`.
- Verification contracts:
  `docs/verification-targets.md` defines phase-gate expectations and failure
  escalation behaviour, but does not currently map those gates to executable CI
  commands.
- Existing contract suites:
  schema and authority lifecycle suites exist in `tests/compatibility/` and
  `tests/security/`.

Term definitions used in this plan:

- phase gate:
  the set of verification suites that must pass before entering a specified
  implementation phase.
- missing suite:
  a mandated suite with no executable CI mapping or no runnable test target.
- escalation behaviour:
  deterministic failure output that instructs maintainers to freeze affected
  merges, add or update regression tests, and restore green gates.

## Plan of work

Stage A: lock executable gate contracts and test fixtures.

- Add a machine-readable phase-gate contract file (for example under `ci/`)
  that maps each phase precondition from `docs/verification-targets.md` to
  concrete command identifiers.
- Add a local execution entrypoint through `Makefile` (for example
  `make phase-gate`) that runs the mapped checks for a selected phase.
- Add unit tests for gate contract parsing and missing-suite detection.
- Add behavioural tests (with `rstest-bdd` v0.5.0 where applicable) that cover:
  all required suites present and passing, missing required suite mapping, and
  failing required suite command.

Go/no-go for Stage A: test scaffolding runs and failing cases demonstrate
expected pre-implementation failures.

Stage B: wire merge-blocking CI jobs.

- Update `.github/workflows/ci.yml` to execute phase-gate checks as explicit,
  required CI steps (or a dedicated job) that fail the workflow on missing or
  failing suites.
- Ensure failure output includes escalation guidance aligned to
  `docs/verification-targets.md` failure policy.
- Keep existing baseline quality jobs intact.

Go/no-go for Stage B: workflow validation and local simulation show phase-gate
failures are merge-blocking and include escalation guidance.

Stage C: documentation and evidence alignment.

- Update `docs/verification-targets.md` with an executable gate-command mapping
  subsection so docs and CI contract remain synchronized.
- Record design decisions in `docs/zamburak-design-document.md` for how phase
  advancement and missing-suite failures are evaluated.
- Update `docs/users-guide.md` if this work changes any consumer-visible API or
  behaviour; otherwise keep the no-change statement explicit in documentation
  notes.

Go/no-go for Stage C: documentation clearly matches implemented CI behaviour
with no ambiguity.

Stage D: finalize and close roadmap task.

- Run mandatory code and docs quality gates with captured logs.
- Mark Task 0.2.2 done in `docs/roadmap.md`.
- Update this ExecPlan status and outcomes sections to reflect delivered
  evidence.

Go/no-go for Stage D: all required gates pass and roadmap state is updated.

## Concrete steps

Run commands from repository root: `/home/user/project`.

1. Baseline state and file discovery.

       git status --short
       rg --files .github docs tests crates

2. Add or update phase-gate contract and execution wiring.

       # edit ci/<phase-gate-contract-file>
       # edit Makefile to add phase-gate target(s)
       # edit .github/workflows/ci.yml for merge-blocking gate job/steps

3. Add tests for gate logic.

       # add unit tests for mapping/validation logic
       # add behavioural tests (rstest-bdd v0.5.0) where scenario text improves
       # observability for happy/unhappy/edge gate outcomes

4. Run targeted suites first.

       set -o pipefail
       cargo test --workspace --all-targets --all-features | tee /tmp/phase-gate-targeted.out

5. Run required quality gates.

       set -o pipefail
       make check-fmt | tee /tmp/check-fmt-zamburak-$(git branch --show-current).out
       set -o pipefail
       make lint | tee /tmp/lint-zamburak-$(git branch --show-current).out
       set -o pipefail
       make test | tee /tmp/test-zamburak-$(git branch --show-current).out

6. Run docs gates (Markdown changes are expected).

       set -o pipefail
       make markdownlint | tee /tmp/markdownlint-zamburak-$(git branch --show-current).out
       set -o pipefail
       make nixie | tee /tmp/nixie-zamburak-$(git branch --show-current).out
       set -o pipefail
       make fmt | tee /tmp/fmt-zamburak-$(git branch --show-current).out

7. Close task tracking.

       # mark docs/roadmap.md Task 0.2.2 as done
       # update this ExecPlan status/progress/outcomes sections

## Validation and acceptance

Acceptance is met when all conditions hold:

- CI includes explicit phase-gate execution that is merge-blocking.
- Phase-gate execution fails when a required suite is missing.
- Phase-gate execution fails when a required suite command fails.
- Failure output includes escalation guidance matching the documented policy.
- Unit tests pass for gate contract resolution and missing-suite handling.
- Behavioural tests pass for happy/unhappy/edge scenarios, using
  `rstest-bdd` v0.5.0 where it is applicable.
- Required gates pass:
  `make check-fmt`, `make lint`, `make test`, `make markdownlint`,
  `make nixie`, and `make fmt`.
- `docs/roadmap.md` marks Task 0.2.2 as done.

## Idempotence and recovery

- All plan steps are intended to be rerunnable without destructive side
  effects.
- If a gate command fails, fix only the reported cause, rerun that command,
  then rerun the full required gate sequence.
- If CI wiring causes unintended broad failures, revert only the new phase-gate
  job/step changes and reapply incrementally with tests first.

## Artifacts and notes

Capture and retain:

- gate logs under `/tmp/*-zamburak-<branch>.out`,
- CI run URL showing merge-blocking phase-gate behaviour,
- failing and passing transcripts for missing-suite and failing-suite paths,
- updated doc references linking verification targets to executable CI gates.

## Interfaces and dependencies

Expected implementation interfaces (names may be refined during delivery):

- `Makefile` target for phase-gate execution (for example `phase-gate`).
- `.github/workflows/ci.yml` job/steps that invoke the phase-gate target and
  fail the workflow on missing or failing required suites.
- A machine-readable phase-gate contract file under repository control (for
  example `ci/phase-gates.*`) used by local and CI execution.
- Test modules validating:
  gate-contract interpretation, missing-suite detection, failing-suite
  escalation messaging, and successful gate pass path.

Dependencies expected to remain unchanged:

- existing Rust workspace and test tooling,
- existing `rstest` and `rstest-bdd` v0.5.0 dev dependencies,
- existing Make-based gateway invocation pattern.

## Revision note

2026-02-18: Replaced this file's previous completed Task 0.1.3 execution record
with a new DRAFT ExecPlan for Task 0.2.2, because the user explicitly requested
planning for phase-gate CI wiring at this path. Remaining work is now focused
on implementing and validating the Task 0.2.2 delivery stages described above.
