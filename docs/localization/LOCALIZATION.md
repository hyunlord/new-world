# Localization Guide

WorldSim uses a directory-based i18n system. All user-visible text lives in JSON files under `res://localization/{locale}/`. Code only contains keys, never raw text.

## Directory Structure

```text
res://localization/
    ko/          ← Korean (default)
        ui.json      — UI labels, buttons, menus
        game.json    — Jobs, statuses, stages, seasons, resources
        traits.json  — Trait names + descriptions (187 traits)
        emotions.json — Emotion names, intensity labels, Dyad names
        events.json  — Chronicle event templates
        deaths.json  — Death causes
        buildings.json — Building names/descriptions
        tutorial.json — Tutorial/help text
    en/          ← English
        (same structure)
```

## Adding a New Language

### Step 1: Create directory
```bash
res://localization/ja/    # Japanese example
```

### Step 2: Copy English files
Copy all `.json` files from `en/` into your new locale directory.

### Step 3: Translate
Edit each JSON file, translating the **values** (right side). Never change the **keys** (left side).

```json
// en/deaths.json
{ "DEATH_STARVATION": "Starvation" }

// ja/deaths.json
{ "DEATH_STARVATION": "餓死" }
```

### Step 4: Register
Add your locale code to `scripts/core/locale.gd`:
```gdscript
const SUPPORTED_LOCALES: Array = ["ko", "en", "ja"]
```

### Step 5: Done
The new language appears automatically in the in-game Settings menu.

---

## Using the Locale API in Code

Always use the `Locale` autoload — never hardcode text.

### Simple lookup
```gdscript
label.text = Locale.tr("UI_HUNGER")         # → "Hunger" or "허기"
```

### Game ID → translation
```gdscript
# Converts internal id to localized string using PREFIX_ID key
job_label.text = Locale.tr_id("JOB", entity.job)      # "gatherer" → "Gatherer"/"채집꾼"
status_label.text = Locale.tr_id("STATUS", entity.status)  # "rest" → "Rest"/"휴식"
cause_label.text = Locale.tr_id("DEATH", cause)        # "old_age" → "Old Age"/"노령"
stage_label.text = Locale.tr_id("STAGE", entity.age_stage) # "adult" → "Adult"/"성인"
season_label.text = Locale.tr_id("SEASON", season)     # "spring" → "Spring"/"봄"
```

### Format strings
```gdscript
# {param} placeholders substituted from dictionary
text = Locale.trf("EVT_CHILD_BORN", {
    "name": child.entity_name,
    "mother": mother.entity_name,
    "father": father.entity_name
})
# → "Aria born to Bea & Cal!" / "Aria이(가) Bea와 Cal 사이에서 태어남!"
```

### JSON data fields (Traits, Emotions)
```gdscript
# Reads name_ko, name_kr, or name_en field from Dictionary
trait_label.text = Locale.tr_data(trait_def, "name")
trait_desc.text = Locale.tr_data(trait_def, "description")
```

### Responding to language changes
All UI panels should connect to `locale_changed` and redraw:
```gdscript
func _ready() -> void:
    Locale.locale_changed.connect(func(_l): queue_redraw())
    # or: Locale.locale_changed.connect(_on_locale_changed)
```

---

## JSON Key Naming Convention

| Pattern | Example | Used For |
|---------|---------|---------|
| `UI_*` | `UI_HUNGER`, `UI_SAVE` | UI labels and buttons |
| `JOB_*` | `JOB_GATHERER`, `JOB_BUILDER` | Job names |
| `STATUS_*` | `STATUS_REST`, `STATUS_BUILD` | Entity action statuses |
| `STAGE_*` | `STAGE_ADULT`, `STAGE_INFANT` | Age stages |
| `SEASON_*` | `SEASON_SPRING` | Calendar seasons |
| `DEATH_*` | `DEATH_OLD_AGE` | Death causes |
| `EVT_*` | `EVT_CHILD_BORN`, `EVT_DIED` | Chronicle event templates |
| `TRAIT_*` | `TRAIT_CURIOUS` | Trait names |
| `TRAIT_*_DESC` | `TRAIT_CURIOUS_DESC` | Trait descriptions |
| `NOTIF_*` | `UI_NOTIF_GAME_SAVED` | Toast notifications |

### Format variables
Use `{lowercase_name}` in JSON values:
```json
"UI_SLOT_FORMAT": "Slot {slot}  Y{year} M{month} - Pop: {pop} - {time_ago}"
```

---

## Generating traits.json and emotions.json

Run the generation script when trait/emotion data changes:
```bash
cd /path/to/project
python3 tools/generate_locale_from_json.py
```

This reads `data/species/human/personality/trait_definitions.json` and generates `localization/ko/traits.json` and `localization/en/traits.json`.

---

## Rules for Developers

1. **Never hardcode text in GDScript.** Use `Locale.tr()` always.
2. **Adding new text:** First add key+value to both `ko/` and `en/` JSON files, then use `Locale.tr("NEW_KEY")` in code.
3. **Comments and print() are exempt** — English is fine there.
4. **Missing key fallback:** `Locale.tr("MISSING_KEY")` returns `"MISSING_KEY"` — easy to spot in testing.
5. **Korean-specific fallback:** `name_kr` field in JSON data is treated as `name_ko` for backward compatibility.

---

## Modding

To override translations via a mod:
Place JSON files at `res://mods/{mod_name}/localization/{locale}/` with the same filenames. The mod system (future) will merge these on top of base translations.
