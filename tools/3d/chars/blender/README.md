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
32/64px game-size references, contact sheet and its ordered layout JSON. Renders use
30° perspective FOV and 50° pitch; 32px is a declared reference size, not a verified
measurement of the eventual M3D-1 viewport. Copy the read-only M3D-1 `villager.glb`
to `EXTERNAL_OUTPUT/placeholder.glb` before running a human subject to include a
height-matched comparison cell.

`rig.py` uses Blender automatic heat weights on shared templates, with normalized
weights and uncovered-vertex rejection. Humans use M3D-1 villager bone names:
`hips`, `spine`, `chest`, `neck`, `head` and `upper_arm/lower_arm/hand/upper_leg/lower_leg/foot`
with `_L/_R` suffixes. Normalized A-pose bones and armature scale permit shared clips
across adult/child heights. This is a template fallback,
not a claim that a model has passed anatomical deformation review. Quadruped scale
uses the central trunk as the shoulder plane; verify it against the actual shoulder.
The posed finite-coordinate check detects numerical corruption, not bad anatomy.

Synthetic smoke test (not an AI asset, no quality acceptance):

```sh
/Applications/Blender.app/Contents/MacOS/Blender -b --factory-startup \
  -P tools/3d/chars/blender/self_test.py -- /absolute/external/test-directory
```

Blender may exit zero after a Python exception: require `M3D2_PREPARE_DONE` for each
subject and `M3D2_SYNTHETIC_SELF_TEST_PASS`, and inspect logs for `Traceback`.
