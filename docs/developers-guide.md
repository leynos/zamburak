# Developers' guide

This guide records internally facing conventions and practices for working on
Zamburak's tooling and continuous integration. Project-wide implementation and
review standards live in the
[engineering standards](zamburak-engineering-standards.md).

## CodeScene coverage publication

[ADR 003](adr-003-main-owns-codescene-coverage-publication.md) records the
decision and the options it rejected.

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
  including local reusable workflows those workflows call and local composite
  actions their steps run, which execute in the caller's job.
- The publisher's check step runs one exact command,
  `echo "available=${{ secrets.CS_ACCESS_TOKEN != '' }}" >> "$GITHUB_OUTPUT"`.
  The upload step runs only when that output is `true` and the ref is
  `refs/heads/main`, and passes the secret straight to the uploader's
  `access-token` input. No `env` block holds the token, because the uploader is
  a composite action that hands its step's `env` to the nested steps it runs.
- The publisher's concurrency group is `coverage-main-${{ github.ref }}`, never
  cancelled. Runs for `main` never overlap, and a newer trigger replaces an
  older pending run rather than queueing behind it. GitHub does not promise to
  start runs in trigger order, so this does not guarantee commit order. A
  manual re-run of an older run keeps its SHA and its run id: it republishes
  that commit's coverage to CodeScene, but replaces no ratchet baseline unless
  the original run saved none.
- Merges made by the Dependabot automerge workflow with `GITHUB_TOKEN` fire no
  push, so they reach the publisher only through a later push or a dispatch.
- A dispatch that replaces a pending push uploads the same or a newer commit.
  generate-coverage saves the baseline only on a push, so the baseline can lag
  by more than one commit until a later push saves it.

`tests/workflow_contracts/codescene_repository_test.py` holds this shape over
the repository's own workflows and local actions, using the readers and rules in
`tests/workflow_contracts/codescene_contract/`. The other
`codescene_*_test.py` files prove that each rule refuses the shape it exists to
refuse. Each case starts from a compliant fixture tree and changes one thing.
Workflows are read strictly: a duplicate key, or a workflow declaring both a
quoted and an unquoted `on` key, is refused rather than silently resolved. Run
the suite with `make test-workflow-contracts`.
