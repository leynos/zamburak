"""Behavioural tests for `scripts/monty_sync.py` using pytest-bdd.

Usage
-----
Run with:

```
make script-test
```
"""

from __future__ import annotations

from dataclasses import dataclass, field
from io import StringIO
from pathlib import Path

import pytest
from pytest_bdd import given, scenarios, then, when

import monty_sync

from monty_sync_test_helpers import (
    CommandInvocation,
    CommandStub,
    QueueRunner,
    failure_outcome,
    successful_outcome,
)


scenarios("features/monty_sync.feature")


@dataclass
class ScenarioState:
    """Mutable state shared across scenario steps."""

    config: monty_sync.SyncConfig
    runner: QueueRunner | None = None
    stdout: StringIO = field(default_factory=StringIO)
    error: monty_sync.MontySyncError | None = None


def _config(tmp_path: Path) -> monty_sync.SyncConfig:
    repo_root = tmp_path / "repo"
    return monty_sync.SyncConfig(repo_root=repo_root)


def _invoke(
    config: monty_sync.SyncConfig,
    *,
    program: str,
    args: tuple[str, ...],
    submodule: bool = False,
) -> CommandInvocation:
    cwd = config.submodule_root if submodule else config.repo_root
    return CommandInvocation(program=program, args=args, cwd=cwd)


@pytest.fixture
def scenario_state(tmp_path: Path) -> ScenarioState:
    """Create isolated scenario state with a repository-root configuration."""
    return ScenarioState(config=_config(tmp_path))


@given("a monty sync happy-path command sequence")
def given_happy_path_sequence(scenario_state: ScenarioState) -> None:
    """Prepare successful command queue for full sync workflow."""
    config = scenario_state.config
    scenario_state.runner = QueueRunner(
        (
            CommandStub(
                _invoke(config, program="git", args=("status", "--porcelain")),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(
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
                _invoke(config, program="git", args=("status", "--porcelain"), submodule=True),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(config, program="git", args=("remote",), submodule=True),
                successful_outcome("origin\nupstream\n"),
            ),
            CommandStub(
                _invoke(
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
                _invoke(config, program="git", args=("rev-parse", "HEAD"), submodule=True),
                successful_outcome("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"),
            ),
            CommandStub(
                _invoke(config, program="git", args=("fetch", "--prune", "origin"), submodule=True),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(
                    config,
                    program="git",
                    args=("fetch", "--prune", "upstream"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(
                    config,
                    program="git",
                    args=("checkout", "-B", "main", "origin/main"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(
                    config,
                    program="git",
                    args=("merge", "--ff-only", "upstream/main"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(config, program="git", args=("rev-parse", "HEAD"), submodule=True),
                successful_outcome("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n"),
            ),
            CommandStub(
                _invoke(config, program="git", args=("add", "third_party/full-monty")),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(config, program="make", args=("check-fmt",)),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(config, program="make", args=("lint",)),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(config, program="make", args=("test",)),
                successful_outcome(),
            ),
        )
    )


@given("a dirty superproject command sequence")
def given_dirty_superproject_sequence(scenario_state: ScenarioState) -> None:
    """Prepare command queue that fails at superproject cleanliness check."""
    config = scenario_state.config
    scenario_state.runner = QueueRunner(
        (
            CommandStub(
                _invoke(config, program="git", args=("status", "--porcelain")),
                successful_outcome(" M README.md\n"),
            ),
        )
    )


@given("a missing fork remote command sequence")
def given_missing_remote_sequence(scenario_state: ScenarioState) -> None:
    """Prepare queue where fork remote is missing in submodule checkout."""
    config = scenario_state.config
    scenario_state.runner = QueueRunner(
        (
            CommandStub(
                _invoke(config, program="git", args=("status", "--porcelain")),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(
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
                _invoke(config, program="git", args=("status", "--porcelain"), submodule=True),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(config, program="git", args=("remote",), submodule=True),
                successful_outcome("upstream\n"),
            ),
        )
    )


@given("a verification gate failure command sequence")
def given_gate_failure_sequence(scenario_state: ScenarioState) -> None:
    """Prepare queue where lint gate fails after sync operations."""
    config = scenario_state.config
    scenario_state.runner = QueueRunner(
        (
            CommandStub(
                _invoke(config, program="git", args=("status", "--porcelain")),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(
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
                _invoke(config, program="git", args=("status", "--porcelain"), submodule=True),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(config, program="git", args=("remote",), submodule=True),
                successful_outcome("origin\nupstream\n"),
            ),
            CommandStub(
                _invoke(
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
                _invoke(config, program="git", args=("rev-parse", "HEAD"), submodule=True),
                successful_outcome("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"),
            ),
            CommandStub(
                _invoke(config, program="git", args=("fetch", "--prune", "origin"), submodule=True),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(
                    config,
                    program="git",
                    args=("fetch", "--prune", "upstream"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(
                    config,
                    program="git",
                    args=("checkout", "-B", "main", "origin/main"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(
                    config,
                    program="git",
                    args=("merge", "--ff-only", "upstream/main"),
                    submodule=True,
                ),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(config, program="git", args=("rev-parse", "HEAD"), submodule=True),
                successful_outcome("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"),
            ),
            CommandStub(
                _invoke(config, program="git", args=("add", "third_party/full-monty")),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(config, program="make", args=("check-fmt",)),
                successful_outcome(),
            ),
            CommandStub(
                _invoke(config, program="make", args=("lint",)),
                failure_outcome("simulated lint gate failure"),
            ),
        )
    )


@when("I run the monty sync workflow")
def when_run_sync(scenario_state: ScenarioState) -> None:
    """Execute sync workflow and capture success or failure state."""
    assert scenario_state.runner is not None, "scenario runner must be prepared"
    try:
        monty_sync.run_monty_sync(
            scenario_state.runner,
            config=scenario_state.config,
            stdout=scenario_state.stdout,
        )
    except monty_sync.MontySyncError as error:
        scenario_state.error = error
    finally:
        scenario_state.runner.assert_exhausted()


@then("monty sync succeeds")
def then_sync_succeeds(scenario_state: ScenarioState) -> None:
    """Assert sync completed without orchestration errors."""
    assert scenario_state.error is None, (
        f"monty sync should succeed but failed with: {scenario_state.error}"
    )


@then("monty sync fails")
def then_sync_fails(scenario_state: ScenarioState) -> None:
    """Assert sync failed with an orchestration error."""
    assert scenario_state.error is not None, "monty sync should fail in this scenario"


@then("the output mentions completion")
def then_output_mentions_completion(scenario_state: ScenarioState) -> None:
    """Assert successful output includes completion marker."""
    assert "monty-sync: completed successfully" in scenario_state.stdout.getvalue()


@then("the failure mentions superproject cleanliness")
def then_failure_mentions_superproject_cleanliness(scenario_state: ScenarioState) -> None:
    """Assert error diagnostics mention superproject worktree cleanliness."""
    assert scenario_state.error is not None
    assert "superproject worktree is not clean" in str(scenario_state.error)


@then("the failure mentions missing fork remote")
def then_failure_mentions_missing_fork_remote(scenario_state: ScenarioState) -> None:
    """Assert error diagnostics mention missing fork remote configuration."""
    assert scenario_state.error is not None
    assert "fork remote `origin` is missing" in str(scenario_state.error)


@then("the failure mentions the lint gate")
def then_failure_mentions_lint_gate(scenario_state: ScenarioState) -> None:
    """Assert error diagnostics identify lint verification gate failure."""
    assert scenario_state.error is not None
    assert "verification gate `lint` failed" in str(scenario_state.error)
