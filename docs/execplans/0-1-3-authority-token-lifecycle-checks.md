# Implement authority token lifecycle conformance checks (Task 0.1.3)

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: DONE

`PLANS.md` is not present in this repository at draft time, so this document is
the governing execution plan for this task.

## Purpose / big picture

Implement roadmap Task 0.1.3 from `docs/roadmap.md`: authority token lifecycle
semantics must be enforced and verified for mint scope, delegation narrowing,
revocation, expiry, and snapshot-restore revalidation.

After this change, a library consumer should be able to observe deterministic,
fail-closed authority lifecycle behaviour: valid transitions are accepted,
invalid transitions are denied, and restored state is conservatively
revalidated against current revocation and expiry facts.

Task completion is observable when unit and behavioural lifecycle transition
suites pass for both valid and invalid transition paths, design decisions are
documented, user-facing API guidance is updated, and roadmap Task 0.1.3 is
marked done.

## Constraints

- Implement to these normative signposts:
  `docs/zamburak-design-document.md` section "Authority token lifecycle
  semantics", `docs/verification-targets.md` row "Authority lifecycle",
  `docs/zamburak-engineering-standards.md` section "Verification and
  endorsement standards", `docs/repository-layout.md` sections
  `crates/zamburak-core` and `tests/security/`.
- Scope is limited to authority lifecycle conformance checks:
  mint scope, delegation narrowing, revocation, expiry, and snapshot-restore
  validation.
- Out of scope: external identity-provider integration.
- Lifecycle checks must fail closed for invalid, stale, revoked, or
  non-revalidatable authority states.
- Keep authority as a separate concept from integrity and confidentiality,
  preserving the three-axis model from the design document.
- Add unit tests and behavioural tests covering happy and unhappy paths plus
  edge cases.
- Use `rstest-bdd` v0.5.0 for behaviour-driven development (BDD) suites where
  lifecycle scenarios are expressed as user-observable transitions.
- Record concrete lifecycle design decisions in
  `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` with any new authority lifecycle API or
  behaviour visible to library consumers.
- Mark roadmap Task 0.1.3 done in `docs/roadmap.md` when implementation and
  verification are complete.
- Required completion gates: `make check-fmt`, `make lint`, and `make test`.
- Because Markdown documentation is changed, run docs gates too:
  `make markdownlint`, `make nixie`, and `make fmt`.

## Tolerances (exception triggers)

- Scope tolerance:
  if implementation requires edits in more than 18 files or exceeds 1,200 net
  changed lines, stop and escalate with a split proposal.
- Interface tolerance:
  if existing stable policy-loading APIs must change signature, stop and
  escalate with compatibility options.
- Dependency tolerance:
  if implementation requires adding external dependencies beyond expected core
  modelling/test crates, stop and escalate before adding them.
- Semantics tolerance:
  if token scope-subset semantics cannot be derived unambiguously from current
  docs, stop and present candidate interpretations.
- Behavioural-test tolerance:
  if lifecycle behaviour cannot be represented with `rstest-bdd` after two
  concrete attempts, stop and document why before using a non-BDD fallback.
- Iteration tolerance:
  if required gates still fail after three focused fix loops, stop and report
  failing suites plus root-cause hypotheses.

## Risks

- Risk: `crates/zamburak-core` does not yet exist in this repository.
  Severity: high Likelihood: high Mitigation: add minimal crate scaffolding
  focused on authority lifecycle contracts only, then expand narrowly for task
  requirements.

- Risk: roadmap traceability names
  `crates/zamburak-core/src/authority.rs`, while repository-layout currently
  maps authority helpers to `capability.rs`. Severity: medium Likelihood:
  medium Mitigation: choose one canonical module path during implementation and
  update documentation consistently in the same change set.

- Risk: time-dependent expiry logic can create flaky tests.
  Severity: high Likelihood: medium Mitigation: use deterministic lifecycle
  evaluation inputs (explicit timestamp parameters or injected clock
  abstractions) instead of ambient wall-clock reads in tests.

- Risk: snapshot-restore semantics may be over-scoped without runtime snapshot
  infrastructure in place. Severity: medium Likelihood: medium Mitigation:
  model restore validation as explicit revalidation of token sets against
  current revocation/expiry state and verify that invalid tokens are stripped
  before downstream checks.

