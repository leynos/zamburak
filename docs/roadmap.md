# Zamburak roadmap

This roadmap defines the implementation sequence for Zamburak.

Authoritative semantics, interfaces, and trust boundaries are defined in
`docs/zamburak-design-document.md`.

Repository ownership and module path expectations are defined in
`docs/repository-layout.md`.

Security implementation standards and quality-gate process are defined in
`docs/zamburak-engineering-standards.md`, `docs/tech-baseline.md`, and
`AGENTS.md`.

Verification evidence requirements are defined in
`docs/verification-targets.md`.

Automation script standards are defined in `docs/scripting-standards.md`.

## Scope model

Roadmap items are expressed as:

- phases: strategic capability milestones,
- steps: coherent workstreams within a phase,
- tasks: measurable implementation units.

Each task includes requirement signposts, dependencies, scope boundaries, and
completion criteria so the work can be sequenced and assessed without ambiguity.

## Phase 0: Design-contract and delivery baseline

### Step 0.1: Canonical policy and authority contracts

- [x] Task 0.1.1: Freeze canonical policy schema v1 in runtime loading paths.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section "Canonical policy schema v1",
    - `docs/verification-targets.md` row "Policy schema loader",
    - `docs/zamburak-engineering-standards.md` section
      "Fail-closed standards",
    - `docs/repository-layout.md` sections `policies/` and
      `crates/zamburak-policy`.
  - Dependencies: none.
  - In scope: parser acceptance of schema version 1 and unknown-version reject
    behaviour.
  - Out of scope: future schema version authoring.
  - Completion criteria: loader accepts schema v1 only and rejects unknown
    schema versions fail-closed.
- [x] Task 0.1.2: Implement explicit schema migration transforms and migration
      conformance tests.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section
      "Schema compatibility and migration semantics",
    - `docs/verification-targets.md` row "Policy schema loader",
    - `docs/repository-layout.md` sections `crates/zamburak-policy` and
      `tests/compatibility/`.
  - Dependencies: Task 0.1.1.
  - In scope: version-to-version migration execution and migration audit
    evidence.
  - Out of scope: compatibility with unknown major schema families.
  - Completion criteria: migration tests prove restrictive-equivalent outcomes
    and produce auditable migration records.
- [x] Task 0.1.3: Implement authority token lifecycle conformance checks.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section
      "Authority token lifecycle semantics",
    - `docs/verification-targets.md` row "Authority lifecycle",
    - `docs/zamburak-engineering-standards.md` section
      "Verification and endorsement standards",
    - `docs/repository-layout.md` sections `crates/zamburak-core` and
      `tests/security/`.
  - Dependencies: none.
  - In scope: mint scope, delegation narrowing, revocation, expiry, and
    snapshot-restore validation.
  - Out of scope: external identity-provider integration.
  - Completion criteria: lifecycle transition suites pass for valid and invalid
    transitions.

### Step 0.2: Baseline tools, verification gates, and script standards

- [x] Task 0.2.1: Align toolchain and quality-gate baseline with repository
      configuration.
  - Requirement signposts:
    - `docs/tech-baseline.md` sections "Canonical version baseline" and
      "Baseline usage contract",
    - `docs/zamburak-engineering-standards.md` section
      "Command and gateway standards",
    - `docs/repository-layout.md` section "Root and operational files".
  - Dependencies: none.
  - In scope: `rust-toolchain.toml`, `Makefile`, and baseline-document
    consistency.
  - Out of scope: introducing additional build systems.
  - Completion criteria: baseline versions and gate commands are consistent
    across config and documentation.
- [x] Task 0.2.2: Wire phase-gate checks from verification targets into
      continuous integration (CI).
  - Requirement signposts:
    - `docs/verification-targets.md` sections
      "Acceptance gates for implementation phases" and
      "Failure and escalation policy",
    - `docs/zamburak-engineering-standards.md` section
      "Testing and verification evidence standards",
    - `docs/repository-layout.md` section `.github/workflows/`.
  - Dependencies: Tasks 0.1.1, 0.1.3.
  - In scope: merge-blocking verification gate wiring and gate-failure
    escalation behaviour.
  - Out of scope: release-train orchestration outside repository CI.
  - Completion criteria: CI blocks phase advancement when mandated verification
    suites are missing or failing.
- [x] Task 0.2.3: Establish automation script baseline for roadmap-delivered
      scripts.
  - Requirement signposts:
    - `docs/scripting-standards.md` sections "Language and runtime",
      "Testing expectations", and
      "CI wiring: GitHub Actions (Cyclopts-first)",
    - `docs/repository-layout.md` section "Root and operational files"
      (`scripts/`),
    - `docs/tech-baseline.md` section "Required engineering tools and
      rationale".
  - Dependencies: none.
  - In scope: script runtime metadata, command invocation patterns, and script
    test conventions.
  - Out of scope: replacing non-script Rust automation with Python.
  - Completion criteria: any new script introduced by roadmap tasks follows the
    scripting standard and has matching script tests.

### Step 0.3: Pre-phase contract conformance gate

- [x] Task 0.3.1: Enforce design-level conformance suites before Phase 1 build
      work.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section
      "Design-level acceptance criteria before phase 1 build-out",
    - `docs/verification-targets.md` rows "Policy schema loader",
      "LLM sink enforcement", and "Authority lifecycle",
    - `docs/zamburak-engineering-standards.md` section
      "Review and change-management standards".
  - Dependencies: Tasks 0.1.1, 0.1.3, 0.2.2.
  - In scope: schema, sink enforcement, and authority lifecycle contract test
    gating.
  - Out of scope: Phase 1 feature implementation.
  - Completion criteria: phase-1 implementation is blocked until all required
    conformance suites pass.

