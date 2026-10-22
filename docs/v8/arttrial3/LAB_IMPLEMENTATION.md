# Character lab implementation

Visual contract: `art/style05/STYLE.md`, `art/character/TEMPLATE.md` and
`LAYER_CONTRACT.md`. This is a standalone Godot art experiment, not game UI.
Geometry tokens: 64x96 upright / 96x64 sleep canvases, 104x112 grid cells,
10 columns at 1x / 5 at 2x, integer 1x/2x nearest display.
Paginated grid windows are at most1040x896 pixels; each page preserves scale. Calm Trial2b meadow/forest tiles
supply the background. No invented character fallback is permitted.

The lab loads `manifest.json` and `CHECKS.json` per route, requires the set
and every combination to pass, loads full-canvas layers and region masks,
and draws the checker's exhaustive combination list. Tab switches available
routes; G switches ground; 1/2 switches integer scale; left/right switch pages. Sleep uses its own
horizontal body and no upright overlay. Normal bodies loop idle, walk a/b,
work a/b; `--activity` selects idle/walk/work/carry/sleep for capture.

Invocation (from repository root):

```sh
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --path art/proto \
  --rendering-driver opengl3 res://scenes/character_lab.tscn -- \
  --route A --zoom 2 --out /Users/rexxa/work/arttrial3-data/lab
```

`--mode crowd --zoom 1 --seconds 60` renders exactly 200 walking people,
with Trial2b hut, hearth, trees and logs. Windowed frame-post-draw counts,
wall clock duration, frame times, source and asset hashes, viewport size and
population are recorded. Headless mode only checks loading and explicitly
cannot provide captures or P1 evidence. `--mode capture --activity walk`
writes eight 8-fps frame PNGs plus a lab PNG; encoding these PNGs into GIFs
is a separate pipeline operation. `--combo N` isolates a checker combination
(zero-based); without it the exhaustive grid is shown.

Asset absence or a failed structural set exits nonzero before drawing people.
Parser and missing-assets checks do not establish visual acceptance. Actual
windowed evidence and fresh GIF reviewers remain required after production.

## Capture orchestration

```sh
python3 -m art.pipeline3.capture_lab art/character/route_a/manifest.json \
  /Users/rexxa/work/arttrial3-data/lab-a all
```

Modes `grid`, `gifs`, `crowd` can be run independently. The launcher reruns
`check_set` before rendering. The scene verifies CHECKS input SHA256 values
before accepting a route. Every launch records command, stdout/stderr, asset
hashes and runtime data. PNG frame captures are encoded to looping GIF with
125 ms frame duration; the GIF format's palette encoding does not alter the
source PNGs. GIFs exceeding 2 MB reject instead of silently reducing quality.
Answer keys remain separate from blind-review images.

## Current verification

- Godot 4.6 `--headless --script res://CharacterLab.gd --check-only`: exit 0,
  no parse/error output (`arttrial3-data/lab-validation/parse.log`).
- Actual scene with no passing route: exit 1 and
  `CHARACTER_LAB_REJECTED no structurally passing route A; no placeholder substituted`
  (`arttrial3-data/lab-validation/missing-assets.log`). No captures produced.
- Python compilation succeeds; CLI usage imports all pipeline dependencies.
- Actual accepted-art render, UI key interaction, GIF readability, style and
  P1 remain **NOT_VERIFIED** until accepted production layers exist. Headless
  checks and synthetic fixtures are never visual or performance evidence.


## Pagination and real-window verification

The previous oversized grid was OS-clamped and did not cover all603
combinations. Those captures are historical invalid exhaustive-grid evidence;
their approximately20FPS is grid timing, never200-person crowd/P1 evidence.
The capture launcher now emits every page for both grounds and both zooms,
checks global combination indices, actual draw count and requested/actual
viewport equality, and rejects any page PNG above1,500,000bytes. It writes
`boards/pages.json`, per-page PNGs and representative `lab_a_1x.png` /
`lab_a_2x.png` (first meadow page, explicitly not the entire exhaustive set).
All pages can remain in the external evidence directory to respect art growth.

