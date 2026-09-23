# Zamburak engineering standards

This document defines engineering standards specific to Zamburak.

`AGENTS.md` remains normative for repository-wide process and quality rules.
Where this document is stricter, the stricter rule applies.

## Normative baseline

The baseline standards are defined in `AGENTS.md`, including:

- Rust quality gates, linting, formatting, and testing expectations,
- commit quality and atomicity requirements,
- error handling and testing discipline,
- documentation and Markdown rules.

This document adds Zamburak-specific requirements for security-critical work.

## Security-oriented design and implementation standards

### Information-flow correctness standards

- Keep integrity labels, confidentiality labels, and authority tokens as
  separate concepts and separate types.
- Treat any attempt to collapse these into one structure as a design defect.
- Require policy checks for every external side effect, including tool calls,
  large language model (LLM) calls, and commit operations.
- Include execution-context summaries in effect checks, not only argument
  summaries.

### Fail-closed standards

- Any overflow, timeout, or uncertainty in provenance analysis must fail closed
  by policy.
- Unknown-top summaries must not be auto-allowed.
- Missing label propagation for any opcode or built-in is release-blocking.

### Verification and endorsement standards

- Verification kinds must be deterministic and testable.
- Verification does not remove confidentiality labels.
- Declassification and endorsement require explicit policy support and, where
  configured, user confirmation.

### Tool and Model Context Protocol (MCP) trust standards

- Resolve tools only through a local pinned catalogue.
- Reject mutable remote tool documentation at runtime.
- Classify each MCP server by trust class and enforce per-server capability
  budgets.

### LLM sink governance standards

- Treat all LLM calls as external communication sinks.
- Enforce label budgets, redaction, and minimization before prompt emission.
- Keep local-only compatibility in interface contracts.
- Maintain explicit contract tests proving P-LLM and Q-LLM sink checks execute
  at the documented enforcement points.

## Audit and observability standards

- Log summaries, identifiers, and hashes by default; avoid raw payload logging.
- Apply label-aware redaction before persistence.
- Protect logs as sensitive assets with retention and size controls.
- Use tamper-evident chaining for append integrity where audit logs are
  enabled.

## Testing and verification evidence standards

### Required test categories

- mechanistic propagation and monotonicity tests,
- mutable-container and aliasing soundness tests,
- strict-mode control-context and side effect tests,
- provenance-budget overflow and fail-closed tests,
- regression tests for each discovered bypass,
- end-to-end LLM and tool sink policy enforcement tests.

### Evidence requirements

A change is not complete without evidence that:

- policy-gated boundaries still enforce invariants,
- new behaviour is covered by unit, integration, or security regression tests,
- existing regressions remain fixed.

## Documentation standards

- Keep `docs/zamburak-design-document.md` architecture-first and authoritative
  for semantics and invariants.
- Keep `docs/roadmap.md` focused on phases, steps, and measurable tasks.
- Keep `docs/repository-layout.md` current when module paths or ownership
  boundaries change.
- Keep `docs/tech-baseline.md` current when toolchain or gate tooling changes.
- Keep `docs/verification-targets.md` current when invariants, suites, or gate
  thresholds change.
- Use sentence-case headings, 80-column paragraph wrapping, and
  en-GB-oxendict spelling.
- Keep diagrams and tables captioned when present.

## Command and gateway standards

For documentation-only changes, run:

```sh
make markdownlint | tee /tmp/markdownlint-zamburak-$(git branch --show-current).out
make nixie | tee /tmp/nixie-zamburak-$(git branch --show-current).out
make fmt | tee /tmp/fmt-zamburak-$(git branch --show-current).out
```

For code-affecting changes, run repository quality gates from `AGENTS.md`
including:

```sh
make check-fmt | tee /tmp/check-fmt-zamburak-$(git branch --show-current).out
make lint | tee /tmp/lint-zamburak-$(git branch --show-current).out
make test | tee /tmp/test-zamburak-$(git branch --show-current).out
```

For script-affecting changes, run:

```sh
make script-baseline | tee /tmp/script-baseline-zamburak-$(git branch --show-current).out
make script-test | tee /tmp/script-test-zamburak-$(git branch --show-current).out
```

For phase-advancement pull requests, CI must also run:

```sh
make phase-gate | tee /tmp/phase-gate-zamburak-$(git branch --show-current).out
```

## CodeScene coverage publication

Main owns CodeScene. `.github/workflows/coverage-main.yml` is the one
publisher: on each push to `main` it measures coverage, writes the ratchet
baseline, and uploads the report. Pull requests measure coverage in `ci.yml`
and ratchet it against that baseline, but never contact CodeScene. The shared
uploader verifies the `cs-coverage` archive against a committed digest, so the
artefact is pinned. What cannot be pinned is the CodeScene API the tool calls,
and the tool refuses to run when that API's answers change shape. Moving the
call to `main` keeps such a change off every pull request's critical path.

- The pull-request lane sets `with-ratchet: 'true'` and
  `publish-artefact: 'false'`, and holds no CodeScene step, client, host, or
  credential. The rule covers every workflow a pull request can start,
  including local reusable workflows those workflows call.
- The publisher's check step runs one exact command,
  `echo "available=${{ secrets.CS_ACCESS_TOKEN != '' }}" >> "$GITHUB_OUTPUT"`.
  The upload step runs only when that output is `true` and the ref is
  `refs/heads/main`, and passes the secret straight to the uploader's
  `access-token` input. No `env` block holds the token, because the uploader is
  a composite action that hands its step's `env` to the nested steps it runs.
- The publisher's concurrency group is `coverage-main-${{ github.ref }}`,
  never cancelled. Runs for `main` never overlap, and the newest trigger
  survives any replacement, so triggered runs (push and dispatch) upload in
  commit order. A manual re-run of an older run keeps its original SHA; it is
  an operator action that republishes that commit's coverage and baseline until
  the next push supersedes it.
- Merges made by the Dependabot automerge workflow with `GITHUB_TOKEN` fire no
  push, so they reach the publisher only through a later push or a dispatch.
- A dispatch that replaces a pending push uploads the same or a newer commit.
  generate-coverage saves the baseline only on a push, so the baseline stays
  one commit behind until the next push.

`tests/workflow_contracts/codescene_repository_test.py` holds this shape over
the repository's own workflows, using the readers and rules in
`tests/workflow_contracts/codescene_contract/`. The other `codescene_*_test.py`
files prove that each rule refuses the shape it exists to refuse. Each case
starts from a compliant fixture tree and changes one thing. Workflows are read
strictly: a duplicate key, or a workflow declaring both a quoted and an unquoted
`on` key, is refused rather than silently resolved. Run the suite with
`make test-workflow-contracts`.

## Review and change-management standards

- Use atomic commits with descriptive rationale.
- Treat security model changes as high-risk and include threat-model impact in
  the change description.
- When semantics change, update design, roadmap, and standards documentation in
  the same change set.
