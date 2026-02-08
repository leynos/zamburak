# Zamburak roadmap

This roadmap is the high-level implementation plan for Zamburak.

Detailed semantics, invariants, interfaces, and trust-boundary rules are
defined in `docs/zamburak-design-document.md`.

Repository path mapping and file ownership are defined in
`docs/repository-layout.md`.

Engineering process and quality gates are defined in
`docs/zamburak-engineering-standards.md` and `AGENTS.md`.

## Scope model

Roadmap items are grouped as phases, steps, and tasks.

- phases define major capability milestones,
- steps define coherent workstreams within each phase,
- tasks define measurable completion criteria.

## Phase 1: Core runtime trust model

### Step 1.1: Label model separation and interfaces

- [ ] Task 1.1.1: Introduce separate runtime representations for integrity
      labels,
  confidentiality labels, and authority tokens.
  - Design reference: `docs/zamburak-design-document.md` section
    "Information flow model".
  - Completion criteria: no shared type is used to represent both data labels
    and authority; policy signatures compile against distinct inputs.
- [ ] Task 1.1.2: Define verification kinds with testable semantics.
  - Design reference: section "Verification and declassification semantics".
  - Completion criteria: each verification kind has a deterministic
    implementation contract and at least one positive and one negative test
    case.

### Step 1.2: Strict-mode control context and effect checks

- [ ] Task 1.2.1: Implement execution-context summaries on all effect
      boundaries.
  - Design reference: section "Policy model".
  - Completion criteria: every effectful call passes argument summaries and
    execution-context summary into policy evaluation.
- [ ] Task 1.2.2: Encode control-flow side-effect regression tests.
  - Design reference: sections "Strict-mode semantics" and
    "Security regression corpus".
  - Completion criteria: regression suite includes call-occurrence and
    call-count exfiltration attempts that are denied or confirmation-gated.

## Phase 2: Mutable-state soundness and fail-closed provenance

### Step 2.1: Container versioning model

- [ ] Task 2.1.1: Implement versioned container state for mutable collections.
  - Design reference: section "Container and mutation semantics".
  - Completion criteria: write operations produce new container versions;
    dependency graph remains acyclic in mutation-heavy tests.
- [ ] Task 2.1.2: Validate aliasing behaviour and provenance propagation.
  - Design reference: sections "Container and mutation semantics" and
    "Mechanistic correctness".
  - Completion criteria: aliasing tests verify consistent provenance after
    interleaved updates and reads.

### Step 2.2: Provenance budgets and fail-closed semantics

- [ ] Task 2.2.1: Add configurable provenance budgets and unknown-top fallback.
  - Design reference: section "Provenance summaries and graph budgets".
  - Completion criteria: budget overflows produce unknown-top summaries and
    conservative policy outcomes.
- [ ] Task 2.2.2: Add explainability witness bounds.
  - Design reference: sections "Provenance summaries and graph budgets" and
    "Policy model".
  - Completion criteria: denial explanations remain bounded and redact raw
    sensitive values.

## Phase 3: Tool and MCP trust boundaries

### Step 3.1: Pinned tool catalogue

- [ ] Task 3.1.1: Implement local pinned tool catalogue with hash verification.
  - Design reference: section "Tool catalogue".
  - Completion criteria: runtime refuses tool binding when version or hash does
    not match catalogue entry.
- [ ] Task 3.1.2: Disallow mutable remote tool documentation at runtime.
  - Design reference: sections "Tool catalogue" and
    "Threat model and trust boundaries".
  - Completion criteria: policy and adapter paths consume only pinned docs.

### Step 3.2: MCP server trust classification

- [ ] Task 3.2.1: Add MCP provider trust classes and capability budgets.
  - Design reference: section "MCP server trust classes".
  - Completion criteria: `RemoteThirdParty` services cannot expose tools
    outside configured budgets.
- [ ] Task 3.2.2: Validate draft and commit lineage enforcement.
  - Design reference: section "Draft and commit pattern".
  - Completion criteria: high-risk tools require a reviewed draft before commit
    and emit linked audit records.

## Phase 4: LLM sink governance and privacy controls

### Step 4.1: LLM sink policy signatures

- [ ] Task 4.1.1: Treat P-LLM and Q-LLM calls as policy-governed sinks.
  - Design reference: section "LLM calls as sinks".
  - Completion criteria: calls are blocked when labels exceed sink budget.
- [ ] Task 4.1.2: Implement mandatory minimization and redaction transforms.
  - Design reference: sections "LLM calls as sinks" and
    "Confidentiality-first logging".
  - Completion criteria: prompt payload paths apply required transforms before
    provider transmission.

### Step 4.2: Privacy boundary validation

- [ ] Task 4.2.1: Add end-to-end tests proving sink policy enforcement.
  - Design reference: sections "Privacy boundary statement" and
    "Mechanistic correctness".
  - Completion criteria: tests demonstrate denial for disallowed secret labels
    on LLM paths.
- [ ] Task 4.2.2: Define local-only compatibility profile.
  - Design reference: section "Local-only mode roadmap".
  - Completion criteria: interfaces support local inference back ends without
    policy model changes.

## Phase 5: Audit, evaluation, and hardening

### Step 5.1: Confidentiality-first audit pipeline

- [ ] Task 5.1.1: Enforce summary-only audit logging by default.
  - Design reference: section "Confidentiality-first logging".
  - Completion criteria: logs store identifiers and hashes by default; plaintext
    storage requires explicit opt-in controls.
- [ ] Task 5.1.2: Add tamper-evident hash chain and retention policy.
  - Design reference: section "Integrity and retention".
  - Completion criteria: append validation tooling detects insertion or
    truncation and retention limits are test-covered.

### Step 5.2: Security evaluation progression

- [ ] Task 5.2.1: Maintain mechanistic correctness and regression corpus gates.
  - Design reference: section "Verification and evaluation strategy".
  - Completion criteria: CI blocks merge on invariant regressions and known
    bypass reintroductions.
- [ ] Task 5.2.2: Integrate model-in-loop adversarial benchmark stage.
  - Design reference: section "End-to-end adversarial evaluation roadmap".
  - Completion criteria: benchmark harness runs on representative task sets and
    publishes comparable trend metrics.

## Cross-phase acceptance criteria

The roadmap is complete when:

- all effectful boundaries are policy-gated with execution context,
- mutable-state provenance remains sound under aliasing and high churn,
- unknown analysis states fail closed,
- LLM and tool communications honour confidentiality budgets,
- audit data remains useful without becoming a secondary data-leak channel.
