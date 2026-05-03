# Feature: a4-causal-tracking-exposure

## Summary

Expose the existing CausalLog backend to GDScript via SimBridge, and display
recent causal events in the entity detail panel (v5 Events tab).

The CausalLog backend was already fully implemented (12+ push sites, ring buffer
32 events per entity). This feature adds the "exposure layer" only — no backend
changes.

## Changes Made

### rust/crates/sim-bridge/src/lib.rs
- Added `use sim_core::CausalEvent;` import
- Added `causal_events_to_vardict(events: Vec<&CausalEvent>) -> Array<VarDictionary>` helper
- Added `recent_causes` field to `runtime_get_entity_detail()` (max 8 events, newest first)

### scripts/ui/panels/entity_detail_panel_v5.gd
- Modified `_refresh_events()` to read `_detail.get("recent_causes", [])` from bridge
- Added "최근 인과 / Recent Causes" section in Events tab showing top 5 events
- Each entry: localized cause label + signed magnitude
- Fallback to raw kind string if locale key missing

### localization/ko/ui.json + localization/en/ui.json
- Added `UI_CAUSAL_RECENT`, `CAUSE_PRAY`, `CAUSE_MOURN`, `CAUSE_RITUAL_COMFORT`,
  `CAUSE_HEARTH_WARMTH`, `CAUSE_SHELTER_SAFETY`, `CAUSE_FOOD_GRADIENT`,
  `CAUSE_BAND_LEADER_ELECTED`, `CAUSE_CONFLICT`, `CAUSE_TOOL_BROKEN`

### rust/crates/sim-test/src/main.rs
- `harness_a4_causal_log_populated_over_simulation`: ≥10 entries in 1000 ticks
- `harness_a4_entity_has_recent_causes`: ≥1 entity with recent events after 2000 ticks
- `harness_a4_pray_recorded_in_causal_log`: Pray produces kind='pray' causal entry

## Verification

- cargo check: clean
- cargo clippy: clean
- harness_a4_*: 3/3 PASS (pray_count=4, delta=1184, entity has 8 events)
- Full sim-test: 301 passed, 0 failed (298 + 3 new)

## Scope

- Backend changes: 0
- No new causal event push sites added
- No filter/search UI
- No global causal viewer
- No causal log persistence
