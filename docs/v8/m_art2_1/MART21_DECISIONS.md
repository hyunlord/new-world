# M-ART2.1 decisions

Goal: `docs/v8/m_art2_1/MART21_GOAL.md`. The goal says "never ask the owner": every choice the goal leaves open is
recorded here with its reason. Raw data lives in `/Users/rexxa/work/mart21-data/` (`$DATA`), outside the repository.

## D1 — intake (Part 0, 2026-10-24 17:2x KST)

- **Base:** `v8/wip` = `origin/v8/wip` = `a1c20220b5dc1d4c86fc983950152b7691ae6a41` (D19 on `v8/main` `d796fcac`),
  as the goal states.
- **Clean checkout:** `git status --porcelain` showed only the three items the goal names as left alone
  (` M .gitignore`, `?? .ignore`, `?? docs/revitalize/`) plus this pass's own `docs/v8/m_art2_1/` (the goal file).
  No other Claude Code process had this checkout as its working directory (`lsof -d cwd` over every `claude`
  process: ten sessions, all in other repositories). Start condition met.
- **Art source:** `origin/v8/art-trees` head **`f182769fe06b1cb60105e4d0bf127b6524277838`**
  ("v8-arttrees(p3): bind owner review artifacts to the verified remote commit"); it contains
  `docs/v8/arttrees/TREES_REPORT.md` (verdict "오너 판단 대기", i.e. the art line hands the files over with the owner
  review open). The branch descends from the current import commit `c8ed8608` (art-polish2); outside `art/flora/`,
  `art/boards/` and `docs/v8/arttrees/` it changes only `art/style05/palette_05.json`, which the import does not copy.
  So moving the import's `source_commit` to `f182769f` leaves every already-imported file byte-identical (checked by
  `art_import.py --check` after the move, Part 2). The branch is not merged; files are read from its git objects.
- **Owner notes:** `docs/v8/m_art2_1/OWNER_FEEDBACK.md` does not exist at start, so §7's owner-notes scope is
  empty (nothing to triage; nothing goes to M-EXT on that account).
