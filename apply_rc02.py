from pathlib import Path


CORE_MOVES = [
    ("simulation", "simulation_system"),
    ("simulation", "simulation_engine"),
    ("simulation", "simulation_bus"),
    ("simulation", "game_config"),
    ("simulation", "game_calendar"),
    ("simulation", "event_logger"),
    ("simulation", "save_manager"),
    ("simulation", "locale"),
    ("entity", "entity_data"),
    ("entity", "entity_manager"),
    ("entity", "personality_data"),
    ("entity", "personality_system"),
    ("entity", "body_attributes"),
    ("entity", "emotion_data"),
    ("entity", "deceased_registry"),
    ("stats", "stat_query"),
    ("stats", "stat_cache"),
    ("stats", "stat_curve"),
    ("stats", "stat_definition"),
    ("stats", "stat_evaluator_registry"),
    ("stats", "stat_graph"),
    ("stats", "stat_modifier"),
    ("world", "world_data"),
    ("world", "world_generator"),
    ("world", "resource_map"),
    ("world", "chunk_index"),
    ("world", "pathfinder"),
    ("settlement", "settlement_data"),
    ("settlement", "settlement_manager"),
    ("settlement", "building_data"),
    ("settlement", "building_manager"),
    ("social", "relationship_data"),
    ("social", "relationship_manager"),
    ("social", "value_defs"),
    ("social", "name_generator"),
    ("social", "species_manager"),
]

UI_MOVES = [
    ("panels", "entity_detail_panel"),
    ("panels", "building_detail_panel"),
    ("panels", "chronicle_panel"),
    ("panels", "list_panel"),
    ("panels", "minimap_panel"),
    ("panels", "stats_panel"),
    ("panels", "stats_detail_panel"),
    ("panels", "pause_menu"),
    ("panels", "trait_tooltip"),
    ("renderers", "entity_renderer"),
    ("renderers", "building_renderer"),
    ("renderers", "world_renderer"),
]

FILES_TO_PATCH = [
    "project.godot",
    "scenes/main/main.gd",
    "scenes/main/main.tscn",
    "scripts/ai/behavior_system.gd",
    "scripts/core/entity/body_attributes.gd",
    "scripts/core/entity/deceased_registry.gd",
    "scripts/core/entity/emotion_data.gd",
    "scripts/core/entity/entity_data.gd",
    "scripts/core/entity/entity_manager.gd",
    "scripts/core/entity/personality_data.gd",
    "scripts/core/entity/personality_system.gd",
    "scripts/core/settlement/building_data.gd",
    "scripts/core/settlement/building_manager.gd",
    "scripts/core/settlement/settlement_data.gd",
    "scripts/core/settlement/settlement_manager.gd",
    "scripts/core/simulation/game_config.gd",
    "scripts/core/simulation/save_manager.gd",
    "scripts/core/social/relationship_manager.gd",
    "scripts/core/social/value_defs.gd",
    "scripts/core/stats/stat_cache.gd",
    "scripts/core/stats/stat_graph.gd",
    "scripts/core/stats/stat_modifier.gd",
    "scripts/core/stats/stat_query.gd",
    "scripts/systems/biology/age_system.gd",
    "scripts/systems/biology/mortality_system.gd",
    "scripts/systems/biology/personality_generator.gd",
    "scripts/systems/biology/population_system.gd",
    "scripts/systems/development/child_stress_processor.gd",
    "scripts/systems/development/childcare_system.gd",
    "scripts/systems/development/intergenerational_system.gd",
    "scripts/systems/development/parenting_system.gd",
    "scripts/systems/psychology/contagion_system.gd",
    "scripts/systems/psychology/coping_system.gd",
    "scripts/systems/psychology/emotion_system.gd",
    "scripts/systems/psychology/mental_break_system.gd",
    "scripts/systems/psychology/morale_system.gd",
    "scripts/systems/psychology/needs_system.gd",
    "scripts/systems/psychology/personality_maturation.gd",
    "scripts/systems/psychology/stress_system.gd",
    "scripts/systems/psychology/trait_system.gd",
    "scripts/systems/psychology/trait_violation_system.gd",
    "scripts/systems/psychology/trauma_scar_system.gd",
    "scripts/systems/psychology/upper_needs_system.gd",
    "scripts/systems/record/chronicle_system.gd",
    "scripts/systems/record/stat_sync_system.gd",
    "scripts/systems/record/stat_threshold_system.gd",
    "scripts/systems/record/stats_recorder.gd",
    "scripts/systems/social/family_system.gd",
    "scripts/systems/social/settlement_culture.gd",
    "scripts/systems/social/social_event_system.gd",
    "scripts/systems/social/value_system.gd",
    "scripts/systems/work/building_effect_system.gd",
    "scripts/systems/work/construction_system.gd",
    "scripts/systems/work/gathering_system.gd",
    "scripts/systems/work/job_assignment_system.gd",
    "scripts/systems/world/migration_system.gd",
    "scripts/systems/world/movement_system.gd",
    "scripts/systems/world/resource_regen_system.gd",
    "scripts/ui/hud.gd",
    "scripts/ui/panels/chronicle_panel.gd",
    "scripts/ui/panels/entity_detail_panel.gd",
    "scripts/ui/panels/list_panel.gd",
    "scripts/ui/renderers/entity_renderer.gd",
    "scripts/ui/renderers/world_renderer.gd",
    "tests/test_stat_curve.gd",
    "tests/test_stat_graph.gd",
]


def flat_core_path(name: str) -> str:
    return "res://scripts/core/" + name + ".gd"


def moved_core_path(folder: str, name: str) -> str:
    return "res://scripts/core/" + folder + "/" + name + ".gd"


def flat_ui_path(name: str) -> str:
    return "res://scripts/ui/" + name + ".gd"


def moved_ui_path(folder: str, name: str) -> str:
    return "res://scripts/ui/" + folder + "/" + name + ".gd"


def build_replacements() -> list[tuple[str, str]]:
    replacements: list[tuple[str, str]] = []

    for folder, name in CORE_MOVES:
        replacements.append((flat_core_path(name), moved_core_path(folder, name)))

    for folder, name in UI_MOVES:
        replacements.append((flat_ui_path(name), moved_ui_path(folder, name)))

    return replacements


REPLACEMENTS = build_replacements()


def apply_replacements(file_path: Path) -> None:
    original_text = file_path.read_text(encoding="utf-8")
    updated_text = original_text

    for old_path, new_path in REPLACEMENTS:
        updated_text = updated_text.replace(old_path, new_path)

    if updated_text != original_text:
        file_path.write_text(updated_text, encoding="utf-8")
        print(f"PATCHED: {file_path.as_posix()}")


def main() -> None:
    for relative_path in FILES_TO_PATCH:
        apply_replacements(Path(relative_path))
    print("RC-02 DONE")


if __name__ == "__main__":
    main()
