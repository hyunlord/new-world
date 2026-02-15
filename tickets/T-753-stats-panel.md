# T-753: Stats Panel

**Priority:** Critical | **Status:** Open | **Depends:** T-750

## Description
160x200px stats panel below minimap. Population graph, resource graph (3 colored lines), job distribution bar. G key toggle.

## Implementation
- New file: `scripts/ui/stats_panel.gd` (extends Control)
- Uses StatsRecorder.history for data
- Population: green polyline
- Resources: yellow (food), brown (wood), gray (stone)
- Job bar: horizontal stacked bar with job colors
- G key toggle in main.gd

## Done Definition
- [ ] Population graph draws correctly
- [ ] Resource graph with 3 colored lines
- [ ] Job distribution bar
- [ ] G key toggles visibility
- [ ] docs/VISUAL_GUIDE.md, CONTROLS.md updated
- [ ] CHANGELOG.md updated
- [ ] Gate PASS