- **Gate on `a1c20220`:** run once, unchanged code, in a detached worktree (`$DATA/gate0wt`, the main clone's
  `Cargo.lock` copied in) so Part 1 can proceed in this checkout meanwhile:
  `BENCH_PASS=M-ART2.1 bash tools/v8/gate.sh $DATA/gate0 --base-data /Users/rexxa/work/m55-data/base/identity`
  (the M-ART2 base). Host load at start 9.2 (contended; this pass's own work runs beside it), so a red `bench`
  there is read as contention unless the final gate repeats it. Result: PENDING.

## D2 — rounded shores and hill edges: a closed-form field per tile quadrant (Part 1)

The goal (§2.1) asks for a signed distance field from the water/land tile boundary, computed per chunk with a 2-tile
apron by an exact EDT on a 4×-upsampled grid and bilinear sampling, with the contour offset so that straight shores
stay on the tile edge while a 90° step becomes a quarter circle of radius ≈ 22 px. Implemented as the exact field of
that shape instead of a sampled one:

- **Shape:** the opening and closing of the water-tile union by a disc of radius `water.shore_radius_px` (22) — the
  shape whose straight shores lie on the tile edges and whose corners are quarter circles. Because the radius is
  under half a tile, every pixel's nearest contour lies inside the 2 × 2 tiles around its tile quadrant's corner, so
  the field is closed-form there: a line for a straight pattern, a rounded quarter plane for an odd quadrant out
  (convex for that class, concave for the other), and for a diagonal pair the two class-0 corners rounded so that
  diagonal water stays joined (a staircase river does not pinch into puddles). This is the field an EDT would
  approximate, without the 16-px sampling error (h²/8R ≈ 1.5 px at R = 22), and the "2-tile apron" is implicit:
  each tile reads its eight neighbours from the whole map, across chunk borders.
- **Cost:** a 32 × 32 mask set per (quadrant, 2 × 2 pattern, noise phase) is computed on first use with a GDScript
  loop and cached; a chunk bake applies them with native `blit_rect_mask` / `blend_rect_mask` only (MART2 D4 rule).
  Seed 0 uses 577 masks over the whole map. Headless timing on a contended host (load 9.2, 144 chunks): fresh bake
  median 2.96 ms, p90 4.34 ms, max 42.1 ms (the first chunks build masks once); rebake median 2.70 ms, p90 3.25 ms,
  max 4.25 ms (budget ≤ 4 ms median: met). The windowed numbers are in the FPS run (Part 4).
- **Coverage:** smoothstep(−6, 6, s) (`water.contour_soft_px`) is applied as a dither probability against the tile's
  hashed threshold, not as an alpha ramp: craft rule 2 (no gradients, no anti-aliasing) and Godot's binary mask
  blend. The M-ART2 10-px bank weight and the 2-px ink (displaced 0–4 px by the noise at the nearest contour point,
  found along −∇s) are measured from the rounded contour. Ink pixels are forced to land.
- **Hill tops** (`hill.edge_radius_px` 16): the same field between undug hill tops (`GroundMap.rock`) and open ground,
  crisp (no dither, no ink), skipped at any corner whose upper row hangs a face (`GroundMap.face`), so faces stay
  straight where they hang. Walls and water never round against hills.
- **Invalidation:** a tile's masks depend on its 3 × 3 neighbourhood; `_mark` already marks all eight neighbour chunks
  of a changed tile, and a changed `rock` flag now counts as a change.
- **Measured (synthetic L, `mart2_ground_test.gd` §7):** contour radius 21.8–22.8 px at the outer and the inner corner
  (14 rays; bar 22 ± 3); straight shore 0.00 px off the tile edge (bar 1); the visible edge (land fraction 0.5) sits
  5.4 px into the water on the straight shore and 6.0 px at the arc (a square corner would read 15.1); hill corner
  16 px (square: 22.6), face corner square. With `shore_radius_px` 0 the radius and bank checks fail.
- The hearth chunk's recorded hash changes `2bb6d230…` → `1b046ce1…` (intended; `--write`).

## D3 — the water texture is back, with a per-chunk lift (Part 1)

- D19's flat recolour (`tone.gd` `water_colour`, `water.colour`, `water.texture_std` 0.025) is removed. The 05 water
  texture is uploaded unchanged: luminance std **0.0635** (the goal's estimate 0.06–0.09), mean Y 0.435, hue 0.42.
- The contrast rule is a lift in the chunk shader (`water_lift`, an RGB offset): per chunk, against every land ground
  touching water (8-neighbourhood) — its prepared texture mean, and its map-tint mix at ½× and ¼× — first a hue
  offset only if the water's hue sits within `minimum_land_hue_distance` + margin of a land (never needed on seed 0:
  min hue distance 0.28 turns), then a grey (hue-neutral) offset so that the water clears every land by
  `minimum_land_luminance_difference` divided by the night multiplier's weakest channel (0.55 × 0.86) + margin, i.e.
  the rule holds at full night. The texture's variation is kept (an offset, no compression); no pixel clips on seed 0.
  The lift is recomputed with the tone (winter snow, drought) because the land means change. `water.texture_lift`
  false turns it off (the negative control: night difference 0.043, the checks fail).
- `water.map_mode_colour` keeps D19's ¼× / ½× map-mode mix target for water (a 0.24 weight at ¼×).
- **Measured:** headless, every seed-0 chunk with water (18), baked land beside water against the lifted water mean:
  min night difference 0.1035 by day tone, 0.1042 winter, 0.1035 drought (bar 0.10); min hue 0.28–0.30 turns (bar 0.12).
  Windowed (`--rendered`, real GPU, interior samples of every class at 2×/1×/½×/¼× by day and at full night): 128
  checks pass; smallest night luminance difference 0.1106 at 1×, 0.1145 at ¼×.
- Ice tint (droppable, §2.1): dropped; winter keeps the same lift rule.
- Fords: the four seed-0 ford tiles keep their 16 stones on water pixels; 2 ford-tile corners turn to land by rounding.

## D4 — ART-TREES in the game: import, flora resolver, season shader, ¼× canopy bake (Part 2)

- **Import:** `tools/v8/art_import.json` `source_commit` → `f182769f` (`v8/art-trees`); every previously imported file
  is byte-identical (`art_import.py --check`: 0 differ; the manifest diff is the commit, the total and nine removed
  rows). A `flora` section imports `art/flora/species.json`, `habit_defaults.json` and both schemas (role
  `flora_data`), every frame, mask and map canopy the catalogue names (`flora_frame` id = stem with its anchor and
  canvas, `flora_mask` id = stem + `_mask`, `flora_texture`), and the data-only fixture `tests/test_alder.json`
  (`flora_fixture`, not in the catalogue) into `assets/v8art/flora/`: 705 files, 14.2 MB of the 40 MB budget.
  `--verify` also checks that every catalogue path is imported and every bound species exists. The TRIAL2b tree,
  stump and bush sprites (`structures/{oak,birch,pine}_{large,small}`, `stump`, `berry_full/bare`) are removed;
  berry bushes bind the new ART-TREES bush frames (`berry_bush_full/bare`, 96 × 96, anchor (48, 80)).
- **Resolver:** `scripts/world/art/flora_resolver.gd` ports `resolve_flora.py` (fallback by habit with the habit's own
  season records, Python-style non-negative modulo, `requested_id` kept) and the schema's cross-checks as `problems`.
  Parity (gate step `resolver_parity`): `tools/v8/flora_parity_fixture.json`, 352 cases — 4 species × 2 sizes ×
  4 seasons × 8 signed hashes (int32 extremes), an unknown species with no habit and each habit, `test_alder` merged
  as boards.py `fixture_catalog` does — against the art line's own `resolve_flora.py` at `f182769f`: 0 differences.
  A planted mutant (unknown species taking the fallback's seasons) gives 8 differences.
- **Species stays a keyed hash** (bindings `trees.species` weights 5/3/2 for oak/birch/pine; size by `large_permille`,
  variant by a third salt): the simulation has no species. **M-EXT must replace the hash with the snapshot's species
  column** (`TreeLayer.tree_species`), and the felled log's material with the same id (TREES_REPORT "M-EXT에 넘길 것").
- **Season shader** (`shaders/tree_season.gdshader`): the CIELAB shift is applied per masked pixel in the shader,
  exactly as `seasons.py` `apply_tint` and `person_variant.gdshader` do, rather than as precomputed RGB multipliers
  (§2.2 says "as for faces"; the faces' shader is per-pixel Lab, so this is the same method, without a mean-colour
  approximation). The snow band is `seasons.py` `snow_layer` (top support pixel per column + depth, within mask ∧
  alpha, alpha floored to 178/255), made once per frame on the CPU. First day of a season: shift and snow alpha blend
  by the day's fraction (quantised to 1/96), a frame change (deciduous into or out of winter) dissolves per pixel by a
  hash; the world's first season starts unblended (no previous season was ever on screen). Materials are shared per
  (frames, season pair); bushes take their species' tint through the same material (mask = frame id + `_mask`).
- **¼× canopy bake:** at texture factor 4 only, each tree tile's quadrants carry the canopy-mass texture (world-sampled)
  over the union of tree tiles rounded by `trees.canopy_radius_px` (20), darkened by `canopy_edge_value` (0.7) inside
  `canopy_edge_px` (6) of the edge, before the shores (water still carves through). Per species and season the
  canopy texture's mean is moved by the season's Lab record as an RGB multiplier on a ¼-size copy scaled back by
  nearest (the ¼ texture keeps every texel; ~4 k pixels per style instead of 65 k); a deciduous winter uses its bare
  frame's mean colour (leafless brown-grey mass, so the ¼× forest stays baked in winter: the art line's offline
  compositor drew bare branch sprites instead, which would put ~80 % of the trees back as sprites). Tree tiles leave
  the map tint at ¼× (the canopy is the forest's class colour). The ¼× canopy switches season at half blend; pine's
  snow is not baked. The tree layer hides every sprite in a canopy chunk (`canopy_baked_at`, re-checked when
  `canopy_version` moves). Seed 0 at ¼×: 3,123 tree sprites held, 0 drawn; 719 drawn again at 1×.
- **Tests** (`tools/v8/mart21_trees_test.gd`, added to gate step 34): four seasons × 4 species × 2 sizes through
  `make_plant` (frames from the right list, bare in winter for both deciduous species, snow exactly where recorded),
  the data-only `test_alder` in four seasons with its own records, the ¼× canopy (tile hue 0.279 vs canopy texture
  0.274, 0.099 at 1×) and hidden sprites. Windowed: summer canopy saturation 0.58–0.62 for every species (bar 0.30;
  no tone cap), autumn b* shifts follow the records (oak +11.1 for +15, birch +25.6 for +32, pine 0.0 for 0,
  test_alder +15.9 for +16). The M-ART2 rendered tree check now draws through `make_plant` in summer (identity tint)
  and its bare-land samples skip tree tiles: 129 windowed checks pass.
- Visual check (seasons strip, day 8 / 14 / 20 / 26 noon, tile (112, 103)): summer green, autumn oak ochre and birch
  yellow with pine unchanged, winter bare oak and birch with snow-tipped pines, spring lighter green.
