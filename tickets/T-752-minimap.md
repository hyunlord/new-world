# T-752: Minimap Panel

**Priority:** Critical | **Status:** Open

## Description
160x160px minimap in top-right corner. Shows biome colors, building markers, entity dots, camera viewport rect. Click to navigate. M key toggle.

## Implementation
- New file: `scripts/ui/minimap_panel.gd` (extends Control)
- Added as child of HUD CanvasLayer
- Image-based rendering, refreshed every 20 ticks
- Camera viewport shown as white rectangle
- Click → camera teleport
- M key toggle in main.gd

## Done Definition
- [ ] Minimap renders biome, buildings, entities
- [ ] Camera viewport rect shown
- [ ] Click navigates camera
- [ ] M key toggles visibility
- [ ] docs/VISUAL_GUIDE.md, CONTROLS.md updated
- [ ] CHANGELOG.md updated
- [ ] Gate PASS
