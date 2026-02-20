# Wire phase-gate checks into CI from verification targets (Task 0.2.2)

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

`PLANS.md` is not present in this repository at draft time, so this document is
the governing execution plan for this task.

## Purpose / big picture

Implement roadmap Task 0.2.2 from `docs/roadmap.md`: wire merge-blocking phase
verification gates in repository continuous integration (CI) using the required
suites defined in `docs/verification-targets.md`.

After this change, phase advancement can be objectively blocked when required
verification suites are missing or failing, and CI output must explicitly
surface escalation actions from the failure policy.

Task completion is observable when CI gate jobs fail closed for missing or
failing mandated suites, pass when mandated suites are green, and roadmap Task
0.2.2 is marked done with supporting test evidence.

## Constraints

- Implement to these signposts:
  `docs/verification-targets.md` sections "Acceptance gates for implementation
  phases" and "Failure and escalation policy",
  `docs/zamburak-engineering-standards.md` section "Testing and verification
  evidence standards", `docs/repository-layout.md` section `.github/workflows/`.
- Respect dependency ordering: Task 0.1.1 and Task 0.1.3 are prerequisites and
  are already complete; do not regress their coverage paths.
- In scope: merge-blocking CI wiring for phase-gate checks and explicit
  gate-failure escalation behaviour in CI output.
- Out of scope: release-train orchestration outside repository CI.
- CI gate logic must fail closed if required suites are not configured,
  missing, or failing.
- Add unit tests and behaviour-driven development (BDD) behavioural tests
  (using `rstest-bdd` v0.5.0 where the gate contract is scenario-driven) for
  happy path, unhappy path, and edge cases.
- Record design decisions in `docs/zamburak-design-document.md` when introducing
  durable contract behaviour for phase-gate enforcement.
- Update `docs/users-guide.md` if any library-consumer-visible behaviour or API
  changes; if there is no such change, document that determination in this
  ExecPlan and avoid speculative user-guide churn.
- Mark roadmap Task 0.2.2 as done in `docs/roadmap.md` only after all quality
  and documentation gates pass.
- Required completion gates: `make check-fmt`, `make lint`, `make test`.
- Because this task updates Markdown documentation, also run:
  `make markdownlint`, `make nixie`, and `make fmt`.

## Tolerances (exception triggers)

- Scope tolerance:
  if implementation requires edits in more than 14 files or 900 net changed
  lines, stop and escalate with a split proposal.
- Interface tolerance:
  if fulfilling the task requires changing existing public library API
  signatures, stop and escalate with compatibility options.
- Dependency tolerance:
  if a new third-party dependency is required for gate wiring or tests, stop
  and escalate before adding it.
- Workflow tolerance:
  if branch-protection assumptions are unclear (for example required-check
  naming), stop and present concrete options and trade-offs.
- Behavioural-test tolerance:
  if `rstest-bdd` cannot express gate-behaviour scenarios after two concrete
  attempts, stop and document why before using a non-BDD fallback.
- Iteration tolerance:
  if required gates still fail after three focused fix loops, stop and report
  failing suites with root-cause hypotheses.

## Risks

- Risk: mandated pre-Phase-1 suites include subsystems not yet implemented,
  which could unintentionally block all routine merges instead of only blocking
  phase advancement. Severity: high Likelihood: medium Mitigation: enforce
  gates through an explicit phase-advancement check path that is merge-blocking
  when phase progression is attempted.

- Risk: CI wiring may silently drift from `docs/verification-targets.md`.
  Severity: high Likelihood: medium Mitigation: centralize gate mapping in one
  machine-checked location and test the mapping against expected phase suites.

- Risk: escalation policy may be encoded only in prose and not surfaced to
  operators during failure. Severity: medium Likelihood: medium Mitigation:
  emit deterministic CI failure summaries that include freeze, regression-test,
  and restore-green actions.

- Risk: over-coupling gate logic directly to workflow YAML can make behaviour
  hard to test. Severity: medium Likelihood: medium Mitigation: isolate
  gate-evaluation logic in testable Rust code, with workflow wiring as a thin
  invocation layer.

