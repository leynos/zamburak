# Zamburak developers' guide

Practical guidance for contributors working on Zamburak: how the project's
tooling, workflows, and checks fit together.

## Mutation-testing workflow contract tests

This repository runs scheduled, informational mutation testing through a thin
caller workflow,
[`.github/workflows/mutation-testing.yml`](../.github/workflows/mutation-testing.yml),
which delegates to the shared reusable workflow
`leynos/shared-actions/.github/workflows/mutation-cargo.yml`. The heavy
lifting — running `cargo-mutants`, sharding, and summarizing survivors —
lives in `shared-actions`; this repository carries only declarative
configuration. The run is **informational only**: it never gates a pull
request. Survivors are reported through the job summary and downloadable
artefacts so they can be triaged into tests, not enforced as a blocking
check.

The workflow runs in two modes. A **daily schedule** fires a change-scoped
run that mutates only the source files touched within the detection window,
so quiet days are cheap no-ops. A **manual dispatch** (the Actions "Run
workflow" control) mutates the whole workspace, fanned out across shards;
select a branch in that control to exercise a feature branch.

The caller passes a small set of configuration inputs, each carrying intent:

- `paths` — the change-detection globs (`src/,crates/,examples/,benches/`)
  that decide whether a scheduled run has anything to mutate, bounding the
  scheduled run to real source changes across the root crate and workspace
  members.
- `exclude-globs` — `crates/test-utils/**`, the test-fixture crate whose
  surviving mutants are noise rather than genuine test gaps; in-source
  `*_tests.rs` and `test_helpers.rs` modules are already `#[cfg(test)]`-gated,
  so `cargo-mutants` skips them without needing an exclude.
- `extra-args` — `--all-features --test-workspace=true`, so the mutation run
  matches the CI test baseline (a mismatch would report feature-gated code as
  untested) and every workspace member is mutated against the full workspace
  test run, rather than only the mutated package's own tests.

The `uses:` reference pins the shared workflow to a full 40-character commit
SHA rather than a branch or tag, so a force-push upstream cannot silently
change what runs here. The contract test asserts only that the pin is a full
commit SHA, not a particular value, so Dependabot bumps it automatically
without any accompanying test edit.

Because the caller is configuration rather than code, a contract test pins
the shape it must uphold, failing the pull request when the caller drifts —
repointing the pin at a branch, widening the token scope, or dropping a
configuration input — rather than letting the breakage surface only in a
scheduled run. Run it locally with `make test-workflow-contracts`. The test
validates:

- the `uses:` reference targets `mutation-cargo.yml` pinned to a full commit
  SHA;
- the `with:` block carries exactly the expected configuration (the paths,
  the test-utils exclude, and the feature arguments above);
- job permissions are least-privilege (`contents: read`, `id-token: write`)
  and the workflow-level default token scope is empty;
- `concurrency` serializes runs per ref without cancelling one in progress;
  and
- the triggers keep the daily schedule and a plain `workflow_dispatch` with
  no legacy branch input.
