"""Shared fixtures for script baseline validation tests."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest


SCRIPTS_ROOT = Path(__file__).resolve().parents[1]
if str(SCRIPTS_ROOT) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_ROOT))


@pytest.fixture
def scripts_root(tmp_path: Path) -> Path:
    """Create an isolated scripts tree for tests."""
    root = tmp_path / "scripts"
    root.mkdir(parents=True)
    (root / "tests").mkdir()
    return root


def write_text(path: Path, text: str) -> None:
    """Write UTF-8 text, creating parent directories as needed."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def create_matching_test(script_path: Path, scripts_root: Path) -> Path:
    """Create the matching `scripts/tests/test_*.py` file for a script."""
    relative_without_suffix = script_path.relative_to(scripts_root).with_suffix("")
    flattened_name = "_".join(relative_without_suffix.parts)
    test_path = scripts_root / "tests" / f"test_{flattened_name}.py"
    write_text(test_path, "def test_placeholder() -> None:\n    assert True\n")
    return test_path

