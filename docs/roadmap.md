# Zamburak roadmap

This roadmap is the high-level implementation plan for Zamburak.

Detailed semantics, invariants, interfaces, and trust-boundary rules are
defined in `docs/zamburak-design-document.md`.

Repository path mapping and file ownership are defined in
`docs/repository-layout.md`.

Engineering process and quality gates are defined in
`docs/zamburak-engineering-standards.md` and `AGENTS.md`.

Technology baseline constraints are defined in `docs/tech-baseline.md`.

Verification target expectations are defined in `docs/verification-targets.md`.

## Scope model

Roadmap items are grouped as phases, steps, and tasks.

- phases define major capability milestones,
- steps define coherent workstreams within each phase,
- tasks define measurable completion criteria.

## Phase 0: Design-contract freeze and baseline artefacts

### Step 0.1: Canonical policy schema contract

- [ ] Task 0.1.1: Freeze canonical policy schema v1 and compatibility rules.
  - Design reference: sections "Canonical policy schema v1" and
    "Schema compatibility and migration semantics".
  - Completion criteria: policy loader accepts only schema v1 and rejects
    unknown versions fail-closed.
- [ ] Task 0.1.2: Implement schema migration semantics and conformance tests.
  - Design reference: section "Schema compatibility and migration semantics".
  - Completion criteria: explicit version migrations are test-covered and
    produce auditable migration records.

### Step 0.2: Sink and authority lifecycle contracts

- [ ] Task 0.2.1: Implement planner large language model (P-LLM) and
      quarantined large language model (Q-LLM) sink enforcement architecture
      contracts.
  - Design reference: section "LLM sink enforcement architecture".
  - Completion criteria: pre-dispatch policy checks, transport guards, and
    linked audit records exist for P-LLM and Q-LLM paths.
- [ ] Task 0.2.2: Implement authority token lifecycle conformance.
  - Design reference: section "Authority token lifecycle semantics".
  - Completion criteria: mint, delegation, revocation, expiry, and
    snapshot-restore behaviour are covered by contract tests.

### Step 0.3: Baseline and verification references

- [ ] Task 0.3.1: Maintain technology baseline reference document.
  - Design reference: section "Context and orientation" in
    `docs/tech-baseline.md`.
  - Completion criteria: toolchain and quality-gate baseline is documented and
    versioned.
- [ ] Task 0.3.2: Maintain verification target matrix reference document.
  - Design reference: section "Verification target matrix" in
    `docs/verification-targets.md`.
  - Completion criteria: each security-relevant subsystem has explicit
    verification target coverage and evidence requirements.

### Step 0.4: Pre-phase conformance gate

- [ ] Task 0.4.1: Enforce design-level conformance gate before Phase 1.
  - Design reference: section
    "Design-level acceptance criteria before phase 1 build-out".
  - Completion criteria: schema, sink enforcement, and authority lifecycle
    contract suites pass in continuous integration (CI) before phase-1 tasks
    are allowed to start.

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

## Phase 3: Tool and Model Context Protocol (MCP) trust boundaries

### Step 3.1: Pinned tool catalogue

- [ ] Task 3.1.1: Implement local pinned tool catalogue with hash verification.
  - Design reference: section "Tool catalogue".
  - Completion criteria: runtime refuses tool binding when version or hash does
    not match catalogue entry.
- [ ] Task 3.1.2: Disallow mutable remote tool documentation at runtime.
  - Design reference: sections "Tool catalogue" and
    "Threat model and trust boundaries".
  - Completion criteria: policy and adapter paths consume only pinned docs.

### Step 3.2: Model Context Protocol (MCP) server trust classification

- [ ] Task 3.2.1: Add MCP provider trust classes and capability budgets.
  - Design reference: section "MCP server trust classes".
  - Completion criteria: `RemoteThirdParty` services cannot expose tools
    outside configured budgets.
- [ ] Task 3.2.2: Validate draft and commit lineage enforcement.
  - Design reference: section "Draft and commit pattern".
  - Completion criteria: high-risk tools require a reviewed draft before commit
    and emit linked audit records.

## Phase 4: large language model (LLM) sink governance and privacy controls

### Step 4.1: LLM sink policy signatures

- [ ] Task 4.1.1: Treat planner large language model (P-LLM) and quarantined
      large language model (Q-LLM) calls as policy-governed sinks.
  - Design reference: section "LLM calls as sinks".
  - Completion criteria: calls are blocked when labels exceed sink budget.
- [ ] Task 4.1.2: Implement mandatory minimisation and redaction transforms.
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
  - Completion criteria: continuous integration (CI) blocks merge on invariant
    regressions and known bypass reintroductions.
- [ ] Task 5.2.2: Integrate model-in-loop adversarial benchmark stage.
  - Design reference: section "End-to-end adversarial evaluation roadmap".
  - Completion criteria: benchmark harness runs on representative task sets and
    publishes comparable trend metrics.

## Roadmap-to-artifact traceability

This table maps roadmap tasks to primary repository artefacts that should exist
when tasks are complete.