### Step 0.4: `full-monty` repository mechanics and guardrails

- [x] Task 0.4.1: Add `full-monty` as a Git submodule and define fork rules.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` sections "Decision" and
      "Process requirements to keep the fork PR-able",
    - `docs/zamburak-design-document.md` section
      "Two-track execution model",
    - `docs/repository-layout.md` section "Root and operational files".
  - Dependencies: Task 0.3.1.
  - In scope: submodule placement, documentation of allowed fork-change
    categories, and prohibition of Zamburak semantics in fork APIs.
  - Out of scope: implementation of hook substrate internals.
  - Completion criteria: submodule and fork-policy document exist, and review
    policy rejects non-generic fork changes.
- [x] Task 0.4.2: Add `make monty-sync` with fork sync and verification gates.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` section "Implementation plan",
    - `docs/zamburak-engineering-standards.md` section
      "Command and gateway standards",
    - `docs/tech-baseline.md` section "Baseline usage contract".
  - Dependencies: Task 0.4.1.
  - In scope: sync workflow for upstream Monty updates, `full-monty` branch
    refresh, and post-sync verification commands.
  - Out of scope: release automation outside repository-local tooling.
  - Completion criteria: maintainers can run one target that syncs fork state
    and executes defined verification suites.

### Step 0.5: Track A (`full-monty`) upstream-friendly substrate

- [x] Task 0.5.1: Implement stable runtime IDs in `full-monty`.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` section
      "A1. Stable, host-only runtime IDs",
    - `docs/zamburak-design-document.md` section
      "Snapshot and resume semantics",
    - `docs/verification-targets.md` row "IFC propagation".
  - Dependencies: Task 0.4.2.
  - In scope: unique, host-only IDs with continuity across `start()` or
    `resume()` and `dump()` or `load()`.
  - Out of scope: policy meanings encoded in runtime IDs.
  - Completion criteria: tests prove ID uniqueness and round-trip continuity
    across suspend, resume, dump, and load.
- [x] Task 0.5.2: Introduce generic runtime observer events in `full-monty`.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` section
      "A2. Lightweight event emission hooks",
    - `docs/zamburak-design-document.md` section
      "Two-track execution model",
    - `docs/verification-targets.md` row "IFC propagation".
  - Dependencies: Task 0.5.1.
  - In scope: canonical observer events `ValueCreated`, `OpResult`,
    `ExternalCallRequested`, `ExternalCallReturned`, and `ControlCondition`,
    matching the ADR Track A minimum event set.
  - Out of scope: policy decision types in observer payloads.
  - Completion criteria: observer events are emitted with no behavioural drift
    and no-op observers preserve baseline semantics.
- [ ] Task 0.5.3: Add generic snapshot extension bytes in `full-monty` when
      required.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` section
      "A3. Snapshot extension seam (optional but preferred)",
    - `docs/zamburak-design-document.md` sections
      "Two-track execution model" and "Snapshot and resume semantics",
    - `docs/verification-targets.md` row "Control context".
  - Dependencies: Task 0.5.2.
  - In scope: embedder-owned opaque bytes attached to snapshot persistence
    without Monty semantic interpretation.
  - Out of scope: Zamburak-specific encoding formats in `full-monty`.
  - Completion criteria: snapshot extension round-trips are stable and API
    contracts remain generic.
- [ ] Task 0.5.4: Enforce Track A compatibility and performance invariants.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` section
      "A4. Compatibility and upstreamability invariants",
    - `docs/zamburak-engineering-standards.md` section
      "Testing and verification evidence standards",
    - `docs/verification-targets.md` row "IFC propagation".
  - Dependencies: Task 0.5.3.
  - In scope: differential behaviour checks versus baseline Monty and
    measurement of hook-disabled or no-op overhead.
  - Out of scope: policy-layer benchmark targets outside Track A.
  - Completion criteria: compatibility suite and performance checks pass for
    both hook-disabled and no-op-observer modes.

### Step 0.6: Track B (Zamburak governance) integration workstream

- [ ] Task 0.6.1: Add `crates/zamburak-monty` adapter crate for governed
      execution.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` section "Track B staged pull
      requests",
    - `docs/zamburak-design-document.md` sections "Architecture overview" and
      "Policy evaluation semantics",
    - `docs/repository-layout.md` section `crates/zamburak-monty`.
  - Dependencies: Tasks 0.5.2 and 0.1.1.
  - In scope: `full-monty` integration, observer installation, and one governed
    run entrypoint.
  - Out of scope: full IFC propagation semantics.
  - Completion criteria: governed execution path uses `full-monty` adapter with
    deterministic external-call mediation hooks.
- [ ] Task 0.6.2: Add IFC core crate with `ValueId`-keyed dependency graph.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` section "B1. IFC substrate",
    - `docs/zamburak-design-document.md` sections
      "Dependency representation" and "Strict-mode effect semantics",
    - `docs/verification-targets.md` row "IFC propagation".
  - Dependencies: Task 0.6.1.
  - In scope: dependency DAG, summary joins, and normal or strict propagation
    mode handling.
  - Out of scope: direct coupling to Monty internal value types.
  - Completion criteria: IFC core unit and property tests validate dependency
    propagation invariants independently of interpreter internals.
