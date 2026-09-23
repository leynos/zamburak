# Architectural Decision Record (ADR) 003: `main` owns CodeScene coverage publication

Date: 2026-09-23

Status: Accepted

## Context and problem statement

Coverage was measured on pull requests, and the pull-request lane also sent the
report to CodeScene in `mode: check`. Both halves failed quietly. A pull
request from a fork cannot read `secrets.CS_ACCESS_TOKEN`, so the step was
skipped for exactly the changes that most need review. CodeScene accepts an
upload only for a branch it analyses, and a pull request's head is not one. The
shared uploader pins the `cs-coverage` archive by digest, but it cannot pin the
CodeScene API the tool calls, and the tool refuses to run when that API's
answers change shape. That had already turned pull-request lanes red across the
estate with no change in the repositories concerned. A lane that holds the
token is also a lane that a pull request's workflow edit could try to reach.

## Decision drivers

- Keep a failing coverage ratchet on every pull request.
- Keep the CodeScene credential, host, and client off every workflow a pull
  request can start, directly or through a called workflow.
- Give the ratchet baseline exactly one writer, so that every pull request is
  measured against `main`.
- Hold the split with an executable contract rather than a convention.

## Considered options

- Keep the CodeScene step on the pull-request lane. Rejected: forks never
  upload, heads are not analysed branches, the lane must hold the token, and an
  API change reddens every pull request.
- Drop CodeScene coverage altogether. Rejected: the trunk figure is still
  wanted, and the ratchet needs a baseline written on `main` anyway.
- Protect the publisher with a deployment environment restricted to `main`.
  Deferred to the repository owner, because it is a settings change. The ref
  guard on the upload stops a dispatch of the unedited publisher from a branch;
  it cannot stop a writer who edits and dispatches a branch's copy.

## Decision outcome

Two workflows split the work, as CV-005 of the estate's continuous integration
standards requires.

- `ci.yml` runs the shared `generate-coverage` action with
  `with-ratchet: 'true'` and `publish-artefact: 'false'`, and nothing a pull
  request can start touches CodeScene.
- `coverage-main.yml` is the one publisher. On a push to `main` it measures
  the same selection, which writes the ratchet baseline, then uploads the
  report in explicit upload mode. A check step with one exact command,
  `echo "available=${{ secrets.CS_ACCESS_TOKEN != '' }}" >> "$GITHUB_OUTPUT"`,
  binds nothing. The upload runs only when that output is `true` and the ref is
  `refs/heads/main`, and passes the secret straight to its `access-token`
  input. A concurrency group keyed on the ref never cancels a run in progress.

`tests/workflow_contracts/codescene_repository_test.py` holds the split. It
reads every workflow a pull request can reach as a closure through local
reusable-workflow calls, and every workflow a push can start for a second
baseline writer. The other `codescene_*_test.py` files drive each rule against
breaching fixtures.

## Consequences

- A pull request's coverage is judged only by the ratchet; CodeScene sees
  `main` alone.
- Dependabot automerge merges made with `GITHUB_TOKEN` fire no push, so they
  reach the publisher only through a later push or a dispatch. A dispatch
  uploads, but writes no baseline, so the baseline can lag by more than one
  commit until a later push saves it.
- Adding a workflow that touches CodeScene, runs ratcheted coverage on a
  push, or changes the coverage selection on one side only fails the contract,
  which names the clause.

The [developers' guide](developers-guide.md) records the publisher's shape and
its operational exceptions.
