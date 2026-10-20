# Rendering notes for M1 — tile layers and bulk drawing

What V7 measured about drawing many things in Godot 4.6, for the M1 renderer rebuild (new top-down tiles and
sprites). Sources are paths at tag `v7-final-art` unless a commit is named.

## Where the time went

- **The simulation was cheap; drawing was not.** At 2,000 people in one camp the sim's mean tick is ≈ 1.98 ms;
  at 2× speed (60 ticks/s) that is 12 % of one core, ≈ 2 ms of a 16.7 ms frame
  (`docs/revitalize/SCALE.md` L18, L51). At 500 agents `AgentRenderer` alone cost 24.8 ms per frame in release
  (46 ms in debug) and failed every 500-agent frame bar — the cost was per-agent GDScript work (seat
  assignment 16 ms, declump 12 ms, O(n²) geometry), then pose/frame choice, metadata and child nodes
  (`docs/art/production10/REPORT.md` L125–162).
- **Bridge reads multiply per frame.** 15 call sites read `get_agent_snapshot` independently; a shared read layer
  (`SimReads`, commit `a779e927`) took the frame from 10.2 ms to 3.97 ms with 11 FFI calls per frame
  (`docs/art/production9/REPORT.md` L11–13).

So for M1: budget the **renderer**, keep per-agent GDScript loops short, and read each snapshot once per frame.

## Tiles: `TileMapLayer`

- Godot 4.3+ uses one `TileMapLayer` node per layer (the old multi-layer `TileMap` is deprecated). M1's
  `scripts/world/colony/view.gd` builds one `TileSet` + `TileSetAtlasSource` from `assets/v8/temp/tiles.png` and
  `tiles.json` (name → rect) and returns the name → atlas-coords map; `terrain_layer.gd` and `tree_layer.gd` fill
  cells with `set_cell(Vector2i(x, y), 0, coords[name])`. The 192 × 192 terrain is filled once per world (and
  again only while the hearth is unknown); trees refill when `get_colony_versions` reports a new trees version.
- Fill on world reset or when the terrain snapshot changes, not every frame (terrain is cached per world in
  `SimReads`). `set_cell` on an unchanged cell still costs; diff before writing.
- A `TileMapLayer` batches its cells into quadrants; many layers × full-map redraws are what cost. Keep the layer
  count small and fixed.
- Nearest filtering is the project default (`rendering/textures/canvas_textures/default_texture_filter=0`); keep
  1 texel = 1 world pixel and integer camera zoom (M1: 1/2/3/4) so tiles never shimmer
  (`docs/art/production/BRIEF.md` L88, L93).

## Many sprites: three options

| technique | use when | WorldSim history |
|---|---|---|
| pooled `Sprite2D` children of one y-sorted `Node2D` | < a few hundred actors that need feet-based y-sort with trees and buildings | chosen for < 200 villagers and 1,500 trees in `Main/WorldObjects`; shadows moved to pooled `Sprite2D` too (`docs/art/production/DECISIONS.md` L16; `docs/art/production10/GOAL.md` L50) |
| `MultiMeshInstance2D` | thousands of identical-mesh instances; one draw call; per-instance transform/colour/custom data | V7 agent renderer (`shaders/palette_swap.gdshader` reads `INSTANCE_CUSTOM`); y-sort across instances is not automatic — kept "for a later 10k renderer" |
| one `_draw()` per layer (`draw_texture_rect_region`, `draw_rect`) | overlays and markers that change rarely | many V7 overlay renderers; call `queue_redraw()` only when the data changed |

Rules that carry over:
- Pool nodes; never create and free a `Sprite2D` per agent per frame.
- Hide with `visible = false` on pooled nodes instead of freeing them; the renderer, not capture tools, decides
  visibility (`SKILL.md` lesson 2026-10-11 "Captures showed villagers absent from the snapshot").
- Visual variation (body, palette, jitter) from integer hashes of stable ids or tile coordinates, never `randf()`
  (`v7-final-art:docs/art/production/BRIEF.md` L119).
- Per-frame animation follows frame delta, not simulation ticks (`docs/art/production10/DECISIONS.md` L28).
- Measure every "optimisation": GDScript caches of `get_meta`/`get_node_or_null` results made W500 19.5 %
  slower than the native calls (`docs/art/production10/DECISIONS.md` L24).
- Pure geometry that is hot can move to Rust behind an exact GDScript fallback; Godot's `Vector2`/`Rect2` are
  f32 while GDScript `float` is f64, so a Rust port must reproduce the operation order bit for bit
  (`docs/art/production12/REPORT.md` L22; commit `37be1b85`).

## Shaders (canvas_item)

- In Godot 4.6 `canvas_item`, `COLOR` at the start of `fragment()` is the vertex colour **already multiplied by
  the sampled `TEXTURE`**. Capture a per-instance tint in `vertex()` into a `varying` (commit `2f4404465`,
  `shaders/palette_swap.gdshader` header).
- `filter_nearest` on palette/LUT samplers; `varying flat` for per-instance integer data.

## Measuring a frame (only for the perf regression check)

Headless timings are dummy-driver numbers and never display performance (`docs/art/production13/CLI_HEADLESS.md`
L15–21). Debug-dylib numbers are not performance either (W500 51 ms debug vs 29 ms release,
`docs/art/production11/REPORT.md` L17). Measure release, windowed, on a quiet machine (see `worldsim-testing`
and the Mac-contention lesson in `worldsim-pitfalls`), and compare A/B pairs within one session.
