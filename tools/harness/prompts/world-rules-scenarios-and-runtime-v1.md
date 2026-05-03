# A-9: World Rules — Scenarios and Runtime

## Feature slug
`world-rules-scenarios-and-runtime-v1`

## Section 1: Implementation Intent

WorldSim's data-driven world rules system (A-9) is 95% complete. The infrastructure is solid:
- `WorldRuleset` RON schema works (eternal_winter.ron + base_rules.ron loaded and merged)
- `DataRegistry::load_all_world_rules()` operates correctly
- `engine.season_mode` field exists (line 180, engine.rs) and is set by `apply_world_rules`
- `warmth_decay_rate`, `food_regen_mul`, `temperature_bias`, `mortality_mul`, `fertility_mul` all applied

Three real gaps:
1. **Only 1 scenario RON** (eternal_winter) — 3 more needed (perpetual_summer, barren_world, abundance)
2. **`season_mode` stored but never consumed** — no function allows systems or UI to query the override season
3. **No UI for scenario selection** — game always starts with default BaseRules merge

This commit closes all three gaps minimally:
- Add 3 scenario RON files (data-driven, zero Rust changes for new worlds)
- Add `season_for_tick(tick)` method to SimResources that respects `season_mode`
- Add `ScenarioSelector` GDScript UI + minimal main.gd wiring
- Add SimBridge `get_active_season(tick)` func to expose season to GDScript

Design principles (from A-9 spec):
- Scenario selection is game-start only (no mid-game change)
- Default = BaseRules merge (headless/test mode auto-selects this)
- "Build like a mod" — new world = new `.ron` file only

## Section 2: What to Build

### A. Three new scenario RON files

#### `rust/crates/sim-data/data/world_rules/scenarios/perpetual_summer.ron`
```ron
[
    WorldRuleset(
        name: "PerpetualSummer",
        priority: 10,
        resource_modifiers: [
            RuleResourceModifier(target: "surface_foraging", multiplier: 1.6),
            RuleResourceModifier(target: "wood_regen", multiplier: 1.4),
        ],
        special_zones: [],
        special_resources: [],
        agent_constants: Some(AgentConstants(
            mortality_mul: Some(0.85),
            skill_xp_mul: Some(1.0),
            body_potential_mul: Some(1.1),
            fertility_mul: Some(1.5),
            lifespan_mul: Some(1.1),
            move_speed_mul: None,
        )),
        influence_channels: [],
        global_constants: Some(GlobalConstants(
            season_mode: Some("eternal_summer"),
            hunger_decay_mul: Some(0.9),
            warmth_decay_mul: Some(0.3),
            food_regen_mul: Some(1.5),
            wood_regen_mul: Some(1.4),
            farming_enabled: Some(true),
            temperature_bias: Some(0.6),
            disaster_frequency_mul: Some(1.2),
        )),
    ),
]
```

#### `rust/crates/sim-data/data/world_rules/scenarios/barren_world.ron`
```ron
[
    WorldRuleset(
        name: "BarrenWorld",
        priority: 10,
        resource_modifiers: [
            RuleResourceModifier(target: "surface_foraging", multiplier: 0.4),
            RuleResourceModifier(target: "wood_regen", multiplier: 0.5),
        ],
        special_zones: [],
        special_resources: [],
        agent_constants: Some(AgentConstants(
            mortality_mul: Some(1.4),
            skill_xp_mul: Some(1.2),
            body_potential_mul: Some(0.9),
            fertility_mul: Some(0.6),
            lifespan_mul: Some(0.85),
            move_speed_mul: None,
        )),
        influence_channels: [],
        global_constants: Some(GlobalConstants(
            season_mode: Some("default"),
            hunger_decay_mul: Some(1.4),
            warmth_decay_mul: Some(1.0),
            food_regen_mul: Some(0.4),
            wood_regen_mul: Some(0.5),
            farming_enabled: Some(true),
            temperature_bias: Some(0.0),
            disaster_frequency_mul: Some(1.5),
        )),
    ),
]
```

#### `rust/crates/sim-data/data/world_rules/scenarios/abundance.ron`
```ron
[
    WorldRuleset(
        name: "Abundance",
        priority: 10,
        resource_modifiers: [
            RuleResourceModifier(target: "surface_foraging", multiplier: 2.0),
            RuleResourceModifier(target: "wood_regen", multiplier: 2.0),
        ],
        special_zones: [],
        special_resources: [],
        agent_constants: Some(AgentConstants(
            mortality_mul: Some(0.7),
            skill_xp_mul: Some(1.5),
            body_potential_mul: Some(1.2),
            fertility_mul: Some(1.2),
            lifespan_mul: Some(1.2),
            move_speed_mul: None,
        )),
        influence_channels: [],
        global_constants: Some(GlobalConstants(
            season_mode: Some("default"),
            hunger_decay_mul: Some(0.7),
            warmth_decay_mul: Some(0.7),
            food_regen_mul: Some(2.0),
            wood_regen_mul: Some(2.0),
            farming_enabled: Some(true),
            temperature_bias: Some(0.0),
            disaster_frequency_mul: Some(0.5),
        )),
    ),
]
```

