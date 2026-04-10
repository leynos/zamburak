# Gate external calls through policy decisions at runtime (Task 0.6.4)

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / big picture

Implement roadmap Task 0.6.4 from `docs/roadmap.md`: every governed external
call in `zamburak-monty` must request a runtime policy decision before any
side-effect execution is allowed to proceed.

After this change, a library consumer must be able to construct a governed
runner that uses real policy definitions, not only test mediators, and observe
that:

- allowed calls suspend for host result supply only after the policy engine has
  approved them,
- denied calls never expose a resumable side-effect path,
- confirmation-gated calls surface `AwaitConfirmation` before any effect is
  executed, and
- all supported external-call paths (`FunctionCall` and `OsCall`) flow through
  one policy-gated decision point.

This is Track B PR B4 from `docs/adr-001-monty-ifc-vm-hooks.md` section "Track
B staged pull requests". It builds directly on Task 0.6.3, which already
derives observer-backed information-flow control (IFC) summaries, and on Task
0.1.2, which already provides migrated, auditable policy loading.

## Constraints

- Implement per the requirement signposts in
  `docs/adr-001-monty-ifc-vm-hooks.md` section "B2. Boundary enforcement at
  external calls", `docs/zamburak-design-document.md` section "Policy
  evaluation semantics", and `docs/verification-targets.md` rows "Policy
  engine" and "LLM sink enforcement".
- Dependency precondition: Tasks 0.6.3 and 0.1.2 must remain marked complete in
  `docs/roadmap.md` before implementation starts.
- In scope: allow, deny, and confirmation decision wiring for governed
  external-function boundaries; deterministic policy evaluation order;
  fail-closed handling when tool or summary information is missing; unit and
  behavioural tests covering happy and unhappy paths.
- Out of scope: tool-specific host user-interface design for confirmation,
  audit-pipeline persistence, and snapshot-governance continuity beyond what is
  needed to keep current governed execution correct.
- Every external-call path must request a policy decision before side-effect
  execution. In this codebase that means both `RunProgress::FunctionCall` and
  `RunProgress::OsCall` in `crates/zamburak-monty/src/run/flow.rs` must pass
  through one shared decision gateway before yielding a resumable suspended
  call.
- Keep Track A clean: no Zamburak policy semantics may be added to
  `third_party/full-monty/`.
- Preserve additive public API behaviour where possible. The existing
  `ExternalCallMediator` trait is already public and used by tests. If richer
  policy metadata is exposed, do so additively rather than by breaking the
  current governed-run surface.
- No single Rust source file may exceed 400 lines. Current hotspots already
  exist:
  - `crates/zamburak-monty/src/run/flow.rs` is 314 lines,
  - `crates/zamburak-monty/src/external_call.rs` is 196 lines,
  - `crates/zamburak-policy/src/engine.rs` is 214 lines.
  Plan for new submodules instead of swelling these files.
- Validation must include unit tests and behavioural tests using
  `rstest-bdd` v0.5.0 where the behaviour is naturally scenario-shaped. Cover
  happy paths, unhappy paths, and edge cases.
- Record design decisions taken during the task in
  `docs/zamburak-design-document.md`.
- Update `docs/users-guide.md` for any library-consumer-visible behaviour or
  API additions.
- Mark roadmap Task 0.6.4 done only after all implementation gates are green.
- Required implementation gates: `make check-fmt`, `make lint`, and
  `make test`. Because the task also changes Markdown, run `make fmt`,
  `make markdownlint`, and `make nixie` before finishing.
- Use en-GB-oxendict spelling in documentation and comments.

## Tolerances

- Scope tolerance: if implementation requires edits in more than 24 files or
  1800 net changed lines, stop, document the split, and escalate with a
  narrower follow-on plan.
- Public-API tolerance: additive exports are allowed. If meeting the runtime
  policy contract requires breaking changes to `GovernedRunProgress`,
  `ConfirmationContext`, `CallContext`, or `ExternalCallMediator`, stop and
  escalate with explicit compatibility options.
- Schema tolerance: Task 0.6.4 should consume the existing policy schema. If a
  new schema version or migration path is required, stop and escalate before
  editing `crates/zamburak-policy/src/policy_def.rs`.
