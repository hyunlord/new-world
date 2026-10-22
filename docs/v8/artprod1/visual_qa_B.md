# Visual QA Pass B

VERDICT: PASS (diagnostic UI rendering only)
CONFIDENCE: HIGH for supplied static surfaces; playback evidence limited as below.

Scope: PROD1 diagnostic GIF lab, not readability scoring, formal R1, or production-art acceptance. The intended raster sprites are legitimate content in a real HTML viewer. No implementation changes made.

## Evidence trace

Directly opened all six requested PNGs: `visual_diff/reference.png`, `visual_diff/actual.png`, and `lab/browser/overview/{375.png,768.png,1280.png,forest_2x_outlines_off.png}` relative to `/Users/rexxa/work/artprod1-data/round0`.

Every field of `visual_diff/diff.json` consumed: command `image-diff`; dimensionsMatch true; reference width 224, height 248; actual width 224, height 248; totalPixels 55552; diffPixels 0; diffRatio 0; similarityScore 100; alphaChannelIntact true; hotspots []; summary `100/100 similarity; 0/55552 pixels differ.` There are zero hotspots and therefore zero hotspot causes. This matches the directly viewed specimen pair, not a whole-page reference comparison.

## Findings

- No blocking rendering defects in supplied captures. At 375 px, controls wrap and the specimens form one column; at 768 px there are three columns, and at 1280 px five. Cards, captions, and status text remain separated without visible overlap or clipping. The 375 px long capture is scaled down by the image viewer; source sizes are preserved by the page, not inferred from that preview.
- Native dimensions: browser_lab.js lines 16–17 assigns manifest width/height; browser_lab.css line 16 uses pixelated rendering and no max-width resampling. Manifest panels are 224×248 at 1× and 448×496 at 2×. These are two-facing/two-ground composites plus diagnostic footer, not the individual 80×112 standing and 112×80 sleep canvases. prod1_diagnostic.py lines 56 and 90 explicitly preserve those sprite canvas dimensions; lines 134/147 use nearest-neighbor scale.
- Transparency: both grounds show through the sprite surroundings without black/white rectangular fills or visible alpha seams. Diff alphaChannelIntact is true. The composed GIF panels intentionally contain opaque grounds; this does not independently certify original per-sprite alpha values.
- Missing markers are visible and intentional: specimen 05 has left-facing MISSING markers on both grounds; specimen 08 has four MISSING markers. They are neither hidden nor replaced with a fabricated person. These remain missing production inputs, not successful art coverage.
- Typography: English labels, status, captions, and both diagnostic footer lines are present. No visible baseline clipping, tofu, or detached labels. CJK precision is N/A because the supplied UI has no CJK text.
- Forest/2×/outlines-off capture shows two columns with larger native panels, unchanged composed meadow/forest rows, and absent outer outlines. The control is explicitly named “Surrounding ground”: CSS changes the figure backdrop, not the baked composite grounds, which occupy the panel. Thus that toggle has little visible effect for these opaque composites; it is not evidence of ground recomposition.
- Responsive limitation: 2× at 375 px is not among supplied captures. CSS line 15 provides figure-local horizontal scrolling when a 448 px native image exceeds available width. This preserves pixels but does not prove narrow 2× scrolling through screenshot evidence. No page-wide overflow is visible in the supplied 1× captures.

## Playback boundary

`lab/browser/playback.json` records Chromium 155.0.8059.85, device scale 1, target 8 Hz, three seconds and 24 actual browser screenshots per specimen, no page errors, and native GIF playback without seek/canvas/frame substitution. 14 of 24 scale/specimen records have VISIBLE_CHANGE; 10 are STATIC_OR_UNOBSERVED. Sampled moving GIFs contain eight frames alternating 120/130 ms, totaling one second, consistent with average 8 Hz under GIF time quantization. Sampled static specimen 10 has one 1000 ms frame. These facts support native playback infrastructure, not a claim that every diagnostic item animates or that every presentation interval was exactly 125 ms. Static overview captures alone cannot prove motion quality or continuous cadence.

## What is good / blocking

Keep native pixels, wrapped controls, explicit MISSING markers, two grounds/two facings, and the prominent `DIAGNOSTIC_REJECTED_SET_NOT_CERTIFIED` / `formal_R1_eligible=false` language. BLOCKING: none for this bounded UI-rendering verdict. Production completeness, semantic readability, formal R1, and art acceptance are not assessed or passed.
