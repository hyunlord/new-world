# Round-two runtime evidence audit

**C6 PASS in the isolated art prototype:** the final primary all-walking 200-person run measures **88.230 draw FPS over 60.002084 seconds at 1×**. The 24-person all-walking run measures 355.719 draw FPS. This is not full simulation performance or a crowd-readability verdict.

Evidence root: `/Users/rexxa/work/arttrial2-data/trial2b/captures-round2/`. Audited `camp-24-1` and `camp-200-1`: `runtime.json`, `frame_times_us.json`, `capture.png`, launcher and logs. Inspected both actual screenshots. Run logs identify Godot 4.6 stable, Apple M4 Max, OpenGL 4.1 compatibility over Metal; `headless=false`, macOS display, VSync disabled, 1408×832 viewport, zoom 1.

## Final primary all-walking runs

Audited `camp-walk-24-1` and `camp-walk-200-1` after `run_walk.sh` completed. Both report `all_walk=true`, active animations exactly `[walk]`, real macOS/OpenGL display, VSync off, 1408×832 and zoom 1. Directly inspected both resulting screenshots. Every actor receives the same bounded scripted loop formula (phase-offset ±8 px sine/cosine motion); this is a renderer fixture, not navigation through the enclosure.

| Measurement | camp-walk-24-1 | camp-walk-200-1 |
|---|---:|---:|
| Wall seconds | 60.002394 | 60.002084 |
| Process samples | 21345 | 5295 |
| Draw callbacks | 21344 | 5294 |
| Draw FPS | 355.719140 | 88.230269 |
| Process interval p95, ms | 3.617 | 12.791 |
| Sum process intervals, seconds | 60.002281 | 60.000886 |
| Viewport-enclosed actor bounds | 24/24 | 200/200 |
| Unique occupied tiles / maximum occupancy | 24 / 1 | 200 / 1 |

Recomputed sample counts, draw-FPS division, p95 and viewport-bound checks. For each primary run all 8 source, 41 person and 17 environment/autotile hashes match current bytes; capture hashes match; logs have no script/parse/ERROR entries. Source and asset mappings also match the retained mixed runs below. Visibility limitations below apply equally: bounds enclosure is not occlusion testing or a blind count.

- `camp-walk-24-1/capture.png`: `7ef1f6b6e3aa637cf559c5652f3441b9fdaa0870d9aba089791c26deec258e87`
- `camp-walk-200-1/capture.png`: `db9d0c337e35c6cfbd8f0ec35ee05f2794b1c703c9c02599c81aedf5c816c9aa`

## Retained mixed-animation evidence

| Measurement | camp-24-1 | camp-200-1 |
|---|---:|---:|
| Wall seconds | 60.000834 | 60.009329 |
| Process samples | 21115 | 5207 |
| Draw callbacks | 21114 | 5206 |
| Process FPS | 351.911775 | 86.769842 |
| Draw FPS | 351.895109 | 86.753178 |
| Process interval p95, ms | 3.425 | 13.214 |
| Sum of recorded process intervals, seconds | 60.000682 | 60.008110 |
| Bounds enclosed by viewport / recorded actors | 24/24 | 200/200 |
| Unique occupied tiles / maximum occupancy | 24 / 1 | 200 / 1 |

Independently recomputed draw FPS as `rendered_frames / wall_seconds`, process FPS as `len(frame_times_us) / wall_seconds`, and p95 as sorted sample index `floor(N*0.95)`, matching recorded values. Sample count matches process frames. The small wall-time excess over summed intervals is consistent with deferred finish bookkeeping. Timing begins after a real `frame_post_draw`; draw callbacks are counted only during measurement. The generic `fps` field is **process** FPS, not draw FPS. Neither is a display scanout or GPU timestamp measurement; uncapped draw throughput can exceed monitor refresh.

## Identity checks

At audit time, all **8 source hashes, 41 loaded person texture hashes and 17 environment/autotile hashes** per run match current disk bytes; both runs have the same mappings. Both capture SHA-256 values match. No `SCRIPT ERROR`, `Parse Error` or `ERROR:` appears in either run log. This verifies recorded files, not an atomic filesystem snapshot throughout the run. Source hashes were captured at startup; person texture hashes are collected at finish.

- `camp-24-1/capture.png`: `35197d7277a7a547714f11d786796f40465151dec591695660918057677ebfe2`
- `camp-200-1/capture.png`: `52a538e0b563a554a181e391fc1d9f0c138344eb426cf02b01176d6697e88d7e`
- Meadow: `fd738face2ed4ca91327c53ee6106052f892fca977f8dfec695ebc1cd27c9b60` (quiet try2).
- `Camp.gd`: `ff356170b42be05bc08131f31f475e01de46b510fe20857b5642aaa775d8b858`
- `PersonComposer.gd`: `221e7cd554b2515773e2f7ded0d50da73d8cf876d4f86def0a54cd2e917e36a3`
- `Companion.gd`: `4381fd75fc2ed80f2d28069d0f57f1988f2924e58390752d98f3cbc6debe752c`

The complete eight-source and texture hash maps remain in each `runtime.json`. Assets are loaded from `art/style05/people`, `art/style05/autotile_v2` and external `trial2b/runtime-env`; capture metadata explicitly marks them candidates unless separately judged. A valid capture does not promote assets.

## Visibility and visual limitations

Recomputed all recorded sprite bounding rectangles against the viewport: all fit. `_population_audit` merges visible Sprite2D bounds and checks viewport enclosure at finish. It does not test alpha silhouettes, overlap with scenery, inter-person occlusion, or continuous visibility during all 60 seconds. The final screenshot is captured after a further draw callback. In the 200-person image some figures overlap the hut, hearth, walls and companion examples; the metadata's word “fully” must not be interpreted as unobscured. No visual crowd count was performed in this audit.

The two retained mixed records use `all_walk=false` and six active animation names: idle, walk, chop, carry, sleep, knap. Only walk/carry actors receive scripted ±8 px oscillation. These runs establish mixed-animation rig throughput, **not** the goal's entire 24-person set walking a scripted loop. The image is a static end-of-run sample, not a video review of motion or attachment stability. Animation samples cover only the early sample window. The isolated prototype has no simulation bridge load; these FPS values must not be extrapolated to the live simulation.

Ground reads as subdued broad wash at actual 1× and leaves people/props more distinct than the dense original crop. Palisade posts form continuous circular rows but read as beads/post tops rather than the hut's inked timber walls. Companion outlines remain mechanically geometric. Detailed findings are in `AUTOTILE_REVIEW_V2.md` and `CODE_STYLE_V2.md`. The 2×/0.5× captures and blind crowd task are outside this bounded audit.
