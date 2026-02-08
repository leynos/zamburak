# Finalise Zamburak docs after Logisphere review and spec retirement

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

`PLANS.md` is not present in this repository at the time this plan was drafted,
so this document is the governing execution plan for this change set.

## Purpose / Big Picture

The current documentation set is close to implementation-ready, but the
Logisphere design review and the final-farewell recommendations identified
missing contracts and traceability that are still required before the original
technical specification can be removed without information loss.

After this plan is executed, a new contributor should be able to implement
Zamburak by reading only:

- `docs/zamburak-design-document.md`,
- `docs/zamburak-engineering-standards.md`,
- `docs/roadmap.md`,
- `docs/repository-layout.md`,
- `docs/tech-baseline.md`,
- `docs/verification-targets.md`.

Success is observable when those documents provide explicit answers, with
in-repo references, for all nine required outcomes:

- canonical policy schema v1 and migration semantics,
- explicit planner and quarantined large language model sink enforcement
  architecture with audit hooks,
- full authority token lifecycle semantics,
- workload assumptions and service-level objective/service-level indicator
  envelopes linked to benchmarks,
- naming glossary with token-name consistency fixes,
- design-level acceptance criteria and contract conformance gates,
- technology baseline and tool rationale,
- verification target matrix and evidence expectations,
- roadmap-to-artefact traceability.

## Constraints

- Keep `docs/zamburak-design-document.md` authoritative for semantics and
  security invariants.
- Keep `docs/roadmap.md` focused on implementation sequencing and measurable
  tasks.
- Keep `docs/zamburak-engineering-standards.md` focused on standards and
  process controls.
- Do not introduce Rust code or runtime-behaviour changes in this work.
- Preserve the documented security position: policy compliance with fail-closed
  enforcement, not universal non-interference claims.
- Maintain en-GB-oxendict spelling and repository Markdown conventions.
- Do not remove existing cross-links unless replaced with more precise links.
- Do not overwrite `docs/execplans/design-document.md`; use this new file as
  the active plan for this change.

If the objective cannot be met without violating a constraint, stop and
escalate with options.

## Tolerances (Exception Triggers)

- Scope: if execution needs edits to more than 10 documentation files, stop and
  escalate.
- Interface: if execution reveals required public Rust API changes, stop and
  escalate before changing code.
- Dependencies: if any new external tool or dependency is needed, stop and
  escalate.
- Iterations: if docs quality gates still fail after three fix cycles, stop and
  escalate with failure details.
- Time: if any milestone exceeds 2.5 hours, stop and escalate with a revised
  breakdown.
- Ambiguity: if multiple materially different schema compatibility models
  remain viable, stop and request a decision instead of assuming.

## Risks

    - Risk: Schema examples and schema contract drift apart across documents.
      Severity: high
      Likelihood: medium
      Mitigation: define one normative schema section and have all other docs
      reference it instead of re-specifying semantics.

    - Risk: LLM sink enforcement remains implied rather than explicitly placed
      in runtime architecture.
      Severity: high
      Likelihood: medium
      Mitigation: add an explicit architecture subsection with boundary owners,
      decision inputs, and audit outputs for each sink call path.

    - Risk: Authority lifecycle rules become incomplete for resume and restore
      semantics.
      Severity: high
      Likelihood: medium
      Mitigation: define mint, delegation, revocation, expiry, and snapshot
      validation semantics in one contiguous section.

    - Risk: SLO and SLI envelopes are stated without workload assumptions.
      Severity: medium
      Likelihood: medium
      Mitigation: pair each envelope with explicit workload assumptions and
      benchmark tie-in.

    - Risk: Naming inconsistency persists across docs and undermines trust.
      Severity: medium
      Likelihood: medium
      Mitigation: add glossary and run an explicit consistency pass, including
      `LlmRemotePromptCap`.

