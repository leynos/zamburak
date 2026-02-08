# Zamburak system design

This document is the authoritative system design for Zamburak.

Implementation sequencing belongs in `docs/roadmap.md`.

Engineering process and quality rules belong in
`docs/zamburak-engineering-standards.md` and `AGENTS.md`.

## Goals and non-goals

### Goals

Zamburak is a capability-governed execution environment for agent-authored
Monty programs. It is designed to reduce prompt injection impact by combining
information flow control (IFC), policy checks at every effect boundary, and
explicit user confirmation for high-risk actions.

The primary security goal is policy compliance under tracked dependencies:

- data from untrusted or sensitive origins is tracked through execution,
- every effectful external call is evaluated with argument summaries and
  control-context summaries, and
- uncertain analysis outcomes fail closed.

### Non-goals

Zamburak does not claim full non-interference across all side channels and does
not claim global control-flow integrity in the classic systems-security sense.

Zamburak does not protect against a compromised host operating system, local
malware with process memory access, or theft of secrets outside the Zamburak
runtime boundary.

## Threat model and trust boundaries

### Attacker capabilities

The attacker can:

- control tool outputs, including email content, fetched web content,
  documents, and MCP response payloads,
- embed indirect instructions in untrusted text,
- attempt data exfiltration via any externally observable side effect,
  including whether a tool call occurs, and
- exploit overly broad tool metadata when tool descriptions are mutable at
  runtime.

### Protected assets

Protected assets include:

- private user data handled by tools,
- authentication material and authorisation secrets,
- audit logs and policy explanations,
- host-minted authority tokens.

### Trust boundaries

Zamburak treats the following boundaries as explicit:

- boundary between Monty execution and host external tools,
- boundary between Zamburak and LLM providers,
- boundary between trusted local MCP services and remote third-party MCP
  services,
- boundary between raw values and verified or endorsed values.

## Trusted computing base

Security claims rely on correctness of the following trusted computing base
(TCB) elements:

- Monty fork and runtime hooks used by Zamburak,
- IFC propagation logic and value/version tracking,
- policy engine and policy configuration loader,
- sanitizer and verifier implementations,
- authority token minting and validation,
- tool adapter layer and MCP catalogue enforcement,
- confirmation interface and commit workflow,
- audit logging and redaction implementation.

Any TCB bypass invalidates policy-compliance claims.

## Language subset and runtime semantics

### Supported execution subset

Zamburak mode supports Monty language features that have complete IFC coverage.
Coverage is defined per opcode and per built-in function. Unsupported
operations must return structured errors, not partial execution without labels.

### Propagation modes

Zamburak defines two propagation modes:

- `Normal`: track direct data dependencies for computed values.
- `Strict`: include active control-context dependencies in all value writes and
  all effect checks.

### Strict-mode semantics

Strict mode does not globally forbid branching on untrusted values. Instead,
strict mode tracks control context and makes control dependencies visible to
policy checks for every external side effect.

Security statement:

- in strict mode, any effectful tool call inherits the current control-context
  label and dependency summary, enabling conservative blocking of conditional
  exfiltration even when call arguments are constant.

### Container and mutation semantics

Mutable containers are modelled as versioned states.

- write operations produce new container versions,
- each new version depends on the previous version and the written value,
- reads depend on the container version observed by that read, and
- aliasing is represented by shared container identity with evolving versions.

This approach preserves acyclicity and avoids mutating provenance in place.

### Exceptions and redaction

Exception content crossing trust boundaries is redacted using label-aware
rules. Raw untrusted payloads are not forwarded to planner repair loops or
audit logs.

## Information flow model

### Separation of concerns

Zamburak separates three axes that were previously conflated:

- integrity and provenance: how trustworthy a value is,
- confidentiality and sensitivity: how harmful leakage would be,
- authority: what side effects code is permitted to perform.

### Integrity and provenance labels

Integrity labels classify trust in data origin and transformation:

- `Integrity::Untrusted`,
- `Integrity::Trusted`,
- `Integrity::Verified(VerificationKind)`.

Only host-registered deterministic verifiers may mint
`Integrity::Verified(VerificationKind)`.

### Confidentiality labels

Confidentiality labels are an independent set, for example:

- `PII`,
- `AUTH_SECRET`,
- `PRIVATE_EMAIL_BODY`,
- `FINANCIAL_ACCOUNT_DATA`.

Confidentiality labels accumulate through propagation unless explicit
policy-governed declassification occurs.

### Authority tokens

Authority is represented by unforgeable host-minted capability tokens, for
example:

- `EmailSendCap`,
- `CalendarWriteCap`,
- `PaymentInitiateCap`.

Agent-authored Monty code cannot mint authority tokens.

### Verification and declassification semantics

Verification and declassification are distinct.

Verification establishes properties about a value shape or membership, such as:

- `VerificationKind::AllowlistedEmailRecipient`,
- `VerificationKind::AllowlistedUrl`,
- `VerificationKind::BoundedAmountGbp(max)`.

Verification may upgrade integrity for the validated value. Verification does
not remove confidentiality labels.

Declassification or endorsement is a separate, explicit policy action, usually
paired with confirmation requirements.

### Provenance summaries and graph budgets

Zamburak uses two provenance representations:

