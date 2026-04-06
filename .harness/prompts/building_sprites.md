# A2: Building Sprites — Renderer Update

## Summary
Replace geometric draw calls (draw_rect, draw_circle, draw_colored_polygon) in building_renderer.gd with draw_texture_rect using pre-generated 32x32 PNG sprites. Preserve original geometric drawing as _*_fallback functions.

## Changes
- File: scripts/ui/renderers/building_renderer.gd
- Add _building_textures cache Dictionary
- Add _load_building_texture() for runtime PNG loading with caching
- Add _draw_building_sprite() that draws texture or falls back to geometric shapes
- Rename _draw_campfire/_draw_shelter/_draw_stockpile to _*_fallback
- Remove dead code block (match after continue on line 114)
- No simulation logic changes. No Rust changes. No new localization keys.

## Verification
- Sprite PNGs already committed (campfire.png, shelter.png, stockpile.png)
- Z3+ strategic dots unchanged
- Z1 interior overlay unchanged
- Construction progress bar unchanged
- Fallback to geometric shapes if PNG missing
