"""Shared Cuprum helpers for script allowlist and command builder setup."""

from __future__ import annotations

from collections.abc import Iterable

from cuprum import Program, ProgramCatalogue, ProjectSettings, SafeCmd, sh


def build_catalogue(*programs: Program) -> ProgramCatalogue:
    """Return a one-project catalogue allowing exactly `programs` for the repo scripts."""
    return ProgramCatalogue(
        projects=(
            ProjectSettings(
                name="repo-scripts",
                programs=tuple(programs),
                documentation_locations=("docs/scripting-standards.md",),
                noise_rules=(),
            ),
        )
    )


def build_commands(
    *,
    catalogue: ProgramCatalogue,
    programs: Iterable[Program],
) -> dict[Program, SafeCmd]:
    """Build a safe command for each program, keyed by program, from `catalogue`."""
    return {program: sh.make(program, catalogue=catalogue) for program in programs}
