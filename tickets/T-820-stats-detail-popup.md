# T-820: Stats Detail Popup [Critical]

## Problem
160px stats panel too small to read graphs/data.

## Changes
1. **New: scripts/ui/stats_detail_panel.gd**: Large centered stats popup
   - 70-80% of screen size
   - Dim overlay background
   - Sections: Population, Resources, Jobs, Settlements
   - Population: graph + current/peak/deaths/birth rate/death rate
   - Resources: graph + current values + delta per 100 ticks
   - Jobs: distribution bar + counts + percentages
   - Settlements: per-settlement comparison (pop, buildings, resources)
   - Close with G, Esc, or X button
2. **stats_recorder.gd**: Track additional data
   - peak_pop, total_deaths, total_births
   - Per-settlement stats
3. **hud.gd**: Wire G key → open detail popup (pause sim)
   - Mini stats click also opens detail popup
4. **main.gd**: G key now opens detail popup instead of toggle

## Files
- scripts/ui/stats_detail_panel.gd (NEW)
- scripts/systems/stats_recorder.gd
- scripts/ui/hud.gd
- scenes/main/main.gd

## Done
- [ ] G key opens large stats popup
- [ ] Simulation pauses when popup open
- [ ] All data sections display correctly
- [ ] docs/CONTROLS.md, VISUAL_GUIDE.md, SYSTEMS.md updated
- [ ] CHANGELOG.md updated