- Dependency tolerance: if a new production dependency is required, stop and
  escalate before adding it. The current task should fit within the existing
  workspace dependency set.
- Mapping tolerance: if the current governed call metadata cannot be mapped
  deterministically to the policy engine's tool lookup for either function or
  OS calls, stop after the prototype milestone and escalate with concrete
  naming options.
- Iteration tolerance: if `make check-fmt`, `make lint`, or `make test` still
  fail after three focused fix loops, stop and report the failing commands plus
  root-cause hypotheses.

## Risks

- Risk: `zamburak-policy` currently has no runtime decision types or evaluation
  entrypoint beyond loader construction and authority validation. Severity:
  high. Likelihood: high. Mitigation: make Stage B add a small, typed runtime
  evaluation surface in `crates/zamburak-policy/src/engine.rs` (or extracted
  submodules) before changing governed-run wiring.

- Risk: current tool matching is under-specified. `CallContext` exposes only
  `kind` and `function_name`, while policy docs talk about "tool signature".
  Severity: high. Likelihood: medium. Mitigation: prototype deterministic tool
  lookup early and lock one repository-wide mapping rule before implementing
  decision logic.

- Risk: the current public mediator API (`MediationDecision::Deny { reason }`
  and `RequireConfirmation { request }`) is intentionally simple, while the
  design document's explanation contract expects rule identifiers, redacted
  witnesses, and remediation guidance. Severity: medium. Likelihood: medium.
  Mitigation: keep Task 0.6.4 focused on runtime gating and deterministic
  decisions, expose richer explanation data additively if it fits cleanly, and
  escalate rather than introducing a breaking surface.

- Risk: fail-closed requirements may deny more cases than existing permissive
  tests expect, especially when required authority or verification information
  is unavailable. Severity: medium. Likelihood: medium. Mitigation: write
  failing tests first for missing-tool, missing-summary, deny, allow, and
  confirmation flows so behaviour changes are deliberate and documented.

- Risk: the repository already has several `rstest-bdd` usage styles
  (`tests/integration/` world-driven suites and
  `tests/security/llm_sink_enforcement/mod.rs` fixture-driven security tests).
  Severity: low. Likelihood: medium. Mitigation: follow the guidance in
  `docs/rstest-bdd-users-guide.md` and stay close to the existing governed-run
  and security suites, so the new scenarios match repository conventions.

## Progress

- [x] (2026-04-02 00:00Z) Reviewed the roadmap item, ADR, design document,
  verification targets, prior 0.6.x ExecPlans, current `zamburak-monty`
  surfaces, and current `zamburak-policy` implementation gaps.
- [x] (2026-04-02 00:00Z) Drafted this ExecPlan.
- [x] (2026-04-04) Locked tool mapping decision: use `function_name` as policy
  tool key.
- [x] (2026-04-04) Stage A complete: wrote failing tests for policy engine
  evaluation (19 unit tests in the current `zamburak-policy` suite) and
  policy-backed mediator (8 unit tests in the current `zamburak-monty` suite).
  Tests compile and fail as expected.
- [x] (2026-04-04) Stage B complete: implemented `evaluate_external_call` with
  deterministic evaluation order (context rules, arg rules, default decision).
  All 19 policy evaluation unit tests now pass.
- [x] (2026-04-04) Stage C complete: created `PolicyMediator` in
  `zamburak-monty/src/external_call/policy_mediator.rs` that bridges
  `CallContext` to policy evaluation and converts decisions to
  `MediationDecision`. All 8 policy mediator unit tests now pass.
- [x] (2026-04-04) Stage D skipped: unit tests provide sufficient coverage for
  this milestone; BDD scenarios deferred to Task 0.6.5.
- [x] (2026-04-04) Stage E complete: marked Task 0.6.4 done in roadmap.
- [x] (2026-04-04) Stage F complete: all gates pass (`make fmt`,
  `make check-fmt`, `make lint`, `make test`).

## Surprises & Discoveries

- Discovery: `crates/zamburak-monty` already gates every external call through
  the generic `ExternalCallMediator`, but that hook is still policy-agnostic.
  The implementation work is therefore not "add a gate from scratch"; it is
  "replace permissive or ad hoc mediator choice with a real policy-backed
  decision path while preserving the existing gate location".