- Risk: delegation narrowing checks may accidentally allow broadened scope.
  Severity: high Likelihood: medium Mitigation: encode subset checks as
  explicit predicate functions and cover broadening attempts in negative tests.

## Progress

- [x] (2026-02-13 18:53Z) Reviewed roadmap, design, verification, standards,
  and repository-layout signposts for Task 0.1.3.
- [x] (2026-02-13 18:53Z) Inspected current code baseline and confirmed the
  repository presently contains `zamburak-policy` only.
- [x] (2026-02-13 18:53Z) Drafted this ExecPlan with lifecycle scope,
  implementation stages, and quality-gate requirements.
- [x] (2026-02-13) Implemented `zamburak-core` authority lifecycle domain model
  and validator (previous session).
- [x] (2026-02-13) Added 8 lifecycle transition unit tests (previous session).
- [x] (2026-02-13) Fixed `Display` impl for `AuthorityTokenId` (required by
  `thiserror` format strings).
- [x] (2026-02-13) Fixed `Makefile` test target to include `--workspace` flag
  so all crate members are tested.
- [x] (2026-02-13) Added 13 `rstest-bdd` behavioural lifecycle scenarios in
  `tests/security/features/authority_lifecycle.feature` with step definitions
  in `tests/security/authority_lifecycle_bdd.rs`.
- [x] (2026-02-13) Integrated lifecycle checks into `zamburak-policy` via
  `PolicyEngine::validate_authority_tokens` delegating to `zamburak-core`.
- [x] (2026-02-13) Updated `docs/zamburak-design-document.md` with lifecycle
  implementation decision block.
- [x] (2026-02-13) Updated `docs/users-guide.md` with authority lifecycle API
  section covering mint, delegation, revocation, boundary validation, restore,
  and error handling.
- [x] (2026-02-13) Updated `docs/repository-layout.md` with `authority.rs`
  entry and refined crate responsibility description.
- [x] (2026-02-13) Marked roadmap Task 0.1.3 as done.
- [x] (2026-02-13) All required quality gates pass (`make check-fmt`,
  `make lint`, `make test`).

## Surprises & discoveries

- Observation: there is currently no `crates/zamburak-core` crate.
  Evidence: workspace members list only `crates/zamburak-policy` and file tree
  has no `zamburak-core` directory. Impact: this task must include minimal
  workspace expansion before lifecycle implementation can proceed.

- Observation: `tests/security/` already exists and is wired as an integration
  suite root. Evidence: `tests/security/main.rs` currently includes
  migration-security coverage. Impact: lifecycle conformance scenarios can be
  colocated in `tests/security/` without introducing a new test harness shape.

- Observation: task traceability and repository-layout currently differ on the
  likely authority module filename (`authority.rs` vs `capability.rs`).
  Evidence: `docs/roadmap.md` table row `0.1.3` vs `docs/repository-layout.md`
  `crates/zamburak-core` mapping. Impact: implementation should normalize this
  path decision and update docs in the same change set to avoid follow-up
  ambiguity.

## Decision log

- Decision: model authority lifecycle with explicit domain types and transition
  validators in `zamburak-core`, not ad hoc checks inside policy parsing code.
  Rationale: lifecycle semantics are security-critical and need focused,
  testable primitives reusable across policy evaluation paths. Date/Author:
  2026-02-13 / Codex

- Decision: treat expiry and restore-time validation as deterministic checks
  using injected/evaluated time inputs rather than ambient system time.
  Rationale: deterministic checks support reliable tests and avoid flaky gate
  outcomes. Date/Author: 2026-02-13 / Codex

- Decision: include behavioural lifecycle transition scenarios under
  `tests/security/` using `rstest-bdd` v0.5.0 where scenario narration improves
  contract clarity. Rationale: verification targets explicitly require
  lifecycle transition fixtures and integration/security evidence. Date/Author:
  2026-02-13 / Codex

- Decision: use `BTreeSet<ScopeResource>` for `AuthorityScope` to guarantee
  deterministic ordering and O(log n) subset checks. `is_strict_subset_of`
  requires proper subset (not equal). Rationale: deterministic ordering avoids
  hash-iteration non-determinism in security checks; strict subset prevents
  lateral delegation (same scope, just relabelled). Date/Author: 2026-02-13

