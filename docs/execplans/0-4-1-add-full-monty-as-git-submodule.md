# Add `full-monty` as a Git submodule and enforce fork guardrails (Task 0.4.1)

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & discoveries`, `Decision log`, and
`Outcomes & retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

`PLANS.md` is not present in this repository at draft time, so this document is
the governing execution plan for this task.

## Purpose / big picture

Implement roadmap Task 0.4.1 from `docs/roadmap.md`: add `full-monty` as a Git
submodule, define fork rules, and enforce review rejection of non-generic fork
changes.

After this change, the repository has a pinned `third_party/full-monty/`
checkout, an explicit fork policy document, and an automated review contract
that fails when proposed fork deltas introduce Zamburak-specific semantics into
Track A APIs. A maintainer can observe success by running the review checker on
allowed and disallowed patch samples, and by running the full quality gates.

This task is repository mechanics and governance only. It must not implement
hook-substrate internals.

## Constraints

- Implement to these requirement signposts:
  `docs/adr-001-monty-ifc-vm-hooks.md` sections "Decision" and "Process
  requirements to keep the fork PR-able", `docs/zamburak-design-document.md`
  section "Two-track execution model", and `docs/repository-layout.md` section
  "Root and operational files".
- Dependency constraint: Task 0.3.1 is complete and must remain green.
- In scope: submodule placement, fork-change category policy,
  prohibition of Zamburak semantics in fork APIs, and enforceable review
  rejection for non-generic fork deltas.
- Out of scope: implementation of runtime hook substrate internals in
  `full-monty`.
- `full-monty` submodule path is fixed at `third_party/full-monty/`.
- Review-policy enforcement must be deterministic and fail closed.
- Add unit tests and behavioural tests using `rstest-bdd` v0.5.0 where
  applicable, covering happy paths, unhappy paths, and relevant edge cases.
- Record design decisions taken for this task in
  `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` for any consumer-visible behaviour or workflow
  changes introduced by submodule usage and fork-boundary policy.
- Mark roadmap Task 0.4.1 as done in `docs/roadmap.md` only after all gates
  pass.
- Required completion gates: `make check-fmt`, `make lint`, and `make test`.
- Because this task updates Markdown docs, also run:
  `make markdownlint`, `make nixie`, and `make fmt`.
- If script files are added or changed, also run:
  `make script-baseline` and `make script-test`.

## Tolerances (exception triggers)

- Scope tolerance: if the implementation requires edits in more than 16 files
  or 1100 net changed lines, stop and escalate with a split proposal.
- Interface tolerance: if enforcing review policy requires breaking an existing
  public API, stop and escalate with compatibility options.
- Dependency tolerance: if a new third-party Rust or Python dependency is
  required, stop and escalate before adding it.
- CI tolerance: if enforcement requires branch-protection or repository settings
  changes outside tracked files, stop and escalate with a manual rollout plan.
- Iteration tolerance: if required gates still fail after three focused fix
  loops, stop and report failing suites plus root-cause hypotheses.
- Ambiguity tolerance: if "generic" versus "non-generic" semantics cannot be
  made precise enough for deterministic checks, stop and present concrete rule
  options and trade-offs.

## Risks

- Risk: submodule checkout drift in Continuous Integration (CI) if checkout
  does not initialize submodules. Severity: high Likelihood: medium Mitigation:
  update CI checkout configuration and include explicit submodule validation in
  acceptance steps.

- Risk: policy checker false positives for benign identifiers or comments.
  Severity: medium Likelihood: medium Mitigation: constrain checks to added
  API-surface lines and codify allowlist categories in tests.

- Risk: policy checker false negatives that miss non-generic semantics encoded
  indirectly. Severity: high Likelihood: medium Mitigation: combine lexical
  denylist checks with path and category checks, and require policy-document
  reviewer checklist as a second control.

- Risk: local `gh` command may be unavailable.
  Severity: low Likelihood: medium Mitigation: use
  `git clone https://github.com/leynos/full-monty.git` as a fallback while
  keeping the canonical repository reference unchanged.

## Progress

- [x] (2026-02-23 19:00Z) Reviewed requirements, signpost docs, and existing
  ExecPlan conventions; drafted this plan.
- [x] (2026-02-23 19:12Z) Added `full-monty` submodule at
  `third_party/full-monty/` and created `.gitmodules`.
- [x] (2026-02-23 19:27Z) Implemented review-policy contract module and
  `monty_fork_review` CLI enforcement path.