## Progress

- [x] (2026-02-18 22:56Z) Reviewed roadmap Task 0.2.2 requirements, scope, and
  traceability row.
- [x] (2026-02-18 22:57Z) Reviewed verification-target acceptance gates and
  failure escalation policy.
- [x] (2026-02-18 22:58Z) Reviewed current CI baseline in
  `.github/workflows/ci.yml` and current test layout.
- [x] (2026-02-18 23:00Z) Drafted this ExecPlan with constraints, tolerances,
  staged delivery, validation criteria, and evidence commands.
- [x] (2026-02-18 23:31Z) Implemented phase-gate contract evaluator in
  `src/phase_gate_contract.rs` and command in `src/bin/phase_gate.rs`.
- [x] (2026-02-18 23:34Z) Added `make phase-gate` target plus CI wiring in
  `.github/workflows/ci.yml`.
- [x] (2026-02-18 23:37Z) Added unit tests and `rstest-bdd` behavioural tests
  for pass, missing-suite block, failing-suite block, and invalid target parse.
- [x] (2026-02-18 23:40Z) Updated roadmap and supporting documentation for
  phase-gate wiring and target-file workflow.
- [x] (2026-02-18 23:48Z) Ran required code and documentation gates with log
  capture.

## Surprises & discoveries

- Observation: current CI runs formatting, linting, tests, and coverage, but it
  does not encode phase-based acceptance gate checks from
  `docs/verification-targets.md`. Evidence: `.github/workflows/ci.yml` contains
  one `build-test` job only. Impact: task implementation must add explicit
  phase-gate checks, not only rely on aggregate `make test`.

- Observation: no project-memory Model Context Protocol (MCP)/Qdrant server is
  available in this session. Evidence: `list_mcp_resources` and
  `list_mcp_resource_templates` returned no resources/templates. Impact: all
  planning context is derived from repository sources in this workspace.

- Observation: reusing `src/phase_gate_contract.rs` from compatibility tests
  via `#[path = ...]` initially triggered dead-code failures under
  `RUSTFLAGS="-D warnings"` for helper items used by the CLI only. Evidence:
  `cargo test -- --list` failed on unused `as_str`, constants, and suite lookup
  symbols. Impact: added a focused compatibility test asserting those symbols
  so strict warning gates stay green without suppression attributes.

## Decision log

- Decision: implement a testable phase-gate contract evaluator in Rust and keep
  workflow YAML as orchestration only. Rationale: this allows unit tests plus
  BDD scenarios for gate policy while keeping CI wiring deterministic and
  maintainable. Date/Author: 2026-02-18 / Codex.

- Decision: treat missing mandated suites as an immediate fail-closed result
  with explicit escalation instructions. Rationale: this directly satisfies the
  completion criterion that phase advancement must be blocked when suites are
  missing or failing. Date/Author: 2026-02-18 / Codex.

- Decision: include a dedicated check for phase-advancement intent rather than
  turning every PR into a global future-phase gate evaluation. Rationale: this
  preserves developer throughput while enforcing hard blocking when phase
  progression is proposed. Date/Author: 2026-02-18 / Codex.

- Decision: store the active phase-advancement target in
  `.github/phase-gate-target.txt` and run gating against that target in CI and
  local `make phase-gate`. Rationale: this gives an explicit, reviewable,
  fail-closed mechanism for advancement intent without external orchestration.
  Date/Author: 2026-02-18 / Codex.

## Outcomes & retrospective

Delivered outcomes:

- Added phase-gate contract mapping and evaluator:
  `src/phase_gate_contract.rs`.
- Added CI command entrypoint:
  `src/bin/phase_gate.rs`.
- Added command surface and target configuration:
  `make phase-gate` and `.github/phase-gate-target.txt`.
- Added merge-blocking CI phase-gate job in `.github/workflows/ci.yml`.
- Added unit tests and `rstest-bdd` behavioural suites for happy and unhappy
  gate paths: `src/phase_gate_contract.rs`,
  `tests/compatibility/phase_gate_bdd.rs`, and
  `tests/compatibility/features/phase_gate.feature`.
