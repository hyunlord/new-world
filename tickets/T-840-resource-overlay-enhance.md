# T-840: Resource Overlay Strengthening [Medium]

## Problem
Resource overlay colors too faint against biome background.

## Changes
1. **world_renderer.gd**: Increase alpha values
   - food: alpha 0.3-0.6 → 0.45-0.7
   - stone: alpha 0.3-0.5 → 0.4-0.65
   - wood: alpha 0.2-0.5 → 0.35-0.6
   - Stronger base colors
2. **entity_renderer.gd**: Draw F/W/S text at zoom >= 4.0
   - When resource overlay visible + LOD 2
   - Small 8px letters on resource tiles

## Files
- scripts/ui/world_renderer.gd
- scripts/ui/entity_renderer.gd

## Done
- [ ] Resource overlay clearly visible against all biomes
- [ ] F/W/S text markers at high zoom
- [ ] docs/VISUAL_GUIDE.md updated
- [ ] CHANGELOG.md updated
