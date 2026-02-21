"""CLI-focused tests for ``scripts/verify_script_baseline.py``.

These tests exercise the script entrypoint (``main()``) covering exit-code
behaviour, rendered output paths, and explicit-path rejection.

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
