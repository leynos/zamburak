#!/usr/bin/env -S uv run python
# /// script
# requires-python = ">=3.13"
# dependencies = ["cuprum==0.1.0"]
# ///
"""Synchronise `full-monty` and run repository verification gates.

The script initializes and refreshes `third_party/full-monty`, fast-forwards
the fork branch against upstream, stages the submodule pointer update, and runs
post-sync verification gates. Run from repository root with `make monty-sync`.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import sys
from typing import TYPE_CHECKING, Protocol, TextIO

from cuprum import ExecutionContext, Program, scoped

from _cuprum_helpers import build_catalogue, build_commands

if TYPE_CHECKING:
    from cuprum import SafeCmd


GIT = Program("git")
MAKE = Program("make")
CATALOGUE = build_catalogue(GIT, MAKE)
COMMANDS = build_commands(catalogue=CATALOGUE, programs=(GIT, MAKE))
git = COMMANDS[GIT]
make = COMMANDS[MAKE]


@dataclass(frozen=True)
class CommandOutcome:
    """Represent one command result consumed by sync orchestration.

    Parameters
    ----------
    ok : bool
    stdout : str
    stderr : str
    exit_code : int
    """

    ok: bool
    stdout: str
    stderr: str
    exit_code: int


@dataclass(frozen=True)
class SyncConfig:
    """Define repository and remote settings for monty-sync.

    Parameters
    ----------
    repo_root : Path
    submodule_path : Path
    fork_remote : str
    fork_branch : str
    upstream_remote : str
    upstream_url : str
    upstream_branch : str
    verification_targets : tuple[str, ...]
    """

    repo_root: Path
    submodule_path: Path = Path("third_party/full-monty")
    fork_remote: str = "origin"
    fork_branch: str = "main"
    upstream_remote: str = "upstream"
    upstream_url: str = "https://github.com/pydantic/monty.git"
    upstream_branch: str = "main"
    verification_targets: tuple[str, ...] = ("check-fmt", "lint", "test")

    @property
    def submodule_root(self) -> Path:
        return self.repo_root / self.submodule_path


@dataclass(frozen=True)
class CommandInvocation:
    """Describe one command invocation expected by a command runner.

    Parameters
    ----------
    program : str
    args : tuple[str, ...]
    cwd : Path
    """

    program: str
    args: tuple[str, ...]
    cwd: Path


class CommandRunner(Protocol):
    """Define the command runner contract used by monty-sync orchestration."""

    def run(self, *, program: str, args: tuple[str, ...], cwd: Path) -> CommandOutcome:
        """Execute one command invocation.

        Parameters
        ----------
        program : str
        args : tuple[str, ...]
        cwd : Path

        Returns
        -------
        CommandOutcome
        """


class MontySyncError(RuntimeError):
    """Signal a fail-closed monty-sync orchestration error."""


class CuprumRunner:
    """Execute command invocations via Cuprum safe command wrappers."""

    def run(self, *, program: str, args: tuple[str, ...], cwd: Path) -> CommandOutcome:
        command = _resolve_command(program)
        context = ExecutionContext(cwd=cwd.as_posix())
        with scoped(allowlist=CATALOGUE.allowlist):
            result = command(*args).run_sync(context=context)
        return CommandOutcome(
            ok=result.ok,
            stdout=result.stdout,
            stderr=result.stderr,
            exit_code=result.exit_code,
        )


def _resolve_command(program: str) -> "SafeCmd":
    """Resolve supported command name to Cuprum command handle."""
    if program == "git":
        return git
    if program == "make":
        return make
    raise MontySyncError(f"unsupported command `{program}`")


def _run_checked(
    runner: CommandRunner,
    *,
    invocation: CommandInvocation,
    failure_summary: str,
) -> CommandOutcome:
    """Execute one command and raise `MontySyncError` on failure."""
    outcome = runner.run(program=invocation.program, args=invocation.args, cwd=invocation.cwd)
    if outcome.ok:
        return outcome

    details = outcome.stderr.strip() or outcome.stdout.strip() or "no error detail"
    command_text = " ".join((invocation.program, *invocation.args))
    raise MontySyncError(
        f"{failure_summary}: `{command_text}` failed with exit code "
        f"{outcome.exit_code}: {details}"
    )


def _log(stdout: TextIO, message: str) -> None:
    """Write one log line to output stream."""
    stdout.write(f"{message}\n")


def _ensure_clean_worktree(runner: CommandRunner, *, cwd: Path, scope_name: str) -> None:
    """Fail when `cwd` worktree has tracked or untracked changes."""
    outcome = _run_checked(
        runner,
        invocation=CommandInvocation(program="git", args=("status", "--porcelain"), cwd=cwd),
        failure_summary=f"unable to inspect {scope_name} worktree status",
    )
    if outcome.stdout.strip():
        raise MontySyncError(
            f"{scope_name} worktree is not clean; commit or stash changes before running monty sync"
        )


def _read_head_revision(runner: CommandRunner, *, cwd: Path) -> str:
    """Return current HEAD revision for one repository checkout."""
    outcome = _run_checked(
        runner,
        invocation=CommandInvocation(program="git", args=("rev-parse", "HEAD"), cwd=cwd),
        failure_summary="unable to read HEAD revision",
    )
    return outcome.stdout.strip()


def _ensure_remotes(runner: CommandRunner, *, config: SyncConfig) -> None:
    """Validate fork remote and configure upstream remote URL."""
    remotes_outcome = _run_checked(
        runner,
        invocation=CommandInvocation(program="git", args=("remote",), cwd=config.submodule_root),
        failure_summary="unable to list full-monty remotes",
    )
    remotes = {line.strip() for line in remotes_outcome.stdout.splitlines() if line.strip()}
    if config.fork_remote not in remotes:
        raise MontySyncError(
            f"fork remote `{config.fork_remote}` is missing in "
            f"{config.submodule_path.as_posix()}"
        )

    if config.upstream_remote not in remotes:
        _run_checked(
            runner,
            invocation=CommandInvocation(
                program="git",
                args=("remote", "add", config.upstream_remote, config.upstream_url),
                cwd=config.submodule_root,
            ),
            failure_summary="unable to add upstream remote",
        )
        return

    _run_checked(
        runner,
        invocation=CommandInvocation(
            program="git",
            args=("remote", "set-url", config.upstream_remote, config.upstream_url),
            cwd=config.submodule_root,
        ),
        failure_summary="unable to update upstream remote URL",
    )


def _refresh_submodule_branch(runner: CommandRunner, *, config: SyncConfig) -> None:
    """Refresh local fork branch by fast-forwarding with upstream branch."""
    _run_checked(
        runner,
        invocation=CommandInvocation(
            program="git",
            args=("fetch", "--prune", config.fork_remote),
            cwd=config.submodule_root,
        ),
        failure_summary="unable to fetch fork remote",
    )
    _run_checked(
        runner,
        invocation=CommandInvocation(
            program="git", args=("fetch", "--prune", config.upstream_remote), cwd=config.submodule_root
        ),
        failure_summary="unable to fetch upstream remote",
    )
    _run_checked(
        runner,
        invocation=CommandInvocation(
            program="git",
            args=(
                "checkout",
                "-B",
                config.fork_branch,
                f"{config.fork_remote}/{config.fork_branch}",
            ),
            cwd=config.submodule_root,
        ),
        failure_summary="unable to refresh local fork branch",
    )
    _run_checked(
        runner,
        invocation=CommandInvocation(
            program="git",
            args=("merge", "--ff-only", f"{config.upstream_remote}/{config.upstream_branch}"),
            cwd=config.submodule_root,
        ),
        failure_summary="unable to fast-forward fork branch with upstream; resolve divergence manually",
    )


def _run_verification_gates(runner: CommandRunner, *, config: SyncConfig) -> None:
    """Run required repository verification gates after sync."""
    for target in config.verification_targets:
        _run_checked(
            runner,
            invocation=CommandInvocation(program="make", args=(target,), cwd=config.repo_root),
            failure_summary=f"verification gate `{target}` failed",
        )


def run_monty_sync(
    runner: CommandRunner,
    *,
    config: SyncConfig,
    stdout: TextIO,
) -> None:
    """Synchronize the `full-monty` submodule and run verification gates.

    Parameters
    ----------
    runner : CommandRunner
    config : SyncConfig
    stdout : TextIO
        Stream receiving progress output.

    Returns
    -------
    None

    Raises
    ------
    MontySyncError
    """
    _log(stdout, "monty-sync: checking superproject worktree cleanliness")
    _ensure_clean_worktree(runner, cwd=config.repo_root, scope_name="superproject")

    _log(stdout, f"monty-sync: initializing {config.submodule_path.as_posix()}")
    _run_checked(
        runner,
        invocation=CommandInvocation(
            program="git",
            args=("submodule", "update", "--init", "--recursive", config.submodule_path.as_posix()),
            cwd=config.repo_root,
        ),
        failure_summary="unable to initialize full-monty submodule",
    )

    _log(stdout, "monty-sync: checking full-monty worktree cleanliness")
    _ensure_clean_worktree(runner, cwd=config.submodule_root, scope_name="full-monty submodule")

    _log(stdout, "monty-sync: ensuring remote configuration")
    _ensure_remotes(runner, config=config)

    before_revision = _read_head_revision(runner, cwd=config.submodule_root)

    _log(stdout, "monty-sync: fetching remotes and refreshing fork branch")
    _refresh_submodule_branch(runner, config=config)

    after_revision = _read_head_revision(runner, cwd=config.submodule_root)

    if before_revision == after_revision:
        _log(stdout, "monty-sync: submodule revision already current")
    else:
        _log(stdout, f"monty-sync: submodule revision updated {before_revision} -> {after_revision}")

    _run_checked(
        runner,
        invocation=CommandInvocation(
            program="git",
            args=("add", config.submodule_path.as_posix()),
            cwd=config.repo_root,
        ),
        failure_summary="unable to stage submodule pointer update",
    )
    _log(stdout, "monty-sync: staged submodule pointer update")

    _log(stdout, "monty-sync: running verification gates")
    _run_verification_gates(runner, config=config)
    _log(stdout, "monty-sync: completed successfully")


def _parse_args(argv: list[str]) -> None:
    """Validate CLI arguments for no-argument command contract."""
    if not argv:
        return
    if len(argv) == 1 and argv[0] in ("-h", "--help"):
        print("usage: monty_sync.py")
        raise SystemExit(0)
    raise SystemExit(f"unsupported arguments: {' '.join(argv)}")


def main(argv: list[str] | None = None) -> int:
    """Run the monty-sync CLI entrypoint.

    Parameters
    ----------
    argv : list[str] | None

    Returns
    -------
    int
    """
    cli_args = [] if argv is None else argv
    try:
        _parse_args(cli_args)
    except SystemExit as exit_signal:
        if isinstance(exit_signal.code, int):
            return exit_signal.code
        print(exit_signal.code, file=sys.stderr)
        return 2

    repo_root = Path(__file__).resolve().parents[1]
    config = SyncConfig(repo_root=repo_root)
    try:
        run_monty_sync(CuprumRunner(), config=config, stdout=sys.stdout)
    except MontySyncError as error:
        print(f"monty-sync error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
