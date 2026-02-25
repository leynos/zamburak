"""Test helpers for `scripts/monty_sync.py` suites."""

from __future__ import annotations

from collections import deque
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path

import monty_sync


@dataclass(frozen=True)
class CommandInvocation:
    """One command invocation key for fake command runner stubs."""

    program: str
    args: tuple[str, ...]
    cwd: Path


@dataclass(frozen=True)
class CommandStub:
    """Stubbed command outcome for a specific invocation key."""

    invocation: CommandInvocation
    outcome: monty_sync.CommandOutcome


def build_config(tmp_path: Path) -> monty_sync.SyncConfig:
    """Create a `SyncConfig` rooted at a temporary repository path."""
    repo_root = tmp_path / "repo"
    return monty_sync.SyncConfig(repo_root=repo_root)


def invocation(
    config: monty_sync.SyncConfig,
    *,
    program: str,
    args: tuple[str, ...],
    submodule: bool = False,
) -> CommandInvocation:
    """Create a `CommandInvocation` for repository or submodule scope."""
    cwd = config.submodule_root if submodule else config.repo_root
    return CommandInvocation(program=program, args=args, cwd=cwd)


def successful_outcome(stdout: str = "") -> monty_sync.CommandOutcome:
    """Return a successful command outcome with optional stdout."""
    return monty_sync.CommandOutcome(ok=True, stdout=stdout, stderr="", exit_code=0)


def failure_outcome(stderr: str, *, exit_code: int = 1) -> monty_sync.CommandOutcome:
    """Return a failing command outcome with deterministic stderr."""
    return monty_sync.CommandOutcome(
        ok=False,
        stdout="",
        stderr=stderr,
        exit_code=exit_code,
    )


class QueueRunner:
    """Fake `CommandRunner` that validates invocation order against a queue."""

    def __init__(self, stubs: Iterable[CommandStub]) -> None:
        self._stubs = deque(stubs)
        self.calls: list[CommandInvocation] = []

    def run(
        self,
        *,
        program: str,
        args: tuple[str, ...],
        cwd: Path,
    ) -> monty_sync.CommandOutcome:
        if not self._stubs:
            raise AssertionError(
                f"unexpected command invocation `{program} {' '.join(args)}` in `{cwd}`"
            )

        next_stub = self._stubs.popleft()
        invocation = CommandInvocation(program=program, args=args, cwd=cwd)
        self.calls.append(invocation)
        if invocation != next_stub.invocation:
            raise AssertionError(
                "command invocation mismatch: "
                f"expected `{next_stub.invocation}` got `{invocation}`"
            )
        return next_stub.outcome

    def assert_exhausted(self) -> None:
        """Assert that all expected command stubs were consumed."""
        assert not self._stubs, (
            f"expected {len(self._stubs)} additional command invocation(s)"
        )
