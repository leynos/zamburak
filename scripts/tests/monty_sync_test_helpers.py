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
    """Represent one command invocation for queued test stubs.

    Parameters
    ----------
    program : str
        Executable name, for example ``git`` or ``make``.
    args : tuple[str, ...]
        Positional arguments passed to ``program``.
    cwd : Path
        Working directory used to execute the command.
    """

    program: str
    args: tuple[str, ...]
    cwd: Path


@dataclass(frozen=True)
class CommandStub:
    """Bind an expected invocation to its deterministic outcome.

    Parameters
    ----------
    invocation : CommandInvocation
        Invocation key matched by :class:`QueueRunner`.
    outcome : monty_sync.CommandOutcome
        Outcome returned when ``invocation`` is consumed.
    """

    invocation: CommandInvocation
    outcome: monty_sync.CommandOutcome


def build_config(tmp_path: Path) -> monty_sync.SyncConfig:
    """Create a sync configuration rooted in a temporary repository.

    Parameters
    ----------
    tmp_path : Path
        Temporary directory fixture provided by pytest.

    Returns
    -------
    monty_sync.SyncConfig
        Configuration whose ``repo_root`` points at ``tmp_path / "repo"``.
    """
    repo_root = tmp_path / "repo"
    return monty_sync.SyncConfig(repo_root=repo_root)


def invocation(
    config: monty_sync.SyncConfig,
    *,
    program: str,
    args: tuple[str, ...],
    submodule: bool = False,
) -> CommandInvocation:
    """Create a command invocation in repository or submodule scope.

    Parameters
    ----------
    config : monty_sync.SyncConfig
        Sync configuration that defines repository paths.
    program : str
        Executable name to invoke.
    args : tuple[str, ...]
        Positional arguments passed to the executable.
    submodule : bool, default=False
        Whether to target ``config.submodule_root`` instead of ``repo_root``.

    Returns
    -------
    CommandInvocation
        Normalised invocation object for use in command stubs.
    """
    cwd = config.submodule_root if submodule else config.repo_root
    return CommandInvocation(program=program, args=args, cwd=cwd)


def build_remote_listing_stub(
    config: monty_sync.SyncConfig,
    *,
    remotes: Sequence[str],
) -> CommandStub:
    """Build a submodule-scoped ``git remote`` listing stub.

    Parameters
    ----------
    config : monty_sync.SyncConfig
        Sync configuration that defines command working directories.
    remotes : Sequence[str]
        Remote names returned by ``git remote`` in stdout order.

    Returns
    -------
    CommandStub
        Stub producing one newline-terminated remote per entry.
    """
    return CommandStub(
        invocation(config, program="git", args=("remote",), submodule=True),
        successful_outcome("".join(f"{remote}\n" for remote in remotes)),
    )


