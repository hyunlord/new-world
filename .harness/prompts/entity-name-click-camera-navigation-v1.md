# Feature: entity-name-click-camera-navigation-v1

## Summary

GDScript-only UI/UX feature: clicking entity names in the Inspector and Chronicle panels moves the camera to that entity, and a 5-step back/forward navigation history is maintained with HUD buttons. Wildlife sprites (Wolf/Bear/Boar) are now clickable and focus the camera.

## What Was Implemented

### New files
- `scripts/ui/navigation_history.gd` — NavigationHistory (RefCounted, MAX_HISTORY=5, consecutive dedup, forward truncation)
- `scripts/ui/entity_navigation_manager.gd` — EntityNavigationManager (Node, subscribes to SimulationBus.entity_navigation_requested, emits entity_focus_requested signal, history tracking)
- `scripts/test/harness_navigation_history.gd` — 7 headless unit tests for NavigationHistory

### Modified files
- `scripts/core/simulation/simulation_bus.gd` — Added `entity_navigation_requested(entity_id: int)` signal
- `scripts/ui/camera_controller.gd` — Added `setup_navigation(nav_manager)` + `_on_entity_focus_requested()` wiring to `handle_notification_clicked`
- `scripts/ui/panels/entity_detail_panel_legacy.gd` — Emit `entity_navigation_requested` in `_navigate_to_entity()` for alive entities
- `scripts/ui/panels/chronicle_panel.gd` — Emit `entity_navigation_requested` in `_navigate_to_entity()` for alive entities
- `scripts/ui/hud.gd` — Create EntityNavigationManager, add ← Back / Forward → buttons in top bar
- `scripts/ui/renderers/entity_renderer.gd` — Decode entity_id from wildlife snapshot (offset+0, u32), store in `_wildlife_entity_ids`, add `_find_clicked_wildlife()` with 24px radius, emit navigation signals on click
- `localization/en/ui.json`, `localization/ko/ui.json` — Added UI_NAV_BACK, UI_NAV_FORWARD, UI_WILDLIFE_DETAIL_HEADER

## Architecture

The design uses SimulationBus as the decoupling hub:
- Panels emit `SimulationBus.entity_navigation_requested(entity_id)` when user clicks an entity name
- EntityNavigationManager subscribes to that signal, looks up entity tile position via entity_manager.get_entity(id).position, pushes to NavigationHistory, emits `entity_focus_requested(entity_id, tile_pos)`
- CameraController subscribes to `entity_focus_requested` via `setup_navigation()`, calls `handle_notification_clicked()` (drone shot animation)
- HUD `_back_button` / `_forward_button` call nav_manager.go_back() / go_forward() which emit `entity_selected` (to update inspector) + `entity_focus_requested` (to move camera)

## Harness Invariants to Verify

1. NavigationHistory.push() correctly tracks current entity (get_current_id())
2. Consecutive push of same entity is deduped (get_size() stays 1)
3. go_back() returns previous entity id in LIFO order
4. go_forward() returns correct id after go_back()
5. MAX_HISTORY=5 cap: pushing 10 entries results in get_size() == 5
6. Push after go_back() truncates forward stack (can_go_forward() == false)
7. can_go_back() is false with 0 or 1 entries, true with 2+

## Regression Check

Zero Rust changes. Pre-existing failing tests (harness_body_damage_api, harness_action_non_idle_ratio_steady_state, harness_a9_behavioral_mortality_mul_isolation) are unrelated to this feature — they are tracked failures from A9 (BodyHealth, ❌ in CLAUDE.md).

Expected: cargo test --workspace shows same failure set as before D1 (4 pre-existing failures, all in sim-test for unrelated systems).

## Scope Boundaries (NOT implemented)

- Bookmark system
- History persistence across save/load
- Multi-tab entity views
- Hover preview for entity names
- Dedicated Wildlife inspector panel
- Keyboard shortcuts (Ctrl+B etc)
