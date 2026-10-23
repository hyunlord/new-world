# ART-POLISH2 original30 portrait geometry review

Status: **30/30 original attempt1 portraits inspected; geometry30/30; source garment match0/30.** No head crop was observed. These are source-visible observations, not blind-test scores, style acceptance, owner selection or adoption. Every original observation remains separate from any bounded second attempt.

Observer: `/root/portrait_geometry`, production geometry observer, ineligible for subsequent blind review. Date:2026-10-24. No blind packet, answer key or reviewer output was read.

## Method and evidence

- Read `PORTRAIT_PROTOCOL.md`, the unchanged `Inspection`/`geometry` definitions in `art/pipeline_env1/portrait_finalize.py`, and the coordinate-sheet convention in `art/pipeline_polish2/portrait_inspect.py`.
- Inspected every preserved `generation/<look>/<v>/try1/canonical.png` at native128 and2x nearest with original128 coordinate labels. Compared its actual attached tinted sprite from `try1_inputs/reference_1.png` at2x. A source-only4x nearest panel clarified the five v1 silhouettes. No pixels of source images or candidates were edited.
- Verified all30 canonical hashes and all30 sprite snapshot hashes against the original `try1/processed.json` records. Later current-reference repairs or selected-attempt replacements cannot change the bytes reviewed here.
- Manually observed conservative head/face bounds, approximately2px uncertainty. Head includes flyaways and beard; face bounds visible facial skin, excluding hair/beard and neck. The rectangular box naturally encloses background where the face contour recedes. These boxes were not inferred from prompt bounds. Diagnostic red-head/cyan-face overlays were then visually checked for all30 observations.
- Hair correspondence here means the coarse visible gathered-hair category and broad colour family; it does not certify exact knot contour, strand placement or pixel-level identity. Top correspondence checks the visible collar height, crossing and shoulder coverage; shared brown fur alone is insufficient. Low-resolution facial identity remains unestablished.

External observations: `/Users/rexxa/work/artpolish2-data/portraits/inspection/observed/try1/<look>/<v>.json` (30 valid `Inspection` objects). Each object's `inspection_method` binds the canonical portrait, saved sprite and viewed panel by absolute path and SHA-256. Panels and observed-box overlays are under the same directory's `panels/`.

Observation digest-list SHA-256: `168c893185c4727641e0ab157ee6349d373a10f9e3ed7f54ff345326beefedb2` (sorted relative JSON path, space, file SHA-256, newline). Source briefs: `/Users/rexxa/work/artpolish2-data/portraits/inspection/source_briefs.json`; SHA-256 `3aae52d600a097b27a497686b2b83075225f80b2223c84dd6de6f6e9690e80fe`. The source-brief JSON binds all30 saved sprite inputs and the source-only panel.

## Findings kept separate

| Check | Result | Meaning |
|---|---:|---|
| Valid Inspection records |30/30|Correct look/variant identity and attempt1|
| Head uncropped |30/30|No visible hair/head meets a canvas edge|
| Unchanged geometry instrument |30/30|Observed face-box corners inside fixed oval, head uncropped, margin at least2px|
| Minimum head margin |2–5px|All are close to the upper edge; this is a narrow safety margin, not generous framing|
| Maximum face-oval corner value |0.847107438|The unchanged passing threshold is at most1|
| Coarse hair correspondence |24/30|All6 `adult_F_fur_cloak_long_tied` silhouettes judged mismatched; other groups retain a gathered-bun category with exaggerated loose detail|
| Garment/neckline correspondence |0/30|All30 lower/open the high source collar; some also uncover a shoulder top|
| Extra jewellery/head ornament |0/30|None visibly added|
| Blind identity/distinctness |NOT_RUN_BY_THIS_OBSERVER|Requires fresh isolated reviewers|
| Owner selection/adoption |OWNER_CHOICE_PENDING|Not inferred from technical or observational checks|

The batch-wide issue is source garment fidelity, not a measured crop failure. The source sprites show a pale collar high at the base of the neck across both shoulder tops; outputs repeatedly open this into a low neckline. Particularly clear one-shoulder changes occur in adult female bun v3/v6, adult male bun v6, and child female bun v5/v6. The long-tied source's rear-left gathered mass and collar-length fall are replaced by a more prominent piled crown/rear bun and short curls. Source resolution does not establish a long hidden ponytail; a repair brief must not invent one. Geometry success cannot override these findings. Any second attempt remains within the existing maximum of two attempts; this report does not authorize more calls.

## Actual source briefs

All directions below are viewer-relative. The summaries describe the saved inputs, not an intended hairstyle implied by a filename. The exact six source snapshots per look are in `source_briefs.json`.