- [x] (2026-02-23 19:31Z) Added unit tests and `rstest-bdd` behavioural tests
  for happy, unhappy, and edge paths.
- [x] (2026-02-23 19:36Z) Updated design, repository-layout, users-guide,
  contents index, and added `docs/monty-fork-policy.md`.
- [x] (2026-02-23 19:45Z) Ran required quality gates:
  `make check-fmt`, `make lint`, `make test`, `make markdownlint`,
  `make nixie`, and `make fmt`.
- [x] (2026-02-23 19:46Z) Marked roadmap Task 0.4.1 done.

## Surprises & discoveries

- Observation: this repository currently has no `third_party/` directory,
  `.gitmodules` file, or existing submodule entries. Evidence: `ls third_party`
  fails and `git submodule status` returns no rows. Impact: this task needs
  first-time submodule bootstrap, not an update.

- Observation: Model Context Protocol (MCP) project-memory tools
  (`qdrant-find` / `qdrant-store`) are not available in this runtime session.
  Evidence: MCP resource listing returned no servers or templates. Impact: plan
  drafting used repository docs directly as source-of-truth.

- Observation: introducing the `full-monty` submodule caused Markdown format
  and lint sweeps to traverse vendored docs and fail repository gates.
  Evidence: `make fmt` and `make markdownlint` emitted errors from
  `third_party/full-monty/*.md`. Impact: added `.fdignore` and a markdownlint
  ignore rule for `**/third_party/**` to keep quality gates scoped to this
  repository's authored documentation.

## Decision log

- Decision: implement review rejection as code, not policy prose alone.
  Rationale: completion criteria requires rejection of non-generic fork
  changes; deterministic checks with tests provide auditable enforcement.
  Date/Author: 2026-02-23 / Codex.

- Decision: keep fork governance and semantic rules split across
  `docs/monty-fork-policy.md` and a Rust policy contract module. Rationale:
  documentation states human review policy; Rust contract enforces
  machine-checkable fail-closed criteria in CI. Date/Author: 2026-02-23 / Codex.

- Decision: treat submodule introduction as consumer-visible operational
  behaviour and update `docs/users-guide.md` accordingly. Rationale: consumers
  building from source need clone and sync guidance once submodules are
  required. Date/Author: 2026-02-23 / Codex.

- Decision: exclude `third_party/` from repository Markdown formatting and lint
  sweeps. Rationale: vendored upstream docs are outside local ownership and
  should not fail local style gates. Date/Author: 2026-02-23 / Codex.

## Outcomes & retrospective

Task 0.4.1 is complete.

Delivered outcomes:

- Added `third_party/full-monty/` as a pinned Git submodule and committed
  `.gitmodules`.
- Added `docs/monty-fork-policy.md` with allowed categories, forbidden
  semantics, and fail-closed review controls.
- Added `src/monty_fork_policy_contract.rs` plus unit tests for semantic-token
  rejection logic.
- Added `src/bin/monty_fork_review.rs` to evaluate patch text or submodule
  pointer deltas between superproject revisions.
- Added `rstest-bdd` behavioural coverage in
  `tests/compatibility/monty_fork_policy/mod.rs`.
- Wired CI to run review-policy checks and to initialize submodules.
- Updated design and consumer documentation, and marked roadmap Task 0.4.1 as
  done.

Retrospective:

- The enforcement split worked as intended: deterministic machine checks for
  forbidden semantics plus policy prose for category review.
- Submodule introduction required explicit gate scoping (`.fdignore` and
  markdownlint ignores) to prevent vendor documentation from breaking local
  formatting and lint gates.

## Context and orientation

Current repository state relevant to this task:

- `third_party/full-monty/` is now present as a pinned Git submodule and
  `.gitmodules` is tracked in the repository root.
- Roadmap Task 0.4.1 is now marked done in `docs/roadmap.md` under
  "Step 0.4: `full-monty` repository mechanics and guardrails".
- ADR-001 requires a constrained `full-monty` fork with generic Track A APIs
  and no Zamburak semantics in that API surface.
- `docs/repository-layout.md` now records `third_party/full-monty/` as an
  active root artefact and includes `docs/monty-fork-policy.md`.
- CI currently checks formatting, linting, script baseline/tests, full tests,
  and phase-gate suites via `.github/workflows/ci.yml`.
- Behavioural tests use `rstest-bdd` patterns under `tests/compatibility/` and
  `tests/security/`, with feature files in `tests/*/features/`.

Key files expected to change:

