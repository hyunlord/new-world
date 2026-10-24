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
