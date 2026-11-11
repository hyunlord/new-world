# B2 climate grounds and flora coverage

B2 produces candidate art on `v8/art-batch-b2`. C2/C3 will import it later.
The production sequence is the required families, their variants, then a
contact-sheet self-review that regenerates the weakest 10–20%. This document
defines scope; manifests and review records establish what was actually adopted.

## Sources and ownership

- Owner briefs: `/Users/rexxa/work/art-batch/BRIEF2.md` and `BRIEF.md`.
- [Content bible issue 690](https://github.com/hyunlord/new-world/issues/690),
  v1.1: eras §1, catalogues §4.1–4.4, climate table **§4.5**. The brief's §4.6
  reference points to the count summary in the current issue.
- C2 priorities: `/Users/rexxa/work/c2-data/GOAL.md`. Bronze wild sources are
  produced first; their art does not activate currently inert content.
- Existing contracts: [flora addition](../flora/ADDING_A_SPECIES.md),
  [projection](../style05/PROJECTION.md),
  [flora catalog](../flora/species.json),
  [season defaults](../flora/habit_defaults.json), and
  [B2 style contract](STYLE_CONTRACT.md).
- Complete source snapshots, source hashes, round-one audit and the detailed
  requirements audit are in `/Users/rexxa/work/art-batch/B2/references/`.
  Runtime facts below were read from `origin/v8/main` at
  `4a44336e7782f2b27615d942b5cd54dbfee89406`; art references are based on
  `f182769fe06b1cb60105e4d0bf127b6524277838`.

| Lane | Records, manifests and boards | Adopted image namespace |
|---|---|---|
| Grounds and edges | `art/climate/grounds/` | `art/style05/climate/grounds/` |
| Herbaceous wild foods and fibres | `art/climate/herbaceous/` | `art/flora/climate/herbaceous/` |
| Shrubs and wetland plants | `art/climate/shrubs/` | `art/flora/climate/shrubs/` |
| Trees and wild fruit trees | `art/climate/trees/` | `art/flora/climate/trees/` |

Raw generations, rejected candidates, logs and review evidence remain outside
the repository under `/Users/rexxa/work/art-batch/B2/`. No runtime Rust,
GDScript, game assets, shared import configuration or other worktree is changed.
Existing source assets and catalogs remain intact.

## Climate and era interpretation

The existing runtime codes are `0 Mild`, `1 Neutral`, `2 Harsh`. These are winter
severity bands, not complete biome definitions. Ground and plant assignments
below that lack a bible origin are **art placement proposals**. Elevation
thresholds, spawn rules and climate selectors are not invented by this pass.

Wild sources can exist before their crops are domesticated. The bible's era
column describes cultivation or use, not the first existence of a wild plant.
Flora therefore has no metal tools, planted rows, farm trellises, fences or dyed
ties. Iron-era views reuse the same natural flora; the bible adds no new iron-era
plant species that would justify an artificial flora family.

## Grounds and edges

All eight named ground families are required. Each has spring, summer, autumn
and winter coverage, relevant climate tags and edge recipes or assets. The
brief does not require an ecologically misleading 3×8 biome cross-product.
Each of the three climate bands must be represented on the review boards.

| Required ground | Primary placement proposal | Seasonal distinction | Edge relationships to demonstrate |
|---|---|---|---|
| Tundra | Harsh open ground or upland | Thaw; low summer growth; tawny autumn; subdued frost/snow | Sparse vegetation to gravel and rock |
| Taiga floor | Harsh forest, optional neutral conifer forest | Needles and moss; autumn litter; winter snow pockets | Forest litter to open ground and rock |
| Steppe | Neutral or harsh open plain; dry mild variant where declared | Fresh spring grass; dry summer; autumn stubble; winter frost | Grass to exposed soil and gravel |
| Temperate meadow | Neutral plain; mild variant where declared | Fresher spring; green summer; straw autumn; muted winter | Meadow to litter, gravel and wet ground |
| Mediterranean scrub | Mild dry plain, hill or coast | Fresh spring; dry summer; muted autumn; cool winter green | Scrub soil to gravel and rock |
| Marsh | Wet ground or river margin across bands | Wet growth; straw litter; cold frost or mild wet winter | Wet mud to dry bank and water |
| River gravel | River margin across bands | Stable mineral identity; seasonal debris or cold frost | Gravel to water and neighboring ground |
| Highland rock | Hill or mountain across bands | Stable mineral identity; sparse growth; cold snow pockets | Rock to gravel or soil |

Ground is full-frame opaque top-down texture. A flat highland texture does not
replace a standing rock face. The ground manifest declares delivery dimensions,
edge orientation/connectivity and the neighboring samples used for seam checks.
Edges may reuse the existing deterministic, world-coordinate blending pipeline;
such edges are recorded as recipes or derived outputs, not new generations.
Texture rotations must not rotate the established lighting.

## Wild herbs, grains and fibres

Every row has full, harvested and winter states. Full stands are irregular wild
growth. Harvested grain and fibre stands visibly lose harvestable tops; fruit
and pod harvesting retains the living plant. Winter is a separately named
state, not a synonym for depletion. Shape or density variants are declared in
the lane manifest, with their source and state relationship.

| Production family | Climate and terrain evidence | State and identity notes |
|---|---|---|
| Wild grain / wheat / emmer | Bible: Mild/Neutral, reduced Harsh; plain/hill | Explicit B2 wheat coverage. Verified `wild_grain` code 3 supplies crop `emmer`; avoid cultivated field rows |
| Wild barley | Bible: all bands; plain/hill | Explicit B2. Verified `wild_barley` code 5; awned grain, then stubble |
| Wild einkorn | Explicit B2 request; separate ecology unspecified | Separate art candidate, not a renamed emmer or an invented runtime code |
| Wild lentil | Bible: Mild/Neutral, no Harsh; plain/river | Explicit B2. Verified `wild_lentil` code 7; low branching plant with small pods |
| Wild pea | Bible: Mild/Neutral, no Harsh; plain/river | Explicit B2. Verified `wild_pea` code 8; tendrils and pods, no farm trellis |
| Wild flax | Bible: all bands; river/plain | Explicit B2. Verified `wild_flax` code 2; slender stems and small leaves/capsules, not clover |
| Wild grape | Bible: Mild only; hill/coast | Explicit B2 and bronze priority; fruit-full, fruit-picked leafy vine, bare winter vine; runtime identity proposed |
| Wild rye | Bible: Neutral/Harsh, reduced Mild; hill/tundra edge | Bible completion. Current `wild_rye` code 6 is narrower: Neutral/Harsh, elevated plain |
| Wild millet | Bible: Mild, reduced Neutral, no Harsh; dry plain | Bible completion. Current `wild_millet` code 9 is Mild only |
| Wild hemp | Bible: Mild/Neutral, reduced Harsh; river | Bible completion and bronze priority; runtime identity proposed |
| Wild root | Existing bible wild source; current all-band plain/forest origin | Bible completion. Verified `wild_root` code 4; do not invent a named domesticated root crop |

**Oats coverage:** the bible groups rye and oats and gives them the same
`wild_rye` source. Sharing that source is intentional. A separate oat botanical
identity or runtime code is not asserted, and its absence is not an unattempted
explicit B2 family.

The exact herb/reed canvas, visible-height band, anchor and footprint are
preregistered in each lane manifest. Their anatomy does not inherit the tree
trunk-mask test merely because the generic normalizer is reused.

## Shrubs and wetland plants

| Production family | Evidence and placement | Full / harvested / winter interpretation |
|---|---|---|
| Harsh berry bush | Explicit B2 climate variant; existing generic berry origin allows all bands | Fruit-full; fruitless leafy; declared cold dormant form |
| Neutral berry bush | Explicit B2 climate variant | Fruit-full; fruitless leafy; declared winter form |
| Mild berry bush | Explicit B2 climate variant | Fruit-full; fruitless leafy; mild winter without mandatory heavy snow |
| Hazel | Bible's existing wild source; current all-band plain/forest origin | Nuts present; nuts removed; dormant form |
| Reeds | Explicit B2; wet margin placement proposed | Full culms/plumes; short cut culms; dry winter stems |
| Mediterranean scrub | Band-specific shrub extension, placement proposed | Full growth; depleted/browsed growth; mild winter form |
| Dwarf willow | Harsh-band shrub extension, placement proposed | Full growth; depleted/browsed growth; dormant winter form |

Berry climate variants are art choices for the existing `berry_bush` code 0,
not three new simulation species. Hazel is verified code 1. Reeds, scrub and
dwarf willow have no verified plant identity in the pinned runtime snapshot.
The last two are botanical art proposals, not newly authorized game content.
Any depleted meaning for non-food shrubs is recorded explicitly in metadata.

## Trees and wild fruit trees

The required oak, birch, pine and wild olive families use both small and large sizes.
The planner's 2026-11-11 economy instruction removes optional new silhouettes
and the added wild apple family from this pass's completion requirement.
Existing two-shape tree assets are reused. Two malformed apple attempts per
shape remain external rejects; they are not adopted or counted as required gaps. Their
four-season coverage follows the existing canopy-only renderer. Winter bare
frames are based on full shape 1 at each size, following the current tree
catalog's model; they are not claimed as two independently generated winter
shapes. Existing oak, birch and pine stumps are reused with source hashes.

| Production family | Full and harvested semantics | Winter semantics | Runtime status |
|---|---|---|---|
| Oak | Full small/large shapes 1–2; harvested wood uses existing oak stump | Bare branches at both sizes | Existing `oak` wood/art identity |
| Birch | Full small/large shapes 1–2; harvested wood uses existing birch stump | Bare branches at both sizes | Existing `birch` wood/art identity |
| Pine | Full small/large shapes 1–2; harvested wood uses existing pine stump | Evergreen crown with controlled upper-edge snow | Existing `pine` wood/art identity |
| Wild olive | Full small/large shapes 1–2; fruit-picked states retain leafy crown | Evergreen, cool mild winter; no forced bare tree or heavy snow | Bible Mild-only bronze wild source; plant and wood identities proposed |
| Wild apple | Full small/large shapes 1–2; fruit-picked states retain crown | Bare dormant tree at both sizes | Verified `wild_apple` code 10; current Neutral/Harsh forest origin |

Wild olive and the brief's olive tree are one species family with size/state
coverage. Fruit-picked olive/apple are not wood-harvest stumps. The pinned main
contains no `olive` wood material, so no draft may present `log_material: olive`
as an existing production contract. Apple likewise stays a wild-food proposal
unless its wood identity is separately verified by the importer owner.

## Import and completion boundaries

Current `plants.by_id` and `plants.by_category` bindings are empty. Terrain
bindings support Plain, Forest, Mountain, Hill and River, without a verified
climate texture selector. Import drafts therefore distinguish verified IDs from
proposed climate, state and season selection. Copying images is not proof of
their runtime selection or warm-up.

The current import configuration is `tools/v8/art_import.json`. Draft rows pin
the final immutable art commit and preserve other families. New assets remain
outside the live catalog until C2/C3 merges them and verifies game integration.

Completion evidence includes the family/state/variant manifest, original and
output hashes, rejected attempts, numerical and visual verdicts, native-scale
and nearest-2× reference boards, and the weakest-10–20% regeneration ledger.
Counts distinguish generated sources, adopted normalized frames, reused assets,
derived seasons and derived edges. The final Korean report is external at
`/Users/rexxa/work/art-batch/B2/REPORT.md`, with explicit branch push and matching
remote SHA evidence. It does not claim a game gate or play test was run.