- `.gitmodules` (new): submodule declaration.
- `third_party/full-monty/` (new Git submodule path): pinned fork checkout.
- `docs/monty-fork-policy.md` (new): allowed categories and rejection policy.
- `docs/zamburak-design-document.md`: design decision updates for fork
  governance.
- `docs/repository-layout.md`: remove "planned" status and add `.gitmodules`
  in root operational files.
- `docs/users-guide.md`: submodule-aware consumer workflow note.
- `docs/contents.md`: add the new fork-policy document to docs index.
- `src/monty_fork_policy_contract.rs` (new): deterministic policy rules.
- `src/bin/monty_fork_review.rs` (new): CLI checker to enforce review policy.
- `src/lib.rs`: export the new policy contract module.
- `tests/compatibility/features/monty_fork_policy.feature` (new): behavioural
  scenarios.
- `tests/compatibility/monty_fork_policy/mod.rs` (new): step definitions.
- `tests/compatibility/main.rs`: include new behavioural module.
- `.github/workflows/ci.yml`: run checker in merge-blocking CI.
- `docs/roadmap.md`: mark Task 0.4.1 done after verification.

## Plan of work

Stage A: repository mechanics and submodule bootstrap.

- Add `full-monty` as a submodule at `third_party/full-monty/` and create
  `.gitmodules` with the canonical upstream URL.
- Validate local submodule initialization and pinning (`git submodule status`).
- Update CI checkout to ensure submodule metadata needed by policy checks is
  available during pull-request runs.

Go/no-go for Stage A: `git submodule status` shows `third_party/full-monty` and
CI checkout still succeeds.

Stage B: fork-policy specification and enforceable contract.

- Create `docs/monty-fork-policy.md` defining:
  allowed change categories, forbidden semantic classes, reviewer checklist,
  and "reject by default" policy for out-of-category deltas.
- Implement `src/monty_fork_policy_contract.rs` with pure validation functions
  for fork-change classification and non-generic semantic rejection.
- Implement `src/bin/monty_fork_review.rs` to evaluate submodule delta content
  and return non-zero on policy violations.
- Wire CI to run the review command so non-generic fork deltas fail the merge.

Go/no-go for Stage B: a synthetic generic delta passes and a synthetic
Zamburak-specific delta fails with clear violation output.

Stage C: verification suites (unit + behavioural).

- Add unit tests in `src/monty_fork_policy_contract.rs` covering:
  allowed categories (happy), forbidden semantics (unhappy), and edge cases
  (mixed diffs, empty diffs, comment-only additions, casing variants).
- Add `rstest-bdd` behavioural tests in
  `tests/compatibility/monty_fork_policy/mod.rs` with feature scenarios proving
  end-to-end policy decisions on representative diff inputs.
- Keep scenarios explicit about expected pass/fail outcomes.

Go/no-go for Stage C: new tests fail before implementation and pass after;
`make test` passes with the new suites included.

Stage D: documentation synchronization, roadmap closure, and full gates.

- Update `docs/zamburak-design-document.md` with design decisions taken for
  fork policy and Track A semantic boundary enforcement.
- Update `docs/repository-layout.md`, `docs/users-guide.md`, and
  `docs/contents.md` to reflect new operational reality.
- Mark Task 0.4.1 as done in `docs/roadmap.md`.
- Run all required quality/documentation/script gates and retain output logs.

Go/no-go for Stage D: all gates pass, docs are synchronized, and roadmap status
is updated.

## Concrete steps

Run from repository root (`/home/user/project`). Use `set -o pipefail` and
`tee` for all gate commands.

