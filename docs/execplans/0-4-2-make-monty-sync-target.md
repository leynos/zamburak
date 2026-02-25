# Add `make monty-sync` with fork sync and verification gates (Task 0.4.2)

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: DRAFT

`PLANS.md` is not present in this repository at draft time, so this document is
the governing execution plan for this task.

## Purpose / big picture

Implement roadmap Task `0.4.2` from `docs/roadmap.md`: add one maintainer
command, `make monty-sync`, that synchronizes the `full-monty` fork with
upstream Monty and then runs repository verification gates.

After this change, maintainers can run a single target to perform the routine
sync workflow: initialize the submodule if needed, fetch upstream and fork
state, refresh the fork branch in the local submodule checkout, update the
superproject pointer, and execute defined verification suites. Success is
observable via deterministic command output and green quality gates.

## Constraints

- Implement to these normative signposts:
  `docs/adr-001-monty-ifc-vm-hooks.md` section "Implementation plan",
  `docs/zamburak-engineering-standards.md` section "Command and gateway
  standards", and `docs/tech-baseline.md` section "Baseline usage contract".
- Respect dependency ordering: Task `0.4.1` is complete and must remain green.
- In scope: upstream Monty sync workflow, local `full-monty` branch refresh,
  and post-sync verification command execution from a single `make` target.
- Out of scope: release automation outside repository-local tooling
  (for example, automated pushes, release tagging, or remote release jobs).
- `make monty-sync` must be deterministic, fail closed, and emit clear
  actionable errors for unhappy paths.
- Add unit tests and behavioural tests covering happy and unhappy paths and
  relevant edge cases. Use `rstest-bdd` `0.5.0` only where Rust behavioural
  tests are introduced; for Python script behaviour, use script-suite
  behavioural testing conventions already used in `scripts/tests/`.
- Record design decisions in `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` if consumer-visible behaviour or workflow changes
  are introduced; if no consumer-visible impact exists, document that
  determination in `Decision Log`.
- Mark Task `0.4.2` done in `docs/roadmap.md` only after all required gates are
  green.
- Required completion gates for this task:
  `make check-fmt`, `make lint`, and `make test`.
- Because this task introduces script changes, also run:
  `make script-baseline` and `make script-test`.
- Because this task updates Markdown documentation, also run:
  `make markdownlint`, `make nixie`, and `make fmt`.
- Run all gate commands using `set -o pipefail` and `tee` log capture per
  `AGENTS.md` and engineering-standards command logging conventions.

## Tolerances (exception triggers)

- Scope tolerance: if implementation exceeds 16 files or 1100 net changed
  lines, stop and escalate with a split proposal.
- Interface tolerance: if meeting requirements requires changing existing public
  Rust library APIs, stop and escalate with compatibility options.
- Dependency tolerance: if a new external dependency is required in Rust or
  Python, stop and escalate before adding it.
- Git-mechanics tolerance: if the target needs destructive history rewriting in
  either repository checkout (for example `reset --hard` on user work), stop
  and escalate.
- Workflow tolerance: if sync cannot be implemented repository-locally without
  requiring immediate remote pushes, stop and present rollout options.
- Iteration tolerance: if required gates remain failing after three focused
  fix loops, stop and escalate with failing logs and root-cause hypotheses.
- Ambiguity tolerance: if branch-refresh semantics are materially ambiguous
  (for example `merge --ff-only` versus `rebase` policy), stop and present
  options with trade-offs before finalizing behaviour.

## Risks

- Risk: `third_party/full-monty/` may be uninitialized in local clones.
  Severity: high. Likelihood: high. Mitigation: make sync command perform
  `git submodule update --init` before any submodule git operations and include
  tests for uninitialized state.

- Risk: local dirty state in superproject or submodule can cause partial sync
  side effects. Severity: high. Likelihood: medium. Mitigation: preflight
  cleanliness checks with fail-closed exits before mutating branch pointers.

