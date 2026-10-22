# TRIAL3B render log

Owner amendment: `TRIAL3B_AMENDMENT_20261022.md`. All render data is external under
`/Users/rexxa/work/arttrial3-data/b/render/`. No generated art or paid prompts.

## Render contract and implementation

- Blender 4.5.14 LTS, Cycles CPU 12 samples, denoising off, transparent film.
- `art/pipeline3/render_b.py` opens the preserved `model.blend`; one `look`
  invocation renders exactly one idle S still. A separate `layers` stage requires
  a `frozen_look.json` with the reviewed look number. Ledger prevents repeats.
- Upright 256×384, orthographic 20° down. S front, E camera azimuth −90°
  (equivalent model +90°), N180°. Projected source BODY geometry sets 56 game-pixel
  height and bottom y88. Idle calibration is reused unchanged through animation.
- Sleep384×256, long axis56 game pixels, midpoint(48,36); source sleep action
  determines head-left geometry. Final RGBA uses Lanczos to64×96 or96×64.
- Look1: Freestyle silhouette/border/crease/material edges, `#141210`,
  2 authoring pixels, 125° crease angle, neutral diffuse surface.
- Look2: same ink; three constant normal/light bands using source colors from
  `art/style05/palette_05.json`; no output color quantization.
- Look3: same bands/ink plus Blender compositor Clouds texture multiplied at8%;
  alpha restored from original render pass.
- Layer membership comes from source object `layer`. Region masks use native
  unfiltered IndexMA. Anatomical masks use native IndexOB assigned from source
  object `anatomical_id` (retained mesh partitions). Source IDs are nearest-resized;
  rendered-ink/resampling support inherits neighboring IDs only within actual
  alpha support, bounded to3 game pixels for material IDs and1 for anatomy.
  Disconnected unsupported islands stay0 and fail the checker. This affects masks
  only: RGBA is never painted, patched, copy-backed, or alpha-repaired.
- No contact shadow is rendered. The lab supplies the template ellipse.

## Setup evidence

| Look | Idle evidence | Decision |
|---|---|---|
|1|`render/look1_idle_S_{4x,1x}.png`|Parent reviewed; numeric/model judge FAIL, scores1/2/2/1|
|2|`render/look2_idle_S_{4x,1x}.png`|Parent reviewed; numeric/model judge FAIL, scores2/2/3/3|
|3|`render/look3_idle_S_{4x,1x}.png`|Parent reviewed; judge FAIL2.5; frozen for diagnostic full render|

All three setup slots are used; no further shader/camera setup is permitted.

Look1 reading: broad short-limbed toy-like figure; outline readable, grey block
wrap and shoe seams remain; no claimed05 style acceptance. Model source SHA,
Blender version, render duration and zero credits are in `render_ledger.json`.

Before any image render, source scene lacked World and Freestyle LineStyle
objects; initialization now creates those missing render-owned objects. Both
initial API failures happened before the ledger/render and are not look trials.
No geometry was edited by the renderer.

## Verification

- Blender native scene node initialization succeeds for all three-look features.
- All three actual Blender idle renders completed and inspected at4×, with
  parent inspection/judge between looks. Look2 adds warm material separation;
  look3 adds subtle paper texture but neither repairs blank face/rigid geometry.
- Ruff syntax/undefined-name checks pass on all five renderer modules.
- Five mask-support boundary tests pass (`b_render_test.py`), including
  disconnected islands and the one-pixel anatomical fringe bound.
- Full34layers rendered:16bodies (5upright frames×3directions+1sleep),6tops,
  6hair,3axes,3log bundles. Full beauty+native-ID render took2.231s according to
  `render_ledger.json`; three look renders total0.338s. These are rendering times,
  not total modeling/engineering effort.
- Native source mask values inspected: BODY IndexMA0/1/5; top0/3; BODY IndexOB0..10.
- Final bounded-mask checker at `../checks/CHECKS.json` reports **FAIL0/34**,
  no accepted combinations; minimum inventory passes. Typical failures include
  disconnected Lanczos alpha support without native mask evidence, projected
  limbs1–3px below4px, silhouette holes, top coverage/extent, and fringe bounds.
  No image pixels were repaired to eliminate those failures.
- Sleep actual image inspected: head-left horizontal body, no upright overlays.
  East work actual image inspected: narrow side silhouette and occluded limbs
  remain visible limitations.
- Basedpyright on finalizer/tests reports0errors with numpy-derived dynamic-type
  warnings; full strict Blender-RNA static typing is not claimed. Blender-native
  execution verifies API behavior, not all static type contracts.
- Carry anchor review found torso bone tail is not the authored shoulder contact;
  source author subsequently supplied its exact bind-space point(0,.12,.72).
  The leader authorized a metadata-only correction: transform that point by
  torso pose×inverse rest×rig world, project through the unchanged camera.
  Actual anchors are S(32,46), E(25,48), N(32,51), without snapping to support.
  The original wrong-anchor metadata is preserved under `render/history/`.
  Final root `b/checks/CHECKS.json` was regenerated after the correction;
  all119 recorded input hashes match the frozen inputs.
- GIF readability and style final aggregation belong to the independent lab/report
  stages; no accepted art or viable route is claimed by this renderer.

API references: [Freestyle settings](https://docs.blender.org/api/3.6/bpy.types.FreestyleSettings.html),
[Blender manual/API entry](https://www.blender.org/support/).
