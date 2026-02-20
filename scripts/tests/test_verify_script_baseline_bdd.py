"""Behavioural tests for script baseline validation using pytest-bdd."""

from __future__ import annotations

from dataclasses import dataclass
from io import StringIO
from pathlib import Path
from contextlib import redirect_stdout

import pytest
from pytest_bdd import given, scenarios, then, when

import verify_script_baseline as baseline


scenarios("features/script_baseline.feature")

VALID_SCRIPT = """#!/usr/bin/env -S uv run python
# /// script
# requires-python = ">=3.13"
# dependencies = ["cuprum==0.1.0"]
# ///
from __future__ import annotations
from cuprum import Program, scoped, sh

TOFU = Program("tofu")
tofu = sh.make(TOFU)

with scoped(allowlist=frozenset([TOFU])):
    result = tofu("plan").run_sync()
"""


@dataclass
class ScenarioState:
    scripts_root: Path
    exit_code: int | None = None
    output: str = ""


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def create_matching_test(script_path: Path, scripts_root: Path) -> None:
    relative_without_suffix = script_path.relative_to(scripts_root).with_suffix("")
    flattened_name = "_".join(relative_without_suffix.parts)
    test_path = scripts_root / "tests" / f"test_{flattened_name}.py"
    write_text(test_path, "def test_placeholder() -> None:\n    assert True\n")


@pytest.fixture
def scenario_state(scripts_root: Path) -> ScenarioState:
    return ScenarioState(scripts_root=scripts_root)


@given("a compliant roadmap script tree")
def given_compliant_tree(scenario_state: ScenarioState) -> None:
    script_path = scenario_state.scripts_root / "release.py"
    write_text(script_path, VALID_SCRIPT)
    create_matching_test(script_path, scenario_state.scripts_root)


@given("a roadmap script without matching tests")
def given_missing_tests_tree(scenario_state: ScenarioState) -> None:
    script_path = scenario_state.scripts_root / "release.py"
    write_text(script_path, VALID_SCRIPT)


@when("I run the script baseline checker")
def when_run_checker(scenario_state: ScenarioState) -> None:
    with StringIO() as output_stream:
        with redirect_stdout(output_stream):
            scenario_state.exit_code = baseline.main(
                ["--root", str(scenario_state.scripts_root)]
            )
        scenario_state.output = output_stream.getvalue()


@then("the checker exits successfully")
def then_checker_succeeds(scenario_state: ScenarioState) -> None:
    assert scenario_state.exit_code == 0
    assert "validation passed" in scenario_state.output


@then("the checker exits with an error")
def then_checker_fails(scenario_state: ScenarioState) -> None:
    assert scenario_state.exit_code == 1


@then("the output mentions missing matching test")
def then_output_mentions_missing_test(scenario_state: ScenarioState) -> None:
    assert "missing matching test" in scenario_state.output