- [ ] Task 0.6.3: Wire `full-monty` observer events into IFC updates.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` section
      "Track B staged pull requests",
    - `docs/zamburak-design-document.md` sections
      "Component responsibilities" and "Strict-mode effect semantics",
    - `docs/verification-targets.md` rows "IFC propagation" and
      "Control context".
  - Dependencies: Tasks 0.6.2 and 0.5.3.
  - In scope: event-to-IFC graph updates and strict-mode control dependency
    tracking.
  - Out of scope: policy decision presentation UX.
  - Completion criteria: integration tests prove observer-driven IFC state is
    complete for supported event classes.
- [ ] Task 0.6.4: Gate external calls through policy decisions at runtime.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` section
      "B2. Boundary enforcement at external calls",
    - `docs/zamburak-design-document.md` section
      "Policy evaluation semantics",
    - `docs/verification-targets.md` rows "Policy engine" and
      "LLM sink enforcement".
  - Dependencies: Tasks 0.6.3 and 0.1.2.
  - In scope: allow, deny, and confirmation policy result wiring for
    external-function boundaries.
  - Out of scope: tool-specific UI interaction design.
  - Completion criteria: every external call path requests a policy decision
    before side-effect execution.
- [ ] Task 0.6.5: Add compatibility, security, and snapshot-governance suites.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` sections
      "B3. Durable and versioned IFC state" and "B4. Auditable decisions",
    - `docs/zamburak-design-document.md` sections
      "Mechanistic correctness requirements" and "Security regression suite",
    - `docs/verification-targets.md` rows "IFC propagation",
      "Control context", and "Audit pipeline".
  - Dependencies: Task 0.6.4.
  - In scope: permissive-policy parity, strict-mode security regressions, and
    snapshot or resume governance continuity checks.
  - Out of scope: model-in-loop benchmark expansion.
  - Completion criteria: suites pass and prove policy-equivalent outcomes for
    uninterrupted and snapshot-restored execution.

### Step 0.7: `full-monty` PR-ability process controls

- [ ] Task 0.7.1: Enforce fork patch-budget and naming constraints in review.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` section "Patch budget",
    - `docs/zamburak-design-document.md` section
      "Two-track execution model",
    - `docs/zamburak-engineering-standards.md` section
      "Review and change-management standards".
  - Dependencies: Task 0.4.1.
  - In scope: reviewer checklist that rejects policy or Zamburak semantics in
    Track A changes.
  - Out of scope: automatic semantic classification tooling.
  - Completion criteria: all Track A pull requests carry explicit patch-budget
    classification and pass review checks.
- [ ] Task 0.7.2: Require upstream-shaped commits for `full-monty` deltas.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` section "Upstream-shaped commits",
    - `docs/zamburak-engineering-standards.md` section "Change quality and
      committing",
    - `docs/verification-targets.md` section
      "Evidence requirements by target class".
  - Dependencies: Task 0.7.1.
  - In scope: small, test-complete commits with upstream-value rationale and
    mapped upstream PR references.
  - Out of scope: batching unrelated maintenance changes with Track A work.
  - Completion criteria: each `full-monty` change is traceable to an upstream
    PR candidate and passes quality gates in isolation.
- [ ] Task 0.7.3: Add continuous `git range-diff` drift checks for fork delta
      growth.
  - Requirement signposts:
    - `docs/adr-001-monty-ifc-vm-hooks.md` section
      "Continuous range-diff control",
    - `docs/zamburak-engineering-standards.md` section
      "Review and change-management standards",
    - `docs/tech-baseline.md` section "Baseline usage contract".
  - Dependencies: Task 0.7.2.
  - In scope: repeatable drift check command and documented escalation on fork
    delta growth.
  - Out of scope: automatic merge conflict resolution.
  - Completion criteria: sync workflow records range-diff output and blocks
    progress when unexplained delta growth is detected.

## Phase 1: Core trust semantics and policy inputs

### Step 1.1: Information-flow model separation

- [ ] Task 1.1.1: Implement separate runtime representations for integrity,
      confidentiality, and authority.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section
      "Three-axis label and authority model",
    - `docs/zamburak-engineering-standards.md` section
      "Information-flow correctness standards",
    - `docs/verification-targets.md` rows "IFC propagation" and
      "Policy engine",
    - `docs/repository-layout.md` section `crates/zamburak-core`.
  - Dependencies: Tasks 0.3.1 and 0.6.5.
  - In scope: data model separation, joins, and policy-input compatibility.
  - Out of scope: authority declassification shortcuts.
  - Completion criteria: no single type conflates data labels with authority
    tokens, and policy checks consume separated inputs.
- [ ] Task 1.1.2: Implement deterministic verification, endorsement, and
      declassification controls.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section
      "Verification, endorsement, and declassification",
    - `docs/zamburak-engineering-standards.md` section
      "Verification and endorsement standards",
    - `docs/verification-targets.md` row "Policy engine",
    - `docs/repository-layout.md` section `crates/zamburak-sanitizers`.
  - Dependencies: Task 1.1.1.
  - In scope: verification-kind contracts, deterministic verifier behaviour,
    and endorsement/declassification policy hooks.
  - Out of scope: probabilistic verifier acceptance.
  - Completion criteria: every verification kind has positive and negative tests
    and cannot be forged through non-verifier paths.

### Step 1.2: Strict-mode effect context propagation

- [ ] Task 1.2.1: Include execution-context summaries in every effect policy
      check.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` sections "Strict-mode effect
      semantics" and "Policy evaluation semantics",
    - `docs/verification-targets.md` row "Control context",
    - `docs/zamburak-engineering-standards.md` section
      "Information-flow correctness standards",
    - `docs/repository-layout.md` sections `crates/zamburak-interpreter` and
      `crates/zamburak-policy`.
  - Dependencies: Task 1.1.1.
  - In scope: policy-check inputs for argument summaries plus control-context
    summaries.
  - Out of scope: static whole-program control-flow restrictions.
  - Completion criteria: effectful call sites pass context summaries and gate
    behaviour changes when context is untrusted.
