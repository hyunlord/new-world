# W1 renderer contract (read-only audit)

Inspected `origin/v8/main` = `4a44336e7782f2b27615d942b5cd54dbfee89406` on 2026-11-11. No runtime files edited. This pin matters: a planner may have newer local changes than the remote ref inspected here.

## Current wall API and geometry

- `scripts/world/art/wall_baker.gd`: `SIZES` requires face 256×44, top_ew 256×64, top_ns 64×256, end 24×44, junction 64×64, door_closed/open 64×60. `prepare_set()` Lab-tints every role, tones/blur only tops and junction; cache keys cover those seven source paths and tint/top value/saturation/blur.
- `_wall_tile()` currently computes four booleans from global `occupancy`: north `(0,-1)`, east `(1,0)`, south `(0,1)`, west `(-1,0)`. **There is no numeric neighbour-mask API or 16-piece atlas lookup.** Pure horizontal means `(e or w) and not (n or s)` and samples top_ew at world-x phase; pure vertical means `(n or s) and not (e or w)` and samples independent top_ns at world-y phase. Every corner/T/cross/isolated cell uses the SAME junction image.
- Top image width is always 64. Top depth is 64 when south connects, otherwise 16. Exposed top perimeter receives rectangular 2px ink. Face occupies strip rows 16..59 (44 px) only when no occupied south neighbour; left/right face discontinuities get the same 24×44 end texture.
- Strip is 64px high, placed at world `(chunk_x*1024, tile_y*64-16)`, with y-sort base `(tile_y*64+43)`. Face base is world y+43; cap top is y-16. Final four strip rows allow a small below-base margin. Ground and hill assembly are independent; hill faces must retain existing behavior.
- Schema `assets/v8art/materials/construction_styles.schema.json` accepts `top.width_px` 1..64 and `top.side_shadow` 0..1. **This Godot baker does not consume either field.** It also does not use per-style ink_edge_px for drawing: global render_params walls ink config supplies it. Python art assembly supports narrower tops (ADDING_A_STYLE), but merely setting width_px in this runtime does not make a narrow wall.
- Current seven-texture schema rejects unknown fields (`additionalProperties:false` in ConstructionStyle and TextureSet). It cannot register separate corner/T/cross/end pieces or oriented door pairs. Supplying atlas pieces as today's `junction` cannot solve this.
- `ground_layer.gd::_wall_cells()` reads `get_space` wall_xs/ys, wall_styles, wall_materials (signed packed value converted to unsigned), wall_categories, wall_kinds, wall_is_door. Occupancy includes all materials and doors, and must keep doing so. `cell_set` only covers non-door cells. Crisp ground uses whole occupied tile; thin transparent wall imagery may therefore expose the existing ground choice rather than neighboring terrain.

## Doors

- `ground_layer.gd::_update_doors()` creates separate 64×60 sprites at `(tile_x*64,tile_y*64-16)` with base y+43, outside the baked strip. A door opens only while any agent snapshot tile exactly matches the door tile; not hover/proximity and not persistent state.
- `_door_texture()` resolves kind `door`, not wall, chooses only door_open/door_closed, and caches by style/material/category/open. **No orientation field or adjacency selection exists.** A wall style's door textures alone will not replace the default `Door` binding's door style.
- `scripts/world/art/site_ghost.gd` uses the same wall baker via `wall_rows()`. Its `door_image(style,open)` also has no orientation. `ring_tiles()` takes explicit size/door coordinates from bindings and `ring_image()` builds the walls/floors/door. Any door-orientation change must cover these previews and finished legacy hut rings too.

## Roofs

- `scripts/world/colony/building_layer.gd::_sync_template_sites()` accepts actual persisted connected, roofed sites, skips cancelled and completed/non-intact sites. Reads already-rotated width/height and top-left x/y; calls `PlaceholderArt.building_texture(row, footprint*64)` where PlaceholderArt is `scripts/ui/item_icons.gd`.
- At this ref, `item_icons.gd::building_texture()` still generates a solid round or angular placeholder polygon plus a glyph; it has no generated-roof registry. `_sync_template_sites()` uses site footprint base and offset `(-width*32,-height*64+64/3)`. Construction is the same roof polygon at `sites.wash_alpha` (default .25), **not a timber frame**. No person-inside/hover fade exists in the roof path. A separately generated cutaway is thus conditional integration work, not already-supported data.
- `buildings.ron` footprints: paleolithic `hut` 3×3; neolithic `log_house`, `stone_house`, `workshop_shed`, `storehouse`, `drying_shed`, `granary`, `smokehouse`, `oven_house` all 4×4. `log_house` and `stone_house` use art roof Round; material tint differs. Requested 5×4 example is an assembly demonstration, not a currently authored template footprint.