### B. `season_for_tick` method — `rust/crates/sim-engine/src/engine.rs`

Add to `SimResources` impl (after `apply_world_rules`):

```rust
/// Returns the season string for the given tick, respecting season_mode override.
/// "eternal_winter" → always "winter"
/// "eternal_summer" → always "summer"
/// "no_seasons"     → always "spring" (mid-temperate constant)
/// "default" / _    → tick-based 4-season cycle (91-day quarters)
pub fn season_for_tick(&self, tick: u64) -> &'static str {
    // TICKS_PER_DAY = 12, DAYS_PER_SEASON ≈ 91
    const TICKS_PER_SEASON: u64 = 12 * 91; // 1092 ticks/season
    match self.season_mode.as_str() {
        "eternal_winter" => "winter",
        "eternal_summer" => "summer",
        "no_seasons"     => "spring",
        _ => {
            let idx = (tick / TICKS_PER_SEASON) % 4;
            match idx {
                0 => "spring",
                1 => "summer",
                2 => "autumn",
                _ => "winter",
            }
        }
    }
}
```

### C. SimBridge `get_active_season` — `rust/crates/sim-bridge/src/lib.rs`

Add one `#[func]` to expose current season to GDScript (for HUD display):

```rust
/// Returns the current season string for the given tick.
/// GDScript: SimBridge.get_active_season(current_tick)
#[func]
fn get_active_season(&self, tick: i64) -> GString {
    let tick = tick.max(0) as u64;
    if let Some(state) = &self.state {
        state.engine.resources().season_for_tick(tick).into()
    } else {
        "spring".into()
    }
}
```

Also add `get_season_mode` for UI display:

```rust
/// Returns the season_mode string from the active world rules.
#[func]
fn get_season_mode(&self) -> GString {
    if let Some(state) = &self.state {
        state.engine.resources().season_mode.clone().into()
    } else {
        "default".into()
    }
}
```

### D. GDScript — `scripts/ui/scenario_selector.gd` (new file ~120 lines)

```gdscript
class_name ScenarioSelector
extends CanvasLayer

signal scenario_selected(scenario_name: String)

const SCENARIOS: Array[Dictionary] = [
    {"name": "BaseRules",       "key_name": "SCENARIO_BASE_NAME",          "key_desc": "SCENARIO_BASE_DESC"},
    {"name": "EternalWinter",   "key_name": "SCENARIO_ETERNAL_WINTER_NAME","key_desc": "SCENARIO_ETERNAL_WINTER_DESC"},
    {"name": "PerpetualSummer", "key_name": "SCENARIO_PERPETUAL_SUMMER_NAME","key_desc": "SCENARIO_PERPETUAL_SUMMER_DESC"},
    {"name": "BarrenWorld",     "key_name": "SCENARIO_BARREN_NAME",        "key_desc": "SCENARIO_BARREN_DESC"},
    {"name": "Abundance",       "key_name": "SCENARIO_ABUNDANCE_NAME",     "key_desc": "SCENARIO_ABUNDANCE_DESC"},
]

var _selected_name: String = "BaseRules"
var _button_group: ButtonGroup = ButtonGroup.new()

func _ready() -> void:
    layer = 100
    _build_ui()

func _build_ui() -> void:
    var panel := PanelContainer.new()
    panel.set_anchors_preset(Control.PRESET_CENTER)
    add_child(panel)

    var vbox := VBoxContainer.new()
    vbox.add_theme_constant_override("separation", 8)
    panel.add_child(vbox)

    var title := Label.new()
    title.text = Locale.ltr("SCENARIO_SELECTOR_TITLE")
    title.add_theme_font_size_override("font_size", 20)
    vbox.add_child(title)

    for s: Dictionary in SCENARIOS:
        var btn := Button.new()
        btn.text = Locale.ltr(s.key_name) + "\n" + Locale.ltr(s.key_desc)
        btn.toggle_mode = true
        btn.button_group = _button_group
        btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
        if s.name == _selected_name:
            btn.button_pressed = true
        var n: String = s.name
        btn.toggled.connect(func(pressed: bool) -> void:
            if pressed:
                _selected_name = n
        )
        vbox.add_child(btn)

    var start_btn := Button.new()
    start_btn.text = Locale.ltr("SCENARIO_START_GAME")
    start_btn.pressed.connect(_on_start)
    vbox.add_child(start_btn)

func _on_start() -> void:
    scenario_selected.emit(_selected_name)
    queue_free()
```