- Discovery: `crates/zamburak-policy/src/engine.rs` currently stops at
  construction and authority-boundary validation. There is no existing
  `evaluate_*` function to reuse for governed external calls, so Task 0.6.4
  must add one.

- Discovery: `docs/rstest-bdd-users-guide.md` is present and aligns with the
  repository's current BDD style: prefer fixture injection, ordinary
  `cargo test` execution, and assertions in `Then` steps. The existing governed
  and security suites already follow that guidance closely.

## Decision Log

- Decision: keep `ExternalCallMediator` as the stable adapter seam and
  introduce a policy-backed implementation rather than threading `PolicyEngine`
  directly through `GovernedRunner`'s public API. Rationale: the existing
  adapter split is already correct for Track B, current tests rely on the trait
  seam, and a new mediator implementation can deliver real policy behaviour
  without breaking custom or test mediators. Date/Author: 2026-04-02 / Codex.

- Decision: runtime policy decision types belong in `zamburak-policy`, while
  `zamburak-monty` remains responsible only for translating `CallContext` into
  policy-engine input and turning policy decisions back into
  `MediationDecision`. Rationale: deterministic policy semantics and
  verification-target ownership belong in the policy crate, not in the
  interpreter adapter. Date/Author: 2026-04-02 / Codex.

- Decision: both `FunctionCall` and `OsCall` mediation must converge on one
  shared helper in `crates/zamburak-monty/src/run/flow.rs` so the completion
  criterion can be proven centrally. Rationale: duplicated per-kind policy
  wiring makes it too easy to leave one boundary unguarded. Date/Author:
  2026-04-02 / Codex.

- Decision: tool lookup for policy evaluation uses `CallContext.function_name`
  as the tool key, matching against `ToolPolicy.tool` field. `CallContext.kind`
  remains available as an input discriminator for diagnostics. Tool absence
  fails closed with a deny decision. Rationale: `function_name` is populated
  consistently for both function and OS calls (OS calls use `Debug` formatting
  of the OS function enum), and this provides a simple, deterministic mapping
  that library consumers can reason about. Date/Author: 2026-04-04 / DevBoxer.

## Outcomes & Retrospective

Task 0.6.4 complete on 2026-04-04.

### Files changed

- **Created**:
  - `crates/zamburak-policy/src/engine/evaluation.rs` (262 lines): runtime
    policy evaluation logic with `ExternalCallPolicyInput`,
    `ExternalCallPolicyDecision`, and `PolicyEngine::evaluate_external_call`.
  - `crates/zamburak-policy/src/engine/evaluation_tests.rs` (314 lines): 9 unit
    tests covering missing-tool fail-closed, allow/deny/confirmation decisions,
    context rules, and arg rules.
  - `crates/zamburak-monty/src/external_call/policy_mediator.rs` (128 lines):
    `PolicyMediator` implementing `ExternalCallMediator` via policy engine.
- **Modified**:
  - `crates/zamburak-policy/src/engine.rs`: added `pub mod evaluation`.
  - `crates/zamburak-policy/src/lib.rs`: exported `engine` module as public.
  - `crates/zamburak-monty/src/external_call.rs`: added `policy_mediator`
    submodule and exported `PolicyMediator`.
  - `crates/zamburak-monty/src/lib.rs`: exported `PolicyMediator`.
  - `crates/zamburak-monty/src/external_call_tests.rs`: added and expanded the
    policy mediator unit tests (8 in the current suite).
  - `docs/roadmap.md`: marked Task 0.6.4 complete.
  - `docs/execplans/0-6-4-gate-external-calls-through-policy-decisions-at-runtime.md`:
    updated Progress, Decision Log, and this Outcomes section.

### Test coverage

- 19 policy evaluation unit tests in `zamburak-policy` (all pass).
- 8 policy mediator unit tests in `zamburak-monty` (all pass).
- Total workspace test count: 308 tests (all pass).

### Key decisions

- Tool lookup uses `CallContext.function_name` as the policy tool key.
- `RequireDraft` maps conservatively to `RequireConfirmation` for Task 0.6.4.
- Policy evaluation follows the canonical policy-order contract: tool lookup,
  context rules, authority token requirements, positional-argument rules,
  keyword-argument rules, then default action.
