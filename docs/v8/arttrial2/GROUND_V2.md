# TRIAL2b ground: tone and seam evidence

Status: **6/6 tone cap PASS; 6/6 original numeric seam limits PASS; fresh model6/6PASS but frozen numeric0/6PASS, therefore combined style0/6FAIL.** These are candidates in `art/style05/ground_v2/`, not accepted environment assets. Readability improvement G1 is measured on the people boards, not inferred from these pixel metrics.

## Method

`art/pipeline2/ground_v2.py` downsamples each full-bleed original to 512×512 with Lanczos. Person foreground is normalized to a maximum dimension of 512 before measuring, using `judge.foreground`: alpha <128 and magenta are excluded, occupied bounds cropped, aspect ratio retained. Both person and ground use the same normalized resolution and the normalized sRGB coefficients 0.2125 R + 0.7154 G + 0.0721 B (the same convention as `skimage.color.rgb2gray`); these are not linear-light radiometric luminance measurements. Ground saturation is mean HSV saturation.

Frozen `person_v2.png` SHA256: `5818c46b0e7bcb23ec6e627d85682ad535a2dad79610e646f3eec8146a8e5f4a`. Foreground luminance standard deviation is **0.271425623**; 60% cap is **0.162855374**. Ground changes use one global RGB contrast multiplier about its channel means, followed by one global chroma multiplier about pixel luminance. There is no palette quantization, generated detail repainting, outline forcing, or local seam correction. A 1/255 contrast margin accounts for final 8-bit rounding; saturation bisection evaluates the actual rounded 8-bit result against a 0.348 target.

The original seam criteria in `art/pipeline2/ground.py` are unchanged: both RGB opposing-edge mean absolute errors ≤18/255, both ratios to interior neighboring-pixel MAE ≤2. All values below are final capped 512×512 pixels. A pass describes this numeric test, not invisible repetition or semantic/style acceptance.

## Selected candidates

| Material | Selected try / generated | Luminance std | Mean saturation | H / V seam MAE (0–255) | H / V seam ratio | Mirror |
|---|---:|---:|---:|---|---|---|
| meadow | 2 / 2 | 0.023738 | 0.322906 | 4.560 / 4.242 | 1.417 / 1.269 | no |
| dark_grass | 2 / 2 | 0.019445 | 0.347552 | 4.445 / 3.768 | 1.678 / 1.356 | no |
| forest_floor | 2 / 2 | 0.025885 | 0.347978 | 5.077 / 4.562 | 1.525 / 1.333 | no |
| packed_earth | 2 / 2 | 0.034674 | 0.347959 | 6.668 / 7.198 | 1.326 / 1.388 | no |
| river_bank | 2 / 2 | 0.018209 | 0.174892 | 3.517 / 3.490 | 1.474 / 1.409 | no |
| hill_rock | 2 / 2 | 0.052821 | 0.159860 | 6.931 / 8.913 | 1.093 / 1.207 | no |

All selected candidates already met the luminance cap before tone processing, so contrast scale was 1.0; final tone adjustments desaturated dark grass, forest floor and packed earth only. Meadow, river bank and hill rock try2 already met both caps. **The major contrast reduction came from new generation, not post-processing.**

Twelve generated ground images total, selected six technical candidates, zero automatic style acceptances. All six selected images are try2; no asset used try3 and no mirror-blend was used. Packed earth try1 failed the original seam test (historical capped MAE 25.318 / 18.203, ratios 2.205 / 1.600); hill rock try1 also failed (17.733 / 19.057, ratios 1.202 / 1.201). Both were regenerated with stronger plain-wash / opposite-edge balance prompts. Meadow, dark grass, forest floor and river bank try1 passed numeric caps/seams but were explicitly **rejected visually as too busy** after the first native-scale people capture. Their original pixels, metadata and exact historical metrics are frozen under `ground/round1_selected`; the later CLI remeasurement uses the corrected saturation implementation.

## Visual inspection and revision