- Risk: upstream or fork remotes may be missing or network fetch may fail.
  Severity: medium. Likelihood: medium. Mitigation: explicit remote validation,
  deterministic error messages, and non-zero exit on fetch failure.

- Risk: verification suites may be omitted or drift from baseline contract.
  Severity: high. Likelihood: medium. Mitigation: centralize post-sync
  verification command list in one script surface and test that every required
  gate command is invoked.

- Risk: automated sync semantics may conflict with future range-diff controls
  from Task `0.7.3`. Severity: medium. Likelihood: medium. Mitigation: keep
  sync output structured so range-diff integration can be added without
  breaking command interface.

## Progress

- [x] (2026-02-25 18:39Z) Reviewed roadmap Task `0.4.2`, ADR signposts,
  command/gateway baseline docs, and prior Task `0.4.1` artefacts.
- [x] (2026-02-25 18:39Z) Confirmed repository state and current `full-monty`
  mechanics (`.gitmodules` present, submodule currently uninitialized in this
  workspace, `monty_fork_review` exists).
- [x] (2026-02-25 18:39Z) Drafted this ExecPlan.
- [ ] Implement `make monty-sync` orchestration and safety checks.
- [ ] Add unit and behavioural test coverage for sync behaviour.
- [ ] Update design/user/roadmap documents and run required gates.

## Surprises & Discoveries

- Observation: `third_party/full-monty/` is currently present but uninitialized
  in this workspace (`git submodule status` line starts with `-`). Evidence:
  `git submodule status --recursive` returned
  `-b316ce4a... third_party/full-monty`. Impact: sync implementation must
  bootstrap submodule initialization instead of assuming a populated checkout.

- Observation: the fork remote configured in `.gitmodules` is
  `https://github.com/leynos/full-monty.git`, and public remote heads currently
  expose `main`. Evidence: `.gitmodules` and `git ls-remote --heads` checks.
  Impact: draft sync defaults should target fork `main` and upstream `main`
  unless explicitly overridden.

## Decision Log

- Decision: implement `make monty-sync` as a Makefile wrapper around a Python
  script under `scripts/`. Rationale: script-based orchestration is easier to
  unit test and behaviour test than inline Make shell, and aligns with existing
  script baseline enforcement. Date/Author: 2026-02-25 / Codex.

- Decision: keep sync repository-local and fail closed; do not auto-push to
  remote fork in this task. Rationale: roadmap scope excludes release
  automation outside repository-local tooling; remote publication can remain an
  explicit maintainer action. Date/Author: 2026-02-25 / Codex.

- Decision: treat `rstest-bdd` as not applicable unless Rust code paths are
  added for this task; use script behavioural tests for Python sync logic.
  Rationale: task behaviour is expected to live in repository scripts and
  Makefile orchestration. Date/Author: 2026-02-25 / Codex.

## Outcomes & Retrospective

Implementation has not started. This section will be completed when the task
reaches `Status: COMPLETE`, including delivered artefacts, gate outcomes,
remaining gaps (if any), and lessons learned.

## Context and orientation

Current repository context relevant to Task `0.4.2`:

- `.gitmodules` defines one submodule:
  `third_party/full-monty` at `https://github.com/leynos/full-monty.git`.
- `src/bin/monty_fork_review.rs` already enforces fork semantic policy checks
  over submodule deltas.
- CI currently runs `monty_fork_review` and baseline quality gates, but there is
  no developer-facing `make monty-sync` target yet.
- `Makefile` contains standard gates plus script gates (`script-baseline` and
  `script-test`) and is the canonical command surface.
- Existing script testing patterns are in `scripts/tests/`, with unit tests and
  behavioural scenarios (`pytest-bdd`) already established.
- Task `0.4.2` remains unchecked in `docs/roadmap.md` and must be marked done
  only after implementation and gate validation complete.

Planned primary touchpoints:

