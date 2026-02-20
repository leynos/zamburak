"""Behavioural tests for script baseline validation using pytest-bdd.

This module covers end-to-end checker behaviour for passing and failing script
baseline scenarios, including metadata and forbidden-command violations.

Usage
-----
Run with:

```
make script-test
```
"""

from __future__ import annotations

from collections.abc import Callable
from contextlib import redirect_stdout
from dataclasses import dataclass
from io import StringIO
from pathlib import Path

import pytest
from pytest_bdd import given, scenarios, then, when

import verify_script_baseline as baseline


scenarios("features/script_baseline.feature")

VALID_SCRIPT: str = """#!/usr/bin/env -S uv run python
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
    """Hold mutable scenario state shared across BDD steps.

    Attributes
    ----------
    scripts_root : Path
        Temporary scripts-root directory used by each scenario.
    exit_code : int | None
        Checker exit code captured after invocation.
    output : str
        Captured stdout output from checker execution.
    """

    scripts_root: Path
    exit_code: int | None = None
    output: str = ""


@pytest.fixture
def scenario_state(scripts_root: Path) -> ScenarioState:
    """Create per-scenario state container.

    Parameters
    ----------
    scripts_root : Path
        Temporary scripts root fixture.

    Returns
    -------
    ScenarioState
        Fresh state object for BDD step coordination.
    """
    return ScenarioState(scripts_root=scripts_root)


@given("a compliant roadmap script tree")
def given_compliant_tree(
    scenario_state: ScenarioState,
    write_text: Callable[[Path, str], None],
    create_matching_test: Callable[[Path, Path], Path],
) -> None:
    """Create a valid roadmap script and matching test.

    Parameters
    ----------
    scenario_state : ScenarioState
        Shared scenario state.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.
    create_matching_test : Callable[[Path, Path], Path]
        Matching-test creation helper fixture.

    Returns
    -------
    None
        This step only prepares fixture state.
    """
    script_path = scenario_state.scripts_root / "release.py"
    write_text(script_path, VALID_SCRIPT)
    create_matching_test(script_path, scenario_state.scripts_root)


@given("a roadmap script without matching tests")
def given_missing_tests_tree(
    scenario_state: ScenarioState,
    write_text: Callable[[Path, str], None],
) -> None:
    """Create a roadmap script without a matching test file.

    Parameters
    ----------
    scenario_state : ScenarioState
        Shared scenario state.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.

    Returns
    -------
    None
        This step only prepares fixture state.
    """
    script_path = scenario_state.scripts_root / "release.py"
    write_text(script_path, VALID_SCRIPT)


@given("a roadmap script missing uv metadata")
def given_missing_uv_metadata(
    scenario_state: ScenarioState,
    write_text: Callable[[Path, str], None],
    create_matching_test: Callable[[Path, Path], Path],
) -> None:
    """Create a script missing uv metadata with a matching test.

    Parameters
    ----------
    scenario_state : ScenarioState
        Shared scenario state.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.
    create_matching_test : Callable[[Path, Path], Path]
        Matching-test creation helper fixture.

    Returns
    -------
    None
        This step only prepares fixture state.
    """
    script_path = scenario_state.scripts_root / "metadata_missing.py"
    write_text(script_path, "from __future__ import annotations\nprint('broken')\n")
    create_matching_test(script_path, scenario_state.scripts_root)


@given("a roadmap script with incorrect requires-python")
def given_incorrect_requires_python(
    scenario_state: ScenarioState,
    write_text: Callable[[Path, str], None],
    create_matching_test: Callable[[Path, Path], Path],
) -> None:
    """Create a script with an incorrect requires-python declaration.

    Parameters
    ----------
    scenario_state : ScenarioState
        Shared scenario state.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.
    create_matching_test : Callable[[Path, Path], Path]
        Matching-test creation helper fixture.

    Returns
    -------
    None
        This step only prepares fixture state.
    """
    script_path = scenario_state.scripts_root / "requires_python_bad.py"
    write_text(
        script_path,
        """#!/usr/bin/env -S uv run python
