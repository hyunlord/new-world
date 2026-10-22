# Standalone style 05 prototype

Godot4.6, GL compatibility, Sprite2D cut-outs with Bone2D/Skeleton2D and AnimationPlayer. Open this folder as a separate project. No game code, assets or GDExtension are imported. See `../style05/people/PERSON_SPEC.md` for the composer API.

From repository root, always use bounded runs:

```sh
bash tools/v8/run_bounded.sh 90 /Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --path art/proto --rendering-driver opengl3 -- --mode camp --count 200 --zoom 1 --seconds 60 --people-root /absolute/candidates/people --env-root /absolute/candidates/env --autotile-root /absolute/candidates/autotile --out /absolute/data/run
```

Use `caffeinate -u -d -t 90` while capturing if the display is asleep. Do not run a game bridge build for this standalone project.

Arguments after `--` are key/value pairs:

| Argument | Values |
|---|---|
| `--mode` | `camp`, `people` (fixed12 people in a4×3 image-only layout), `season` (ground-only shader comparison, no rig evidence), or `season_compare` (tinted base left, generated reference right) |
| `--count` | Camp24 or200; other positive counts useful for diagnostics |
| `--all-walk` | `true` forces every camp person to play walk and follow an8px loop; default `false` preserves mixed activities. Appearance and held props stay unchanged. Ignored outside camp mode. |
| `--zoom` |0.5,1,2 |
| `--ground` |`meadow` or `forest_floor` |
| `--season` |0 spring,1 summer unchanged,2 autumn,3 winter |
| `--seconds` |60 for performance;2 for still captures |
| `--people-root` |Flat generated PNGs, or parts/ and overlays/ subdirectories |
| `--env-root` |Generated textures; searches root and one family-directory level |
| `--autotile-root` |16 wall PNGs, `wall_00.png` through `wall_15.png` |
| `--out` |External absolute evidence directory |

Environment names: `meadow.png`, `forest_floor.png`, `hut.png`, `hearth_0.png`, `log_pile_small.png`, `oak_large.png`. Wall indices encode north1/east2/south4/west8 connections. Interiors use `floor_00.png` and the south grass-edge row uses `floor_01.png`. The10×8 enclosure has a two-cell south doorway and an internal partition. `enclosure_cells.json` records every mask/cell for seam auditing. Camp1× uses1408×832; Camp2× expands the viewport to2816×1664 so the whole enclosure remains visible at true2×; people2× remains1280×720 around its compact layout.

Each run writes `runtime.json`, `animation_samples.json`, `chop_grips.json`, raw `frame_times_us.json` and windowed `capture.png`. Runtime metadata includes capture SHA-256, source hashes at startup, actual loaded asset SHA-256 values and explicitly separates candidate art from acceptance. People mode also writes `answer_key.json`; keep it away from blind reviewers. Camp mode records its loop as scripted motion, not game simulation. Vsync is disabled; FPS is measured rendered-process frames divided by monotonic wall time. Headless metadata is explicitly marked and cannot satisfy FPS or visual acceptance.

Required capture matrix: people mode meadow/forest_floor×1×/2×; camp24 at0.5×/1×/2×; camp200 at1×; runtime60s camp24 and200 at1×. Four code companions appear at lower right beside generated camp props. Season comparisons require separately generated autumn/winter references; the shader alone is no verdict.

All missing textures emit explicit errors and the harness exits1. No shape/body placeholders are provided. `PROTO_DONE` must appear, exit code must be0, and logs must have no unexpected errors before treating a run as technically successful. Inspect captures directly before any aesthetic or readability claim.

Season comparison additionally takes `--reference-root` and `--reference` (PNG basename). Crowd centers lie on a64px grid; walk/carry follow8px within-cell loops, other activities stay on their tile. Both24 and200 repeat the same12-attribute composition. Final runtime metadata audits unique occupied tiles and full sprite bounds inside the viewport.

## Evidence boundaries from this rehearsal

