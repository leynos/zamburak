#!/usr/bin/env -S uv run python
# /// script
# requires-python = ">=3.13"
# dependencies = []
# ///
"""Validate roadmap script baseline contracts for `scripts/`.

The checker enforces the Phase 0.2.3 baseline for roadmap-delivered scripts:

- script runtime metadata (uv shebang and metadata block),
- command invocation posture (Cuprum-first, no Plumbum/subprocess shelling),
- matching script tests under `scripts/tests/`.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

UV_SHEBANG = "#!/usr/bin/env -S uv run python"
UV_BLOCK_START = "# /// script"
UV_BLOCK_END = "# ///"
REQUIRES_PYTHON = ">=3.13"
FORBIDDEN_PATTERNS: tuple[tuple[re.Pattern[str], str], ...] = (
    (
        re.compile(r"^\s*(from|import)\s+plumbum\b", flags=re.MULTILINE),
        "Plumbum imports are forbidden; use Cuprum",
    ),
    (
        re.compile(r"^\s*import\s+subprocess\b", flags=re.MULTILINE),
        "subprocess imports are forbidden; use Cuprum",
    ),
    (
        re.compile(r"^\s*from\s+subprocess\s+import\b", flags=re.MULTILINE),
        "subprocess imports are forbidden; use Cuprum",
    ),
    (
        re.compile(r"\bsubprocess\.[A-Za-z_]+\(", flags=re.MULTILINE),
        "subprocess invocation is forbidden; use Cuprum",
    ),
    (
        re.compile(r"\bos\.(system|popen)\(", flags=re.MULTILINE),
        "shell execution via os.system/os.popen is forbidden",
    ),
    (
        re.compile(r"^\s*from\s+cuprum\s+import\s+local\b", flags=re.MULTILINE),
        "`from cuprum import local` is forbidden in baseline scripts",
    ),
    (
        re.compile(r"^\s*from\s+cuprum\.cmd\s+import\b", flags=re.MULTILINE),
        "`cuprum.cmd` imports are forbidden in baseline scripts",
    ),
)


@dataclass(frozen=True)
class BaselineIssue:
    """Represents one baseline violation for a script."""

    path: Path
    message: str


def is_roadmap_script(path: Path, scripts_root: Path) -> bool:
    """Return `True` when a path is a roadmap script entrypoint candidate."""
    if path.suffix != ".py":
        return False
    if path.name.startswith("_"):
        return False
    if path.name == "__init__.py":
        return False

    relative = path.relative_to(scripts_root)
    return "tests" not in relative.parts


def discover_roadmap_scripts(scripts_root: Path) -> list[Path]:
    """Find all roadmap script entrypoint candidates under `scripts_root`."""
    return sorted(
        path for path in scripts_root.rglob("*.py") if is_roadmap_script(path, scripts_root)
    )


def expected_test_path(script_path: Path, scripts_root: Path) -> Path:
    """Return the expected matching test file path for one script."""
    relative_without_suffix = script_path.relative_to(scripts_root).with_suffix("")
    return (
        scripts_root
        / "tests"
        / relative_without_suffix.parent
        / f"test_{relative_without_suffix.name}.py"
    )


def parse_uv_metadata(script_text: str) -> dict[str, str]:
    """Parse key-value lines from the inline uv metadata block."""
    lines = script_text.splitlines()
    try:
        block_start = lines.index(UV_BLOCK_START)
    except ValueError:
        return {}

    try:
        block_end = lines.index(UV_BLOCK_END, block_start + 1)
    except ValueError:
        return {}

    metadata: dict[str, str] = {}
    for line in lines[block_start + 1 : block_end]:
        cleaned = line.removeprefix("#").strip()
        if "=" not in cleaned:
            continue
        key, raw_value = cleaned.split("=", maxsplit=1)
        metadata[key.strip()] = raw_value.strip()

    return metadata


def validate_runtime_metadata(path: Path, script_text: str) -> list[BaselineIssue]:
    """Validate uv shebang and metadata baseline expectations."""
    issues: list[BaselineIssue] = []
    lines = script_text.splitlines()
    shebang = lines[0] if lines else ""
    if shebang != UV_SHEBANG:
        issues.append(
            BaselineIssue(
                path=path,
                message=f"missing expected uv shebang `{UV_SHEBANG}`",
            )
        )

    metadata = parse_uv_metadata(script_text)
    if not metadata:
        issues.append(
            BaselineIssue(
                path=path,
                message="missing uv metadata block (`# /// script` ... `# ///`)",
            )
        )
        return issues

    requires_python = metadata.get("requires-python")
    if requires_python is None or REQUIRES_PYTHON not in requires_python:
        issues.append(
            BaselineIssue(
                path=path,
                message=f"requires-python must include `{REQUIRES_PYTHON}`",
            )
        )

    if "dependencies" not in metadata:
        issues.append(
            BaselineIssue(
                path=path,
                message="uv metadata must declare `dependencies`",
            )
        )

    return issues


def validate_command_invocation(path: Path, script_text: str) -> list[BaselineIssue]:
    """Validate command invocation conventions for roadmap scripts."""
    issues: list[BaselineIssue] = []
    for pattern, violation in FORBIDDEN_PATTERNS:
        if pattern.search(script_text):
            issues.append(
                BaselineIssue(
                    path=path,
                    message=violation,
                )
            )

    uses_cuprum_programs = "Program(" in script_text or "sh.make(" in script_text
    if not uses_cuprum_programs:
        return issues

    if "scoped(" not in script_text:
        issues.append(
            BaselineIssue(
                path=path,
                message="Cuprum command execution must use `scoped(...allowlist=...)`",
            )
        )

    has_run_invocation = "run_sync(" in script_text or re.search(r"\.run\(", script_text)
    if not has_run_invocation:
        issues.append(
            BaselineIssue(
                path=path,
                message="Cuprum commands must use `run_sync()` or `run()`",
            )
        )

    return issues


def validate_matching_test(script_path: Path, scripts_root: Path) -> list[BaselineIssue]:
    """Ensure each roadmap script has a matching pytest file."""
    expected = expected_test_path(script_path, scripts_root)
    if expected.exists():
        return []

    return [
        BaselineIssue(
            path=script_path,
            message=f"missing matching test `{expected.relative_to(scripts_root)}`",
        )
    ]


def validate_script(script_path: Path, scripts_root: Path) -> list[BaselineIssue]:
    """Validate one roadmap script against all baseline checks."""
    try:
        script_text = script_path.read_text(encoding="utf-8")
    except OSError as error:
        detail = error.strerror if error.strerror else str(error)
        return [
            BaselineIssue(
                path=script_path,
                message=f"unable to read script: {detail}",
            )
        ]

    return [
        *validate_runtime_metadata(script_path, script_text),
        *validate_command_invocation(script_path, script_text),
        *validate_matching_test(script_path, scripts_root),
    ]


def render_issues(issues: list[BaselineIssue], scripts_root: Path) -> str:
    """Render issues in a deterministic, review-friendly format."""

    def display_path(path: Path) -> Path:
        try:
            return path.relative_to(scripts_root)
        except ValueError:
            return path

    sorted_issues = sorted(issues, key=lambda issue: (str(issue.path), issue.message))
    lines = [
        "script baseline validation failed:",
        *[
            f"- {display_path(issue.path)}: {issue.message}"
            for issue in sorted_issues
        ],
    ]
    return "\n".join(lines)


def parse_args(argv: list[str]) -> argparse.Namespace:
    """Parse command-line arguments for baseline validation."""
    parser = argparse.ArgumentParser(
        description="Validate roadmap script baseline contracts."
    )
    parser.add_argument(
        "paths",
        nargs="*",
        type=Path,
        help="Optional script paths to validate. Defaults to all roadmap scripts.",
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parent,
        help="Scripts root directory (default: scripts/).",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    """Entry point for script baseline validation."""
    args = parse_args(argv or [])
    scripts_root = args.root.resolve()

    explicit_path_issues: list[BaselineIssue] = []
    if args.paths:
        script_paths = []
        for path in args.paths:
            resolved_path = path.resolve()
            if resolved_path.exists():
                if not resolved_path.is_file():
                    explicit_path_issues.append(
                        BaselineIssue(
                            path=resolved_path,
                            message="explicit path is not a file",
                        )
                    )
                    continue
                try:
                    if not is_roadmap_script(resolved_path, scripts_root):
                        explicit_path_issues.append(
                            BaselineIssue(
                                path=resolved_path,
                                message=(
                                    "explicit path is not a roadmap-delivered "
                                    "script entrypoint"
                                ),
                            )
                        )
                        continue
                except ValueError:
                    explicit_path_issues.append(
                        BaselineIssue(
                            path=resolved_path,
                            message=(
                                "explicit path must be under scripts root and "
                                "match roadmap script discovery rules"
                            ),
                        )
                    )
                    continue
            script_paths.append(resolved_path)
    else:
        script_paths = discover_roadmap_scripts(scripts_root)

    issues = explicit_path_issues + [
        issue
        for script_path in script_paths
        for issue in validate_script(script_path, scripts_root)
    ]

    if issues:
        print(render_issues(issues, scripts_root))
        return 1

    print(f"script baseline validation passed for {len(script_paths)} script(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
