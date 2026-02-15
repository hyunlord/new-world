# T-830: Entity/Building Detail Popup + E Key [Medium]

## Problem
Selection panels too small; no action history.

## Changes
1. **entity_data.gd**: Add tracking fields
   - total_gathered: float = 0.0
   - buildings_built: int = 0
   - action_history: Array = [] (max 20 entries)
2. **behavior_system.gd**: Push to action_history on action change
3. **hud.gd**: Enlarge entity panel 250x220 → 320x280
4. **New: scripts/ui/entity_detail_panel.gd**: Entity detail popup
   - 50% screen, centered, dim overlay
   - Status, needs, stats, inventory, action history
   - Close with E or Esc
5. **New: scripts/ui/building_detail_panel.gd**: Building detail popup
   - Similar layout for buildings
6. **hud.gd + main.gd**: E key opens detail popup when entity/building selected

## Files
- scripts/core/entity_data.gd
- scripts/ai/behavior_system.gd
- scripts/ui/hud.gd
- scripts/ui/entity_detail_panel.gd (NEW)
- scripts/ui/building_detail_panel.gd (NEW)
- scenes/main/main.gd

## Done
- [ ] Entity panel enlarged
- [ ] E key opens detail popup
- [ ] Action history tracked and displayed
- [ ] docs/CONTROLS.md, VISUAL_GUIDE.md, SYSTEMS.md updated
- [ ] CHANGELOG.md updated