### `adult_F_fur_cloak_bun`

Swept dark front/crown, with a compact oval gathered knot above the viewer-left/back of the head. The visible side hair reaches the neck/collar; no exposed long tail is discernible.

A thick pale fur collar forms a high V/crossed wrap close below the chin and spans both shoulder tops over a brown torso panel. Upper arms are exposed below the collar/shoulder edge. A dark cape hangs behind the viewer-right arm and down the side; the source does not show a deeply open chest or a fully bare shoulder top.

Limit: The face and knot are only several source pixels across; fine parting, fastening and hidden rear hair cannot be established.

### `adult_F_fur_cloak_long_tied`

The source has a broad swept crown and a gathered swelling at the upper viewer-left/back of the head. A thicker dark fall sits beside the viewer-left face/neck and reaches the pale collar. It is broader and lower at the rear than the compact-knot source above; no tail extending below the collar is unambiguously exposed.

A broad pale fur collar crosses high at the base of the neck and covers both shoulder tops, over a closed brown chest panel. Bare upper arms start below the collar. A dark hanging cape is most visible behind the viewer-right arm. The collar obscures the lower extent of rear hair.

Limit: The filename does not prove long hair length. A waist-length ponytail, exact tie position and the hidden section behind the cape are not visually established.

### `adult_M_fur_cloak_bun`

A small distinct rounded knot rises above the viewer-left/back of the swept hair cap. Side hair is close to the face/ear and ends around the neck; no long exposed fall is visible.

A broad cream fur band makes a high V across both shoulder tops, over a largely closed brown chest. A thinner pale diagonal strap crosses the brown chest below the collar. Upper arms are bare below the shoulder edge, while the dark cloak continues behind the sides. The source is not an open deep-V fur vest.

Limit: Source facial hair and facial features are too small for exact identity inference; the observed garment strap is part of clothing, not jewellery.

### `child_F_fur_cloak_bun`

A compact dark cap surrounds the small face, with a relatively tall rounded gathered knot at the upper viewer-left/back. Side hair ends beside the neck and collar; no long trailing tail is visible.

A pale high V/crossed fur collar hugs the neck and spans both shoulder tops above a brown closed chest panel. Arms emerge below the wrap. Dark cloak/fur panels hang at the sides toward the hips. The source does not show a low broad exposed upper chest.

Limit: The child source face and collar occupy very few pixels; the precise seam, knot construction and rear hairstyle are unresolved.

### `child_M_fur_cloak_bun`

A small upright rounded knot sits above the viewer-left/back of a low swept hair cap. The visible hair is short at the sides/neck, with no exposed long tail.

A pale collar forms a high V/crossed band immediately below the neck and covers both shoulder tops over a brown chest panel; a pale diagonal clothing line continues across the chest. Arms are exposed below the collar. Dark side panels continue to the hips.

Limit: At the original sprite scale the exact overlap of collar and strap, and the hidden back of the hair, cannot be resolved.

## Per-portrait observations

Bounds are `[left,top,right,bottom]` in original128 coordinates. All rows have head_uncropped=true, geometry=true, top_match=false and extra_accessory=false. The separate hair column preserves the six hairstyle mismatches.

