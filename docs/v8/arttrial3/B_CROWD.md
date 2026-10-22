# B crowd — rejected-input diagnostic, 2026-10-22

**P1 eligible = false.** The B structural set fails, so this run cannot satisfy
P1 or change the route verdict. `CharacterLab.gd`'s accepted-set admission gate
was not edited or bypassed. A separate `Trial3BCrowd.gd` draws rejected inputs
only to record the available standalone rendering measurement.

## Observed run

| Measurement | Result |
|---|---:|
| Actual sprites / scale | 200 / 1× |
| Viewport and arrangement | 1280×960, 20×10 |
| Measurement duration | 60.001147 s |
| `RenderingServer.frame_post_draw` events | 29,389 |
| Render-loop FPS | 489.807 |
| Hidden/not-drawable/process frames | 0 |
| One-second visibility samples | 60; all can_draw=true, not minimized, 200 in viewport |
| Inputs changed during measurement | false |
| Dataset status | DIAGNOSTIC_REJECTED |

Godot 4.6 stable official `89cea1439`, macOS display server, OpenGL 4.1 Metal
compatibility renderer, Apple M4 Max. Vsync disabled: the FPS is rendered-frame
submission throughput, **not the physical display refresh rate**. There was a
2-second warmup before the full 60-second sample. The process exited 0 and
emitted `B_CROWD_DONE verified=true ... P1_eligible=false`. No headless FPS
measurement was used; the headless command was parse validation only.

This was a **contended shared-host diagnostic**: load average during the run
was approximately 29, with another Rust test using roughly 289% CPU and Unity
roughly 98%. No quiet-session admission or simulation workload was present.
Consequently it is not a production FPS claim or a comparative benchmark.

## Actual inputs and drawing

The frozen B renderer manifest and browser-lab provenance bind every original
pass by SHA256. Only the browser samples whose activities are `walk` or `carry`
are selected; every one of the 200 sprites alternates actual `walk_a`/`walk_b`
at 8 fps, with staggered phases. Idle/work/sleep samples are excluded.

Each texture is an actual 64×96 RGBA image: `Image.blend_rect` alpha-composites
the original body/top/hair/optional-item passes at (0,0). No cropping, scaling,
quantization, repainting, anchor correction, or screenshot pixel repair occurs.
Ground is a plain diagnostic meadow color. The separate ellipse is centred on
(32,88), radii (14,4), alpha 110/255; it does not repair body contact.

All 200 are real Sprite2D nodes at unit scale. Every process frame checks
viewport enclosure, visibility, window drawability and minimization. Frame
intervals come from `frame_post_draw`, not `_process` count or estimated FPS.
Input hashes and the script hash are rechecked at completion.

## Evidence and validation

External evidence directory:
`/Users/rexxa/work/arttrial3-data/b/crowd-diagnostic/`.

- `measurement.json`: raw frame intervals, 60 visibility records, hashes, metrics.
- `crowd_final.png`: unmodified Godot viewport readback, visually inspected;
  all 20 columns × 10 rows fit without viewport clipping.
- `sample_*_frame_*.png`: exact 64×96 composited textures used in the run.
- `run.log`, `host_during.txt`: real renderer and host contention evidence.
- `crowd.tscn`: external scene referencing `res://Trial3BCrowd.gd`.

macOS `screencapture` failed with `could not create image from display`, so no
OS desktop screenshot is claimed. Real-window evidence is the macOS/OpenGL
startup, window visibility/drawability samples and rendered viewport readback.

Godot parse-only and native editor import completed without script errors.
The editor generated `Trial3BCrowd.gd.uid`; no project settings or unrelated UID
files were manually changed. The script was unchanged from its measured SHA.

Command (from repository root):

```sh
bash tools/v8/run_bounded.sh 90 /Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
  --path art/proto --rendering-driver opengl3 --resolution 1280x960 \
  /Users/rexxa/work/arttrial3-data/b/crowd-diagnostic/crowd.tscn -- \
  --lab /Users/rexxa/work/arttrial3-data/b/lab \
  --render /Users/rexxa/work/arttrial3-data/b/render \
  --out /Users/rexxa/work/arttrial3-data/b/crowd-diagnostic
```