- [ ] Task 1.2.2: Add strict-mode side-channel regression tests for call
      occurrence and call count.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section "Security regression suite",
    - `docs/verification-targets.md` row "Control context",
    - `docs/zamburak-engineering-standards.md` section
      "Required test categories",
    - `docs/repository-layout.md` section `tests/security/`.
  - Dependencies: Task 1.2.1.
  - In scope: regression fixtures for occurrence and count exfiltration.
  - Out of scope: mitigation of non-effect timing channels outside policy scope.
  - Completion criteria: security regressions fail before enforcement and pass
    once strict-mode checks are active.

## Phase 2: Mutable-state soundness and bounded provenance

### Step 2.1: Container mutation and opcode completeness

- [ ] Task 2.1.1: Implement versioned container-state propagation with alias
      safety.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section
      "Container and mutation semantics",
    - `docs/verification-targets.md` row "IFC propagation",
    - `docs/zamburak-engineering-standards.md` section
      "Required test categories",
    - `docs/repository-layout.md` sections `crates/zamburak-core` and
      `tests/property/`.
  - Dependencies: Tasks 1.1.1, 1.2.1.
  - In scope: versioned container writes and alias-consistent reads.
  - Out of scope: mutable object models outside supported Monty subset.
  - Completion criteria: container mutation tests show acyclic provenance and
    stable alias behaviour.
- [ ] Task 2.1.2: Complete opcode and built-in information-flow coverage.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` sections "Supported language subset"
      and "Mechanistic correctness requirements",
    - `docs/verification-targets.md` row "IFC propagation",
    - `docs/zamburak-engineering-standards.md` section
      "Fail-closed standards",
    - `docs/repository-layout.md` section `crates/zamburak-interpreter`.
  - Dependencies: Task 2.1.1.
  - In scope: propagation rules for every supported opcode and built-in.
  - Out of scope: unsupported Monty feature classes listed as out of scope in
    the design.
  - Completion criteria: opcode coverage tests fail on missing propagation and
    pass only with complete coverage.

### Step 2.2: Summary budgets and explainability

- [ ] Task 2.2.1: Implement dependency-summary fast path with bounded witness
      capture.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section "Dependency representation",
    - `docs/verification-targets.md` rows "Policy engine" and
      "Control context",
    - `docs/repository-layout.md` sections `crates/zamburak-core` and
      `crates/zamburak-policy`.
  - Dependencies: Task 2.1.2.
  - In scope: summary joins, witness truncation limits, and cacheable summaries.
  - Out of scope: unbounded full provenance graph traversal in hot paths.
  - Completion criteria: policy checks run from summaries in bounded time and
    retain bounded witness output for denials.
- [ ] Task 2.2.2: Enforce fail-closed behaviour on budget overflow and redact
      explanation surfaces.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` sections "Fail-closed rules" and
      "Explanation contract",
    - `docs/verification-targets.md` rows "Policy engine" and "Audit
      pipeline",
    - `docs/zamburak-engineering-standards.md` section
      "Fail-closed standards".
  - Dependencies: Task 2.2.1.
  - In scope: unknown-top fallback, deny-or-confirm conservative outcomes, and
    redacted explanation payloads.
  - Out of scope: permissive fallback on overflow.
  - Completion criteria: overflow tests demonstrate conservative policy outcomes
    and explanation payloads exclude raw untrusted or secret values.

## Phase 3: Tool and Model Context Protocol (MCP) trust boundaries

### Step 3.1: Tool catalogue and trust-class enforcement

- [ ] Task 3.1.1: Implement local pinned tool catalogue loading and hash
      validation.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section "Tool catalogue and pinning",
    - `docs/verification-targets.md` row
      "Tool catalogue and MCP boundary",
    - `docs/zamburak-engineering-standards.md` section
      "Tool and Model Context Protocol (MCP) trust standards",
    - `docs/repository-layout.md` sections `policies/` and
      `crates/zamburak-tools`.
  - Dependencies: Task 0.1.1.
  - In scope: immutable catalogue entries, schema hash checks, and
    documentation hash checks.
  - Out of scope: runtime acceptance of mutable remote documentation.
  - Completion criteria: runtime rejects tool binding when catalogue hash or
    version mismatches.
- [ ] Task 3.1.2: Enforce MCP trust classes and per-server capability budgets.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section "MCP server trust classes",
    - `docs/verification-targets.md` row
      "Tool catalogue and MCP boundary",
    - `docs/zamburak-engineering-standards.md` section
      "Tool and Model Context Protocol (MCP) trust standards",
    - `docs/repository-layout.md` section `crates/zamburak-tools`.
  - Dependencies: Task 3.1.1.
  - In scope: `TrustedLocal` and `RemoteThirdParty` class enforcement with
    policy budget checks.
  - Out of scope: implicit trust promotion by adapter defaults.
  - Completion criteria: remote third-party servers cannot expose tools outside
    configured budgets.

### Step 3.2: Draft and commit side-effect governance

- [ ] Task 3.2.1: Implement draft-to-commit lineage checks for irreversible
      operations.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section "Draft and commit action
      model",
    - `docs/verification-targets.md` rows "Control context" and
      "Audit pipeline",
    - `docs/zamburak-engineering-standards.md` section
      "Information-flow correctness standards",
    - `docs/repository-layout.md` sections `crates/zamburak-agent` and
      `crates/zamburak-tools`.
  - Dependencies: Tasks 1.2.1, 3.1.1.
  - In scope: mandatory draft identifiers, commit-time re-evaluation, and audit
    linkage.
  - Out of scope: direct commit execution without lineage evidence.
  - Completion criteria: high-risk commits require approved draft lineage and
    emit linked audit records.

