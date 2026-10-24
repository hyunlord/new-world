# M-ART2 decisions

Numbered decisions of the M-ART2 pass (`docs/v8/m_art2/MART2_GOAL.md`). Each: context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/mart2-data/` (`$DATA`), outside the repository. The goal says "never ask the
owner": every open question is decided here.

## D1 — Machine, base, art source and time

- Base: `origin/v8/main` = `origin/v8/wip` = `29f720aa` (checked with `git ls-remote` at the start). The local branch
  ref `v8/main` is not used; `v8/main` moves on the remote by explicit refspec (`git push origin <sha>:refs/heads/v8/main`).
- Worktrees: `../new-world-mart2-base` detached at `29f720aa` (the old renderer for the comparison shots and the base
  probe); `../new-world-art-src` detached at `c8ed8608` (`v8/art-polish2`), read only, deleted at the end. No art
  branch is merged, pushed to or checked out in this clone.
- Start 2026-10-24 09:38 KST (`$DATA/session.txt`); target 20 h, hard stop 2026-10-25 09:38 KST (then finish the
  step, push, go to Part 7). An owned `caffeinate -i -d -u` runs for the session (pid in `$DATA/caffeinate.pid`).
- The tree has the owner's local `.gitignore` change and the untracked `.ignore`, `docs/revitalize/`; none is staged.

## D2 — What the owner adopted, and from which art commit

All from `v8/art-polish2` at `c8ed8608`, which contains the whole art lineage (route I, ENV1, STRUCT, POLISH, POLISH2).
The art reports themselves say "owner verdict pending"; the adoption is the owner's, recorded in the goal's "Why"
(2026-10-24) and §2 "Owner choices fixed for this pass":
- **People: route I** (ART-PROD3, `159ef0f9`, 34/36 looks; two `adult_M_*_long_tied` looks fall back to the bun of the
  same age, sex and top — `docs/v8/artprod3/HANDOVER.md`). Owner/planner decision D4 of 2026-10-23.
- **One projection** (ART-STRUCT, `b6baa73f`): ground top-down, everything standing in the same front 3/4, walls, hill
  faces, water and edges by code. Owner message 2026-10-24: "ㅇㅇ ㅇㅋ".
- **Material system** (ART-POLISH2): `construction_styles.json`, `material_looks.json`, `category_defaults.json`,
  textures for log, drystone, mud brick, fired brick, packed earth, two door styles. **Wall top option (b)** (full
  width, along the run), with `blur_px` 0–1 and `top.value ≈ 0.80` as starting values (sharper than rendered).
- **Face variants v1** (POLISH2): six per look with matching sprite tints; the skin mask must be fixed so it never
  tints clothing (POLISH2's S3 variants darkened fur cloaks).
- **Portraits** (POLISH2, 128 px, one per look × variant), **items** (POLISH handoff list), **deer and wolf** (POLISH
  `try2` sheets, the 35° S facing rule). Roof not drawn; N facing = S; elders = adult look + an older face variant + a
  grey hair tint.

## D3 — M5.5 owner decisions, recorded for M-EXT Part 0

Received with this goal (§5.4); implemented in M-EXT Part 0, not here (this pass changes no simulation behaviour):
1. The population cap also applies at acceptance (a wave that would exceed it cannot be accepted whole).
2. Trader goods include items of techniques with ≤ 2 practitioners in the colony and rare next-era artifacts.
3. Meat goes beside the hearth when no stockpile has room (instead of rotting where it lies).
4. F4 and F9 bars are rewritten with their arithmetic in M-EXT's evaluation.
5. The migrant decline cost stays −5.

## D4 — Performance plan (goal §4), written before coding

Budgets (Mac, Godot 4.6, windowed 1920×1080, vsync off, release library copied to the debug path with its SHA
checked and restored after): seed 0 colony at day 20 (≈ 30 people) at 1×, ½×, ¼×: **≥ 120 FPS**; the 1,000-person
world (`reset_world_stress(seed, 1000)`, the bridge's stress world, from a debug scene) at ½× and ¼× with LOD:
**≥ 60 FPS**; chunk rebake ≤ 4 ms per chunk (compose + upload, measured in the game); texture memory ≤ 512 MB
(`Performance.RENDER_TEXTURE_MEM_USED` and `RENDER_VIDEO_MEM_USED`); simulation tick cost unchanged (the bench gate
step). Each FPS run: 30 s warm-up, 60 s measured, frames counted by `Engine.get_frames_drawn()`, the load average
recorded before and after; a run under 1-minute load ≥ 8 is labelled contended and repeated.

Design choices made for these budgets (measured before coding, `$DATA/notes/image_bench.txt`: 768 native `Image`
blit/blend ops on 64×64 tiles = 8.4 ms, about 11 µs each):
- **Ground chunks: 16×16 tiles = 1,024 × 1,024 px RGBA8 (4 MB).** A whole 192 × 192 map would be 144 chunks = 576 MB,
  over the memory budget, so chunks are baked **lazily** for the view plus a one-chunk margin and kept in an LRU cache
  (`ground.chunk_cache_max` in the params, default 48 = 192 MB).
- **Composition uses native `Image` ops only** (`blit_rect`, `blend_rect_mask`): contiguous tiles of one ground in a
  row are one blit; edges blend with precomputed binary dither masks (see D5). A per-pixel GDScript loop costs
  ~25 µs per 4,096 bytes and would put one chunk at ≥ 100 ms.
- **Walls, doors and hill faces** are assembled per chunk into one strip image per tile row (1,024 × 64 + 16 px
  above), each strip a `Sprite2D` at the row's base y in the y-sorted layer.
- **People**: pooled `Sprite2D` with one `ShaderMaterial` per look (the variant colours are instance uniforms), so the
  200-person lab result (ART-PROD2 P1: 524.8 FPS) is the reference; at ½× and ¼× people become markers drawn by one
  node (`_draw` of circles), which is the 1,000-person path.
- If an FPS budget misses: (1) atlas the frames per look, (2) MultiMesh for the LOD markers, (3) bake trees into the
  ground chunks at ¼× (goal §4), in that order, each measured.

## D5 — Ground: dithered edges, texture period, tone at load, water through the chunk alpha

- **Noise-dithered binary blend.** `edges.py` blends with a continuous weight (12 px, half strength between grounds;
  10 px full strength into water). Godot's `blend_rect_mask` takes a binary mask (a pixel blends when the mask's alpha
  is non-zero), so the weight becomes a dither: a pixel takes the neighbour's texture when
  `weight × strength > dither(x, y)`, with `dither` the same fixed-seed world noise (`edges.py`'s `_noise`, seed 5005,
  integer hash) — "noise-dithered" literally. Masks depend on the tile's position only through the noise phase; the
  noise is periodic in 256 px (4 × 4 tiles), so masks are cached per (side, phase): 4 sides × 16 phases per width.
- **Texture period 128 px.** The ENV1/STRUCT boards drew each 512-px ground texture shrunk to one 64-px tile, which
  shows a tile grid on a large map (STRUCT's own note: "the wide cap repetition"); 1:1 at 512 px is far busier than
  the adopted calm ground. The import keeps the 512-px sources; the renderer resamples them once to
  `ground.texture_period_px` (default 128, tunable) and samples with world offsets.
- **Tone cap at texture load**, per ground texture (the `tone.py` contrast affine and chroma factor, the ENV1 locked
  targets mean luminance ≤ 0.32, std ≤ 0.25 × the person reference, saturation ≤ 0.22); a baked chunk is tested
  against the same caps. Seasons (winter desaturation, the snow overlay, drought) are applied to the texture set, and
  a season change rebakes the cached chunks.
- **Water through the chunk alpha.** The goal names "a shader on a water TileMapLayer"; a TileMapLayer draws whole
  64-px cells, so its edge against the bank is a square tile edge — exactly what ART-STRUCT rejected. Instead the
  chunk's alpha is 0 on water and the 10-px dithered bank falloff, and the chunk's own shader draws the seamless water
  texture with world offsets and a 0.05 Hz UV drift beneath it. Same parts (shader, world offset, drift, code bank
  mask, 2-px ink on the land side), no square edge. Fords: water plus the four stepping-stone sprites placed by code
  (`edges.py` `ford_overlay` positions).

## D6 — Material ids across the bridge

`MaterialId` is the djb2 hash of the material key (u32; `sim-core/src/material/id.rs`), not an index. The goal asks
for `wall_materials` as `PackedInt32Array` with `-1` for none: the column carries the u32 bit pattern as i32
(GDScript reads `v & 0xFFFFFFFF`), and a test asserts that no registry material hashes to `0xFFFFFFFF`, so `-1` is
unambiguous. `wall_styles` carries the blueprint name that built the tile (`LogWall`, `StoneWall`, `Door`, matching
`construction_styles.json`'s `sim_blueprints`), and `wall_categories` / `floor_categories` carry the material's
registry category name, because the art loader contract (`ADDING_A_STYLE.md`, "M-ART2 loading contract") requires the
category for a material the art data does not list.

## D7 — The import (Part 1)

- **Read from git objects, not the worktree.** `tools/v8/art_import.py` reads every file with `git cat-file --batch`
  at the pinned commit, so a re-import needs only the commit (the art worktree is deleted at the end); bytes are
  copied unchanged (PNG stays PNG). 664 files, 13.9 MB of the 40 MB budget (`--verify` is gate step `art_manifest`;
  9 unit tests incl. negative cases and the look-grid = `appearance.ron` check).
- **What "adopted" means per family** (D2): people = the 34 route I looks × S/E × 5 poses (340 frames; the art line's
  2,040 baked variant frames are not imported — the game tints by shader, D9); portraits = POLISH2's 204 (look ×
  variant, 128 px); items = the POLISH handoff list (`docs/v8/artpolish/HANDOFF_PATHS.md`, 22 items + ENV1's spoiled
  meat) with the ENV1 anchors; structures and trees = the STRUCT set (`docs/v8/artstruct/assets.json` anchors and
  footprints; door sprites are not imported — doors come from the material system); berry bushes = the ENV ground
  catalog (the only bush art); ground = the six ENV ground textures, STRUCT's water, ENV1's snow overlay, STRUCT's
  rock face and ford stones; animals = POLISH `try2` wolf and deer (anchors from `animal_manifest.json`); data =
  the three material JSON and their schemas, `FACE_VARIANTS.json`, `variant_transforms.json`, the compositor's
  `top_colours.json` (LOD marker colours).
- **Not imported:** a contact shadow (no art asset exists; HANDOVER says draw a 28 × 8 ellipse — code-drawn,
  `people.shadow_*` in the params), the 3.7 MB person anchor (only its measured luminance std, 0.170613, the value the
  art line's `tone.person_limit` returns, is recorded in the manifest as the tone reference), the art line's masks
  (rebuilt and fixed by `tools/v8/art_masks.py` from the commit, Part 4), archives, diagnostics, pipelines.
- **Hand-written data next to the import:** `bindings.json` (sim codes → art ids, family fallbacks; ADR 0033: the
  renderer names no content) and `render_params.json` (every tunable; F12 reload).

## D8 — Bridge reads and appearance (Part 2; the Part 2 worker's decisions, checked by the lead)

1. Categories need their own lookup: the game's worlds start with an empty `MaterialRegistry`, so the bridge asks
   the live registry first, then an embedded copy of every `sim-core/data/material/**/*.ron` (built by
   `sim-bridge/build.rs` from the directory, so a new material file needs no code; a test checks the count).
2. `packed_earth` has no material file, so its category is `""`; the renderer falls back by style (no invented
   category).
3. Style names are the blueprint's own name in PascalCase (`log_wall` → `LogWall`), equal to the art data's
   `sim_blueprints`; a dug tile has no blueprint and is `DugFloor` (documented constant); `wall_styles` uses the same
   classification as `wall_kinds` (tested).
4. Elder = snapshot life stage 2 (the art line used "age ≥ 50", which the snapshot does not carry); elders keep the
   adult look and take variant 3 or 4 (the art line's v4, v5).
5. A person without a `Sex` component counts as male; the hair hash uses its own multiplier and the high half of the
   product, independent of the variant hash.
6. A broken `appearance.ron` is refused without a panic: looks and variants go to 0, sexes stay correct.
7. No `#[func]` was added (the bridge API list is unchanged at 97 methods, `bridge_api_m5_5.txt`); "bridge API list
   v7" in the goal is covered by the snapshot schema's version 7.