Mixed-activity performance evidence is external `proto-perf-24-v2/` and `proto-perf-200-v2/`:60-second windowed runs with all six activity states, one center per64px tile and every person fully in the viewport. These historical runs belong to commit `12df4dfe`, with Camp.gd SHA-256 `4c866b31c2f2a0af8ae101df1c209abdf0ccb9b1870f0f3f1615bf2aa954c128`; they do not claim the later all-walk option's source hash. The earlier unsuffixed performance runs retain the superseded compact-grid/walk-only workload and must not be used as final C6 evidence.

Blind review2 is tied to the exact `proto-final-people-{meadow,forest_floor}-{1,2}/capture.png` files. Subsequent `proto-people-…-v2/` images use the same visual algorithm but differ in sampled live animation phase:2,747/8,117/3,128/8,639 changed pixels out of921,600. They add full provenance and population metadata; do not silently transfer exact-image review claims between these captures. The external `proto-people-review-image-comparison.json` records both sets of hashes. Original answer-key sex labels are retained with `sex_rendered=false`; absence of a visual sex encoding remains a failed requirement, not an exemption from the20/24 bar.

Observed visual limitations: generated bodies/tops remain gray and low-contrast on dense ground; repeated terrain and wall material joins remain visible; the winter tint cannot create snow-cover geometry; the0.5× image shows the finite ground patch edges. These captures exercise the prototype and do not convert numeric-rejected assets into accepted style05 assets.

## Explicit all-walking rehearsal

`--all-walk true` changes only camp activity selection; the same12-person appearance/prop composition repeats for24 and200. Every rig plays `walk` and moves on an8px radius loop within its64px tile. Source SHA-256 for these runs: Camp.gd `3df6ae0c313d49ae7fa52615b50f5dd760341f8985258495d832ba075078d1b7`. Historical mixed runs above retain their original hashes.

Exact run commands from repository root (external output roots preserve all earlier mixed runs):

```sh
caffeinate -u -d -t 150 bash tools/v8/run_bounded.sh 20 /Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --path art/proto --rendering-driver opengl3 -- --mode camp --count 24 --all-walk true --zoom 0.5 --seconds 2 --people-root /Users/rexxa/work/arttrial2-data/people --env-root /Users/rexxa/work/arttrial2-data/runtime-env --autotile-root /Users/rexxa/work/arttrial2-data/autotile/derived --out /Users/rexxa/work/arttrial2-data/proto-all-walk-24-0.5
caffeinate -u -d -t 95 bash tools/v8/run_bounded.sh 90 /Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --path art/proto --rendering-driver opengl3 -- --mode camp --count 24 --all-walk true --zoom 1 --seconds 60 --people-root /Users/rexxa/work/arttrial2-data/people --env-root /Users/rexxa/work/arttrial2-data/runtime-env --autotile-root /Users/rexxa/work/arttrial2-data/autotile/derived --out /Users/rexxa/work/arttrial2-data/proto-all-walk-24-1
caffeinate -u -d -t 95 bash tools/v8/run_bounded.sh 90 /Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --path art/proto --rendering-driver opengl3 -- --mode camp --count 24 --all-walk true --zoom 2 --seconds 2 --people-root /Users/rexxa/work/arttrial2-data/people --env-root /Users/rexxa/work/arttrial2-data/runtime-env --autotile-root /Users/rexxa/work/arttrial2-data/autotile/derived --out /Users/rexxa/work/arttrial2-data/proto-all-walk-24-2
caffeinate -u -d -t 95 bash tools/v8/run_bounded.sh 90 /Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --path art/proto --rendering-driver opengl3 -- --mode camp --count 200 --all-walk true --zoom 1 --seconds 60 --people-root /Users/rexxa/work/arttrial2-data/people --env-root /Users/rexxa/work/arttrial2-data/runtime-env --autotile-root /Users/rexxa/work/arttrial2-data/autotile/derived --out /Users/rexxa/work/arttrial2-data/proto-all-walk-200-1
```

The0.5× capture used a20-second bound and150-second caffeinate assertion; its scene arguments above are identical. Activate the actual process after `PROTO_READY`, from a separate terminal:

```sh
osascript -e 'tell application "System Events" to tell process "Godot" to set visible to true' -e 'tell application "System Events" to tell process "Godot" to set frontmost to true'
```

