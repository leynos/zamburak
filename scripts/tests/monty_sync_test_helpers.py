"""Shared helpers for `scripts/monty_sync.py` test suites.

Purpose
-------
Provide reusable `CommandStub` builders and a deterministic `QueueRunner` for
unit and behavioural tests that exercise monty-sync orchestration without
running real git or make commands.

Utility
-------
Test modules use these helpers to keep command ordering fixtures concise and
consistent across workflow, failure, CLI, and BDD coverage.

Usage
-----
Compose scenario stubs and execute them through `QueueRunner`:

```python
config = build_config(tmp_path)
runner = QueueRunner(
    build_preflight_stubs(config)
    + (build_remote_listing_stub(config, remotes=("origin", "upstream")),)
    + build_sync_operation_stubs(
        config,
        remotes=("origin", "upstream"),
        old_revision="a" * 40,
        new_revision="b" * 40,
    )
    + gate_stubs(config)
)
```
"""

from __future__ import annotations

from collections import deque
from collections.abc import Iterable, Sequence
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


def build_remote_listing_stub(
    config: monty_sync.SyncConfig,
    *,
    remotes: Sequence[str],
) -> CommandStub:
    """Build stub for `git remote` output scoped to the submodule checkout."""
    return CommandStub(
        invocation(config, program="git", args=("remote",), submodule=True),
        successful_outcome("".join(f"{remote}\n" for remote in remotes)),
    )


def build_preflight_stubs(config: monty_sync.SyncConfig) -> tuple[CommandStub, ...]:
    """Build stubs for preflight checks and submodule initialisation."""
    return (
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
                    config.submodule_path.as_posix(),
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
    )


def build_remote_setup_stubs(
    config: monty_sync.SyncConfig,
    has_upstream: bool = False,
) -> tuple[CommandStub, ...]:
    """Build stubs for remote configuration (add or set-url upstream)."""
    remotes = ("origin", "upstream") if has_upstream else ("origin",)
    remote_cmd = "set-url" if has_upstream else "add"
    return (
        build_remote_listing_stub(config, remotes=remotes),
        CommandStub(
            invocation(
                config,
                program="git",
                args=(
                    "remote",
                    remote_cmd,
                    "upstream",
                    "https://github.com/pydantic/monty.git",
                ),
                submodule=True,
            ),
            successful_outcome(),
        ),
    )


def _build_fetch_stubs_helper(config: monty_sync.SyncConfig) -> tuple[CommandStub, ...]:
    """Build stubs for fetch operations from fork and upstream remotes."""
    return (
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
    )


def _build_checkout_merge_stubs_helper(
    config: monty_sync.SyncConfig,
) -> tuple[CommandStub, ...]:
    """Build stubs for branch checkout and fast-forward merge operations."""
    return (
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
    )


def _build_post_sync_stubs_helper(config: monty_sync.SyncConfig, *, new_revision: str) -> tuple[CommandStub, ...]:
    """Build stubs for post-sync revision capture and pointer staging."""
    after_revision = new_revision.rstrip("\n")
    return (
        CommandStub(
            invocation(config, program="git", args=("rev-parse", "HEAD"), submodule=True),
            successful_outcome(f"{after_revision}\n"),
        ),
        CommandStub(
            invocation(config, program="git", args=("add", config.submodule_path.as_posix())),
            successful_outcome(),
        ),
    )


def build_sync_operation_stubs(
    config: monty_sync.SyncConfig,
    *,
    remotes: Sequence[str],
    old_revision: str,
    new_revision: str,
) -> tuple[CommandStub, ...]:
    """Build stubs for remote setup, sync operations, and pointer staging."""
    upstream_command = "set-url" if "upstream" in remotes else "add"
    before_revision = old_revision.rstrip("\n")
    remote_setup_stub = (
        CommandStub(
            invocation(
                config,
                program="git",
                args=(
                    "remote",
                    upstream_command,
                    "upstream",
                    "https://github.com/pydantic/monty.git",
                ),
                submodule=True,
            ),
            successful_outcome(),
        ),
    )
    pre_sync_revision_stub = (
        CommandStub(
            invocation(config, program="git", args=("rev-parse", "HEAD"), submodule=True),
            successful_outcome(f"{before_revision}\n"),
        ),
    )
    return (
        remote_setup_stub
        + pre_sync_revision_stub
        + _build_fetch_stubs_helper(config)
        + _build_checkout_merge_stubs_helper(config)
        + _build_post_sync_stubs_helper(config, new_revision=new_revision)
    )


def build_sync_stubs(
    config: monty_sync.SyncConfig,
    old_rev: str,
    new_rev: str,
) -> tuple[CommandStub, ...]:
    """Build stubs for fetch/merge sync operations."""
    return build_sync_operation_stubs(
        config,
        remotes=("origin", "upstream"),
        old_revision=old_rev,
        new_revision=new_rev,
    )[1:]


def build_gate_stubs(config: monty_sync.SyncConfig) -> tuple[CommandStub, ...]:
    """Build stubs for verification gate targets."""
    return (
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


def happy_path_stubs_up_to_sync(
    config: monty_sync.SyncConfig,
    *,
    has_upstream: bool = True,
    old_revision: str = "1111111111111111111111111111111111111111",
) -> tuple[CommandStub, ...]:
    """Build stubs from preflight checks through merge for happy-path sync."""
    remotes = ("origin", "upstream") if has_upstream else ("origin",)
    return (
        build_preflight_stubs(config)
        + (build_remote_listing_stub(config, remotes=remotes),)
        + build_sync_operation_stubs(
            config,
            remotes=remotes,
            old_revision=old_revision,
            new_revision=old_revision,
        )[:6]
    )


def post_sync_stubs(
    config: monty_sync.SyncConfig,
    *,
    new_revision: str = "1111111111111111111111111111111111111111",
) -> tuple[CommandStub, ...]:
    """Build stubs for post-sync revision capture and pointer staging."""
    normalised_new_revision = new_revision.rstrip("\n")
    return (
        CommandStub(
            invocation(config, program="git", args=("rev-parse", "HEAD"), submodule=True),
            successful_outcome(f"{normalised_new_revision}\n"),
        ),
        CommandStub(
            invocation(config, program="git", args=("add", config.submodule_path.as_posix())),
            successful_outcome(),
        ),
    )


def gate_stubs(
    config: monty_sync.SyncConfig,
    *,
    fail_at: str | None = None,
) -> tuple[CommandStub, ...]:
    """Build stubs for verification gates, optionally failing at one target."""
    stubs: list[CommandStub] = []
    for target in ("check-fmt", "lint", "test"):
        if target == fail_at:
            stubs.append(
                CommandStub(
                    invocation(config, program="make", args=(target,)),
                    failure_outcome("clippy: simulated lint failure"),
                )
            )
            break
        stubs.append(
            CommandStub(
                invocation(config, program="make", args=(target,)),
                successful_outcome(),
            )
        )
    return tuple(stubs)


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
