# Align toolchain and quality-gate baseline with repository configuration (Task 0.2.1)

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

`PLANS.md` is not present in this repository at draft time, so this document is
the governing execution plan for this task.

## Purpose / big picture

Implement roadmap Task 0.2.1 from `docs/roadmap.md`: align toolchain and
quality-gate baseline with repository configuration.

After this change, every version pin, quality-gate command, and engineering
tool referenced in documentation will match the actual repository configuration
files, and vice versa. A contributor reading any one of
`docs/tech-baseline.md`, `AGENTS.md`, `docs/zamburak-engineering-standards.md`,
or `docs/repository-layout.md` will find commands and versions that correspond
exactly to what `rust-toolchain.toml`, `Makefile`, `Cargo.toml`, and
`clippy.toml` actually do.

Task completion is observable when `make check-fmt`, `make lint`, and
`make test` pass, and a manual audit of configuration-to-documentation
consistency reveals no remaining drift.

## Constraints

- Implement to these normative signposts:
  `docs/tech-baseline.md` sections "Canonical version baseline" and "Baseline
  usage contract"; `docs/zamburak-engineering-standards.md` section "Command
  and gateway standards"; `docs/repository-layout.md` section "Root and
  operational files".
- In scope: `rust-toolchain.toml`, `Makefile`, and baseline-document
  consistency.
- Out of scope: introducing additional build systems.
- The tech-baseline update policy (line 78) requires that
  `rust-toolchain.toml`, `Makefile`, and the baseline document stay in sync
  "within the same change set". All changes in this plan must be committed
  together.
- No new external dependencies may be introduced.
- No changes to public library API signatures.
- Required completion gates: `make check-fmt`, `make lint`, `make test`.
- Because this task updates Markdown documentation, also run:
  `make markdownlint`, `make nixie`, and `make fmt`.
- Update `docs/users-guide.md` only if library-consumer-visible behaviour or
  API changes; if no such change exists, document that determination in this
  ExecPlan.
- Mark roadmap Task 0.2.1 as done in `docs/roadmap.md` only after all quality
  and documentation gates pass.

## Tolerances (exception triggers)

- Scope tolerance:
  if implementation requires edits in more than 10 files or 500 net changed
  lines, stop and escalate with a split proposal.
- Interface tolerance:
  if fulfilling the task requires changing existing public library API
  signatures, stop and escalate with compatibility options.
- Dependency tolerance:
  if a new third-party dependency is required, stop and escalate before adding
  it.
- Behavioural-test tolerance:
  if any Rust code change introduced by this task warrants `rstest-bdd`
  scenarios but they cannot be expressed after two concrete attempts, stop and
  document why before using a non-BDD fallback.
- Iteration tolerance:
  if required gates still fail after three focused fix loops, stop and report
  failing suites with root-cause hypotheses.

## Risks

- Risk: the `RUSTDOC_FLAGS` variable is used in the Makefile `lint` target but
  never defined, meaning `cargo doc` runs without `-D warnings` and broken
  rustdoc links or warnings pass silently. Severity: high Likelihood: confirmed
  (variable is absent from the Makefile). Mitigation: define
  `RUSTDOC_FLAGS ?= -D warnings` in the Makefile.

- Risk: the Makefile `lint` target runs `cargo doc --no-deps` and
  `cargo clippy` without `--workspace`, meaning workspace member crates
  (`zamburak-core`, `zamburak-policy`) may not be linted or have their docs
  checked. Severity: high Likelihood: confirmed (the `--workspace` flag is
  absent from both commands in the `lint` target). Mitigation: add
  `--workspace` to both commands.

- Risk: `AGENTS.md` documents command expansions that do not match the actual
  Makefile commands, causing contributors to have incorrect expectations about
  what quality gates enforce. Severity: medium Likelihood: confirmed (four
  specific discrepancies identified). Mitigation: update `AGENTS.md`
  descriptions to match actual Makefile behaviour.

- Risk: `docs/tech-baseline.md` omits the `script-baseline` and `script-test`
  targets added in Task 0.2.3, creating baseline-document drift on the same
  branch. Severity: medium Likelihood: confirmed. Mitigation: add the missing
  entries to tech-baseline tables.