- Decision: `PolicyEngine::validate_authority_tokens` delegates to
  `zamburak-core::validate_tokens_at_policy_boundary` rather than duplicating
  logic. Rationale: single source of truth for lifecycle verdicts prevents
  divergence between engine and core validation paths. Date/Author: 2026-02-13

- Decision: delegation from revoked or expired parent checks run before scope
  and lifetime narrowing checks. Rationale: fail-closed ordering — a revoked or
  expired parent should be rejected as early as possible regardless of whether
  the delegation request is otherwise well-formed. Date/Author: 2026-02-13

- Decision: fixed Makefile `test` target to include `--workspace` flag.
  Rationale: without `--workspace`, `cargo test` only tests the root package,
  omitting `zamburak-core` and `zamburak-policy` unit tests from CI gating.
  Date/Author: 2026-02-13

- Decision: BDD step definitions avoid `expect()` in favour of `let...else`
  with `panic!()` to satisfy the workspace `clippy::expect_used` deny lint.
  Helper functions (`require_mint_result`, `require_delegation_result`,
  `require_boundary_result`) centralize option unwrapping. Rationale:
  consistency with existing compatibility BDD tests and workspace lint rules.
  Date/Author: 2026-02-13

## Outcomes & retrospective

All expected outcomes are met:

- Authority lifecycle transition checks implemented and fail-closed in
  `crates/zamburak-core/src/authority.rs`.
- 8 unit tests in `zamburak-core` covering valid/invalid mint, delegation
  scope narrowing, delegation lifetime narrowing, revocation, expiry, policy
  boundary validation, and restore revalidation.
- 13 BDD scenarios in `tests/security/features/authority_lifecycle.feature`
  covering mint (trusted/untrusted/invalid-lifetime), delegation (narrowed,
  widened, equal-scope, non-narrowed-lifetime, revoked-parent, expired-parent),
  boundary validation (revoked/expired stripping), and snapshot restore
  (conservative revalidation, all-expired stripping).
- `PolicyEngine::validate_authority_tokens` wires policy-engine authority
  checks to `zamburak-core` lifecycle validation.
- Design document updated with implementation decision block.
- User's guide updated with authority lifecycle API section.
- Repository layout updated with `authority.rs` entry.
- Roadmap Task 0.1.3 marked `[x]`.
- 46 total tests pass across workspace: 8 core + 17 policy + 4 compatibility +
  17 security.
- `make check-fmt`, `make lint`, `make test` all pass.

Retrospective notes:

- The previous session created `zamburak-core` but left a compilation error
  (`AuthorityTokenId` missing `Display` impl for `thiserror` format strings).
  This was caught immediately by running quality gates first.
- The Makefile `test` target was missing `--workspace`, causing `zamburak-core`
  unit tests to be silently excluded from `make test`. Fixed as part of this
  task.
- BDD step parameter capture includes literal quotes from Gherkin text;
  removing quotes from the feature file is cleaner than stripping in code.
- The `too_many_arguments` clippy lint fires on BDD step functions with many
  Gherkin parameters; a tightly-scoped `#[expect]` annotation is the correct
  response since the parameter count is driven by the scenario text.

## Context and orientation

Current repository state relevant to Task 0.1.3:

- Workspace currently contains `crates/zamburak-policy` only.
- `crates/zamburak-policy/src/engine.rs` currently handles policy loading but
  does not yet enforce authority-token lifecycle transitions.
- Existing behavioural tests cover policy schema and migration contracts in
  `tests/compatibility/`.
- Existing security tests cover migration fail-closed behaviour in
  `tests/security/`.
- Root dev dependencies already include `rstest-bdd = "0.5.0"` and
  `rstest-bdd-macros = "0.5.0"`.

Target state for Task 0.1.3:

- `crates/zamburak-core` exists with authority lifecycle module(s),
- lifecycle checks enforce mint, delegation, revocation, expiry, and
  restore-time revalidation semantics,
- `zamburak-policy` authority evaluation path uses lifecycle validation where
  traceability requires,
- lifecycle transition conformance suites exist in unit and security-level
  behavioural tests,
