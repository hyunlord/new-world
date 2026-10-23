# ART-POLISH2 final adult portrait review

Status: **OBSERVATION COMPLETE — 96/96 final adult portraits individually viewed.** The initial 30 observations and `PORTRAIT_GEOMETRY_REVIEW.md` remain separate and byte-identical. Old attempt 1 portraits replaced by attempt 2 are not counted as final.

Observer: `/root/portrait_geometry`; date 2026-10-24. Scope is 16 actual adult looks × 6 variants only. No children, blind-test score or owner approval is included. Owner choice remains pending. This report counts distinct latest-observed variants separately from all immutable attempt measurements: 96 distinct variants, 96 observed attempt records, 0 superseded attempt records. This display choice never mutates production selection.

## Evidence and observation method

The final continuation queue supplies each exact request path and attempt. A portrait becomes eligible only after its matching `tryN.call.json` is NORMALIZED and `tryN/processed.json` exists. Every observed canonical portrait and all attached input snapshots are SHA-256 checked against that attempt's immutable request/processing records. Actual native 128 × 128 and 2× nearest-neighbor images provide manual geometry; the exact attached sprites at 2× provide hair and visible garment comparison. Source-only 4× panels clarify low-resolution source silhouettes. Bounds use original 128-pixel coordinates with approximately 2 px uncertainty. Head includes all visible hair/flyaways and beard; face bounds visible facial skin excluding hair/beard and neck. Boxes are manually observed, never inferred from requested composition.

Hair correspondence means coarse visible hairstyle and broad colour family; it does not establish an exact strand contour. Garment correspondence compares actual shoulder coverage, collar height and neckline. No hidden hair length or garment construction is inferred from asset names. Facial identity remains unestablished from the tiny source face.

Immutable observations: `/Users/rexxa/work/artpolish2-data/portraits/inspection/observed/final_adult/<look>/<v>.<attempt>.json`. Each Inspection binds exact request, raw image, canonical image, all attached input snapshots and viewed panel paths/hashes in `inspection_method`. Numbered `ready-*.json` manifests hand observations to the production owner; this observer never processes or selects records. Panels are independent composites and never replace originals.

Observation digest-list SHA-256: `ab10df3efa6df6bdd7e1e51f01dd8a635f29cf4ce705933fa585182d3efbc425` (sorted relative JSON path, space, SHA-256, newline).

Final verification: all 96 Inspection records validate against the existing schema and exactly cover the adult continuation queue. All 32 ready manifests reproduce the unchanged geometry calculation. Request, raw, preserved canonical and viewed-panel hashes match; all 288 attached reference snapshots match their requests and attached originals. The original 30 observations, original geometry report and initial source briefs remain byte-identical. The machine-readable result is `/Users/rexxa/work/artpolish2-data/portraits/inspection/observed/final_adult/verification-first96.json`.

## Results kept separate

| Check | Count |
|---|---:|
| Individually viewed and schema-validated |96/96|
| Unchanged formal geometry pass |94/96|
| Hair correspondence |96/96|
| Garment/neckline correspondence |89/96|
| Extra accessory observed |0/96|

All 96 portraits were actually viewed; none was inferred from similar outputs. Geometry is computed by the unchanged `art.pipeline_env1.portrait_finalize.geometry` after importing the exact Inspection model with the existing Pillow/Pydantic/Typer/NumPy/SciPy/scikit-image environment. Passing geometry does not override a hair/top/accessory finding. Three explicit beard contradictions remain separate from these counts. Approximate measurements and source resolution remain explicit limits.

## Individual measurements

Bounds are [left, top, right, bottom] in original 128-pixel coordinates. Correspondence flags are independent of formal geometry.

