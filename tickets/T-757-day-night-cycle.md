# T-757: Day/Night Visual Cycle

**Priority:** Low | **Status:** Open

## Description
World brightness changes based on game hour. Dawn/day/dusk/night color tints.

## Implementation
- 6-18h: Day Color(1,1,1)
- 18-20h: Dusk Color(1.0, 0.85, 0.7)
- 20-6h: Night Color(0.4, 0.4, 0.6)
- 4-6h: Dawn Color(0.7, 0.7, 0.85)
- Apply to WorldRenderer modulate with smooth lerp

## Done Definition
- [ ] Smooth transitions between time periods
- [ ] Visual distinction between day and night
- [ ] docs/VISUAL_GUIDE.md updated
- [ ] CHANGELOG.md updated
- [ ] Gate PASS
