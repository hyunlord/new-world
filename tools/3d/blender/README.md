# Procedural 3D models (Blender, headless)

Scripts in this folder are the source of truth for every model in `assets/3d/models/` and every
vertex-animation texture in `assets/3d/vat/`. Never hand-edit the outputs; change the tables and rerun.

## Run

```sh
# from the repo root (Blender 5.3; any 4.4+ with slotted actions should work)
blender -b --factory-startup -P tools/3d/blender/build_all.py          # everything, ~3 s
blender -b --factory-startup -P tools/3d/blender/stations.py           # one family (merges into manifest)
blender -b --factory-startup -P tools/3d/blender/animals.py -- wolf    # one species
blender -b --factory-startup -P tools/3d/blender/chars_b.py -- b_man   # set B (textured) chars + their _lod bake
blender -b --factory-startup -P tools/3d/blender/chars_c.py -- c_wolf  # living animals/people chars + _lod
python3 tools/3d/blender/verify_vat.py                                 # VAT + glb checks, exit 1 on failure
blender -b --factory-startup -P tools/3d/blender/render_contact.py -- [out_dir] [id filters...]
blender -b --factory-startup -P tools/3d/blender/render_b_lod_contact.py -- [out_dir]  # set B full vs LOD
blender -b --factory-startup -P tools/3d/blender/chars_c.py -- contact [out_dir]       # living full vs LOD
```

`render_contact.py` writes `blender_models_<stamp>.png` (every glb, front-3/4, labelled with id and
triangle count) and `blender_poses_<stamp>.png` (every character re-posed from its VAT `.bin` at the
middle of each clip, coloured by the COLOR_0 tint mask, or by the atlas for textured characters). Default out dir: `/Users/rexxa/work/3d-data/screens`.

Output is deterministic: two full builds produce byte-identical files (seeded RNG per model, fixed
data order, no timestamps in the glTF). The build never deletes files; it overwrites each output in
place and rewrites `manifest.json`.

## Files

| file | contents |
|------|----------|
| `common.py` | scene reset, material API + default colours, `Model` mesh builder (box/cyl/rod/ball/tube/lathe), glb export, manifest, armature/action/VAT helpers |
| `flora.py` | `SPECIES` (oak/birch/pine proportions) + `TREES` variants (seed + overrides) + `SMALL` (bush, berry_bush, sapling, stump) |
| `rocks.py` | `ROCKS` table: boulders, flint/ore outcrops, clay bank |
| `stations.py` | `STATIONS` table (footprint, sockets, colour overrides, params) + one builder per id |
| `items.py` | `ITEMS` table + builders |
| `people.py` | `BODY` proportions, bone list, `CLIPS`, procedural actions for `villager` |
| `animals.py` | `SPECIES` table driving one quadruped generator, `CLIPS`, shared actions |
| `chars_b.py` | bakes the accepted, atlas-textured set B glbs (`assets/3d/source/chars/B/`) into `vat/b_*`, plus a decimated `vat/b_*_lod` crowd bake per subject |
| `chars_c.py` | bakes the M3D-ASSETS living animals/people glbs (`assets/3d/living/`) into `vat/c_*`/`vat/p_*`, same shape as `chars_b.py` (shares its helpers), plus their `_lod` bake; writes its own fragment `vat_c_manifest.json` |
| `build_all.py` | runs all families, writes `assets/3d/models/manifest.json` |
| `verify_vat.py` | plain python3 checker (no Blender) |
| `render_contact.py` | review images |
| `render_b_lod_contact.py` | full-vs-LOD review image for set B characters |

## Conventions

* 1 Blender unit = 1 m = 1 tile. glb with +Y up (default exporter conversion). Scripts build in Blender
  Z-up with the model FRONT facing **-Y**; glTF maps (x, y, z) -> Godot (x, z, -y), so the front faces
  Godot **+Z** (south). The character's left is +X.
* Origin = centre of the footprint on the ground. A W x H model fits inside W x H m (W along X, H along
  Godot Z). Trees/bushes may overhang their tile (`"overhang": true` in the manifest).
* Low-poly, flat-shaded (every face is flat; no smoothing), colour only through materials.
* **Material names are an API** (Godot recolours by name). Only these exist (`common.PALETTE`):
  `trunk canopy fruit stone vein earth wood wood_dark clay metal glow thatch hide cloth skin hair
  leather horn fur fur_light snow`. Unknown names raise an error. A model may override the *default*
  colour baked into its own glb (`colors={...}`, e.g. pale birch `trunk`) without changing the name.
  Where no name fits: flint nodules use `vein` (dark), water surfaces (retting pit, well) use `metal`
  (blue-grey), meat uses `fruit` + `horn` for the bone, pot openings/contents use `earth`.