| Task  | Primary artefact paths                                                                                                                            | Expected outcome                                                                          |
| ----- | ------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| 0.1.1 | `policies/schema.json`, `crates/zamburak-policy/src/policy_def.rs`, `docs/zamburak-design-document.md`                                            | Canonical policy schema v1 is normative and enforced fail-closed.                         |
| 0.1.2 | `crates/zamburak-policy/src/migration.rs`, `tests/compatibility/`, `tests/security/`                                                              | Schema migrations are explicit, test-covered, and auditable.                              |
| 0.2.1 | `crates/zamburak-agent/src/planner.rs`, `crates/zamburak-tools/src/llm_adapter.rs`, `crates/zamburak-policy/src/engine.rs`, `tests/integration/`  | P-LLM and Q-LLM sink checks run at defined enforcement points with linked audit evidence. |
| 0.2.2 | `crates/zamburak-core/src/authority.rs`, `crates/zamburak-policy/src/engine.rs`, `crates/zamburak-interpreter/src/snapshot.rs`, `tests/security/` | Authority lifecycle semantics are implemented and contract-tested.                        |
| 0.3.1 | `docs/tech-baseline.md`                                                                                                                           | Technology and toolchain baseline is documented and maintained.                           |
| 0.3.2 | `docs/verification-targets.md`                                                                                                                    | Verification target matrix defines required evidence by subsystem.                        |
| 0.4.1 | `.github/workflows/`, `Makefile`, `docs/zamburak-design-document.md`                                                                              | CI blocks phase-1 execution when design-contract conformance fails.                       |
| 1.1.1 | `crates/zamburak-core/src/trust.rs`, `crates/zamburak-core/src/authority.rs`, `crates/zamburak-policy/src/engine.rs`                              | Integrity, confidentiality, and authority remain separate types and interfaces.           |
| 1.1.2 | `crates/zamburak-sanitizers/src/`, `tests/security/`                                                                                              | Verification kinds are deterministic and regression-tested.                               |
| 1.2.1 | `crates/zamburak-core/src/control_context.rs`, `crates/zamburak-policy/src/engine.rs`                                                             | Effect checks include argument plus execution-context summaries.                          |
| 1.2.2 | `tests/security/`, `tests/integration/`                                                                                                           | Control-flow side-effect exfiltration regressions are covered.                            |
| 2.1.1 | `crates/zamburak-core/src/dependency_graph.rs`, `crates/zamburak-core/src/container_state.rs`                                                     | Mutable containers use versioned, acyclic provenance state.                               |
| 2.1.2 | `tests/property/`, `tests/security/`                                                                                                              | Aliasing and mutation provenance remain sound under churn.                                |
| 2.2.1 | `crates/zamburak-core/src/summary.rs`, `crates/zamburak-policy/src/engine.rs`                                                                     | Budget overflow yields unknown-top and conservative decisions.                            |
| 2.2.2 | `crates/zamburak-core/src/witness.rs`, `crates/zamburak-policy/src/decision.rs`                                                                   | Explanations remain bounded and redacted.                                                 |
| 3.1.1 | `crates/zamburak-tools/src/catalogue.rs`, `policies/`, `tests/compatibility/`                                                                     | Runtime enforces pinned tool schema and hash constraints.                                 |
| 3.1.2 | `crates/zamburak-tools/src/catalogue.rs`, `crates/zamburak-tools/src/mcp_bridge.rs`                                                               | Mutable remote tool documentation is rejected at runtime.                                 |
| 3.2.1 | `crates/zamburak-tools/src/mcp_bridge.rs`, `crates/zamburak-policy/src/engine.rs`                                                                 | MCP trust classes enforce per-server capability budgets.                                  |
| 3.2.2 | `crates/zamburak-tools/src/`, `crates/zamburak-agent/src/confirmation.rs`, `tests/integration/`                                                   | Draft/commit lineage is policy-enforced and audit-linked.                                 |
| 4.1.1 | `crates/zamburak-agent/src/planner.rs`, `crates/zamburak-tools/src/llm_adapter.rs`, `crates/zamburak-policy/src/engine.rs`                        | All LLM calls are sink-governed with enforceable policy signatures.                       |
| 4.1.2 | `crates/zamburak-interpreter/src/redaction.rs`, `crates/zamburak-tools/src/llm_adapter.rs`, `tests/security/`                                     | Prompt emission applies mandatory minimisation and redaction.                             |
| 4.2.1 | `tests/integration/`, `tests/security/`                                                                                                           | End-to-end privacy boundary behaviour is validated.                                       |
| 4.2.2 | `crates/zamburak-agent/src/planner.rs`, `crates/zamburak-tools/src/llm_adapter.rs`                                                                | Interfaces support local-only inference back ends.                                        |
| 5.1.1 | `crates/zamburak-policy/src/audit.rs`, `crates/zamburak-cli/src/commands/audit.rs`                                                                | Audit defaults remain summary-only and confidentiality-first.                             |
| 5.1.2 | `crates/zamburak-policy/src/audit_chain.rs`, `tests/integration/`                                                                                 | Tamper-evident chaining and retention controls are enforced.                              |
| 5.2.1 | `.github/workflows/`, `tests/security/`, `tests/property/`                                                                                        | CI enforces mechanistic and regression corpus gates.                                      |
| 5.2.2 | `tests/benchmarks/`, `scripts/`, `.github/workflows/`                                                                                             | Model-in-loop adversarial benchmark trends are reported.                                  |

## Cross-phase acceptance criteria

The roadmap is complete when:

- all effectful boundaries are policy-gated with execution context,
- mutable-state provenance remains sound under aliasing and high churn,
- unknown analysis states fail closed,
- LLM and tool communications honour confidentiality budgets,
- audit data remains useful without becoming a secondary data-leak channel.
