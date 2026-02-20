#!/usr/bin/env -S uv run python
# /// script
# requires-python = ">=3.13"
# dependencies = ["astroid"]
# ///
"""Validate roadmap script baseline contracts for ``scripts/``.

This module provides a deterministic, fail-closed checker for
roadmap-delivered automation scripts. It enforces runtime metadata
requirements, command invocation posture, and matching script-test coverage.

Utility
-------
Use this checker in local development and continuous integration to prevent
script regressions from merging.

Usage
-----
Run from the repository root:

```
make script-baseline
```
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

UV_SHEBANG: str = "#!/usr/bin/env -S uv run python"
UV_BLOCK_START: str = "# /// script"
UV_BLOCK_END: str = "# ///"
REQUIRES_PYTHON: str = ">=3.13"
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
    """Return whether a path is a roadmap script entrypoint candidate.

    Parameters
    ----------
    path : Path
        Candidate path to evaluate.
    scripts_root : Path
        Repository `scripts/` root path.

    Returns
    -------
    bool
        ``True`` when the path is an eligible roadmap script entrypoint.
    """
    if path.suffix != ".py":
        return False
    if path.name.startswith("_"):
        return False
    if path.name == "__init__.py":
        return False

    relative = path.relative_to(scripts_root)
    return "tests" not in relative.parts


def discover_roadmap_scripts(scripts_root: Path) -> list[Path]:
    """Find roadmap script entrypoint candidates under `scripts_root`.

    Parameters
    ----------
    scripts_root : Path
        Repository `scripts/` root path.

    Returns
    -------
    list[Path]
        Sorted list of eligible script entrypoint paths.
    """
    return sorted(
        path for path in scripts_root.rglob("*.py") if is_roadmap_script(path, scripts_root)
    )


def expected_test_path(script_path: Path, scripts_root: Path) -> Path:
    """Return the expected matching test file path for one script.

    Parameters
    ----------
    script_path : Path
        Script entrypoint path.
    scripts_root : Path
        Repository `scripts/` root path.

    Returns
    -------
    Path
        Expected pytest module path for the script.
    """
    relative_without_suffix = script_path.relative_to(scripts_root).with_suffix("")
    return (
        scripts_root
        / "tests"
        / relative_without_suffix.parent
        / f"test_{relative_without_suffix.name}.py"
    )


def parse_uv_metadata(script_text: str) -> dict[str, str]:
    """Parse key-value lines from the inline uv metadata block.

    Parameters
    ----------
    script_text : str
        Raw script text to parse.

    Returns
    -------
    dict[str, str]
        Parsed metadata mapping, or an empty mapping when no valid block exists.
    """
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


def _has_required_python_baseline(requires_python: str) -> bool:
    """Return whether a `requires-python` constraint includes the baseline."""
    cleaned = requires_python.strip().strip("'\"")
    constraints = [part.strip().strip("'\"") for part in cleaned.split(",")]
    required_pattern = re.compile(r">=\s*3\.13(?:\.\d+)?$")
    return any(required_pattern.fullmatch(part) for part in constraints)


def validate_runtime_metadata(path: Path, script_text: str) -> list[BaselineIssue]:
    """Validate uv shebang and metadata baseline expectations.

    Parameters
    ----------
    path : Path
        Script path for issue attribution.
    script_text : str
        Raw script text to validate.

    Returns
    -------
    list[BaselineIssue]
        Validation issues discovered in runtime metadata.
    """
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
    if requires_python is None or not _has_required_python_baseline(requires_python):
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


def _check_forbidden_patterns(path: Path, script_text: str) -> list[BaselineIssue]:
    """Check script text against forbidden command patterns."""
    issues: list[BaselineIssue] = []
    for pattern, violation in FORBIDDEN_PATTERNS:
        if pattern.search(script_text):
            issues.append(
                BaselineIssue(
                    path=path,
                    message=violation,
                )
            )
    return issues


def _is_sh_make_call(func_node) -> bool:
    """Check if an AST node represents a sh.make(...) call pattern."""
    from astroid import nodes

    match func_node:
        case nodes.Attribute(attrname="make", expr=nodes.Name(name="sh")):
            return True
        case _:
            return False


def _detect_cuprum_usage(script_text: str) -> bool:
    """Detect whether script uses Cuprum Program or sh.make constructs.

    Uses AST parsing when available, falls back to heuristic search.
    """
    try:
        import astroid
        from astroid import nodes
    except ImportError:
        return "Program(" in script_text or "sh.make(" in script_text

    try:
        tree = astroid.parse(script_text)
    except astroid.AstroidSyntaxError:
        return "Program(" in script_text or "sh.make(" in script_text

    for call_node in tree.nodes_of_class(nodes.Call):
        match call_node.func:
            case nodes.Name(name="Program"):
                return True
            case _ if _is_sh_make_call(call_node.func):
                return True
            case _:
                continue

    return False


def _detect_cuprum_programs(script_text: str) -> bool:
    """Detect whether the script uses Cuprum Program or sh.make constructs."""
    return _detect_cuprum_usage(script_text)


def _has_cuprum_imports(tree) -> bool:
    """Return whether a parsed module imports Cuprum symbols."""
    from astroid import nodes

    for node in tree.body:
        match node:
            case nodes.Import():
                if any("cuprum" in name for name, _alias in node.names):
                    return True
            case nodes.ImportFrom():
                if node.modname and "cuprum" in node.modname:
                    return True
            case _:
                continue

    return False


def _has_cuprum_run_calls(tree) -> bool:
    """Return whether a parsed module calls `run` or `run_sync`."""
    from astroid import nodes

    return any(
        isinstance(call_node.func, nodes.Attribute)
        and call_node.func.attrname in ("run", "run_sync")
        for call_node in tree.nodes_of_class(nodes.Call)
    )


def _run_invocation_present(script_text: str) -> bool:
    """Return whether Cuprum run invocation requirements are satisfied."""
    try:
        import astroid
    except ImportError:
        return bool("run_sync(" in script_text or re.search(r"\.run\(", script_text))

    try:
        tree = astroid.parse(script_text)
    except astroid.AstroidSyntaxError:
        # Fall back to text heuristics when parsing fails on syntactically invalid scripts.
        return bool("run_sync(" in script_text or re.search(r"\.run\(", script_text))

    return _has_cuprum_imports(tree) and _has_cuprum_run_calls(tree)


def _validate_cuprum_requirements(path: Path, script_text: str) -> list[BaselineIssue]:
    """Validate that Cuprum usage follows baseline requirements."""
    issues: list[BaselineIssue] = []
    if "scoped(" not in script_text:
        issues.append(
            BaselineIssue(
                path=path,
                message="Cuprum command execution must use `scoped(...allowlist=...)`",
            )
        )

    if not _run_invocation_present(script_text):
        issues.append(
            BaselineIssue(
                path=path,
                message="Cuprum commands must use `run_sync()` or `run()`",
            )
        )

    return issues


def _validate_cuprum_rules(path: Path, script_text: str) -> list[BaselineIssue]:
    """Validate Cuprum-specific invocation rules (scoped and run/run_sync)."""
    return _validate_cuprum_requirements(path, script_text)


def validate_command_invocation(path: Path, script_text: str) -> list[BaselineIssue]:
    """Validate command invocation conventions for roadmap scripts.

    Parameters
    ----------
    path : Path
        Script path for issue attribution.
    script_text : str
        Raw script text to validate.

    Returns
    -------
    list[BaselineIssue]
        Validation issues discovered in command invocation usage.
    """
    issues = _check_forbidden_patterns(path, script_text)
    if not _detect_cuprum_programs(script_text):
        return issues
    issues.extend(_validate_cuprum_rules(path, script_text))
    return issues


def validate_matching_test(script_path: Path, scripts_root: Path) -> list[BaselineIssue]:
    """Ensure each roadmap script has a matching pytest file.

    Parameters
    ----------
    script_path : Path
        Script path to validate.
    scripts_root : Path
        Repository `scripts/` root path.

    Returns
    -------
    list[BaselineIssue]
        Missing-test issue if no matching test file exists.
    """
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
    """Validate one roadmap script against all baseline checks.

    Parameters
    ----------
    script_path : Path
        Script path to validate.
    scripts_root : Path
        Repository `scripts/` root path.

    Returns
    -------
    list[BaselineIssue]
        Aggregated issues from metadata, command, and matching-test checks.
    """
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
    """Render issues in a deterministic, review-friendly format.

    Parameters
    ----------
    issues : list[BaselineIssue]
        Validation issues to render.
    scripts_root : Path
        Repository `scripts/` root path.

    Returns
    -------
    str
        Stable text output for terminal and CI reporting.
    """

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
    """Parse command-line arguments for baseline validation.

    Parameters
    ----------
    argv : list[str]
        Command-line tokens excluding executable name.

    Returns
    -------
    argparse.Namespace
        Parsed command-line arguments.
    """
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


def _validate_explicit_path(path: Path, scripts_root: Path) -> BaselineIssue | None:
    """Validate a single explicit path argument and return a BaselineIssue if invalid, otherwise None."""
    resolved_path = path.resolve()
    if not resolved_path.exists():
        return None

    if not resolved_path.is_file():
        return BaselineIssue(
            path=resolved_path,
            message="explicit path is not a file",
        )

    try:
        if not is_roadmap_script(resolved_path, scripts_root):
            return BaselineIssue(
                path=resolved_path,
                message="explicit path is not a roadmap-delivered script entrypoint",
            )
    except ValueError:
        return BaselineIssue(
            path=resolved_path,
            message=(
                "explicit path must be under scripts root and "
                "match roadmap script discovery rules"
            ),
        )

    return None


def _process_explicit_paths(
    paths: list[Path],
    scripts_root: Path,
) -> tuple[list[Path], list[BaselineIssue]]:
    """Process explicit path arguments and validate them.

    Returns a tuple of (valid script paths, validation issues).
    """
    script_paths = []
    issues: list[BaselineIssue] = []
    for path in paths:
        issue = _validate_explicit_path(path, scripts_root)
        resolved_path = path.resolve()
        if issue is not None:
            issues.append(issue)
        else:
            script_paths.append(resolved_path)

    return script_paths, issues


def main(argv: list[str] | None = None) -> int:
    """Run script-baseline validation and return process exit code.

    Parameters
    ----------
    argv : list[str] | None, optional
        Optional command-line tokens. When ``None``, uses an empty list.

    Returns
    -------
    int
        ``0`` on success, ``1`` when validation issues are present.
    """
    args = parse_args(argv or [])
    scripts_root = args.root.resolve()

    if args.paths:
        script_paths, explicit_path_issues = _process_explicit_paths(
            args.paths, scripts_root
        )
    else:
        script_paths = discover_roadmap_scripts(scripts_root)
        explicit_path_issues = []

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
