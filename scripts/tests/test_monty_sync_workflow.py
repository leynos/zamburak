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
    build_config,
    invocation,
    successful_outcome,
)


def test_run_monty_sync_happy_path_updates_revision_and_runs_gates(
    tmp_path: Path,
) -> None:
    """Verify successful sync updates revision and executes gate targets."""
    config = build_config(tmp_path)
    runner = QueueRunner(
        (
            CommandStub(
                invocation(config, program="git", args=("status", "--porcelain")),
                successful_outcome(),
            ),
            CommandStub(
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
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(
                    config,
                    program="git",
                    args=("status", "--porcelain"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(config, program="git", args=("remote",), submodule=True),
                successful_outcome("origin\nupstream\n"),
            ),
            CommandStub(
                invocation(
                    config,
                    program="git",
                    args=(
                        "remote",
                        "set-url",
                        "upstream",
                        "https://github.com/pydantic/monty.git",
                    ),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(config, program="git", args=("rev-parse", "HEAD"), submodule=True),
                successful_outcome("1111111111111111111111111111111111111111\n"),
            ),
            CommandStub(
                invocation(
                    config,
                    program="git",
                    args=("fetch", "--prune", "origin"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(
                    config,
                    program="git",
                    args=("fetch", "--prune", "upstream"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(
                    config,
                    program="git",
                    args=("checkout", "-B", "main", "origin/main"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(
                    config,
                    program="git",
                    args=("merge", "--ff-only", "upstream/main"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(config, program="git", args=("rev-parse", "HEAD"), submodule=True),
                successful_outcome("2222222222222222222222222222222222222222\n"),
            ),
            CommandStub(
                invocation(config, program="git", args=("add", "third_party/full-monty")),
                successful_outcome(),
            ),
            CommandStub(
                invocation(config, program="make", args=("check-fmt",)),
                successful_outcome(),
            ),
            CommandStub(
                invocation(config, program="make", args=("lint",)),
                successful_outcome(),
            ),
            CommandStub(
                invocation(config, program="make", args=("test",)),
                successful_outcome(),
            ),
        )
    )
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
    """Verify submodule initialization occurs before submodule-scoped commands."""
    config = build_config(tmp_path)
    runner = QueueRunner(
        (
            CommandStub(
                invocation(config, program="git", args=("status", "--porcelain")),
                successful_outcome(),
            ),
            CommandStub(
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
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(
                    config,
                    program="git",
                    args=("status", "--porcelain"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(config, program="git", args=("remote",), submodule=True),
                successful_outcome("origin\n"),
            ),
            CommandStub(
                invocation(
                    config,
                    program="git",
                    args=(
                        "remote",
                        "add",
                        "upstream",
                        "https://github.com/pydantic/monty.git",
                    ),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(config, program="git", args=("rev-parse", "HEAD"), submodule=True),
                successful_outcome("1111111111111111111111111111111111111111\n"),
            ),
            CommandStub(
                invocation(
                    config,
                    program="git",
                    args=("fetch", "--prune", "origin"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(
                    config,
                    program="git",
                    args=("fetch", "--prune", "upstream"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(
                    config,
                    program="git",
                    args=("checkout", "-B", "main", "origin/main"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(
                    config,
                    program="git",
                    args=("merge", "--ff-only", "upstream/main"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                invocation(config, program="git", args=("rev-parse", "HEAD"), submodule=True),
                successful_outcome("1111111111111111111111111111111111111111\n"),
            ),
            CommandStub(
                invocation(config, program="git", args=("add", "third_party/full-monty")),
                successful_outcome(),
            ),
            CommandStub(
                invocation(config, program="make", args=("check-fmt",)),
                successful_outcome(),
            ),
            CommandStub(
                invocation(config, program="make", args=("lint",)),
                successful_outcome(),
            ),
            CommandStub(
                invocation(config, program="make", args=("test",)),
                successful_outcome(),
            ),
        )
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
