# S2 motion capture action specs

Run `tools/3d/blender/mocap_build.py <spec>` from this checkout. The build uses
the shared GPU lock for Blender. It writes the near VAT, skin and socket rows,
source/contact receipts and a contact sheet outside the checkout. It needs
Blender and Pillow. The pilot comparison also needs FFmpeg.

```sh
tools/3d/blender/mocap_build.py tools/3d/blender/mocap/pilot.ron --compare
tools/3d/blender/mocap_build.py tools/3d/blender/mocap/pick_up.ron
```

`pilot.ron` contains the two comparison clips. `pick_up.ron` runs the same
pipeline with a tag-selected bend-and-lift take. Outputs default to
`mocap1/builds/<spec name>/`. For another attempt pass a new `--out` path.

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
Use `heading` for a constant yaw correction in degrees. `reverse` plays the
same trim backward so a lift can also provide a set-down. The rig is baked at
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
| `target.kind` | `swing` aligns a working edge at contact. `circle` traces a level circle. `reach` reaches a point and supports picking up or setting down a prop. `free` keeps the captured pose. |
| `target.anchor` | The swing strike point, pot rim centre or reach point. Stirring starts at the rim plane and applies the depth offset. |
| `target.radius`, `target.depth` | Circle radius and depth below the rim in metres (depth defaults to 0.065). |
| `target.tip` | Working point in prop-local metres. |
| `target.orientation` | Direction of the prop's local +Z axis at the target. |
| `fixture` and `target.fixture_radius` | Optional reference geometry for captures and its size in metres. |
| `target.solid` | Keep the prop above a strike surface within `fixture_radius`. |

To use a capture again, copy its source selection and trim to another visual
key. Reuse the swing with a different edge or strike point, or change the
reach anchor and switch `reverse` for putting an object down.

The `sockets` rows are the two skin anchors in right/left order. `props` rows
hold the shared tool transform, which can remain at the anchor before pickup.