- design and user docs reflect shipped behaviour and API,
- roadmap Task 0.1.3 is marked done.

## Plan of work

Stage A: contract lock and scaffold authority-core surfaces.

- Add workspace member `crates/zamburak-core` with crate-level docs and a
  minimal public API for authority lifecycle modelling.
- Introduce authority lifecycle domain types and error enums in
  `crates/zamburak-core/src/authority.rs` (or selected canonical equivalent),
  including token identity, subject, capability, scope, expiry, delegation
  lineage, and revocation references.
- Add explicit transition predicate helpers for mint validity, delegation
  narrowing, revocation, expiry, and restore revalidation.
- Resolve and document canonical filename decision if `authority.rs` and
  `capability.rs` naming diverges.

Go/no-go for Stage A: crate compiles, public authority lifecycle interfaces are
well-scoped, and no behaviour is implied without tests.

Stage B: test-first conformance suites.

- Add unit tests under `crates/zamburak-core` using `rstest` fixtures and
  parameterized cases for: valid mint, invalid mint scope/expiry, valid
  narrowed delegation, invalid widened delegation, revocation invalidation,
  expiry invalidation, and restore-time token stripping.
- Add behavioural lifecycle scenarios using `rstest-bdd` v0.5.0 under
  `tests/security/` (feature files and step bindings) that exercise full
  transition narratives and attack-shaped unhappy paths.
- Ensure happy and unhappy paths plus edge cases are explicitly represented,
  including boundary-time expiry and parent-child lifetime equality limits.

Go/no-go for Stage B: new lifecycle tests fail for expected reasons before
implementation, while existing schema and migration suites remain stable.

Stage C: implement lifecycle validation and policy-engine integration.

- Implement lifecycle transition operations and validation outcomes in
  `zamburak-core` with fail-closed defaults.
- Add revocation-index and restore-revalidation helpers that evaluate token sets
  against current revocation and expiry state.
- Integrate lifecycle checks into the relevant authority path in
  `crates/zamburak-policy/src/engine.rs` so policy-facing logic consumes
  canonical lifecycle verdicts instead of local ad hoc checks.
- Export required types from `crates/zamburak-core/src/lib.rs` and root
  re-export surfaces as needed.

Go/no-go for Stage C: all lifecycle unit and behavioural/security tests pass,
and invalid transitions are denied deterministically.

Stage D: documentation and roadmap closure.

- Update `docs/zamburak-design-document.md` with concrete authority lifecycle
  design decisions taken during implementation (scope model, narrowing rules,
  revocation semantics, restore revalidation contract).
- Update `docs/users-guide.md` with authority lifecycle API usage and observable
  fail-closed outcomes.
- Update `docs/roadmap.md` by marking Task 0.1.3 as `[x]` once all completion
  criteria and gates are satisfied.
- If module-path decisions changed ownership mapping, update
  `docs/repository-layout.md` in the same change set.

Go/no-go for Stage D: docs and roadmap match shipped behaviour and do not
contradict code or tests.

Stage E: full validation and evidence capture.

- Run targeted lifecycle suites and then mandatory repository gates.
- Capture logs proving pass status for code and docs gates.

Go/no-go for Stage E: `make check-fmt`, `make lint`, `make test`,
`make markdownlint`, `make nixie`, and `make fmt` all pass.

## Concrete steps

Run commands from repository root: `/home/user/project`.

1. Baseline orientation.

       git status --short
       rg --files crates tests docs | sort

2. Scaffold and wire `zamburak-core`.

       mkdir -p crates/zamburak-core/src
       # Edit workspace Cargo manifests and add:
       # - crates/zamburak-core/Cargo.toml
       # - crates/zamburak-core/src/lib.rs
       # - crates/zamburak-core/src/authority.rs

3. Add lifecycle tests before implementation.

       mkdir -p tests/security/features
       # Edit/add:
       # - crates/zamburak-core/src/authority.rs (unit tests)
       # - tests/security/features/authority_lifecycle.feature
       # - tests/security/authority_lifecycle_bdd.rs
       # - tests/security/main.rs