8. M-EXT debt found: `built_at` counts only granite and limestone as stone, so a stone wall of another stone would be
   classed `LogWall` (simulation code, left as is).
- Evidence: identity against `/Users/rexxa/work/m55-data/base/identity` — summary PASS (228,748 values, 0
  differences), chronicle PASS (64 files), raw samples PASS, determinism PASS; fmt, clippy (all targets), the new tests
  (engine_appearance_ffi 5, engine_space_ffi 4), sim-bridge 51, schema docs, check_docs all green
  (`$DATA/part2_report.md`; each new test was seen failing under a deliberate break).

## D9 — Variant tint: the art line's Lab shift in the shader, not RGB multipliers

- The goal says "the variant's Lab-derived target colours precomputed to RGB multipliers". The approved sprites are the
  art line's baked frames, made by a per-pixel CIELAB shift (`variant_transforms.json`: one `skin_delta_lab` and one
  `hair_delta_lab` per look × variant, L* ≤ 18 and alpha ≤ 127 protected). A single RGB multiplier per region cannot
  reproduce a Lab shift across the region's light and dark pixels (it scales shadows and highlights by the same
  factor, shifting hue where the Lab shift does not), so the shader (`shaders/person_variant.gdshader`) does the same
  sRGB → Lab → shift → sRGB per masked pixel with the record's deltas — the "precomputed" part is the deltas. Cost: a
  few dozen ALU ops per masked pixel of ~30–200 people at 1× (bodies hide below 1×). The numbers come straight from the
  imported record (tested: `mart2_people_test.gd` `variant_tint_deterministic` compares every person's shader
  parameters with the record read independently).