# /// script
# requires-python = ">=3.12"
# dependencies = ["cuprum==0.1.0"]
# ///
print("hello")
""",
    )
    create_matching_test(script_path, scenario_state.scripts_root)


@given("a roadmap script with forbidden command imports")
def given_forbidden_imports(
    scenario_state: ScenarioState,
    write_text: Callable[[Path, str], None],
    create_matching_test: Callable[[Path, Path], Path],
) -> None:
    """Create a script using forbidden command imports.

    Parameters
    ----------
    scenario_state : ScenarioState
        Shared scenario state.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.
    create_matching_test : Callable[[Path, Path], Path]
        Matching-test creation helper fixture.

    Returns
    -------
    None
        This step only prepares fixture state.
    """
    script_path = scenario_state.scripts_root / "forbidden_import.py"
    write_text(script_path, VALID_SCRIPT + "\nimport plumbum\n")
    create_matching_test(script_path, scenario_state.scripts_root)


@when("I run the script baseline checker")
def when_run_checker(scenario_state: ScenarioState) -> None:
    """Run baseline checker and capture output in scenario state.

    Parameters
    ----------
    scenario_state : ScenarioState
        Shared scenario state.

    Returns
    -------
    None
        This step captures exit code and output for assertions.
    """
    with StringIO() as output_stream:
        with redirect_stdout(output_stream):
            scenario_state.exit_code = baseline.main(
                ["--root", str(scenario_state.scripts_root)]
            )
        scenario_state.output = output_stream.getvalue()


@then("the checker exits successfully")
def then_checker_succeeds(scenario_state: ScenarioState) -> None:
    """Assert successful checker execution.

    Parameters
    ----------
    scenario_state : ScenarioState
        Shared scenario state.

    Returns
    -------
    None
        This step asserts success conditions.
    """
    assert scenario_state.exit_code == 0, "expected checker to exit successfully"
    assert "validation passed" in scenario_state.output, (
        "expected success output to mention validation passed"
    )


@then("the checker exits with an error")
def then_checker_fails(scenario_state: ScenarioState) -> None:
    """Assert failing checker execution.

    Parameters
    ----------
    scenario_state : ScenarioState
        Shared scenario state.

    Returns
    -------
    None
        This step asserts failure exit code.
    """
    assert scenario_state.exit_code == 1, "expected checker to return a failing exit code"


@then("the output mentions missing matching test")
def then_output_mentions_missing_test(scenario_state: ScenarioState) -> None:
    """Assert output reports missing matching tests.

    Parameters
    ----------
    scenario_state : ScenarioState
        Shared scenario state.

    Returns
    -------
    None
        This step asserts expected error text.
    """
    assert "missing matching test" in scenario_state.output, (
        "expected output to report missing matching test"
    )


@then("the output mentions missing uv metadata")
def then_output_mentions_missing_metadata(scenario_state: ScenarioState) -> None:
    """Assert output reports missing uv metadata block.

    Parameters
    ----------
    scenario_state : ScenarioState
        Shared scenario state.

    Returns
    -------
    None
        This step asserts expected error text.
    """
    assert "missing uv metadata block" in scenario_state.output, (
        "expected output to report missing uv metadata"
    )


@then("the output mentions invalid requires-python")
def then_output_mentions_invalid_requires_python(scenario_state: ScenarioState) -> None:
    """Assert output reports invalid requires-python declaration.

    Parameters
    ----------
    scenario_state : ScenarioState
        Shared scenario state.

    Returns
    -------
    None
        This step asserts expected error text.
    """
    assert "requires-python" in scenario_state.output, (
        "expected output to report requires-python violations"
    )


@then("the output mentions forbidden command imports")
def then_output_mentions_forbidden_imports(scenario_state: ScenarioState) -> None:
    """Assert output reports forbidden command imports.

    Parameters
    ----------
    scenario_state : ScenarioState
        Shared scenario state.

    Returns
    -------
    None
        This step asserts expected error text.
    """
    assert "Plumbum imports are forbidden" in scenario_state.output, (
        "expected output to report forbidden import usage"
    )