- `Makefile` for the new `monty-sync` target.
- `scripts/monty_sync.py` (new) for sync orchestration.
- `scripts/tests/` for unit and behavioural test coverage of sync workflow.
- `docs/zamburak-design-document.md` for durable design decisions.
- `docs/users-guide.md` if behaviour affects source consumers.
- `docs/roadmap.md` to mark Task `0.4.2` done at completion.
- Optional supporting docs (`docs/adr-001-monty-ifc-vm-hooks.md` or
  `docs/monty-fork-policy.md`) only if command contract text needs updates for
  consistency.

## Plan of work

Stage A: define executable sync contract and failure semantics.

- Specify the exact `monty-sync` contract: preflight checks, sync operations,
  verification commands, and exit behaviour.
- Define branch and remote defaults (`origin/main` fork branch and
  `upstream/main` source branch), with explicit override inputs only if needed.
- Define what constitutes a no-op sync versus a changed submodule pointer.
- Freeze error taxonomy for unhappy paths (dirty tree, fetch failure, missing
  remotes, gate failure, merge or rebase conflict).

Go/no-go for Stage A: command contract is unambiguous and test cases can be
written before implementation.

Stage B: scaffold tests first (red), then implement script and Make target
(green).

- Add unit tests for command-construction and guardrail logic in
  `scripts/tests/test_monty_sync_*.py`.
- Add behavioural tests in `scripts/tests/test_monty_sync_bdd.py` with feature
  scenarios under `scripts/tests/features/monty_sync.feature` for: happy path
  sync, uninitialized submodule bootstrap, dirty-state failure, fetch failure,
  and verification gate failure.
- Implement `scripts/monty_sync.py` with:
  - `uv` metadata block and Python baseline compliance,
  - explicit command execution through Cuprum helpers,
  - dependency-injected command runner seams so tests do not mutate real git
    state,
  - deterministic logging and non-zero exits for all failure modes.
- Add `monty-sync` target to `Makefile` that executes the script.

Go/no-go for Stage B: newly added tests fail before implementation and pass
after implementation; `make monty-sync --help` (or equivalent usage mode)
prints expected contract.

Stage C: wire verification suites and documentation updates.

- Ensure post-sync verification command list includes required code gates:
  `make check-fmt`, `make lint`, and `make test`.
- Ensure script- and docs-scope gates are run in this change set:
  `make script-baseline`, `make script-test`, `make markdownlint`,
  `make nixie`, and `make fmt`.
- Update `docs/zamburak-design-document.md` with the `monty-sync` contract and
  branch-refresh guardrails.
- Evaluate whether `docs/users-guide.md` needs updates. If source-consumer
  workflow changes are observable, document them; otherwise record explicit
  no-change rationale in `Decision Log`.
- Mark roadmap Task `0.4.2` done in `docs/roadmap.md` only after all gates pass.

Go/no-go for Stage C: documentation is consistent, roadmap state is updated,
all required gates are green with log evidence.

Stage D: hardening and handoff evidence.

- Validate idempotence by running `make monty-sync` twice in a row on unchanged
  remotes and confirming stable success behaviour.
- Validate failure recovery path by simulating a gate failure and verifying the
  command exits non-zero with clear rerun guidance.
- Capture concise transcripts proving sync and gate execution behaviour.

Go/no-go for Stage D: both idempotence and recovery checks pass and evidence is
recorded in this ExecPlan.

## Concrete steps

All commands run from repository root (`/home/user/project`) unless stated
otherwise.

1. Establish red tests for new sync behaviour.

```sh
set -o pipefail
uv run --with pytest --with pytest-bdd --with pytest-mock --with cmd-mox --with astroid \
  pytest scripts/tests/test_monty_sync_cli.py scripts/tests/test_monty_sync_bdd.py \
  | tee /tmp/test-monty-sync-zamburak-$(git branch --show-current).out
```

Expected pre-implementation transcript excerpt:

```plaintext
E   ModuleNotFoundError: No module named 'monty_sync'
FAILED scripts/tests/test_monty_sync_...
```

1. Implement script and Makefile target, then run targeted script suites.

