# ART-POLISH2 final child portrait geometry review

Status: **COMPLETE_OBSERVATION — 108/108 final child portraits observed; geometry102/108.** Source correspondence, blind scores and owner adoption remain distinct.

Observer: `/root/portrait_child_geometry`, production geometry observer, ineligible for subsequent blind review. Date:2026-10-24. Final cohort:18 child looks ×6 variants. The two fur-bun looks use attempt2; remaining child looks use the exact continuation-manifest attempt. Original attempt1 observations and raw outputs remain untouched.

## Method and bindings

Every recorded portrait was actually viewed at native128 and2x nearest beside its saved tinted S/idle reference at2x. Source-only4x panels were also viewed for all18 v1 silhouettes. Original128 coordinate labels guide conservative observed head/face boxes, with approximately2px uncertainty. Head includes all visible hair/flyaways and face; face bounds visible facial skin excluding hair and neck. Every red-head/cyan-face diagnostic overlay was viewed after measurement.

Hair correspondence means the coarse visible gathered/short category and broad colour family, not exact knot contour or strand placement. Garment correspondence checks neckline height, overlap and shoulder coverage; shared brown material is insufficient. Facial identity is not established from the small source face. A narrow measured margin is a nominal instrument result, not a guarantee under measurement uncertainty.

The unchanged `Inspection` and `geometry` definitions in `art/pipeline_env1/portrait_finalize.py` validate every observation. Preparation requires the exact continuation request path and attempt, a NORMALIZED call and matching request/raw/canonical/reference hashes. No intended prompt bounds are copied as measurements. No generation, processing, candidate selection, blind packet/key access, staging or integration was performed by this observer.

External observations: `/Users/rexxa/work/artpolish2-data/portraits/inspection/observed/final_child/<look>/<v>.<attempt>.json`. Ready manifests bind exact request, raw, preserved canonical, saved sprite, observation, viewed panel and overlay paths plus SHA256. The generation owner receives each manifest for processing with the exact request path and attempt.

## Current findings

| Check | Observed result |
|---|---:|
|Valid exact-attempt observations|108/108|
|Unchanged geometry instrument|102/108|
|No definite head truncation observed|108/108|
|Coarse hair correspondence|108/108|
|Garment/neckline correspondence|42/108|
|Extra accessory observed|0/108|
|Blind distinctness/identity|NOT_RUN_BY_THIS_OBSERVER|
|Owner selection/adoption|OWNER_CHOICE_PENDING|

Known margin failures are retained: `child_M_fur_cloak_bun/v3` (attempt2, margin1px), `child_M_fur_cloak_bun/v5` (attempt2, margin1px), `child_F_loincloth_long_tied/v4` (attempt1, margin1px), `child_F_loincloth_bun/v1` (attempt1, margin1px), `child_F_loincloth_bun/v3` (attempt1, margin1px), `child_F_loincloth_bun/v5` (attempt1, margin1px). The unchanged minimum is2px. No definite truncation was observed; wispy hair versus paper at the top is uncertain by approximately1px. Additional8x top16-row diagnostics were actually viewed and hash-bound in the relevant notes. A nominal2px pass remains uncertain near the boundary.

A readiness-wrapper race was detected when the generation importer polled ready-005 before the post-overlay status seal. The exact provisional bytes are preserved as history/ready-005.pre-overlay.json (SHA256 f7e5690565284af04b450db04810f280f7e40e117934bce6edbb3e2e8318eb20), matching the importer-observed hash. Only wrapper status and added overlay-check prose changed; all three entries and their bound observations/inputs are identical. Generation preserved both versions and its prior ledger explicitly. Future manifests use pending-* and atomic publication as ready-* only after actual overlay review; published ready files never change.

All18 female hide_wrap attempt1 portraits (bun, long_tied and short) mismatch the actual source garment coverage: the diagonal pale edge drops substantially below the high clavicle/base-neck line and exposes more upper chest. The one-shoulder category and brown material alone do not establish correspondence. The per-look source-only crop12x plus portrait3x diagnostic is hash-bound in each observation note.

