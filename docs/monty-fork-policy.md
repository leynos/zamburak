# `full-monty` fork policy

This document defines the governance contract for Track A changes in
`third_party/full-monty/`.

The normative architecture split is defined in
[ADR 001: Monty IFC VM hooks](adr-001-monty-ifc-vm-hooks.md) and the
[Zamburak design document](zamburak-design-document.md).

## Scope and intent

`full-monty` is a constrained fork used as a generic runtime substrate. Changes
in this fork must stay upstream-shaped and must not encode Zamburak policy
semantics.

## Allowed change categories

Only these categories are permitted in fork deltas:

- stable, host-only runtime IDs,
- generic observer and event hook substrate,
- optional generic snapshot-extension seam,
- narrowly necessary refactors that directly enable the above.

Any change outside these categories is rejected by policy.

## Forbidden semantic classes

Fork API surface must not introduce Zamburak semantics or nomenclature.
Rejected examples include:

- `zamburak` names,
- `taint` semantics,
- `policy` semantics,
- capability semantics in Track A API naming.

This prohibition applies to new public API lines and API doc-comment lines in
fork deltas.

## Review requirements

Every pull request that updates `third_party/full-monty/` must satisfy both
controls:

- machine check: `monty_fork_review` reports zero violations,
- human review: maintainers confirm each delta hunk maps to an allowed
  category and remains upstream-PR-able.

If either control fails, the change is blocked.

## Automated checker contract

The repository provides `src/bin/monty_fork_review.rs` as a fail-closed
review-policy check.

Modes:

- `--diff-file <PATH>` evaluates a prepared unified diff,
- `--base-superproject-rev <REV> --head-superproject-rev <REV>` resolves
  `third_party/full-monty` pointers at both revisions, builds the submodule
  diff, and evaluates the result.

Exit behaviour:

- exits `0` when no violations are found,
- exits non-zero when violations exist or when the diff cannot be resolved.

Continuous Integration (CI) executes this checker on pull requests and blocks
merges on failure.
