# T-2017-01: Detail Panel Deceased Entity Sticking Bug

## Objective
Fix bug where viewing a deceased entity's detail panel causes the panel to stay stuck on that deceased entity even when selecting a different living entity.

## Root Cause Analysis (already completed)

The bug is in `scripts/ui/entity_detail_panel.gd`.

**Selection flow:**
1. `popup_manager.open_entity(entity_id)` calls `_entity_panel.set_entity_id(entity_id)` (popup_manager.gd:104)
2. `set_entity_id()` (line 76-78) sets `_entity_id` and `_scroll_offset` but does NOT clear `_showing_deceased`
3. When a deceased entity was previously shown via `_show_deceased()`, `_showing_deceased = true` persists
4. `_draw()` at line 120-122 checks `_showing_deceased` FIRST and short-circuits to `_draw_deceased()`, ignoring the new `_entity_id`

**Current `set_entity_id()` (line 76-78):**
```gdscript
func set_entity_id(id: int) -> void:
	_entity_id = id
	_scroll_offset = 0.0
```

**The fix:** Clear deceased mode in `set_entity_id()`:
```gdscript
func set_entity_id(id: int) -> void:
	_entity_id = id
	_scroll_offset = 0.0
	_showing_deceased = false
	_deceased_record = {}
```

This ensures ALL code paths that set a new entity_id (via `set_entity_id`) automatically exit deceased mode. The key callers:
- `popup_manager.open_entity()` → calls `set_entity_id()` — **this is the buggy path, now fixed**
- `_navigate_to_entity()` → calls `set_entity_id()` then `_showing_deceased = false` — redundant but harmless
- `show_entity_or_deceased()` → calls `_showing_deceased = false` then `set_entity_id()` — correct

Note: `_show_deceased()` sets `_entity_id` DIRECTLY (not via `set_entity_id`), so it won't interfere with the fix.

## File to Modify

### `scripts/ui/entity_detail_panel.gd`

Find `set_entity_id` function (around line 76-78) and add the two lines to clear deceased state:

```gdscript
func set_entity_id(id: int) -> void:
	_entity_id = id
	_scroll_offset = 0.0
	_showing_deceased = false
	_deceased_record = {}
```

That's it. Two lines added.

## Non-goals
- Do NOT refactor the selection flow or merge code paths
- Do NOT remove the now-redundant `_showing_deceased = false` in `_navigate_to_entity()` — leave existing code alone
- Do NOT modify popup_manager.gd, hud.gd, or list_panel.gd
- Do NOT modify `_show_deceased()`, `show_entity_or_deceased()`, or `_draw()`

## Acceptance Criteria
- [ ] `set_entity_id()` clears `_showing_deceased` to `false`
- [ ] `set_entity_id()` clears `_deceased_record` to `{}`
- [ ] No other functions modified
- [ ] No GDScript parse errors
- [ ] Expected behavior: view deceased → click living entity → panel shows living entity info

## Godot 4.6 Notes
- entity_detail_panel.gd extends Control (Node-based), CAN keep class_name
- No `class_name` issues here since it's a UI script
