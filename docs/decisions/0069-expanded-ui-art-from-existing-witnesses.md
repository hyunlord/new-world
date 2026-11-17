# ADR 0069 — Expand optional UI art from existing simulation witnesses

## Context

Part U2 of M3D-IMG2 extends the first generated-art pass without adding panels or changing the simulation. All 166 knowledge-node images and existing building/station portraits are already covered. The new scope is 119 item illustrations, 72 additional person portraits, 24 migrant/trader looks, 20 event vignettes, four future growth cards, three selected era splashes and one paved-road portrait: 243 adopted assets from 249 generated options. Each era has three options; six alternatives stay outside the runtime tree. External asset validation, actual-scene headless checks and GPU-locked windowed captures pass. Final remote receipts remain external; the planner publication gate and integration remain pending.

ADR0068's optional image lookup, stable identity and existing-surface boundaries remain in force. Its four-look portrait selection is extended here; historical generation and verification results in that ADR are not revised.

## Decision

- Extend each age/sex/era portrait family from four looks to eight. Derive selection from immutable UI snapshot identity, retaining the corresponding original four-look image if the newly selected file is absent, then the existing authored portrait if no generated image exists. Do not change simulation RNG or saves.
- Add 256-square item illustrations in the separate `item_illustrations` family, addressed by canonical item ID. Show them alongside the existing item inspector/list content. Preserve the GLB-derived icons and the loose-world texture provider.
- Resolve chronicle illustrations through `assets/ui/chronicle_art.json`. Rules match existing witness fields and catalogue IDs to `events` or `eras` assets. Unknown or unmatched rows retain text-only presentation; a decorative image must not create a new event, claim an unsupported first occurrence or change filtering and command behavior.
- Display era art within the existing chronicle surface. Do not create a full-screen transition, new dialog or title menu to show it. Reuse the existing migrant panel for optional migrant looks. Keep trader portraits, future M-GROW cards and vignettes lacking an appropriate real event data-only.
- Preserve aspect ratio, linear filtering, localized text and non-interactive image nodes. Missing images keep established controls and content usable. No Rust code, numeric simulation data, bridge methods or save fields are changed.
- Generate images through cloud image generation with the established warm, clean painterly palette. Fictional people remain fully clothed; text, logos, castles, stone towers, city walls and fortifications are excluded. Preserve every adopted prompt and bounded attempt record externally in `/Users/rexxa/work/art-img-data/ui2/`; local captures and contact sheets require the shared GPU lock.
- Work and normally push only `v8/art-ui2`; the planner owns integration and publication. Asset counts, render evidence, headless checks and remote landing must be reported separately. The scoped asset/UI validation does not claim a full publication gate or a merge.

## Alternatives considered

- Replace the existing item icons with generated art: rejected because the GLB icons already match the game objects, while the new request is for complementary inspector illustrations.
- Trigger every supplied vignette from an approximate condition: rejected because art must follow actual recorded witnesses rather than invent events or first-occurrence semantics.
- Add trader, growth or era-transition panels: rejected under the existing-surface boundary; retain the assets as data until a suitable surface exists.
- Require all eight portrait files to load: rejected because partial generation or a missing import must preserve the earlier working portrait set.

## Consequences

The existing UI can accept image batches without becoming dependent on their availability. Content-to-art associations remain inspectable data, while the simulation and its event stream stay authoritative. Data-only art is delivered separately from connected art and is not counted as a working UI surface. The completed asset validation covers all 249 unique outputs and preserves all 119 original GLB icons; actual-scene headless verification reports ten surfaces with zero failures. Windowed verification now independently confirms ten surfaces each at native 1920×1080 and 1280×720 with zero capture failures, and thirteen final contact sheets cover all 249 options. The capture tool applies `--capture-size` after camera window fitting; round-1 files labeled laptop were actually 1920×1080, whereas the corrected U2 PNG dimensions are confirmed (`capture_resolution_issue.json`, `capture-size-validation.json`). The successful locked lane ends with `UI2_ART_REAL_CAPTURE_LANE_DONE` in `capture_retry_logs/attempt_8.log`. Final remote receipts remain external and planner publication remains a separate obligation.

### Visibility follow-up scoped proof — 2026-11-17

Polish step39 found a presentation regression introduced by `322a3a0b8`: a 288×112 illustration above each decision card increases the HUD stack and changes obstacle relocation and camp framing. The failing `day1_night_half` agent4 has 0 qualifying pixels against the required60 beneath TaskPlaque. The camera is `[6014,6160]`, versus `[5642,6276]` after reverting art-ui. Showing art only on the top card did not change the failing camera; the scratch revert `a26cbb69e` passed 8/8 and polish reverted the art-ui merge as `176385964`.

The superseded compact-card experiment `62ca33b21` retained every image and localized control beside its text at 128×72, but actual-save headless geometry proves card compaction alone insufficient. With card height534 and camera `[5856,6136]`, the solver stores agent4 y70.77 and reports a synthetic180px shift predicting y250.77, apparently clear of TaskPlaque bottom235. Actual `ViewProjection` after90 frames gives y171.28, still beneath the plaque. An external no-card-art probe gives camera `[5642,6276]` and agent4 x597..633, partly outside the plaque right edge620. Card growth is the trigger; the underlying fault is using a 2D translation and synthetic validation for a tilted 3D camera. This is diagnostic geometry evidence, not a GPU capture failure: capture2 had not started.

The verified follow-up changes camera framing only: bounded actual reprojection synchronizes the rig through its existing `update_camera` path reused from `_animate_zoom`. It restores the original 288×112 vertical illustration on every card and removes the experimental compact-card height test; the reduced-card experiment remains historical external evidence, not a delivered feature. With original card art, the actual day1-night saved-scene regression reports all ten freshly projected bodies clear and camera `[6014,5894]` (`projection-final-original-art.log`). The live camp test now reads fresh projection and passes389 assertions, including Home idempotence, cancellation and unchanged simulation-state bytes (`projection-final-live-camp.log`). Final strict checks pass (`projection-final-strict.log`); simulation behavior, visibility thresholds and the oracle remain unchanged. Exact step39 capture-2 passes all eight scenes and 80/80 residents on frozen `e82591cc5453c77c24775acf1e5a07633a1b1955`; the receipt verifies the same HEAD. In `day1_night_half`, agent4 now has 1061 qualifying pixels against the unchanged requirement60 (previously0), at camera `[6014,5894]`. External `ui2/visibility-fix/capture-2/run/visibility.json` reports no failures or changed inputs. This establishes scoped visibility proof; the full planner publication gate and integration remain pending. Evidence: external `ui2/visibility-fix/geometry-actual-2.log`. The earlier asset/headless/20-screenshot results remain scoped asset/surface evidence; neither those checks nor this step39 result establish a full-gate pass. Preserve failed receipts. Sources: external `ui2/visibility-fix/DEBUG.md` and `/Users/rexxa/work/polish-data/PROGRESS.md` at 13:13.

## Date

2026-11-17

## Source

Owner task: Part U2 of `/Users/rexxa/work/art-img-data/BRIEF_IMG2.md`; hard rules and style in `BRIEF_IMG.md`, planner notes in `/Users/rexxa/work/art-img-data/ui/PROGRESS.md`. Implementation boundaries: `scripts/ui/generated_art.gd`, `assets/ui/chronicle_art.json`, existing item, chronicle, inspector and migrant panels; ADR0068 and ADR0033.
