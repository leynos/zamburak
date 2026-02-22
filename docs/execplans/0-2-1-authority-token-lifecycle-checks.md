# Align toolchain and quality-gate baseline with repository configuration (Task 0.2.1)

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: DRAFT

`PLANS.md` is not present in this repository at draft time, so this document is
the governing execution plan for this task.

## Purpose / big picture

Implement roadmap Task 0.2.1 from `docs/roadmap.md`: align repository baseline
configuration and baseline documentation so toolchain versioning and quality
commands are consistent and auditable across `rust-toolchain.toml`, `Makefile`,
and baseline documents.

After this change, contributors and CI should use one coherent baseline for
`make check-fmt`, `make lint`, and `make test`, with no conflicting command
contract across referenced docs. Drift should be detectable with tests that
fail closed when toolchain pins or gate-command expectations diverge.

Task completion is observable when baseline consistency tests pass for happy,
unhappy, and edge cases; required quality gates succeed; any consumer-visible
behaviour changes are reflected in `docs/users-guide.md`; and roadmap Task
0.2.1 is marked done.

## Constraints

- Implement to these signposts:
  `docs/tech-baseline.md` sections "Canonical version baseline" and "Baseline
  usage contract", `docs/zamburak-engineering-standards.md` section "Command
  and gateway standards", `docs/repository-layout.md` section "Root and
  operational files".
- Respect roadmap scope and boundaries:
  in scope: `rust-toolchain.toml`, `Makefile`, and baseline-document
  consistency; out of scope: introducing additional build systems.
- Keep baseline changes deterministic and fail closed. Ambiguous or conflicting
  command contracts must not be left unresolved.
- Add unit tests and behavioural tests that cover happy and unhappy paths, plus
  relevant edge cases for baseline drift detection.
- Use `rstest-bdd` v0.5.0 for behavioural tests where scenario-style baseline
  contracts are expressed.
- Record durable design decisions in `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` if this task changes library-consumer-visible
  behaviour or API; if no consumer-visible change exists, record that decision
  in this ExecPlan outcomes.
- Mark roadmap Task 0.2.1 done in `docs/roadmap.md` only after all gates pass.
- Required completion gates:
  `make check-fmt`, `make lint`, and `make test`.
- Because this task changes Markdown docs, docs gates are also required:
  `make markdownlint`, `make nixie`, and `make fmt`.

## Tolerances (exception triggers)

- Scope tolerance:
  if implementation requires edits in more than 12 files or exceeds 700 net
  changed lines, stop and escalate with a split proposal.
- Interface tolerance:
  if fulfilling this task requires changing public Rust API signatures, stop
  and escalate with compatibility options.
- Dependency tolerance:
  if a new third-party dependency is required, stop and escalate before adding
  it.
- Contract ambiguity tolerance:
  if command baselines in signpost docs cannot be reconciled into one canonical
  contract without altering requirements, stop and present options with
  trade-offs.
- Behavioural-test tolerance:
  if baseline drift scenarios cannot be represented with `rstest-bdd` after two
  concrete attempts, stop and document why before using non-BDD fallback tests.
- Iteration tolerance:
  if required gates still fail after three focused fix loops, stop and report
  failing suites with root-cause hypotheses.

## Risks

- Risk: baseline command definitions are currently split across `AGENTS.md`,
  `docs/tech-baseline.md`, and `Makefile` and may not be fully aligned.
  Severity: high Likelihood: high Mitigation: establish one canonical command
  expansion for each gate and then synchronize all task signpost docs and Make
  targets in the same change set.

- Risk: `make lint` currently includes steps beyond strict clippy gating, which
  may complicate reconciliation with documented "command and gateway" language.
  Severity: medium Likelihood: medium Mitigation: decide explicitly whether
  extra lint-adjacent checks belong in the canonical baseline and document
  rationale in the design document.

- Risk: baseline checks may regress in future if alignment is only documented
  and not test-enforced. Severity: high Likelihood: medium Mitigation: add
  automated unit and behavioural tests that fail when toolchain pin or
  gate-command contract drifts.

- Risk: user-guide churn may be introduced despite no consumer-visible API
  change. Severity: low Likelihood: medium Mitigation: apply a strict
  consumer-impact check; update `docs/users-guide.md` only if runtime API or
  user-observable library behaviour changed.

