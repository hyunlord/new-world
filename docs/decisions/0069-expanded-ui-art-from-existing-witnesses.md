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

### Visibility follow-up candidate — 2026-11-17

Polish step39 found a presentation regression introduced by `322a3a0b8`: a 288×112 illustration above each decision card increases the HUD stack and changes obstacle relocation and camp framing. The failing `day1_night_half` agent4 has 0 qualifying pixels against the required60 beneath TaskPlaque. The camera is `[6014,6160]`, versus `[5642,6276]` after reverting art-ui. Showing art only on the top card did not change the failing camera; the scratch revert `a26cbb69e` passed 8/8 and polish reverted the art-ui merge as `176385964`.

The follow-up candidate retains every image and localized control but places each card illustration beside its text at 128×72. This corrects the card layout boundary rather than changing camera policy, simulation behavior, visibility thresholds or the oracle. Exact step39 validation is pending; the earlier asset/headless/20-screenshot results remain scoped evidence and do not establish a full-gate pass or resolution of this defect. Preserve failed receipts. Sources: external `ui2/visibility-fix/DEBUG.md` and `/Users/rexxa/work/polish-data/PROGRESS.md` at 13:13.

## Date

2026-11-17

## Source

Owner task: Part U2 of `/Users/rexxa/work/art-img-data/BRIEF_IMG2.md`; hard rules and style in `BRIEF_IMG.md`, planner notes in `/Users/rexxa/work/art-img-data/ui/PROGRESS.md`. Implementation boundaries: `scripts/ui/generated_art.gd`, `assets/ui/chronicle_art.json`, existing item, chronicle, inspector and migrant panels; ADR0068 and ADR0033.
