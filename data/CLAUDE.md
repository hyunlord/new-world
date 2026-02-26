# data/ — CLAUDE.md

> Data definitions: species, traits, emotions, stressors, buildings, skills.
> All game content is defined here as JSON/Resource files. Code reads these — never hardcodes content.

---

## Principle: Data-Driven Design

Game content (what exists) is separated from game logic (how it behaves).

```
data/           ← defines WHAT (species, traits, buildings, skills, tech)
scripts/        ← defines HOW (systems that process data)
```

**Adding new content** (a new trait, building, skill) should NEVER require code changes.
If it does, the system is not data-driven enough — fix the system first.

---

## Directory Structure

```
data/
  species/
    human.json              — Human species definition
  traits/
    personality_traits.json — All 187+ trait definitions
    trait_categories.json   — Trait categorization
  stressors/
    stressor_events.json    — Stressor definitions with severity/type
  emotions/
    emotion_presets.json    — Base emotion configurations
    composite_emotions.json — Plutchik composite definitions
  buildings/
    buildings.json          — Building definitions (cost, effects, requirements)
  skills/
    skill_definitions.json  — All skill definitions (~150+)
    skill_categories.json   — Skill categorization
  mental_breaks/
    mental_break_types.json — Mental break definitions (10 types)
  naming/
    name_pools.json         — Name generation data by culture
  decay/
    decay_parameters.json   — Need decay rates, emotion decay, etc.
```

---

## JSON Format Rules

### File Structure
Every data JSON file follows this pattern:
```json
{
  "version": "1.0",
  "description": "Brief description of this data file",
  "data": [
    {
      "id": "unique_snake_case_id",
      "display_name_key": "LOCALE_KEY_FOR_NAME",
      "description_key": "LOCALE_KEY_FOR_DESC",
      ...
    }
  ]
}
```

### Naming Conventions
- File names: `snake_case.json`
- IDs: `snake_case` (e.g., `"bronze_sword"`, `"crop_farming"`, `"fearful"`)
- Localization keys: `UPPER_SNAKE_CASE` (e.g., `"TRAIT_BRAVE"`, `"SKILL_ARCHERY"`)
- **Never put user-visible text directly in data files** — always use locale keys

### Required Fields
Every data entry MUST have:
- `id`: unique identifier within its file
- `display_name_key`: localization key for the display name

### Numeric Values
- Percentages: 0.0 ~ 1.0 (not 0~100)
- Ranges: use `"min"` / `"max"` / `"mean"` / `"std"` (for normal distributions)
- Always document units in comments or description

---

## Species Definition Format

```json
{
  "id": "human",
  "display_name_key": "SPECIES_HUMAN",
  "lifespan": {"mean": 70, "std": 10},
  "growth_stages": {
    "child": {"age_range": [0, 12], "attribute_multiplier": 0.5},
    "teen": {"age_range": [12, 18], "attribute_multiplier": 0.8},
    "adult": {"age_range": [18, 55], "attribute_multiplier": 1.0},
    "elder": {"age_range": [55, 999], "attribute_multiplier": 0.7}
  },
  "body_template": "humanoid",
  "fertility": {"min_age": 15, "max_age": 45, "base_rate": 0.1},
  "attribute_ranges": {
    "strength": {"min": 0.0, "max": 1.0, "mean": 0.5, "std": 0.15},
    ...
  },
  "diet": ["omnivore"],
  "temperature_comfort": {"min": 10, "max": 35}
}
```

---

## Trait Definition Format

```json
{
  "id": "brave",
  "display_name_key": "TRAIT_BRAVE",
  "description_key": "TRAIT_BRAVE_DESC",
  "category": "personality",
  "source_facets": {
    "HEXACO_E": {"direction": "low", "threshold": 0.3, "weight": 0.6},
    "HEXACO_X": {"direction": "high", "threshold": 0.7, "weight": 0.4}
  },
  "effects": {
    "combat_morale": 0.15,
    "fear_resistance": 0.2,
    "risk_tolerance": 0.1
  },
  "conflicts_with": ["cowardly"],
  "flavor_text_key": "TRAIT_BRAVE_FLAVOR"
}
```

---

## Skill Definition Format

```json
{
  "id": "blacksmithing",
  "display_name_key": "SKILL_BLACKSMITHING",
  "category": "crafting",
  "primary_attribute": "strength",
  "secondary_attribute": "agility",
  "intelligence_affinity": "kinesthetic",
  "prerequisites": [{"skill_id": "mining", "min_level": 5}],
  "required_tech": "bronze_age",
  "max_level": 20,
  "xp_curve": "logarithmic",
  "tags": ["physical", "crafting"]
}
```

---

## Building Definition Format

```json
{
  "id": "granary",
  "display_name_key": "BUILDING_GRANARY",
  "category": "storage",
  "construction_cost": {"wood": 20, "stone": 10},
  "construction_skill": "architecture",
  "construction_min_level": 3,
  "construction_time_ticks": 500,
  "effects": {
    "food_storage_capacity": 200,
    "food_decay_modifier": -0.5
  },
  "required_tech": "tribal",
  "max_per_settlement": 3
}
```

---

## Validation Rules

When adding or modifying data files:

1. **IDs must be unique** within their file
2. **All locale keys must exist** in both `localization/en/` and `localization/ko/`
3. **Numeric ranges must be valid**: min ≤ mean ≤ max, std > 0
4. **Prerequisites must reference existing IDs** (no dangling references)
5. **required_tech must match a tech tree node ID**
6. **No user-visible text in data files** — only locale keys

### Validation Command
```bash
# TODO: Add validation script
python tools/validate_data.py data/
```

---

## Adding New Content

### New Trait
1. Add entry to `traits/personality_traits.json`
2. Add locale keys to `localization/en/traits.json` and `localization/ko/traits.json`
3. Add to `trait_categories.json` if new category
4. Verify no ID collision with existing traits
5. Test: trait appears correctly in EntityDetailPanel with localized name

### New Skill
1. Add entry to `skills/skill_definitions.json`
2. Add locale keys to both language files
3. Verify prerequisite skills exist
4. Verify required_tech exists in tech tree
5. Test: skill can be learned and levels up correctly

### New Building
1. Add entry to `buildings/buildings.json`
2. Add locale keys to both language files
3. Add building effect constants to GameConfig if new effect type
4. Test: building can be constructed and applies effects

---

## Modding Extensibility

The data-driven architecture enables user mods:
- Mods add new JSON files to `data/` subdirectories
- Mods can override existing entries by matching `id`
- Systems scan data directories at startup — no code changes needed
- Future: `mods/` directory with load order configuration

---

## Rust Migration Notes

Data loading stays in GDScript (JSON parsing, Godot Resource system).
The data itself may be consumed by Rust systems via FFI — ensure data structures are simple (no Godot-specific types in hot-path data).

**Rule:** Data files define content. Whether GDScript or Rust processes that content is an implementation detail that data files don't care about.