The601st–603rd combinations were observed together on the last2x page:
1040x224,127438bytes,1.000911s. Full first pages were then visually inspected:
1x indices0–79,80draws,1040x896,913864bytes;2x indices0–19,20draws,
1040x896,483957bytes. No clipped last row was observed. Evidence is under
`/Users/rexxa/work/arttrial3-data/lab-pagination-qa/`. These are actual windowed
OpenGL/Metal captures of the structurally passing incomplete subset, not full
route acceptance. Fresh reviewers and the complete page sweep remain separate.

`--allow-incomplete` on the Python CLI (Godot: `--allow-incomplete true`)
permits only a `NOT_COMPLETE` report with every supplied layer, cross-frame
check and combination passing. Only missing minimum inventory is tolerated;
missing inventory is recorded and `p1_eligible` stays false. The default still
requires complete route PASS. Runtime termination uses monotonic wall time,
not animation delta. Script SHA is captured at startup and a separate boolean
records whether the source changed during the run. Live command/log files
survive subprocess timeout. One60s page-observation timeout preceded the
successful rerun; its cause was not established, and no synthetic capture
was substituted.

## macOS visibility diagnosis and foreground measurement

The earlier crowd run produced 6,870 process iterations but only 87 rendered
frames over 60.031153 seconds. Its 1.449 rendered FPS is retained as a run with
unverified visibility, not silently replaced by its 114.44 process FPS.
During a subsequent stalled grid, Orca was frontmost; display-sleep prevention
was active through Godot, Amphetamine and caffeinate. Display sleep was not
supported as the cause by those observations.

Godot 4.6's [macOS occlusion callback](https://github.com/godotengine/godot/blob/4.6-stable/platform/macos/godot_window_delegate.mm#L361)
sets window visibility from the OS occlusion state. The [macOS draw predicate](https://github.com/godotengine/godot/blob/4.6-stable/platform/macos/display_server_macos.mm#L2554)
uses that visibility, and the [main loop](https://github.com/godotengine/godot/blob/4.6-stable/main/main.cpp#L4585)
skips renderer drawing when no window is drawable while process iterations
continue. This explains the observed discrepancy as an occlusion hypothesis;
the old run did not record enough visibility data to prove every hidden interval.

The harness now requests a real always-on-top window and moves it to the
foreground at startup. It records per-second `can_draw`, focus, rendered and
process counts, plus hidden process frames. It does not call force-draw,
substitute a headless renderer or infer present FPS from process FPS.
The crowd layout retains 200 people at 1x in a 1312x1088 window, with 64x96
full-canvas cells and the same small horizontal walking displacement.

Observed rerun: 60.009588 seconds, 4,734 rendered frames, **78.8874 rendered
FPS**. All 61 visibility samples had `can_draw=true`; hidden process frames
were zero. Focus samples were false while another application was active,
so foreground focus itself is not necessary once the window remains visible.
The capture is 1,313,374 bytes and all 200 conservative canvas-plus-motion
bounds fit. The screenshot was inspected; no last-row clipping was observed.
Evidence: `/Users/rexxa/work/arttrial3-data/lab-foreground/crowd/`, including
`runtime.json`, `population_bounds.json`, `godot.log`, `command.json` and PNG.
Source remained unchanged throughout. This is a diagnostic incomplete-subset
measurement: `dataset_status=NOT_COMPLETE` and `p1_eligible=false` remain.

The numeric 60 FPS bar is met by the 78.8874 FPS diagnostic measurement;
full-route P1 eligibility is still false because inventory is incomplete.
Post-fix grid checks also completed without hidden frames: 1x rendered 80
combinations in 1040x896 (929,500-byte PNG); 2x rendered 20 combinations in
1040x896 (496,980-byte PNG). Both recorded unchanged source and all sampled
windows drawable. These short grid rates are not crowd performance evidence.
