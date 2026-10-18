# P12 activity and crowd capture protocol

Frozen before captures. GOAL6.6b/c/d/f is authoritative. Synthetic tests are tooling evidence only. Art Pass11 body-only results remain historical failures; no body-only re-test.

## Sources and order

Use game_v25, live seed20261010 at displayday30 10:00 and15:00, then seed20261018 at displayday60 10:00. Capture each at Z4 andZ1. Fresh reset and exact tick from world_info ticks_per_day/clock_offset. Keep raw PNG, full sidecar, launch logs, source/build hashes and failed launches. P12 capture copies retain the P10 oracle-metadata activation and full world sprite ordering; schema3 adds icon geometry. Existing independent pose oracle must pass.

For each activity kind0..6 select the first two arrived holders (until_tick>=0), ordered by AgentId within captures in the fixed order above. Deduplicate (seed,AgentId,kind), as in the audited P11 planner. If insufficient, retain exclusions-before-crops.json and add only same seed/day at11:00 or14:00, centred on the lowest-AgentId arrived holder of the recorded missing kind. Supplement runner saves the missing-kind plan and preregistration before launch. No stress worlds or arbitrary replacement seeds.

## Geometry and exclusions

Body rect is the actual transformed Sprite2D rect. A visible icon uses its actual transformed rect and independent full draw rank. Hidden/absent icons use head_icon_position(sprite.position), the renderer ICON_SIZE_PX footprint (16x16 worldpx), bottom-centred at that anchor, transformed by the sprite parent's canvas transform. Hidden fallback uses holder body draw rank and explicitly records that fact. No hidden image is synthesized.

For each footprint, union the intersections of rectangles drawn above its own rank. Overlapping blockers count once. The holder and its attached decorations are excluded as foreign blockers; other bodies, buildings and icons remain blockers. More than50% body OR more than50% icon coverage excludes; exactly50% does not. Log both fractions and reason. Missing geometry/order evidence fails closed. Require body visibility, full body inclusion and full final crop inclusion.

Union body+icon rectangles, grow8 screen pixels on all sides, then square around the union centre using the larger dimension. Convert to native pixel boundaries outward: floor left/top; ceil right/bottom; retain the larger resulting integer extent on both axes. This covers fractional boundary pixels without clipping. Crop original pixels; no resize, resampling or4x display artifact. Freeze exclusions and input/image hashes on disk before producing any crop. A shortage produces no blind crops unless an explicit terminal diagnostic is registered; that diagnostic cannot pass14-item acceptance.

Fresh reader receives only blind crops and seven names, never protocol/icon sheet/keys. Freeze answers before village questions and reveal no key until village answers are frozen. Activity bar>=10/14 and no kind0/2. Village task: six Z1(village,capture) pairs, at most two per(seed,village), excluding tied top counts; bar>=5/6. If dropped record NOT_RUN.

## Crowd proof

Select a real entity using P12_ACTIVITY_SELECTED_ENTITY; collector records actual inspector-selected entity at capture. Keep selection consistent with root's actual UI/crowd run; tooling does not alter z-order. For each of three distinct Z4 captures, count raw live agent positions within Euclidean10tiles inclusive of camera centre (world origin448,28 plus tile centre8,8). Require>=25, selected person in that set and visible/inside viewport, and both selected body/icon foreign coverage fractions exactly0. check_crowd_p12.py writes hashes and derived proof without changing raw sidecars. H9 remains awaiting the fresh life-reader's3/3 activity answers even if geometry passes.

## Commands (leader schedules Godot)

- Matrix: `P12_DEADLINE_EPOCH=... P12_ACTIVITY_BUILD_PROFILE=release python3 tools/art_pipeline/run_activity_captures_p12.py CHECKOUT NEWDIR 20261018 docs/art/production12/activity/protocol.md`
- Prepare activity crops: `python3 tools/art_pipeline/prepare_blind_crops_p12.py CHECKOUT NEWDIR Z4_D30_10_JSON Z4_D30_15_JSON Z4_D60_10_JSON [SUPPLEMENTS...]`
- Supplement: `P12_EXTRA_BASE_PLAN=.../exclusions-before-crops.json P12_DEADLINE_EPOCH=... python3 tools/art_pipeline/run_extra_activity_p12.py CHECKOUT NEWDIR SEED DAY HOUR KIND`
- Village task: `python3 tools/art_pipeline/prepare_activity_villages_p12.py FROZEN_ACTIVITY_ANSWERS NEWDIR Z1_JSON...`
- Crowd: `python3 tools/art_pipeline/check_crowd_p12.py NEW_PROOF_JSON CAPTURE1_JSON CAPTURE2_JSON CAPTURE3_JSON`
- Offline: `PYTHONPATH=tools/art_pipeline python3 -m unittest tools/art_pipeline/test_activity_geometry_p12.py -v`

Runtime capture validation and visual/reader acceptance are pending. Parser checks and unit tests do not replace them.
