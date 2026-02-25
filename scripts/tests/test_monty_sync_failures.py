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
    gate_stubs,
    happy_path_stubs_up_to_sync,
    invocation,
    post_sync_stubs,
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
    rev = "1111111111111111111111111111111111111111"
    runner = QueueRunner(
        happy_path_stubs_up_to_sync(config, has_upstream=True, old_revision=rev)
        + post_sync_stubs(config, new_revision=rev)
        + gate_stubs(config, fail_at="lint")
    )

    with pytest.raises(monty_sync.MontySyncError, match="verification gate `lint` failed"):
        monty_sync.run_monty_sync(runner, config=config, stdout=StringIO())
    runner.assert_exhausted()