- Updated documentation and roadmap traceability:
  `docs/verification-targets.md`, `docs/zamburak-design-document.md`,
  `docs/tech-baseline.md`, `docs/zamburak-engineering-standards.md`,
  `docs/repository-layout.md`, and `docs/roadmap.md` (Task 0.2.2 marked done).
- No library-consumer API or runtime behaviour changed, so
  `docs/users-guide.md` required no update for this task.

Gate outcomes:

- passed: `make phase-gate`,
- passed: `make check-fmt`,
- passed: `make lint`,
- passed: `make test`,
- passed: `make markdownlint`,
- passed: `make nixie`,
- passed: `make fmt`.

Retrospective:

- A tracked phase-target file made advancement intent explicit and reviewable.
- Keeping gate semantics in Rust reduced workflow YAML complexity and improved
  testability of failure modes.

## Context and orientation

Current baseline relevant to this task:

- `.github/workflows/ci.yml` currently defines one `build-test` job that runs
  formatting, markdown lint, clippy lint, tests with coverage, and optional
  CodeScene upload.
- `docs/verification-targets.md` defines phase-specific gate expectations and a
  three-step failure escalation policy.
- `docs/roadmap.md` Task 0.2.2 expects phase-gate suites to be wired as
  merge-blocking checks and lists primary artefacts: `.github/workflows/`,
  `Makefile`, and `docs/verification-targets.md`.
- Existing behavioural suites already use `rstest-bdd` v0.5.0 in
  `tests/compatibility/` and `tests/security/`, providing a known pattern for
  scenario-driven contract tests.

Planned repository touchpoints:

- `.github/workflows/ci.yml` for CI gate wiring and merge-blocking job shape.
- `Makefile` for developer-facing phase-gate invocation targets used both
  locally and in CI.
- Rust module(s) in the root crate for phase-gate contract evaluation and
  deterministic failure summaries (final path chosen during Stage A).
- `tests/` integration coverage for unit and BDD tests of gate behaviour.
- `docs/zamburak-design-document.md` and `docs/roadmap.md` for decision and
  completion traceability.
- `docs/users-guide.md` only if consumer-visible behaviour/API changes.

## Plan of work

Stage A: define gate contract and execution surface (no workflow edits yet).

- Introduce a small phase-gate contract model in Rust that maps each
  implementation phase gate to required verification suites and blocking
  semantics.
- Add deterministic evaluation result types (pass/fail/missing) and escalation
  messages tied directly to the failure policy text.
- Expose the evaluator through one stable command surface (for example a
  `make` target backed by Rust code) so CI and local runs share logic.

Go/no-go for Stage A: the evaluator compiles, has no side effects, and produces
deterministic output for fixed inputs.

Stage B: test-first verification of gate logic.

- Add unit tests for phase-to-suite mapping, missing-suite detection,
  failing-suite detection, and non-blocking success outcomes.
- Add behavioural tests with `rstest-bdd` v0.5.0 (if applicable) describing:
  successful phase-gate pass, missing mandated suite, and mandated suite
  failure with escalation instructions.
- Include at least one edge case for ambiguous or unsupported phase input.

Go/no-go for Stage B: new tests fail before evaluator implementation is
complete and pass after implementation, while existing suites remain stable.

Stage C: wire CI and merge-blocking behaviour.

- Update `.github/workflows/ci.yml` to add explicit phase-gate check job(s)
  that call the shared gate command surface.
- Ensure job naming is stable for branch-protection configuration and that
  failures are hard-fail (non-allow-failure).
- Emit escalation guidance in CI logs/summary when gates fail, matching policy:
  freeze merges affecting subsystem, add/update regression test, restore gate
  green before continuing feature work.

Go/no-go for Stage C: CI configuration validates, gate job fails on injected
missing/failing suite conditions, and passes in expected green scenarios.

Stage D: documentation, roadmap closure, and full validation.

