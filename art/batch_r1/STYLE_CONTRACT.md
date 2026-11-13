# R1 — shipped-rig wardrobe replacement

This art-only pass implements ART-BATCH round 3 R1 and BRIEF4 R1 pass 2 on `v8/art-batch-r1`,
based on `f182769fe06b1cb60105e4d0bf127b6524277838`. Generation records and
rejected raw sheets live in `/Users/rexxa/work/art-batch/R1/`, never here.

## Scope and frozen sources

Seven wardrobe IDs, twelve age/sex/hair combinations each, in production order:
`neo_plant`, `neo_first_wool`, `bronze_linen`, `bronze_wool_cloak`,
`bronze_earth_linen`, `iron_work`, `iron_winter`. Child/adult × M/F ×
short/long_tied/bun. No simulation state, live importer, appearance codes or
renderer files are changed. Earlier hide, loincloth and fur remain shipped.

The matching shipped `art/character/route_i` figures are authoritative, not B1.
Use hide_wrap references for plant/first wool/linen/work; fur_cloak references
for cloak/winter. Adult male long_tied uses the explicitly shipped bun fallback
as measurement reference, while the proposed new hair must actually be tied.
Record both identities rather than pretending a missing source exists.

Generate complete figures by editing the matching original source sheet.
Preserve the stocky shoulders/hips, skin, head scale, outline and actual poses:
idle, walking step, axe raised overhead, logs over shoulders, horizontal sleep.
S and E remain the shipped directions; N reuses S and W mirrors E at display.
Do not replace generic wardrobe work with standing or shoulder loads with chest bundles.
Supplemental activity sheets are explicitly separate: their declared grinding,
spinning, hoe or tongs work and named carried loads replace only those actions
(see `TOOLS_SCOPE.md`); all source/body and strict measurement gates remain.

## Shape and image acceptance

Reuse TEMPLATE_V3 ownership, uniform S-idle-derived standing scaling and
separate sleep scaling unchanged. Ten complete source components are required;
all ten normalized frames must pass `whole_checks.check_whole`. Standing
80×112, anchor(40,104); sleep112×80, anchor(56,72), footprint(1,1).
Pass 2 permits recorded deterministic whole-sheet colour correction when geometry
already passes. `retry.py` applies spatially uniform HSV/ink gains, preserving
alpha and dimensions; skin/hair changes require renewed visual review. Its
recorded `ink_cutoff` controls the colour curve only, never the frozen
alpha16 measurement probes or acceptance thresholds. No
probe-specific edits, stretched bodies, limb surgery or halo repair are permitted.

Each new look must also pass all five S-idle comparison ratios in [0.9,1.1]:
shoulder width, hip width, clothing mean HSV saturation/value, outline thickness.
Reviewed per-reference pixel annotations are in `regions/`; their overlays,
reference values and method history are external evidence. Empty/invalid
measurements are failures, never exemptions. Source and candidate are measured
at the actual game size; reference masks are frozen before adoption.
The frozen clothing polygon uses every visible alpha≥128 garment pixel and
requires at least90% coverage in both frames; skin, trim and accessories are
excluded by annotation, never by colour. HSV means are over this declared
garment interior, not an inferred whole-image clothing segmentation.
Outline thickness is effective dark-ink coverage: along each fixed probe,
sum alpha/255 × (1−maxRGB/255) over the first two alpha≥16 pixels, then
average probes. It measures antialiased edge darkness in native pixel units,
not an integer stroke run. The earlier binary-run diagnostic is not a gate.
Both methods and all22 masks were frozen before any adoption.
Other poses require the ten-frame structural and side-by-side visual checks.
The numerical measurements alone do not certify pose or historical materials.

Visual review compares every pose at1× and nearest2× on a common background.
It must score at least90 and bind every frame hash before promotion. Rejected
whole sheets supply no individual accepted frames. Pass 1 used at most three
image calls per asset; Pass 2 body limits are specified below and include
failed calls. Do not relax a gate to meet a quota.

## Surface language and era honesty

Warm dark brown ink and saturated tawny earth wash from shipped figures,
upper-left light, same camera. Main contour:internal line target2:1. Cloth
3–5 broad folds and one group of3 parallel `/` hatches; wool sparse soft edge
interruptions, not shaggy fur; leather3 seam/fold accents; wood3 broken grain
accents; metal2 restrained edge highlights. These mark counts are prompt and
visual targets, not automated stroke-count claims.

Neolithic plant fibre/first natural wool are authorized by BRIEF2/R1; no metal
or dyed cloth. Bronze adds linen/wool and simple earth dye. Iron alone permits
iron tools. No modern collars, closures, badges or decorative hardware.
Portraits follow the existing six-identity FACE_VARIANTS order and parchment
128px format; Pass 2 starts them only after the whole wardrobe reaches 12/12. A missing body is not silently supplied
with a B1 or shipped image under a new wardrobe name.

## Pass 2 budget and completeness

BRIEF4 raises the total body attempt limit to six per combination, including
Pass 1 attempts and deterministic edits. Record each approach and prior failed
ratios before execution; no unrecorded parameter sweeps. Frozen measurements,
annotations, thresholds and structural checks remain unchanged. A wardrobe
needs all twelve accepted bodies before portrait work and draft importability.
Partial wardrobes are evidence only, with no fallback aliases.
