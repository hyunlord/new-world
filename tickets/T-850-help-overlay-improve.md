# T-850: Help Overlay Improvements [Low]

## Problem
Help too small, doesn't pause simulation.

## Changes
1. **hud.gd**: H key pauses simulation when help opens
   - Track `_was_running_before_help` state
   - Resume on close if was running
2. **hud.gd**: Enlarge help to 65% of viewport
   - Title: 24px, body: 16px
   - Two-column layout (Camera/Game, Panels/Display)
3. **main.gd**: Pass sim_engine pause state through hud

## Files
- scripts/ui/hud.gd
- scenes/main/main.gd

## Done
- [ ] H pauses simulation
- [ ] Help panel fills 65% of screen
- [ ] Two-column layout readable
- [ ] docs/CONTROLS.md, VISUAL_GUIDE.md updated
- [ ] CHANGELOG.md updated
