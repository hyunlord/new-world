# T-460: Resource Overlay Enhancement

## Action: DISPATCH (Codex)
## Files: scripts/ui/world_renderer.gd

### Changes to update_resource_overlay():
Increase color saturation and alpha for better visibility:
- Food > 2.0: Color(1.0, 0.9, 0.1, clampf(food / 8.0, 0.3, 0.6)) — bright yellow
- Stone > 2.0: Color(0.5, 0.7, 1.0, clampf(stone / 8.0, 0.3, 0.5)) — sky blue (NOT gray)
- Wood > 3.0: Color(0.0, 0.7, 0.3, clampf(wood / 10.0, 0.2, 0.5)) — emerald green

### Priority order: food > stone > wood (unchanged)
### Keep the existing Image-based texture approach
