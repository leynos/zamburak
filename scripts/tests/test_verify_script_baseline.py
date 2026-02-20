"""Unit tests for `scripts/verify_script_baseline.py`."""

from __future__ import annotations

from collections.abc import Callable
from pathlib import Path

import pytest

import verify_script_baseline as baseline


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


def test_discover_roadmap_scripts_skips_helpers_and_tests(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
) -> None:
    write_text(scripts_root / "_helper.py", "print('ignore')\n")
    write_text(scripts_root / "tests" / "test_demo.py", "def test_demo():\n    assert True\n")
    write_text(scripts_root / "demo.py", VALID_SCRIPT)
    write_text(scripts_root / "nested" / "tool.py", VALID_SCRIPT)

    discovered = baseline.discover_roadmap_scripts(scripts_root)
    discovered_relatives = [path.relative_to(scripts_root) for path in discovered]
    assert discovered_relatives == [Path("demo.py"), Path("nested/tool.py")]


def test_validate_script_accepts_compliant_script_with_matching_test(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
    create_matching_test: Callable[[Path, Path], Path],
) -> None:
    script_path = scripts_root / "release.py"
    write_text(script_path, VALID_SCRIPT)
    create_matching_test(script_path, scripts_root)

    issues = baseline.validate_script(script_path, scripts_root)
    assert issues == []


def test_validate_script_reports_missing_matching_test(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
) -> None:
    script_path = scripts_root / "release.py"
    write_text(script_path, VALID_SCRIPT)

    issues = baseline.validate_script(script_path, scripts_root)
    assert any("missing matching test" in issue.message for issue in issues)


def test_validate_script_reports_missing_uv_metadata(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
    create_matching_test: Callable[[Path, Path], Path],
) -> None:
    script_path = scripts_root / "broken.py"
    write_text(
        script_path,
        "from __future__ import annotations\nprint('broken')\n",
    )
    create_matching_test(script_path, scripts_root)

    issues = baseline.validate_script(script_path, scripts_root)
    assert any("uv shebang" in issue.message for issue in issues)
    assert any("uv metadata block" in issue.message for issue in issues)


@pytest.mark.parametrize(
    ("snippet", "expected_fragment"),
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
    snippet: str,
    expected_fragment: str,
) -> None:
    script_path = scripts_root / "forbidden.py"
    write_text(
        script_path,
        VALID_SCRIPT + "\n" + snippet,
    )
    create_matching_test(script_path, scripts_root)

    issues = baseline.validate_script(script_path, scripts_root)
    assert any(expected_fragment in issue.message for issue in issues)


def test_validate_script_requires_run_sync_or_run_for_cuprum_programs(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
    create_matching_test: Callable[[Path, Path], Path],
) -> None:
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
    assert any("run_sync()" in issue.message for issue in issues)


def test_main_returns_non_zero_and_renders_relative_paths(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = scripts_root / "broken.py"
    write_text(script_path, VALID_SCRIPT)

    exit_code = baseline.main(["--root", str(scripts_root)])
    output = capsys.readouterr().out

    assert exit_code == 1
    assert "broken.py" in output
    assert "missing matching test" in output


@pytest.mark.parametrize(
    ("source", "expected_message_fragment"),
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
    source: str,
    expected_message_fragment: str,
) -> None:
    script_path = scripts_root / "metadata_case.py"
    write_text(script_path, source)
    create_matching_test(script_path, scripts_root)

    issues = baseline.validate_script(script_path, scripts_root)
    assert any(expected_message_fragment in issue.message for issue in issues)


def test_validate_script_reports_missing_file_read_error(scripts_root: Path) -> None:
    script_path = scripts_root / "missing.py"
    issues = baseline.validate_script(script_path, scripts_root)
    assert any("unable to read script" in issue.message for issue in issues)


def test_main_reports_non_roadmap_explicit_path(
    scripts_root: Path,
    write_text: Callable[[Path, str], None],
    capsys: pytest.CaptureFixture[str],
) -> None:
    non_roadmap_path = scripts_root / "tests" / "test_helper.py"
    write_text(non_roadmap_path, "def test_helper() -> None:\n    assert True\n")

    exit_code = baseline.main(
        ["--root", str(scripts_root), str(non_roadmap_path)]
    )
    output = capsys.readouterr().out
    assert exit_code == 1
    assert "not a roadmap-delivered script entrypoint" in output


def test_expected_test_path_avoids_collisions_for_nested_scripts(
    scripts_root: Path,
) -> None:
    flat_script = scripts_root / "a_b.py"
    nested_script = scripts_root / "a" / "b.py"
    flat_test = baseline.expected_test_path(flat_script, scripts_root)
    nested_test = baseline.expected_test_path(nested_script, scripts_root)

    assert flat_test != nested_test