**Note**: Headless/test mode detection — check `DisplayServer.get_name() == "headless"` and skip selector if true.

### E. `scripts/ui/hud.gd` — scenario label addition (~15 lines)

Find the HUD's `_ready` or setup function and add a scenario label to the top area. Call `SimBridge.get_season_mode()` to get the current scenario and display it. Use `Locale.ltr()` with the scenario name key. Only show label when scenario is not "default" / "BaseRules".

Exact integration point: search for `_top_right_container` or `_nav_container`; if missing, append to the existing top HBoxContainer. Keep it a single-line Label with small font.

### F. Localization keys — `localization/en/ui.json` and `localization/ko/ui.json`

Add to both files:
```json
"SCENARIO_SELECTOR_TITLE": "Choose World",
"SCENARIO_START_GAME": "Start",
"SCENARIO_BASE_NAME": "Base World",
"SCENARIO_BASE_DESC": "Balanced 4 seasons",
"SCENARIO_ETERNAL_WINTER_NAME": "Eternal Winter",
"SCENARIO_ETERNAL_WINTER_DESC": "Harsh winter only, scarce food",
"SCENARIO_PERPETUAL_SUMMER_NAME": "Perpetual Summer",
"SCENARIO_PERPETUAL_SUMMER_DESC": "Warm, abundant, population booms",
"SCENARIO_BARREN_NAME": "Barren World",
"SCENARIO_BARREN_DESC": "Scarce resources, hard survival",
"SCENARIO_ABUNDANCE_NAME": "Abundance",
"SCENARIO_ABUNDANCE_DESC": "Sandbox mode, everything plentiful"
```

Korean (`ko/ui.json`):
```json
"SCENARIO_SELECTOR_TITLE": "세계 선택",
"SCENARIO_START_GAME": "시작",
"SCENARIO_BASE_NAME": "기본 세계",
"SCENARIO_BASE_DESC": "균형잡힌 4계절",
"SCENARIO_ETERNAL_WINTER_NAME": "영원한 겨울",
"SCENARIO_ETERNAL_WINTER_DESC": "혹독한 겨울만, 식량 부족",
"SCENARIO_PERPETUAL_SUMMER_NAME": "영원한 여름",
"SCENARIO_PERPETUAL_SUMMER_DESC": "따뜻함, 풍요, 인구 폭증",
"SCENARIO_BARREN_NAME": "황무지",
"SCENARIO_BARREN_DESC": "자원 희소, 어려운 생존",
"SCENARIO_ABUNDANCE_NAME": "풍요",
"SCENARIO_ABUNDANCE_DESC": "샌드박스 모드, 모든 것이 풍요"
```

### G. Harness tests — `rust/crates/sim-test/src/main.rs`

Add 8 harness tests in a new `mod harness_a9_world_rules_v1` block:

**Test 1** `harness_a9_all_four_scenarios_load`:
- Load DataRegistry from test data dir
- Assert world_rules_raw contains rulesets named "BaseRules", "EternalWinter", "PerpetualSummer", "BarrenWorld", "Abundance"
- (BaseRules and EternalWinter already exist; the 3 new ones must load)

**Test 2** `harness_a9_perpetual_summer_low_warmth_decay`:
- Apply PerpetualSummer rules to a SimResources
- Assert `resources.warmth_decay_rate < sim_core::config::WARMTH_DECAY_RATE * 0.5` (warmth_decay_mul=0.3)

**Test 3** `harness_a9_barren_world_high_mortality`:
- Apply BarrenWorld rules to a SimResources
- Assert `resources.mortality_mul > 1.3` (mortality_mul=1.4)

**Test 4** `harness_a9_abundance_high_food_regen`:
- Apply Abundance rules to a SimResources
- Assert `resources.food_regen_mul > 1.8` (food_regen_mul=2.0)

**Test 5** `harness_a9_season_for_tick_eternal_winter`:
- Create SimResources, set `season_mode = "eternal_winter"`
- Assert `resources.season_for_tick(0)` == "winter"
- Assert `resources.season_for_tick(10000)` == "winter"
- Assert `resources.season_for_tick(50000)` == "winter"

