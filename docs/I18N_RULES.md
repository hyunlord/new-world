# WorldSim i18n Rules

**Absolute rule:** Every string visible to the player must come from `Locale`. No Korean or English text may be hardcoded in GDScript. No dual-language display.

## Locale API

| Function | Use |
|----------|-----|
| `Locale.ltr(key)` | Static string from JSON key |
| `Locale.trf(key, params)` | Formatted string with `{placeholder}` substitution |
| `Locale.tr_id(prefix, id)` | Compound key — `"PREFIX_" + id.to_upper()` |
| `Locale.tr_data(dict, field)` | Reads `field_ko` or `field_en` from a data dict |

## Signal — locale_changed

Every UI script that holds static labels (set once at build time) **must** connect to `Locale.locale_changed` and refresh those labels.

```gdscript
func _ready() -> void:
    Locale.locale_changed.connect(_on_locale_changed)

func _on_locale_changed(_locale: String) -> void:
    _refresh_texts()
```

Dynamic labels updated every `_process()` tick refresh automatically — no explicit connection needed.

## JSON Files

- `localization/en/ui.json` — English strings
- `localization/ko/ui.json` — Korean strings
- Both files **must** have identical key sets. No key may exist in one but not the other.
- Game-data strings (traits, jobs, emotions, stages) live in `localization/{lang}/game.json` and `emotions.json`.

### Naming Conventions

| Pattern | Example | Use |
|---------|---------|-----|
| `UI_*` | `UI_SAVE` | Generic UI strings |
| `UI_*_FMT` | `UI_POP_FMT` | Format strings with `{placeholders}` |
| `BUILDING_TYPE_*` | `BUILDING_TYPE_SHELTER` | Building type names |
| `TOOLTIP_*` | `TOOLTIP_EFFECTS` | Trait tooltip labels |
| `STAGE_*` | `STAGE_ADULT` | Age stage names |
| `JOB_*` | `JOB_GATHERER` | Job names |
| `FACET_*` | `FACET_CURIOSITY` | HEXACO facet names |
| `TRAIT_*` | `TRAIT_CURIOUS` | Trait display names |
| `ACTION_*` | `ACTION_GATHER_FOOD` | Behavior action names |
| `EMOTION_MOD_*` | `EMOTION_MOD_VALENCE` | Emotion modifier labels |

## Forbidden Patterns

```gdscript
# ❌ Hardcoded string
_label.text = "Population"

# ❌ Dual-language display
header = "%s (%s)" % [name_ko, name_en]

# ❌ Direct locale branch in code
if Locale.current_locale == "ko":
    name = trait.get("name_kr", "")
else:
    name = trait.get("name_en", "")

# ❌ capitalize() as translation
var type_name = building_type.capitalize()
```

## Correct Patterns

```gdscript
# ✅ Static label
_label.text = Locale.ltr("UI_POPULATION")

# ✅ Formatted label
_pop_label.text = Locale.trf("UI_POP_FMT", {"n": pop})

# ✅ ID-based lookup (job, stage, building type)
var job_name = Locale.tr_id("JOB", entity.job)

# ✅ Data-dict lookup (trait name from JSON data)
var trait_name = Locale.tr_data(trait_dict, "name")

# ✅ Single-language header only
header = "%s %s" % [icon, trait_name]
```

## Adding New UI Strings

1. Add the key to **both** `localization/en/ui.json` and `localization/ko/ui.json`.
2. Use `Locale.ltr(key)` or `Locale.trf(key, params)` in code.
3. Verify with: `python3 -c "import json; en=set(json.load(open('localization/en/ui.json'))); ko=set(json.load(open('localization/ko/ui.json'))); print('OK' if en==ko else f'MISMATCH: {en^ko}')"`

## Verification Commands

```bash
# Korean hardcoding — must return 0 lines
grep -rn "[가-힣]" --include="*.gd" scripts/ | grep -v "##\|#.*[가-힣]\|print(\|push_warning\|push_error"

# English .text assignments — must return 0 lines
grep -rn '\.text\s*=\s*"[^"]*[A-Za-z가-힣]' --include="*.gd" scripts/ | grep -v 'Locale\.\|#\|print\|push_'

# locale_changed connections — list all UI scripts missing it
grep -rL "locale_changed" scripts/ui/*.gd
```
