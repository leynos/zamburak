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

`make test-workflow-contracts` holds this shape over the repository's own
workflows and local actions by running `cv005-contracts check`, the shared
contract library in `leynos/shared-actions` (`packages/cv005-contracts`), from
a full commit named by `CV005_CONTRACTS_REF` in the Makefile. A fix to the
rules is therefore a pin bump. The repository's only parameter is `repository`
in `.github/cv005.toml`. The library's own suite proves each rule refuses the
shape it exists to refuse, so this repository keeps no copy of the readers or
the refusal cases. Workflows are read strictly: a duplicate key, or a workflow
declaring both a quoted and an unquoted `on` key, is refused rather than
silently resolved, and a reading failure exits 2 rather than passing. The
remaining contracts in `tests/workflow_contracts` run under pytest in the same
target.

## Spelling gate

`make spelling` enforces en-GB-oxendict spelling by running the
`typos-config-builder` gate, pinned by `TYPOS_CONFIG_BUILDER_VERSION` in the
`Makefile` (currently `v0.1.3`). The gate regenerates `typos.toml` from the
live shared dictionary and this repository's `typos.local.toml` overlay on
every run, runs Typos over tracked Markdown, and enforces the shared phrase
corrections that single-word checks cannot express. `typos.toml` is generated
and never drift checked in continuous integration; commit the regenerated file
when it changes, and keep repository-specific exceptions in `typos.local.toml`
as narrow patterns. The builder requires Python 3.14 or newer, so the target
passes `--python 3.14` and `uv` fetches that interpreter when the host lacks
one. Raise the pin together with the regenerated `typos.toml`, never on its own.

The script tooling no longer installs `pathspec`: the builder now owns the
tracked-file scan, so `SCRIPT_UV_DEPS` in the `Makefile` lists only the
packages the script tests import.

## Markdown formatting

`make fmt` and `make check-fmt` run `mdtablefix` over the Markdown files Git
tracks and the untracked files it does not ignore, with the repository's
configured flags. They need mdtablefix 0.6.0 or later, the first release with
`--check` and `--git`. CI installs 0.6.1 through the shared
`install-mdtablefix` action; the pin is in `.github/workflows/ci.yml`. 0.6.1
fixes the 0.6.0 bugs that changed Markdown content: `--wrap` breaking text with
no whitespace, Setext headings turned into rules, and `--renumber` changing a
list's start. Install the same version locally with
`cargo binstall mdtablefix@0.6.1`.

`--renumber` treats a fence at column 0 as the end of an ordered list, so
indent the content of every list item under its marker (three columns for `1.`
to `9.`, four for `10.` to `99.`) to keep a numbered procedure in one list.
