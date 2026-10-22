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

Final performance evidence is external `proto-perf-24-v2/` and `proto-perf-200-v2/`:60-second windowed runs with all six activity states, one center per64px tile and every person fully in the viewport. The earlier unsuffixed performance runs retain the superseded compact-grid/walk-only workload and must not be used as final C6 evidence.

Blind review2 is tied to the exact `proto-final-people-{meadow,forest_floor}-{1,2}/capture.png` files. Subsequent `proto-people-…-v2/` images use the same visual algorithm but differ in sampled live animation phase:2,747/8,117/3,128/8,639 changed pixels out of921,600. They add full provenance and population metadata; do not silently transfer exact-image review claims between these captures. The external `proto-people-review-image-comparison.json` records both sets of hashes. Original answer-key sex labels are retained with `sex_rendered=false`; absence of a visual sex encoding remains a failed requirement, not an exemption from the20/24 bar.

Observed visual limitations: generated bodies/tops remain gray and low-contrast on dense ground; repeated terrain and wall material joins remain visible; the winter tint cannot create snow-cover geometry; the0.5× image shows the finite ground patch edges. These captures exercise the prototype and do not convert numeric-rejected assets into accepted style05 assets.
