# Scenario Selector Activation — A-9 Slot 1

## Section 1: Implementation Intent

Wire the existing `ScenarioSelector` UI into the main game flow so players choose a scenario before simulation starts. The `ScenarioSelector` node was written but never instantiated; `RuntimeBootstrapPayload` had no `scenario_id` channel; and `SimResources` had no API to activate a named scenario ruleset.

This closes the final gap for A-9 Slot 1: the 5 scenarios (default, eternal_winter, perpetual_summer, barren_world, abundance) now load and apply their `WorldRuleset` via `activate_scenario_by_name()` before the first tick.

## Section 2: What to Build

### Rust — sim-engine `SimResources`
- `active_scenario_name: Option<String>` field
- `activate_scenario_by_name(&mut self, name: &str) -> Result<(), String>` — finds ruleset by name in registry, applies global/agent constants, sets `season_mode`, records name
- `get_active_scenario_name(&self) -> Option<&str>`

### Rust — sim-bridge `RuntimeBootstrapPayload`
- `scenario_id: String` field (`#[serde(default)]`)
- In `bootstrap_world_core()`: map `scenario_id` → ruleset name, call `activate_scenario_by_name()`

### GDScript — `scenes/main/main.gd`
- `const ScenarioSelectorClass = preload("res://scripts/ui/scenario_selector.gd")`
- `_selected_scenario_id: String = "default"`, `_scenario_selector: CanvasLayer = null`
- `_enter_setup_mode()`: headless → bypass directly; display → show ScenarioSelector
- `_on_scenario_confirmed(id)`: store id, free selector, continue setup
- `_build_runtime_bootstrap_payload()`: inject `"scenario_id": _selected_scenario_id`
- After `bootstrap_world()`: call `hud.set_active_scenario(_selected_scenario_id)`

### GDScript — `scripts/ui/hud.gd`
- `_active_scenario_id: String = "default"` field
- `set_active_scenario(id)`: sets `_scenario_label` text via `Locale.ltr()` for non-default scenarios

### Rust — sim-test harness tests (6 tests in `harness_a9b_scenario_activation`)
1. `harness_a9b_activate_eternal_winter_sets_season_mode`
2. `harness_a9b_default_has_no_active_scenario`
3. `harness_a9b_unknown_scenario_returns_error`
4. `harness_a9b_eternal_winter_season_for_tick_always_winter`
5. `harness_a9b_perpetual_summer_season_for_tick_always_summer`
6. `harness_a9b_idempotent_double_activation`

## Section 3: How to Implement

Already implemented. Verify:

```bash
cd rust && cargo test -p sim-test harness_a9b -- --nocapture
cd rust && cargo clippy --workspace -- -D warnings
cd rust && cargo test --workspace
```

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|-------------|------|------------|
| T1 | sim-engine/src/engine.rs — activate_scenario_by_name | DIRECT | — |
| T2 | sim-bridge/src/runtime_queries.rs — scenario_id + activation | DIRECT | T1 |
| T3 | scenes/main/main.gd — ScenarioSelector wiring | DIRECT | T2 |
| T4 | scripts/ui/hud.gd — set_active_scenario | DIRECT | T3 |
| T5 | sim-test/src/main.rs — 6 harness tests | DIRECT | T1 |

## Section 5: Localization Checklist

| Key | JSON File | en | ko |
|-----|-----------|----|----|
| SCENARIO_ETERNAL_WINTER_NAME | game.json | Eternal Winter | 영원한 겨울 |
| SCENARIO_PERPETUAL_SUMMER_NAME | game.json | Perpetual Summer | 영원한 여름 |
| SCENARIO_BARREN_WORLD_NAME | game.json | Barren World | 황량한 세계 |
| SCENARIO_ABUNDANCE_NAME | game.json | Abundance | 풍요 |

## Section 6: Verification

```bash
# Rust unit tests (6 harness tests)
cd rust && cargo test -p sim-test harness_a9b -- --nocapture
# Expected: 6/6 PASS

# Full workspace
cd rust && cargo test --workspace

# Clippy
cd rust && cargo clippy --workspace -- -D warnings

# Smoke test: launch game (non-headless), select "Eternal Winter", verify HUD label shows
```

Expected harness test output: 6 tests, 0 failed.
Evaluator verdict target: APPROVE (adjusted_score ≥ 90).