## Progress

    - [x] (2026-02-08 19:04Z) Confirmed branch name is `design-document`.
    - [x] (2026-02-08 19:04Z) Confirmed `PLANS.md` is absent.
    - [x] (2026-02-08 19:16Z) Confirmed `docs/execplans/design-document.md`
      already exists and selected a unique ExecPlan filename.
    - [x] (2026-02-08 19:18Z) Drafted canonical policy schema v1 and migration
      semantics updates.
    - [x] (2026-02-08 19:18Z) Drafted LLM sink enforcement architecture and
      audit-path updates.
    - [x] (2026-02-08 19:18Z) Drafted authority token lifecycle semantics
      updates.
    - [x] (2026-02-08 19:18Z) Drafted workload assumption and SLO/SLI envelope
      updates.
    - [x] (2026-02-08 19:18Z) Drafted glossary and naming consistency updates.
    - [x] (2026-02-08 19:18Z) Drafted design-level acceptance criteria updates.
    - [x] (2026-02-08 19:18Z) Added `docs/tech-baseline.md`.
    - [x] (2026-02-08 19:18Z) Added `docs/verification-targets.md`.
    - [x] (2026-02-08 19:18Z) Added roadmap-to-artefact traceability mapping
      in `docs/roadmap.md`.
    - [x] (2026-02-08 19:18Z) Cross-linked affected docs for discoverability
      and consistency.
    - [x] (2026-02-08 19:18Z) Ran Markdown quality gates and captured logs.
    - [x] (2026-02-08 19:18Z) Re-checked that original spec can be removed
      without information loss.

## Surprises & Discoveries

    - Observation: `docs/execplans/design-document.md` already exists.
      Evidence: directory listing shows that exact path with prior content.
      Impact: this plan uses a new filename to avoid clobbering an existing
      planning artefact.

## Decision Log

    - Decision: Use `docs/execplans/design-document-logisphere-farewell.md`
      as the active plan path.
      Rationale: avoids collision with existing `docs/execplans/design-document.md`
      while keeping discoverability under the same folder.
      Date/Author: 2026-02-08 / Codex

    - Decision: Keep roadmap traceability in `docs/roadmap.md` rather than a
      new file.
      Rationale: implementation sequencing and artefact outcomes should stay in
      one execution document.
      Date/Author: 2026-02-08 / Codex

    - Decision: Use standalone `docs/tech-baseline.md` and
      `docs/verification-targets.md`.
      Rationale: these references are cross-cutting and should remain concise,
      independently reviewable artefacts.
      Date/Author: 2026-02-08 / Codex

    - Decision: Standardize authority token naming to `LlmRemotePromptCap`.
      Rationale: removes the remaining mixed-case inconsistency and aligns with
      glossary naming rules.
      Date/Author: 2026-02-08 / Codex

    - Decision: Add a Phase 0 contract-freeze section to the roadmap.
      Rationale: design-level conformance requirements now gate phase-1 build
      start and must appear explicitly in implementation sequencing.
      Date/Author: 2026-02-08 / Codex

## Outcomes & Retrospective

Implementation completed for all planned documentation artefacts.

Delivered outcomes:

- all nine required outcomes are now explicitly documented across design,
  standards, roadmap, and the two new reference docs,
- quality gates passed for docs formatting, linting, and Mermaid validation,
- cross-linking now lets a new reader navigate from architecture to baseline
  and verification obligations without relying on the old specification.

Accepted deviations:

- none; plan scope was completed within constraints and tolerances.

Remaining gaps before deleting the original technical specification:

- no unresolved design-contract gaps were identified in this execution pass.

Lessons learned:

- a dedicated phase for design-contract freeze materially improves roadmap
  clarity before implementation starts,
- explicit glossary and traceability artefacts prevent terminology drift and
  make review faster.

## Context and Orientation

Current core docs and responsibilities:

- `docs/zamburak-design-document.md`: semantics, threat model, interfaces, and
  invariants.
- `docs/zamburak-engineering-standards.md`: security and engineering standards
  plus quality-gate commands.
- `docs/roadmap.md`: phase-step-task execution sequencing.
- `docs/repository-layout.md`: repository structure and file-purpose mapping.

The Logisphere review prioritised six additions:

- canonical policy schema v1 with compatibility and migration semantics,
- explicit LLM sink enforcement architecture with audit placement,
- authority token lifecycle semantics,
- workload assumptions with SLO/SLI capacity envelopes,
- naming glossary and token-name consistency,
- acceptance criteria requiring contract conformance tests before Phase 1
  build-out.

Final-farewell recommendations added three documentation artefacts:

- `docs/tech-baseline.md`,
- `docs/verification-targets.md`,
- roadmap-to-artefact traceability mapping in `docs/roadmap.md`.

## Plan of Work

Stage A updates `docs/zamburak-design-document.md` with missing normative
semantics and contracts:

- add canonical policy schema v1 section, versioning contract, and migration
  semantics,
- add explicit LLM sink enforcement architecture that states where planner and
  quarantined sink checks run and how each decision is audited,
- add authority token lifecycle contract covering mint scope, delegation,
  revocation, expiry, and snapshot restore validation,
- add workload assumptions plus SLO/SLI envelopes that tie benchmark budgets to
  realistic operating assumptions,
- add glossary entries and resolve naming inconsistency for
  `LlmRemotePromptCap`,
- add design-level acceptance criteria that gate Phase 1 on contract
  conformance tests.

Stage B creates supporting reference docs:

- `docs/tech-baseline.md` for versions, tools, and rationale,
- `docs/verification-targets.md` for verification matrix, expected evidence,
  and fail criteria.

Stage C updates execution guidance:

- add a roadmap-to-artefact traceability table in `docs/roadmap.md`,
- add or refine cross-links in standards and layout docs so readers can find
  the new references quickly.

Each stage ends with validation. Do not proceed while the current stage fails
its validation checks.

## Concrete Steps

Run all commands from repository root: `/data/leynos/Projects/zamburac`.

1. Implement Stage A in `docs/zamburak-design-document.md`:
   - insert new subsections in policy, LLM interaction, authority model,
     performance, and acceptance sections,
   - preserve existing threat model and fail-closed semantics while extending
     explicitness.
2. Implement Stage B by creating:
   - `docs/tech-baseline.md`,
   - `docs/verification-targets.md`.
3. Implement Stage C in `docs/roadmap.md` and relevant cross-links.
4. Run quality gates with log capture:
   - `make fmt | tee /tmp/fmt-zamburak-$(git branch --show-current).out`
   - `make markdownlint | tee /tmp/markdownlint-zamburak-$(git branch --show-current).out`
   - `make nixie | tee /tmp/nixie-zamburak-$(git branch --show-current).out`
5. Validate execution-readiness manually:
   - confirm each of the nine required outcomes is directly answerable from
     current docs without needing the original specification.

## Validation and Acceptance

Acceptance is met only when all criteria below pass:

- Documentation quality:
  - `make fmt` completes without unexpected destructive changes.
  - `make markdownlint` passes.
  - `make nixie` passes.
- Design completeness:
  - canonical policy schema v1 exists with explicit compatibility semantics.
  - LLM sink enforcement architecture explicitly places policy checks and audit
    logging for planner and quarantined paths.
  - authority token lifecycle semantics include snapshot restore validation.
  - workload assumptions and SLO/SLI envelopes are documented with benchmark
    linkage.
  - glossary resolves token-name inconsistency, including
    `LlmRemotePromptCap`.
  - design-level acceptance criteria require contract conformance tests before
    Phase 1 build-out.
- Farewell artefacts:
  - `docs/tech-baseline.md` exists and is cross-linked.
  - `docs/verification-targets.md` exists and is cross-linked.
  - roadmap-to-artefact traceability is present in `docs/roadmap.md`.

## Idempotence and Recovery

The documentation edits in this plan are idempotent. Re-running formatting and
linting commands should produce stable results after the first clean pass.

If a quality gate fails, fix only the reported issue, rerun the failed command,
then rerun the full documentation gate sequence. Do not proceed to commit until
all gates pass.

## Artifacts and Notes

Expected output evidence to collect during execution:

- `git diff -- docs/` showing only intended docs updates,
- gate logs under `/tmp/*-zamburak-<branch>.out`,
- a short checklist mapping each of the nine outcomes to the file and section
  that now answers it.

## Interfaces and Dependencies

No new runtime dependencies are introduced. This plan modifies documentation
interfaces only.

Documentation interfaces that must be explicit after execution:

- policy schema interface: versioning, compatibility, and migration semantics,
- LLM sink enforcement interface: policy decision inputs and audit output
  obligations for planner and quarantined sink calls,
- authority lifecycle interface: token state transitions and validity checks,
- verification interface: target matrix and evidence contract,
- roadmap interface: task-to-artefact mapping.

## Revision note

Initial draft created to supersede filename collision with
`docs/execplans/design-document.md` and to encode the Logisphere plus
final-farewell recommendations in one execution-ready plan.

Revision (2026-02-08): updated status to `COMPLETE`, checked all progress
items, and recorded implementation outcomes after documentation delivery and
quality-gate validation.