- Risk: `docs/repository-layout.md` Table 14 (root files) omits `Makefile`,
  `AGENTS.md`, `clippy.toml`, and other operational files, while Table 13
  (docs/) omits several documentation files that exist on disk. Severity:
  medium Likelihood: confirmed. Mitigation: reconcile both tables with actual
  repository contents.

## Progress

- [x] (2026-02-22 00:00Z) Reviewed roadmap Task 0.2.1 requirements, scope, and
  traceability signposts.
- [x] (2026-02-22 00:01Z) Audited all configuration files
      (`rust-toolchain.toml`,
  `Makefile`, `Cargo.toml`, `clippy.toml`, `.github/workflows/ci.yml`) against
  all documentation files (`AGENTS.md`, `docs/tech-baseline.md`,
  `docs/zamburak-engineering-standards.md`, `docs/repository-layout.md`).
- [x] (2026-02-22 00:02Z) Identified 12 concrete inconsistencies across 5 files.
- [x] (2026-02-22 00:03Z) Drafted this ExecPlan with constraints, tolerances,
  staged delivery, and validation criteria.
- [x] (2026-02-22 00:10Z) Stage A: fixed Makefile defects (defined
  `RUSTDOC_FLAGS`, added `--workspace` to `lint` target). Also fixed renamed
  `missing_crate_level_docs` lint in `Cargo.toml` and refactored a
  `zamburak-core` test to reduce parameter count below the threshold.
- [x] (2026-02-22 00:15Z) Stage B: updated `AGENTS.md` command descriptions
  to match actual Makefile behaviour.
- [x] (2026-02-22 00:17Z) Stage C: updated `docs/tech-baseline.md` with
  missing entries and `clippy.toml` threshold documentation.
- [x] (2026-02-22 00:18Z) Stage D: updated
  `docs/zamburak-engineering-standards.md` with script-affecting command scope.
- [x] (2026-02-22 00:20Z) Stage E: reconciled `docs/repository-layout.md`
  with actual repository contents (Tables 10, 11, 12, 13, 14).
- [x] (2026-02-22 00:25Z) Stage F: all eight quality gates passed.
- [x] (2026-02-22 00:27Z) Stage G: marked roadmap Task 0.2.1 done and
  finalised this ExecPlan.

## Surprises & discoveries

- Observation: adding `--workspace` to the `lint` target exposed a
  `too_many_arguments` clippy lint in
  `crates/zamburak-core/src/authority/tests.rs` function
  `delegation_rejects_invalid_timing`. The original code used
  `#[expect(clippy::too_many_arguments)]` which worked when clippy ran only on
  the root crate, but failed when running with `--workspace` because the
  `-D warnings` CLI flag overrides per-item `#[expect]` and `#[allow]`
  attributes in `rstest` macro expansions. Evidence: `make lint` exited with
  error on the first run after adding `--workspace`. Impact: refactored the
  test to pass `DelegationTiming` as a single case parameter instead of two
  separate `u64` parameters, reducing the parameter count from 5 to 4.

- Observation: the `missing_crate_level_docs` lint in `Cargo.toml`
  `[lints.rust]` has been renamed to `rustdoc::missing_crate_level_docs` in
  nightly Rust. Evidence: `cargo doc` emitted a warning about the renamed lint.
  Impact: moved the lint from `[lints.rust]` to `[lints.rustdoc]` to suppress
  the renamed-lint warning.

- Observation: `docs/repository-layout.md` had more discrepancies than
  initially identified. The `policies/`, `fuzz/`, and `tests/` tables also
  contained entries for items that do not yet exist on disk. Evidence:
  directory listing showed `policies/strict.yaml`, `policies/examples/`,
  `fuzz/`, and several `tests/` subdirectories are absent. Impact: annotated
  these as "(planned)" consistently across all tables.

## Decision log

- Decision: this task is primarily a documentation-and-configuration
  synchronisation effort, not a feature implementation. Rationale: the
  completion criterion is "baseline versions and gate commands are consistent
  across config and documentation", which means aligning existing files rather
  than introducing new behaviour. Date/Author: 2026-02-22 / Codex.

- Decision: fix the Makefile `RUSTDOC_FLAGS` omission and missing `--workspace`
  flags as part of this task rather than filing a separate bug. Rationale:
  these are configuration-documentation drift issues squarely within the task
  scope ("gate commands are consistent across config and documentation"), and
  the tech-baseline update policy requires config and docs to be updated in the
  same change set. Date/Author: 2026-02-22 / Codex.

