# T-760: Zoom LOD Refinement

**Priority:** Medium | **Status:** Open

## Description
Update LOD thresholds per spec. Add settlement name labels in strategic view (LOD 0).

## Changes
- LOD 0 (zoom < 1.5): no agents, 2x2 building blocks, settlement labels
- LOD 1 (1.5-4.0): current behavior
- LOD 2 (>= 4.0): detailed view with names
- Hysteresis: ±0.3

## Done Definition
- [ ] LOD thresholds match spec
- [ ] Settlement labels in strategic view
- [ ] Smooth transitions with hysteresis
- [ ] docs/VISUAL_GUIDE.md updated
- [ ] CHANGELOG.md updated
- [ ] Gate PASS