The new four-reference child_M_hide_wrap_bun/v1 attempt1 pilot also fails garment coverage height. Its exact saved reference4 upper-three-quarter source crop at12x, bust-detail sidecar and a paper-composited context were actually viewed/bound in ready-013. Head[28,22,98,95], face[39,53,89,92] pass the unchanged geometry instrument with22px margin; coarse bun/hair matches, top_match=false, no added accessory or definite crop. The fourth reference does not establish a repaired output.

A second unstarted pilot, child_M_hide_wrap_bun/v2 attempt1, also fails upper-edge height. Its covered viewer-right shoulder and bare opposite shoulder/arm correspond to source, but the diagonal cream edge still lies too low and exposes extra upper chest. It is somewhat higher/broader thanv1; source edge uncertainty is approximately1 source pixel. Head[30,18,98,90] and face[39,49,86,88] pass geometry with18px margin. Exact saved reference4, sidecar and actually viewed source12x/portrait3x context are bound in ready-014. Earlier49 observations are unchanged.

A third unstarted pilot, child_M_hide_wrap_bun/v3 attempt1, has a visibly shallower/higher cream edge and broader right-shoulder coverage than the preceding two, but still leaves excess inner upper chest bare versus source. Top_match remains false; opposite outer shoulder/arm exposure matches coarsely. Head[27,17,98,90], face[37,55,87,88], geometry margin17px; no extra accessory or definite crop. The exact fourth source crop and actually viewed comparison are bound in ready-015; no earlier result is overwritten.

The final bounded clothing pilot child_M_hide_wrap_bun/v4 attempt1 repairs high inner-chest coverage while retaining covered right shoulder and bare opposite outer shoulder/arm. Its remaining top_match=false is a different issue: the shallow diagonal wrap is rendered as a continuous raised round neck collar and bound armhole, so full wrap/neckline construction is not established. This improvement does not erase the earlier low-coverage failures. Source topology has approximately1-source-pixel uncertainty. Head[27,8,98,85], face[38,37,89,83], margin8px pass geometry; coarse copper bun matches and no added ornament or definite crop is observed. Exact fourth reference and context are bound in ready-016.

All18 female loincloth portraits preserve a horizontal brown bodice and both bare shoulders, but lower its upper edge substantially below the actual source high chest band. Their top_match results remain false. Source-context crops were actually viewed for each hairstyle; tiny source neckline topology remains uncertain by approximately1 source pixel.

The new male hide_wrap long_tied source has a steeper diagonal upper edge than the male bun/short sources. All six variants retain that source-specific edge height, covered viewer-right shoulder and bare opposite outer shoulder; their top_match results are true. Male hide_wrap bun/short and all18 male loincloth outputs still lower the edge across the inner chest relative to their own shallower high source. Per-look distinctions are preserved instead of transferring one pilot verdict to other source shapes.

Completed fur looks retain a high pale collar over both shoulder tops and covered brown chest with diagonal front trim. Gathered long_tied sources expose a raised mass and short visible side/back fall; no long hidden tail is inferred. Detailed loose hairs and fur marks remain much denser than the source sprite. These observations do not establish blind identity or style acceptance.

All108 final portraits have actual views and complete Inspection records.

## Failure inventory

Geometry failures (6): male fur-cloak bun v3/v5 final attempt2; female loincloth bun v1/v3/v5 and female loincloth long_tied v4 attempt1. Each has a nominal1px top flyaway margin; face-ellipse and no-definite-truncation checks pass. The same unchanged2px minimum applies to all108.

Garment failures (66): all female hide_wrap18, all female loincloth18, male hide_wrap bun6 and short6, and all male loincloth18. The male hide_wrap bun/v4 pilot fails wrap construction (raised round collar/bound armhole), while the other65 fail source-relative upper-edge coverage. Garment correspondence passes for all36 fur-cloak portraits and male hide_wrap long_tied6. Hair mismatches0; extra accessories0.