- Decision: no unit tests or `rstest-bdd` behavioural tests are required for
  this task. Rationale: the changes are to Makefile variable definitions,
  documentation prose, and documentation tables. No new Rust library code,
  functions, or behaviour is being introduced. The existing test suite
  (`make test`) validates that the Makefile changes do not break compilation or
  tests, and `make lint` validates the corrected lint target. The quality gates
  themselves serve as the verification mechanism. Date/Author: 2026-02-22 /
  Codex.

- Decision: move the `missing_crate_level_docs` lint from `[lints.rust]` to
  `[lints.rustdoc]` in `Cargo.toml`. Rationale: the lint has been renamed to
  `rustdoc::missing_crate_level_docs` in nightly Rust. Placing it in the
  correct section eliminates the `renamed_and_removed_lints` warning that would
  otherwise appear during `cargo doc`. Date/Author: 2026-02-22 / Codex.

- Decision: refactor `delegation_rejects_invalid_timing` test to use
  `DelegationTiming` struct as a single case parameter instead of two `u64`
  parameters. Rationale: this reduces the function parameter count from 5 to 4,
  satisfying the `too-many-arguments-threshold = 4` in `clippy.toml` without
  needing lint suppression attributes that do not propagate through `rstest`
  macro expansion. Date/Author: 2026-02-22 / Codex.

- Decision: annotate planned (not-yet-existing) items in
  `docs/repository-layout.md` tables with "(planned)" rather than removing
  them. Rationale: the document describes itself as a "proposed" layout and
  these entries represent design intent for future phases. Annotation preserves
  the roadmap context while making clear what exists today. Date/Author:
  2026-02-22 / Codex.

- Decision: do not update `docs/users-guide.md` for Task 0.2.1. Rationale:
  this task changes build tooling configuration and documentation
  synchronisation, not public Rust API behaviour consumed by library users.
  Date/Author: 2026-02-22 / Codex.

## Outcomes & retrospective

Task 0.2.1 implementation is complete.

Delivered outcomes:

- defined `RUSTDOC_FLAGS ?= -D warnings` in `Makefile` to enforce rustdoc
  warning policy,
- added `--workspace` to both `cargo doc` and `cargo clippy` in the `lint`
  target to cover all workspace members,
- moved `missing_crate_level_docs` from `[lints.rust]` to `[lints.rustdoc]`
  in `Cargo.toml` to address the renamed lint,
- refactored `delegation_rejects_invalid_timing` test in `zamburak-core` to
  reduce parameter count from 5 to 4 using `DelegationTiming` struct,
- corrected `AGENTS.md` command descriptions for `make check-fmt`,
  `make lint`, `make test`, and `make fmt` to match actual Makefile commands,
- added script baseline, script launcher, and rustdoc warning policy entries
  to `docs/tech-baseline.md` Table 1,
- added `make script-baseline` and `make script-test` to
  `docs/tech-baseline.md` Table 2,
- added script-affecting scope to `docs/tech-baseline.md` baseline usage
  contract,
- added `clippy.toml` threshold documentation to `docs/tech-baseline.md`,
- added script-affecting command block to
  `docs/zamburak-engineering-standards.md`,
- reconciled `docs/repository-layout.md` Tables 10, 11, 12, 13, and 14 with
  actual repository file inventory, annotating planned items,
- marked roadmap Task 0.2.1 as done in `docs/roadmap.md`,
- preserved `docs/users-guide.md` unchanged, as there are no library
  consumer-visible API or behaviour changes.

Gate outcomes:

- passed: `make check-fmt`,
- passed: `make lint`,
- passed: `make test` (71 tests),
- passed: `make markdownlint` (29 files),
- passed: `make nixie`,
- passed: `make script-baseline`,
- passed: `make script-test` (28 tests).

Retrospective:

- Adding `--workspace` to the lint target exposed pre-existing clippy issues
  in workspace member crates that were silently passing. This validates the
  importance of this alignment task.
- The `rstest` macro and `#[expect]`/`#[allow]` attribute interaction with
  CLI `-D warnings` is a known pain point. Refactoring to reduce parameter
  counts is a cleaner long-term solution than lint suppression.
- Annotating planned vs existing items in `docs/repository-layout.md` tables
  improves onboarding clarity by distinguishing aspiration from reality.

## Context and orientation

