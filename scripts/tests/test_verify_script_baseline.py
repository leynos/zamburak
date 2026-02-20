"""Unit tests for `scripts/verify_script_baseline.py`."""

from __future__ import annotations

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


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def create_matching_test(script_path: Path, scripts_root: Path) -> Path:
    relative_without_suffix = script_path.relative_to(scripts_root).with_suffix("")
    flattened_name = "_".join(relative_without_suffix.parts)
    test_path = scripts_root / "tests" / f"test_{flattened_name}.py"
    write_text(test_path, "def test_placeholder() -> None:\n    assert True\n")
    return test_path


def test_discover_roadmap_scripts_skips_helpers_and_tests(scripts_root: Path) -> None:
    write_text(scripts_root / "_helper.py", "print('ignore')\n")
    write_text(scripts_root / "tests" / "test_demo.py", "def test_demo():\n    assert True\n")
    write_text(scripts_root / "demo.py", VALID_SCRIPT)
    write_text(scripts_root / "nested" / "tool.py", VALID_SCRIPT)

    discovered = baseline.discover_roadmap_scripts(scripts_root)
    discovered_relatives = [path.relative_to(scripts_root) for path in discovered]
    assert discovered_relatives == [Path("demo.py"), Path("nested/tool.py")]


def test_validate_script_accepts_compliant_script_with_matching_test(
    scripts_root: Path,
) -> None:
    script_path = scripts_root / "release.py"
    write_text(script_path, VALID_SCRIPT)
    create_matching_test(script_path, scripts_root)

    issues = baseline.validate_script(script_path, scripts_root)
    assert issues == []


def test_validate_script_reports_missing_matching_test(scripts_root: Path) -> None:
    script_path = scripts_root / "release.py"
    write_text(script_path, VALID_SCRIPT)

    issues = baseline.validate_script(script_path, scripts_root)
    assert any("missing matching test" in issue.message for issue in issues)


def test_validate_script_reports_missing_uv_metadata(scripts_root: Path) -> None:
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
        ("import plumbum\n", "Plumbum imports are forbidden"),
        ("from cuprum import local\n", "forbidden in baseline scripts"),
    ],
)
def test_validate_script_reports_forbidden_command_patterns(
    scripts_root: Path,
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
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = scripts_root / "broken.py"
    write_text(script_path, VALID_SCRIPT)

    exit_code = baseline.main(["--root", str(scripts_root)])
    output = capsys.readouterr().out

    assert exit_code == 1
    assert "broken.py" in output
    assert "missing matching test" in output
