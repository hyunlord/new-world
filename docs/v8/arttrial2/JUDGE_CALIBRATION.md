# Style-judge calibration — V2 combined PASS; model selectivity weak

Historical V1: numeric calibration six anchors pass; all three deliberately wrong controls fail numerically. Actual image-model calibration: five of six anchors pass, two of three wrong controls are rejected. Combined rule: **5/6 anchors pass, 3/3 wrong controls rejected**. The required 6/6 anchor gate is unmet. Do not certify production acceptance from this calibration.

## Methods and frozen thresholds

Original anchor bytes verified against `anchors/manifest.json`. Colour/edge/wash metrics use foreground crop, alpha>=128 and magenta exclusion, Lanczos normalization to256 longest edge. HSV local saturation uses9×9 windows. Stroke thickness is a separate1024-longest-edge dark-ridge/skeleton estimate near Canny edges, not literal pigment opacity. Scene aspect ratio is retained.

24 dominant Lab bins supply measured centroids; each leave-one-out fold builds its palette from only the other five images. Equal-weight mean anchor histograms are compared with symmetric chi-square. Threshold0.13273046462922072 is the maximum six-fold distance, frozen before production outcomes. Final palette uses all six. Edge-density bounds0.172255291005291–0.41087648022171835 are anchor extrema expanded30%; saturation bounds0.4352504445329481–0.5352829956344147 are exact anchor extrema. No candidate-dependent retuning.

## Actual image-model results

Each tool invocation received two locked anchor references and one candidate with a neutral scoring prompt. The image-generation tool supplies raster output rather than structured text, so its plain scorecards were transcribed and originals/hash retained externally. Underlying model version/internal context isolation is not exposed. These are actual generated-model judgments, not a substitute Codex score. Timing fields for batch-launched calibration calls are enclosing observations unless individually measured; do not sum them as true inference latency.

Scores are medium / line / light / palette; every score must be>=4.

| Candidate | Numeric | Image-model scores | Combined |
|---|---|---|---|
| person | PASS | 5/5/5/5 | PASS |
| hut | PASS | 4/5/5/5 | PASS |
| props | PASS | 4/5/4/5 | PASS |
| ground | PASS | 3/5/4/5 | FAIL |
| scene | PASS | 4/4/4/4 | PASS |
| portrait | PASS | 4/4/4/4 | PASS |
| negative_painterly | FAIL | 3/3/4/4 | FAIL |
| negative_woodcut | FAIL | 4/5/4/5 | FAIL |
| negative_photo_try1 | FAIL | 2/3/4/4 | FAIL |

Ground was rejected for richer colour and softer multi-tone modelling. Woodcut was wrongly accepted by the image model but rejected by numeric colour/saturation tests. Scene candidate scored4/4/4/4 although its duplicate reference column received5s; the inconsistency is retained, not corrected to expected results. Exact reasons and scorecard hashes: `IMAGE_MODEL_CALIBRATION.json`. Combined numeric results: `CALIBRATION_RESULTS.json`.

## Independent corroboration, not replacement

A separate fresh-context multimodal Codex reviewer gave six anchor passes and three negative rejections. Its verbatim output is `CALIBRATION_REVIEW.json`. This disagreement is evidence of judge instability; it does not override the requested image model's failed calibration.

## Verification scope

Eight behavioral tests pass, including absent-score fail-closed behavior, invalid JSON CLI scores, numeric boundaries, calibration and empty/chroma-only images. Ruff passes. Type-check errors were fixed without changing calibration values;130 upstream/dynamic typing warnings remain, so strict typecheck is NOT_GREEN. Game gate/identity is not an art-only gate and was not run; no game integration or behavior-preservation claim is made.

## V2 bounded relative-reference calibration

One neutral relative-reference round, nine calls, no retries. Exact prompt: JUDGE_PROTOCOL_V2.md. Numeric calibration and original images remain unchanged. Combined rule is numeric PASS AND all four model scores >=4, as preregistered. Result: **anchors6/6 PASS, negatives3/3 rejected**. Model alone rejects **0/3** negatives; every negative rejection relies on the numeric signal. This satisfies the combined calibration but does not establish an independently discriminating model judge. Preserve both facts.