This section describes the current state of the repository files relevant to
Task 0.2.1 and the specific inconsistencies that must be resolved.

### Key files

- `rust-toolchain.toml` — pins the Rust toolchain to `nightly-2026-01-30` with
  `rustfmt` and `clippy` components. Currently consistent with
  `docs/tech-baseline.md`.
- `Cargo.toml` — workspace root defining edition `2024`, resolver `3`, and
  workspace members `crates/zamburak-core` and `crates/zamburak-policy`.
  Contains 41 Clippy deny rules and 2 Rust deny rules.
- `clippy.toml` — sets `cognitive-complexity-threshold = 9`,
  `too-many-arguments-threshold = 4`, `too-many-lines-threshold = 70`,
  `excessive-nesting-threshold = 4`, and `allow-expect-in-tests = true`.
- `Makefile` — defines quality-gate targets including `check-fmt`, `lint`,
  `test`, `markdownlint`, `fmt`, `nixie`, `phase-gate`, `script-baseline`, and
  `script-test`.
- `AGENTS.md` — repository-wide process and quality rules, including
  descriptions of what each `make` target expands to.
- `docs/tech-baseline.md` — canonical version baseline and quality-gate
  documentation.
- `docs/zamburak-engineering-standards.md` — project-specific command and
  gateway standards.
- `docs/repository-layout.md` — proposed repository structure and file-purpose
  reference.
- `.github/workflows/ci.yml` — CI workflow with `build-test` and `phase-gate`
  jobs.

### Identified inconsistencies

The following inconsistencies have been confirmed by reading the actual files:

A. Makefile defects (config is wrong):

A1. `RUSTDOC_FLAGS` is referenced in the `lint` target (line 35:
`RUSTDOCFLAGS="$(RUSTDOC_FLAGS)"`) but never defined. This means `cargo doc`
runs with an empty `RUSTDOCFLAGS` environment variable, so rustdoc warnings are
not treated as errors. The tech-baseline states "warnings denied in gates" for
Clippy, but rustdoc warnings silently pass.

A2. The `lint` target runs `cargo doc --no-deps` without `--workspace` (line
35). Only the root crate's documentation is checked, not workspace members
`zamburak-core` and `zamburak-policy`.

A3. The `lint` target runs `cargo clippy` without `--workspace` (line 36). The
`CLIPPY_FLAGS` variable expands to
`--all-targets --all-features -- -D warnings` but does not include
`--workspace`. However, `AGENTS.md` line 131 documents this as
`cargo clippy --workspace --all-targets --all-features -- -D warnings`. The
Makefile is wrong; `AGENTS.md` is correct on this point.

B. `AGENTS.md` discrepancies (docs are wrong):

B1. Line 124: documents `make check-fmt` as `cargo fmt --workspace -- --check`,
but the actual Makefile command is `cargo fmt --all -- --check`. The `--all`
flag formats all workspace members including path dependencies; `--workspace`
is not a valid `cargo fmt` flag (it uses `--all` instead).

B2. Lines 136-143: documents `make test` as `cargo test --workspace`, but the
actual Makefile command is
`RUSTFLAGS="-D warnings" cargo test --workspace --all-targets --all-features`.
The documentation omits the `RUSTFLAGS`, the `--all-targets` flag, and the
`--all-features` flag.

B3. Lines 128-134: documents `make lint` as only
`cargo clippy --workspace --all-targets --all-features -- -D warnings`, but the
actual Makefile target also runs
`RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" cargo doc --no-deps` first. The documentation
omits the `cargo doc` step entirely.

B4. Line 143: documents `make fmt` as `cargo fmt --workspace`, but the actual
Makefile command runs `cargo fmt --all` followed by `mdformat-all`. The
documentation omits the `--all` correction and the Markdown formatting step.

C. `docs/tech-baseline.md` gaps (docs are incomplete):

C1. Table 1 (canonical version baseline) and Table 2 (required engineering
tools) do not include the `script-baseline` and `script-test` targets added in
Task 0.2.3. The baseline usage contract section also omits a script-affecting
scope.

C2. Table 1 references `clippy.toml` as a source of truth for "Clippy warning
policy" but the document never specifies what thresholds `clippy.toml` sets.

C3. The document does not mention the rustdoc warning policy. The `lint` target
runs `cargo doc` but the baseline does not document this as a gate component.

C4. Table 1 does not mention the `uv` version used for script execution
(`0.10.2`, pinned in `.github/workflows/ci.yml`).