* `glow` is emissive (fire, embers, molten metal). Lit stations have a `light` socket.
* Triangle budgets (checked at build time, warnings printed): trees 150-400, rocks 40-150, stations
  200-1200, items 20-150, villager 500-1000, animals 300-900.

## manifest.json

`models.<id>`: `file`, `footprint` [w, h] tiles, `height` m, `triangles`, `materials`, `sockets`
(Godot coords: `light`, `work` = where a worker stands, defaults to just in front of the footprint),
`family`, plus extras (`overhang`, `species`, `pivot` for `door`; characters add `animations`,
`bones`, `vat`).

`door` is only the leaf (0.9 x 2.0 x 0.08 m); its pivot is the left edge (x = 0), the leaf extends to
+X. The frame is drawn by Godot.

## Characters and VAT

Characters are rigid-skinned: every primitive is weighted 100 % to one bone (`bone=` argument), which
keeps the low-poly look and makes baking exact. Poses are written as rotations about **world** axes
(converted to each bone's rest frame, so bone roll never matters): rotation about +X turns forward
(-Y) towards down, i.e. positive = nod/lean forward for upward bones, swing *back* for limbs hanging
down. Actions are keyed every frame at 30 fps from functions of normalised loop time `s` (0..1) and
exported as glTF animations named exactly as the clip.

* villager: `idle walk work carry sleep`; bones `hips spine chest neck head upper_arm_L/R lower_arm_L/R
  hand_L/R upper_leg_L/R lower_leg_L/R foot_L/R`; materials `skin hair cloth leather`.
* animals: `idle walk run graze rest attack` (attack does not loop); bones `hips chest neck head tail
  {front,back}_{upper,lower}_{L,R}`; materials `fur fur_light horn`.

For every character `export_character` also writes:

1. `vat/<id>_static.glb` - rest pose, not skinned, every face split (one vertex per loop, flat
   normals), ONE material (`vat_<id>`), `TEXCOORD_1` = ((vertex_index + 0.5) / width, 0.5), `COLOR_0`
   tint mask: skin/fur (1,0,0), hair/fur_light (0,1,0), cloth/horn (0,0,1), leather/other (0,0,0).
2. `vat/<id>_vat.bin` - float32 LE RGBA texels, row-major, width = vertex count. Rows `[0, frames)`
   are positions (Godot xyz, w = 1) for every frame of every clip at 15 fps; rows
   `[normals_offset, 2*frames)` are the matching normals (w = 0). Texel x for a vertex = `UV2.x * width`
   (the glb's vertex order may differ from the bake order; always look up through UV2).
3. `vat/<id>_vat.json` - `width, vertex_count, frames, fps (15), normals_offset, clips{name: start,
   count, loop, duration}, bounds_min, bounds_max, height`.

Looping clips store `duration * 15` rows (the end frame equals the first and is dropped); non-looping
clips store one extra row so the last pose is included.

## Set B characters (textured, `chars_b.py`)

`b_man b_woman b_child b_goat b_aurochs` are not modelled here: they are the accepted skinned, atlas-textured
glbs from `v8/art-3d-chars` commit `f0b91f999`, copied once into `assets/3d/source/chars/B/<subject>/`
(`SOURCE.md` names the source). `chars_b.py` imports each glb and bakes it like `export_character`, except:

* the static glb keeps the source **UV0 (`TEXCOORD_0`) = atlas UVs**, copied per loop by the split; `UV2`,
  `COLOR_0` (all black: no tint region) and the single material are as above. It is posed at **idle frame 0**
  (the bind pose is an A-pose; the shader replaces positions anyway).
* VAT columns are the body's vertices followed by any object-animated mesh (the humans' 24-vertex `CarryLoad`,
  scaled to ~0 outside `carry`; then a procedurally-built ~20-vertex `HandTool`, scaled to ~0 outside `work`),
  so each bundle appears only in its own clip.
* `HandTool` (`_hand_tool` in `chars_b.py`) is a hex-section haft (~0.55 m) with a small wedge head, built
  directly in the armature's rest space with the grip centred on `hand_R` and the haft along that bone's
  rest head->tail axis, then rigidly skinned 100 % to `hand_R` (the same technique as the body's own rigid
  low-poly parts), so it follows `work` automatically; scale is keyframed per clip (1 in `work`, 0 elsewhere)
  through its own NLA tracks, same mechanism as `CarryLoad`. UV0 is two solid texels hand-picked from the
  subject's own `atlas.png` (`TOOL_UV`): a wood-brown patch for the haft and the atlas's least-saturated
  patch for the head. Man and child have no true grey pixel anywhere in their atlas (every uniform patch is
  a skin/hair/cloth brown); for them the "head" texel is simply the darkest, most desaturated patch found,
  not literal grey. Woman's atlas does have a genuinely near-neutral grey patch (used as-is).
* Clips are sampled at 15 fps from the source actions (1.333 s each); `loop` is true when the last pose matches
  the first within 1 cm (all do). Game names the source lacks are **aliases** with `"alias_of"` and the same
  rows: animals `run -> walk`, `rest -> lie_down`, `attack -> idle`. Source names (`lie_down`) are kept too.
* json adds `"textured": true, "atlas": "b_<subject>_atlas.png", "source_commit", "source"`;
  `vat/b_<subject>_atlas.png` is a byte copy of the source atlas. Source scale is kept (man/woman 1.7 m,
  child 1.2 m). The manifest `file` is the source glb (no `models/b_*.glb` is written), `materials` is empty.
* No `.bin` is above 2.3 MB (about 2800 columns x 100 rows x 8 bytes).

To refresh: re-copy the five folders from the art worktree, update `SOURCE_COMMIT` and `SOURCE.md`, rerun.

### Crowd LOD (`<id>_lod`)

Each subject also gets a low-detail `b_<subject>_lod` bake, for drawing thousands of far-away animated
people: `vat/b_<subject>_lod_{static.glb, vat.bin, vat.json}`, same clips/fps/conventions as the full
bake, and the json's `atlas` points at the **same** `b_<subject>_atlas.png` (no second atlas is written).
It is built by `_decimate_body` in `chars_b.py`:

* a Decimate (Collapse) modifier is inserted **before** the Armature modifier on the body mesh and its
  ratio is binary-searched so the decimated rest mesh keeps at most 400 vertices used by a face, then
  the modifier is applied (baked into the mesh) so the fixed, already-decimated topology is what the
  Armature deforms per frame; this keeps clip sampling and skin weights correct (Decimate carries the
  vertex-group layer over when it merges vertices). The body is many disconnected low-poly islands, so
  Decimate can collapse a small island's faces away while leaving its vertices in the mesh; any vertex
  left with no face after applying is deleted.
* the humans' 24-vertex `CarryLoad` bundle and ~20-vertex `HandTool` are kept at full resolution
  (already tiny; decimating them further would risk breaking their shape for no real savings) and
  still become extra VAT columns, as in the full bake; the body's own decimate budget is reduced by
  their combined vertex count first, so the character total still fits.
* budgets are asserted at bake time: width (VAT columns) <= 400, `split_vertex_count` (static mesh,
  one vertex per loop) <= 900; actual sizes are man 400/750, woman 399/786, child 398/783, goat
  400/792, aurochs 400/816 (width/split).
* the manifest gets a `b_<subject>_lod` entry alongside `b_<subject>`, same shape (family, animations,
  bones, `vat`), so the game loads it exactly like the full character.

`verify_vat.py` needs no changes: it drives its checks off every `family == "character"` manifest
entry, so `b_<subject>_lod` is checked the same way as `b_<subject>` automatically. Likewise
`build_all.py` needs no changes: `chars_b.build()` now bakes the full and the LOD for every subject.

`render_b_lod_contact.py` renders a full-vs-LOD contact sheet (each character re-posed from its VAT at
the middle of `walk`, both columns textured from the shared atlas) for visual review; it is not part of
the asset build.

## Living characters (textured, `chars_c.py`)

`c_<species>` (`aurochs deer wild_goat reindeer wild_boar wild_sheep wolf`) and `p_<age>_<sex>_<era>`
(18 combinations of `adult/child/elder`, `M/F`, `neolithic/bronze/iron`) bake the M3D-ASSETS skinned
glbs from `assets/3d/living/` the same way `chars_b.py` bakes set B (same helpers, same static/bin/json
shape, same `_lod` crowd bake); `assets/3d/models/manifest.json` is never touched, since this pass owns
only VAT output -- rows go to the fragment `assets/3d/models/vat_c_manifest.json` (same pattern as
`stations_w3d.py`), and `verify_vat.py` reads it alongside the main manifest automatically.

* Animals: one skinned body mesh, no extras; same `run -> walk`, `rest -> lie_down`, `attack -> idle`
  aliases as `b_goat`. The body mesh's object name is the source directory (`aurochs/deer/goat/
  reindeer/wild_boar/wild_sheep/wolf`; `wild_goat`'s source directory kept its old name `goat`).
* People: the 18 `person_<age>_<sex>_<era>.glb` files share one of three bodies (`man`/`woman`/`child`,
  by sex/age) but each adds its own era wardrobe: extra skinned mesh objects (same armature, so they
  pose for free) with solid-colour materials that have **no texture of their own** -- unlike the body
  and the atlas-textured `CarryLoad` bundle, which both keep their real UVs. Since the baked VAT keeps
  one shared material/atlas, every wardrobe mesh's UV0 is overwritten with a single texel sampled from
  the body's own `atlas.png`: the atlas pixel closest to the wardrobe material's base colour, matched
  against a 3x3 box-blur of the atlas so a single noisy/AA pixel can't win the search by chance
  (`chars_c._solid_texel`). Wardrobe meshes are tiny (24-48 verts, like `CarryLoad`) and are never
  decimated, in the full bake or the `_lod` one. A `HandTool` is added exactly as for `b_man`/`b_woman`/
  `b_child`, with its own wood/grey texels found the same automatic way (nearest-to-brown for the haft,
  least-saturated patch for the head) in the living atlas, instead of `chars_b.TOOL_UV` (hand-picked
  from a different, unrelated atlas image).
* `source_commit` reuses `chars_b.SOURCE_COMMIT`: `tools/3d/chars/README.md` documents the whole living
  pipeline (set B bodies, the shared-rig animal extension and the wardrobe attachments) as built on the
  same `v8/art-3d-chars` commit.
* `vat/<id>_atlas.png` is a byte copy of the subject's own `assets/3d/living/<dir>/atlas.png` (every
  era variant of the same body shares identical atlas bytes, verified by sha256, but still gets its own
  copy per id, matching every other output).
* `blender -b --factory-startup -P tools/3d/blender/chars_c.py -- contact [out_dir]` renders a
  full-vs-LOD contact sheet the same way as `render_b_lod_contact.py` (reusing its `render_walk_mid`,
  since that helper is already generic by id); not part of the asset build.

## W3D craft stations (`stations_w3d.py`)

Recognisable replacements for the M3D-ASSETS station primitives (`asset_smelter_*`, `asset_bloomery_*`,
`asset_forge_*`, `asset_charcoal_clamp_*`, `asset_casting_bench_*`, `asset_kiln_*`):
`w3d_<station>_idle` / `w3d_<station>_working` for smelter, bloomery, forge, casting_bench, kiln, plus
`w3d_charcoal_clamp_{idle,working,done}` (stacked wood / smoking turf mound / opened charcoal heap).
Rows go to the fragment `assets/3d/models/w3d_stations_manifest.json` (same row schema, `family: station`,
`state`); `manifest.json` is not touched. `build_all.py` runs it after the main manifest.

* Footprints: smelter, bloomery, forge, kiln 2x2; charcoal_clamp 3x3; casting_bench 2x1.
* Only `working` has `glow` (embers, fire, molten metal) and a `light` socket; smoke puffs use `snow`
  with a grey default colour. Every model is asserted to stay inside its footprint, z >= 0, <= 3000 tris.
* Reuses `stations.py` helpers (`pot`, `basket`, `log`, `stone_ring`, `anvil`); `placed(m, x, y, yaw, z, k)`
  moves/turns/scales the parts built inside it (bellows, the anvil).
* Review: `blender -b --factory-startup -P tools/3d/blender/stations_w3d.py -- contact [out_dir]` writes
  `blender_w3d_stations_<stamp>.png`: per row the replaced primitive at game zoom (FOV 30, pitch 50,
  64 px/m), the new model at the same zoom, and the new model at 2x.

## Adding a model by data

* New tree variant: add `"tree_oak_d": ("oak", 14, {"clusters": (6, 6)})` to `flora.TREES`.
* New species of tree: add a `flora.SPECIES` entry (`kind` = `broadleaf` or `conifer`).
* New rock/item/station: add a table entry (footprint, sockets, `colors`) and a builder function
  `def my_id(m, p)` that calls the `Model` primitives (`m.box/cyl/rod/ball/tube/lathe(material, ...)`);
  for stations and rocks the function is looked up by id. Keep it inside the footprint and on z >= 0.
* New animal: add a `animals.SPECIES` row (proportions, colours, `extras` such as `antlers`,
  `goat_horns`, `curl_horns`, `long_horns`, `tusks`, `bristles`, `beard`, `wool`); the generator,
  actions (graze angle is solved from the neck/head geometry) and VAT come for free.
* Rerun `build_all.py`, then `verify_vat.py` and `render_contact.py`, and look at the images.

## Tripo proof pipeline (M3D-TRIPO1)

This opt-in experiment reuses four already generated, licensed sources. Do not call
Tripo or spend credits. Keep the raw GLBs in `/Users/rexxa/work/art-img-data/tripo-test/`
and run evidence in `/Users/rexxa/work/art-img-data/tripo1/`, outside Git. Commit
processed game assets only. Run from the repository root, replacing `blender` with
the installed Blender executable when necessary:

```sh
blender -b --factory-startup -t 4 --python-exit-code 1 -P tools/3d/blender/tripo_build.py -- /Users/rexxa/work/art-img-data/tripo-test /Users/rexxa/work/art-img-data/tripo1
blender -b --factory-startup --python-exit-code 1 -P tools/3d/blender/tripo_rig.py -- woman
blender -b --factory-startup --python-exit-code 1 -P tools/3d/blender/tripo_rig.py -- goat
blender -b --factory-startup --python-exit-code 1 -P tools/3d/blender/tripo_vat.py -- woman
blender -b --factory-startup --python-exit-code 1 -P tools/3d/blender/tripo_vat.py -- goat
```

`tripo_build.py` accepts optional subject names after the two directories. It
normalizes scale/ground pivot and builds `models/tripo_<subject>_lod{0,1,2}.glb`
with baked `_albedo.png` / `_normal.png` alongside them; per-subject processing
JSON records raw counts, scale, final rows and elapsed time. Budgets are
8000/3000/1000 triangles for house and 4000/1500/500 for the other three subjects.
The house uses a 2048 texture, other LOD0/1 subjects 1024, and every LOD2 512.
These are requested budgets; processing receipts establish actual counts.

`tripo_rig.py` overwrites each woman/goat processed GLB with its shared animated
rig; optional final `0`, `1` or `2` limits work to one LOD. It welds UV seam
vertices for heat auto weights, uses the inherited limited residual-weight repair,
and transfers the existing procedural clips onto the shared rest skeleton (with
arm lowering for the woman). There are no manually authored Tripo weights.

`tripo_vat.py` has the same subject/optional-LOD arguments and reuses `chars_c` /
`chars_b`. Base ids `tripo_woman` and `tripo_goat` are LOD0; `_lod1` / `_lod2` are
explicit LOD ids and `_lod` aliases the LOD2 crowd bake. Each has
`vat/<id>_static.glb`, `<id>_vat.bin`, `<id>_vat.json`, and an albedo atlas.
The existing JSON schema records width, split vertex count, frames, clip aliases,
textured atlas and bounds; binary data is float16 RGBA position rows. Model rows
retain the existing `vat` path metadata in
`models/tripo_vat_manifest.json` (`{"models": {"<id>": <row>}}`), preserving
unrelated entries when one subject/LOD is rebuilt. Static fragment
`models/tripo_manifest.json` uses the same keyed `models` map, with each model's
footprint, bounds, triangle count, materials and sockets; processing receipts
supply those rows.

Select the replacement overlay **before** starting the existing game/capture
command:

```sh
export WORLDSIM_3D_BINDINGS=res://assets/3d/bindings3d_tripo.json
# Run the existing game/capture command only after machine-gate admission.
unset WORLDSIM_3D_BINDINGS
```

Unset is baseline. Absolute external overlay paths also work. Nested dictionaries
merge while leaves/arrays replace, so the shipped overlay explicitly includes all
baseline manifest patterns plus the two Tripo fragments. It binds a complete
`log_house`, bloomery station, neolithic adult-female variant and goat. Completed
house replacement suppresses procedural footprint walls/room roof only when the
bound GLB is available; floors and authoritative simulation geometry remain.

Limits: the imported house is one opaque mesh with no roof cutaway/interior
visibility; bloomery idle/working share one mesh and retain the odd generated tube.
Baked normal maps survive static/rigged GLB export, but the existing crowd shader
uses position VAT, flat normals and albedo, so VAT crowds do not display those
normal maps. Heat weighting and shared procedural motion still require visual
review; schema checks cannot establish rig quality. These commands and bindings
do not prove actual same-spot captures, strict Godot checks or 500-agent FPS.
Focused offline asset checks: `python3 tools/3d/blender/test_tripo.py`. This validates processed schemas/budgets, not runtime rendering.

Godot windows/renders require the existing nonblocking machine gate lock; never
launch them merely to test these commands while another pass owns the gate.
