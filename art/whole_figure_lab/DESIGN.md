# Whole-figure evidence lab

## 1. Purpose
Anonymous native-size evidence viewer, matching the TRIAL4 laboratory. Labels expose only specimen number and zoom. Attribute answers remain external.

## 2. Palette
Paper `#e0bd8e`, ink `#18110a`, border `#7d6950`; meadow `#8a8242`, forest `#48442d`. Actual scene grounds use unchanged `art/style05/ground_v2` pixels.

## 3. Typography
System UI, body 16px, heading 24px, caption 14px; body line height 1.5.

## 4. Geometry
4px base unit; 16px inset/gap, 8px control padding. The default conforming contract uses a 104×112 stage and a 208×224 review panel (S/E columns, meadow/forest rows). Its diagnostic flag adds a separate 16px status footer.

Actual Round 0 and Round 1 use the nonconforming extent path: a 104×127 display cell, common alpha128 foot line y=94, and a 208×282 review panel including a separate 28px warning footer. At 2× that panel is 416×564. These larger cells preserve full extents but do not meet the standard sprite-canvas contract or certify S1/R1. All 2× output is nearest-neighbour enlargement, never fitted to viewport.

## 5. Components
Reuse the existing anonymous native GIF viewer: toolbar, specimen figure, status line. Overflow within figure preserves exact pixels. Borders only, no shadows added to character pixels. Canvas crowd is a separate measurement surface.

## 6. Motion
Native GIF playback. Eight source frames/second encoded as alternating 120/130ms. Whole sprite translation only for bob; no body-part transforms. Crowd draws all 200 complete sprites each requestAnimationFrame.

## 7. QA
Executed in both rounds: actual Chrome playback sampled at target 8Hz with timestamps and input hashes; full-page overview screenshots from viewports 375×900, 768×900 and 1280×900 (the saved images include the entire scroll height); individual capture viewports 375×900 at 1× and 448×924 at 2×. The recorder exercised scale, surrounding-ground and outline controls. Every first specimen screenshot crop matched a native GIF frame exactly (24/24 per round).

Hover, keyboard-focus, loading-failure and error-state interaction tests were not executed and are not claimed here. Crowd measurement was not run because the selected sample needed nonconforming canvases; P1 remains NOT_MEASURED_NONCONFORMING. Any future browser crowd timing would measure isolated rendering, not simulation/game FPS.

Native-size figures use an outline rather than a box-model border: an image exactly
as wide as the available viewport must not lose two pixels to border subtraction.

## PROD1 conforming surface
`prod1.py` uses 80×112 standing frames with foot line y104, or 112×80
sleep frames with foot line y72. A 112×112 stage preserves the full sleep
canvas; stage placement aligns both kinds at ground y104. Each stage draws
a 28×8 translucent ink contact shadow centred at (56,104), equivalent to
(40,104) in the standing sprite's coordinates. Review panels are 224×224,
with S/E columns and unchanged meadow/forest source-pixel rows. No diagnostic
input fallback exists. `prod1_crowd.js` uses a 2240×1120 canvas for 20×10
complete 112×112 cells. PROD1 browser evidence is pending accepted inputs.
