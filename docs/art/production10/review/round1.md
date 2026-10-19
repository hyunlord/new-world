# Independent review — round 1 of 2

Verdict: no actionable source defect found in the reviewed changes. Provisional source/evidence review only; final acceptance remains pending round 2, integrated inspector, final performance/quiet records, and visual evidence.

Scope: merge `0caabf5fd1c62b9d3c99845daab1805e79799db7` through frozen source `0c044ec543eb6f4e5f2c425e7cf605b70cbd992c`. Read-only production review. Did not read GOAL.md or original attachment sections 3–7. No checkout/reset/build/Godot/production edits or delegation performed. External Part3 patch reviewed provisionally. Applied WorldSim localization and code-review guidance. Memory search returned no relevant Art Pass10 entries.

## Findings

No critical/high/medium production defect established. No assertion weakening found in the reviewed existing-test changes: activity getter is added to protected reads and equivalence coverage; old tests retain assertions. The additional capture safe-counter exemption remains guarded by the same structural predicate.

Informational evidence boundary — external `activity-oracle-draft/check_activity_sidecars.py:89–110` validates body, logical activity, pose, facing, atlas/frame, mirror, overrides and visibility, but not arrived position/seat/declump/lateral offsets or craft/explore icon children. An offline synthetic fixture with `seated=true`, offsets `[999,999]`, drawn position `[99999,99999]`, and `activity_icon_visible=false` still returns `pose_check_complete=true`. This is not evidence of a renderer defect: the reported flag is pose-specific, H7 remains false in the checker, installed position fixtures cover position behavior, and image/icon review is separately pending. Do not describe the sidecar oracle alone as full position/icon validation. No capture-contract change requested midcohort.

## Source observations

- Seat-cache key covers eligibility and distance inputs, invalidates on hearth/blocked changes, resets and toggle; returned mapping is copied. Spatial hash preserves original ordinal accumulation order, indexes complete rectangles and moves entries after walker relief. No changed overlap threshold observed.
- Allocation path computes frame index every frame and returns a fresh shallow frame dictionary; selection index rebuilt each frame; pose output is consumed before reuse; region-path cache is reset on toggle. Rejected tick-cache early-return logic is absent.
- Activity snapshot uses protected `SimReads`; snapshot entity bits join separately from art IDs. Arrival uses existing bridge `until_tick >= 0` sentinel. Arrived agents are excluded from camp seating, declumping, and walker lateral relief; same-tile spread remains. Existing interpolation/smoothing still runs.
- Part3 patch uses selected entity bits for component getters, hides absent components, sets all four bars to 0–1000, resolves family art IDs to live entity bits again on click, and caps visible children to five while counting living children. ScrollContainer wraps content in existing 620px panel. Actual fit, clicks, and localization runtime remain provisional.

## Independently verified evidence

- Spatial/allocation archives: ZIP CRC checks pass. All indexed member SHA256 hashes match (191 spatial, 202 allocation); rejected-tick archive has 249 matching indexed hashes. Tick archive SHA256 matches recorded `ed84ff190b910adb26efa73fc79f04433f0b067d29d589c54c8a4401f702fe23`.
- Decoded archived RGBA pixels independently compared: spatial hash and reduced allocation each have all six A/Aprime/B framings byte-exact. Their W0 and W200 baseline/candidate JSONL dumps are byte-identical and contain exactly 600 rows each.
- Spatial and allocation identity/pair source dictionaries and build hashes match. All six optimized production source files in allocation provenance match git blobs at accepted `ebadc0b9` (renderer, camp seats, frame choices, pose moments, sprite library, selection index). Thus the historical accepted Part1 source binding is supported; this does not claim final activity/inspector source identity.
- Recorded three tick failures retain 1680/816/1680 differing pixels, rejection status, contended performance qualification, and running check not reached. No mask/threshold weakening found. Earlier seat reset-race and allocation rejection evidence remains present.
- Current committed diff contains only authorized path families; Rust change is confined to sim-test static read-boundary test. `.gitignore` modification and untracked `.ignore` existed in working status and are outside this frozen commit review; neither was modified by reviewer.
- Compiled EN/KO add exactly eight activity keys, with no old string changes. Existing registry IDs unchanged; new IDs 5226–5233. Upcoming six inspector keys are referenced in provisional patch; final compiled strings/registry IDs must be checked after integration in round 2.

## Remaining verification limits

Seat acceptance records inspected (three 150-frame pairs, six-framing and two running-check references); its raw image cohort was not independently decoded in this round. No new runtime tests/captures or visual acceptance performed. Final H1–H5 matrix, quiet-period provenance, integrated Part3 tests and screenshots, actual activity sidecar cohort and blind visual results remain round-2 inputs. Preserve original failed artifacts and distinguish historical Part1 identity from the intentionally changed activity appearance.