## Phase 4: Large language model (LLM) sink governance

### Step 4.1: Planner and quarantined sink policy enforcement

- [ ] Task 4.1.1: Implement sink-policy signatures for planner LLM (P-LLM) and
      quarantined LLM (Q-LLM) paths.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` sections
      "Planner and quarantined processing" and
      "LLM calls as exfiltration sinks",
    - `docs/verification-targets.md` row "LLM sink enforcement",
    - `docs/zamburak-engineering-standards.md` section
      "LLM sink governance standards",
    - `docs/repository-layout.md` sections `crates/zamburak-agent` and
      `crates/zamburak-tools`.
  - Dependencies: Tasks 1.1.1, 3.1.1.
  - In scope: per-path sink signatures, confidentiality budgets, and required
    authority requirements.
  - Out of scope: treating LLM calls as non-effectful operations.
  - Completion criteria: policy engine evaluates every P-LLM and Q-LLM call as
    a sink with enforceable signature rules.
- [ ] Task 4.1.2: Implement three-point sink enforcement architecture and
      transport-time minimization controls.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section
      "LLM sink enforcement architecture",
    - `docs/verification-targets.md` row "LLM sink enforcement",
    - `docs/zamburak-engineering-standards.md` sections
      "LLM sink governance standards" and
      "Audit and observability standards",
    - `docs/repository-layout.md` sections `crates/zamburak-interpreter`,
      `crates/zamburak-tools`, and `crates/zamburak-policy`.
  - Dependencies: Task 4.1.1.
  - In scope: pre-dispatch checks, adapter guards, and post-dispatch audit
    linkage.
  - Out of scope: best-effort redaction without hard enforcement.
  - Completion criteria: sink calls are blocked when required transforms are
    missing or label budgets are exceeded.

### Step 4.2: Privacy boundary and local-only compatibility

- [ ] Task 4.2.1: Add end-to-end privacy boundary integration tests.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section "Privacy boundary statement",
    - `docs/verification-targets.md` row "LLM sink enforcement",
    - `docs/zamburak-engineering-standards.md` section
      "Required test categories",
    - `docs/repository-layout.md` section `tests/integration/`.
  - Dependencies: Task 4.1.2.
  - In scope: test scenarios proving policy-mediated denial or confirmation for
    disallowed payloads.
  - Out of scope: provider-side data residency guarantees.
  - Completion criteria: integration suites demonstrate enforcement for
    disallowed label combinations on LLM egress.
- [ ] Task 4.2.2: Implement local-only compatibility profile for provider
      abstraction.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` sections "Privacy boundary statement"
      and "Workload assumptions and service envelopes",
    - `docs/zamburak-engineering-standards.md` section
      "LLM sink governance standards",
    - `docs/repository-layout.md` sections `crates/zamburak-agent` and
      `crates/zamburak-tools`.
  - Dependencies: Task 4.1.1.
  - In scope: interface contracts that support local back ends without policy
    model changes.
  - Out of scope: mandatory local inference for all deployments.
  - Completion criteria: local back-end adapters run through identical policy
    interfaces and sink checks.

## Phase 5: Audit hardening, evaluation, and automation delivery

### Step 5.1: Confidentiality-first audit pipeline

- [ ] Task 5.1.1: Implement summary-first audit record persistence.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` sections "Audit record schema" and
      "Confidentiality-first defaults",
    - `docs/verification-targets.md` row "Audit pipeline",
    - `docs/zamburak-engineering-standards.md` section
      "Audit and observability standards",
    - `docs/repository-layout.md` sections `crates/zamburak-policy` and
      `crates/zamburak-cli`.
  - Dependencies: Tasks 2.2.2, 4.1.2.
  - In scope: summary references, tokenized identifiers, and default redaction.
  - Out of scope: plaintext payload logging by default.
  - Completion criteria: audit logs persist summaries and hashes only unless
    explicit secure override controls are enabled.
- [ ] Task 5.1.2: Implement tamper-evident audit chaining with retention
      controls.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section "Integrity and retention",
    - `docs/verification-targets.md` row "Audit pipeline",
    - `docs/zamburak-engineering-standards.md` section
      "Audit and observability standards",
    - `docs/repository-layout.md` sections `crates/zamburak-policy` and
      `tests/integration/`.
  - Dependencies: Task 5.1.1.
  - In scope: append-only record chaining, retention by age and size, and
    chain-validation tooling.
  - Out of scope: immutable external ledger back ends.
  - Completion criteria: validation detects insertion or truncation and
    retention policies are test-covered.

### Step 5.2: Verification progression and benchmark gates

- [ ] Task 5.2.1: Maintain mechanistic and security regression gates as
      merge-blocking controls.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` sections
      "Mechanistic correctness requirements" and
      "Security regression suite",
    - `docs/verification-targets.md` sections
      "Verification target matrix" and
      "Failure and escalation policy",
    - `docs/zamburak-engineering-standards.md` section
      "Testing and verification evidence standards".
  - Dependencies: Tasks 1.2.2, 2.2.2, 3.1.2.
  - In scope: invariant regression gating and bypass-regression preservation.
  - Out of scope: non-security functional test expansion unrelated to design
    invariants.
  - Completion criteria: CI blocks merges on invariant regressions, uncovered
    opcode flow paths, or known bypass reintroductions.
