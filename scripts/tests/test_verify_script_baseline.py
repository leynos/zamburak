"""Unit tests for ``scripts/verify_script_baseline.py``.

These tests validate script-baseline rule enforcement for discovery, metadata,
command invocation, explicit-path handling, and matching-test contracts.

Usage
-----
Run with:

```
make script-test
```
"""

from __future__ import annotations

from collections.abc import Callable
from pathlib import Path
from typing import NamedTuple

import pytest

import verify_script_baseline as baseline


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


class ScriptValidationParams(NamedTuple):
    """Named parameters for script validation issue assertions."""

    script_name: str
    script_content: str
    expected_fragment: str


def _validate_script_with_issue_assertion(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
    create_matching_test: Callable[[Path, Path], Path],
    validation_params: ScriptValidationParams,
) -> None:
    """Validate a script and assert that a specific issue fragment is reported.

    Parameters
    ----------
    scripts_root : Path
        Temporary scripts root fixture.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.
    create_matching_test : Callable[[Path, Path], Path]
        Matching-test creation helper fixture.
    validation_params : ScriptValidationParams
        Named script-name/content/expected-fragment bundle.

    Returns
    -------
    None
        This helper asserts expected validation output.
    """
    script_name = validation_params.script_name
    script_content = validation_params.script_content
    expected_fragment = validation_params.expected_fragment
    script_path = scripts_root / script_name
    write_text(script_path, script_content)
    create_matching_test(script_path, scripts_root)
    issues = baseline.validate_script(script_path, scripts_root)
    assert any(
        expected_fragment in issue.message for issue in issues
    ), f"expected issue fragment not found: {expected_fragment}"


def test_discover_roadmap_scripts_skips_helpers_and_tests(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
) -> None:
    """Verify discovery excludes helper and tests paths.

    Parameters
    ----------
    scripts_root : Path
        Temporary scripts root fixture.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.

    Returns
    -------
    None
        This test asserts discovery behaviour only.
    """
    write_text(scripts_root / "_helper.py", "print('ignore')\n")
    write_text(scripts_root / "tests" / "test_demo.py", "def test_demo():\n    assert True\n")
    write_text(scripts_root / "demo.py", VALID_SCRIPT)
    write_text(scripts_root / "nested" / "tool.py", VALID_SCRIPT)

    discovered = baseline.discover_roadmap_scripts(scripts_root)
    discovered_relatives = [path.relative_to(scripts_root) for path in discovered]
    assert discovered_relatives == [Path("demo.py"), Path("nested/tool.py")], (
        "roadmap discovery should include only non-helper, non-test Python scripts"
    )


def test_validate_script_accepts_compliant_script_with_matching_test(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
    create_matching_test: Callable[[Path, Path], Path],
) -> None:
    """Verify compliant scripts with matching tests produce no issues.

    Parameters
    ----------
    scripts_root : Path
        Temporary scripts root fixture.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.
    create_matching_test : Callable[[Path, Path], Path]
        Matching-test creation helper fixture.

    Returns
    -------
    None
        This test asserts successful validation.
    """
    script_path = scripts_root / "release.py"
    write_text(script_path, VALID_SCRIPT)
    create_matching_test(script_path, scripts_root)

    issues = baseline.validate_script(script_path, scripts_root)
    assert issues == [], "compliant script should have no validation issues"


def test_validate_script_reports_missing_matching_test(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
) -> None:
    """Verify missing matching-test files are reported.

    Parameters
    ----------
    scripts_root : Path
        Temporary scripts root fixture.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.

    Returns
    -------
    None
        This test asserts missing-test failure messaging.
    """
    script_path = scripts_root / "release.py"
    write_text(script_path, VALID_SCRIPT)

    issues = baseline.validate_script(script_path, scripts_root)
    assert any(
        "missing matching test" in issue.message for issue in issues
    ), "expected issue fragment not found: missing matching test"


