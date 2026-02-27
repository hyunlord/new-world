# PROGRESS — Prompt 03: Skill System Expansion (19 → 43 skills)

## Classification Table

| Ticket | Description | Type | Tool | Status |
|--------|-------------|------|------|--------|
| T1 | stat_sync_system.gd — data-driven skill sync | 🔴 DIRECT | — | ✅ Done |
| T2 | entity_detail_panel.gd — data-driven skill display | 🔴 DIRECT | — | ✅ Done |
| T3 | stats/skills/ — 9 stone_age gathering/crafting/knowledge JSON | 🟢 DISPATCH→DIRECT | — | ✅ Done |
| T4 | stats/skills/ — 5 stone_age combat JSON | 🟢 DISPATCH→DIRECT | — | ✅ Done |
| T5 | stats/skills/ — 4 stone_age social/survival/construction JSON | 🟢 DISPATCH→DIRECT | — | ✅ Done |
| T6 | stats/skills/ — 6 tribal era JSON (social+knowledge+combat) | 🟢 DISPATCH→DIRECT | — | ✅ Done |
| T7 | localization en+ko ui.json — 24 UI_SKILL_* keys | 🟢 DISPATCH→DIRECT | — | ✅ Done |

**Dispatch ratio:** 5/7 = 71%. Implemented DIRECT this session (ask_codex MCP not available).

## Changes Made

### stats/skills/ (24 new files)
Stone age: trapping, herbalism, bone_crafting, flint_knapping, leatherworking, tracking,
  weather_reading, first_aid, brawling, spear_hunting, club_fighting, storytelling,
  swimming, fire_making, shelter_construction
Tribal: rope_making, basket_weaving, negotiation, ritual, basic_astronomy,
  irrigation, selective_breeding, fortification, javelin_throwing

### scripts/systems/record/stat_sync_system.gd
- Replaced hardcoded 5-skill array with data-driven loop over `entity.skill_levels.keys()`
- All skills the entity has trained now sync to StatCache automatically

### scripts/ui/panels/entity_detail_panel.gd
- Replaced hardcoded 5-entry `_skill_entries` array with data-driven build from `entity.skill_levels.keys()`
- Skills sorted by level descending (highest skill shown first)
- Color: flat Color(0.6, 0.8, 0.6) for all skills (data-driven label key: UI_SKILL_<name>)

### localization/en/ui.json + localization/ko/ui.json
- Added 24 UI_SKILL_* keys after UI_SKILL_SWORDSMANSHIP

## Gate
- `bash scripts/gate.sh` → ✅ PASS (43 skill files)

---

# PROGRESS — Prompt 02: Layer 4.7 Economic Tendencies Behavior Connection

## Classification Table

| Ticket | Description | Type | Tool | Status |
|--------|-------------|------|------|--------|
| T1 | game_config.gd — 11 ECON_* threshold constants | 🔴 DIRECT | — | ✅ Done |
| T2 | behavior_system.gd — dynamic deliver_to_stockpile scoring | 🟢 DISPATCH→DIRECT | — | ✅ Done |
| T3 | behavior_system.gd — share_food action + _find_hungry_neighbor | 🟢 DISPATCH→DIRECT | — | ✅ Done |
| T4 | behavior_system.gd — _current_tick field + setter | 🟢 DISPATCH→DIRECT | — | ✅ Done |
| T5 | localization en+ko events.json + game.json keys | 🟢 DISPATCH→DIRECT | — | ✅ Done |

**Dispatch ratio:** 4/5 = 80% target. Implemented DIRECT this session (ask_codex MCP not available).

## Changes Made

### scripts/core/simulation/game_config.gd
- Added 11 ECON_* constants after ECON_BEHAVIOR_WEIGHTS block:
  ECON_DELIVER_BASE_SCORE, ECON_DELIVER_SAVING_SCALE, ECON_DELIVER_MATERIALISM_SUPPRESS,
  ECON_HOARD_MATERIALISM_THRESHOLD, ECON_HOARD_CARRY_MULTIPLIER,
  ECON_SHARE_GENEROSITY_THRESHOLD, ECON_SHARE_NEIGHBOR_HUNGER_THRESHOLD,
  ECON_SHARE_MIN_SURPLUS, ECON_SHARE_FOOD_AMOUNT, ECON_SHARE_SCORE_BASE,
  ECON_SHARE_SCORE_GENEROSITY_SCALE

### scripts/ai/behavior_system.gd
- var _current_tick: int = 0 field added; set at top of execute_tick()
- deliver_to_stockpile: replaced hardcoded 0.9/0.6 score with tendency-driven scoring
  (saving boosts delivery eagerness; materialism raises carry threshold to 6.0)