D. `docs/zamburak-engineering-standards.md` gap:

D1. The "Command and gateway standards" section lists three scopes
(documentation-only, code-affecting, phase-advancement) but omits the
script-affecting scope added by Task 0.2.3 (`make script-baseline` and
`make script-test`).

E. `docs/repository-layout.md` gaps:

E1. Table 14 (root and operational files) omits `Makefile`, `AGENTS.md`,
`clippy.toml`, `.gitignore`, `.markdownlint-cli2.jsonc`, and `codecov.yml`.

E2. Table 13 (docs/) omits several files that exist on disk, including
`docs/users-guide.md`, `docs/contents.md`, `docs/scripting-standards.md`,
`docs/documentation-style-guide.md`, `docs/policy-examples.md`,
`docs/policy-examples-financial-services-scenarios.md`,
`docs/complexity-antipatterns-and-refactoring-strategies.md`,
`docs/reliable-testing-in-rust-via-dependency-injection.md`,
`docs/rstest-bdd-users-guide.md`, `docs/rust-doctest-dry-guide.md`,
`docs/rust-testing-with-rstest-fixtures.md`,
`docs/localizable-rust-libraries-with-fluent.md`,
`docs/adr-002-localization-and-internationalization-with-fluent.md`, and the
`docs/execplans/` directory. It also lists `docs/monty-fork-policy.md` which
does not exist on disk.

E3. Table 14 lists `.env.example` and `third_party/full-monty/` which do not
exist on disk. Since `docs/repository-layout.md` describes itself as a
"proposed" layout, these should be annotated as "(planned)" to distinguish
intent from current state.

## Plan of work

Stage A: fix Makefile defects.

Edit `Makefile` to resolve the three confirmed configuration defects:

1. Add `RUSTDOC_FLAGS ?= -D warnings` as a variable definition after the
   `RUST_FLAGS` definition on line 8. This ensures rustdoc warnings are treated
   as errors in the `lint` target, matching the tech-baseline's "warnings
   denied in gates" policy.

2. Add `--workspace` to the `cargo doc` invocation in the `lint` target,
   changing `$(CARGO) doc --no-deps` to `$(CARGO) doc --workspace --no-deps`.

3. Add `--workspace` to the `cargo clippy` invocation in the `lint` target,
   changing `$(CARGO) clippy $(CLIPPY_FLAGS)` to
   `$(CARGO) clippy --workspace $(CLIPPY_FLAGS)`.

Go/no-go for Stage A: `make lint` runs successfully with the corrected
commands, linting all workspace members and treating doc warnings as errors.

Stage B: update `AGENTS.md` command descriptions.

Edit `AGENTS.md` to correct the four documented command expansions so they
match the actual (post-Stage-A) Makefile behaviour:

1. Line 124: change `cargo fmt --workspace -- --check` to
   `cargo fmt --all -- --check`.

