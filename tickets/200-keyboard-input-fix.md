# Ticket 200: Keyboard Input Fix [Critical]

## Objective
Replace Input Map action-based input with direct keycode checks. Space/Period/Comma are non-responsive because project.godot [input] section may have incorrect physical_keycode mappings.

## Non-goals
- No new keybindings beyond what exists
- No Input Map rework (we're abandoning it entirely)

## Files to change
- `scenes/main/main.gd` — rewrite `_unhandled_input` to use `event.keycode` matching
- `project.godot` — remove [input] section entries

## Steps
1. In main.gd `_unhandled_input`, replace action checks with direct keycode match:
   - KEY_SPACE → toggle_pause
   - KEY_PERIOD → increase_speed
   - KEY_COMMA → decrease_speed
2. Remove [input] section from project.godot (no longer needed)

## Done Definition
- Space toggles pause (HUD shows ⏸/▶)
- Period increases speed (HUD updates)
- Comma decreases speed (HUD updates)
- No Input Map entries in project.godot
