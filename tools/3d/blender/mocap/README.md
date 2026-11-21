# S2 motion capture action specs

Run `tools/3d/blender/mocap_build.py <spec>` from this checkout. The build uses
the shared GPU lock for Blender. It writes the near VAT, skin and socket rows,
source/contact receipts and a contact sheet outside the checkout. It needs
Blender and Pillow. The pilot comparison also needs FFmpeg.

```sh
tools/3d/blender/mocap_build.py tools/3d/blender/mocap/pilot.ron --compare
tools/3d/blender/mocap_build.py tools/3d/blender/mocap/pick_up.ron
tools/3d/blender/mocap_build.py tools/3d/blender/mocap/remaining.ron --review-slice
```

`pilot.ron` contains the two comparison clips. `pick_up.ron` runs the same
pipeline with a tag-selected bend-and-lift take. `remaining.ron` adds work,
social and carry poses. `walk_reference.ron` builds the review-only walk.
Pass several spec paths to build one pack with unique action keys. Outputs
default to `mocap1/builds/<first spec name>/`. Use a new `--out` for another
attempt. `--review-slice` captures both camera angles at the review pose and at
the measured detail frames. Use `--bake-only` when captures are unnecessary.

## Select and trim a take

Each top-level key is an ACTION1 visual key. Use the RON map subset with quoted
keys, strings, arrays, finite decimal numbers and booleans. These files also
parse as JSON. Omit optional properties. The source index and the cache live
outside the checkout:

    mocap1/index.json
    mocap1/raw/<take_id>.bvh

Search descriptions and indexed tags with `tags`. The selector ranks by the
number of matched phrases and breaks ties by take ID among downloaded clips.
Add an ordered `takes` array to pin the selection to known captures. If no
suitable clip is cached, the error lists candidate IDs and URLs. Download the
licensed take into `raw/` and retry. Keep the original author and licence in
the index and source record. An indexed SHA-256, when present, is checked
before loading the BVH.

`trim` gives the start and end in seconds on the original capture clock.
Divide zero-based BVH sample indices by `source_fps` from the index. Some
converted BVHs carry a different playback rate in their header. A trim starting
at zero begins on the first captured frame, skipping the calibration pose.
`loop` selects seam wrapping or a clamped one-shot. `speed` is a per-body playback multiplier
(default 1.0, p4 0.85). `stoop` adds forward bend in degrees (default p4 7).
Set `duration` in seconds when the output differs from the trim length. Set
`retime` to pairs of normalized playback and source phases. Both axes must
increase from `[0, 0]` to `[1, 1]`.
Use `heading` for a constant yaw correction in degrees. `reverse` plays the
same trim backward. `in_place` removes net root travel from a take with moving feet.
The rig is baked at
one unit height across the four mesh shapes. Review renders use the S2 scale
factors 1.00, 0.95, 0.70 and 0.97 for p1 through p4, including their props.

## Place the action

Positions and offsets use Blender metres with +Z up, −Y forward and +X toward
the villager's left. `posture` translates the whole retargeted body. Enable
`foot_lock` for a planted stance. Ordered `contacts` are phases from 0 to 1 in
output playback order.

| Field | Meaning |
| --- | --- |
| `prop` | A prop mesh identifier. Omit it for an empty hand. |
| `hands` | Map of `R` and/or `L` grips. Use `target: "prop"` for a prop-local offset or `target: "anchor"` for a world-aligned offset from the fixture. Omit a hand to preserve its captured motion. |
| `slide` | Optional phase and distance pairs for a grip sliding along local +Z of a prop, added to the static offset. |
| `target.kind` | `swing` aligns a working edge at contact. `circle` traces a level circle. `reach` reaches a point. `carry` fits an upright prop between the hands. `free` keeps the captured pose. |
| `target.anchor` | The swing strike point, pot rim centre or reach point. Stirring starts at the rim plane and applies the depth offset. |
| `target.radius`, `target.depth` | Circle radius and depth below the rim in metres (depth defaults to 0.065). |
| `target.tip` | Working point in prop-local metres. |
| `target.orientation` | Direction of the prop's local +Z axis at the target. |
| `fixture` and `target.fixture_radius` | Optional reference geometry for captures and its size in metres. |
| `target.solid` | Keep the prop above a strike surface within `fixture_radius`. |
| `target.release` | For `reach`, leave the prop at the anchor after contact. |
| `impact_window` | Approach and release widths in playback phase units. |
| `target.release_offset`, `target.release_pitch` | Optional blade-tip movement in metres and upward pitch in degrees after impact. |
| `windup` | Phase and world-space prop offset at the swing apex. |
| `face_clearance`, `rim_clearance`, `rebound_clearance` | Minimum distances in metres at the swing apex, above a stir fixture and after impact. |

For a layered pose, give `base_layer` a take or search tags and its own `trim`
and `heading`. `weight` multiplies each per-bone value in `mask`. The `root`
channel blends hip displacement. Bone channels blend parent-local rotations.
Use matching hip, leg and foot weights for a planted kneel or squat. A trim with
equal endpoints holds a pose. Props and contact constraints run on the blended pose.

`arm_limits` accepts per-body `upper_roll`, `lower_roll` and `elevation` in
degrees from horizontal. `elbow_flexion` limits how far the elbow can fold from
a straight arm. `clavicle_lift` in metres and `spine_extension` in
degrees support an overhead reach. For these actions, the bone pivots follow the
scanned A pose and a local surface correction holds the palms fixed. The build
checks shoulder and elbow edge stretch (at most 1.35×) and triangle area (at least
0.35×) against the prepared mesh in both the baked poses and the decoded VAT.
Every mocap action also checks that body edges at least 2 mm long stay within
3× their rest length. The contacts receipt records the worst frame for each
check.

To use a capture again, copy its source selection and trim to another visual
key. Reuse the swing with a different edge or strike point, or change the
reach anchor and switch `reverse` for putting an object down.

The `sockets` rows are the two skin anchors in right/left order. `props` rows
hold the shared tool transform, which can remain at the anchor before pickup.
