# ADR 0034 — The colony renderer's data flow: snapshot → resolution → chunks and sprites → tone

Status: accepted (M-ART2, 2026-10-24; drafted in Part 0, final in Part 7 with the measured numbers).

## Context
- The owner adopted, in order: route I whole-figure people (ART-PROD3), one projection with walls, hill faces, water
  and edges drawn by code (ART-STRUCT), the material system with wall tops along the run (ART-POLISH2 option b), face
  variants v1 with matching sprite tints, portraits, items and animals (ART-POLISH, ART-POLISH2).
- The game still drew the M1 16-px code sheet (`assets/v8/tiles.png` from `tools/v8/gen_tiles.py`), one TileMapLayer
  and a set of pooled Node2D layers keyed by sheet names (ADR 0015).
- ADR 0033 forbids content names in renderer code; ADR 0016 C2 keeps pixels out of the simulation.

## Decision
The renderer is a pipeline of four stages, each reading only the previous stage's output:
1. **Snapshot.** Bridge reads only (`SimReads`): terrain, space (walls, doors and floors with their material and the
   blueprint that built them), agents (with `sexes` and the appearance columns `look_ids`, `variants` from the pure
   `collect_appearance`), animals, items, buildings, trees, overlay.
2. **Resolution.** Pure GDScript resolvers turn sim identities into drawables through imported data:
   `material_resolver.gd` (a port of the art line's `resolve.py`; parity tested) for `(style, material)`,
   the asset manifest and `bindings.json` for items, buildings, animals and looks; nothing is named in code.
3. **Chunks and sprites.** The ground (terrain textures, noise-dithered edges, banks, floors) is baked on the CPU per
   16×16-tile chunk into an `ImageTexture` and rebaked only when a tile in the chunk changes; walls, doors and hill faces
   are assembled per chunk into row strips that stand in the y-sorted world layer with people, animals and things
   (depth = base y). People are pooled sprites with a variant tint shader.
4. **Tone.** Ground tone caps at bake, the season at bake, day and night as a global modulate plus hearth lights; LOD
   markers replace people and animals below 1×.
Every tunable lives in `assets/v8art/render_params.json`, reloaded with F12 in debug builds.

## Alternatives considered
- **A full-screen ground shader** (no baking): cheapest at run time, but a headless test cannot read back its pixels, so
  determinism, the edge test and the data-only wall test could not run in the gate.
- **Walls baked into the ground chunks:** fewer nodes, but a wall face must cover a person standing north of it and
  be covered by one standing south; that needs the face in the y-sorted layer.
- **TileMapLayer per terrain with autotile terrains:** square tile edges at every boundary, which ART-STRUCT measured
  and rejected.

## Consequences
- The M1 sheet path (`tools/v8/gen_tiles.py`, `assets/v8/tiles.*`, the sheet palettes in `view.gd`) is deleted.
- Texture memory is bounded by the chunk cache (only chunks near the view are baked).
- People are pooled `Node2D`s with one `ShaderMaterial` each (uniforms set only on change); below 1× they are not
  drawn and one MultiMesh draws every marker (`lod_markers.gd`); chunk and strip textures are ½ or ¼ size below 1×.
- Measured (`docs/v8/m_art2/MART2_DECISIONS.md` D4 update; windowed 1920×1080, vsync off, release library, not
  contended): seed 0 day 20 at 1× 328.6 FPS, ½× 238.2, ¼× 120.65 (budget 120); the 1,000-person world at ½× 186.8 and
  ¼× 173.5 (budget 60); texture memory 265 / 176 / 87 MB at 1× / ½× / ¼× (budget 512); chunk bake median 1.8 ms,
  rebake max 3.7 ms (budget 4), the first bake after a load 17.5 ms once. The ¼× view draws every standing sprite of
  the map (2,252 draw calls) and has the thinnest margin; baking trees into the ¼× chunks is the next step if it misses.
- People at night carry a pale halo away from a fire (D17), the one change to the art line's night transform.

## Date
2026-10-24

## Source
docs/v8/m_art2/MART2_GOAL.md §3.2–§3.5, §4; docs/v8/m_art2/MART2_DECISIONS.md; branch `v8/art-struct` (`b6baa73f`):
docs/v8/artstruct/STRUCT_REPORT.md; branch `v8/art-polish2` (`c8ed8608`): docs/v8/artpolish2/POLISH2_REPORT.md,
art/compositor/walls.py, art/compositor/edges.py, art/materials/resolve.py