These are literal production observations, not blind-test results or owner acceptance. No earlier output, measurement, or failure was replaced by later prompt wording.

## Immutable ready manifests

| Manifest | Count | SHA256 |
|---|---:|---|
|`ready-001.json`|3|`913c9085ed81fde19de97d3df8b1a4e713ab761d6a0d493211c8acf6078ae27f`|
|`ready-002.json`|9|`e5e1a1b5077dc31b85809f51c3dfd4203ff48ac9d939dbdc66064c49504b85e0`|
|`ready-003.json`|3|`73d7ddde766091b9e5dc59eb83dc4218871301476d93776c1da0d90d736d18c8`|
|`ready-004.json`|9|`47718fdbf0b6aed5810b56e80f11f8bcd5a1f2e8e6a57459a28b74bcac1595e4`|
|`ready-005.json`|3|`10eb1e6e1b60923fd26b77f8105cff91f5354668097d45d61742b5c159eea8f0`|
|`ready-006.json`|3|`dd855534051f4822becbf7c1678dbbed13667858a8107cfa6503021b9632a43d`|
|`ready-007.json`|3|`623667acdcfd9b6b66c902e139301fcc87d00c54b2105a47bd047b9ba8b4fbd3`|
|`ready-008.json`|3|`34252a226cc8861f48cdb2fb2d69b31ecc67697f1a2b0bbb45587adc13591fb4`|
|`ready-009.json`|3|`685bd018194f0cdab0edc34ecc349a5dc2c6b4e1fd4ed779439b6e4b1da45889`|
|`ready-010.json`|3|`b8802e458a65d573bc015535469fb3acdd310006a4acb8dbe8be32cd116f9b8f`|
|`ready-011.json`|3|`b941a97522703b33c62469781fe866a2e2666bdab96d93e4c72decfad5765078`|
|`ready-012.json`|3|`bfbc53c96000d8ba70b57f90fe79ef0858eacb28bc3a59c9d65641be2758762a`|
|`ready-013.json`|1|`9975095b9c592ac40850ebfcca88ce982f99b66b0fd5d6cc9d05308101dfd851`|
|`ready-014.json`|1|`e6ae10f2d9e651178efb7c1946d24c0e016dc88092dec2786e20f5551534012b`|
|`ready-015.json`|1|`7fddfa117665370d62cce35889fa7c94d76b525035d759aabc4561cc293e96ce`|
|`ready-016.json`|1|`fcf4d5c20524d5dc1fc23b0ba3e5846c64fb46614226b489cadce407b7f2b345`|
|`ready-017.json`|6|`14e9664676d319283f7669936d8cbc58fd834554b0c34ad181c99ae533d4e4bc`|
|`ready-018.json`|3|`2a1ac417da57f96571cf22cbfcbe26e12e0f8e09ab015cb35a47fccb41c0efa7`|
|`ready-019.json`|9|`703cc3594f816b0397cfbd5103dcb447e13706a78615ebcb2cc1832364d3792e`|
|`ready-020.json`|6|`458630f6dd7c5fe0f087ed7307b82a0d6a8d75d65771b4e1e5199e7d07be43ec`|
|`ready-021.json`|3|`7e84bc5a87a1012460b435279b989d5e73b9b5f23fe86b6d0027e9cf3a3ac8d9`|
|`ready-022.json`|3|`c985b0d02e2c84bf34398a4f8dfea24eb70409601bafd71ffcd56902ab622a11`|
|`ready-023.json`|3|`56767145236c5f744e5188a7d3dc76e08383a2e3f4dbd98b0414c6055d893de4`|
|`ready-024.json`|3|`3872187dd7e8c124cbdc96ee50f9dd95f1c7ed70f6b0006ff3fbc28740e0586d`|
|`ready-025.json`|3|`d78000af95c1596dc3a3248a2a82ebaa7045ff193005e8f44d64cb4aec6f3b37`|
|`ready-026.json`|7|`5c33412b04f9104543bd1a0d4624d4b0bba90451f84d3b7fcd7933c98b4e4dc3`|
|`ready-027.json`|6|`b8b55b6340794f426807773a2ad19dd7bf080d70993a9eac0ceff9b6a259173a`|
|`ready-028.json`|4|`eb6efb201660a4c02db1e3b23891594a783e01b7524061509a7b40f72b440edc`|

