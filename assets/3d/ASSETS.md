# M3D-ASSETS delivery

Static delivery: 572 additional GLBs (634 including the original 62).

This pack extends M3D-1; it does not change the simulation or the renderer. The
before-work inventory is [INVENTORY.md](INVENTORY.md). Its requested remote C3
ref was unavailable; the baseline and provisional local C3 SHA are recorded in
[INVENTORY.json](INVENTORY.json). This distinction is intentional.

## Static models

Run from the repository root with Blender installed:

```sh
blender -b --factory-startup -P tools/3d/blender/asset_stations.py
blender -b --factory-startup -P tools/3d/blender/asset_items.py
blender -b --factory-startup -P tools/3d/blender/asset_blueprints.py
blender -b --factory-startup -P tools/3d/blender/content_flora.py
blender -b --factory-startup -P tools/3d/blender/asset_buildings.py
python3 tools/3d/blender/merge_assets.py \
  assets/3d/models/asset_stations_manifest.json \
  assets/3d/models/asset_items_manifest.json \
  assets/3d/models/asset_blueprints_manifest.json \
  assets/3d/flora_manifest.json assets/3d/buildings_manifest.json
python3 tools/3d/blender/test_asset_static.py
python3 tools/3d/blender/verify_assets.py assets/3d/assets_manifest.json
```

`assets_manifest.json` is the combined lookup including original M3D-1 entries;
`models/manifest.json` remains untouched. `merge_assets.py` rejects duplicate
model IDs and unresolved blueprint references. Pass the living fragment too when
assembling the complete pack; the living pipeline has a separate reproduction
procedure under `tools/3d/chars/README.md`.

The fragments expose:

- Item `item_code`, `content_id`, `stack_display_count` (1, 3, 6): each small,
  medium and large mesh depicts that many objects. The renderer selects by its
  actual count; these are display thresholds, not a change to stack capacity.
- Station `state`; joining fence `connections` with north/east/south/west bits.
- Building `template`, `stage`, `source_status`, `piece_count`, `roof_style`:
  foundation/half/complete are display previews, not new simulation enum values.
  `inert_concept` means an art concept for an unconnected template, not proof of
  an executable building or a newly available simulation feature.
- Plant/crop `content_family`, `content_id`, `phase`; material-specific outcrops.
- Blueprint code mapping in `asset_blueprints_manifest.json`; shared models are
  explicitly referenced rather than relabelled as unique geometry.

For units, axes, palette, socket semantics and the original skeleton/VAT API see
[the Blender README](../../tools/3d/blender/README.md). Some stack meshes exceed
M3D-1's single-item triangle recommendation; actual counts are in the manifest.

## Review camera

```sh
blender -b --factory-startup -P tools/3d/blender/render_assets_contact.py -- \
  /absolute/external/review-directory --manifest=assets/3d/assets_manifest.json
```

Use the renderer's documented argument order if selecting individual fragments.
Sheets compare M3D-1 at 1×/2× with the delivered model at 1×/2×: perspective FOV
30°, pitch 50°, ground scale 64/128 px per metre. These are asset review renders;
they do not claim a game scene, runtime state selection, performance acceptance,
or a full simulation publication gate. Captures, failed attempts, detailed
checks and the Korean REPORT.md live outside the repository under
`/Users/rexxa/work/3d-data/m3d-assets/`.