2. Lines 128-135: replace the `make lint` expansion with both steps:
   `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` followed by
   `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

3. Lines 136-143: replace the `make test` expansion with
   `RUSTFLAGS="-D warnings" cargo test --workspace --all-targets --all-features`.

4. Lines 142-143: replace the `make fmt` expansion with `cargo fmt --all`
   followed by `mdformat-all`.

Go/no-go for Stage B: `make markdownlint` passes on the updated `AGENTS.md`.

Stage C: update `docs/tech-baseline.md`.

Edit `docs/tech-baseline.md` to add missing entries and document `clippy.toml`
configuration:

1. Add three rows to Table 1 (canonical version baseline):
   - Script baseline verification: `verify_script_baseline.py`, source of
     truth `Makefile`, `scripts/verify_script_baseline.py`.
   - Script launcher: `uv 0.10.2`, source of truth
     `.github/workflows/ci.yml`.
   - Rustdoc warning policy: warnings denied in gates, source of truth
     `Makefile`.

2. Add two rows to Table 2 (required engineering tools):
   - `make script-baseline`: validates script runtime metadata and command
     invocation contracts.
   - `make script-test`: runs the script baseline test suite.

3. Add a fourth scope to the baseline usage contract section:
   script-affecting changes require `make script-baseline` and
   `make script-test`.

4. Add a new subsection after Table 1 documenting the `clippy.toml` threshold
   configuration with a table showing each threshold, its value, the default,
   and the rationale (alignment with CodeScene ceiling).

Go/no-go for Stage C: `make markdownlint` and `make fmt` pass on the updated
file.

Stage D: update `docs/zamburak-engineering-standards.md`.

Add a fourth command block to the "Command and gateway standards" section
(after the phase-gate block, before "Review and change-management standards"):

    For script-affecting changes, run:

        make script-baseline | tee /tmp/script-baseline-zamburak-$(git branch --show-current).out
        make script-test | tee /tmp/script-test-zamburak-$(git branch --show-current).out

Go/no-go for Stage D: `make markdownlint` passes on the updated file.

Stage E: reconcile `docs/repository-layout.md`.

1. Table 14 (root and operational files): add rows for `Makefile`, `AGENTS.md`,
   `clippy.toml`, `.gitignore`, `.markdownlint-cli2.jsonc`, and `codecov.yml`
   with appropriate purpose descriptions. Annotate `.env.example` and
   `third_party/full-monty/` as "(planned)" since they do not yet exist.

2. Table 13 (docs/): remove the `docs/monty-fork-policy.md` row (file does not
   exist). Add rows for all documentation files that exist on disk but are not
   listed. Add a row for the `docs/execplans/` directory.

3. Table 11 (tests/): add a row for `tests/test_utils/` which exists on disk.
   Annotate `tests/integration/`, `tests/property/`, and `tests/benchmarks/` as
   "(planned)" since they do not yet exist.

Go/no-go for Stage E: `make markdownlint` and `make fmt` pass on the updated
file.

Stage F: run all quality gates and capture logs.

Run the full gate suite to confirm all changes are clean:

    set -o pipefail && make check-fmt 2>&1 | tee /tmp/check-fmt-zamburak-$(git branch --show-current).out
    set -o pipefail && make lint 2>&1 | tee /tmp/lint-zamburak-$(git branch --show-current).out
    set -o pipefail && make test 2>&1 | tee /tmp/test-zamburak-$(git branch --show-current).out
    set -o pipefail && make markdownlint 2>&1 | tee /tmp/markdownlint-zamburak-$(git branch --show-current).out
    set -o pipefail && make nixie 2>&1 | tee /tmp/nixie-zamburak-$(git branch --show-current).out
    set -o pipefail && make fmt 2>&1 | tee /tmp/fmt-zamburak-$(git branch --show-current).out
    set -o pipefail && make script-baseline 2>&1 | tee /tmp/script-baseline-zamburak-$(git branch --show-current).out
    set -o pipefail && make script-test 2>&1 | tee /tmp/script-test-zamburak-$(git branch --show-current).out

Go/no-go for Stage F: all eight commands exit zero.

Stage G: roadmap closure and ExecPlan finalisation.

1. Mark Task 0.2.1 as done in `docs/roadmap.md` by changing `- [ ]` to
   `- [x]` on line 81.
2. Update this ExecPlan: fill in `Progress` timestamps, `Outcomes &
   Retrospective`, and change status from `DRAFT` to `COMPLETE`.

## Concrete steps

Run all commands from repository root: `/home/user/project`.

1. Fix Makefile defects.

   Edit `Makefile`:
   - After line 8 (`RUST_FLAGS ?= -D warnings`), add:
     `RUSTDOC_FLAGS ?= -D warnings`
   - Line 35: change `$(CARGO) doc --no-deps` to
     `$(CARGO) doc --workspace --no-deps`
   - Line 36: change `$(CARGO) clippy $(CLIPPY_FLAGS)` to
     `$(CARGO) clippy --workspace $(CLIPPY_FLAGS)`

   Validate:

       make lint 2>&1 | tail -5

   Expected: exit zero with clippy and doc warnings passing for all workspace
   members.

2. Update `AGENTS.md` command descriptions.

   Edit lines 121-143 to match actual Makefile commands as described in Stage B.

   Validate:

       make markdownlint

3. Update `docs/tech-baseline.md`.

   Add missing table rows, clippy threshold subsection, and script-affecting
   scope as described in Stage C.

   Validate:

       make markdownlint && make fmt

4. Update `docs/zamburak-engineering-standards.md`.

   Add script-affecting command block as described in Stage D.

   Validate:

       make markdownlint

5. Reconcile `docs/repository-layout.md`.

   Update Tables 11, 13, and 14 as described in Stage E.

   Validate:

       make markdownlint && make fmt

6. Run full quality-gate suite.

       set -o pipefail && make check-fmt 2>&1 | tee /tmp/check-fmt-zamburak-$(git branch --show-current).out
       set -o pipefail && make lint 2>&1 | tee /tmp/lint-zamburak-$(git branch --show-current).out
       set -o pipefail && make test 2>&1 | tee /tmp/test-zamburak-$(git branch --show-current).out
       set -o pipefail && make markdownlint 2>&1 | tee /tmp/markdownlint-zamburak-$(git branch --show-current).out
       set -o pipefail && make nixie 2>&1 | tee /tmp/nixie-zamburak-$(git branch --show-current).out
       set -o pipefail && make fmt 2>&1 | tee /tmp/fmt-zamburak-$(git branch --show-current).out
       set -o pipefail && make script-baseline 2>&1 | tee /tmp/script-baseline-zamburak-$(git branch --show-current).out
       set -o pipefail && make script-test 2>&1 | tee /tmp/script-test-zamburak-$(git branch --show-current).out

7. Mark roadmap task done and update this ExecPlan.

   Edit `docs/roadmap.md` line 81: change `- [ ]` to `- [x]`.

## Validation and acceptance

Acceptance criteria for Task 0.2.1:

- The `Makefile` `lint` target runs `cargo doc --workspace --no-deps` with
  `RUSTDOCFLAGS="-D warnings"` and `cargo clippy --workspace` with warnings
  denied. Observable by reading `Makefile` and by `make lint` exiting zero.
- `AGENTS.md` command descriptions for `make check-fmt`, `make lint`,
  `make test`, and `make fmt` match the actual Makefile commands verbatim.
  Observable by comparing the two files.
- `docs/tech-baseline.md` Table 1 includes entries for script baseline, script
  launcher, and rustdoc warning policy. Table 2 includes `make script-baseline`
  and `make script-test`. The baseline usage contract includes script-affecting
  scope. A new subsection documents `clippy.toml` thresholds.
- `docs/zamburak-engineering-standards.md` "Command and gateway standards"
  includes a script-affecting command block.
- `docs/repository-layout.md` Tables 11, 13, and 14 match the actual
  repository file inventory with planned items annotated.
- `docs/roadmap.md` marks Task 0.2.1 as done.
- All eight quality gates exit zero: `make check-fmt`, `make lint`,
  `make test`, `make markdownlint`, `make nixie`, `make fmt`,
  `make script-baseline`, `make script-test`.

## Idempotence and recovery

- All Makefile changes are idempotent variable definitions and command flags.
- All documentation changes are prose and table edits.
- If any gate fails after edits, fix only the failing cause, rerun that gate,
  then rerun the full required sequence.
- If `make lint` fails after adding `--workspace`, it likely means a workspace
  member has warnings that were previously invisible. Fix those warnings in the
  source code before proceeding.

## Artefacts and notes

Evidence to capture during implementation:

- gate logs in `/tmp/*-zamburak-<branch>.out`,
- final `git diff` showing all edits across the change set,
- criterion-to-evidence mapping in outcomes section.

## Interfaces and dependencies

This task introduces no new Rust library code, functions, traits, or APIs. The
interfaces are:

- `Makefile` variable `RUSTDOC_FLAGS` (new, defaulting to `-D warnings`).
- `Makefile` `lint` target with corrected `--workspace` flag on both `cargo
  doc` and `cargo clippy` invocations.
- `Cargo.toml` `[lints.rustdoc]` section (new, migrated from
  `[lints.rust]` for the renamed `missing_crate_level_docs` lint).
- No changes to `rust-toolchain.toml` (already consistent).
- No changes to `clippy.toml` (already consistent).
- No changes to `.github/workflows/ci.yml` (already consistent).

No new dependencies are required.

Revision note (2026-02-22):

- Updated plan state from `DRAFT` to `COMPLETE`.
- Recorded implemented files, decisions, and validation evidence.
- Marked all execution progress items complete.
- Added surprises for `--workspace` lint exposure, `missing_crate_level_docs`
  rename, and additional repository-layout table discrepancies.
- Added decisions for `Cargo.toml` lint migration, test parameter refactoring,
  and planned-item annotation strategy.
- Updated interfaces section to reflect the `Cargo.toml` change.
- Documented the no-change decision for `docs/users-guide.md`.
