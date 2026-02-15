# T-810: Right Sidebar Layout Fix [Critical]

## Problem
Stats panel overlaps minimap.

## Changes
1. **stats_panel.gd**: Fix positioning to be strictly below minimap with 10px gap
   - Minimap: y=38..198 (top_bar 28 + margin 10 + 160)
   - Stats: y=208..408 (198 + 10 gap)
2. **stats_panel.gd**: Add numeric values to mini view
   - Population number, resource numbers, job counts
3. **stats_panel.gd**: Enable mouse input for click-to-detail

## Files
- scripts/ui/stats_panel.gd

## Done
- [ ] No overlap between minimap and stats
- [ ] Numeric values visible in mini stats
- [ ] docs/VISUAL_GUIDE.md updated
- [ ] CHANGELOG.md updated