- [ ] Task 5.2.2: Integrate model-in-loop adversarial benchmark stage.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section
      "End-to-end adversarial evaluation roadmap",
    - `docs/verification-targets.md` row "LLM sink enforcement",
    - `docs/repository-layout.md` sections `tests/benchmarks/` and `scripts/`.
  - Dependencies: Task 5.2.1.
  - In scope: benchmark harness execution on representative workloads and trend
    output publication.
  - Out of scope: autonomous red-team generation beyond roadmap scope.
  - Completion criteria: benchmark stage runs in CI and publishes comparable
    trend metrics for sink enforcement and policy latency.

### Step 5.3: Scripted automation quality and CI integration

- [ ] Task 5.3.1: Implement benchmark and verification helper scripts using
      Cyclopts, Cuprum, and Pathlib conventions.
  - Requirement signposts:
    - `docs/scripting-standards.md` sections
      "Cyclopts CLI pattern (environment-first)",
      "Cuprum: command invocation and capability-scoped execution", and
      "Pathlib: robust path manipulation",
    - `docs/repository-layout.md` section "Root and operational files"
      (`scripts/`),
    - `docs/tech-baseline.md` section "Baseline usage contract".
  - Dependencies: Task 0.2.3 and Task 5.2.2.
  - In scope: script ergonomics, deterministic command execution, and
    repository-local execution semantics.
  - Out of scope: shell-only script replacements for Python standards.
  - Completion criteria: scripts are idempotent, dependency-pinned in `uv`
    blocks, and callable in local and CI contexts.
- [ ] Task 5.3.2: Add script test suites and CI wiring for script artefacts.
  - Requirement signposts:
    - `docs/scripting-standards.md` sections "Testing expectations" and
      "CI wiring: GitHub Actions (Cyclopts-first)",
    - `docs/verification-targets.md` section
      "Evidence requirements by target class",
    - `docs/repository-layout.md` section "Root and operational files"
      (`scripts/`, `.github/workflows/`).
  - Dependencies: Task 5.3.1.
  - In scope: `pytest`-based coverage, command mocking, environment-path test
    coverage, and workflow execution wiring.
  - Out of scope: non-deterministic external-call test execution in CI.
  - Completion criteria: script tests pass in CI with deterministic fixtures,
    and script failures are merge-blocking.
- [ ] Task 5.3.3: Implement automatic Mermaid diagram generation from
      Zamburak policy files with mmdr rendering.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section "Policy visualization",
    - `docs/zamburak-design-document.md` section
      "Policy evaluation semantics",
    - `docs/scripting-standards.md` sections
      "Cyclopts CLI pattern (environment-first)" and
      "Cuprum: command invocation and capability-scoped execution",
    - `docs/repository-layout.md` section "Root and operational files"
      (`scripts/`).
  - Dependencies: Task 0.1.1, Task 5.3.1, and Task 5.3.2.
  - In scope: Rust binary that reads a canonical v1 policy
    YAML file and emits one Mermaid `flowchart TD` per tool plus a
    global summary diagram; rendering of emitted Mermaid text to SVG
    and PNG via mmdr; CI integration so that diagram generation runs
    on policy file changes and rendering failures are merge-blocking.
  - Out of scope: interactive policy editors, real-time dashboard
    rendering, diagram generation for schema v0 without prior
    migration.
  - Completion criteria: running the generator against
    `policies/default.yaml` produces valid Mermaid text that passes
    nixie validation, rendered SVG and PNG artefacts are written to
    `docs/generated/`, and CI wiring blocks merges when generation or
    rendering fails.

## Phase 6: Localization and user-facing diagnostics

### Step 6.1: Injection-first localization foundation

- [ ] Task 6.1.1: Introduce core localization contracts and fallback-localizer
      behaviour.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` sections
      "Localization and user-facing diagnostics" and
      "Localization contract and ownership",
    - `docs/adr-002-localization-and-internationalization-with-fluent.md`
      sections "Decision outcome" and "Proposed architecture",
    - `docs/repository-layout.md` sections `crates/zamburak-core` and
      `tests/integration/`.
  - Dependencies: Task 0.3.1.
  - In scope: `Localizer` contract introduction, `NoOpLocalizer` fallback
    semantics, and explicit localized-rendering call-shape requirements.
  - Out of scope: policy for host application locale preferences.
  - Completion criteria: localized rendering APIs require explicit localizer
    context and return deterministic fallback text when lookups are unavailable.
- [ ] Task 6.1.2: Implement optional Fluent adapters and embedded catalogue
      loading helpers.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` sections
      "Fallback and Fluent layering semantics" and
      "Fluent adapter integration profile",
    - `docs/adr-002-localization-and-internationalization-with-fluent.md`
      sections "Fallback order" and "Locale ownership and negotiation",
    - `docs/repository-layout.md` sections `crates/zamburak-core` and
      `crates/zamburak-tools`.
  - Dependencies: Task 6.1.1.
  - In scope: host-owned `FluentLanguageLoader` adapters, embedded `en-US`
    resource exposure, and helper loading APIs.
  - Out of scope: initializing the global singleton localization loader.
  - Completion criteria: Fluent-backed localization composes with host-managed
    loaders and preserves defined fallback ordering.

### Step 6.2: Localized diagnostic rollout and conformance