- share_food scoring block: generous agents (>0.30) with food surplus seek hungry neighbors
- share_food match case: transfers food, emits entity_shared_food, clears meta
- _find_hungry_neighbor() helper: finds hungriest alive neighbor in same settlement within r=8
- Pre-existing bugfix: entity.get("erg_regressing_to_existence", false) →
  entity.erg_regressing_to_existence (Object.get() takes 1 arg in GDScript 4.6)

### localization/en/events.json + localization/ko/events.json
- 3 new keys: EVT_ENTITY_SHARED_FOOD, EVT_ENTITY_HOARDING, EVT_ECON_TENDENCY_SHIFT

### localization/en/game.json + localization/ko/game.json
- 1 new key: ACTION_SHARE_FOOD (in game.json per ACTION_* convention)

## Verification
- Gate: [gate] PASS ✅
- ECON_HOARD_MATERIALISM_THRESHOLD + ECON_SHARE_GENEROSITY_THRESHOLD in game_config.gd ✅
- _find_hungry_neighbor + share_food in behavior_system.gd (≥5 lines) ✅
- EVT_ENTITY_SHARED_FOOD in en/events.json ✅

---

# PROGRESS — Prompt 01: Needs Expansion + ERG Frustration-Regression

## Classification Table

| Ticket | Description | Type | Tool | Status |
|--------|-------------|------|------|--------|
| T1 | game_config.gd — flip flag + ERG constants | 🔴 DIRECT | — | ✅ Done |
| T2 | entity_data.gd — 4 ERG fields + serialize/deserialize | 🟢 DISPATCH→DIRECT | — | ✅ Done |
| T3 | movement_system.gd — sit_by_fire / seek_shelter arrival | 🟢 DISPATCH→DIRECT | — | ✅ Done |
| T4 | needs_system.gd — `_update_erg_frustration()` + `_last_tick` | 🟢 DISPATCH→DIRECT | — | ✅ Done |
| T5 | behavior_system.gd — ERG score boost block | 🟢 DISPATCH→DIRECT | — | ✅ Done |
| T6 | localization/en + localization/ko events.json | 🟢 DISPATCH→DIRECT | — | ✅ Done |

**Dispatch ratio note:** T2-T6 classified DISPATCH but implemented DIRECT.
**Justification:** ask_codex MCP deferred tool not discoverable this session.
All tickets self-contained, low-risk, exactly-specified — direct implementation appropriate.

## Changes Made

### scripts/core/simulation/game_config.gd
- Line 398: `NEEDS_EXPANSION_ENABLED = false` → `true`
- Lines 439-447: Added 6 ERG constants (ERG_FRUSTRATION_WINDOW, thresholds, boosts, stress rate)

### scripts/core/entity/entity_data.gd
- Lines 234-237: 4 ERG fields (erg_growth_frustration_ticks, erg_relatedness_frustration_ticks, erg_regressing_to_existence, erg_regressing_to_relatedness)
- Lines 369-372: Added to to_dict()
- Lines 531-534: Added to from_dict()

### scripts/systems/world/movement_system.gd
- Lines 196-229: sit_by_fire → warmth restore + emit entity_warmed
- Lines 212-229: seek_shelter → warmth+safety restore + emit entity_sheltered

### scripts/systems/psychology/needs_system.gd
- Line 10: var _last_tick: int = 0
- Line 27: _last_tick = tick (at top of execute_tick)
- Line 196: _update_erg_frustration(entity) call (end of loop body)
- Lines 237-239: set_stress_system() setter
- Lines 244-291: _update_erg_frustration() — growth + relatedness frustration regression

### scripts/ai/behavior_system.gd
- Lines 309-317: ERG score multiplier block (before economic modifiers)

### localization/en/events.json + localization/ko/events.json
- 5 new keys each: EVT_ENTITY_DRANK, EVT_ENTITY_WARMED, EVT_ENTITY_SHELTERED,
  EVT_ERG_REGRESSION_STARTED, EVT_ERG_RELATEDNESS_REGRESSION

## Verification
- NEEDS_EXPANSION_ENABLED=true ✅
- 12 erg_ references in entity_data.gd (4 fields + 4 to_dict + 4 from_dict) ✅
- _update_erg_frustration: call at line 196, definition at line 244 ✅
- _last_tick declared + injected ✅
- set_stress_system() exists ✅
- No hardcoded strings ✅
- 5 locale keys in en + ko ✅