## Progress

- [x] (2026-02-22) Reviewed `docs/roadmap.md` Task 0.2.1 scope, signposts,
  and completion criteria.
- [x] (2026-02-22) Reviewed `rust-toolchain.toml`, `Makefile`,
  `docs/tech-baseline.md`, `docs/zamburak-engineering-standards.md`, and
  `docs/repository-layout.md` for baseline drift points.
- [x] (2026-02-22) Reviewed testing guidance docs requested for this task,
  including `docs/rstest-bdd-users-guide.md` and
  `docs/rust-testing-with-rstest-fixtures.md`.
- [x] (2026-02-22) Drafted this ExecPlan with staged implementation,
  test strategy, validation gates, and documentation update criteria.
- [ ] Implement baseline contract code and tests.
- [ ] Align `rust-toolchain.toml`, `Makefile`, and signpost docs.
- [ ] Update design documentation, users guide decision, and roadmap status.
- [ ] Run all required code and docs gates and capture logs.

## Surprises & discoveries

- Observation: no Qdrant project-memory MCP resources are available in this
  session. Evidence: `list_mcp_resources` and `list_mcp_resource_templates`
  returned empty lists. Impact: this plan is based on repository sources only.

- Observation: Task 0.2.1 remains unchecked in `docs/roadmap.md` while related
  tasks 0.2.2 and 0.2.3 are complete. Evidence: roadmap Step 0.2 status
  markers. Impact: this task must avoid regressing already-completed gate
  wiring and script baseline work.

- Observation: existing behavioural coverage patterns for roadmap contract tasks
  already use `rstest-bdd` under `tests/compatibility/` and `tests/security/`.
  Evidence: `tests/compatibility/phase_gate_bdd.rs` and
  `tests/security/authority_lifecycle_bdd`. Impact: Task 0.2.1 can follow
  established BDD harness conventions.

## Decision log

- Decision: implement baseline consistency as testable contract logic rather
  than doc-only alignment. Rationale: Task 0.2.1 completion criterion requires
  sustained consistency; a testable contract prevents silent drift.
  Date/Author: 2026-02-22 / Codex

- Decision: keep scope focused on baseline alignment artefacts and avoid
  introducing new build systems or orchestration layers. Rationale: roadmap
  scope explicitly excludes new build systems. Date/Author: 2026-02-22 / Codex

- Decision: treat `docs/users-guide.md` updates as conditional on
  consumer-visible API or behaviour changes, and document either outcome.
  Rationale: follows user instruction while avoiding non-functional churn.
  Date/Author: 2026-02-22 / Codex

## Outcomes & retrospective

Pending implementation.

Expected completion outcomes:

- baseline toolchain and gate-command contracts are synchronized across
  `rust-toolchain.toml`, `Makefile`, and baseline signpost docs,
- automated unit and behavioural tests verify happy and unhappy baseline cases,
- required code and docs quality gates pass,
- roadmap Task 0.2.1 is marked done,
- design decisions are recorded in `docs/zamburak-design-document.md`,
- `docs/users-guide.md` is updated only if consumer-visible behaviour or API
  changed.

## Context and orientation

Current state relevant to Task 0.2.1:

- `docs/roadmap.md` defines Task 0.2.1 as baseline alignment for
  `rust-toolchain.toml`, `Makefile`, and baseline-document consistency.
- `rust-toolchain.toml` currently pins `nightly-2026-01-30` with `rustfmt` and
  `clippy` components.
- `Makefile` currently defines quality gates (`check-fmt`, `lint`, `test`) and
  docs gates (`markdownlint`, `nixie`, `fmt`) used by contributors and CI.
- `docs/tech-baseline.md` defines canonical baseline tables and usage contract.
- `docs/zamburak-engineering-standards.md` defines command and gateway
  standards and tee-log conventions.
- `docs/repository-layout.md` defines root and operational files where baseline
  artefacts live.

Target state for Task 0.2.1:

- canonical baseline contract is explicit and conflict-free across signpost
  docs and configuration,
- baseline drift is prevented by automated tests,
- contributor and CI invocation paths stay predictable via `make` targets,
- roadmap traceability reflects completion without expanding beyond task scope.

## Plan of work

Stage A: lock the baseline contract and identify drift points (no file edits).

