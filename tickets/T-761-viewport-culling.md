# T-761: Viewport Culling & Performance

**Priority:** Medium | **Status:** Open

## Description
Skip drawing entities/buildings outside camera viewport. Target: 200+ entities at 60fps.

## Implementation
- EntityRenderer: calculate visible rect, skip entities outside
- BuildingRenderer: same culling logic
- Minimap: 20-tick refresh (not every frame)
- Stats: 50-tick refresh

## Done Definition
- [ ] Entities outside viewport not drawn
- [ ] Buildings outside viewport not drawn
- [ ] No visual artifacts at viewport edges
- [ ] docs/SYSTEMS.md updated
- [ ] CHANGELOG.md updated
- [ ] Gate PASS
