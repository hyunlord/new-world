# A1 animal production rules

Status: production specification; generated frames remain **PENDING** until their
recorded structural and visual reviews pass. This document does not certify any
asset or supersede a historical owner verdict.

## References and medium

Attach the actual files for every generation and retry:

- `art/style05/anchors/person.png`: locked contour/detail and wash reference.
- `art/style05/anchors/scene.png`: locked palette, material and lighting reference.
- `/Users/rexxa/work/artpolish-data/animals/deer_try2.png`: animal layout and
  herbivore reference, with normalized frames in `art/animal/polish/deer/try2/`.
- `/Users/rexxa/work/artpolish-data/animals/wolf_try2.png`: additional canine
  anatomy/layout reference, with normalized frames in `art/animal/polish/wolf/try2/`.

Person plus scene are required; choose the relevant animal sheet as the additional
reference. Record the actual attachment list and byte hashes for each call. Listing
a reference here does not prove it was attached. The historical animal sheets are
comparators, not a new claim that their old pending owner decisions were resolved.

Use controlled brown-black ink over restrained earth-coloured wash, following
`art/style05/STYLE.md`. Keep broad lit planes and one clear shadow plane, with
natural local wash variation. Never quantize the result into literally two colours.
Natural hide, bone, cream fleece, ochre, umber and charcoal sit within the reference
palette. Do not invent measured colour values or physical paint-opacity figures.

## Fixed family mark budget

These are generation and visual-review targets, not post-processing instructions.
Count clusters on the visible main body plane; occluded far-side surfaces need no
duplicate marks. Tiny faces and hooves use only the marks needed for anatomy.

| Material | Fixed marks | Review constraint |
|---|---|---|
| Short hide / shorn coat | 3 clusters, each 2–3 short marks | Shoulder, flank and haunch; broad quiet planes between clusters |
| Fleece | 6–8 broad connected curls; 2 shadow clusters | Read the outer fleece mass first; avoid dense all-over spirals |
| Bristle / mane / feather group | 3 groups, each 3 marks | Group at neck/back/tail as anatomy permits; no detached hairs |
| Horn | 0–2 ring marks | Broad attached base and connected tip |
| Antler | Structural branch outlines only | Connected branches; no detached thin tips or decorative stipple |

Main contour to interior line width is **2:1** throughout the family. Hatching
descends from upper left to lower right. Light comes from upper left; the shadow
wash remains below/right. Avoid dense cross-hatching, fuzzy airbrush shading,
photoreal fur, modern breed styling and decorative outlines added in processing.

## Camera, canvas and ground contact

- One world tile is 64×64 px. Animals use the existing front three-quarter camera,
  about 20° down. S shows the front; E faces screen right in side three-quarter.
- Upright frames: **96×80**, ground anchor **(48,72)**. Dead frames: **96×64**,
  anchor **(48,56)**. Dead keeps the existing 8-px bottom margin.
- Whole figure only, true transparent RGBA, generous source gutters. No ground,
  cast/contact shadow, text, labels, cell borders, equipment or background scenery.
- At runtime the declared sprite anchor aligns to the tile's standing base
  `(tile_left+32, tile_top+43)` per `art/style05/PROJECTION.md`; no runtime change is
  applied by this pass. Record each frame's measured alpha footprint separately.
- Derive one uniform upright scale from manually recorded S-stand shoulder-to-hoof
  landmarks; use it unchanged for every upright pose and both facings of that set.
  Antlers, ears and head height are not shoulder landmarks. Never independently
  fit-shrink a pose or facing. Dead has its declared long-axis length target.
- `catalog.json` holds the proposed family sizes. They are art production
  parameters, not biological measurements or simulation balance data. Any change
  must be reasoned and recorded before normalization, not made to hide clipping.

Young use approximately **0.6×** the corresponding adult target, rounded to whole
pixels. Shorter muzzle, larger head relative to body and immature horns make age
readable as well as size. These traits are production choices, not new content data.

## Pose reading and species identity

Every set has S/E stand, walk, run, graze and dead. Wild-capable sets additionally
have S/E attack. Stand has a settled base; walk has a lifted bent foreleg and
opposite hind step; run has extended stride; graze lowers the muzzle to foot level;
dead is limp, horizontal and non-graphic. Attack must differ from run in the head,
neck or striking action. For dog, `graze` means head-down sniff/forage; for birds it
means peck. These are art pose aliases, not claims that dogs graze grass.

Use the catalog's silhouette cues to distinguish species at 1×, then verify at 2×.
Do not bake wild/penned markers into the animal body: C2 owns that separate state
marker. Bare primitive animals keep the same body art across later eras. No metal,
dyed cloth, bell, harness, armour or anachronistic breed accessories are needed.

## Acceptance record

Keep raw generations, exact prompts, tool/model response records, available seed
metadata, source hashes, crop ownership and rejects outside the repository under
`/Users/rexxa/work/art-batch/A1/`. If the tool provides no seed, record `null` and
the reason; never manufacture a seed or model version.

Use the existing whole-figure checks: correct canvas; visible alpha ≥16; opaque
alpha ≥128; exactly one 8-connected opaque component; no visible canvas-border
pixels; ground contact at anchor y; no source opaque-border clipping; ownership
reconstructs raw RGBA; no discarded visible pixels; dead wider than tall with the
declared length. Cropping, uniform resizing and whole-image placement are the
allowed normalization operations. Animal sprite seams are **not applicable**;
check cutout edges and atlas gutters instead of claiming tileability.

Inspect each native-scale and 2× contact sheet beside the existing deer/wolf and
locked style references. Record species, pose, anatomy, marks, line ratio, wash,
palette, light and projection differences. Use a persisted visual-verdict JSON
before a retry. A numeric pass alone is not adoption. Do not relabel old numeric
style calibration as a new pass or claim an image-model review that was not run.
Adoption requires the current pass's explicit recorded verdict; failed sheets and
frames retain their reasons and provenance.