| Portrait | Try | Head bounds | Face bounds | Margin | Oval max | Geometry | Hair | Top | Extra |
|---|---:|---|---|---:|---:|---|---|---|---|
|adult_F_fur_cloak_bun/v1|2|[28, 3, 97, 86]|[43, 31, 93, 78]|3|0.767241|PASS|true|true|false|
|adult_F_fur_cloak_bun/v2|2|[29, 3, 96, 85]|[42, 30, 93, 78]|3|0.789800|PASS|true|true|false|
|adult_F_fur_cloak_bun/v3|2|[31, 3, 96, 84]|[44, 30, 93, 76]|3|0.789800|PASS|true|true|false|
|adult_F_fur_cloak_bun/v4|2|[27, 2, 96, 83]|[40, 28, 93, 77]|2|0.837138|PASS|true|true|false|
|adult_F_fur_cloak_bun/v5|2|[28, 2, 96, 84]|[42, 28, 93, 78]|2|0.837138|PASS|true|true|false|
|adult_F_fur_cloak_bun/v6|2|[32, 2, 95, 83]|[41, 31, 91, 77]|2|0.709390|PASS|true|true|false|
|adult_F_fur_cloak_long_tied/v1|2|[19, 2, 98, 87]|[40, 31, 94, 80]|2|0.797716|PASS|true|true|false|
|adult_F_fur_cloak_long_tied/v2|2|[25, 2, 98, 87]|[41, 30, 94, 79]|2|0.820275|PASS|true|true|false|
|adult_F_fur_cloak_long_tied/v3|2|[21, 2, 99, 87]|[42, 29, 95, 80]|2|0.875083|PASS|true|true|false|
|adult_F_fur_cloak_long_tied/v4|2|[26, 3, 98, 82]|[40, 26, 95, 77]|3|0.949417|PASS|true|true|false|
|adult_F_fur_cloak_long_tied/v5|2|[25, 3, 99, 84]|[43, 27, 95, 79]|3|0.923899|PASS|true|true|false|
|adult_F_fur_cloak_long_tied/v6|2|[18, 2, 97, 89]|[41, 32, 93, 79]|2|0.745422|PASS|true|true|false|
|adult_F_fur_cloak_short/v1|1|[25, 4, 98, 90]|[38, 29, 92, 79]|4|0.783657|PASS|true|true|false|
|adult_F_fur_cloak_short/v2|1|[23, 4, 100, 83]|[39, 25, 94, 80]|4|0.944166|PASS|true|true|false|
|adult_F_fur_cloak_short/v3|1|[26, 4, 99, 80]|[35, 24, 87, 77]|4|0.940688|PASS|true|true|false|
|adult_F_fur_cloak_short/v4|1|[26, 5, 100, 87]|[35, 25, 87, 79]|5|0.913691|PASS|true|true|false|
|adult_F_fur_cloak_short/v5|1|[24, 4, 100, 87]|[40, 26, 95, 79]|4|0.949417|PASS|true|true|false|
|adult_F_fur_cloak_short/v6|1|[26, 3, 102, 86]|[41, 30, 96, 80]|3|0.884325|PASS|true|true|false|
|adult_F_hide_wrap_bun/v1|1|[31, 3, 96, 87]|[42, 32, 93, 80]|3|0.745422|PASS|true|true|false|
|adult_F_hide_wrap_bun/v2|1|[29, 2, 93, 87]|[42, 31, 90, 80]|2|0.682014|PASS|true|true|false|
|adult_F_hide_wrap_bun/v3|1|[27, 3, 97, 86]|[34, 30, 87, 81]|3|0.820275|PASS|true|true|false|
|adult_F_hide_wrap_bun/v4|1|[27, 2, 95, 82]|[34, 28, 86, 79]|2|0.867613|PASS|true|true|false|
|adult_F_hide_wrap_bun/v5|1|[26, 2, 100, 84]|[36, 31, 89, 82]|2|0.737799|PASS|true|true|false|
|adult_F_hide_wrap_bun/v6|1|[31, 4, 98, 86]|[42, 33, 95, 82]|4|0.786325|PASS|true|true|false|
|adult_F_hide_wrap_long_tied/v1|1|[13, 6, 100, 94]|[42, 30, 95, 81]|6|0.851784|PASS|true|true|false|
|adult_F_hide_wrap_long_tied/v2|1|[15, 3, 100, 93]|[42, 30, 96, 83]|3|0.884325|PASS|true|true|false|
|adult_F_hide_wrap_long_tied/v3|1|[14, 0, 102, 106]|[43, 27, 96, 79]|0|0.956440|FAIL|true|true|false|
|adult_F_hide_wrap_long_tied/v4|1|[12, 2, 103, 86]|[39, 28, 97, 81]|2|0.965237|PASS|true|true|false|
|adult_F_hide_wrap_long_tied/v5|1|[16, 2, 102, 92]|[44, 27, 97, 81]|2|0.990015|PASS|true|true|false|
|adult_F_hide_wrap_long_tied/v6|1|[15, 2, 104, 89]|[43, 29, 98, 81]|2|0.975806|PASS|true|true|false|
|adult_F_hide_wrap_short/v1|1|[25, 4, 101, 82]|[38, 31, 96, 81]|4|0.861766|PASS|true|true|false|
|adult_F_hide_wrap_short/v2|1|[23, 2, 101, 82]|[40, 28, 96, 78]|2|0.931662|PASS|true|true|false|
|adult_F_hide_wrap_short/v3|1|[26, 2, 101, 87]|[40, 29, 96, 82]|2|0.907624|PASS|true|true|false|
|adult_F_hide_wrap_short/v4|1|[26, 3, 101, 83]|[39, 27, 96, 80]|3|0.956440|PASS|true|true|false|
|adult_F_hide_wrap_short/v5|1|[26, 3, 102, 84]|[35, 32, 90, 82]|3|0.745422|PASS|true|true|false|
|adult_F_hide_wrap_short/v6|1|[24, 3, 104, 84]|[38, 29, 96, 81]|3|0.907624|PASS|true|true|false|
|adult_F_loincloth_bun/v1|1|[35, 22, 91, 91]|[47, 47, 87, 85]|22|0.486262|PASS|true|true|false|
|adult_F_loincloth_bun/v2|1|[36, 21, 91, 86]|[47, 43, 88, 83]|21|0.476515|PASS|true|true|false|
|adult_F_loincloth_bun/v3|1|[40, 24, 88, 85]|[49, 45, 85, 82]|24|0.390881|PASS|true|true|false|
|adult_F_loincloth_bun/v4|1|[29, 17, 94, 88]|[36, 42, 84, 86]|17|0.636098|PASS|true|true|false|
|adult_F_loincloth_bun/v5|1|[36, 19, 89, 80]|[46, 40, 85, 79]|19|0.390881|PASS|true|true|false|
|adult_F_loincloth_bun/v6|1|[37, 21, 88, 82]|[46, 43, 85, 80]|21|0.361295|PASS|true|true|false|
|adult_F_loincloth_long_tied/v1|1|[24, 22, 94, 95]|[46, 42, 91, 84]|22|0.572186|PASS|true|false|false|
|adult_F_loincloth_long_tied/v2|1|[23, 20, 92, 92]|[47, 43, 88, 83]|20|0.476515|PASS|true|true|false|
|adult_F_loincloth_long_tied/v3|1|[23, 20, 93, 94]|[47, 41, 89, 83]|20|0.501825|PASS|true|true|false|
|adult_F_loincloth_long_tied/v4|1|[25, 23, 95, 96]|[45, 40, 91, 86]|23|0.607689|PASS|true|true|false|
|adult_F_loincloth_long_tied/v5|1|[24, 23, 95, 95]|[45, 43, 91, 86]|23|0.607689|PASS|true|true|false|
|adult_F_loincloth_long_tied/v6|1|[25, 20, 95, 107]|[46, 43, 90, 85]|20|0.562191|PASS|true|true|false|
|adult_F_loincloth_short/v1|1|[31, 25, 91, 85]|[42, 43, 87, 84]|25|0.468880|PASS|true|true|false|
|adult_F_loincloth_short/v2|1|[30, 24, 97, 86]|[37, 43, 86, 85]|24|0.589567|PASS|true|true|false|
|adult_F_loincloth_short/v3|1|[31, 20, 94, 81]|[43, 38, 91, 80]|20|0.572186|PASS|true|true|false|
|adult_F_loincloth_short/v4|1|[31, 25, 92, 87]|[41, 40, 88, 84]|25|0.493157|PASS|true|true|false|
|adult_F_loincloth_short/v5|1|[31, 20, 97, 86]|[37, 41, 86, 84]|20|0.572186|PASS|true|true|false|
|adult_F_loincloth_short/v6|1|[30, 24, 94, 85]|[43, 41, 91, 84]|24|0.572186|PASS|true|true|false|
|adult_M_fur_cloak_bun/v1|2|[28, 1, 97, 84]|[42, 30, 93, 67]|1|0.789800|FAIL|true|true|false|
|adult_M_fur_cloak_bun/v2|2|[27, 2, 97, 86]|[42, 30, 94, 68]|2|0.820275|PASS|true|true|false|
|adult_M_fur_cloak_bun/v3|2|[29, 5, 95, 86]|[44, 32, 93, 67]|5|0.745422|PASS|true|true|false|
|adult_M_fur_cloak_bun/v4|2|[28, 2, 94, 86]|[40, 29, 92, 68]|2|0.783657|PASS|true|true|false|
|adult_M_fur_cloak_bun/v5|2|[27, 2, 94, 85]|[40, 30, 92, 67]|2|0.760358|PASS|true|true|false|
|adult_M_fur_cloak_bun/v6|2|[28, 5, 97, 85]|[40, 31, 92, 67]|5|0.737799|PASS|true|true|false|
|adult_M_fur_cloak_short/v1|1|[29, 4, 94, 78]|[35, 28, 86, 67]|4|0.837138|PASS|true|true|false|
|adult_M_fur_cloak_short/v2|1|[33, 5, 97, 80]|[37, 26, 88, 67]|5|0.829582|PASS|true|true|false|
|adult_M_fur_cloak_short/v3|1|[32, 5, 96, 82]|[43, 27, 93, 66]|5|0.861916|PASS|true|true|false|
|adult_M_fur_cloak_short/v4|1|[27, 6, 95, 85]|[37, 25, 93, 70]|6|0.913691|PASS|true|true|false|
|adult_M_fur_cloak_short/v5|1|[32, 3, 97, 81]|[43, 25, 95, 67]|3|0.975674|PASS|true|true|false|
|adult_M_fur_cloak_short/v6|1|[28, 5, 95, 85]|[37, 32, 93, 80]|5|0.745422|PASS|true|true|false|
|adult_M_hide_wrap_bun/v1|1|[26, 4, 94, 87]|[34, 36, 87, 85]|4|0.696015|PASS|true|true|false|
|adult_M_hide_wrap_bun/v2|1|[33, 18, 94, 96]|[38, 47, 86, 77]|18|0.443848|PASS|true|true|false|
|adult_M_hide_wrap_bun/v3|1|[29, 16, 92, 91]|[35, 43, 83, 80]|16|0.567907|PASS|true|true|false|
|adult_M_hide_wrap_bun/v4|1|[29, 22, 93, 101]|[37, 44, 85, 81]|22|0.524479|PASS|true|true|false|
|adult_M_hide_wrap_bun/v5|1|[32, 18, 93, 93]|[43, 43, 89, 83]|18|0.501825|PASS|true|true|false|
|adult_M_hide_wrap_bun/v6|1|[28, 15, 96, 92]|[34, 44, 86, 90]|15|0.775897|PASS|true|true|false|
|adult_M_hide_wrap_short/v1|1|[27, 21, 99, 92]|[35, 44, 86, 90]|21|0.745422|PASS|true|true|false|
|adult_M_hide_wrap_short/v2|1|[29, 21, 94, 94]|[36, 44, 83, 76]|21|0.511837|PASS|true|true|false|
|adult_M_hide_wrap_short/v3|1|[30, 21, 93, 86]|[38, 43, 83, 75]|21|0.468996|PASS|true|true|false|
|adult_M_hide_wrap_short/v4|1|[30, 20, 93, 93]|[36, 39, 83, 76]|20|0.583953|PASS|true|true|false|
|adult_M_hide_wrap_short/v5|1|[30, 23, 94, 90]|[42, 41, 87, 79]|23|0.421173|PASS|true|true|false|
|adult_M_hide_wrap_short/v6|1|[28, 20, 95, 88]|[35, 40, 84, 86]|20|0.665540|PASS|true|true|false|
|adult_M_loincloth_bun/v1|1|[29, 21, 95, 95]|[36, 51, 84, 93]|21|0.783657|PASS|true|true|false|
|adult_M_loincloth_bun/v2|1|[31, 20, 96, 98]|[44, 47, 92, 79]|20|0.524781|PASS|true|true|false|
|adult_M_loincloth_bun/v3|1|[31, 17, 94, 91]|[36, 43, 85, 80]|17|0.538465|PASS|true|true|false|
|adult_M_loincloth_bun/v4|1|[33, 22, 93, 99]|[43, 46, 88, 81]|22|0.445450|PASS|true|true|false|
|adult_M_loincloth_bun/v5|1|[31, 20, 91, 92]|[39, 46, 84, 79]|20|0.442653|PASS|true|true|false|
|adult_M_loincloth_bun/v6|1|[34, 15, 96, 88]|[43, 44, 91, 87]|15|0.626550|PASS|true|true|false|
|adult_M_loincloth_long_tied/v1|1|[27, 20, 94, 91]|[42, 47, 84, 89]|20|0.539941|PASS|true|true|false|
|adult_M_loincloth_long_tied/v2|1|[27, 18, 94, 94]|[42, 43, 86, 76]|18|0.369822|PASS|true|true|false|
|adult_M_loincloth_long_tied/v3|1|[25, 23, 92, 89]|[43, 43, 86, 78]|23|0.369822|PASS|true|true|false|
|adult_M_loincloth_long_tied/v4|1|[28, 21, 92, 94]|[40, 42, 86, 77]|21|0.431027|PASS|true|true|false|
|adult_M_loincloth_long_tied/v5|1|[30, 22, 94, 90]|[44, 44, 87, 80]|22|0.406750|PASS|true|true|false|
|adult_M_loincloth_long_tied/v6|1|[27, 22, 94, 89]|[41, 44, 87, 88]|22|0.542844|PASS|true|true|false|
|adult_M_loincloth_short/v1|1|[29, 20, 94, 87]|[36, 43, 83, 86]|20|0.636098|PASS|true|false|false|
|adult_M_loincloth_short/v2|1|[31, 22, 94, 91]|[37, 43, 80, 78]|22|0.496372|PASS|true|false|false|
|adult_M_loincloth_short/v3|1|[30, 21, 95, 85]|[39, 44, 82, 77]|21|0.429709|PASS|true|false|false|
|adult_M_loincloth_short/v4|1|[30, 21, 91, 94]|[39, 41, 85, 76]|21|0.470760|PASS|true|false|false|
|adult_M_loincloth_short/v5|1|[29, 22, 95, 95]|[36, 44, 84, 80]|22|0.538465|PASS|true|false|false|
|adult_M_loincloth_short/v6|1|[28, 22, 98, 93]|[37, 49, 85, 92]|22|0.731949|PASS|true|false|false|

