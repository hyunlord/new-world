# M3D-2a Blender stage

Run with Blender's embedded Python (no added dependencies):

```sh
/Applications/Blender.app/Contents/MacOS/Blender -b --factory-startup \
  -P tools/3d/chars/blender/prepare.py -- RAW.glb EXTERNAL_OUTPUT man
```

Subjects: `man`, `woman`, `child`, `goat`, `aurochs`. Raw inputs are never overwritten.
The imported input must be upright, Blender Z-up/front -Y. Inspect and correct a
misoriented source before using this stage; do not infer an animal's front from its
bounding box. Export matches M3D-1: Godot Y-up/front +Z, ground-centred origin, metres.

Outputs: textured GLB with one embedded 512×512 atlas, atlas PNG, receipt with
triangle count and skeleton names, four turntable frames, a frame from every clip,
32/64px diagnostic thumbnails, contact sheet and its ordered layout JSON. Renders use
30° perspective FOV and 50° pitch; these thumbnails are not game-size evidence.
Use compare.py below for actual camera projection and the read-only M3D-1 placeholder.
Use `--force-remesh` when direct decimation damages the silhouette despite meeting
the count budget; inspect the raw and retain both attempts. Every output directory must be new; failed and previous attempts remain intact.

`rig.py` uses Blender automatic heat weights on shared templates, with normalized
weights, rejection above1% uncovered, and explicitly recorded nearest-bone repair
for residual vertices at or below1%. Humans use M3D-1 villager bone names:
`hips`, `spine`, `chest`, `neck`, `head` and `upper_arm/lower_arm/hand/upper_leg/lower_leg/foot`
with `_L/_R` suffixes. Shared A-pose proportions have height-specific rest lengths,
with all object transforms applied. Rotation curves transfer directly;
`rig.shared_action` scales root displacement by target/source height. Resting
poses additionally need target-mesh ground alignment: the actual B adult-to-child
transfer penetrated4.33cm before this correction; `clips` performs per-model
evaluated grounding for final sleep/lie-down poses. This is a template fallback,
not a claim that a model has passed anatomical deformation review. Quadruped scale
uses the highest central-width surface point in a front-trunk withers band,
40% of nose-to-tail length from the front (±4% length, central40% width).
The receipt records its measured XYZ coordinate and full rest bounds; verify the
landmark against the actual shoulder in the side views. For swept horns overlapping
the default band, `--narrow-withers` selects central10% width; this trial option
still requires an external side-view landmark marker review.
The posed finite-coordinate check detects numerical corruption, not bad anatomy.

Humans include a 12-triangle packed load that follows the hands during `carry`;
its object scale is 1e-6 in every other clip and in the default pose (visually hidden,
non-singular for Godot imports). Constant
object channels must be retained during NLA export (`export_optimize_animation_keep_anim_object=True`),
otherwise Blender drops visibility scales despite correct studio renders. The
self-test reads exported binary scale samples, rather than trusting clip names.

True game-size A/B comparison uses the current read-only M3D-1 camera contract:

```sh
blender -b --factory-startup -P tools/3d/chars/blender/compare.py -- \
  A/man.glb B/man.glb /Users/rexxa/work/new-world-3d/assets/3d/models/villager.glb \
  man /absolute/external/comparison
```

This renders 1920×1080 at 30° vertical FOV, 50° pitch and the game's distance formula,
asserts a ground metre projects to 64/128px at 1×/2×, and crops fixed native pixels.
The comparison receipt binds the source camera parameters and model hashes.
Lighting approximates Godot; this does not claim renderer parity. Children use the
M3D-1 placeholder's actual `.62` scale, rather than silently matching candidate height.

Synthetic smoke test (not an AI asset, no quality acceptance):

```sh
/Applications/Blender.app/Contents/MacOS/Blender -b --factory-startup \
  -P tools/3d/chars/blender/self_test.py -- /absolute/external/test-directory
```

Blender may exit zero after a Python exception: require `M3D2_PREPARE_DONE` for each
subject and `M3D2_SYNTHETIC_SELF_TEST_PASS`, and inspect logs for `Traceback`.