def build_preflight_stubs(config: monty_sync.SyncConfig) -> tuple[CommandStub, ...]:
    """Build stubs for preflight checks and submodule initialization.

    Parameters
    ----------
    config : monty_sync.SyncConfig
        Sync configuration that defines repository and submodule paths.

    Returns
    -------
    tuple[CommandStub, ...]
        Stubs for superproject cleanliness, submodule update, and submodule
        cleanliness checks.
    """
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
    *,
    has_upstream: bool = False,
) -> tuple[CommandStub, ...]:
    """Build stubs for remote inspection and upstream configuration.

    Parameters
    ----------
    config : monty_sync.SyncConfig
        Sync configuration that defines remote names and URLs.
    has_upstream : bool, default=False
        Whether the initial remote listing already includes upstream.

    Returns
    -------
    tuple[CommandStub, ...]
        Stub sequence for ``git remote`` and the matching
        ``git remote add|set-url`` command.
    """
    remotes = (
        config.fork_remote,
        config.upstream_remote,
    ) if has_upstream else (config.fork_remote,)
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
                    config.upstream_remote,
                    config.upstream_url,
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
                args=("fetch", "--prune", config.fork_remote),
                submodule=True,
            ),
            successful_outcome(),
        ),
        CommandStub(
            invocation(
                config,
                program="git",
                args=("fetch", "--prune", config.upstream_remote),
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
                args=(
                    "checkout",
                    "-B",
                    config.fork_branch,
                    f"{config.fork_remote}/{config.fork_branch}",
                ),
                submodule=True,
            ),
            successful_outcome(),
        ),
        CommandStub(
            invocation(
                config,
                program="git",
                args=(
                    "merge",
                    "--ff-only",
                    f"{config.upstream_remote}/{config.upstream_branch}",
                ),
                submodule=True,
            ),
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
    """Build stubs for remote setup, sync operations, and pointer staging.

    Parameters
    ----------
    config : monty_sync.SyncConfig
        Sync configuration that defines remotes, branches, and paths.
    remotes : Sequence[str]
        Remote names available before upstream reconciliation.
    old_revision : str
        Revision reported before sync; trailing newline is tolerated.
    new_revision : str
        Revision reported after sync; trailing newline is tolerated.

    Returns
    -------
    tuple[CommandStub, ...]
        Ordered stubs covering remote setup, revision capture, fetch, checkout,
        merge, post-sync revision capture, and pointer staging.
    """
    upstream_command = "set-url" if config.upstream_remote in remotes else "add"
    before_revision = old_revision.rstrip("\n")
    remote_setup_stub = (
        CommandStub(
            invocation(
                config,
                program="git",
                args=(
                    "remote",
                    upstream_command,
                    config.upstream_remote,
                    config.upstream_url,
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
        + post_sync_stubs(config, new_revision=new_revision)
    )


def build_sync_stubs(
    config: monty_sync.SyncConfig,
    old_rev: str,
    new_rev: str,
) -> tuple[CommandStub, ...]:
    """Build stubs for sync operations after remotes are already listed.

    Parameters
    ----------
    config : monty_sync.SyncConfig
        Sync configuration that defines remotes, branches, and paths.
    old_rev : str
        Revision reported before fetch and merge.
    new_rev : str
        Revision reported after fetch and merge.

    Returns
    -------
    tuple[CommandStub, ...]
        Sync-operation subset that excludes the initial remote listing stub.
    """
    return build_sync_operation_stubs(
        config,
        remotes=(config.fork_remote, config.upstream_remote),
        old_revision=old_rev,
        new_revision=new_rev,
    )[1:]


def build_gate_stubs(config: monty_sync.SyncConfig) -> tuple[CommandStub, ...]:
    """Build successful stubs for configured verification gates.

    Parameters
    ----------
    config : monty_sync.SyncConfig
        Sync configuration containing ``verification_targets``.

    Returns
    -------
    tuple[CommandStub, ...]
        One successful ``make <target>`` stub for each verification target.
    """
    return tuple(
        CommandStub(
            invocation(config, program="make", args=(target,)),
            successful_outcome(),
        )
        for target in config.verification_targets
    )


def happy_path_stubs_up_to_sync(
    config: monty_sync.SyncConfig,
    *,
    has_upstream: bool = True,
    old_revision: str = "1111111111111111111111111111111111111111",
) -> tuple[CommandStub, ...]:
    """Build happy-path stubs from preflight checks through merge.

    Parameters
    ----------
    config : monty_sync.SyncConfig
        Sync configuration that defines remotes, branches, and paths.
    has_upstream : bool, default=True
        Whether the initial remote listing includes upstream.
    old_revision : str, default="1111111111111111111111111111111111111111"
        Revision value to return for pre-sync and post-sync HEAD checks.

    Returns
    -------
    tuple[CommandStub, ...]
        Stub sequence that stops after the merge operation.
    """
    remotes = (
        config.fork_remote,
        config.upstream_remote,
    ) if has_upstream else (config.fork_remote,)
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
    """Build stubs for post-sync revision capture and pointer staging.

    Parameters
    ----------
    config : monty_sync.SyncConfig
        Sync configuration that defines repository and submodule paths.
    new_revision : str, default="1111111111111111111111111111111111111111"
        Revision value returned by the post-sync ``rev-parse`` command.

    Returns
    -------
    tuple[CommandStub, ...]
        Stubs for post-sync revision capture and ``git add`` pointer staging.
    """
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
    """Build gate stubs, optionally failing at a specific verification target.

    Parameters
    ----------
    config : monty_sync.SyncConfig
        Sync configuration containing verification targets.
    fail_at : str | None, default=None
        Target name that should fail; later targets are omitted.

    Returns
    -------
    tuple[CommandStub, ...]
        Gate stubs up to and including ``fail_at`` when provided.
    """
    stubs: list[CommandStub] = []
    for target in config.verification_targets:
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
    """Create a successful command outcome for command stubs.

    Parameters
    ----------
    stdout : str, default=""
        Standard-output payload returned by the stubbed command.

    Returns
    -------
    monty_sync.CommandOutcome
        Successful outcome with exit code ``0``.
    """
    return monty_sync.CommandOutcome(ok=True, stdout=stdout, stderr="", exit_code=0)


def failure_outcome(stderr: str, *, exit_code: int = 1) -> monty_sync.CommandOutcome:
    """Create a failing command outcome for command stubs.

    Parameters
    ----------
    stderr : str
        Standard-error payload returned by the stubbed command.
    exit_code : int, default=1
        Exit code reported by the failing command.

    Returns
    -------
    monty_sync.CommandOutcome
        Failed outcome with empty stdout.
    """
    return monty_sync.CommandOutcome(
        ok=False,
        stdout="",
        stderr=stderr,
        exit_code=exit_code,
    )


class QueueRunner:
    """Run queued command stubs while validating strict invocation ordering.

    Notes
    -----
    Each call to :meth:`run` consumes one stub. A mismatch raises
    :class:`AssertionError` to fail tests immediately.
    """

    def __init__(self, stubs: Iterable[CommandStub]) -> None:
        """Initialize the queue-backed runner.

        Parameters
        ----------
        stubs : Iterable[CommandStub]
            Ordered command stubs expected during the test.
        """
        self._stubs = deque(stubs)
        self.calls: list[CommandInvocation] = []

    def run(
        self,
        *,
        program: str,
        args: tuple[str, ...],
        cwd: Path,
    ) -> monty_sync.CommandOutcome:
        """Execute one queued stub and validate invocation identity.

        Parameters
        ----------
        program : str
            Executable name expected by the next stub.
        args : tuple[str, ...]
            Argument tuple expected by the next stub.
        cwd : Path
            Working directory expected by the next stub.

        Returns
        -------
        monty_sync.CommandOutcome
            Outcome associated with the consumed stub.

        Raises
        ------
        AssertionError
            Raised when no stubs remain or the invocation does not match.
        """
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
        """Assert that all queued stubs were consumed.

        Returns
        -------
        None

        Raises
        ------
        AssertionError
            Raised when queued stubs remain after the test run.
        """
        assert not self._stubs, (
            f"expected {len(self._stubs)} additional command invocation(s)"
        )