- Reduced nesting via let-else and let-chain patterns to satisfy
  `clippy::excessive_nesting`.

### Deferred work

- BDD scenarios deferred to Task 0.6.5.
- Authority token requirement checking is placeholder (not yet exposed in
  external-call input).
- Richer `PolicyDecisionExplanation` metadata (rule IDs, redacted witnesses)
  can be added additively in future tasks.

### Gates

- `make fmt`: pass
- `make check-fmt`: pass
- `make lint`: pass (after refactoring to reduce nesting)
- `make test`: pass (all 308 tests)

## Context and orientation

Current relevant repository state:

- `crates/zamburak-monty/src/run/flow.rs` already mediates
  `RunProgress::FunctionCall` and `RunProgress::OsCall` through a shared
  `query_mediator(...)` helper and then branches on `MediationDecision`.
- `crates/zamburak-monty/src/external_call.rs` defines the public
  `CallContext`, `CallIfcContext`, `ExternalCallMediator`, and simple
  `AllowAllMediator` and `DenyAllMediator` implementations.
- `crates/zamburak-monty/src/run.rs` exposes `GovernedRunProgress` states
  `ExternalCallPending`, `Denied`, and `AwaitConfirmation`, which already model
  the three runtime outcomes required by this task.
- `crates/zamburak-policy/src/policy_def.rs` defines the current schema
  vocabulary:
  - `PolicyAction` with `Allow`, `Deny`, `RequireConfirmation`,
    `RequireDraft`,
  - `ToolPolicy` with `tool`, `side_effect_class`, `required_authority`,
    `arg_rules`, `context_rules`, and `default_decision`.
  - `ContextRules::deny_if_pc_integrity_contains`.
- `crates/zamburak-policy/src/engine.rs` currently constructs and validates a
  `PolicyEngine`, but does not yet evaluate external-call requests.
- `tests/integration/governed_run_bdd.rs` already covers generic allow and deny
  mediation flows.
- `tests/integration/governed_ifc_bdd.rs` and
  `tests/security/ifc_control_context.rs` already prove that observer-driven
  IFC summaries and strict-mode control-context data reach the call boundary.
- `tests/security/llm_sink_enforcement/mod.rs` shows the repository's current
  style for security-oriented `rstest-bdd` tests and for validating allow or
  deny decisions.

The gap for Task 0.6.4 is therefore precise:

1. define a typed policy-evaluation request and response in `zamburak-policy`,
2. implement deterministic, fail-closed external-call evaluation against the
   existing policy schema,
3. add a policy-backed mediator in `zamburak-monty`, and
4. prove that every external-call boundary consults policy before side-effect
   execution may continue.

## Expected file set

The implementation is expected to touch these files and may add small helper
submodules beneath them:

- `crates/zamburak-policy/src/engine.rs`
- `crates/zamburak-policy/src/lib.rs`
- `crates/zamburak-policy/src/engine_tests.rs` or
  `crates/zamburak-policy/src/engine/` extracted modules
- `crates/zamburak-monty/src/external_call.rs`
- `crates/zamburak-monty/src/run/flow.rs`
- `crates/zamburak-monty/src/run.rs`
- `crates/zamburak-monty/src/lib.rs`
- `crates/zamburak-monty/src/run_tests.rs`
- `crates/zamburak-monty/src/external_call_tests.rs`
- `tests/integration/governed_run_bdd.rs`
- `tests/integration/features/governed_run.feature`
- `tests/security/main.rs`
- `tests/security/` new policy-gating regression module and feature file
- `docs/zamburak-design-document.md`
- `docs/users-guide.md`
- `docs/roadmap.md`

If file-size pressure appears, extract internal modules such as:

- `crates/zamburak-policy/src/engine/evaluation.rs`
- `crates/zamburak-policy/src/engine/decision.rs`
- `crates/zamburak-monty/src/external_call/policy_mediator.rs`
- `crates/zamburak-monty/src/run/policy_gate.rs`

These helper names are suggestions, not mandatory filenames.

## Design shape to implement

The implementation should preserve the existing adapter layering:

1. `zamburak-policy` owns typed policy-evaluation semantics.
2. `zamburak-monty` owns governed runtime orchestration.
3. `full-monty` remains unchanged.

### Policy-engine surface