- fast-path summary for O(1) policy checks,
- bounded witness provenance for explainability.

When provenance analysis exceeds configured budgets, summary state escalates to
an unknown-top state and policy evaluation fails closed.

Required budgets include:

- maximum value instances per execution,
- maximum parent edges recorded per value before summary fallback,
- maximum closure traversal steps per policy check,
- maximum witness graph depth for explain responses.

## Policy model

### Policy input contract

Every effectful external call is checked with:

- tool signature,
- argument label summaries,
- execution context summary.

`ExecutionContextSummary` includes at least:

- active control-context label summary,
- active control-context dependency identifiers,
- coarse effect counters for rate and occurrence checks.

### Decision model

Policy decisions are explicit and structured:

- `Allow`,
- `Deny`,
- `RequireConfirmation`,
- `RequireDraft`.

Policy defaults are deny-oriented for unknown tools, unknown labels, and
unknown summary states.

### Invariants

The policy engine must uphold these invariants:

- every external side effect is policy-gated,
- effect checks include control context and argument context,
- uncertain or incomplete provenance produces conservative outcomes,
- explanations reference label and dependency identifiers without exposing raw
  sensitive or untrusted content.

## Tool model and MCP trust model

### Tool catalogue

Zamburak resolves tools only through a local pinned catalogue.

Each catalogue entry contains:

- stable tool identifier,
- version,
- schema hash,
- static documentation hash,
- policy signature,
- trust class of provider.

Runtime acceptance of mutable remote tool documentation is not permitted.

### MCP server trust classes

MCP servers are classified as:

- `TrustedLocal`,
- `RemoteThirdParty`.

Per-server capability budgets restrict which tools may be bound and which
authority tokens may be presented.

### Draft and commit pattern

High-risk actions must use a draft and commit workflow:

- draft creation is reversible and reviewable,
- commit is separately policy-gated and optionally confirmation-gated,
- commit requests must include lineage to the reviewed draft.

## LLM interaction model

### LLM calls as sinks

P-LLM and Q-LLM calls are treated as external communication sinks.

Each call has a policy signature defining:

- allowed confidentiality labels,
- forbidden labels,
- required redaction or minimization transforms,
- maximum payload size and context class.

### Privacy boundary statement

Zamburak can prevent hostile content from steering tool use while still leaking
private data to an LLM provider if sink policy permits that flow.

Protection against provider disclosure requires explicit sink gating,
minimization, and deployment choices.

### Local-only mode roadmap

Local-only LLM execution is a near-term roadmap objective. It is not an MVP
requirement, but interfaces must preserve compatibility with local back ends.

## Observability and audit model

### Confidentiality-first logging

Audit logging defaults to summaries rather than raw values.

Required behaviour:

- log stable identifiers and content hashes instead of plaintext values,
- apply label-aware redaction before persistence,
- avoid storing raw argument content by default,
- treat logs as sensitive assets with restricted access.

### Integrity and retention

Tamper evidence is provided by append-only hash chaining for audit records.
Optional local signing can be enabled where key management is available.

Retention requirements:

- time-based deletion policy,
- size caps with deterministic eviction rules,
- explicit handling policy for exported logs and backups.

## Verification and evaluation strategy

### Mechanistic correctness

The verification baseline includes:

- opcode and built-in IFC completeness checks,
- property-based tests for propagation and monotonicity,
- mutation and aliasing soundness tests,
- strict-mode control-context propagation tests,
- fail-closed behaviour tests for budget exhaustion.

### Security regression corpus

A permanent regression corpus is required for:

- prompt injection variants through tool outputs,
- control-flow side-channel attempts,
- laundering attempts across string or serialisation transforms,
- tool-catalogue and documentation trust-boundary bypass attempts.

### End-to-end adversarial evaluation roadmap

Evaluation stages:

- MVP: mechanistic suite plus curated regression corpus,
- next stage: model-in-loop benchmark integration, including AgentDojo-class
  tasks or equivalent,
- later stage: continuous red-teaming with automated attack generation.

## Performance model and budgets

Performance expectations are framed as measurable budgets rather than absolute
startup equivalence claims.

Budget categories:

- VM overhead budget: IFC propagation cost per representative opcode mix,
- policy overhead budget: policy check latency for bounded summary sizes,
- agent-step budget: end-to-end latency envelope including tool and LLM calls.

Budget exceedance policy:

- alert when soft budgets are exceeded,
- fail closed when safety budgets are exceeded and provenance certainty is lost.

## Open risks and deferred items

The following remain known risks or deferred work:

- formal non-interference proofs are deferred,
- cryptographic provenance signing for tool documentation is deferred,
- complete local-only deployment profile is deferred,
- continuous automated red-team generation is deferred.

Deferred items do not weaken current invariants and must remain compatible with
this system design.

## References

- Pydantic Monty: <https://github.com/pydantic/monty>
- CaMeL paper: <https://arxiv.org/abs/2503.18813>
- CaMeL reference implementation:
  <https://github.com/google-research/camel-prompt-injection>
- Simon Willison on the lethal trifecta:
  <https://simonwillison.net/2025/Jun/16/the-lethal-trifecta/>
- Simon Willison on CaMeL:
  <https://simonwillison.net/2025/Apr/11/camel/>