- Cross-map canonical baseline expectations from signpost docs to concrete
  configuration in `rust-toolchain.toml` and `Makefile`.
- Produce an explicit list of baseline assertions that must hold, including
  toolchain channel and exact quality-gate command contract.
- Decide how strict command matching should be (exact command strings vs
  required command invariants) and capture rationale in this ExecPlan and the
  design document update task.

Go/no-go for Stage A: all baseline assertions are concrete, testable, and
mapped to in-scope artefacts.

Stage B: add test-first baseline consistency coverage.

- Add a small baseline contract module in the root crate
  (`src/baseline_contract.rs`) with pure functions that evaluate:
  - toolchain pin alignment,
  - required Make targets present,
  - required gate command invariants,
  - baseline-doc contract consistency.
- Add unit tests in the module using `rstest` fixtures and parameterized cases
  for:
  - happy path where all baseline assertions align,
  - unhappy path where toolchain pin drifts,
  - unhappy path where a required `Makefile` gate target is missing,
  - edge cases around whitespace and command formatting noise.
- Add behavioural tests with `rstest-bdd` v0.5.0 in
  `tests/compatibility/baseline_alignment_bdd.rs` with scenarios in
  `tests/compatibility/features/baseline_alignment.feature` describing:
  - aligned baseline passes,
  - mismatched toolchain blocks,
  - gate-command drift blocks,
  - missing required gate target blocks.

Go/no-go for Stage B: new baseline tests fail before alignment edits and pass
only when baseline assertions are satisfied.

Stage C: align configuration and baseline documentation.

- Update `Makefile` and, if needed, `rust-toolchain.toml` to conform to the
  chosen canonical baseline contract from Stage A.
- Update `docs/tech-baseline.md` so "Canonical version baseline" and
  "Baseline usage contract" mirror actual configuration and `make` commands.
- Update `docs/zamburak-engineering-standards.md` command examples if command
  expansions changed.
- Update `docs/repository-layout.md` only if root-operational baseline file
  expectations changed.

Go/no-go for Stage C: baseline contract tests and docs reflect the same
commands and version pins with no unresolved conflicts.

Stage D: record decisions, consumer-impact check, and roadmap closure.

- Record baseline-alignment decisions in `docs/zamburak-design-document.md`
  with concise rationale and constraints.
- Evaluate whether this task changes library-consumer-visible API or behaviour.
  - If yes, update `docs/users-guide.md` accordingly.
  - If no, retain the file unchanged and note the decision in outcomes.
- Mark Task 0.2.1 as done in `docs/roadmap.md` once all validations are green.

Go/no-go for Stage D: decision record is updated, user-guide disposition is
explicit, and roadmap status is synchronized.

Stage E: full validation and evidence capture.

- Run required code and docs quality gates with tee logs.
- Confirm baseline contract unit and behavioural tests cover happy and unhappy
  paths plus edge cases.
- Verify final diff is limited to Task 0.2.1 scope.

Go/no-go for Stage E: all required gates pass and evidence logs are available.

## Concrete steps

Run all commands from repository root (`/home/user/project`).

1. Baseline discovery and drift inventory.

       rg -n "Task 0.2.1|Canonical version baseline|Baseline usage contract" \
         docs/roadmap.md docs/tech-baseline.md
       rg -n "Command and gateway standards|Root and operational files" \
         docs/zamburak-engineering-standards.md docs/repository-layout.md
       cat rust-toolchain.toml
       sed -n '1,260p' Makefile

2. Implement testable baseline contract and tests.

       # Edit src/baseline_contract.rs, src/lib.rs,
       # tests/compatibility/baseline_alignment_bdd.rs,
       # tests/compatibility/features/baseline_alignment.feature,
       # and tests/compatibility/main.rs as needed.
       cargo test --workspace baseline_contract -- --nocapture
       cargo test --workspace baseline_alignment -- --nocapture

3. Align configuration and docs to the chosen canonical contract.

       # Edit rust-toolchain.toml, Makefile, docs/tech-baseline.md,
       # docs/zamburak-engineering-standards.md, docs/repository-layout.md,
       # docs/zamburak-design-document.md, docs/users-guide.md (if needed),
       # and docs/roadmap.md.
       rg -n "nightly-|check-fmt|make lint|make test|Baseline usage contract" \
         rust-toolchain.toml Makefile docs/tech-baseline.md \
         docs/zamburak-engineering-standards.md docs/repository-layout.md \
         docs/roadmap.md

