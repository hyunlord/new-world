# TRIAL5 whole-figure lab

This isolated evidence lab reads the frozen twelve-look selection in `samples.py`.
It never generates characters or installs assets into the game.

```sh
python3 -m art.whole_figure_lab.build art/character/route_i /Users/rexxa/work/arttrial5-data/production/lab
python3 -m art.whole_figure_lab.comparison art/character/route_i /Users/rexxa/work/arttrial5-data/production/lab/boards
python3 -m art.whole_figure_lab.browser /Users/rexxa/work/arttrial5-data/production/lab capture
python3 -m art.whole_figure_lab.browser /Users/rexxa/work/arttrial5-data/production/lab crowd
```

Requires the existing Pillow, NumPy and Playwright environment. No new dependencies.
Build refuses missing sprites, failed frame sidecars, or changed PNG hashes.
The source set's 100%/72-sheet completeness remains the production checker's gate;
this lab checks its selected inputs and cannot certify that full-set gate.

Outputs: 24 native GIFs, six transparent style stills, 24 exhaustive board pages,
two summary boards, private answer key, source hashes and a separate crowd manifest.
Only public GIF manifests enter the blind viewer. `comparison` reuses the exact
historical ABP board pixels and logs the unsupported-N and sleep-clothing mismatch.

The browser recorder adapts the existing TRIAL3b implementation with a 900px
minimum viewport height so wrapped controls cannot clip the captured image. It screenshots
actual native playback at target 8Hz with per-sample timestamps, never seeks GIFs,
and records actual cadence rather than assuming timing. Crowd measurement is a
1600×1000 Canvas2D view containing all 200 native-size sprites, drawn each rAF for
60 seconds. Report it as isolated browser rendering; it does not measure Godot or
simulation performance, nor prove display scanout.

Raw source/measurement PNGs, browser screenshots, private answer keys and runtime
samples remain outside the repository. Derived review copies of 24 GIFs per round
and selected boards are published under `art/character/trial5_diagnostics/round0/`
and `round1/`, preserving their diagnostic status and provenance. The owning
executor verifies each committed copy is at most 1.5 MB; these copies are review
artifacts, not accepted game assets. Native 2× specimen
panels overflow inside figures on small viewports intentionally, preserving pixels.

## Explicit diagnostic review

Rejected sheets can be shown only when all five normalized whole PNGs and their
original sidecars exist. `--diagnostic` never accepts raw panels, missing identities,
stale hashes or mismatched attempts. It does not change the production default.
Provide an external JSON selection with all 24 preregistered look/facing keys:

```json
{"sheets": {"adult_M_hide_wrap_short/S": "/absolute/slices-v2/adult_M_hide_wrap_short/S/try1", "adult_M_hide_wrap_short/E": "/absolute/slices-v2/adult_M_hide_wrap_short/E/try1"}}
```

The example is abbreviated; all keys from `samples.PEOPLE` × S/E are required.
Paths can also be relative to the selection JSON. Each directory must contain
`sheet_checks.json` and `{idle,walk_a,work_a,carry,sleep}.{png,json}` from the same
attempt. No automatic best/latest attempt search or identity fallback is used.

```sh
python3 -m art.whole_figure_lab.build /absolute/diagnostic_selection.json /absolute/diagnostic_lab --diagnostic
python3 -m art.whole_figure_lab.browser /absolute/diagnostic_lab capture
```

Every GIF/panel adds a separate 16px footer `DIAGNOSTIC - NOT ACCEPTED` outside
the image area; source figures retain exact pixels and scale. Boards repeat the
status. Dataset/browser/crowd manifests retain `diagnostic_not_accepted`;
`provenance.json` preserves full original per-frame check/judge and per-sheet
summaries, plus exact input/selection hashes. Transparent style stills and crowd
textures remain unmodified and must always travel with this status/provenance.
A diagnostic review is evidence of failures, never structural acceptance.

`comparison SELECTION_JSON OUTPUT --diagnostic` supports the same explicit format;
include its six historical-comparison look/facing keys as well as the review keys.
It labels the I column diagnostic and stores each original frame sidecar in the
comparison provenance. Historical A/B/P pixels are unchanged.

## Nonconforming raw-extent exploration

When normalized canvases fail but v3 owned native crops exist, use the separate
command below. This is explicitly **not** delivery or a qualifying R1 review.

```sh
python3 -m art.whole_figure_lab.extent SELECTION_JSON EXTERNAL_LAB
python3 -m art.whole_figure_lab.browser EXTERNAL_LAB capture
```

The selection adds `native_sheets`, with the same 24 keys as `sheets`, mapped to
`panel-judges-v3/<look>_<facing>_tryN` directories. Each must contain all five
`<frame>_native.png` files and `native_panel_judges.json`. The original diagnostic
sheet directory still supplies identity, scale, ownership and failed results.
Native hash/raw identity must match; rejected source pixels must equal the native
crop whenever both exist. No fallback identity, frame, pose or direction is used.

All five full owned crops receive exactly the sheet's idle-derived scale, once,
with Lanczos. No trimming, opacity thresholding, repainting or canvas clipping is
applied. The common cell grows to fit the entire resized bounds including alpha
tails and the prescribed 1px bob. Every frame's alpha128 bottom aligns with one
recorded common foot line. Enlarged review cells use tiled unchanged backgrounds.
A separate footer states nonconformance; all sidecars/manifests retain
`diagnostic_nonconforming_canvas` and `S1_R1_eligible: false`, plus the original
normalization failures. The content is meaningful exploratory animation evidence,
not a repair of rejected standard canvases. This mode does not produce crowd FPS.

Round 0 and Round 1 actual playback QA also compare the first screenshot crop of every GIF
against its decoded native frames; all24 must match exactRGB before reviewer handoff.
Earlier clipped-view QA captures are retained separately, never used in blind packets.

## PROD1

The historical TRIAL5 entrypoints remain unchanged. Production entrypoints:

```sh
python3 -m art.whole_figure_lab.prod1 ROUTE OUTPUT
CHROMIUM_EXECUTABLE='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' python3 -m art.whole_figure_lab.prod1_browser OUTPUT capture
CHROMIUM_EXECUTABLE='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' python3 -m art.whole_figure_lab.prod1_browser OUTPUT crowd
```

OUTPUT is outside the repository. Builder preflights all ten frames per look,
requires exact sidecar image hashes plus structural and numeric panel passes,
and selects twelve distinct looks with the specified demographic/hair/top
coverage. Insufficient accepted input fails before output creation. Names are
`p1_01..12_{1x,2x}.gif`. Review answers remain in `answer_key.private.json`,
never in the published blind viewer. `prod1_boards.boards` generates the full
36-look sheet (missing looks marked) and ten-state boards; `comparison(route,
output, records_path)` generates `compare_ABPT2_2x.png` from the frozen TRIAL4
six-case mapping and discloses N-to-S proxies and missing accepted cases.

Playback capture targets 8 Hz and records actual timestamps, not a claim that
each requested sample arrived on time. Crowd uses 200 fully visible sprites,
60 seconds and native 1× pixels. Its Canvas2D rAF measurement is isolated browser
rendering, not Godot, simulation, or display-scanout performance.
