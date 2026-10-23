# Animal template A1 — ENV1 candidate whole figures

Status: CANDIDATE, not adopted. Family negative-control calibration currently FAIL;
Final round1: deer passes every reviewer/zoom group; wolf try3 fails reviewer D's
pose bar at both zooms (7/10, required 8/10), exhausting its three tries. See
`docs/v8/artenv1/ANIMALS.md`; neither species is owner-adopted by this pass.

## Geometry

- Upright canvas96×80, foot anchor(48,72). Dead canvas96×64, anchor(48,56).
  The GOAL's y72 cannot lie inside its64-pixel dead canvas. Dead preserves the
  same8-pixel bottom margin; this explicit interpretation does not change uprighty72.
- Deer S-stand withers-to-hoof height40px; wolf28px. Source S-stand shoulder
  landmarks are manually identified, not inferred from total antler/head height.
  Deer source y194→495: scale40/301; wolf final try3 source y252→474: scale28/222.
  The wolf landmark is the shoulder/back plane below the mane in front view.
  Historical try2 usedy270→517(scale28/247), retained in the round0 archive.
  Source camera foreshortening means E apparent shoulder height may differ;
  there is no independent facing or pose rescaling.
- One S-stand scale applies to all upright frames in both directions.
  Dead frames use opaque bbox long-axis body length: deer64px, wolf56px,
  allowed visible rounding ±4px. Dead must be wider than tall.
- Visible alpha≥16, opaque alpha≥128; exactly one8-connected opaque component,
  no visible pixel on canvas border, visible lowest pixel at anchor y, no opaque
  pixels on the original sheet boundary. Structural anatomy is reviewed visually.
- The route I `split_panels(rows=2)` identifies exactly10 major components,
  retains each complete connected figure and assigns dust without deleting it.
  Ownership reconstruction must equal raw source RGBA. Cropping, uniform Lanczos
  scale, whole-image placement are the only operations. Only alpha<16 outside
  the canvas may be discarded, with count/max logged. No pixel surgery,
  quantization, recolour, part assembly, or per-pose fit shrink.

## Sheet and paths

Two rows(S thenE), five columns:

| Species | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|
| deer | stand | walk | run/flee | graze | dead |
| wolf | stand | walk | run | attack/lunge | dead |

`art/animal/<wolf|deer>/<S|E>/<pose>.png`, matching JSON and species MANIFEST.
Whole figure carcass is the dead frame. W flips entireE image; noN view exists.
1world pixel equals1PNG pixel. Ground anchor is tile lower third at1×;
contact shadow is separate and is not baked into these sprites.
`boards/round1/<species>_sheet_1x.png` and`_2x.png` are current ordered sheets, not runtime atlases.
At½× and lower compositor uses wolf red/deer tan triangle markers as ENV1 requires.

## Acceptance

A whole sheet requires all10 structural and native-panel numeric passes, successful
family calibration including negative controls, image-model medium/line/light/palette
≥4/5, an executor reading, and two fresh reviewers. Each reviewer names species and
pose for each10 species frames at1× and2×: species≥9/10, pose≥8/10 independently.
Do not promote individual frames out of a failed sheet. Numeric pass, readability,
and owner adoption are separate evidence. Retain every failed generation.

Family calibration uses frozen wolftry2+deertry1 S-stand native panels as animal
anchors after actual image-model review against locked person+scene originals.
Each is held out against the other using the unchanged judge-v2 bounds. This is a
new-family calibration interpretation because ENV1 names no pre-existing animal
anchor files. The two generated animals do not become owner-approved references.
