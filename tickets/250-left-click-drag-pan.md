# Ticket 250: Left-Click Drag Pan + Click Select Coexistence

## Objective
Add left-click drag to pan camera. Short clicks (< 5px movement) still select entities.

## Non-goals
- No right-click context menu
- No drag selection box

## Files to change
- `scripts/ui/camera_controller.gd` — add left-click drag with threshold detection

## Steps
1. Add state vars: _left_dragging, _left_drag_start, _left_was_dragged, DRAG_THRESHOLD=5.0
2. On left press: start tracking drag
3. On mouse motion while left held: if moved > threshold, mark as drag and pan camera
4. On left release: if was drag, consume event (set_input_as_handled); if not, let it pass to EntityRenderer
5. Camera node is after EntityRenderer in scene tree → receives _unhandled_input first

## Done Definition
- Left-click drag pans camera
- Short left-click selects entity (existing behavior)
- Drag release doesn't trigger selection
- All existing inputs still work
