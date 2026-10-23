# Current reused-building recheck

Fresh production-source numeric checks pass **3/3**, and the fresh image-model scorecard passes **3/3**. Additional numeric checks on the exact runtime PNGs pass **1/3**. These are distinct measurement domains; the runtime failures remain recorded. No historical judgment, frozen manifest, source image or previous ledger was changed.

| Asset | Retained raw original / current runtime size | Fresh raw numeric | Fresh runtime numeric | Model medium / line / light / palette |
|---|---|---|---|---|
| hut | 1254×1254 / 192×192 | PASS | PASS | 5 / 5 / 5 / 5 |
| knapping_spot | 1254×1254 / 64×64 | PASS | FAIL: stroke_q25, stroke_q75, stroke_q90, wash_flatness | 4 / 4 / 4 / 4 |
| woodworking_bench | 1774×887 / 128×64 | PASS | FAIL: wash_flatness | 4 / 4 / 4 / 4 |

All numeric runs used the unchanged `buildings` family in `docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json` (SHA256 `7218ac77979065ec6398c77cb6156bce5bd523979ed48732945f2ab642a5d5fd`) and the existing `art/pipeline2/judge.py`. Every retained original was found and its bytes matched the original SHA256 in the asset's historical sidecar. Every calibration crop hash was verified. Thresholds were neither refitted nor relaxed.

## Representation and normalization

The 23 new built sources and the historical reused-asset numeric records measure larger raw originals. For a like-for-like production comparison, this recheck freshly measured those same retained originals. The runtime diagnostic instead measured the exact committed PNGs. Historical processing chroma-keyed, cropped and Lanczos-contained those images into their smaller runtime canvases; the complete original operations are copied into the JSON record.

The judge crops foreground and measures most features at a longest edge of 256 px; ink widths use 1024 px normalization. Enlarging a 64 px runtime raster does not recover the original ink ridges or wash detail. Knapping's raw stroke quantiles are 2 / 2 / 2.8284 / 4; the runtime estimator finds no qualifying ink ridges and returns 0 / 0 / 0 / 0. Its wash variance rises from 0.016643 to 0.044953. Woodworking-bench wash variance rises from 0.017835 to 0.029091. Both runtime wash values exceed the frozen upper bound 0.023011. These are measured failures, not grounds to alter the thresholds.

The single fresh model call viewed the **runtime** candidates, with their full geometry shown at 2× nearest enlargement, 1× and ½×. Consequently, the raw-numeric/model combination is explicitly cross-representation: raw production numeric PASS plus runtime visual model PASS. It is not a direct raw-image model judgment, and it does not erase runtime numeric failures.

## Current readings and scale

- **Hut:** four bedrolls around a lit central hearth and a clear south opening remain recognizable at 1× and ½×; larger posts punctuate the log-wall corners.
- **Knapping spot:** a dark flint/hammerstone area against pale rock and a curved bone tool reads as a compact work spot at 1×; at ½× the rock and flint merge and the pale tool carries most of the silhouette.
- **Woodworking bench:** the long wood slab, supports and tools read as a workbench at 1×; at ½× the bench silhouette survives while individual tools soften.

The executor re-inspected the unchanged `art/style05/env1/built/reuse_scale_board.png`: the hut's approximately 12–16 px wall band is comparable in scale to the new 16 px bands, with larger corner posts and different texture density. At 32 px tiles these widths become approximately 6–8 px versus 8 px. This is a non-blind scale check, not seamless-join acceptance. The model's scale sentence uses the declared measurements; the new wall bands were not included in that model call. The full model sentences and separate executor readings are retained in JSON.

## Evidence and call accounting

- [Complete current record](built_reuse_recheck.json): reservation, timestamps, full prompt, five input hashes, candidate identities, both numeric scopes, thresholds, actual score transcription, source hashes and normalization operations.
- [Actual model scorecard](../../../art/style05/env1/built/reuse_current_scorecard.png).
- External raw output: `/Users/rexxa/work/artenv1-data/built/reuse_current/reuse_current_scorecard_raw.png`.
- External whole-geometry input: `/Users/rexxa/work/artenv1-data/built/reuse_current/reuse_current_candidates.png`.
- Separate immutable pre-call reservation and call-start records remain in that external directory. Output timing is the locally recorded receipt time; the service does not expose its internal completion timestamp or model version.

The call used two frozen building-family crops, the locked original hut, the locked original scene and the candidate contact sheet: five images total. All were inspected before the call. The returned scorecard and saved preview were visually inspected before transcription. The tool was builtin `image_gen.imagegen`, not an ordinary vision-subagent review. No historical scores were supplied in the prompt.

Exactly **one model call and zero asset generations** were added: built-lane cumulative calls are **56** (47 asset attempts + 9 model judgments); the leader reserved this as global call **135**. The existing 55-call ledger remains frozen and this record is its separate supplement. The scorecard preview is an aspect-preserving geometric reduction, with no quantization or local pixel edits, and is below 1.5 MB. No owner approval or scene-readability acceptance is claimed.