1. Bootstrap submodule and verify wiring.

    gh repo clone leynos/full-monty /tmp/full-monty
    FULL_MONTY_REPO_URL="$(gh repo view leynos/full-monty --json url --jq .url).git"
    git submodule add "$FULL_MONTY_REPO_URL" third_party/full-monty
    git submodule status

   Expected evidence includes one line like:

    `SHA` third_party/full-monty (heads/`branch-or-detached`)

   If `gh` is unavailable, set `FULL_MONTY_REPO_URL` to the clone URL for
   [`leynos/full-monty`](https://github.com/leynos/full-monty), then run:

    git clone "$FULL_MONTY_REPO_URL" /tmp/full-monty

2. Implement policy docs and Rust enforcement code.

    Edit docs/monty-fork-policy.md and Rust files listed in this plan.

3. Run focused tests for the new contract and behavioural suite.

    set -o pipefail
    RUSTFLAGS="-D warnings" cargo test --workspace --all-targets \
      --all-features monty_fork_policy \
      | tee /tmp/test-monty-fork-policy.out

   Expected evidence includes passing unit tests and behavioural scenarios for
   happy and unhappy paths.

4. Run repository quality gates.

    set -o pipefail
    make check-fmt | tee /tmp/check-fmt-zamburak-$(git branch --show-current).out
    set -o pipefail
    make lint | tee /tmp/lint-zamburak-$(git branch --show-current).out
    set -o pipefail
    make test | tee /tmp/test-zamburak-$(git branch --show-current).out

5. Run documentation and script gates when corresponding files change.

    set -o pipefail
    make markdownlint | tee /tmp/markdownlint-zamburak-$(git branch --show-current).out
    set -o pipefail
    make nixie | tee /tmp/nixie-zamburak-$(git branch --show-current).out
    set -o pipefail
    make fmt | tee /tmp/fmt-zamburak-$(git branch --show-current).out

    set -o pipefail
    make script-baseline | tee /tmp/script-baseline-zamburak-$(git branch --show-current).out
    set -o pipefail
    make script-test | tee /tmp/script-test-zamburak-$(git branch --show-current).out

6. Mark roadmap completion only after all gates are green.

    Edit docs/roadmap.md: change Task 0.4.1 checkbox from [ ] to [x]

## Validation and acceptance

Acceptance behaviours:

- `git submodule status` reports `third_party/full-monty`.
- Fork-policy document exists and defines allowed categories plus explicit
  non-generic rejection rules.
- Review checker exits success for generic deltas and non-zero for non-generic
  deltas, with deterministic violation messages.
- Unit and behavioural tests cover:
  at least one happy path, one unhappy path, and edge cases.
- `make check-fmt`, `make lint`, and `make test` all pass.
- Documentation gates pass for changed docs.
- Roadmap Task 0.4.1 is marked done.

Quality criteria:

- Tests: contract unit tests + `rstest-bdd` behavioural tests pass.
- Lint/typecheck: `make lint` passes with warnings denied.
- Formatting: `make check-fmt` passes before `make fmt`; no residual diffs.
- Security/process: CI includes enforced review-policy rejection for
  non-generic fork changes.

## Idempotence and recovery

- Submodule add is idempotent only when the path is absent. If partially added,
  cleanly retry by removing the staged gitlink and `.gitmodules` entry, then
  re-run `git submodule add`.
- Policy checker and tests are safe to rerun repeatedly.
- If CI checkout changes introduce unrelated failures, revert only the CI
  checkout delta and retry with a minimal submodule-aware configuration.
- Keep rollback simple: submodule pointer, `.gitmodules`, and policy-enforcement
  files can be reverted together as one atomic set.

## Artefacts and notes

Expected command transcript snippets at completion:

    $ git submodule status
    <sha> third_party/full-monty (…)

    $ cargo run --bin monty_fork_review -- --diff-file /tmp/generic.diff
    monty-fork-review: pass (0 violation(s))

    $ cargo run --bin monty_fork_review -- --diff-file /tmp/non_generic.diff
    monty-fork-review: fail (1 violation)
    - forbidden semantic token in added API line: "Zamburak…"

    $ make test
    …
    test result: ok.

## Interfaces and dependencies

Implement the following stable interfaces.

- In `src/monty_fork_policy_contract.rs`:

  - `pub enum MontyForkChangeCategory` with variants for allowed fork categories
    from ADR-001.
  - `pub struct MontyForkViolation` containing a machine-readable code and
    human-readable message.
  - `pub fn evaluate_added_lines(added_lines: &[&str]) -> Vec<MontyForkViolation>`
    for deterministic lexical enforcement.
  - `pub fn evaluate_patch_text(patch_text: &str) -> Vec<MontyForkViolation>`
    for convenience over raw diff text.

- In `src/bin/monty_fork_review.rs`:

  - CLI arguments to evaluate either a provided diff file or a git-derived
    submodule diff range.
  - Exit status contract: `0` when no violations; non-zero when violations or
    IO/parse errors occur.

- In behavioural tests:

  - Scenario names and step definitions must map explicit allowed and rejected
    delta examples to expected decision outcomes.

Dependencies:

- Do not add new third-party dependencies unless tolerance escalation approves
  it.
- Reuse existing workspace crates (`camino`, `cap_std`, and `rstest-bdd` test
  stack) where possible.

## Revision note

- Updated status from `DRAFT` to `COMPLETE`.
- Recorded implemented milestones and validation evidence.
- Added a discovery and decision about excluding `third_party/` from Markdown
  formatting and lint gates.
