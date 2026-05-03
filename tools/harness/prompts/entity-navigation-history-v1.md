# Entity Navigation History (D1)

## Summary

Add browser-style back/forward navigation history for the entity inspector panel.

## User Story

When inspecting agents or wildlife, clicking linked entities (in chronicle, entity detail, etc.)
should allow the player to navigate back to the previous entity — like browser history.

## Scope

### New Files
- `scripts/ui/navigation_history.gd` — Stack-based history (MAX_HISTORY=5), push/go_back/go_forward
- `scripts/ui/entity_navigation_manager.gd` — Wraps NavigationHistory, emits `entity_focus_requested` to camera
- `scripts/test/harness_navigation_history.gd` — 12 headless unit tests

### Modified Files
- `scripts/core/simulation/simulation_bus.gd` — Added `entity_navigation_requested(entity_id)` signal
- `scripts/ui/camera_controller.gd` — `setup_navigation()` wires nav_manager focus signal to camera
- `scripts/ui/renderers/entity_renderer.gd` — Wildlife click emits `entity_navigation_requested`
- `scripts/ui/hud.gd` — Back/Forward buttons in top bar, EntityNavigationManager initialization
- `scripts/ui/panels/chronicle_panel.gd` — `_navigate_to_entity` emits navigation signal
- `scripts/ui/panels/entity_detail_panel_legacy.gd` — Same

### Localization
- `UI_NAV_BACK`, `UI_NAV_FORWARD` keys added to en/ and ko/

## Acceptance Criteria

1. NavigationHistory.push() deduplicates consecutive identical IDs
2. go_back() returns previous ID and updates current_index
3. go_forward() returns next ID after go_back()
4. Sentinel: go_back() at index 0 returns -1, state unchanged
5. Sentinel: go_forward() at end returns -1, state unchanged
6. MAX_HISTORY=5 truncates oldest entries on overflow
7. push() after go_back() truncates forward stack
8. Back/forward buttons in HUD are disabled when history is empty/at boundary
9. Wildlife click triggers navigation history push
10. Chronicle and entity_detail panel navigation pushes history
