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
