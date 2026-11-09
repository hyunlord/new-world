# A4 knowledge-map art contract

Art-only candidates for content bible issue #690. Source base `f182769fe06b1cb60105e4d0bf127b6524277838`; UI contracts inspected at game commit `4a44336e7782f2b27615d942b5cd54dbfee89406`. No game import is performed here.

## Medium and production controls

Use the locked `art/style05/anchors/props.png` and `scene.png`, plus adopted item references in `art/style05/polish/items/`. Fine brown-black ink, restrained earth wash, clear base and shadow tone, upper-left illumination. Standing subjects use STRUCT front three-quarter, approximately 20 degrees down; flat diagrams use plan view. No horizon, isometric grid, baked label or frame.

The outer-to-interior stroke target is **2:1**. For each principal material patch: wood has three broken longitudinal marks; stone/clay three cracks or facets; reed/fiber three parallel strands; hide/fur three edge tufts; metal two rim highlights without chrome. At most one shadow hatch cluster per object: three short parallel diagonal strokes, 45 degrees. These fixed counts are prompt and visual-review targets, not pixel measurements recovered from the generator. Natural material edges and essential semantic details are not counted as decorative marks.

Sepia ink, bone highlights, ochre, slate, muted sage. Copper is subdued terracotta; bronze golden brown; iron charcoal gray. Amber fire is the warmest accent. Strong silhouettes and selective marks must survive 32/48/64px comparison. No dense cross-hatching, photorealism, 3D rendering, pixel quantization, contour repainting, or invented palette remapping.

## Canvas, anchors and projection

| Family | Adopted canvas | Center anchor | Footprint | Review |
|---|---|---|---|---|
| Knowledge icon | 64x64 | 32,32 | null: UI | 32/48/64 on dark and light |
| Era vignette | 264x88 | 132,44 | null: UI | actual size and 2x |
| Chapter illustration | 520x260 | 260,130 | null: UI | actual size and 2x |

These are **new candidate dimensions**, inferred from current card/header width 264 and chapter text width 520. Existing UI does not yet contain image slots. UI center anchors are intentionally distinct from world-sprite feet `(32,43)`; world height bands and seam checks are not applicable. Camera and light still require visual checks.

Generated alpha is preserved. Mechanical operations only: alpha-bounds crop with fringe, proportional Lanczos fit, transparent canvas padding. Icons fit within 58x58; banners/chapters retain at least two pixels of outer padding. Do not stretch, sharpen, recolor, invent alpha from an opaque backing, or repair a weak subject in code.

## Era and catalogue honesty

The exact union contains **167 concrete knowledge IDs**: bronze 48, neolithic 50, iron 37, paleolithic 32. The bible headings 55/50/~190 are targets, not fully enumerated IDs. `basket_loom -> loom` is an ambiguous placeholder, recorded separately rather than inventing knowledge IDs.

Paleolithic/neolithic contain no metal or dyed clothing. The live neolithic `dyeing` icon represents pigment roots next to undyed fiber. `mud_brick` represents raw clay and a wooden mould, not a finished brick house. Bronze permits copper, tin, bronze, wool, linen and finished mud brick; iron adds dark forged iron. No steel/chrome-looking bronze tools.

Chapter assets use semantic IDs. Current chapter 6 `neolithic_village` conflicts with planned chapter 6 `first_copper`; both illustrations remain distinct and the importer must resolve numbering. Localization stays in the UI.

## Evidence and acceptance

Each generation records exact prompt, actual artifact path, raw hash, attempt, tool, unavailable seed, reference delivery mechanism, and self-review. Raw files live outside this checkout at `/Users/rexxa/work/art-batch/A4/`. Built-in image generation receives the viewed originals in conversation context; records do not claim explicit reference-file attachments.

Adoption requires nonempty alpha, transparent margins, correct dimensions/hash/anchor, semantic and era review, and a separate visual reviewer comparing both existing art and the A4 family. Visual verdict threshold 90; medium/line/light/palette each at least 4/5. General vision review is named as such and is not relabelled as the historical image-generation-model judge.

The existing item numeric judge is also run and retained as a diagnostic. Its calibration concerns world item textures, not the new sparse UI family: report every failed metric rather than silently claiming that the legacy gate passed. New UI structural checks plus the explicit visual verdict are the candidate-selection gates. No owner approval or game-readiness is implied.

## Variants

Knowledge is depicted once per node. The bible's four states (stable, endangered, unknown, inert) are UI treatments, preserving the same subject: full color, outline warning, dimmed, gray with diagonal annotation. The draft bindings describe these treatments separately; no seasonal, depleted, count-bucket, or damaged-object sprites apply to knowledge nodes or chapter/era headings. Chapters and banners carry no painted text so KO/EN titles use the same art.
