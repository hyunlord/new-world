# Building Renderer Visual Improvements Verification

## Feature Summary
Three visual enhancements to `scripts/ui/renderers/building_renderer.gd`:

1. **Floor tile visibility**: Alpha increased from 0.35 to 0.55, added 0.5px border outline
2. **Furniture icon size + stockpile label**: Icon size 0.6→0.7, storage_pit furniture gets localized "비축소" label via `Locale.ltr("BUILDING_TYPE_STOCKPILE")`
3. **Wall autotile**: New `_draw_wall_tile()` function connects adjacent walls with 2px bridge rects (right + down directions)

## Files Changed
- `scripts/ui/renderers/building_renderer.gd`

## FFI Verifier Checks
- `SimBridge.get_tile_grid_walls()` returns Dictionary with keys: `floor_x`, `floor_y`, `wall_x`, `wall_y`, `wall_material`, `furniture_x`, `furniture_y`, `furniture_id`
- Floor arrays are non-empty (stockpile/campfire stamps floors)
- Wall arrays are non-empty (shelter builds walls)
- Furniture arrays contain `storage_pit` entries

## Visual Verify Checks
- **Floor tiles visible**: Brown-tinted floor rectangles visible under buildings at Z1-Z2 zoom (alpha 0.55, not invisible)
- **Wall tiles visible**: Stone/wood colored wall blocks rendered around shelters
- **Wall autotile**: Adjacent wall tiles appear connected (no visible gap between neighboring walls)
- **Furniture icons**: Emoji icons (📦🔥🛏⚒) rendered inside buildings at close zoom
- **Stockpile label**: "비축소" (or localized equivalent) text visible below storage_pit furniture icon
- **No rendering artifacts**: No overlapping draw calls, no z-fighting between floor/wall/furniture layers

## Test Scenario
1. Start simulation with seed 42
2. Run 500+ ticks (enough for buildings to be constructed)
3. Zoom to Z1 (close) on a settlement with stockpile + shelter
4. Verify floor/wall/furniture rendering at Z1 and Z2 zoom levels
5. Zoom out to Z3+ and verify walls still render (inset=0)
