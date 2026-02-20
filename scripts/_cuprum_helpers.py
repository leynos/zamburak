"""Shared Cuprum helpers for script allowlist and command builder setup."""

from __future__ import annotations

from collections.abc import Iterable

from cuprum import Program, ProgramCatalogue, ProjectSettings, SafeCmd, sh


def build_catalogue(*programs: Program) -> ProgramCatalogue:
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
    return {program: sh.make(program, catalogue=catalogue) for program in programs}