## Structural failures

- `adult_F_hide_wrap_long_tied/v3` try1: head_uncropped=False, margin=0px, oval_max=0.956440. The copper-brown gathered rear-left hair mass falls beside the neck, with an additional loose curl extending onto the upper chest. The coarse tied hairstyle corresponds, but the highest hair is visibly cut by the top canvas edge: actual top0 and head_uncropped=false, confirmed by the8x top-strip diagnostic. The cream-edged diagonal wrap covers the viewer-right shoulder and leaves the opposite shoulder bare, corresponding to source coverage. No added jewellery or ornament is visible. Facial identity is not certified from the tiny source face.
- `adult_M_fur_cloak_bun/v1` try2: head_uncropped=True, margin=1px, oval_max=0.789800. The raised gathered bun with swept front and short nape strands corresponds to the actual source. The pale fur collar covers both shoulder tops and crosses over a closed brown chest, leaving a narrow neck opening. No added jewellery or ornament is visible. Identity-brief discrepancy: the actual portrait has a full dark beard although this exact request says clean shaven. The highest hair/flyaway is observed at approximately y1 in an8x top-strip diagnostic, so the2px margin requirement fails despite the head appearing uncropped. This is already attempt2. Facial identity is not certified from the tiny source face.

## Correspondence findings

