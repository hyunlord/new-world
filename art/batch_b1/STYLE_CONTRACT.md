# B1 — People through the eras

This is an art-only candidate pass on `v8/art-batch-b1`, based on
`f182769fe06b1cb60105e4d0bf127b6524277838` (`origin/v8/art-trees`).
Sources: both ART-BATCH briefs, content bible issue 690 v1.1, C2 GOAL,
`art/character/TEMPLATE_V3.md`, and the A1–A4 reports dated 2026-11-10.
Raw generations, trial frames, rejected attempts and measurements live in
`/Users/rexxa/work/art-batch/B1/`, outside this repository.

## Production contract

TEMPLATE_V3 and the final adoption note in ADR 0018 supersede the original
layered cut-out rig. Wardrobe changes are complete Route I figure sheets.
Held-tool and carried-load alternatives are complete figure frames too.
Do not cut limbs, recolour garments, or revive the historical rig to simulate
part swaps. The live importer has no job-specific held-tool selector; the
supplementary frames require an explicit C2/C3 selector integration.

Standing canvas 80×112, anchor (40,104); sleep 112×80, anchor (56,72).
S-idle supplies a common uniform scale for every standing pose in both
facings: adult target 56px, child 40px. Sleep uses its own long-axis scale.
Use the existing ownership, normalization and whole-figure checks unchanged.
All ten panels must pass before a sheet can contribute adopted frames.
No border touch, detached opaque pixels, hidden resizes or painted repairs.
N reuses S; W mirrors E at display time. Shadows remain a renderer layer.

Inspector portraits are 128×128 warm-parchment busts, preserving wardrobe
neckline/shoulder coverage and hair silhouette. Six identities per look
follow the existing skin/hair/age/beard variant plan; two adult identities
are older. The appearance grid has only child/adult, not an elder axis.

## Frozen visual targets

- Use I-1 and adopted Route I figures as the person style anchors; use the
  existing portrait anchor for portrait medium only, never its accessories.
- Warm dark-brown ink, restrained earth-tone wash, upper-left light,
  approximately 20° downward front three-quarter view; E faces right.
- Main silhouette : internal line weight target 2:1. No uniform heavy cartoon
  outline and no noisy all-over texture. Compare at native size and nearest 2×.
- Cloth: 3–5 main fold strokes per garment; one shadow group of three parallel
  `/` strokes. Leather: three seam/fold accents. Wool: 3–5 broad folds with a
  few soft edge interruptions, not shaggy fur. Hair: three coherent clumps
  with selective strand marks. Wood: three broken grain accents per major
  exposed piece. Stone/clay: three face/crease accents. Metal: two edge
  highlights, no white chrome shine. These are prompt/review targets, not
  automated stroke-count measurements.
- Neolithic plant fibre and first wool are explicitly authorized by BRIEF2.
  No metal or dyed garments in paleolithic/neolithic candidates. Bronze
  permits copper/bronze/tin, wool/linen and simple earth dye under the bible's
  dyeing chain. Iron tools appear only in iron sets. No modern fasteners,
  collars, printed patterns, jewellery, steel jaws in bronze casting tongs,
  or unrequested civilization-specific insignia.

## Scope and variants

Seven new top IDs, each covering the 12 ordered age × sex × hair combinations:

| Top | Earliest era | Wardrobe |
|---|---|---|
| neo_plant | neolithic | undyed plant-fibre tunic, cord belt |
| neo_first_wool | neolithic | first natural wool wrap, cord tie |
| bronze_linen | bronze | cream linen tunic, tied leather belt |
| bronze_wool_cloak | bronze | oatmeal wool tunic and natural brown cloak |
| bronze_earth_linen | bronze | muted ochre/russet linen, simple tied belt |
| iron_work | iron | linen tunic with leather work apron |
| iron_winter | iron | natural dark wool tunic and warm knee cloak |

Existing paleolithic hide/loincloth/fur looks are retained by reference.
Earlier textiles remain valid in later eras. Winter clothing, six portrait
identities, tool choices and filled/empty carried containers are the planned
variant classes. No arbitrary damaged-human or count-bucket variants apply.

After the first complete family contact sheet, regenerate the weakest
10–20% of each lane's sheets. Preserve the earlier images, judge the new
attempt against the same targets, and retain the better passing candidate.
An old passing result is never retroactively described as a new generation.

## Acceptance and records

Each generated source has its exact prompt, reference paths and hashes,
source bytes/hash, tool-return record, attempt index and seed `null` when
the generator does not expose a seed. Failed calls are separate from images.
Structural checks and a recorded visual-verdict of at least 90 are required
for candidate selection. Source-resolution legacy numeric style diagnostics
are retained with their real outcome; missing historical model scores are
not reported as passes. Visual reviews are labelled, not blind user tests.
Sidecars hash-bind source, prompt, normalized output and review summary.
Counts distinguish generated sheets, frame slots, portraits, adopted outputs,
rejected attempts and superseded passing alternatives.

The import draft preserves the old grid and fallbacks, appends new top IDs
in the same order as the proposed `appearance.ron` note, and explicitly
describes source-pattern/portrait/mask/face-plan integration. No runtime
files are changed and no new simulation code IDs are invented.