Try1 meadow/forest/dark grass contained dense small foliage texture; river bank had bright water veins. Desaturation and the relative cap did not remove this competing detail. The second prompt used ground and scene anchors for medium only, plus the quieter packed-earth try2 as a density ceiling. It explicitly required near-flat wash, three or four diffuse variations, and no drawn foliage, veins, stippling or grain.

The four new originals, their native 512×512 candidates (the prototype displays ground tiles at 512px), and 3×3 repeats were directly inspected. The second set has soft broad low-contrast washes with no discrete grass blades, leaves or water veins. Fine wash texture remains, but no longer reads as hundreds of little objects. Meadow standard deviation fell from 0.089197 to 0.023738; forest floor from 0.094627 to 0.025885. All four new seam MAEs are below 5.1/255. Packed earth is similarly quiet; hill rock retains subdued broad fissures. Broad repeating wash motifs remain detectable across 3×3 tiles, especially rock; the numeric seam pass is not a claim of perfect aperiodicity. Independent second-round people-board readability remains necessary.

## Artifacts and provenance

- Final review/runtime candidates: `art/style05/ground_v2/{name}.png` and `{name}.json`; each sidecar contains selected attempt, exact generation prompt, anchor paths, builtin generator output path/ID, original and final SHA256, all executed attempt metrics, and technical-candidate status, superseded by the final style result below.
- Raw originals, exact requests and rejected outputs: `/Users/rexxa/work/arttrial2-data/trial2b/ground/{name}/tryN_raw.png`, `tryN_request.json`, `tryN_capped/metrics.json`.
- Full-size repeats: same external directory, `tryN_capped/repeat_3x3.png` (1536×1536); source image remains untouched.
- Flat runtime candidates: `/Users/rexxa/work/arttrial2-data/trial2b/ground/working/{name}.png`.
- Frozen first-round selection: `/Users/rexxa/work/arttrial2-data/trial2b/ground/round1_selected/` contains all six original selected PNG+JSON; current sidecars point to this history.
- Complete measured attempt ledger: `/Users/rexxa/work/arttrial2-data/trial2b/ground/tone_results.json`; twelve rows, retaining every try1 and try2.
- All committed candidate PNGs are below 0.60 MB, below the 1.5 MB/file cap.

Reproduction:

```sh
uv run art/pipeline2/ground_v2.py /Users/rexxa/work/arttrial2-data/trial2b/ground art/style05/anchors/person_v2.png
uv run --with pytest --with pillow --with numpy --with scikit-image python -m pytest art/pipeline2/test_ground_v2.py -q
uvx ruff check art/pipeline2/ground_v2.py art/pipeline2/test_ground_v2.py
```

Verification: the actual six-asset CLI completed; three targeted tests passed (8-bit rounding at a low contrast cap, chroma-background exclusion at matched resolution, and saturation rounding of a flat shade); Ruff passed. Basedpyright on both new Python files passed with 0 errors and 0 warnings. An initial synthetic contrast test exposed a 0.050735 output against a 0.05 cap, fixed by reserving an absolute 1/255 rounding margin. A second synthetic flat-color regression reproduced saturation 0.352 against the 0.35 cap; moving 8-bit rounding into every bisection evaluation fixed it. These lessons are recorded here because this trial permits edits only under `art/**` and `docs/v8/arttrial2/**`.

## Final style cross-check

`GROUND_STYLE_V2.json` binds all six exact final processed PNGs to frozen numeric checks and six actual fresh same-family image-model scorecards. Model6/6PASS; numeric0/6PASS, hence combined0/6PASS. Most quiet washes fail edge density and/or stroke distribution and wash ranges learned from the detailed original ground anchor; hill rock additionally fails saturation q75 and wash flatness. This exposes a conflict between the new calm-ground brief and the old ground reference texture density. Do not retune after observing the failures or call these accepted environment replacements. Tone/seam6/6PASS and actual readability G1FAIL remain separate. Original38C2 and35style acceptances do not change.