4. Run targeted suites during implementation loops.

       set -o pipefail && cargo test -p zamburak-core authority | tee /tmp/test-0-1-3-authority-unit.out
       set -o pipefail && cargo test --test security authority_lifecycle | tee /tmp/test-0-1-3-authority-security.out

5. Implement lifecycle checks and integrate policy-engine usage.

       # Edit/add:
       # - crates/zamburak-core/src/authority.rs
       # - crates/zamburak-core/src/lib.rs
       # - crates/zamburak-policy/src/engine.rs
       # - crates/zamburak-policy/src/lib.rs
       # - src/lib.rs

6. Update design, guide, and roadmap artefacts.

       # Edit:
       # - docs/zamburak-design-document.md
       # - docs/users-guide.md
       # - docs/roadmap.md
       # - docs/repository-layout.md (if path mapping changed)

7. Run mandatory quality gates with logs.

       set -o pipefail && make check-fmt | tee /tmp/check-fmt-0-1-3.out
       set -o pipefail && make lint | tee /tmp/lint-0-1-3.out
       set -o pipefail && make test | tee /tmp/test-0-1-3.out

8. Run docs gates because Markdown is modified.

       set -o pipefail && make markdownlint | tee /tmp/markdownlint-0-1-3.out
       set -o pipefail && make nixie | tee /tmp/nixie-0-1-3.out
       set -o pipefail && make fmt | tee /tmp/fmt-0-1-3.out

## Validation and acceptance

Task 0.1.3 acceptance is satisfied only when all conditions below hold:

- Lifecycle behaviour:
  mint scope, delegation narrowing, revocation, expiry, and snapshot-restore
  revalidation transitions enforce valid paths and deny invalid paths.
- Unit tests:
  `zamburak-core` lifecycle tests cover happy and unhappy transitions plus edge
  cases.
- Behavioural/security tests:
  lifecycle transition scenarios (including attack-shaped invalid transitions)
  pass in `tests/security/` with `rstest-bdd` v0.5.0 where applicable.
- Policy integration:
  policy-engine authority checks consume lifecycle validation outcomes and
  remain fail-closed on invalid state.
- Documentation:
  design and user guide updates describe shipped lifecycle semantics and API.
- Roadmap state:
  Task 0.1.3 in `docs/roadmap.md` is marked `[x]`.
- Required gates:
  `make check-fmt`, `make lint`, and `make test` succeed.

## Idempotence and recovery

- All steps in this plan are re-runnable; edits should be additive and
  deterministic.
- If a targeted lifecycle suite fails, fix that failing transition class first,
  rerun targeted suites, then rerun full gates.
- If snapshot-restore validation semantics remain ambiguous during
  implementation, stop and record alternatives in `Decision Log` before
  continuing.
- Do not weaken lifecycle assertions to pass tests; fix implementation defects
  or clarify contract decisions in docs.

## Artefacts and notes

Keep these artefacts for review and traceability:

- final `git diff` and changed-file summary,
- targeted logs:
  `/tmp/test-0-1-3-authority-unit.out`,
  `/tmp/test-0-1-3-authority-security.out`,
- full-gate logs:
  `/tmp/check-fmt-0-1-3.out`, `/tmp/lint-0-1-3.out`, `/tmp/test-0-1-3.out`,
  `/tmp/markdownlint-0-1-3.out`, `/tmp/nixie-0-1-3.out`, `/tmp/fmt-0-1-3.out`,
- a criterion-to-evidence mapping from acceptance requirements to concrete test
  files and gate outputs.

## Interfaces and dependencies

Planned interface surface after Task 0.1.3:

- `zamburak-core` exposes authority lifecycle domain types and validators for:
  token minting, delegation checks, revocation checks, expiry checks, and
  restore revalidation.
- `zamburak-policy` consumes lifecycle verdicts from `zamburak-core` for
  authority-related policy checks instead of duplicating transition logic.
- Behavioural fixtures in `tests/security/` use the public API surface only,
  ensuring transition conformance is verified from consumer-observable paths.

Dependency posture:

- Prefer existing dependencies.
- Add new crates only when strictly required for lifecycle modelling or test
  determinism, and document rationale in the change set.

## Revision note

Initial draft created for roadmap Task 0.1.3 with explicit lifecycle scope,
unit and behavioural/security verification strategy, documentation obligations,
and completion gates.
