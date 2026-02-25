"""Workflow-focused tests for `scripts/monty_sync.py`.

Usage
-----
Run with:

```
make script-test
```
"""

from __future__ import annotations

from io import StringIO
from pathlib import Path

import pytest

import monty_sync

from monty_sync_test_helpers import (
    CommandStub,
    QueueRunner,
    build_gate_stubs,
    build_config,
    build_preflight_stubs,
    build_remote_setup_stubs,
    build_sync_stubs,
    gate_stubs,
    happy_path_stubs_up_to_sync,
    invocation,
    post_sync_stubs,
    successful_outcome,
)


def build_happy_path_command_stubs(config: monty_sync.SyncConfig) -> tuple[CommandStub, ...]:
    """Build command stub sequence for happy-path monty-sync workflow."""
    old_rev = "1111111111111111111111111111111111111111"
    new_rev = "2222222222222222222222222222222222222222"
    return (
        build_preflight_stubs(config)
        + build_remote_setup_stubs(config, has_upstream=True)
        + build_sync_stubs(config, old_rev, new_rev)
        + build_gate_stubs(config)
    )


def test_run_monty_sync_happy_path_updates_revision_and_runs_gates(
    tmp_path: Path,
) -> None:
    """Verify successful sync updates revision and executes gate targets."""
    config = build_config(tmp_path)
    runner = QueueRunner(build_happy_path_command_stubs(config))
    stdout = StringIO()

    monty_sync.run_monty_sync(runner, config=config, stdout=stdout)

    runner.assert_exhausted()
    output = stdout.getvalue()
    assert "monty-sync: submodule revision updated" in output
    assert "monty-sync: running verification gates" in output
    assert "monty-sync: completed successfully" in output


def test_run_monty_sync_initializes_submodule_before_submodule_operations(
    tmp_path: Path,
) -> None:
    """Verify submodule initialisation occurs before submodule-scoped commands."""
    config = build_config(tmp_path)
    rev = "1111111111111111111111111111111111111111"
    runner = QueueRunner(
        happy_path_stubs_up_to_sync(config, has_upstream=False, old_revision=rev)
        + post_sync_stubs(config, new_revision=rev)
        + gate_stubs(config)
    )

    monty_sync.run_monty_sync(runner, config=config, stdout=StringIO())
    runner.assert_exhausted()

    submodule_update_idx = runner.calls.index(
        invocation(
            config,
            program="git",
            args=(
                "submodule",
                "update",
                "--init",
                "--recursive",
                "third_party/full-monty",
            ),
        )
    )
    first_submodule_scoped_idx = next(
        idx
        for idx, call in enumerate(runner.calls)
        if call.cwd == config.submodule_root
    )
    assert submodule_update_idx < first_submodule_scoped_idx


def test_run_monty_sync_fails_when_superproject_dirty(tmp_path: Path) -> None:
    """Verify dirty superproject worktree fails before sync mutation."""
    config = build_config(tmp_path)
    runner = QueueRunner(
        (
            CommandStub(
                invocation(config, program="git", args=("status", "--porcelain")),
                successful_outcome(" M docs/roadmap.md\n"),
            ),
        )
    )

    with pytest.raises(monty_sync.MontySyncError, match="superproject worktree is not clean"):
        monty_sync.run_monty_sync(runner, config=config, stdout=StringIO())
    runner.assert_exhausted()
