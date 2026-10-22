# PROD2 native-size crowd

Standalone Godot 4.6 project; no game code, bridge or simulation dependencies.
Loads each promoted `route_i/<look>/MANIFEST.json`, requires all ten structural
frame results and numeric panel checks, validates the ten PNG hashes and native
canvas sizes, and records every input hash. Run after the final route is frozen.

From the repository root, in a quiet host interval:

```sh
mkdir -p /Users/rexxa/work/artprod2-data/godot-crowd
caffeinate -u -d -t 90 bash tools/v8/run_bounded.sh 85 \
  /Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
  --path art/proto/prod2 --rendering-driver opengl3 -- \
  --out /Users/rexxa/work/artprod2-data/godot-crowd \
  > /Users/rexxa/work/artprod2-data/godot-crowd/run.log 2>&1
```

Require `PROD2_CROWD_DONE`, no script errors, `measurement_verified: true`, and
`P1_pass: true` in `measurement.json`. A `--seconds 2` run is a smoke test only;
it deliberately cannot satisfy P1. `--route /absolute/path` selects a frozen
copy when necessary. Do not compare a short smoke's FPS to the 60-second bar.

The 1600×960 visible viewport contains 200 sprites at scale 1 and nearest
filtering, each anchored at (40,104) upright or (56,72) asleep. An independent
28×8 contact shadow sits on the same foot position. Each person cycles through
both facings and all five original poses at 8 fps. The benchmark counts
`RenderingServer.frame_post_draw` events and cross-checks against
`Engine.get_frames_drawn()`. The timer excludes warm-up and initial PNG
readback; final PNG readback occurs after timing ends. Per-frame intervals,
one-second window/visibility samples, person-to-look identities, engine/GPU
identity, and start/end viewport PNGs are retained outside the repository.

This measures rendered frames, not physical display scanout. Window resizing,
minimization, non-drawable frames, hidden sprites, changed inputs, failed
readback, enabled vsync, or a duration shorter than 60 seconds invalidate P1.
No isolated rendering result establishes production simulation performance.
