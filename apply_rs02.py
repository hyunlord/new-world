#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import sys


REPLACEMENTS: list[tuple[str, list[tuple[str, str]]]] = [
    (
        "scenes/main/main.gd",
        [
            (
                'preload("res://scripts/systems/needs_system.gd")',
                'preload("res://scripts/systems/psychology/needs_system.gd")',
            ),
            (
                'preload("res://scripts/systems/movement_system.gd")',
                'preload("res://scripts/systems/world/movement_system.gd")',
            ),
            (
                'preload("res://scripts/systems/resource_regen_system.gd")',
                'preload("res://scripts/systems/world/resource_regen_system.gd")',
            ),
            (
                'preload("res://scripts/systems/gathering_system.gd")',
                'preload("res://scripts/systems/work/gathering_system.gd")',
            ),
            (
                'preload("res://scripts/systems/construction_system.gd")',
                'preload("res://scripts/systems/work/construction_system.gd")',
            ),
            (
                'preload("res://scripts/systems/building_effect_system.gd")',
                'preload("res://scripts/systems/work/building_effect_system.gd")',
            ),
            (
                'preload("res://scripts/systems/job_assignment_system.gd")',
                'preload("res://scripts/systems/work/job_assignment_system.gd")',
            ),
            (
                'preload("res://scripts/systems/population_system.gd")',
                'preload("res://scripts/systems/biology/population_system.gd")',
            ),
            (
                'preload("res://scripts/systems/migration_system.gd")',
                'preload("res://scripts/systems/world/migration_system.gd")',
            ),
            (
                'preload("res://scripts/systems/stats_recorder.gd")',
                'preload("res://scripts/systems/record/stats_recorder.gd")',
            ),
            (
                'preload("res://scripts/systems/social_event_system.gd")',
                'preload("res://scripts/systems/social/social_event_system.gd")',
            ),
            (
                'preload("res://scripts/systems/emotion_system.gd")',
                'preload("res://scripts/systems/psychology/emotion_system.gd")',
            ),
            (
                'preload("res://scripts/systems/age_system.gd")',
                'preload("res://scripts/systems/biology/age_system.gd")',
            ),
            (
                'preload("res://scripts/systems/family_system.gd")',
                'preload("res://scripts/systems/social/family_system.gd")',
            ),
            (
                'preload("res://scripts/systems/mortality_system.gd")',
                'preload("res://scripts/systems/biology/mortality_system.gd")',
            ),
            (
                'preload("res://scripts/systems/childcare_system.gd")',
                'preload("res://scripts/systems/development/childcare_system.gd")',
            ),
            (
                'preload("res://scripts/systems/stress_system.gd")',
                'preload("res://scripts/systems/psychology/stress_system.gd")',
            ),
            (
                'preload("res://scripts/systems/mental_break_system.gd")',
                'preload("res://scripts/systems/psychology/mental_break_system.gd")',
            ),
            (
                'preload("res://scripts/systems/trauma_scar_system.gd")',
                'preload("res://scripts/systems/psychology/trauma_scar_system.gd")',
            ),
            (
                'preload("res://scripts/systems/trait_violation_system.gd")',
                'preload("res://scripts/systems/psychology/trait_violation_system.gd")',
            ),
            (
                'preload("res://scripts/systems/phase4/coping_system.gd")',
                'preload("res://scripts/systems/psychology/coping_system.gd")',
            ),
            (
                'preload("res://scripts/systems/phase4/morale_system.gd")',
                'preload("res://scripts/systems/psychology/morale_system.gd")',
            ),
            (
                'preload("res://scripts/systems/phase4/contagion_system.gd")',
                'preload("res://scripts/systems/psychology/contagion_system.gd")',
            ),
            (
                'const Phase4CoordinatorScript = preload("res://scripts/systems/phase4/phase4_coordinator.gd")',
                'const PsychologyCoordinatorScript = preload("res://scripts/systems/psychology/psychology_coordinator.gd")',
            ),
            (
                'preload("res://scripts/systems/phase5/child_stress_processor.gd")',
                'preload("res://scripts/systems/development/child_stress_processor.gd")',
            ),
            (
                'preload("res://scripts/systems/phase5/intergenerational_system.gd")',
                'preload("res://scripts/systems/development/intergenerational_system.gd")',
            ),
            (
                'preload("res://scripts/systems/phase5/parenting_system.gd")',
                'preload("res://scripts/systems/development/parenting_system.gd")',
            ),
            (
                'preload("res://scripts/systems/value_system.gd")',
                'preload("res://scripts/systems/social/value_system.gd")',
            ),
            (
                'preload("res://scripts/systems/stat_sync_system.gd")',
                'preload("res://scripts/systems/record/stat_sync_system.gd")',
            ),
            (
                'preload("res://scripts/systems/stat_threshold_system.gd")',
                'preload("res://scripts/systems/record/stat_threshold_system.gd")',
            ),
            (
                'preload("res://scripts/systems/upper_needs_system.gd")',
                'preload("res://scripts/systems/psychology/upper_needs_system.gd")',
            ),
            ('Phase4CoordinatorScript', 'PsychologyCoordinatorScript'),
            ('phase4_coordinator', 'psychology_coordinator'),
        ],
    ),
    (
        "scenes/debug/debug_commands.gd",
        [
            (
                'load("res://scripts/systems/phase5/attachment_system.gd")',
                'load("res://scripts/systems/development/attachment_system.gd")',
            ),
            (
                'load("res://scripts/systems/phase5/ace_tracker.gd")',
                'load("res://scripts/systems/development/ace_tracker.gd")',
            ),
        ],
    ),
    (
        "scenes/debug/debug_console.gd",
        [
            (
                'preload("res://scripts/systems/trait_system.gd")',
                'preload("res://scripts/systems/psychology/trait_system.gd")',
            ),
        ],
    ),
    (
        "scripts/ai/behavior_system.gd",
        [
            (
                'preload("res://scripts/systems/trait_system.gd")',
                'preload("res://scripts/systems/psychology/trait_system.gd")',
            ),
            (
                'preload("res://scripts/systems/value_system.gd")',
                'preload("res://scripts/systems/social/value_system.gd")',
            ),
        ],
    ),
    (
        "scripts/core/entity_manager.gd",
        [
            (
                'preload("res://scripts/systems/personality_generator.gd")',
                'preload("res://scripts/systems/biology/personality_generator.gd")',
            ),
            (
                'preload("res://scripts/systems/trait_system.gd")',
                'preload("res://scripts/systems/psychology/trait_system.gd")',
            ),
            (
                'preload("res://scripts/systems/value_system.gd")',
                'preload("res://scripts/systems/social/value_system.gd")',
            ),
        ],
    ),
    (
        "scripts/systems/biology/personality_generator.gd",
        [
            (
                'preload("res://scripts/systems/personality_generator.gd")',
                'preload("res://scripts/systems/biology/personality_generator.gd")',
            ),
            (
                'preload("res://scripts/systems/trait_system.gd")',
                'preload("res://scripts/systems/psychology/trait_system.gd")',
            ),
        ],
    ),
    (
        "scripts/systems/psychology/personality_maturation.gd",
        [
            (
                'preload("res://scripts/systems/trait_system.gd")',
                'preload("res://scripts/systems/psychology/trait_system.gd")',
            ),
        ],
    ),
    (
        "scripts/systems/development/parenting_system.gd",
        [
            (
                'load("res://scripts/systems/phase5/attachment_system.gd")',
                'load("res://scripts/systems/development/attachment_system.gd")',
            ),
        ],
    ),
    (
        "scripts/systems/social/value_system.gd",
        [
            (
                'preload("res://scripts/systems/value_system.gd")',
                'preload("res://scripts/systems/social/value_system.gd")',
            ),
            (
                'preload("res://scripts/systems/settlement_culture.gd")',
                'preload("res://scripts/systems/social/settlement_culture.gd")',
            ),
        ],
    ),
    (
        "scripts/ui/entity_detail_panel.gd",
        [
            (
                'preload("res://scripts/systems/trait_system.gd")',
                'preload("res://scripts/systems/psychology/trait_system.gd")',
            ),
        ],
    ),
    (
        "scripts/ui/trait_tooltip.gd",
        [
            (
                'preload("res://scripts/systems/trait_system.gd")',
                'preload("res://scripts/systems/psychology/trait_system.gd")',
            ),
        ],
    ),
    (
        "scripts/systems/psychology/psychology_coordinator.gd",
        [
            ('class_name Phase4Coordinator', 'class_name PsychologyCoordinator'),
        ],
    ),
]


def patch_file(path: Path, replacements: list[tuple[str, str]]) -> None:
    content = path.read_text(encoding="utf-8")
    for old, new in replacements:
        content = content.replace(old, new)
    path.write_text(content, encoding="utf-8")
    print(f"PATCHED: {path.as_posix()}")


def main() -> int:
    root = Path(".")
    try:
        for rel_path, replacements in REPLACEMENTS:
            patch_file(root / rel_path, replacements)
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    print("RS-02 DONE")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