```sh
set -o pipefail
make script-baseline | tee /tmp/script-baseline-zamburak-$(git branch --show-current).out
set -o pipefail
make script-test | tee /tmp/script-test-zamburak-$(git branch --show-current).out
```

Expected success excerpt:

```plaintext
script baseline checks passed
... passed in ...s
```

1. Run documentation gates for updated docs.

```sh
set -o pipefail
make markdownlint | tee /tmp/markdownlint-zamburak-$(git branch --show-current).out
set -o pipefail
make nixie | tee /tmp/nixie-zamburak-$(git branch --show-current).out
set -o pipefail
make fmt | tee /tmp/fmt-zamburak-$(git branch --show-current).out
```

1. Run required code gates.

```sh
set -o pipefail
make check-fmt | tee /tmp/check-fmt-zamburak-$(git branch --show-current).out
set -o pipefail
make lint | tee /tmp/lint-zamburak-$(git branch --show-current).out
set -o pipefail
make test | tee /tmp/test-zamburak-$(git branch --show-current).out
```

1. Smoke-test the new maintainer entrypoint.

```sh
set -o pipefail
make monty-sync | tee /tmp/monty-sync-zamburak-$(git branch --show-current).out
```

Expected success excerpt:

```plaintext
monty-sync: initialized submodule third_party/full-monty
monty-sync: fetched origin and upstream
monty-sync: refreshed fork branch
monty-sync: running verification gates
monty-sync: completed successfully
```

## Validation and acceptance

Acceptance is behaviour-based and must be demonstrable:

- `make monty-sync` performs sync and verification with one command.
- Happy path: with clean working tree and reachable remotes, the command exits
  `0` and runs defined verification suites.
- Unhappy path: command exits non-zero with clear diagnostics when:
  working tree is dirty, remotes cannot be fetched, branch refresh fails, or a
  verification suite fails.
- Edge path: if submodule is uninitialized, the command initializes it and
  continues without manual bootstrap.
- Tests:
  - unit tests validate sync command planning, preflight logic, and gate list,
  - behavioural tests validate end-to-end success and failure scenarios.
- Required gate outcomes for completion:
  `make check-fmt`, `make lint`, `make test`, `make script-baseline`,
  `make script-test`, `make markdownlint`, `make nixie`, and `make fmt` all
  pass.
- `docs/roadmap.md` Task `0.4.2` is marked `[x]` only after all validations are
  green.

## Idempotence and recovery

- `make monty-sync` must be safe to rerun. Re-running after a successful sync
  with no upstream delta should be a no-op plus verification rerun.
- If sync fails before submodule pointer update, rerun is safe after correcting
  the underlying issue (network, branch state, or remotes).
- If sync fails during verification after pointer update, fix failing gates and
  rerun `make monty-sync`; command should detect current state and proceed
  safely.
- Script must not issue destructive git commands against user changes.

## Artifacts and notes

During implementation, append concise evidence snippets here for:

- successful sync log excerpt,
- failing unhappy-path excerpt (for one representative failure),
- gate-pass summaries,
- final diff summary of touched files.

## Interfaces and dependencies

Planned script interface (subject to Stage A confirmation):

- `make monty-sync` as canonical entrypoint.
- `scripts/monty_sync.py` as implementation surface.
- Optional script flags for maintainers (if needed) should be additive and keep
  default behaviour stable.

Required existing dependencies and tools:

- Git CLI for submodule and remote operations,
- existing Python script baseline stack (`uv`, `cuprum`, and test tooling),
- existing repository Make targets for verification gates.

No new dependency should be introduced unless a tolerance escalation is
approved.

## Revision note

Initial draft created for roadmap Task `0.4.2`.

- What changed: added a complete DRAFT ExecPlan for `make monty-sync` delivery,
  including constraints, tolerances, staged work, concrete command steps,
  acceptance criteria, and recovery semantics.
- Why it changed: user requested an execplans-based implementation plan rooted
  in roadmap and signpost documents.
- Effect on remaining work: implementation can now proceed milestone-by-
  milestone after explicit user approval.