| Candidate | Model M/line/light/palette | Numeric | Combined |
|---|---|---|---|
| person | 5/5/5/5 | PASS | PASS |
| hut | 5/5/4/5 | PASS | PASS |
| props | 5/4/5/4 | PASS | PASS |
| ground | 5/5/4/5 | PASS | PASS |
| scene | 5/5/4/5 | PASS | PASS |
| portrait | 5/5/5/5 | PASS | PASS |
| negative_painterly | 5/4/5/5 | FAIL | FAIL |
| negative_woodcut | 4/4/4/4 | FAIL | FAIL |
| negative_photo_try1 | 4/4/4/4 | FAIL | FAIL |

Verbatim sentences, candidate/reference/card hashes and numeric recomputation: IMAGE_MODEL_CALIBRATION_V2.json. Raw cards remain external under image-judge-calibration-v2. Some cards repaint reference/candidate panels despite the requested plain format; only their explicit score text is transcribed, never the repainted panel as a source asset. The photo reason admits photographic rendering while assigning four4s. V1 is preserved above and in its original JSON.

Full six-file pipeline typecheck after fixes:0 errors,150 warnings (exit1); Ruff passes and ten tests pass. The earlier130-warning count applied only to the judge subset. Strict typecheck remains NOT_GREEN.

## TRIAL2b numeric judge v2 (2026-10-22): numeric calibration PASS

This is a new numeric implementation, distinct from the earlier “V2” model-prompt
round above. **12/12 true leave-one-crop-out checks pass; 3/3 wrong-style person
controls fail.** Fresh same-family model review remains a separate acceptance gate.

### Family references and scale

The original six PNGs are unchanged. `anchors/manifest_v1.json` preserves the old
manifest; `anchors/manifest.json` v2 lists twelve native-resolution crops with exact
source paths/hashes, crop rectangles and derivative hashes. Four ground quadrants
are upper-left river, upper-right forest, lower-left meadow, lower-right stone;
two hut/roof details are buildings, two log/hearth details are items, two oak/birch
canopy details are plants, and full/upper-body crops of the new person raw source
are people. Misc props use items; smoke still has no genuinely matched reference.
The old person stays withdrawn for camera history. The two new person crops share
a source and overlap: this is crop LOO, not independent-source validation. Plants
were selected by the planner's explicit visual acceptance, not by this new score.

Crops retain original source pixels; no512px reference pre-resize remains. Large
hut/tree references use native detail crops to respect1.5MB/file rather than image
quantization. Numeric candidate inputs are the **original try1 raw generations**,
not tiny delivered sprites enlarged to match references. The candidate cohort
remains the same38 generation attempts, all counted as try1. Original working-image
hashes are retained alongside raw hashes for traceability. Delivered-size
readability and extraction remain separate QA. Measurement alone normalizes
foreground longest edge to256px for local measures and1024px for stroke measures.
This matches comparison sampling, but does not guarantee identical physical ink
scale across differently framed crops; the limitation remains explicit.

### Measurements and calibration revisions

No global Lab histogram remains. Eight components measure edge density, local HSV
saturation q25/q75, stroke width q25/q50/q75/q90, and wash flatness. Stroke width is
the distance transform on dark ridges within4px of Canny edges, sampled on ridge
skeletons; direct distance transforms on one-pixel Canny edges would be meaningless.
Wash variance uses connected regions after closing1px edge gaps, erodes region
boundaries to remove edge antialiasing, rejects interiors smaller than16px, then
weights interior luminance variance by pixel count.

Family ranges use extrema plus half their midpoint, with absolute floors
`.04,.12,.12,1,2,2,1,.003`. Wash uses multiplicative bounds `[min/2.2,max*2.2]`.
The stroke q90 tail uses25% of the reference midpoint, at least1.75px. These are
empirically calibrated bounds, not physically derived constants. The1.75px floor
covers the1.65685px building-reference tail difference in LOO. No environment
candidate scores were used to choose final thresholds.

Retained development evidence documents the instrument corrections:

1. Initial seven-feature implementation passed10/10 crop LOO after a building
   tolerance adjustment, but accepted the wrong-style controls. Those controls
   were also incorrectly tested across environment families despite being people.
2. Region-gap closing/erosion removed boundary contamination and exposed the
   woodcut's flat wash. Painterly/photo still overlapped environment references.