def test_validate_script_reports_missing_uv_metadata(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
    create_matching_test: Callable[[Path, Path], Path],
) -> None:
    """Verify scripts without uv metadata report expected errors.

    Parameters
    ----------
    scripts_root : Path
        Temporary scripts root fixture.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.
    create_matching_test : Callable[[Path, Path], Path]
        Matching-test creation helper fixture.

    Returns
    -------
    None
        This test asserts metadata-related failures.
    """
    script_path = scripts_root / "broken.py"
    write_text(
        script_path,
        "from __future__ import annotations\nprint('broken')\n",
    )
    create_matching_test(script_path, scripts_root)

    issues = baseline.validate_script(script_path, scripts_root)
    assert any(
        "uv shebang" in issue.message for issue in issues
    ), "expected issue fragment not found: uv shebang"
    assert any(
        "uv metadata block" in issue.message for issue in issues
    ), "expected issue fragment not found: uv metadata block"


@pytest.mark.parametrize(
    "test_case",
    [
        ("import subprocess\n", "subprocess imports are forbidden"),
        ("from subprocess import run\n", "subprocess imports are forbidden"),
        ("import plumbum\n", "Plumbum imports are forbidden"),
        ("from cuprum import local\n", "forbidden in baseline scripts"),
        ("from cuprum.cmd import git\n", "cuprum.cmd"),
        ("import os\nos.system('echo hi')\n", "os.system"),
        ("import os\nos.popen('echo hi')\n", "os.system/os.popen"),
        (
            "import subprocess\nsubprocess.run(['echo'])\n",
            "subprocess invocation is forbidden",
        ),
    ],
)
def test_validate_script_reports_forbidden_command_patterns(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
    create_matching_test: Callable[[Path, Path], Path],
    test_case: tuple[str, str],
) -> None:
    """Verify forbidden command patterns are surfaced.

    Parameters
    ----------
    scripts_root : Path
        Temporary scripts root fixture.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.
    create_matching_test : Callable[[Path, Path], Path]
        Matching-test creation helper fixture.
    test_case : tuple[str, str]
        Snippet and expected message fragment pair.

    Returns
    -------
    None
        This test asserts forbidden-pattern diagnostics.
    """
    snippet, expected_fragment = test_case
    _validate_script_with_issue_assertion(
        scripts_root,
        write_text,
        create_matching_test,
        ScriptValidationParams(
            script_name="forbidden.py",
            script_content=VALID_SCRIPT + "\n" + snippet,
            expected_fragment=expected_fragment,
        ),
    )


def test_validate_script_requires_run_sync_or_run_for_cuprum_programs(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
    create_matching_test: Callable[[Path, Path], Path],
) -> None:
    """Verify Cuprum command calls require run/run_sync invocation.

    Parameters
    ----------
    scripts_root : Path
        Temporary scripts root fixture.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.
    create_matching_test : Callable[[Path, Path], Path]
        Matching-test creation helper fixture.

    Returns
    -------
    None
        This test asserts run invocation enforcement.
    """
    script_path = scripts_root / "missing_run.py"
    write_text(
        script_path,
        """#!/usr/bin/env -S uv run python
# /// script
# requires-python = ">=3.13"
# dependencies = ["cuprum==0.1.0"]
# ///
from cuprum import Program, scoped

TOFU = Program("tofu")

with scoped(allowlist=frozenset([TOFU])):
    print("missing run_sync")
""",
    )
    create_matching_test(script_path, scripts_root)

    issues = baseline.validate_script(script_path, scripts_root)
    assert any(
        "run_sync()" in issue.message for issue in issues
    ), "expected issue fragment not found: run_sync()"


