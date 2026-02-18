# Zamburak verification targets

This document defines the verification target matrix and evidence contract for
Zamburak.

System semantics and invariants are defined in
`docs/zamburak-design-document.md`.

Implementation sequencing is defined in `docs/roadmap.md`.

Engineering quality-gate process is defined in
`docs/zamburak-engineering-standards.md` and `AGENTS.md`.

This document uses:

- information flow control (IFC),
- large language model (LLM),
- continuous integration (CI).

## Verification target matrix

Each target below is normative. A target is complete only when its required
evidence exists and all listed gates pass.

| Subsystem                       | Verification target                                                  | Required tests                          | Evidence artefacts                              | Gate behaviour                                                  |
| ------------------------------- | -------------------------------------------------------------------- | --------------------------------------- | ----------------------------------------------- | --------------------------------------------------------------- |
| Policy schema loader            | Canonical schema v1 acceptance and unknown-version fail-close        | unit parser tests, compatibility tests  | `tests/compatibility/`, loader snapshots        | block merge on parse drift                                      |
| Policy engine                   | Deterministic decision order and fail-closed unknown state handling  | unit tests, property tests              | decision trace fixtures, rule-order checks      | block merge on decision nondeterminism                          |
| IFC propagation                 | Monotonic dependency propagation and no missing opcode rules         | property tests, opcode coverage tests   | coverage report, propagation fixtures           | block merge on uncovered opcode                                 |
| Control context                 | Strict-mode control-context inclusion in all effect checks           | unit tests, security regressions        | policy-check call traces                        | block merge on missing context input                            |
| LLM sink enforcement            | Pre-dispatch checks, redaction guard, and linked audit output        | integration tests, security regressions | sink-call audit chain fixtures                  | block merge on unenforced sink call                             |
| Authority lifecycle             | Mint, delegation, revocation, expiry, and restore revalidation       | unit tests, integration tests           | lifecycle transition fixtures                   | block merge on invalid lifecycle transition                     |
| Localization contract           | Explicit localizer injection, fallback ordering, and no global state | unit tests, integration tests           | fallback-order fixtures, localization traces    | block merge on fallback-order drift or ambient-state dependency |
| Tool catalogue and MCP boundary | Pinned schema/doc hashes and trust-class budget enforcement          | integration tests, security regressions | catalogue fixtures, trust-class outcomes        | block merge on unpinned or over-budget path                     |
| Audit pipeline                  | Confidentiality-first logging and tamper-evident chaining            | unit tests, integration tests           | redaction fixtures, hash-chain validator output | block merge on plaintext leak or chain break                    |
| Snapshot and resume             | Policy-equivalent behaviour pre- and post-restore                    | integration tests, metamorphic tests    | before/after decision transcripts               | block merge on policy divergence                                |

_Table 1: Verification targets, evidence artefacts, and gate behaviour._

## Evidence requirements by target class

Every verification target must include:

- at least one positive and one negative test case,
- deterministic fixtures or generated evidence reproducible in CI,
- a failure mode that is clearly mapped to `Deny`, `RequireConfirmation`, or
  explicit execution abort semantics where applicable.

Security-regression targets must additionally include:

- an attack-shaped input fixture,
- an expected blocked or mediated outcome,
- a link to prior bypass class where one exists.

## Acceptance gates for implementation phases

Phase-gate expectations:

- before Phase 1:
  schema, LLM sink enforcement, authority lifecycle, and localization contract
  conformance suites must pass,
- before Phase 2:
  container mutation, aliasing, and budget overflow fail-closed suites must
  pass,
- before Phase 3:
  tool catalogue pinning and Model Context Protocol (MCP) trust-class suites
  must pass,
- before Phase 4:
  LLM sink privacy-boundary integration suite must pass,
- before Phase 5:
  audit confidentiality and tamper-evidence suites must pass.
- before roadmap completion:
  localization fallback ordering and no-global-state conformance suites must
  pass.

### Repository CI wiring contract

Phase-gate checks are enforced in repository CI as follows:

- `.github/phase-gate-target.txt` declares the target phase currently permitted
  for advancement.
- `make phase-gate` evaluates required suites for that target by:
  - checking that mandated suites are present in the test catalog produced by
    `cargo test --workspace --all-targets --all-features -- --list`,
  - executing mandated suites by their configured test filters.
- `.github/workflows/ci.yml` runs `make phase-gate` in the merge-blocking
  `phase-gate` job.

If a mandated suite is missing or failing, the phase gate fails closed.

## Failure and escalation policy

A verification target failure is release-blocking when it affects:

- security invariant enforcement,
- policy decision determinism,
- fail-closed semantics,
- audit confidentiality constraints.

When a blocking failure appears:

1. freeze merges affecting the failing subsystem,
2. add or update a regression test reproducing the failure,
3. restore gate green status before continuing feature work.

CI phase-gate output must include these escalation actions on failure.

## Maintenance expectations

This matrix must be updated when:

- new security-critical subsystems are added,
- policy semantics or lifecycle contracts change,
- localization fallback contracts or locale-integration constraints change,
- new attack classes are discovered.

Updates to this file should occur in the same change set as related design or
roadmap updates to preserve traceability.