3. New person references enabled the correct person-family control evaluation.
   Stroke q90 was added because median widths quantize at2px for both fine ink and
   photographic detail. Native source crops replaced512px thumbnails, and all
   final bounds were recalibrated using anchors/controls before raw candidate scoring.

The preliminary working-sprite rescore and earlier calibrations are historical
external evidence (`env-rescore-v2-initial.json`, `calibration-trial2b-initial.json`,
`calibration-v2-initial-final.json`, `judge-v2-initial.py` under arttrial2-data).
They are not final C2 measurements. Final evidence is `JUDGE_V2_CALIBRATION.json`.

| Person-family control | Numeric v2 | Failed components |
|---|---|---|
| painterly | FAIL | stroke q75/q90, wash flatness |
| woodcut | FAIL | stroke q90, wash flatness |
| photo | FAIL | stroke q90 |

Controls depict old overhead people while new references are front-view stylized
people. Passing this small calibration does not isolate medium from camera,
proportions or framing. No claim of general style-classification accuracy is made.

### Frozen raw first-attempt re-score

| Family | n | Numeric v2 pass | Numeric + historical model diagnostic |
|---|---:|---:|---:|
| Ground |6|6|6|
| Plants |9|9|8|
| Buildings |6|5|4|
| Items |10|4|4|
| Misc props |7|6|6|
| Total |38|30 (78.95%)|28 (73.68%)|

`ENV_RESCORE_V2.json` records every check, candidate/working hash and legacy model
scorecard identity. Legacy scores used scene references and sometimes the working
sprite rather than the raw candidate; they are **diagnostic only**. Strict v2
acceptance requires fresh reviews with two same-family crops and the raw candidate.
No automatic promotion occurs from the historical scores. Final C1/C2 must use the
fresh-review result, retaining original try counts and rejected attempts.

Reproduce with `uv run --with pillow --with numpy --with scikit-image python -m
art.pipeline2.rescore_v2`. Eight behavioral tests cover family isolation, missing/
legacy-model fail-closed behavior, all four model dimensions, blank inputs,
calibration serialization and exact numeric boundaries. Ruff passes. Basedpyright
reports0 errors and174 warnings for judge/rescore; strict typecheck is NOT_GREEN.
No game-runtime/identity claim is made for this art-only work.

### Audit corrections and existing retry inventory

A source-path audit found the six legacy ground records pointed to processed
`ground/<name>_try1/candidate.png`. The final re-score now explicitly resolves
`ground/<name>_try1.png`, the actual generation original. All six still pass and
the total remains30/38. The preceding results are retained externally as
`env-rescore-v2-before-ground-source-audit.json`; this is a provenance correction,
not a new generation. Calibration status requires both allLOO passes and all
negative rejections. Candidate certification additionally fails closed for empty,
incomplete or failed LOO evidence; a new regression reproduced the former loophole.

`ENV_RETRY_NUMERIC_V2.json` verifies sidecar/raw hashes for all existing attempts of
the eight numeric failures. Drying rack has two attempts consumed; try2 also fails
wash flatness, leaving only try3. Firewood bundle, flint pile, hide, large/small log
piles, stone axe and deadwood each have only try1 consumed in the recorded ledgers,
so their next generation is try2, with at most two remaining. No generation or
model-score substitution was performed by this audit. Nine behavioral tests pass.
The final three-file judge/rescore/retry typecheck reports0 errors,216 warnings;
Ruff and `git diff --check` pass. Warning increase reflects the added retry audit
module; strict typecheck remains NOT_GREEN.

Targeted precommit typing cleanup reduced the three-file count from216 to178
warnings, still0 errors. Concrete NumPy array/region/list annotations and explicit
write-result handling were added; no ignored diagnostics, casts or `Any` annotations
were introduced. Regenerated calibration is byte-identical (`cmp`) to the frozen
pre-cleanup JSON. Remaining diagnostics originate mainly in untyped SciPy/skimage
signatures and dynamic JSON/argparse boundaries; eliminating them requires a typed
boundary/stub refactor beyond this bounded cleanup. Strict typecheck stays
NOT_GREEN, not waived or relabeled. The same nine tests and Ruff pass.