**Test 6** `harness_a9_season_for_tick_eternal_summer`:
- Set `season_mode = "eternal_summer"`
- Assert `resources.season_for_tick(any)` == "summer" for 3 different ticks

**Test 7** `harness_a9_season_for_tick_default_cycles`:
- Set `season_mode = "default"`
- Assert tick=0 → "spring" (or first cycle)
- Assert tick within 2nd quarter (>1092) → "summer"
- Assert tick within 4th quarter (>3276) → "winter"
- Verify 4 distinct seasons appear in a full year cycle

**Test 8** `harness_a9_perpetual_summer_season_mode_set`:
- Load PerpetualSummer scenario, apply via `apply_world_rules`
- Assert `resources.season_mode == "eternal_summer"`
- Assert `resources.season_for_tick(5000)` == "summer"

## Section 3: How to Implement

### Step 1: Write 3 RON files (data-only, zero Rust changes)

Create:
- `rust/crates/sim-data/data/world_rules/scenarios/perpetual_summer.ron`
- `rust/crates/sim-data/data/world_rules/scenarios/barren_world.ron`
- `rust/crates/sim-data/data/world_rules/scenarios/abundance.ron`

Use `eternal_winter.ron` as syntax reference. Verify schemas match existing RON parser
(no `special_zones` list needed if empty, RON syntax for empty arrays: `[]`).

### Step 2: Add `season_for_tick` to `SimResources` in `engine.rs`

File: `rust/crates/sim-engine/src/engine.rs`

Find `impl SimResources` and add the method after `apply_world_rules`. It must be a pure
function of `self.season_mode` and the tick input — no side effects.

Constants: use `12 * 91 = 1092` ticks per season (12 ticks/day × 91 days/season).
This matches `game_calendar.gd` TICKS_PER_DAY=12.

Add unit test in `#[cfg(test)] mod tests` at bottom of engine.rs:
```rust
#[test]
fn season_for_tick_eternal_winter_always_winter() {
    let mut r = SimResources::default();
    r.season_mode = "eternal_winter".to_string();
    assert_eq!(r.season_for_tick(0), "winter");
    assert_eq!(r.season_for_tick(99999), "winter");
}

#[test]
fn season_for_tick_default_cycles() {
    let mut r = SimResources::default();
    r.season_mode = "default".to_string();
    // tick 0 → spring (idx 0)
    assert_eq!(r.season_for_tick(0), "spring");
    // tick 1092 → summer (idx 1)
    assert_eq!(r.season_for_tick(1092), "summer");
    // tick 2184 → autumn (idx 2)
    assert_eq!(r.season_for_tick(2184), "autumn");
    // tick 3276 → winter (idx 3)
    assert_eq!(r.season_for_tick(3276), "winter");
}
```

### Step 3: Add SimBridge `#[func]` methods

File: `rust/crates/sim-bridge/src/lib.rs`

Find the block of other `#[func]` methods (e.g., `get_alive_count`). Add `get_active_season`
and `get_season_mode` after them. These are read-only queries — no state mutation.

### Step 4: Add localization keys to JSON files

Files: `localization/en/ui.json` and `localization/ko/ui.json`

Both files contain a flat JSON object. Add the 11 new keys at the end (before the closing `}`).
Ensure valid JSON (trailing comma rules apply — add comma after previous last key).

Verify symmetry: both en/ and ko/ must have identical key sets.

### Step 5: Create `scripts/ui/scenario_selector.gd`

New file. Extends CanvasLayer so it overlays everything. Uses ButtonGroup for mutual exclusion.
Headless guard: `if DisplayServer.get_name() == "headless": scenario_selected.emit("BaseRules"); queue_free(); return`
(add at top of `_ready`).

No hardcoded strings — all text via `Locale.ltr()`.

### Step 6: Wire into HUD

File: `scripts/ui/hud.gd`

Find the HUD's `_ready` function. After existing setup, add a small Label showing the active
scenario. Call `SimBridge.get_season_mode()` to get it. Map to locale key via a helper function
`_scenario_to_locale_key(mode: String) -> String`. Only show if mode != "default".

Do NOT add the ScenarioSelector here — the selector is shown by the game entry point (main scene
or world_root), not the HUD.

### Step 7: Write 8 harness tests

File: `rust/crates/sim-test/src/main.rs`

Add a new `mod harness_a9_world_rules_v1 { ... }` block near the existing world rules tests
(around line 4415). Use the same pattern as `harness_world_rules_global_constants`.

