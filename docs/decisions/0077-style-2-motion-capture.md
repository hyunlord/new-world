# 0077. Style 2 motion capture

## Context

The CLIPS1 chop and stir previews lacked anticipation, weight transfer and
recovery. The same action set must serve four body shapes and let authors
reuse a take with a new tool or contact point.

## Decision

Index the CMU Graphics Lab Motion Capture Database and author each action with
the [MOCAP1 specs](../../tools/3d/blender/mocap/README.md). Preserve row zero
for calibration. Trim, layer, reverse or remap source time in the spec. Fit
both hands and the tool together, and pin loaded feet in the ground frame.
Bake the S2 rig and grip transforms at 30 Hz. Keep the source takes outside
Git and record their licensing and attribution in the MOCAP1 source record.

Run the [playbook](../../tools/3d/blender/mocap/PLAYBOOK.md) checks before
packing the reviewed actions. Measure balance from segment centres of mass
and physical contacts. Measure garment stretch and hand clearance on the
decoded VAT as well as checking the post-IK skeleton. Keep grip positions and
physical exceptions in the spec. Export those skeleton curves for offline
consumers and reserve a post-filter stage.

Use the new VAT only for the near tier of `--style=s2`. The mid and far
tiers use their existing 15 Hz VATs. Walk and loaded carry advance with
world distance. The near player clamps transfer clips at their terminal
pose and samples tool transforms on the same clock as the corresponding
VAT. Grounded baskets remain at their baked anchor through the transfer.
Write near poses and props only for bins inside the near tier's range.
Seed their current poses on entry and advance transfer timers across tiers.

The branch derives presentation from read-only agent, activity and
workshop snapshots. It starts transfer clips on stock-job cargo edges and
interrupts them when the walker starts another step. The basket is the
transfer prop and the walking clip uses logs for wood. Cooking uses active
food bills at a hearth and places the matching clay vessel above the
existing fire. Recognized work displays a baked local fixture.

## Alternatives considered

Hand animation would require work for each action and body. Runtime
skeletal IK adds animation and terrain queries to crowd rendering. PHYS1
tested offline physics tracking and its output did not improve the motion.

## Consequences

A new action can reuse a capture, layer, grip and working surface through
data. The rig, contact constraints and validation run offline. The raw takes,
review decisions, per-frame measurements and retarget curves remain under
`mocap1/` beside the checkout.

## Date

2026-11-21

## Source

`BRIEF_MOCAP1.md`, D-MOCAP1-2 through D-MOCAP1-5 and the
[CMU motion capture database](https://mocap.cs.cmu.edu/).