- `adult_F_loincloth_long_tied/v1` try1: A broad dark-brown gathered rear-left hair mass falls beside the neck to shoulder level, corresponding to the visible source hairstyle. Both shoulders are bare as in the source, but the portrait's cream strip crosses diagonally over a brown-top bandeau instead of forming the source's straight horizontal cream upper edge. This garment/neckline correspondence failure is independent of passing geometry. No added jewellery or ornament is visible. Facial identity is not certified from the tiny source face.
- `adult_M_loincloth_short/v1` try1: Short tousled dark brown hair matches the source's coarse short silhouette. The youthful face is clean shaven. Garment discrepancy: the actual attached sprite has both shoulders and chest bare above a hip-level loincloth; the portrait invents a cream-edged brown strapless garment across the chest. Both shoulders remain bare, so the mismatch concerns added upper-torso coverage rather than shoulder coverage. No added accessory is visible. Facial identity is not certified from the tiny source face.
- `adult_M_loincloth_short/v2` try1: Short tousled near-black hair matches the source's coarse short silhouette, and a full dark beard is visible. Garment discrepancy: the actual attached sprite has a bare chest and both shoulders above its hip-level loincloth, but the portrait adds a cream-edged strapless brown chest garment. The shoulders remain bare; the failure is invented upper-torso coverage. No added accessory is visible. Facial identity is not certified from the tiny source face.
- `adult_M_loincloth_short/v3` try1: Short tousled auburn hair retains the source's coarse short silhouette; a short auburn beard is visible. Garment discrepancy: the actual attached sprite has a bare chest and both shoulders above its hip-level loincloth, while the portrait invents a cream-edged brown strapless chest garment. Both shoulders are bare, so the failure concerns added upper-torso coverage. No added accessory is visible. Facial identity is not certified from the tiny source face.
- `adult_M_loincloth_short/v4` try1: Short tousled grey hair corresponds to the source's coarse short silhouette, with a full grey beard and older facial lines visible. Garment discrepancy: the attached sprite has both shoulders and its chest bare above a hip-level loincloth, but the portrait adds a cream-edged strapless brown garment across the chest. Both shoulders remain bare; this is invented upper-torso coverage. No added accessory is visible. Facial identity is not certified from the tiny source face.
- `adult_M_loincloth_short/v5` try1: Short tousled auburn hair agrees with the source's coarse short silhouette; a short beard and older facial lines are visible. Garment discrepancy: the actual attached sprite has a bare chest and bare shoulders above its hip-level loincloth, while the portrait adds a cream-edged diagonal brown chest wrap whose edge rises toward the viewer-right shoulder. This invented upper-torso garment does not match the source's bare torso. No separate added accessory is visible. Facial identity is not certified from the tiny source face.
- `adult_M_loincloth_short/v6` try1: Short tousled dark brown hair retains the source's coarse short silhouette; the youthful face is clean shaven. Garment discrepancy: the attached sprite has a bare chest and both shoulders above a hip-level loincloth, but the portrait adds a cream-edged brown chest wrap with a center tie and an edge rising toward the viewer-right shoulder. The invented upper-torso garment contradicts the source's bare torso. No separate added accessory is visible. Facial identity is not certified from the tiny source face.

