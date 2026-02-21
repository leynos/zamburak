"""Shared fixtures for script baseline validation tests.

This module centralises test-fixture helpers used by both unit and behavioural
script-baseline suites, including temporary scripts-root setup and convenience
file writers.

Usage
-----
Run the script test suite:

```
make script-test
```
"""

from __future__ import annotations

from collections.abc import Callable
import sys
from pathlib import Path

import pytest


SCRIPTS_ROOT: Path = Path(__file__).resolve().parents[1]
if str(SCRIPTS_ROOT) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_ROOT))

import verify_script_baseline as baseline


@pytest.fixture
def scripts_root(tmp_path: Path) -> Path:
    """Create an isolated scripts tree for tests.

    Parameters
    ----------
    tmp_path : Path
        Pytest-provided temporary directory.

    Returns
    -------
    Path
        Fresh scripts root containing an empty `tests/` directory.
    """
    root = tmp_path / "scripts"
    root.mkdir(parents=True)
    (root / "tests").mkdir()
    return root


@pytest.fixture
def write_text() -> Callable[[Path, str], None]:
    """Return a helper that writes UTF-8 text with parent creation.

    Returns
    -------
    Callable[[Path, str], None]
        Helper that writes text after creating parent directories.
    """

    def _write_text(path: Path, text: str) -> None:
        """Write UTF-8 text to a path, creating parent directories."""
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    return _write_text


@pytest.fixture
def create_matching_test(
    write_text: Callable[[Path, str], None],
) -> Callable[[Path, Path], Path]:
    """Return a helper that creates a matching test file for a script.

    Parameters
    ----------
    write_text : Callable[[Path, str], None]
        Text-writing helper fixture.

    Returns
    -------
    Callable[[Path, Path], Path]
        Helper that creates and returns a matching pytest path.
    """

    def _create_matching_test(script_path: Path, scripts_root: Path) -> Path:
        """Create and return the matching pytest file path for a script."""
        test_path = baseline.expected_test_path(script_path, scripts_root)
        write_text(
            test_path,
            'def test_placeholder() -> None:\n    assert True, "placeholder assertion"\n',
        )
        return test_path

    return _create_matching_test
