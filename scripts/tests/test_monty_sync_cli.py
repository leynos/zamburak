"""CLI-focused tests for `scripts/monty_sync.py`.

Usage
-----
Run with:

```
make script-test
```
"""

from __future__ import annotations

import pytest

import monty_sync


def test_main_reports_error_when_run_monty_sync_raises(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Verify CLI returns non-zero and stderr output on sync failures."""

    def _raise_sync_error(
        runner: monty_sync.CommandRunner,
        *,
        config: monty_sync.SyncConfig,
        stdout,
    ) -> None:
        raise monty_sync.MontySyncError("simulated failure")

    monkeypatch.setattr(monty_sync, "run_monty_sync", _raise_sync_error)
    exit_code = monty_sync.main([])
    captured = capsys.readouterr()

    assert exit_code == 1
    assert "monty-sync error: simulated failure" in captured.err


def test_main_success_runs_monty_sync(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Verify CLI success path wiring and exit code when sync succeeds."""
    recorded: dict[str, object] = {}

    def _fake_sync(
        runner: monty_sync.CommandRunner,
        *,
        config: monty_sync.SyncConfig,
        stdout,
    ) -> None:
        recorded["runner"] = runner
        recorded["config"] = config
        recorded["stdout"] = stdout

    monkeypatch.setattr(monty_sync, "run_monty_sync", _fake_sync)
    exit_code = monty_sync.main([])

    assert exit_code == 0
    assert isinstance(recorded.get("config"), monty_sync.SyncConfig)


def test_main_accepts_help() -> None:
    """Verify help flag returns zero without running sync."""
    assert monty_sync.main(["--help"]) == 0


def test_main_rejects_unknown_arguments(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Verify unsupported CLI arguments return error exit code."""
    exit_code = monty_sync.main(["--unexpected"])
    captured = capsys.readouterr()

    assert exit_code == 2
    assert "unsupported arguments" in captured.err
