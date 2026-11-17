# L2 — buildings and props

Presentation-only art pass on `v8/look2`, based on `origin/v8/main` (`fd3f59801`).
The shared art-direction image is `/Users/rexxa/work/art-img-data/look1/TARGET.png`;
source prompts are in that directory's `manifest.json`. The brief and raw evidence
live in `/Users/rexxa/work/art-img-data/look2/`.

## Runtime boundary

A complete `building_*` GLB is an authored preview, not the live room renderer.
The live view must keep drawing actual room tiles, partial destruction and doors
from SimBridge snapshots. Detailed plinths, frames and roof trim therefore share
the existing chunk mesh and cutaway shader. No simulation, save or rig changes
are part of L2.

Old model IDs, material names, footprints, work/light sockets and NESW fence masks
remain integration contracts. Foundation/half/complete IDs are retained; the new
frame previews are inserted between foundation and half in presentation bindings.
Station state candidates still choose the existing `w3d_*` models first.

## Reproduction and review

Use the existing Blender executable in background mode for mesh export; rendering
requires holding `tools/v8/mux2_gate_lock.py -- <command>`, not a standalone check.
The canonical lock check requires an inherited descriptor. Nonblocking failures
are retained, with retries separated by five minutes.

The comparison baseline is the complete frozen `46894acf3` archive, containing
the UI2 merge but no L2 model changes. Same-ID model comparison avoids the old
contact renderer's unrelated hearth fallback. Review sheets use FOV 30°, pitch
50°, fixed physical scale; illustration targets are explicitly labelled as such.
Actual game captures remain necessary because Blender sheets do not exercise
Godot materials, shadows, picking or cutaway.

The 500-agent and 10,000 render-only floors require real windowed measurements,
identical binaries/configuration, one measuring process and admission load <8.
Headless frame rates cannot establish these floors. External Korean `REPORT.md`
records exact commands, source identity, checks, screenshots and outstanding
integration or machine-admission limits; it is the delivery receipt.

## Rebuild order

```sh
blender -b --factory-startup -P tools/3d/blender/look2_props.py
blender -b --factory-startup -P tools/3d/blender/asset_items.py
blender -b --factory-startup -P tools/3d/blender/asset_buildings.py
python3 tools/3d/blender/merge_assets.py assets/3d/models/asset_stations_manifest.json assets/3d/models/asset_items_manifest.json assets/3d/models/asset_blueprints_manifest.json assets/3d/flora_manifest.json assets/3d/buildings_manifest.json assets/3d/living/manifest.json
```

The props generator updates 42 baseline manifest entries, including five appended
yard identities. Its separate fragment is a review inventory; do not pass it to
the aggregate merger because the baseline already contains those rows. The
original first-wins runtime manifest order remains unchanged.

Focused checks: `look2_building_test.py`, `test_look2_props.py`,
`test_asset_static.py`, `test_fence_directions.py`, followed by Godot import and
`look2_data_test.gd`, `look2_structures_test.gd`, the existing data/render tests
and `world3d_texture_test.gd`. For comparisons use `look2_contact.py --check`
before its gated Blender render command. New yard assets need a snapshot-driven
yard consumer from the M-GROW/polish integration; assets alone do not prove
placement.

## Integrated material boundary

L1 `d728ad708`, T2 `4085470bf` and UI2 `83d1c7e4e` are normal-merged.
`Data3D` preserves source materials and imported LODs; structures and props adapt
copies through `Textures3D.mesh`. Flora and plants consume source colours through
their seasonal and wind shaders. Tinted prop copies retain LODs and choose the
requested metal atlas tile after the model-specific override. Shared building
roofs retain L2 trim, gables, room ownership, cutaway and picking. Roof-plane
regressions inspect complete half-metre sheet coverage separately from trim.

## GPU review corrections

The first full contact review exposed building shells collapsed by interior
placement: BMesh wrapper identity is not stable across builder operations.
Tag existing vertices with an integer layer and remove it only after transforms;
the seven-builder regression checks every old coordinate. Roof frames now meet
wall plates; bounded thatch fringes and filled hay cores improve silhouettes.
Legacy furnaces reuse the detailed state builders; bench details stay on the
matching tabletop. Re-export remains deterministic and within unchanged budgets
(buildings: 6,404 triangles and nine materials maximum).

A real Metal capture exposed a shared shader argument named `albedo` resolving
to the plants material's vec4 uniform. Rename that argument and check all seven
consumer namespaces; headless import alone cannot prove GPU shader compilation.
Failed captures, first contact reviews and their hashes remain in external
`after-failed-compile`, `triptychs-failed-compile`, `contacts-v1-rejected` archives.
