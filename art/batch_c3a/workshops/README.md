# C3A iron workshop compositions

These are proposed **independent layers**, not full-room runtime sprites.
`rooms/*/manifest.json` records every layer's world origin, source anchor and
SHA256. Room previews and contact sheets are visual evidence only. Each room is
4×4 tiles with an EW door at(2,3), drystone/granite walls and packed-earth floor.
The working source is the existing A2 lit bloomery/working forge, pinned to
`227529c26307bc3a854eb7bf819e88d02af39046`.

`python -m art.batch_c3a.workshops.build` regenerates all outputs using the
existing art Python dependencies. Raw images and generation records stay under
`/Users/rexxa/work/art-batch/C3A`. Every station uses only generated alpha crop,
uniform Lanczos scaling and transparent padding. Station metadata records raw
prompt/hash and unchanged frozen numeric gate. Failure is held, never exported
as a selected station. Review is still required for numeric passes.

The 12 room recipes cover smithy/bloomery_house foundation, half-built, built
idle, built working, damaged and ruined. Foundation exposes24px wall courses;
full courses60px. This is the same B3 exact top/face crop, not shrinking a built
building or fading it. Damaged loses southern upper courses; ruined retains
only northern low courses. Construction lifecycle names remain proposed art
selectors; C3 integration must bind only simulation states that actually exist.
Blueprints are the existing B3 display operation, not extra generated PNGs.

The native contact sheet uses256px room width (4×64) and a separate56px adult
scale reference. The doubled sheet uses nearest-neighbour2×. The adult is not
part of any room or import layer.

`material_sources.json` pins material inputs from the art-tree baseline. Each
A2 prop layer is pinned by source path and hash. The room-level source_commit
identifies A2 props, not current-tree material inputs. Do not flatten previews,
rescale layers, assume modern-anvil geometry, or submit missing anvil PNGs.

Known visual limitation: inherited NS top surfaces are broad/blurred compared
with EW faces. Working glow is supplied by the original A2 state and subtle at
native scale. No Godot import, runtime state selector or engine gate was run.

## Pass1 anvil gap (historical)

No anvil attempt passed both unchanged gates. Try1 failed wash_flatness, try2
failed edge_density, and try3 passed all8 style metrics but failed the existing
normalized global-light projection (176.39°, positive y gradient). Geometry
height28px and anchor error0 passed all three. At the3-attempt cap, anvil stays
held: no anvil PNG or importer row is submitted, and production smithy layers
omit it. The final try3 normalized image remains external in
`C3A/candidates/workshops/anvil_try3.png` for review. The standalone bench passes
all8 style metrics and projection. A smithy shell/forge recipe is therefore
available; a complete forge-plus-anvil production template still has this gap.

## Pass2 approved anvil

BRIEF4 grants the anvil up to three more attempts. Cumulative try4 (first new
attempt) passes all8 unchanged style metrics and normalized projection
(height28px, base error0, light247.33degrees). Independent visual review passes
at native and nearest2x beside the adult and forge, on grey/green alpha mattes.
The raw generation includes a warm glow outside the object; the unchanged
opaque-bounds+2px-fringe crop leaves no conspicuous halo in the normalized
output. No alpha editing or gate change was used. The anvil PNG and built
idle/working smithy layers are now included. Other stages follow the existing
state recipe. Remaining native stone-wall replacement is delivered separately
under ../walls_pass2; these historical room recipes still use the legacy wall
kit and are not a runtime wall-selector update.