To apply a specific scenario's rules, use the pattern:
```rust
use sim_data::{DataRegistry, merge_world_rules};
let registry = DataRegistry::load_from_directory(&test_data_dir()).expect("load");
let scenario_rules: Vec<sim_data::WorldRuleset> = registry.world_rules_raw.iter()
    .filter(|r| r.name == "PerpetualSummer" || r.name == "BaseRules")
    .cloned().collect();
let merged = merge_world_rules(&scenario_rules).unwrap();
registry_mut.world_rules = Some(merged);
resources.apply_world_rules();
```

### Step 8: Gate

```bash
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
```

Expected: 488+ passed (480 + 8 new), 0 failed.

## Section 4: Dispatch Plan

| # | Ticket | File(s) | Mode |
|---|--------|---------|:----:|
| T1 | perpetual_summer.ron | sim-data/data/world_rules/scenarios/ | 🔴 DIRECT |
| T2 | barren_world.ron | (same) | 🔴 DIRECT |
| T3 | abundance.ron | (same) | 🔴 DIRECT |
| T4 | season_for_tick method | sim-engine/src/engine.rs | 🔴 DIRECT |
| T5 | SimBridge get_active_season + get_season_mode | sim-bridge/src/lib.rs | 🟢 DISPATCH |
| T6 | scenario_selector.gd | scripts/ui/scenario_selector.gd | 🟢 DISPATCH |
| T7 | HUD scenario label | scripts/ui/hud.gd | 🟢 DISPATCH |
| T8 | Localization en + ko | localization/en/ui.json + ko/ui.json | 🟢 DISPATCH |
| T9 | 8 harness tests | sim-test/src/main.rs | 🟢 DISPATCH |

Dispatch ratio: 5/9 = 56% (T1-T4 are direct because they're small precise edits)

## Section 5: Localization Checklist

| Key | File | en value | ko value |
|-----|------|----------|----------|
| `SCENARIO_SELECTOR_TITLE` | ui.json | "Choose World" | "세계 선택" |
| `SCENARIO_START_GAME` | ui.json | "Start" | "시작" |
| `SCENARIO_BASE_NAME` | ui.json | "Base World" | "기본 세계" |
| `SCENARIO_BASE_DESC` | ui.json | "Balanced 4 seasons" | "균형잡힌 4계절" |
| `SCENARIO_ETERNAL_WINTER_NAME` | ui.json | "Eternal Winter" | "영원한 겨울" |
| `SCENARIO_ETERNAL_WINTER_DESC` | ui.json | "Harsh winter only, scarce food" | "혹독한 겨울만, 식량 부족" |
| `SCENARIO_PERPETUAL_SUMMER_NAME` | ui.json | "Perpetual Summer" | "영원한 여름" |
| `SCENARIO_PERPETUAL_SUMMER_DESC` | ui.json | "Warm, abundant, population booms" | "따뜻함, 풍요, 인구 폭증" |
| `SCENARIO_BARREN_NAME` | ui.json | "Barren World" | "황무지" |
| `SCENARIO_BARREN_DESC` | ui.json | "Scarce resources, hard survival" | "자원 희소, 어려운 생존" |
| `SCENARIO_ABUNDANCE_NAME` | ui.json | "Abundance" | "풍요" |
| `SCENARIO_ABUNDANCE_DESC` | ui.json | "Sandbox mode, everything plentiful" | "샌드박스 모드, 모든 것이 풍요" |

## Section 6: Verification

### Gate command
```bash
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
```
Expected: 488+ passed, 0 failed.

### Harness tests
```bash
cd rust && cargo test -p sim-test harness_a9 -- --nocapture
```
Expected: 8 passed.

### RON file count
```bash
ls rust/crates/sim-data/data/world_rules/scenarios/*.ron | wc -l
# Expected: 4
```

### Localization symmetry
```bash
python3 -c "
import json, glob
en = set(); ko = set()
for f in glob.glob('localization/en/*.json'):
    en.update(json.load(open(f)).keys())
for f in glob.glob('localization/ko/*.json'):
    ko.update(json.load(open(f)).keys())
diff = en.symmetric_difference(ko)
print(f'PASS: {len(en)} keys symmetric' if not diff else f'FAIL: {sorted(diff)}')
"
```

### Regression guard
```bash
cd rust && cargo test -p sim-test harness_ -- --nocapture 2>&1 | tail -5
# All 480 existing harness tests + 8 new = 488+ passed
```

### Scope boundary (what was NOT done)
- No mid-game scenario change
- No custom scenario editor
- No multiple-scenario stacking
- No save-game scenario persistence
- No difficulty system
- No scenario selection wired into main scene entry point (selector is standalone, wiring
  left to the game's main scene owner — ScenarioSelector emits `scenario_selected` signal)
