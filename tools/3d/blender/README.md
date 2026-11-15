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
python3 tools/3d/blender/verify_vat.py                                 # VAT + glb checks, exit 1 on failure
blender -b --factory-startup -P tools/3d/blender/render_contact.py -- [out_dir] [id filters...]
blender -b --factory-startup -P tools/3d/blender/render_b_lod_contact.py -- [out_dir]  # set B full vs LOD
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
  scaled to ~0 outside `carry`), so the bundle appears only in `carry`.
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
* the humans' 24-vertex `CarryLoad` bundle is kept at full resolution (already tiny; decimating it
  further would risk breaking its shape for no real savings) and still becomes extra VAT columns, as
  in the full bake.
* budgets are asserted at bake time: width (VAT columns) <= 400, `split_vertex_count` (static mesh,
  one vertex per loop) <= 900; actual sizes are man 399/744, woman 400/759, child 400/765, goat
  400/792, aurochs 400/816 (width/split).
* the manifest gets a `b_<subject>_lod` entry alongside `b_<subject>`, same shape (family, animations,
  bones, `vat`), so the game loads it exactly like the full character.

`verify_vat.py` needs no changes: it drives its checks off every `family == "character"` manifest
entry, so `b_<subject>_lod` is checked the same way as `b_<subject>` automatically. Likewise
`build_all.py` needs no changes: `chars_b.build()` now bakes the full and the LOD for every subject.

`render_b_lod_contact.py` renders a full-vs-LOD contact sheet (each character re-posed from its VAT at
the middle of `walk`, both columns textured from the shared atlas) for visual review; it is not part of
the asset build.

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
