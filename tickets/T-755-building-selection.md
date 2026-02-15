# T-755: Building Selection Panel

**Priority:** Medium | **Status:** Open

## Description
Click building → show info panel. Building info varies by type (stockpile storage, shelter capacity, campfire range).

## Implementation
- Modify entity_renderer.gd click handler to check buildings
- Add building_selected signal to SimulationBus
- New building panel in hud.gd (reuses entity panel position)
- Type-specific info display

## Done Definition
- [ ] Buildings clickable
- [ ] Stockpile shows storage amounts
- [ ] Shelter shows type info
- [ ] Campfire shows range/bonus info
- [ ] docs/VISUAL_GUIDE.md, CONTROLS.md updated
- [ ] CHANGELOG.md updated
- [ ] Gate PASS