## Per-portrait observations

Bounds use `[left,top,right,bottom]` in original128 coordinates. Approximate measurement uncertainty:2px.

| Look/variant | Attempt | Head bounds | Face bounds | Margin | Oval max | Geometry | Hair | Garment | Extra |
|---|---:|---|---|---:|---:|---|---|---|---|
|`child_F_fur_cloak_bun/v1`|2|`[24, 2, 99, 85]`|`[39, 33, 91, 80]`|2|0.666490|True|True|True|False|
|`child_F_fur_cloak_bun/v2`|2|`[30, 3, 96, 79]`|`[39, 33, 86, 77]`|3|0.612771|True|True|True|False|
|`child_F_fur_cloak_bun/v3`|2|`[26, 4, 99, 80]`|`[36, 31, 91, 77]`|4|0.737799|True|True|True|False|
|`child_F_fur_cloak_bun/v4`|2|`[28, 4, 98, 83]`|`[38, 32, 90, 76]`|4|0.660194|True|True|True|False|
|`child_F_fur_cloak_bun/v5`|2|`[29, 2, 96, 80]`|`[40, 32, 87, 76]`|2|0.608541|True|True|True|False|
|`child_F_fur_cloak_bun/v6`|2|`[28, 3, 96, 84]`|`[39, 35, 88, 77]`|3|0.572831|True|True|True|False|
|`child_F_fur_cloak_long_tied/v1`|1|`[18, 2, 100, 79]`|`[39, 32, 91, 78]`|2|0.687570|True|True|True|False|
|`child_F_fur_cloak_long_tied/v2`|1|`[24, 2, 101, 82]`|`[43, 32, 94, 77]`|2|0.775897|True|True|True|False|
|`child_F_fur_cloak_long_tied/v3`|1|`[14, 3, 102, 86]`|`[39, 34, 92, 83]`|3|0.674559|True|True|True|False|
|`child_F_fur_cloak_long_tied/v4`|1|`[18, 2, 100, 85]`|`[38, 31, 91, 79]`|2|0.709390|True|True|True|False|
|`child_F_fur_cloak_long_tied/v5`|1|`[18, 3, 100, 84]`|`[39, 31, 93, 79]`|3|0.767241|True|True|True|False|
|`child_F_fur_cloak_long_tied/v6`|1|`[18, 4, 103, 88]`|`[41, 32, 94, 84]`|4|0.775897|True|True|True|False|
|`child_F_fur_cloak_short/v1`|1|`[28, 25, 100, 92]`|`[39, 46, 84, 86]`|25|0.553970|True|True|True|False|
|`child_F_fur_cloak_short/v2`|1|`[26, 26, 99, 94]`|`[41, 46, 88, 89]`|26|0.587461|True|True|True|False|
|`child_F_fur_cloak_short/v3`|1|`[27, 23, 105, 94]`|`[38, 45, 85, 86]`|23|0.580313|True|True|True|False|
|`child_F_fur_cloak_short/v4`|1|`[28, 26, 96, 92]`|`[42, 47, 87, 88]`|26|0.542844|True|True|True|False|
|`child_F_fur_cloak_short/v5`|1|`[26, 26, 97, 94]`|`[39, 46, 86, 89]`|26|0.612771|True|True|True|False|
|`child_F_fur_cloak_short/v6`|1|`[26, 26, 100, 94]`|`[39, 46, 83, 87]`|26|0.572831|True|True|True|False|
|`child_F_hide_wrap_bun/v1`|1|`[31, 21, 95, 93]`|`[39, 46, 86, 90]`|21|0.633851|True|True|False|False|
|`child_F_hide_wrap_bun/v2`|1|`[33, 22, 95, 92]`|`[40, 47, 87, 89]`|22|0.587461|True|True|False|False|
|`child_F_hide_wrap_bun/v3`|1|`[33, 23, 95, 92]`|`[42, 45, 88, 89]`|23|0.587461|True|True|False|False|
|`child_F_hide_wrap_bun/v4`|1|`[31, 21, 95, 94]`|`[40, 46, 86, 90]`|21|0.608541|True|True|False|False|
|`child_F_hide_wrap_bun/v5`|1|`[31, 19, 96, 94]`|`[40, 46, 87, 90]`|19|0.608541|True|True|False|False|
|`child_F_hide_wrap_bun/v6`|1|`[34, 22, 96, 94]`|`[42, 46, 86, 89]`|22|0.539941|True|True|False|False|
|`child_F_hide_wrap_long_tied/v1`|1|`[17, 23, 96, 97]`|`[40, 49, 88, 90]`|17|0.608541|True|True|False|False|
|`child_F_hide_wrap_long_tied/v2`|1|`[20, 24, 96, 96]`|`[40, 45, 88, 89]`|20|0.587461|True|True|False|False|
|`child_F_hide_wrap_long_tied/v3`|1|`[21, 23, 96, 97]`|`[44, 43, 90, 89]`|21|0.639114|True|True|False|False|
|`child_F_hide_wrap_long_tied/v4`|1|`[24, 28, 96, 98]`|`[43, 49, 88, 90]`|24|0.608541|True|True|False|False|
|`child_F_hide_wrap_long_tied/v5`|1|`[23, 27, 98, 94]`|`[42, 48, 89, 88]`|23|0.592431|True|True|False|False|
|`child_F_hide_wrap_long_tied/v6`|1|`[19, 25, 100, 99]`|`[43, 47, 91, 91]`|19|0.709390|True|True|False|False|
|`child_F_hide_wrap_short/v1`|1|`[27, 27, 97, 92]`|`[40, 50, 86, 89]`|27|0.587461|True|True|False|False|
|`child_F_hide_wrap_short/v2`|1|`[25, 25, 96, 91]`|`[40, 46, 88, 88]`|25|0.567121|True|True|False|False|
|`child_F_hide_wrap_short/v3`|1|`[27, 28, 97, 93]`|`[38, 49, 88, 90]`|27|0.660194|True|True|False|False|
|`child_F_hide_wrap_short/v4`|1|`[27, 22, 97, 89]`|`[41, 48, 87, 88]`|22|0.542844|True|True|False|False|
|`child_F_hide_wrap_short/v5`|1|`[26, 23, 98, 91]`|`[40, 46, 90, 89]`|23|0.639114|True|True|False|False|
|`child_F_hide_wrap_short/v6`|1|`[26, 23, 99, 92]`|`[40, 47, 89, 90]`|23|0.633851|True|True|False|False|
|`child_F_loincloth_bun/v1`|1|`[28, 1, 98, 82]`|`[41, 38, 91, 79]`|1|0.572186|False|True|False|False|
|`child_F_loincloth_bun/v2`|1|`[33, 4, 97, 79]`|`[43, 35, 90, 76]`|4|0.599174|True|True|False|False|
|`child_F_loincloth_bun/v3`|1|`[28, 1, 97, 81]`|`[42, 34, 90, 78]`|1|0.618774|False|True|False|False|
|`child_F_loincloth_bun/v4`|1|`[26, 2, 95, 81]`|`[37, 40, 76, 78]`|2|0.539641|True|True|False|False|
|`child_F_loincloth_bun/v5`|1|`[26, 1, 96, 85]`|`[40, 37, 88, 81]`|1|0.510538|False|True|False|False|
|`child_F_loincloth_bun/v6`|1|`[32, 8, 97, 87]`|`[43, 40, 90, 84]`|8|0.544810|True|True|False|False|
|`child_F_loincloth_long_tied/v1`|1|`[13, 2, 98, 86]`|`[39, 33, 90, 79]`|2|0.639114|True|True|False|False|
|`child_F_loincloth_long_tied/v2`|1|`[15, 2, 100, 88]`|`[40, 34, 93, 80]`|2|0.704001|True|True|False|False|
|`child_F_loincloth_long_tied/v3`|1|`[9, 3, 101, 86]`|`[38, 31, 92, 80]`|3|0.737799|True|True|False|False|
|`child_F_loincloth_long_tied/v4`|1|`[15, 1, 98, 85]`|`[40, 34, 90, 78]`|1|0.618774|False|True|False|False|
|`child_F_loincloth_long_tied/v5`|1|`[14, 7, 98, 86]`|`[40, 38, 92, 83]`|7|0.600595|True|True|False|False|
|`child_F_loincloth_long_tied/v6`|1|`[12, 3, 103, 89]`|`[39, 37, 93, 82]`|3|0.647419|True|True|False|False|
|`child_F_loincloth_short/v1`|1|`[18, 7, 100, 82]`|`[35, 31, 91, 78]`|7|0.767241|True|True|False|False|
|`child_F_loincloth_short/v2`|1|`[20, 8, 99, 83]`|`[37, 32, 89, 79]`|8|0.687570|True|True|False|False|
|`child_F_loincloth_short/v3`|1|`[22, 7, 100, 83]`|`[38, 31, 91, 79]`|7|0.709390|True|True|False|False|
|`child_F_loincloth_short/v4`|1|`[23, 6, 100, 81]`|`[38, 33, 90, 78]`|6|0.639114|True|True|False|False|
|`child_F_loincloth_short/v5`|1|`[23, 4, 99, 79]`|`[41, 32, 90, 77]`|4|0.660194|True|True|False|False|
|`child_F_loincloth_short/v6`|1|`[25, 9, 98, 81]`|`[40, 39, 90, 79]`|9|0.528168|True|True|False|False|
|`child_M_fur_cloak_bun/v1`|2|`[27, 2, 97, 81]`|`[37, 35, 87, 77]`|2|0.626550|True|True|True|False|
|`child_M_fur_cloak_bun/v2`|2|`[27, 2, 95, 82]`|`[36, 34, 87, 77]`|2|0.674559|True|True|True|False|
|`child_M_fur_cloak_bun/v3`|2|`[25, 1, 99, 84]`|`[36, 35, 90, 81]`|1|0.654959|False|True|True|False|
|`child_M_fur_cloak_bun/v4`|2|`[25, 2, 96, 80]`|`[38, 35, 88, 77]`|2|0.599174|True|True|True|False|
|`child_M_fur_cloak_bun/v5`|2|`[25, 1, 96, 86]`|`[35, 35, 89, 81]`|1|0.684401|False|True|True|False|
|`child_M_fur_cloak_bun/v6`|2|`[25, 2, 95, 86]`|`[36, 35, 89, 82]`|2|0.654959|True|True|True|False|
|`child_M_fur_cloak_long_tied/v1`|1|`[18, 2, 101, 83]`|`[38, 39, 91, 80]`|2|0.555544|True|True|True|False|
|`child_M_fur_cloak_long_tied/v2`|1|`[21, 2, 101, 85]`|`[38, 36, 92, 83]`|2|0.636098|True|True|True|False|
|`child_M_fur_cloak_long_tied/v3`|1|`[14, 2, 102, 87]`|`[35, 32, 93, 82]`|2|0.745422|True|True|True|False|
|`child_M_fur_cloak_long_tied/v4`|1|`[21, 3, 101, 87]`|`[38, 37, 91, 83]`|3|0.589567|True|True|True|False|
|`child_M_fur_cloak_long_tied/v5`|1|`[17, 2, 104, 89]`|`[39, 34, 94, 83]`|2|0.734477|True|True|True|False|
|`child_M_fur_cloak_long_tied/v6`|1|`[16, 2, 103, 90]`|`[37, 37, 94, 84]`|2|0.677894|True|True|True|False|
|`child_M_fur_cloak_short/v1`|1|`[24, 8, 99, 80]`|`[39, 37, 88, 78]`|8|0.535848|True|True|True|False|
|`child_M_fur_cloak_short/v2`|1|`[25, 9, 101, 83]`|`[41, 44, 90, 81]`|9|0.497103|True|True|True|False|
|`child_M_fur_cloak_short/v3`|1|`[24, 11, 100, 84]`|`[40, 43, 90, 82]`|11|0.512265|True|True|True|False|
|`child_M_fur_cloak_short/v4`|1|`[26, 9, 101, 84]`|`[38, 42, 91, 81]`|9|0.524479|True|True|True|False|
|`child_M_fur_cloak_short/v5`|1|`[25, 11, 100, 83]`|`[39, 42, 89, 81]`|11|0.470760|True|True|True|False|
|`child_M_fur_cloak_short/v6`|1|`[23, 8, 100, 84]`|`[40, 43, 90, 82]`|8|0.512265|True|True|True|False|
|`child_M_hide_wrap_bun/v1`|1|`[28, 22, 98, 95]`|`[39, 53, 89, 92]`|22|0.678230|True|True|False|False|
|`child_M_hide_wrap_bun/v2`|1|`[30, 18, 98, 90]`|`[39, 49, 86, 88]`|18|0.592431|True|True|False|False|
|`child_M_hide_wrap_bun/v3`|1|`[27, 17, 98, 90]`|`[37, 55, 87, 88]`|17|0.646150|True|True|False|False|
|`child_M_hide_wrap_bun/v4`|1|`[27, 8, 98, 85]`|`[38, 37, 89, 83]`|8|0.562191|True|True|False|False|
|`child_M_hide_wrap_bun/v5`|1|`[32, 25, 94, 93]`|`[43, 61, 85, 90]`|25|0.538810|True|True|False|False|
|`child_M_hide_wrap_bun/v6`|1|`[33, 20, 93, 88]`|`[44, 57, 85, 86]`|20|0.458928|True|True|False|False|
|`child_M_hide_wrap_long_tied/v1`|1|`[22, 20, 94, 90]`|`[39, 55, 85, 87]`|20|0.572831|True|True|True|False|
|`child_M_hide_wrap_long_tied/v2`|1|`[27, 26, 91, 91]`|`[42, 58, 83, 87]`|26|0.500000|True|True|True|False|
|`child_M_hide_wrap_long_tied/v3`|1|`[22, 25, 92, 89]`|`[40, 54, 85, 86]`|22|0.528660|True|True|True|False|
|`child_M_hide_wrap_long_tied/v4`|1|`[25, 26, 91, 92]`|`[40, 59, 81, 89]`|25|0.587461|True|True|True|False|
|`child_M_hide_wrap_long_tied/v5`|1|`[25, 28, 94, 92]`|`[43, 59, 85, 89]`|25|0.517730|True|True|True|False|
|`child_M_hide_wrap_long_tied/v6`|1|`[25, 22, 94, 93]`|`[43, 57, 86, 89]`|22|0.539941|True|True|True|False|
|`child_M_hide_wrap_short/v1`|1|`[24, 24, 94, 88]`|`[38, 54, 85, 86]`|24|0.580313|True|True|False|False|
|`child_M_hide_wrap_short/v2`|1|`[29, 24, 96, 87]`|`[42, 54, 87, 84]`|24|0.468880|True|True|False|False|
|`child_M_hide_wrap_short/v3`|1|`[29, 25, 95, 86]`|`[43, 55, 85, 83]`|25|0.406783|True|True|False|False|
|`child_M_hide_wrap_short/v4`|1|`[26, 20, 97, 88]`|`[41, 50, 85, 85]`|20|0.486262|True|True|False|False|
|`child_M_hide_wrap_short/v5`|1|`[29, 27, 96, 89]`|`[42, 55, 86, 87]`|27|0.500000|True|True|False|False|
|`child_M_hide_wrap_short/v6`|1|`[27, 22, 94, 88]`|`[42, 54, 85, 85]`|22|0.463018|True|True|False|False|
|`child_M_loincloth_bun/v1`|1|`[31, 24, 93, 92]`|`[42, 61, 84, 89]`|24|0.539941|True|True|False|False|
|`child_M_loincloth_bun/v2`|1|`[37, 26, 93, 93]`|`[47, 62, 84, 90]`|26|0.517632|True|True|False|False|
|`child_M_loincloth_bun/v3`|1|`[30, 23, 96, 94]`|`[44, 60, 86, 91]`|23|0.582840|True|True|False|False|
|`child_M_loincloth_bun/v4`|1|`[33, 23, 94, 92]`|`[42, 58, 84, 89]`|23|0.539941|True|True|False|False|
|`child_M_loincloth_bun/v5`|1|`[31, 20, 94, 91]`|`[43, 54, 85, 87]`|20|0.477789|True|True|False|False|
|`child_M_loincloth_bun/v6`|1|`[31, 20, 91, 92]`|`[42, 55, 83, 88]`|20|0.519601|True|True|False|False|
|`child_M_loincloth_long_tied/v1`|1|`[27, 23, 95, 92]`|`[41, 55, 86, 90]`|23|0.584265|True|True|False|False|
|`child_M_loincloth_long_tied/v2`|1|`[29, 25, 94, 91]`|`[43, 57, 85, 88]`|25|0.497390|True|True|False|False|
|`child_M_loincloth_long_tied/v3`|1|`[28, 23, 95, 93]`|`[42, 53, 85, 90]`|23|0.561021|True|True|False|False|
|`child_M_loincloth_long_tied/v4`|1|`[26, 22, 95, 94]`|`[41, 58, 85, 91]`|22|0.606084|True|True|False|False|
|`child_M_loincloth_long_tied/v5`|1|`[24, 24, 92, 90]`|`[40, 54, 83, 87]`|24|0.547521|True|True|False|False|
|`child_M_loincloth_long_tied/v6`|1|`[26, 22, 93, 92]`|`[41, 56, 84, 89]`|22|0.563185|True|True|False|False|
|`child_M_loincloth_short/v1`|1|`[33, 25, 92, 84]`|`[43, 53, 83, 82]`|25|0.390881|True|True|False|False|
|`child_M_loincloth_short/v2`|1|`[30, 24, 93, 86]`|`[42, 54, 84, 84]`|24|0.445636|True|True|False|False|
|`child_M_loincloth_short/v3`|1|`[29, 24, 96, 88]`|`[41, 49, 84, 84]`|24|0.468880|True|True|False|False|
|`child_M_loincloth_short/v4`|1|`[32, 25, 94, 88]`|`[42, 54, 83, 86]`|25|0.481139|True|True|False|False|
|`child_M_loincloth_short/v5`|1|`[29, 28, 93, 88]`|`[41, 56, 83, 86]`|28|0.504383|True|True|False|False|
|`child_M_loincloth_short/v6`|1|`[31, 26, 92, 86]`|`[42, 55, 84, 83]`|26|0.428994|True|True|False|False|

## Validation and limits

Fresh validation checked108 schema objects, unique exact look/variant identities, matching attempts, unchanged geometry results, and every bound request/raw/canonical/sprite/observation/panel/overlay hash. Ready manifests were sealed only after every relevant overlay was viewed. No scoring or acceptance inference replaces these literal observations.