4. Run code quality gates with required tee logging.

       set -o pipefail; make check-fmt | tee /tmp/check-fmt-zamburak-$(git branch --show-current).out
       set -o pipefail; make lint | tee /tmp/lint-zamburak-$(git branch --show-current).out
       set -o pipefail; make test | tee /tmp/test-zamburak-$(git branch --show-current).out

5. Run docs quality gates because Markdown files are updated.

       set -o pipefail; make markdownlint | tee /tmp/markdownlint-zamburak-$(git branch --show-current).out
       set -o pipefail; make nixie | tee /tmp/nixie-zamburak-$(git branch --show-current).out
       set -o pipefail; make fmt | tee /tmp/fmt-zamburak-$(git branch --show-current).out

Expected success indicators:

- baseline contract unit tests pass,
- `rstest-bdd` behavioural baseline scenarios pass,
- baseline artefacts and signpost docs are consistent,
- `make check-fmt`, `make lint`, and `make test` exit zero,
- `make markdownlint`, `make nixie`, and `make fmt` exit zero,
- roadmap Task 0.2.1 is marked `[x]`.

## Validation and acceptance

Acceptance criteria for Task 0.2.1:

- Tests:
  unit tests validate baseline consistency logic; behavioural tests using
  `rstest-bdd` v0.5.0 validate aligned and drifted scenarios (happy and unhappy
  paths) plus edge cases.
- Lint and formatting:
  `make check-fmt`, `make lint`, and `make test` pass.
- Documentation:
  baseline signpost docs and config are synchronized; design decision entry is
  present; `docs/users-guide.md` is updated only if consumer-visible
  behaviour/API changed.
- Roadmap traceability:
  `docs/roadmap.md` marks Task 0.2.1 done only after all required gates pass.

Quality method:

- local execution of the required make targets with tee logs,
- behavioural verification via BDD scenario outcomes,
- final consistency check using `rg` over baseline artefacts and signpost docs.

## Idempotence and recovery

- Baseline contract tests and quality-gate commands are safe to rerun.
- If a gate fails, fix only the failing assertion or command drift and rerun the
  same gate until green.
- If `make fmt` produces broad markdown wrapping beyond Task 0.2.1 scope,
  preserve only scope-relevant edits and rerun docs gates.
- Avoid destructive git commands; inspect diffs and revert only intentional,
  out-of-scope changes if they were introduced during this task.

## Artifacts and notes

Expected evidence artefacts:

- `/tmp/check-fmt-zamburak-<branch>.out`
- `/tmp/lint-zamburak-<branch>.out`
- `/tmp/test-zamburak-<branch>.out`
- `/tmp/markdownlint-zamburak-<branch>.out`
- `/tmp/nixie-zamburak-<branch>.out`
- `/tmp/fmt-zamburak-<branch>.out`
- `git diff` covering baseline contract code/tests, baseline docs alignment,
  design decision entry, users-guide disposition, and roadmap status update.

## Interfaces and dependencies

Planned additions or edits (subject to implementation details):

- `src/baseline_contract.rs`:
  parse and evaluate baseline consistency across toolchain and gate commands.
- `src/lib.rs`:
  export baseline contract module if needed by integration tests.
- `tests/compatibility/baseline_alignment_bdd.rs` and
  `tests/compatibility/features/baseline_alignment.feature`: behavioural
  baseline drift scenarios via `rstest-bdd` v0.5.0.
- `tests/compatibility/main.rs`:
  register new BDD module.
- `rust-toolchain.toml`, `Makefile`, `docs/tech-baseline.md`,
  `docs/zamburak-engineering-standards.md`, `docs/repository-layout.md`,
  `docs/zamburak-design-document.md`, `docs/users-guide.md` (if needed), and
  `docs/roadmap.md` for baseline synchronization and traceability.

No new third-party dependencies are expected.

## Revision note

Initial draft created for roadmap Task 0.2.1 planning. This revision captures
current repository baseline observations, proposed implementation stages,
explicit tolerances, and validation gates. It establishes a test-first
consistency strategy and defines closure requirements including design-doc
recording, conditional users-guide updates, and roadmap completion marking.
