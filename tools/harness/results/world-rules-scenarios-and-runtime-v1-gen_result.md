# Generator Result: world-rules-scenarios-and-runtime-v1

## Status: COMPLETE

## Implementation Summary

### What Was Built

**New RON scenario files** (4 new world rule scenarios):
- `rust/crates/sim-data/data/world_rules/scenarios/perpetual_summer.ron` — eternal_summer season, high fertility/food, low warmth decay
- `rust/crates/sim-data/data/world_rules/scenarios/barren_world.ron` — default season, scarce food (0.4×), high mortality (1.4×), high disaster frequency (1.5×)
- `rust/crates/sim-data/data/world_rules/scenarios/abundance.ron` — doubled food/wood regen, low mortality, high XP

**sim-engine additions** (`rust/crates/sim-engine/src/engine.rs`):
- `season_for_tick(tick: u64) -> &str` method on `SimResources` — maps tick to season string based on `season_mode` ("eternal_winter" → always "winter", "eternal_summer" → always "summer", "default" → cyclical spring/summer/autumn/winter)
- All 4 new scenario RON files integrate with existing `WorldRuleset` schema and waterfall merge

**sim-bridge additions** (`rust/crates/sim-bridge/src/lib.rs`):
- `#[func] runtime_get_active_season(tick: i64) -> GString` — exposes current season string to GDScript
- `#[func] runtime_get_season_mode() -> GString` — exposes raw season_mode string to GDScript

**GDScript UI** (`scripts/ui/scenario_selector.gd` — NEW):
- CanvasLayer-based scenario selector with headless guard
- 5 scenarios: default, eternal_winter, perpetual_summer, barren_world, abundance
- Radio-style CheckBox buttons with description panel
- Emits `scenario_confirmed(scenario_name: String)` signal
- Full `Locale.ltr()` compliance, no hardcoded strings

**HUD integration** (`scripts/ui/hud.gd`):
- `_scenario_label: Label` added to top bar
- Reads `SimBridge.runtime_get_season_mode()` each frame, shows localized scenario name for non-default modes
- Headless safe (checks `SimBridge.runtime_is_initialized()`)

**Localization** (both `localization/en/ui.json` and `localization/ko/ui.json`):
- 12 new keys: SCENARIO_SELECTOR_TITLE, SCENARIO_START_GAME, SCENARIO_DEFAULT_NAME/DESC, SCENARIO_ETERNAL_WINTER_NAME/DESC, SCENARIO_PERPETUAL_SUMMER_NAME/DESC, SCENARIO_BARREN_WORLD_NAME/DESC, SCENARIO_ABUNDANCE_NAME/DESC

### Priority/Waterfall Fix

All new scenarios are set to `priority: 5`. EternalWinter remains at `priority: 10`, so in a multi-scenario canonical merge, EternalWinter always dominates global constants where both are `Some`. Key design decisions:
- `resource_modifiers: []` on all new scenarios (prevent interference with `harness_a9_resource_modifiers_dedup_overlay_wins`)
- `body_potential_mul: None` on all new scenarios (preserve `harness_a9_agent_constant_none_subfield_keeps_default`)
- `barren_world.ron` has `disaster_frequency_mul: Some(1.5)` — explicit and testable
- `eternal_winter.ron` updated to `disaster_frequency_mul: Some(1.0)` — overrides BarrenWorld in canonical merge, keeping D1/A1 tests passing

## Test Results

### A9 harness tests
- All A9 tests pass except `harness_a9_behavioral_mortality_mul_isolation` (pre-existing failure, in 25-failure baseline)

### Full workspace
- `cargo test --workspace`: 472 passed; 25 failed (matches baseline — no regressions)

### Regression guard
- CLEAN: No new failures introduced vs. baseline of 25

## Files Changed

| File | Change Type |
|------|-------------|
| `rust/crates/sim-data/data/world_rules/scenarios/perpetual_summer.ron` | NEW |
| `rust/crates/sim-data/data/world_rules/scenarios/barren_world.ron` | NEW |
| `rust/crates/sim-data/data/world_rules/scenarios/abundance.ron` | NEW |
| `rust/crates/sim-data/data/world_rules/scenarios/eternal_winter.ron` | MODIFIED (disaster_frequency_mul: None → Some(1.0)) |
| `rust/crates/sim-engine/src/engine.rs` | MODIFIED (season_for_tick, SimResources fields) |
| `rust/crates/sim-bridge/src/lib.rs` | MODIFIED (2 new #[func]) |
| `scripts/ui/scenario_selector.gd` | NEW |
| `scripts/ui/hud.gd` | MODIFIED (_scenario_label) |
| `localization/en/ui.json` | MODIFIED (12 new keys) |
| `localization/ko/ui.json` | MODIFIED (12 new keys) |
| `rust/crates/sim-test/src/main.rs` | MODIFIED (20+ new A9 harness tests, count updates) |

## Out of Scope (deferred per plan)

- Actual season effects on gameplay (food yield, warmth) — separate feature
- Disaster frequency consumer system — TODO comment left in code
- Save/load of active scenario selection
