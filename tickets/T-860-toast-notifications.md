# T-860: Toast Notification Improvements [Low]

## Problem
Toasts invisible or missing. Position overlaps with HUD.

## Changes
1. **hud.gd**: Move toast position to left side, below top bar
   - toast_x=20, toast_start_y=40
2. **hud.gd**: Improve toast styling
   - Background color bar per type (not just colored text)
   - 14px font, min height 28px
   - Duration: 4s (was 3s), fade last 1s (was 0.5s)
3. **hud.gd**: Better event triggers
   - Population milestones: every 10 (was every 50)
   - Building completed: batch every 5
   - Food shortage warning: settlement food < pop * 0.5
4. **main.gd**: Startup toast "WorldSim started! Pop: N"

## Files
- scripts/ui/hud.gd
- scenes/main/main.gd

## Done
- [ ] Toasts visible on left side
- [ ] Background color bars per type
- [ ] Startup toast appears
- [ ] docs/VISUAL_GUIDE.md, SYSTEMS.md updated
- [ ] CHANGELOG.md updated
