# Assembly review — §5 / §6.1

Read-only review of `build_boards.py` and existing original cutouts on 2026-10-22. Only this review file was written. No art was edited, no generation performed, no boards assembled by this reviewer. This is a pre-assembly review, not visual acceptance of final boards.

## Findings

### A1 — Medium: the 56 px target measures the whole carried-object silhouette, not the person

`build_boards.py:149` obtains the bounding box of every non-magenta pixel; `:157` resizes that entire box to 56 px. §6.1 and decision D05 say the **person** is approximately 56 px high. Logs projecting above the head (and axes or residual ground below the feet) make body height smaller, with a concept-dependent scale bias. The output manifest nevertheless calls this `person_height: 56`.

This is material for a comparison of tiny activities: visual inspection of the actual `06_cave_ochre/C2_1.png` shows the logs reaching approximately y=34 while the hair crown starts around y=110–140 and the lowest foot ends around y=1125. The measured keyed bounding box is `(157, 34, 1113, 1125)`. Thus the body would be roughly 50–52 px, not 56; the numerical crown estimate is manual and approximate. The same issue is visible in `02_folk_woodcut/C2_1.png` (keyed box `(156, 29, 1207, 1176)`). Different carried-item proportions receive different effective body scales.

Action: record manually inspected crown-to-lowest-foot vertical bounds for each selected cutout, derive scale from that body height, and apply that uniform scale to the entire unretouched keyed sprite (allow its carried items to extend beyond 56 px). Keep source bounds and resulting body/sprite dimensions in assembly evidence. Align feet if comparing equal game scale. This is permitted downscaling, not art retouching. If retaining current implementation, label the quantity honestly as `sprite_height`, explicitly disclose the scale deviation, and do not assert that the preregistered body scale was met. Prefer correcting before blind review.

### A2 — Medium integration issue: default manifest reader cannot consume the current generator manifests

`build_boards.py:69` selects only `raw['selected']`, then `:73` requires C1 and C2 in that same mapping. At review time `08_cold_north/manifest.json` keeps S1–S8 in `selected`, cutout paths in `cutouts`, and validity separately in `cutout_valid`. Other generation manifests also vary. A default run therefore cannot be relied upon to discover valid cuts or reject failed ones correctly.

Action: generate and validate an explicit assembly selection file containing all ten keys for all eight concepts before invocation. `08_cold_north.C2` must be JSON null, not its extant failed-image path. Alternatively normalize the schema explicitly; never infer usability merely from file existence. This is an integration prerequisite, not proof that the explicit-selection code path is broken.

## Checks supported by current code/evidence

- Concept board uses two columns and four rows; S1–S8 retain aspect ratio and are initially reduced to a 480 px short side. S7's wider cell creates dark margins around square subjects, not stretching. Whole-board downscaling to satisfy the 1.5 MB limit is explicitly allowed by §5.
- Label strips are dark; each concept cell includes subject id and Korean pitch. Long labels shrink to fit. Actual final text legibility remains unverified until final PNGs are opened, because repeated whole-board shrinking also shrinks text.
- Overview people and hut use eight columns, and scene uses two. Readability uses two terrain cells per concept row, consistent with the registered two-crop interpretation.
- Forest is top-right and meadow bottom-left. A 4×4 patch at 64 px repeats to cover 640×360 without final crop rescaling. This cannot literally be a single 256×256 patch cropped larger; D05 already documents the necessary repetition interpretation.
- Chroma removal modifies alpha only according to an explicit magenta threshold. No quantization, painted correction, outline or despill appears in the script. Border cropping and downscaling are appropriate to the composite.
- The validity guard is necessary but not semantic segmentation: 20% magenta and three magenta corners cannot prove absence of a non-magenta ground patch or disconnected artifacts. Actual selected cuts still require visual inspection; synthetic test images cannot establish this.
- Actual `08_cold_north/C2_1.png` has **0.000** keyed fraction and a full-image bounding box `(0, 0, 1254, 1254)`, confirming that the guard would reject it. The failed generator result must be null in selection.
- Missing either cutout suppresses both crops, emits NOT_VERIFIED placeholders on the board, and records unavailable inputs. This is honest; it must propagate to book/report as unavailable scores, never 0/3 or completed review. A one-person substitute would change the test.
- Fresh-evidence preflight prevents stale `crop_*.png` reuse. The output manifest records source SHA-256 and board paths. Verify generated PNG sizes, total directory budget, original hash preservation and final labels after assembly.

## Required follow-through

Resolve A1 or explicitly disclose its deviation before asking the blind reviewer. Prepare the authoritative explicit selection (A2), assemble once into fresh evidence, inspect actual boards and crops, then run fresh-context image-only review. No final crop/board acceptance is claimed here.
