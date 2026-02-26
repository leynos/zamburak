# Zamburak technology baseline

This document defines the technology and tooling baseline for Zamburak.

Design semantics, interfaces, and invariants are defined in
`docs/zamburak-design-document.md`.

Implementation sequencing is defined in `docs/roadmap.md`.

Verification expectations are defined in `docs/verification-targets.md`.

## Context and orientation

Zamburak is a Rust-first security runtime for tool-using agent workflows. The
baseline in this document exists to keep implementation and verification
reproducible while the system design is hardened and executed.

The baseline is normative for:

- toolchain versions used in development and continuous integration (CI),
- quality-gate tools required before merge,
- documentation tooling required for design and roadmap artefacts.

## Canonical version baseline

| Component                    | Baseline                    | Source of truth                                 |
| ---------------------------- | --------------------------- | ----------------------------------------------- |
| Rust toolchain channel       | `nightly-2026-01-30`        | `rust-toolchain.toml`                           |
| Rust edition                 | `2024`                      | `Cargo.toml`                                    |
| Cargo lockfile discipline    | committed lockfile          | `Cargo.lock`                                    |
| Clippy warning policy        | warnings denied in gates    | `Makefile`, `Cargo.toml`, `clippy.toml`         |
| Rustdoc warning policy       | warnings denied in gates    | `Makefile`                                      |
| Markdown linting             | `markdownlint-cli2`         | `Makefile`, `.markdownlint-cli2.jsonc`          |
| Markdown formatting          | `mdformat-all`              | `Makefile`                                      |
| Mermaid validation           | `nixie`                     | `Makefile`                                      |
| Phase-gate verification      | `make phase-gate`           | `Makefile`, `.github/workflows/ci.yml`          |
| Script baseline verification | `verify_script_baseline.py` | `Makefile`, `scripts/verify_script_baseline.py` |
| Script launcher              | `uv 0.10.2`                 | `.github/workflows/ci.yml`                      |

_Table 1: Canonical technology and tooling baseline._

### Clippy threshold configuration

The `clippy.toml` file sets stricter-than-default thresholds aligned with the
CodeScene analysis ceiling. These thresholds apply workspace-wide.

| Threshold                        | Value  | Default | Rationale                       |
| -------------------------------- | ------ | ------- | ------------------------------- |
| `cognitive-complexity-threshold` | 9      | 25      | Aligned with CodeScene ceiling  |
| `too-many-arguments-threshold`   | 4      | 7       | Aligned with CodeScene ceiling  |
| `too-many-lines-threshold`       | 70     | 100     | Aligned with CodeScene ceiling  |
| `excessive-nesting-threshold`    | 4      | off     | Aligned with CodeScene ceiling  |
| `allow-expect-in-tests`          | `true` | `false` | Permits `expect()` in test code |

_Table 1a: Clippy threshold configuration (`clippy.toml`)._

## Required engineering tools and rationale

| Tool or target          | Why it is required                                                      |
| ----------------------- | ----------------------------------------------------------------------- |
| `make check-fmt`        | Verifies deterministic source formatting and prevents style drift.      |
| `make lint`             | Enforces strict Rust linting and warnings-as-errors quality discipline. |
| `make test`             | Validates behavioural and regression correctness across the workspace.  |
| `make markdownlint`     | Enforces documentation consistency and readability constraints.         |
| `make fmt`              | Normalizes Rust and Markdown formatting before review.                  |
| `make nixie`            | Validates Mermaid diagrams to prevent broken architecture renderings.   |
| `make phase-gate`       | Enforces phase-advancement verification suites in CI fail-closed mode.  |
| `make script-baseline`  | Validates script runtime metadata and command invocation contracts.     |
| `make script-typecheck` | Runs Python script type checks with `ty`.                               |
| `make script-test`      | Runs the script baseline test suite.                                    |

_Table 2: Required engineering tools and quality-gate rationale._

## Baseline usage contract

All repository changes must run the quality gates that match scope:

- documentation-only changes:
  `make markdownlint`, `make nixie`, and `make fmt`,
- code-affecting changes:
  `make check-fmt`, `make lint`, and `make test`.
- script-affecting changes:
  `make script-baseline`, `make script-typecheck`, and `make script-test`.
- phase-advancement changes:
  `make phase-gate` in CI must pass for the configured
  `.github/phase-gate-target.txt` target.

Command logging convention should follow `AGENTS.md`, using branch-qualified
paths such as:

`/tmp/<action>-zamburak-$(git branch --show-current).out`.

## Baseline update policy

Baseline updates must be explicit and reviewable.

- Update this document when any toolchain version, quality gate, or baseline
  tool changes.
- Include rationale for each change, especially where stricter or looser
  checks affect security confidence.
- Keep `rust-toolchain.toml`, `Makefile`, and this document in sync within the
  same change set.
- If a baseline update weakens enforcement, include threat-model impact and
  mitigation in the change description.

## Drift detection expectations

Drift from this baseline is a process defect and should be treated as blocking.
Drift signals include:

- continuous integration runs using toolchain versions different from this
  baseline without an approved update,
- documentation commands that pass locally but fail in CI because tooling
  versions differ,
- quality-gate commands omitted from merge-critical pull requests.

When drift is detected, restore alignment before accepting additional
implementation work.