- Record durable design decisions in `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` only if this change introduces consumer-visible
  behaviour/API changes; otherwise record "no consumer API change" in this
  ExecPlan outcomes.
- Mark roadmap Task 0.2.2 as done in `docs/roadmap.md` after successful
  validation.
- Run all required code and docs gates with logs.

Go/no-go for Stage D: all required gates pass, documentation is synchronized,
and roadmap status is updated.

## Concrete steps

Run commands from repository root (`/home/user/project`).

1. Inspect and confirm baseline before edits.

       rg -n "Task 0.2.2|Acceptance gates for implementation phases|Failure and escalation policy" docs/roadmap.md docs/verification-targets.md
       sed -n '1,260p' .github/workflows/ci.yml

2. Implement phase-gate evaluator and tests.

       # edit Rust evaluator modules, Makefile targets, and tests
       cargo test --workspace phase_gate -- --nocapture

3. Validate CI workflow syntax and integration wiring.

       rg -n "phase-gate|verification gate|escalation" .github/workflows/ci.yml Makefile

4. Run required repository quality gates with log capture.

       set -o pipefail; make check-fmt | tee /tmp/check-fmt-zamburak-$(git branch --show-current).out
       set -o pipefail; make lint | tee /tmp/lint-zamburak-$(git branch --show-current).out
       set -o pipefail; make test | tee /tmp/test-zamburak-$(git branch --show-current).out

5. Run documentation quality gates because Markdown will change.

       set -o pipefail; make markdownlint | tee /tmp/markdownlint-zamburak-$(git branch --show-current).out
       set -o pipefail; make nixie | tee /tmp/nixie-zamburak-$(git branch --show-current).out
       set -o pipefail; make fmt | tee /tmp/fmt-zamburak-$(git branch --show-current).out

Expected indicators of success:

- Phase-gate tests report passing cases for success and intentional failure
  paths.
- CI workflow contains explicit phase-gate job names and no optional/failure
  bypass flags.
- `make check-fmt`, `make lint`, and `make test` exit zero.
- Markdown gates exit zero after documentation updates.

## Validation and acceptance

Acceptance criteria for Task 0.2.2:

- CI blocks phase advancement when mandated verification suites are missing.
- CI blocks phase advancement when mandated verification suites fail.
- CI failure output includes escalation behaviour consistent with
  `docs/verification-targets.md`.
- Unit tests cover gate mapping and evaluator outcomes (happy/unhappy/edge).
- Behavioural tests using `rstest-bdd` v0.5.0 cover scenario-level gate
  behaviour where applicable.
- Roadmap marks Task 0.2.2 as done only after all quality gates pass.

Required gates:

- Code gates: `make check-fmt`, `make lint`, `make test`.
- Docs gates: `make markdownlint`, `make nixie`, `make fmt`.

## Idempotence and recovery

- Phase-gate evaluation must be read-only and safe to rerun.
- CI wiring changes are idempotent when reapplied.
- If a gate fails, fix only the failing condition, rerun that gate, then rerun
  the full required gate sequence.
- If workflow wiring causes unexpected broad merge blocking, revert only the
  phase-gate wiring commit and keep test assets for diagnosis.

## Artefacts and notes

Evidence to capture during implementation:

- updated `.github/workflows/ci.yml` with phase-gate job(s),
- `Makefile` target(s) used by CI and local verification,
- unit test and BDD test files proving pass/fail/missing suite handling,
- gate logs in `/tmp/*-zamburak-<branch>.out`,
- criterion-to-evidence mapping in final outcomes section.

## Interfaces and dependencies

Prescriptive interface goals for this task:

- A stable command surface for gate execution, invokable from CI and local
  runs (through `make` target(s)).
- A Rust phase-gate evaluator API that returns structured outcomes rather than
  string parsing, so tests can assert deterministic semantics.
- No new external dependencies unless tolerance escalation is approved.
- Continued use of `rstest` and `rstest-bdd` v0.5.0 for tests.

Revision note: implementation outcomes, quality-gate evidence, and
documentation updates were recorded; status changed from `DRAFT` to `COMPLETE`.
