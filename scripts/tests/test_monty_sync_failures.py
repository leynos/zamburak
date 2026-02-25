"""Failure-path tests for `scripts/monty_sync.py`.

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
    failure_outcome,
    invocation,
    successful_outcome,
)


def test_run_monty_sync_fails_when_fork_remote_missing(tmp_path: Path) -> None:
    """Verify missing origin remote fails with deterministic error."""
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
                successful_outcome("upstream\n"),
            ),
        )
    )

    with pytest.raises(monty_sync.MontySyncError, match="fork remote `origin` is missing"):
        monty_sync.run_monty_sync(runner, config=config, stdout=StringIO())
    runner.assert_exhausted()


def test_run_monty_sync_fails_when_verification_gate_fails(tmp_path: Path) -> None:
    """Verify gate failure aborts sync flow with non-success result."""
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
                failure_outcome("clippy: simulated lint failure"),
            ),
        )
    )

    with pytest.raises(monty_sync.MontySyncError, match="verification gate `lint` failed"):
        monty_sync.run_monty_sync(runner, config=config, stdout=StringIO())
    runner.assert_exhausted()