Two initial24-person timed attempts reached `PROTO_READY` but timed out at90s, preserving `proto-all-walk-24-1.log` and `proto-all-walk-24-1-retry.log`; neither produced a runtime result. The third attempt (`-retry2.log`) completed after explicit process visibility/frontmost activation. Its measured60.020471s includes an inactive/throttled window period; a process sample (`all-walk-timeout-sample.txt`) observed `OS::add_frame_delay` sleeping. This is an observed-condition measurement, not a clean foreground benchmark or a proven root-cause diagnosis. The default mixed-activity path remains unchanged.

The200-person first attempt also timed out (`proto-all-walk-200-1.log`); the retry (`-retry.log`) completed following late process activation. Its60-second interval likewise was not continuously foreground-controlled. A4-second periodic activation loop was started near the end, and continued during the2× capture; `all-walk-focus.log` preserves its timestamps. These condition differences prevent a controlled24-vs200 scaling comparison.

| Historical process-only all-walk run | Seconds | Process frames | Process FPS | Process frame p95 ms | Fully visible / unique tiles |
|---|---:|---:|---:|---:|---:|
|24 at1× |60.020471 |7560 |125.957026 |24.690 |24 /24 |
|200 at1× |60.012420 |8037 |133.922278 |14.091 |200 /200 |

All four captures (`proto-all-walk-24-0.5`, `proto-all-walk-24-1`, `proto-all-walk-24-2`, `proto-all-walk-200-1`) passed source/asset/capture SHA checks, frame-count consistency, four all-walk animation snapshots, full sprite bounds, one person per tile, and final center displacement8px within0.001px. Verification: external `proto-all-walk-verification.json` and `verify_all_walk.py`. Successful process exits were0, every log contains `PROTO_DONE`, and no parse/script errors appeared. The headless editor import also exited0 without parse errors. Direct capture inspection confirms preserved dense terrain/gray-body contrast limitations; these results do not establish style acceptance.

## Draw-counted performance evidence

The earlier `fps`/`frames` fields count process callbacks. Their rendered FPS is `NOT_VERIFIED`, including the historical mixed runs and the first all-walk runs. Late activation demonstrated that process callbacks can continue without matching draws. Current Camp.gd SHA-256 `41e0b2ec633c0029f30097adf8317e7ad4726879f5dd08d8f2409a043eea373a` adds an actual `RenderingServer.frame_post_draw` counter, explicit `rendered_frames`/`rendered_fps`, and separate `process_frames`/`process_fps`. Legacy `fps` remains a process-FPS alias. Windowed timing starts at the first post-draw signal after initialization; headless evidence cannot establish rendered performance.

Reproduction command (driver source and exact child argv retained externally):

```sh
/Users/rexxa/work/arttrial2-data/.venv/bin/python /Users/rexxa/work/arttrial2-data/run_all_walk_draw.py
```

The driver runs24 then200 sequentially for60 seconds each, restores Godot visible/frontmost every3 seconds (actual intervals about3.2s), and keeps a200-second caffeinate assertion. The `proto-all-walk-draw-{24,200}-1-driver.json` files preserve command argv, timestamps and successful activation exit codes. Their `before` strings are empty because AppleScript discarded the intermediate query result; continuous foreground visibility was **not** independently established. This limitation does not erase the direct post-draw counter. No headless performance figures are substituted.

| People | Measured seconds | Rendered frames | Rendered FPS | Process frames | Process FPS | Process p95 ms |
|---|---:|---:|---:|---:|---:|---:|
|24 |60.000379 |14318 |238.631826 |16660 |277.664913 |6.912 |
|200 |60.012859 |4874 |81.215927 |4875 |81.232591 |13.965 |

Both runs exited0 with `PROTO_DONE` and no parse/script errors. All rigs played walk in all four snapshots, all24/200 sprite bounds fit the viewport, all occupied distinct64px tiles, and every final center was8px from its start. Current source hashes, actual loaded asset hashes, capture hashes and frame counts passed `verify_all_walk_draw.py`; results are in `proto-all-walk-draw-verification.json`. The0.5×/2× captures remain tied to the prior `3df6ae…` source hash; new1× draw-counted captures are in `proto-all-walk-draw-24-1/capture.png` and `proto-all-walk-draw-200-1/capture.png`. Art acceptance remains separate.
