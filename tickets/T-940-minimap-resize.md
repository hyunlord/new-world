# T-940: Minimap Size + Toggle Cycle

## Status: TODO

## Description
Minimap default 200px. M key cycles: 200→300→hidden→200.
Stats panel moves to bottom-right.

## Files
- minimap_panel.gd: MINIMAP_SIZE 160→200, resize support
- stats_panel.gd: reposition to bottom-right
- hud.gd: toggle_minimap cycle logic

## Done
- [ ] Minimap 200px default
- [ ] M cycles 200/300/hidden
- [ ] Stats panel bottom-right
- [ ] No overlap
- [ ] docs/ updated
