# ART-TRIAL2 COST — final image-call freeze

Snapshot **2026-10-22T01:32:33.796440+00:00**, leader-confirmed final image-call freeze. **196 distinct tool-output sources**: **79 production, 1 negative photo control, and 116 review calls**. Reviews comprise 18 calibration (9 V1 + 9 V2), 38 environment, 7 followup, 3 targeted crop, 25 people-part and 25 material reviews. No future image calls are assumed. Call-by-call evidence: `/Users/rexxa/work/arttrial2-data/accounting-final.json`.

The first-pass snapshot remains unchanged at `/Users/rexxa/work/arttrial2-data/accounting-firstpass.json`: 56 production and 88 total sources at 2026-10-22T00:22:54Z. This later snapshot adds 23 production calls (ground 5, plants 9, buildings 6, autotile 3), 9 calibration-v2 calls, 16 further environment reviews and 60 followup/targeted/people/material reviews. The scoreprobe is already among the first 9 calibration reviews; it is not an extra call.

## Production calls and provenance

All **79 production originals** exist, match their recorded SHA-256 and are byte-identical to their distinct tool-source files. Review sources and the negative control also exist and have snapshot hashes; missing prior hashes are null, not claimed verified. Currency cost is **NOT_EXPOSED**, not zero.

| Family | Distinct candidate asset/part units | Calls including retries | Sum of observed intervals, s | Union of observed intervals, s | Observed interval per call, s |
|---|---:|---:|---:|---:|---:|
| ground | 6 | 12 | 515.650 | 515.650 | 27.075–73.761 |
| plants | 9 | 18 | 596.075 | 596.075 | 21.747–57.397 |
| buildings | 6 | 12 | 1173.275 | 370.412 | 20.417–231.612 |
| items | 10 | 10 | 1278.776 | 227.948 | 26.279–227.948 |
| misc | 7 | 7 | 667.479 | 238.242 | 23.678–202.414 |
| people | 25 | 12 | 341.412 | 341.412 | 17.760–49.375 |
| autotile | 3 | 6 | 200.654 | 200.654 | 20.828–52.439 |
| seasons | 2 | 2 | 97.969 | 65.549 | 32.420–65.549 |

Environment has **38 distinct generated assets, 59 calls, and 2 accepted assets**. Ground, plants and buildings now have two attempts per asset; items and misc have one. The two derived ghosts add no image calls. Body sheet is one call exposing 14 parts, plus 11 individually generated overlays/equipment: 12 calls and 25 part exposures. Autotile has 3 distinct material sources attempted twice; derived wall masks/floors add no calls. Shader seasons likewise add no generation calls beyond the two recorded comparison images. Copies, boards, mirrored-ground processing and source reuse never increment the count.

## Time observations

Production interval sums total **4871.290s (81.188min)**; their cross-family interval union is **1555.080s**. The final all-category timed brackets sum to **14459.780s (240.996min)**, with cross-category union **3646.918s (60.782min)**. There are **194 timed brackets among 196 calls**. The negative-photo tool response separately reported **35.1s**, with no start/end timestamps; scoreprobe duration is unknown. Neither belongs in a timestamp union. Timing details by category and every original/source/hash are in the external accounting JSON.


| Review category | Calls | Timed brackets | Interval sum, s | Interval union, s |
|---|---:|---:|---:|---:|
| Calibration V1 | 9 | 8 | 1379.000 | 180.000 |
| Calibration V2 | 9 | 9 | 1171.075 | 546.841 |
| Environment | 38 | 38 | 4149.000 | 935.000 |
| Followup V2 | 7 | 7 | 508.000 | 221.000 |
| Targeted crop V2 | 3 | 3 | 246.000 | 82.000 |
| People parts | 25 | 25 | 1040.331 | 484.224 |
| Materials | 25 | 25 | 1095.084 | 483.465 |

Adding the separately reported photo elapsed time gives 14494.880s of heterogeneous observed-call time; scoreprobe remains unknown. This is not a billed-compute total. Per-category unions must not be summed to estimate global elapsed time.

These are observed await/batch brackets, often overlapping and including queueing/result observation. A sum is not elapsed wall time; a union is only the occupied portion of recorded intervals, not project duration, person-hours, GPU-hours or isolated service time. Unknown processing, evaluation and integration time is not zero. Runtime/performance render captures do not add image-generation calls. No image-tool billing or exact underlying image-model version was exposed.

## Acceptance and projection limits

At first-pass snapshot, numeric eligibility was **1/38**, combined acceptance **0/38**, and original calibration **FAIL** (model: anchors 5/6 pass, negatives 2/3 rejected; combined numeric/model: anchors 5/6 pass, negatives 3/3 rejected). After retries, latest numeric eligibility is **2/38**, including processed hill-rock and packed-earth candidates; both new model reviews scored 5/5/5/5 and the leader confirmed acceptance. Combined accepted count is now **2/38 generated assets (5.26%), or 2/40 including derived ghost slots (5%)**. Calibration-v2 combined checks passed 6/6 anchors and rejected 3/3 negatives, but its model alone passed all 9 inputs and rejected 0/3 negatives; numeric checks supplied all negative rejections. This does not rewrite the first calibration failure.

Environment attempts per accepted asset = **59/2 = 29.5**. Ground-family attempts per accepted asset = **12/2 = 6**; all other environment-family accepted denominators remain zero. Full successful-production/reject forecasts remain **NOT_ESTIMABLE**. Calls per attempted asset = 59/38 = 1.553 is descriptive spending, not C1. Only the two confirmed ground acceptances contribute; numeric-only results and raw model scores do not establish acceptance. Model reliability limitations remain real costs: the targeted code-only request returned an unusable illustration instead of a scorecard, but its actual call remains counted, with no inferred refund. Broader model review completion does not erase numeric failures or the V2 negative-control weakness. See [PROJECTION.md](PROJECTION.md).