- Elders (life stage 2) get the grey hair target `bindings.people.grey_hair_rgb` (the art line's H4, 160/155/140) as
  a Lab delta from the look's source hair Lab, on top of their older face variant (goal §2: "an older face variant +
  grey hair tint").
- A frame without a fixed mask is drawn untinted (`has_mask` false); a missing mask texture must never tint the whole
  figure (the first windowed run did exactly that: a sampler without a texture reads white).

## D10 — Walls, doors and hill faces in the y-sorted world; people as feet-anchored nodes

- Each chunk's walls and hill faces are baked into one strip per tile row (1,024 × 64: the 16-px cap above the row,
  the 44-px face, a joined N–S top reaching 48 px down) and stand in `ColonyView/World` (y-sorted) at the row's base
  y (`tile_top + 43`), so a person north of a wall is covered by its face and cap and one south of it covers the wall
  (PROJECTION.md's rule). Doors stand there as their own sprites (open while a person is on the tile).
- Layer nodes keep their old paths (`ColonyView/AgentLayer` …; UI scripts reference them) and parent their sprites
  to `ColonyView/World`; one y-sort scope holds walls, people, animals, trees, structures and items, all at z 0.
- A person node sits at its feet (the base point); the frame is offset by −anchor (mirrored for W), so the anchor
  pixel lands on the feet in every facing (tested in four facings).

## D11 — The edge instrument and a check that can fail

ART-STRUCT's registered instrument (no run of more than 24 identical non-zero luminance steps across a boundary)
passes square edges between two textured grounds — the steps vary pixel by pixel. Shown on the real hearth chunk:
with `blend_px` 0 it read 1 px (pass) while the edges were square. `mart2_ground_test.gd` keeps the instrument (goal
§8) and adds `edges_dithered`: in a tile whose east neighbour is another ground, 8–60 % of its 12-px band must differ
from its own texture (taken from the neighbour) and its far side none; it read 0.26 / 0.00 and failed (0.00) with
blending off.

## D12 — Space changes rebake incrementally

The first version recomputed all 36,864 tiles of the ground map on every `get_space` version change (75 ms, a hitch
at every wall finished or tile dug). `_apply_space` now recomputes only the tiles whose dug, floor or wall state
changed (and the tile north of each, whose hill face depends on it): 1.2 ms for a dig including the one chunk rebake
(`dig_rebakes_one_chunk`: exactly the dug tile's chunk is rebaked, 1.1–1.3 ms headless).

## D13 — The owner screen set, the old renderer, and where images live

- Scenes come from colony seed 0 saves written by the probe with `tools/v8/m55_u_script.json` (the M5.5 script: a
  house, a storehouse, a dug pocket, workshops, drafting, migrants): day 6 noon (tick 7,440), day 6 02:00 (6,840),
  first winter noon (26,160; the colony season is 6 days), and day 20 noon (27,600) for the FPS runs. Every shot also
  saves the world at its exact moment; `tools/v8/mart2_old_shots.gd`, run in the base worktree (`29f720aa`, the M1
  sheet renderer), loads those saves and frames the same world area (old zoom = new zoom × 4) — the comparison is the
  identical moment, the drafting and all (saves are compatible: no simulation change).
- `check_docs.sh` allows no binary under `docs/` except ≤ 12 images in `docs/images/`, while the goal names
  `docs/v8/m_art2/screens/`. The PNGs therefore go to `docs/images/` as two sheets — `mart2_contact_sheet.png` (the new
  renderer set, replacing `m55_contact_sheet.png`) and `mart2_compare_sheet.png` (old vs new per scene) — and
  `docs/v8/m_art2/screens/SCREENS.md` lists every shot (caption, tick, zoom, SHA-256 of the raw PNG kept in
  `$DATA/screens/`).

## D14 — The skin mask fix: what it found and what it does

- `tools/v8/art_masks.py` rebuilds one RGBA mask per frame (R skin, G hair; 340 files, 136 KB) from the art line's
  last masks at `c8ed8608` (read through git) and the source-authored geometry (`*_SOURCE_ENVELOPES.json`,
  `variant_anatomy.json`). A skin pixel is removed if it lies in a garment or prop polygon, if its colour's nearest
  centre (k-means over each look's garment-polygon interiors vs trusted skin and hair centres) is a clothing centre,
  or if it lies outside the frame's skin regions; a hair pixel by the polygon and colour rules. Pixels are never added.
- **Finding:** the masks at `c8ed8608` (the art line's v3 revision) already had 0 skin or hair pixels inside garment
  or prop polygons; with them, an S3 tint matches the art line's baked v3 frames within one level and the fur cloaks
  do not darken (`$DATA/masks_sheet.png`: six fur-cloak looks, S idle and E work_a, old vs new mask). The darkened
  cloaks the owner saw came from an earlier revision (POLISH2's v5 put six fur/hem pixels into the masks; later
  revisions removed them). The figure of a deep-brown-skinned person in a dark-brown cloak still reads darker overall —
  a contrast of colours, not a mask defect; recorded for the owner.
- So the fix is a guard: it removes 177 skin pixels by colour (dark borders where skin meets hair, cloak or a carried
  log; a few on child frames' logs, whose envelopes have no prop polygons) and 3 by region (armpit pixels under a
  raised arm), 63,672 → 63,492 skin pixels (99.72 %), 54 hair pixels; the largest loss in one frame is 5.7 %.
- **Regions** (TEMPLATE_V3 defines none; the art line's boxes exist only for S idle and the 8 owner frames): computed
  per frame from the source geometry and template constants, never from the masks — head (the hair envelope's box,
  extended down by the neck rows of the owner supports: adult 5, child 8), legs (hip row to the foot line: adult 82,
  child 92), arms (a shoulder band down to the head box's bottom; left and right of a torso core in S, the figure right
  of the head box's left edge in E), sleep (head left; arms 30 %, legs the last 45 % of the body length), and a torso
  region only for the bare-chested tops (loincloth, hide wrap; the goal's "face/forearm/leg boxes" would leave a
  loincloth chest untinted, a visible two-tone body). Stored in `assets/v8art/people/masks/skin_regions.json`.
- Colour guards: a clothing centre within ΔE 12 of a skin centre is not used (tan hide and skin shadow share colours:
  without the guard the colour rule removed 23.5 % of skin, mostly limb shading); hair guard ΔE 8 (dark fur and dark
  hair share colours; without it 13 % of hair went as speckles).
- Tests (`tools/v8/test_art_masks.py`, 9, in gate step 16): files and sizes, `--check` idempotent, 0 clothing-cluster
  pixels in any skin mask and 0 polygon pixels in any skin or hair mask over all 340 frames, every skin pixel inside its
  regions, masks only shrink against the git originals, skin coverage ≥ 80 % (observed 99.72 %); each shown failing on
  a planted fur pixel and on a planted out-of-region pixel.

## D15 — Things on the map (Part 4; the things worker's decisions, checked by the lead)

- **Finished huts** are drawn from the bindings ring (`buildings.hut.ring`: the hut's wall/door ring around its 3×3),
  because `get_space` carries no wall cells for a hut; a hut's bedrolls are not drawn separately inside it.
- **Trees, nodes and items** are made only near the view (16×16-tile chunks, the ground's chunk grid), as nodes in
  `ColonyView/World` so they y-sort with people; far chunks are freed.
- **Map labels** are 24 screen px whatever the zoom (`labels.follow_camera`), on the dark backing box (craft rule 6).
- **Hearth states**: unlit / firing / lit from the settlement's fire, each a manifest sprite; the lit one carries the
  light flag (`buildings.*.light`) the daylight controller reads.
- **Kills**: a carcass is not an animal; an item whose bindings row names `carcass_of` (the kill's meat and hide)
  lying outside every stockpile zone draws that species' dead frame (`animals.codes[*].dead`) instead of its stack, so
  a kill reads as a dead deer until it is hauled.
- **Item stacks**: buckets 1 / 2–8 / 9+ (`items.bucket_limits`); a kind with `assets` names one sprite per bucket,
  else `copies` repeats one sprite by `items.stack_layout`.
- **Designation marks** are ink glyphs on a pale disc; **site ghosts** a pale wash plus an ink outline of the finished
  sprite (`site_ghost.gd`).
- **Visitors** (migrant waves, caravans) are route I people with an adult look picked by a keyed hash of the group;
  traders carry; the pack is the `visitors.pack` item sprite.
- **Trees** are one y-sorted sprite each (crown included), so a crown south of a person covers them as in the boards;
  `tree_layer.under_canopy(t)` reports when a crown can hide a person on tile t, but nothing fades crowns in this pass
  (a hidden person still shows their LOD marker below 1×); a crown fade is left
  to the owner (report, "오너 결정 필요").
- `tools/v8/verify_v8_0.gd` (M1's V8-0 screen check, not in the gate) still assumes 16-px tiles; it is M1 history,
  kept as tooling of record and not ported (it would need the M1 renderer to mean anything).

## D16 — What stays of the old art path

- Deleted (Part 6): the M1 tile sheet and its generator (`tools/v8/gen_tiles.py`, its test, `assets/v8/tiles.png/json`),
  `economy_text.gd`'s sheet names, and the porter drawing nothing called.
- **Kept**: `scripts/world/art_sprite_library.gd` and `assets/art/atlas`, because the HUD's 32-px icons
  (`art_ui.gd` → `ArtUI.icon`) and `wood_visuals.gd` (the settlement fire read the daylight controller uses) still use
  them; they draw no map content. Moving the HUD icons to the manifest is M-EXT work (recorded in the report's limits).
- Content names still in UI code (M-EXT debt, outside this pass's renderer): `economy_text.gd` `ITEM_NAMES` and
  `MATERIAL_KEYS` (locale keys per item code), the `built_at` stone classification; in `scripts/world/colony/
  order_tool.gd` the M1 blueprint order names (`"hut"` as the default build kind and the toast's hut exception), which
  are snake_case command names outside the grep's id set. The extensibility grep checks double-quoted string literals
  in `scripts/world/**/*.gd` against the manifest, look, material, style and bindings ids; it does not read shaders
  (which hold no ids) or UI scripts, and it misses single-quoted strings, concatenated or formatted ids and ids inside
  paths (code review S4). Widening it is M-EXT work.

## D17 — People at night: a pale halo away from the fire

- The goal (§3.5) asks the night tone to keep people's silhouettes separable: luminance contrast person vs ground
  ≥ 0.15 on the night screen. The adopted night transform (the art line's `tone.py night_value`) multiplies people and
  ground alike by 0.55, so a person's contrast at night away from a fire is about 0.55 × their day contrast. Measured
  on the day-6 02:00 save (`tools/v8/mart2_ui_test.gd`, 10 people asleep in the walled shelter outside the hearth's
  5-tile light): median 0.047 by the fill measure, below the bar; day 20 noon by day: median 0.147.
- Fix: the project's craft rule 5 (worldsim-godot: "a halo where they could be lost: … at night away from a fire").
  A person who is neither selected nor drafted gets the shader's 1-px outline in `people.night_halo_rgba`
  (232, 226, 206) at alpha `people.night_halo_alpha` (0.8) × `DaylightController.dark_at(feet)` (the night factor
  times one minus the strongest lit hearth's linear restore there), in 1/16 steps so the shader parameter changes
  rarely. The people's own colours are untouched (goal §3.5: only the global value change touches them); the rim is
  multiplied by the night tone like everything else. By day and inside the hearth light there is no halo.
- Measure (now in the UI test): per person on the night screen, Rec.709 luminance means of the opaque frame pixels
  (fill), the 1-px rim just outside the silhouette, and the uncovered ground 4–6 px outside; contrast = the larger of
  |fill − ground| and |rim − ground|; the median over the people on screen must be ≥ 0.15. With the halo: median 0.204
  (min 0.023 for a sleeper at the hearth's edge, where the light restores day contrast). The check fails without the
  halo (`night_halo_alpha` 0: median 0.047).

## D18 — Review dispositions (Part 7)

Two fresh reviewers, each with a file report (`$DATA/reviews/code_review.md`, `$DATA/reviews/visual_review.md`).

Code review (verdict "request changes"):
- **B1 (blocker, fixed):** `agent_layer.gd`'s `const HALO_STEPS: float = 16.0` matched the no-16-px-tile grep, so gate
  steps 16 and 33 were red at `bab5ec4a` (the p5 commit came after step 33 was added and the step was not re-run).
  Now an `int`. Lesson: run the gate's Python steps after every commit that touches `scripts/world/`.
- **S1 (fixed):** an F12 reload cleared the wall texture sets and rebaked before re-applying the space, so built walls
  vanished until the next space change. **S2 (fixed):** after a tone change (season, drought) or F12 the hill-face
  cap kept the previous tone's image. Both: a reload or tone change now re-applies the space in full (sets, hill cap,
  faces rebuilt) before rebaking. New checks in `mart2_ui_test.gd`: `f12_keeps_walls` (95 → 95 strips, sets 1) and
  `f12_refreshes_hill_cap`; both fail with the old path (95 → 82 strips, sets 0).
- **S3 (fixed):** a wall's end piece reads its diagonal south neighbours, but a change marked only the 4-neighbour
  chunks; `_mark` now marks all eight (the dig test still rebakes exactly one chunk for an interior tile).
- **S4 (recorded):** the grep's limits and the order tool's blueprint names are listed in D16 as M-EXT debt.
- **S5 (fixed in part):** the parity fixture grows from 30 to 39 cases (a caller category against a known material's
  own; exact styles asked for another kind). Mutating `material_resolver.gd` now fails parity for "exact match
  ignores kind" (15 differences) and "caller category overrides" (5). Not covered, because the art line's data has no
  case for them: an exact style id that is also another style's blueprint, and `texture_override` (no material has
  one); a fixture material for them would diverge from the art line's documents.
- **Notes (recorded, not changed):** the animal flip offset is latent (all anchors centred); every space change
  re-resolves all walls (11.7 µs each); zooming out to an unseen area bakes every new chunk in one frame (estimate:
  ≈ 40 chunks × 1.8 ms ≈ 70 ms once; not measured); ford sprites recreated on space changes; elder portraits have no
  grey hair (the portraits are the art line's per-variant files).

Visual review (verdict "acceptable", no blockers):
- **Hills read as plain ground (fixed):** the ENV1 cap brought `hill_rock` to dark grass's mean (78, 73, 62 vs
  80, 76, 62), so only the south-edge cliffs showed, as floating slabs at ½× and ¼×. `ground.value_by_ground`
  (render params, data; applied after the cap, so the ENV1 maxima still hold) darkens hill tops to 0.6: the rock mass
  reads as in the old renderer and in STRUCT's board, where hill interiors are rock ground.
- **The dug pocket did not read as a room (fixed):** dug floors used the same hill rock; the art line draws dug cells
  as a floor asset, so `bindings.json` now maps `DugFloor` and `dug_ground` to `packed_earth_ground`.
- **Drafted outline faint at 1× (fixed):** `people.drafted_outline_px` 2 (selection and halo stay 1 px).
- **Migrant dialog covered the migrants (fixed):** the shot frames the camp 5 tiles below the centre.
- **Notes kept:** LOD markers share the tan top colour of most looks and can sit on items; glyphs ≈ 12×7 px; no LOD
  halo at night; a map label cut at the screen edge; elders' grey reads blue-grey; bedrolls look like sticks (as in the
  boards); the storehouse's north wall under the HUD in the 2× shot.

## D4 update — what was built

- People are pooled `Node2D`s with one `ShaderMaterial` each (the variant tint as per-material uniforms set only on
  change), not one material per look: per-person materials cost nothing measurable at 1× (≈ 30 people) and the
  1,000-person world never draws bodies (LOD). LOD markers are fallback step 2, taken: `draw_circle` per person held
  the 1,000-person world at 27 FPS at ½×; MultiMesh discs plus a glyph atlas gave 211 FPS (Part 6 commit).
- Chunk and strip textures are reduced ½ or ¼ below 1× (re-textured 4 per frame): texture memory at ¼× fell from
  546 MB to 87 MB.
- **Final measurements** (2026-10-24 11:1x KST, `tools/v8/mart2_fps.sh`, 30 s warm-up + 60 s, windowed 1920×1080,
  vsync off, release library; raw `$DATA/fps/fps.jsonl`; 1-minute load 2.9–5.7, none contended): seed 0 day 20
  (12 people) 1× **328.6 FPS**, ½× **238.2**, ¼× **120.65** (budget 120: met by 0.5 %; 2,252 draw calls, the whole
  map's standing sprites); 1,000-person stress world ½× **186.8**, ¼× **173.5** (budget 60). Texture memory 265 MB at 1×,
  176 MB at ½×, 87 MB at ¼× (budget 512). Chunk bakes (a one-off windowed script, not kept: `GroundLayer.bake_chunk_now`
  over 144 chunks of the day-20 world, then 16 again; load ≈ 6): fresh median 1.8 ms, p90 2.4 ms, max 5.3 ms (1 of 144 over 4 ms); rebakes median 2.1 ms, max 3.7 ms
  (budget 4 ms: met); a dig in the test applies space and rebakes in 1.4 ms. The first bake after a load reads 17.5 ms
  (cold texture conversion, once per load) and the load's `warm()` 101 ms. The ¼× margin is thin: fallback step 3
  (trees baked into chunks at ¼×) was not needed and is left for the owner's call (report).
