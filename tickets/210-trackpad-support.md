# Ticket 210: macOS Trackpad Support [Critical]

## Objective
Add macOS trackpad pinch-zoom and two-finger scroll pan to CameraController.

## Non-goals
- No touch screen support
- No gesture configuration UI

## Files to change
- `scripts/ui/camera_controller.gd` — add InputEventMagnifyGesture and InputEventPanGesture handlers

## Steps
1. In `_unhandled_input`, add:
   - `InputEventMagnifyGesture`: factor-based zoom (factor > 1 = zoom in)
   - `InputEventPanGesture`: delta-based camera pan
2. Ensure existing mouse wheel and middle-click drag still work

## Done Definition
- Trackpad pinch zooms in/out
- Trackpad two-finger scroll pans camera
- Mouse wheel zoom still works
- Middle-click drag still works