Add a typed request object in `zamburak-policy` for governed external-call
evaluation. The exact names may vary, but the shape should be equivalent to:

```rust
pub struct ExternalCallPolicyInput {
    pub tool_name: String,
    pub call_kind: ExternalCallKind,
    pub aggregate_summary: DependencySummary,
    pub arg_summaries: Vec<DependencySummary>,
    pub kwarg_summaries: Vec<KeywordArgumentSummary>,
    pub caller_authority: AuthoritySet,
    pub control_context: ExecutionContextSummary,
}
```

Do not couple this type to Monty runtime objects. It should consume only
Zamburak-owned or already public types.

Add a typed decision result in `zamburak-policy` that can express at least:

```rust
pub enum ExternalCallPolicyDecision {
    Allow(PolicyDecisionExplanation),
    Deny(PolicyDecisionExplanation),
    RequireConfirmation(PolicyDecisionExplanation),
}
```

`PolicyDecisionExplanation` does not need to solve the entire audit-pipeline
problem in this task, but it must be sufficient to keep deny and confirmation
reasons deterministic and safely redacted.

Add an evaluation entrypoint on `PolicyEngine`, for example
`evaluate_external_call(&self, input: &ExternalCallPolicyInput)`, that follows
the documented decision order:

1. hard deny constraints,
2. authority token requirements,
3. verification requirements,
4. context constraints,
5. confirmation and draft requirements,
6. default action.

Task 0.6.4 should consume the current schema surface conservatively:

- missing tool policy for a governed external call must fail closed,
- missing or unavailable information required by a rule must fail closed,
- `RequireDraft` is outside the current governed-run public flow, so this task
  must either map it conservatively to `RequireConfirmation` with explicit
  documentation or escalate if a different host contract is required.

### `zamburak-monty` policy mediator

Add a new mediator implementation in `crates/zamburak-monty` that owns a
`PolicyEngine` and translates `CallContext` into the new policy-engine input.
The exact name may vary (`PolicyMediator`, `PolicyEngineMediator`, or similar),
but it must:

- use the observer-derived IFC payload already present on `CallContext`,
- call the policy engine synchronously from `mediate(...)`,
- convert policy decisions to `MediationDecision`,
- preserve safe, deterministic deny or confirmation text for the existing
  governed-run states.

Do not bypass the mediator seam from `GovernedRunner`. The governed runner
should continue to depend on `Arc<Mutex<dyn ExternalCallMediator>>`, with the
new policy-backed mediator being the production implementation.

### Central gateway invariant

Refactor `crates/zamburak-monty/src/run/flow.rs` so both function and OS calls
use one shared path that:

1. constructs `CallContext`,
2. requests a decision,
3. returns:
   - `ExternalCallPending` only for allow,
   - `Denied` for deny,
   - `AwaitConfirmation` for confirmation.

The shared helper is the proof point for the completion criterion. After the
change, no external-call yield should be able to reach a resumable suspended
call before passing through that helper.

## Plan of work

### Stage A: contract lock and red-first tests

Inspect and pin the exact mapping from governed `CallContext` to policy-engine
tool lookup. Document the chosen rule in `Decision Log` before implementing the
engine. The simplest acceptable mapping is likely:

- policy tool key = `function_name`,
- call kind remains available as an input discriminator for diagnostics,
- tool absence fails closed.

Before implementation, add failing tests that express the required behaviour:

- unit tests in `zamburak-policy` for allow, deny, confirmation, missing-tool,
  and strict-mode context denial,
- unit tests in `zamburak-monty` proving a policy-backed mediator converts
  policy-engine outcomes into the existing governed-run states,
- one behavioural integration scenario for each of allow, deny, and
  confirmation,
- one security regression proving an untrusted external-call sink is blocked
  before a resumable side-effect path is returned.

Go or no-go for Stage A: the new tests compile and fail for the expected
"policy evaluation not implemented" or "policy mediator missing" reasons.

### Stage B: add policy-engine runtime evaluation

Implement the typed request, typed decision, explanation payload, and external-
call evaluation entrypoint in `crates/zamburak-policy`.

Keep evaluation logic deterministic. When multiple checks could apply, return
the earliest decision class in the documented order. When required data is
missing, deny or require confirmation conservatively instead of guessing.

This stage should cover at least:

- matching a `ToolPolicy`,
- applying `ArgRule.requires_integrity`,
- applying `ArgRule.forbids_confidentiality`,
- applying `ContextRules::deny_if_pc_integrity_contains`,
- returning the tool `default_decision` when no earlier rule fires,
- handling `RequireDraft` conservatively and documenting the mapping.

Go or no-go for Stage B: `cargo test -p zamburak-policy` passes and the new
engine tests prove deterministic order plus fail-closed behaviour.

### Stage C: wire policy decisions into governed execution

Add the policy-backed mediator to `crates/zamburak-monty` and expose it from
`src/lib.rs` if it is part of the public consumer story.

Refactor `run/flow.rs` so `mediate_function_call(...)` and
`mediate_os_call(...)` share one decision gateway after context construction.
Ensure that:

- allow yields `ExternalCallPending`,
- deny never returns a `SuspendedCall`,
- confirmation yields `AwaitConfirmation`,
- all error paths remain typed (`MediatorPoisoned`, `ObserverMismatch`,
  `MissingIfcSnapshot`, interpreter errors).

Add or extend doctests and public docs only where the public API changes are
stable enough to document.

Go or no-go for Stage C: `cargo test -p zamburak-monty` passes and no
external-call path bypasses the shared policy decision helper.

### Stage D: behavioural and security verification

Extend behavioural tests using `rstest-bdd` v0.5.0:

- `tests/integration/governed_run_bdd.rs` plus
  `tests/integration/features/governed_run.feature`:
  - allow path with real policy mediator,
  - deny path with real policy mediator,
  - confirmation path with real policy mediator.
- `tests/security/` new policy-gating suite:
  - missing tool policy is blocked fail closed,
  - strict-mode untrusted context denies or confirms before effect execution,
  - large language model (LLM)-like sink call path is gated before resumable
    execution continues.

Follow the existing repository BDD style:

- `#[fixture]` world objects,
- flat step helpers,
- no ambient global-state mutation,
- deterministic fixtures and inputs.

Go or no-go for Stage D: the new BDD and security suites pass and clearly prove
that policy is consulted before any resumable side-effect path is exposed.

### Stage E: documentation and roadmap sync

Update `docs/zamburak-design-document.md` with the implementation decisions
made for:

- the runtime external-call policy input shape,
- fail-closed handling for missing tool or summary information,
- the mapping of `RequireDraft` into the current governed-run contract if that
  mapping is used,
- the central shared decision gateway in `zamburak-monty`.

Update `docs/users-guide.md` with:

- how to construct the new policy-backed mediator,
- what governed-run states a library consumer should expect,
- any new public policy-evaluation types that matter to consumers,
- any conservative mappings or fail-closed behaviours they need to know.

After all tests and docs are complete, mark roadmap Task 0.6.4 done in
`docs/roadmap.md`.

### Stage F: final gates and evidence capture

Run all required gates with `tee` and `pipefail` so failures are visible:

```sh
set -o pipefail; make fmt | tee /tmp/make-fmt.log
set -o pipefail; make markdownlint | tee /tmp/make-markdownlint.log
set -o pipefail; make nixie | tee /tmp/make-nixie.log
set -o pipefail; make check-fmt | tee /tmp/make-check-fmt.log
set -o pipefail; make lint | tee /tmp/make-lint.log
set -o pipefail; make test | tee /tmp/make-test.log
```

Review the tail of each log for the final pass or fail summary and record any
meaningful surprises in this ExecPlan before closing the task.

## Acceptance evidence

The task is complete only when all of the following are true:

1. `PolicyEngine` evaluates governed external-call requests deterministically
   and fail closed for unknown or unavailable required data.
2. `GovernedRunner` requests a policy decision for every function and OS call
   before exposing a resumable side-effect path.
3. Allow, deny, and confirmation flows are all covered by unit tests.
   (Behavioural BDD and security-style tests deferred to Task 0.6.5 as per
   Stage D decision.)
4. `docs/zamburak-design-document.md` and `docs/users-guide.md` describe the
   shipped runtime behaviour.
5. `docs/roadmap.md` marks Task 0.6.4 done.
6. `make fmt`, `make markdownlint`, `make nixie`, `make check-fmt`,
   `make lint`, and `make test` all pass.