## Exact proposed integration (Claude Code, not applied here)

Prefer a generic capability, not a style-id branch (ADR0033): add optional versioned `wall_pieces` metadata to construction styles, with `canvas`, `anchor`, `pieces` mapping string masks 0..15 to image paths, `damaged` map for mask5/mask10, and `doors` pairs keyed ew/ns. Preserve existing seven-texture path as fallback. Extend schema/parser/loader and resolver parity in BOTH art Python and runtime before registering these rows; they are **draft extension rows**, not schema-v1-ready entries. Strictly validate 16 masks, images, anchors, canonical paths and dimensions.

Proposed numeric mask: `m = (1 if n else 0) | (2 if e else 0) | (4 if s else 0) | (8 if w else 0)` from **whole occupancy including doors and other materials**. This is a NEW contract, not a claim about existing code.

| Mask | Connections | Piece |
|---:|---|---|
|0|none|isolated|
|1|N|end_n|
|2|E|end_e|
|3|NE|corner_ne|
|4|S|end_s|
|5|NS|straight_ns|
|6|ES|corner_se|
|7|NES|t_nes|
|8|W|end_w|
|9|NW|corner_nw|
|10|EW|straight_ew|
|11|NEW|t_new|
|12|SW|corner_sw|
|13|NSW|t_nsw|
|14|ESW|t_esw|
|15|NESW|cross|

1. `wall_baker.gd::prepare_set`, `_preparation_key`, `prepare_cached`: load/tint the declared mask pieces and cache every path/geometry input; do not blur the interlocking ink drawing as if it were a raw top texture. `_wall_tile`: for a prepared mask set, select exactly m and composite the complete piece at its declared anchor; bypass old rectangular ink/end/face additions (those would widen/overwrite the authored narrow joint). Keep old branch for legacy texture sets and `_hill_tile` unchanged. If sprite pixels extend beyond existing 64×64 strip, expand strip bounds/offset and chunk apron OR draw each piece as a y-sorted sprite. Do not silently crop protruding interlocking ends. The asset canvas/anchor determines required padding.
2. `ground_layer.gd::_bake_chunk` and `site_ghost.gd::wall_rows/wall_tile/ring_image`: carry any expanded image origin/anchor; retain physical base y and y/x sort. Update dirty-neighbor/chunk invalidation and side-shadow ground exposure accordingly. A full authored piece should include contact shadow without overlaying a second rectangular shadow. Damage selection requires authoritative damage state; present bridge arrays do not supply it here, so damaged assets remain unbound until such state exists.
3. `ground_layer.gd::_door_texture/_update_doors`, `site_ghost.gd::door_image/ring_image`: choose orientation from occupancy. For unambiguous straight door, EW when both east+west connected and NS when north+south connected. At ambiguous corner/cross/isolated doors require explicit orientation metadata (with documented legacy EW fallback), never rotate the EW drawing. Include orientation in cache keys. Align using declared source anchor; retain agent-on-door opening rule unless separately changed.
4. `building_layer.gd::_sync_template_sites`: resolve an optional roof asset capability from data by template id/footprint/state and place via its declared anchor at existing physical footprint base; complete → built, building → actual frame. Fallback to existing placeholder only if no roof binding. `template_preview_texture` should use same resolver. Register/warm imported roof textures through existing asset resource warmup. Optional fade requires inside-footprint agent or pointer test and modulation alpha .35; do not call this feature available until implemented. An image with baked .35 alpha is unnecessary if modulation is supported.
5. Tests: extend `mart2_ground_test.gd` beyond existing data_only_wall fixture (only resolution and broad strip opacity) with a new data-only mask capability fixture, every mask, no missing arms, thin widths/transparent exterior, both door orientations/open apertures, material-crossing connectivity, chunk boundary joins, canonical anchor/base bounds, and legacy/hill pixel identity. Add site-ghost parity; real in-game 5×4 hut/T layout screenshot. Roof tests cover 3×3/4×4 footprints, rotated bounds, state transition/frame, optional fade and texture warmup. Do not reinterpret existing broad opaque-coverage expectation as a valid thin-wall assertion.

## Audit limits

Static source audit only, no Godot import/play or renderer execution. No claim that proposed capability already works or that art can import by schema-v1 data alone. No production/runtime edits.