def test_main_returns_non_zero_and_renders_relative_paths(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Verify CLI output includes relative paths for failures.

    Parameters
    ----------
    scripts_root : Path
        Temporary scripts root fixture.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.
    capsys : pytest.CaptureFixture[str]
        Captured stdout fixture.

    Returns
    -------
    None
        This test asserts exit code and rendered output.
    """
    script_path = scripts_root / "broken.py"
    write_text(script_path, VALID_SCRIPT)

    exit_code = baseline.main(["--root", str(scripts_root)])
    output = capsys.readouterr().out

    assert exit_code == 1, "baseline checker should fail when matching test is missing"
    assert "broken.py" in output, "output should include the failing script path"
    assert "missing matching test" in output, "output should report missing matching tests"


@pytest.mark.parametrize(
    "test_case",
    [
        (
            """#!/usr/bin/env -S uv run python
# /// script
# requires-python = ">=3.12"
# dependencies = ["cuprum==0.1.0"]
# ///
print("hello")
""",
            "requires-python",
        ),
        (
            """#!/usr/bin/env -S uv run python
# /// script
# requires-python = ">=3.13"
# ///
print("hello")
""",
            "dependencies",
        ),
        (
            """#!/usr/bin/env -S uv run python
# /// script
# requires-python = ">=3.13"
print("hello")
""",
            "uv metadata block",
        ),
    ],
)
def test_validate_script_reports_metadata_edge_cases(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
    create_matching_test: Callable[[Path, Path], Path],
    test_case: tuple[str, str],
) -> None:
    """Verify metadata edge-case failures are reported.

    Parameters
    ----------
    scripts_root : Path
        Temporary scripts root fixture.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.
    create_matching_test : Callable[[Path, Path], Path]
        Matching-test creation helper fixture.
    test_case : tuple[str, str]
        Script source and expected error-fragment pair.

    Returns
    -------
    None
        This test asserts metadata diagnostics.
    """
    source, expected_message_fragment = test_case
    _validate_script_with_issue_assertion(
        scripts_root,
        write_text,
        create_matching_test,
        ScriptValidationParams(
            script_name="metadata_case.py",
            script_content=source,
            expected_fragment=expected_message_fragment,
        ),
    )


def test_validate_script_reports_missing_file_read_error(scripts_root: Path) -> None:
    """Verify missing explicit script files report read errors.

    Parameters
    ----------
    scripts_root : Path
        Temporary scripts root fixture.

    Returns
    -------
    None
        This test asserts missing-file diagnostics.
    """
    script_path = scripts_root / "missing.py"
    issues = baseline.validate_script(script_path, scripts_root)
    assert any(
        "unable to read script" in issue.message for issue in issues
    ), "expected issue fragment not found: unable to read script"


def test_main_reports_non_roadmap_explicit_path(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Verify explicit non-roadmap paths are rejected by the CLI.

    Parameters
    ----------
    scripts_root : Path
        Temporary scripts root fixture.
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.
    capsys : pytest.CaptureFixture[str]
        Captured stdout fixture.

    Returns
    -------
    None
        This test asserts explicit-path failure handling.
    """
    non_roadmap_path = scripts_root / "tests" / "test_helper.py"
    write_text(non_roadmap_path, "def test_helper() -> None:\n    assert True\n")

    exit_code = baseline.main(
        ["--root", str(scripts_root), str(non_roadmap_path)]
    )
    output = capsys.readouterr().out
    assert exit_code == 1, "non-roadmap explicit paths should fail validation"
    assert "not a roadmap-delivered script entrypoint" in output, (
        "output should explain why explicit test paths are rejected"
    )


def test_expected_test_path_avoids_collisions_for_nested_scripts(
    scripts_root: Path,
) -> None:
    """Verify nested and flat script names map to distinct test paths.

    Parameters
    ----------
    scripts_root : Path
        Temporary scripts root fixture.

    Returns
    -------
    None
        This test asserts mapping uniqueness.
    """
    flat_script = scripts_root / "a_b.py"
    nested_script = scripts_root / "a" / "b.py"
    flat_test = baseline.expected_test_path(flat_script, scripts_root)
    nested_test = baseline.expected_test_path(nested_script, scripts_root)

    assert flat_test != nested_test, (
        "flat and nested scripts must not resolve to the same matching test path"
    )
