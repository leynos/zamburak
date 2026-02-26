"""Failure-path tests for `scripts/monty_sync.py`.

Usage
-----
Run with:

```sh
make script-test
```
"""

from __future__ import annotations

from io import StringIO
from pathlib import Path

import pytest

import monty_sync

from monty_sync_test_helpers import (
    QueueRunner,
    build_config,
    build_preflight_stubs,
    build_remote_listing_stub,
    gate_stubs,
    happy_path_stubs_up_to_sync,
    post_sync_stubs,
)


def test_run_monty_sync_fails_when_fork_remote_missing(tmp_path: Path) -> None:
    """Verify missing origin remote fails with deterministic error."""
    config = build_config(tmp_path)
    runner = QueueRunner(
        build_preflight_stubs(config)
        + (build_remote_listing_stub(config, remotes=("upstream",)),)
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