- [ ] Task 6.2.1: Refactor user-facing diagnostics to explicit localized
      rendering entrypoints.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` section
      "Localized rendering semantics",
    - `docs/adr-002-localization-and-internationalization-with-fluent.md`
      section "Message rendering contract",
    - `docs/repository-layout.md` sections `crates/zamburak-policy`,
      `crates/zamburak-agent`, and `tests/security/`.
  - Dependencies: Tasks 4.1.2 and 6.1.1.
  - In scope: explicit `&dyn Localizer` plumbing for user-facing denial,
    verification, and confirmation diagnostics.
  - Out of scope: replacing stable English `Display` output used for logs and
    machine assertions.
  - Completion criteria: every public user-facing diagnostic has a localized
    rendering path with explicit fallback copy.
- [ ] Task 6.2.2: Add localization conformance tests and integration
      documentation.
  - Requirement signposts:
    - `docs/zamburak-design-document.md` sections
      "Fallback and Fluent layering semantics" and
      "Design-level acceptance criteria before phase 1 build-out",
    - `docs/adr-002-localization-and-internationalization-with-fluent.md`
      sections "Implementation plan" and "Acceptance criteria",
    - `docs/repository-layout.md` sections `tests/integration/`,
      `tests/security/`, and `docs/`.
  - Dependencies: Task 6.2.1 and Task 6.1.2.
  - In scope: fallback-order tests, interpolation-failure tests, missing-key
    behaviour tests, and host-integration usage documentation.
  - Out of scope: locale-specific copywriting policy outside bundled resources.
  - Completion criteria: localization suites demonstrate deterministic fallback
    layering and prove absence of ambient global localization state.

## Roadmap-to-artefact traceability

This table maps each task to primary repository artefacts that should exist
when the task is complete.

| Task  | Primary artefact paths                                                                                                                    | Expected outcome                                                       |
| ----- | ----------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| 0.1.1 | `policies/schema.json`, `crates/zamburak-policy/src/policy_def.rs`, `crates/zamburak-policy/src/engine.rs`                                | Canonical schema v1 is enforced fail-closed.                           |
| 0.1.2 | `crates/zamburak-policy/src/migration.rs`, `tests/compatibility/`, `tests/security/`                                                      | Migration transforms are explicit, test-covered, and auditable.        |
| 0.1.3 | `crates/zamburak-core/src/authority.rs`, `crates/zamburak-policy/src/engine.rs`, `tests/security/`                                        | Authority lifecycle transitions are enforced and validated.            |
| 0.2.1 | `rust-toolchain.toml`, `Makefile`, `docs/tech-baseline.md`                                                                                | Baseline config and baseline documentation remain synchronized.        |
| 0.2.2 | `.github/workflows/`, `Makefile`, `docs/verification-targets.md`                                                                          | Phase-gate suites are wired as merge-blocking checks.                  |
| 0.2.3 | `scripts/`, `scripts/tests/`, `docs/scripting-standards.md`                                                                               | Script delivery follows defined runtime and testing standards.         |
| 0.3.1 | `.github/workflows/`, `tests/compatibility/`, `tests/integration/`, `tests/security/`                                                     | Phase 1 start is blocked until design-contract suites pass.            |
| 0.4.1 | `.gitmodules`, `third_party/full-monty/`, `docs/monty-fork-policy.md`                                                                     | `full-monty` submodule exists with explicit fork-policy constraints.   |
| 0.4.2 | `Makefile`, `scripts/`, `docs/adr-001-monty-ifc-vm-hooks.md`                                                                              | `make monty-sync` provides repeatable fork sync plus verification.     |
| 0.5.1 | `third_party/full-monty/`, `tests/compatibility/`                                                                                         | Stable runtime IDs survive suspend or resume and dump or load cycles.  |
| 0.5.2 | `third_party/full-monty/`, `tests/compatibility/`, `tests/security/`                                                                      | Generic observer events exist with no-op semantic parity checks.       |
| 0.5.3 | `third_party/full-monty/`, `tests/compatibility/`                                                                                         | Snapshot extension seam round-trips opaque embedder state safely.      |
| 0.5.4 | `tests/compatibility/`, `tests/benchmarks/`, `docs/verification-targets.md`                                                               | Track A compatibility and overhead invariants are enforced in gates.   |
| 0.6.1 | `crates/zamburak-monty/src/`, `src/`, `tests/integration/`                                                                                | Governed execution path wraps `full-monty` through one adapter API.    |
| 0.6.2 | `crates/zamburak-ifc/src/`, `tests/property/`, `tests/security/`                                                                          | IFC dependency graph and propagation rules are implemented and tested. |
| 0.6.3 | `crates/zamburak-monty/src/observer.rs`, `crates/zamburak-ifc/src/`, `tests/integration/`                                                 | Observer events drive complete IFC updates in normal and strict modes. |
| 0.6.4 | `crates/zamburak-monty/src/external_call.rs`, `crates/zamburak-policy/src/engine.rs`, `tests/security/`                                   | Every external call is policy-gated before effect execution.           |
| 0.6.5 | `tests/compatibility/`, `tests/security/`, `tests/integration/`                                                                           | Compatibility, security, and snapshot-governance suites are active.    |
| 0.7.1 | `docs/monty-fork-policy.md`, `docs/zamburak-engineering-standards.md`                                                                     | Track A patch-budget and naming constraints are enforced in review.    |
| 0.7.2 | `third_party/full-monty/`, `docs/adr-001-monty-ifc-vm-hooks.md`                                                                           | Track A commits are upstream-shaped and mapped to upstream PRs.        |
| 0.7.3 | `Makefile`, `scripts/`, `docs/monty-fork-policy.md`                                                                                       | Range-diff drift checks are repeatable and escalation-triggered.       |
| 1.1.1 | `crates/zamburak-core/src/trust.rs`, `crates/zamburak-core/src/capability.rs`, `crates/zamburak-policy/src/engine.rs`                     | Integrity, confidentiality, and authority remain separate types.       |
| 1.1.2 | `crates/zamburak-sanitizers/src/`, `crates/zamburak-policy/src/engine.rs`, `tests/security/`                                              | Verification and endorsement semantics are deterministic and enforced. |
| 1.2.1 | `crates/zamburak-core/src/control_context.rs`, `crates/zamburak-interpreter/src/external_call.rs`, `crates/zamburak-policy/src/engine.rs` | Effect checks include execution-context summaries.                     |
| 1.2.2 | `tests/security/`, `tests/integration/`                                                                                                   | Control-context side-channel regressions are covered.                  |
| 2.1.1 | `crates/zamburak-core/src/dependency_graph.rs`, `crates/zamburak-core/src/container_state.rs`, `tests/property/`                          | Mutable containers use versioned, alias-safe provenance state.         |
| 2.1.2 | `crates/zamburak-interpreter/src/opcodes.rs`, `tests/property/`, `tests/security/`                                                        | Supported opcode and built-in propagation coverage is complete.        |
| 2.2.1 | `crates/zamburak-core/src/summary.rs`, `crates/zamburak-core/src/witness.rs`, `crates/zamburak-policy/src/engine.rs`                      | Summary fast path and bounded witness behaviour are implemented.       |
| 2.2.2 | `crates/zamburak-policy/src/decision.rs`, `crates/zamburak-policy/src/audit.rs`, `tests/security/`                                        | Overflow handling is fail-closed and explanations are redacted.        |
| 3.1.1 | `crates/zamburak-tools/src/catalogue.rs`, `policies/`, `tests/compatibility/`                                                             | Tool binding enforces pinned schema and documentation hashes.          |
| 3.1.2 | `crates/zamburak-tools/src/mcp_bridge.rs`, `crates/zamburak-policy/src/engine.rs`, `tests/security/`                                      | MCP trust classes enforce per-server capability budgets.               |
| 3.2.1 | `crates/zamburak-tools/src/`, `crates/zamburak-agent/src/confirmation.rs`, `tests/integration/`                                           | Irreversible operations require valid draft-to-commit lineage.         |
| 4.1.1 | `crates/zamburak-agent/src/planner.rs`, `crates/zamburak-tools/src/llm_adapter.rs`, `crates/zamburak-policy/src/engine.rs`                | P-LLM and Q-LLM calls use explicit sink-policy signatures.             |
| 4.1.2 | `crates/zamburak-interpreter/src/external_call.rs`, `crates/zamburak-tools/src/llm_adapter.rs`, `crates/zamburak-policy/src/audit.rs`     | Three-point sink enforcement is active with audit linkage.             |
| 4.2.1 | `tests/integration/`, `tests/security/`                                                                                                   | Privacy-boundary behaviour is validated end to end.                    |
| 4.2.2 | `crates/zamburak-agent/src/planner.rs`, `crates/zamburak-tools/src/llm_adapter.rs`                                                        | Local-only profile is supported through the same policy interfaces.    |
| 5.1.1 | `crates/zamburak-policy/src/audit.rs`, `crates/zamburak-cli/src/commands/audit.rs`, `tests/integration/`                                  | Audit defaults are confidentiality-first and summary-based.            |
| 5.1.2 | `crates/zamburak-policy/src/audit_chain.rs`, `tests/integration/`, `tests/security/`                                                      | Tamper-evident chaining and retention enforcement are active.          |
| 5.2.1 | `.github/workflows/`, `tests/property/`, `tests/security/`                                                                                | Mechanistic and regression suites are merge-blocking gates.            |
| 5.2.2 | `tests/benchmarks/`, `scripts/`, `.github/workflows/`                                                                                     | Model-in-loop adversarial benchmark trends are produced in CI.         |
| 5.3.1 | `scripts/`, `scripts/tests/`                                                                                                              | New scripts comply with Cyclopts, Cuprum, and Pathlib standards.       |
| 5.3.2 | `scripts/tests/`, `.github/workflows/`, `docs/scripting-standards.md`                                                                     | Script tests and CI wiring are deterministic and enforced.             |
| 5.3.3 | `scripts/`, `scripts/tests/`, `docs/generated/`, `.github/workflows/`                                                                     | Policy diagrams are auto-generated via mmdr and CI-enforced.           |
| 6.1.1 | `crates/zamburak-core/src/i18n/mod.rs`, `crates/zamburak-core/src/i18n/localizer.rs`, `tests/integration/`                                | Core localization contracts enforce explicit localizer injection.      |
| 6.1.2 | `crates/zamburak-core/src/i18n/fluent_adapter.rs`, `crates/zamburak-core/src/i18n/localizations.rs`, `locales/`                           | Fluent adapters compose with host loaders and bundled resources.       |
| 6.2.1 | `crates/zamburak-policy/src/diagnostics.rs`, `crates/zamburak-agent/src/confirmation.rs`, `crates/zamburak-tools/src/`                    | User-facing diagnostics expose explicit localized rendering paths.     |
| 6.2.2 | `tests/integration/`, `tests/security/`, `docs/`                                                                                          | Localization fallback and integration behaviour are test-covered.      |

_Table 1: Roadmap tasks mapped to primary implementation artefacts._

## Cross-phase acceptance criteria

The roadmap is complete when:

- each task completion criterion is met with objective evidence,
- all verification-target gates remain green in CI,
- all effectful boundaries are policy-gated with execution context,
- mutable-state provenance remains sound under aliasing and high churn,
- unknown analysis states fail closed,
- LLM and tool communications honour confidentiality budgets,
- audit records remain useful without becoming a secondary data-leak channel,
- `full-monty` delta stays upstream-PR-able and free of Zamburak semantics,
- localized diagnostics use explicit localizer injection with deterministic
  fallback layering and no global mutable localization state,
- automation scripts required by roadmap tasks comply with
  `docs/scripting-standards.md`.