| Look/variant | Head bounds | Face bounds | Margin px | Oval max | Hair match |
|---|---|---|---:|---:|---|
|`adult_F_fur_cloak_bun/v1`|`[24, 3, 99, 85]`|`[34, 33, 83, 79]`|3|0.754817|true|
|`adult_F_fur_cloak_bun/v2`|`[28, 4, 98, 87]`|`[35, 33, 84, 78]`|4|0.724342|true|
|`adult_F_fur_cloak_bun/v3`|`[27, 3, 99, 88]`|`[34, 34, 85, 81]`|3|0.734477|true|
|`adult_F_fur_cloak_bun/v4`|`[25, 3, 100, 85]`|`[35, 31, 87, 82]`|3|0.767241|true|
|`adult_F_fur_cloak_bun/v5`|`[25, 3, 99, 85]`|`[35, 31, 86, 81]`|3|0.767241|true|
|`adult_F_fur_cloak_bun/v6`|`[24, 2, 100, 87]`|`[35, 33, 86, 81]`|2|0.724342|true|
|`adult_F_fur_cloak_long_tied/v1`|`[24, 3, 100, 88]`|`[35, 33, 84, 82]`|3|0.724342|false|
|`adult_F_fur_cloak_long_tied/v2`|`[28, 2, 99, 86]`|`[35, 31, 85, 79]`|2|0.767241|false|
|`adult_F_fur_cloak_long_tied/v3`|`[25, 3, 99, 85]`|`[35, 31, 86, 80]`|3|0.767241|false|
|`adult_F_fur_cloak_long_tied/v4`|`[26, 2, 101, 84]`|`[34, 30, 86, 81]`|2|0.820275|false|
|`adult_F_fur_cloak_long_tied/v5`|`[27, 3, 102, 87]`|`[36, 32, 88, 82]`|3|0.715979|false|
|`adult_F_fur_cloak_long_tied/v6`|`[24, 2, 100, 85]`|`[35, 33, 85, 81]`|2|0.724342|false|
|`adult_M_fur_cloak_bun/v1`|`[27, 3, 99, 83]`|`[36, 34, 85, 81]`|3|0.674559|true|
|`adult_M_fur_cloak_bun/v2`|`[26, 2, 99, 86]`|`[35, 31, 86, 69]`|2|0.767241|true|
|`adult_M_fur_cloak_bun/v3`|`[28, 3, 99, 86]`|`[36, 32, 85, 70]`|3|0.715979|true|
|`adult_M_fur_cloak_bun/v4`|`[25, 3, 101, 90]`|`[34, 30, 88, 70]`|3|0.820275|true|
|`adult_M_fur_cloak_bun/v5`|`[28, 5, 102, 89]`|`[35, 32, 90, 73]`|5|0.745422|true|
|`adult_M_fur_cloak_bun/v6`|`[24, 2, 99, 86]`|`[34, 34, 88, 85]`|2|0.734477|true|
|`child_F_fur_cloak_bun/v1`|`[23, 3, 102, 93]`|`[32, 35, 89, 85]`|3|0.778926|true|
|`child_F_fur_cloak_bun/v2`|`[26, 3, 101, 90]`|`[34, 34, 90, 86]`|3|0.734477|true|
|`child_F_fur_cloak_bun/v3`|`[20, 2, 102, 90]`|`[30, 35, 89, 86]`|2|0.847107|true|
|`child_F_fur_cloak_bun/v4`|`[27, 2, 104, 95]`|`[35, 35, 91, 86]`|2|0.684401|true|
|`child_F_fur_cloak_bun/v5`|`[27, 3, 103, 90]`|`[35, 32, 90, 82]`|3|0.745422|true|
|`child_F_fur_cloak_bun/v6`|`[26, 2, 103, 93]`|`[35, 34, 91, 86]`|2|0.704001|true|
|`child_M_fur_cloak_bun/v1`|`[25, 3, 102, 88]`|`[35, 34, 90, 86]`|3|0.704001|true|
|`child_M_fur_cloak_bun/v2`|`[27, 2, 100, 87]`|`[35, 35, 88, 81]`|2|0.684401|true|
|`child_M_fur_cloak_bun/v3`|`[26, 4, 99, 88]`|`[35, 34, 88, 83]`|4|0.704001|true|
|`child_M_fur_cloak_bun/v4`|`[24, 4, 100, 89]`|`[34, 32, 88, 83]`|4|0.775897|true|
|`child_M_fur_cloak_bun/v5`|`[26, 2, 101, 84]`|`[34, 34, 88, 80]`|2|0.734477|true|
|`child_M_fur_cloak_bun/v6`|`[27, 3, 99, 86]`|`[35, 35, 88, 82]`|3|0.684401|true|

## Qualitative style notes — report only

The30 portraits share a consistent warm brown/ochre palette, mottled parchment background, dark contours, modeled face shadows and fine hair/fur marks. The hair, flyaways and ragged fur edges are busy at128px, while the face is still the strongest contrast area. Many outputs repeat a very similar three-quarter direction, raised messy hair mass and low fur neckline across groups. Adult greying/beards and copper versus dark hair remain visibly different in the relevant images, but this observer did not conduct blind matching or distinctness scoring. The rendering tends toward detailed naturalistic illustration with dense small texture rather than broad sparse washes. No numeric/model style score is assigned here, and this paragraph cannot override the source-match findings.

## Validation and limits

Fresh validation imported the existing `Inspection` and `geometry` with `uv run --with pillow --with pydantic --with typer --with numpy --with scipy --with scikit-image python`. It validated exactly30 JSON files, matching each look/variant to its path, integer coordinate values and attempt1, then evaluated the unchanged instrument: geometry30/30, uncropped30/30, hair24/30, top0/30, extra accessories0/30, minimum-margin range2–5px and maximum oval value0.8471074380165289. Hash checks verified30 original portrait/reference pairs. Schema validation proves record shape; manual observations remain approximate and are not pixel segmentation or facial-identity certification.

No candidate collection, selected-record processing, image generation, scorecard, blind packet/key access, staging or integration was performed by this observer. Original try1 observations must remain intact if another attempt replaces a selected candidate.
