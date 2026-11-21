# S2 motion capture playbook

Author each action as a visual key in a [RON spec](README.md). Run the [build](../mocap_build.py) with the shared GPU lock and keep each attempt’s evidence outside the checkout.

1. **Vet one intent per take.** Trim one complete action, including any anticipation and recovery. Reject step-overs, stumbles, turns in place and extra foot swaps unless they are intended in the spec. Write the take, window, reason, URL, licence and attribution in `mocap1/SOURCES.md`.
   - **Why** Extra movements can survive a trim and turn a stationary task into an unsupported pose.
   - **Check** Select by `tags` or pin `takes`, use indexed source Hz for `trim`, keep row 0 for calibration, and inspect the full timeline. Declare intended steps under `plausibility.steps`.
2. **Measure the rest pose and distribute twist.** Compare row-0 shoulder–elbow, elbow–wrist, hip–knee and knee–ankle vectors with the S2 bind directions. Use the measured directions for alignment, including for captures with inclined arms. Distribute axial twist across the upper and lower arm before solving grip positions.
   - **Why** A rest offset or concentrated twist makes a smooth source motion fold or stretch the rig.
   - **Check** [`Capture`](../mocap_pose.py) uses row-0 rotation deltas for every bone and measures hip-to-ankle leg scale. `_align` measures arm directions for actions with `arm_limits` or `base_layer`; review bind-axis alignment for the legs and the legacy arm path. For authored grips with `arm_limits`, `ArmChain.from_capture` splits and caps roll before the palm solve. Keep joint ranges in `plausibility_rig.ron` and grip limits in per-body `arm_limits`.
3. **Declare each contact.** Set `hands` and grip offsets or `slide`, plus tool `target`, `contacts` and `impact_window` for the working tip or edge. Use `target.release` for a one-shot set-down. Declare `plausibility.steps` and `plausibility.plants` by `R`/`L` and `plausibility.support` by `hand_R`/`hand_L` or `knee_R`/`knee_L`. Use `foot_lock` for a fully planted stance. Solve the hand and tool constraints after sharing arm twist, preserving the captured roll within the authored limits.
   - **Why** Explicit timing lets the bake distinguish a grip, planted sole or weight-bearing knee from an incidental pose.
   - **Check** The [pose bake](../mocap_pose.py) applies the body layer and foot lock before constraining the props and grips. The plausibility gate verifies declared hand and knee ground support and measures sole height and slide. The [schema](../mocap_spec.py) takes ordered, closed windows in normalized output phase. Split windows that cross the seam and keep steps and plants separate on each foot.
4. **Reuse approved motion.** Try the same `takes` and `trim` first. Use `base_layer.mask` with `base_layer.weight` for hips, legs and feet, `retime` for phase changes and `reverse` when the action runs backward. Adapt prop, grip and target geometry in the new spec.
   - **Why** Reuse preserves proven weight transfer and keeps new actions authorable in data.
   - **Check** Compare the shared axe and mallet swing in [chop](pilot.ron) and [hammer/build](stake_build.ron), and the reversed bend and grounded release in [set-down](set_down.ron). Review the new contacts on all four bodies.
5. **Size props to the body and the task.** Use the S2 palette and round-bottomed clay, wood and stone geometry suited to the action. Record rim, handle and stake heights for each body class where relevant. Place the working edge with `target.anchor`, `target.tip`, and the authored grip offsets.
   - **Why** A reachable grip can still read out of scale or put the working end above the surface.
   - **Check** Set `plausibility.prop_ratio` to the allowed range for the largest local bounding-box dimension divided by normalized rig height. Check the working heights in the captures at p1–p4 scale factors 1.00, 0.95, 0.70 and 0.97.
6. **Review the rounds in order.** Start with the chop/stir pilot against CLIPS1, side and ¾ views with six samples per clip and body, plus p1 old/new GIFs at 30 fps. Resolve pilot fixes with close-ups and get approval before completing the action set. Then show every motion across p1–p4 in both views at matched scale. For fixes, include exact contact for all bodies in both views, a 12-frame p1 strip in both views, and close-ups of the requested details. Get final approval before gameplay wiring.
   - **Why** Contact frames reveal placement errors and full strips reveal problems between the measured poses.
   - **Check** Use `--compare`, `--review-slice` and `--review-strip` with the [build](../mocap_build.py), and `final` or `revision` with the [review compositor](../mocap_review.py). Publish `REVIEW-REQUEST`, wait in one shell loop for `REVIEW-OK` or `REVIEW-FIX`, and archive the response after reading. Gate the final bake before packing.

## Bake gate

Check every motion and body. The post-IK contact track feeds the plausibility checks and exports per-clip, per-body `.npz` curves at 30 Hz with local joint rotations, root transforms, bone names and rest offsets. Leave `post_filter` unset for this pipeline. The reserved hook sits after retarget and contact IK and before VAT.

| Check | Required result |
| --- | --- |
| Balance and foot lift | Project segment-weighted CoM into the support polygon. Allow at most 3 cm outside during `dynamic` and unsupported frames only during `airborne`. A sole more than 5 cm above ground requires its `steps` window. |
| Lean and joint limits | Pelvis-to-neck lean stays within `lean_limit` (15° by default) outside `work`. Check joint angles against per-rig data. |
| Penetration and face | Hands and forearms against the torso and head, and the opposite thigh against the torso, stay within 1 cm of capsule penetration. Keep hands 4 cm clear of the face outside `face`. |
| Garment, planted feet, prop | Whole-body edge stretch stays within 3×, sole slide within 1 cm during `plants`, and prop size within `prop_ratio`. |

Use the recorded body, frame and measured breach to fix the source window, retarget or contact geometry, and re-run the bake. If a previously approved motion fails, preserve the original evidence and report the frame, measured value and proposed fix before accepting the revision. Use exception windows only for explicitly authored movement and keep the gate limits fixed.