## Identity-brief discrepancies

- `adult_M_fur_cloak_bun/v1` try2: The raised gathered bun with swept front and short nape strands corresponds to the actual source. The pale fur collar covers both shoulder tops and crosses over a closed brown chest, leaving a narrow neck opening. No added jewellery or ornament is visible. Identity-brief discrepancy: the actual portrait has a full dark beard although this exact request says clean shaven. The highest hair/flyaway is observed at approximately y1 in an8x top-strip diagnostic, so the2px margin requirement fails despite the head appearing uncropped. This is already attempt2. Facial identity is not certified from the tiny source face.
- `adult_M_fur_cloak_bun/v6` try2: The dark gathered bun and short nape hair correspond to the saved source hairstyle. Pale fur covers both shoulder tops and crosses high over the closed brown chest. No added jewellery is visible. Identity-brief discrepancy: the actual portrait has a full dark beard although this exact final request says clean shaven. This is separate from hair/top correspondence; attempt2 is already the permitted final attempt. Facial identity is not certified from the tiny source face.
- `adult_M_fur_cloak_short/v1` try1: Short tousled dark hair reaches the ears/upper neck as in the actual source. The high broad fur collar covers both shoulders above a closed brown chest/cape. No added jewellery is visible. Identity-brief discrepancy: the actual portrait has a full dark beard although the exact request says clean shaven. This is retained separately from geometry and source garment/hairstyle correspondence. Facial identity is not certified from the tiny source face.

## Superseded observed attempts — failures retained

None yet in the final-adult lane. The separate initial 30 observations remain unchanged.

## Qualitative style — report only

The inspected outputs use warm parchment backgrounds, dark fine contours, modeled face shadows and textured hair/fur detail. Style notes carry no score and cannot override geometry or source correspondence. No blind distinctness or identity test is conducted by this production-context observer.

The initial 30 records, generation limits and owner-pending status remain unchanged. No source/candidate pixel editing, generation, candidate collection, selected-record processing, blind-key access, staging or integration was performed in this lane. Any later bounded repair requires a new immutable attempt observation; this closure does not erase the findings above.
