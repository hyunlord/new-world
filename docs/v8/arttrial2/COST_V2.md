# TRIAL2b cost — explicit accounting scopes

Currency is **NOT_EXPOSED**. Full workflow time is **NOT_MEASURED**. The historical freeze remains [COST.md](COST.md), backed by `accounting-final.json` outside the repository: 79 production calls, 116 review calls and one negative-control call, 196 distinct outputs. V2 does not erase rejected or superseded outputs.

| Production scope | New v2 image calls | Accounting unit |
|---|---:|---|
| Environment retries | 11 | Eight retries followed by three final retries; five numeric/model passes |
| Calm ground | 12 | Six textures, two attempts each; separate replacement campaign |
| People | 8 | Two person-anchor calls plus six parts/overlay sheets |
| Autotile | 5 | Straight ×2, corner ×2, generated floor boundary ×1; floor source reused from ground |
| Total incremental production | 36 | Outputs, not accepted assets |
| Historical + incremental production | 115 | 79 + 36; no review calls included |

The environment spending ledger is **59 historical + 11 retries + 12 calm-ground = 82 calls**. The 38-asset re-score/retry cohort uses 70 calls including sunk historical spending; the 12 calm-ground calls are reported separately and do not increase that cohort's accepted denominator. Six new calm textures replace existing slots, not six extra v1.0 inventory units. Derived ghosts, cutouts, boards, mask variants and Godot captures add no image generation calls.

People expose 42 produced part files, 40 active after supersession. The conservative per-part attempt numerator is **46**, including old work and charging four replacement heads as try 2; it is not 46 image calls. There are six sheet calls plus two anchor calls. Fresh style model: **40/40 pass**. Frozen numeric and combined checks: **2/40 pass**, therefore C7 **46/2 = 23**, FAIL. Whole-person reference measurements applied to small parts remain a documented instrument limitation; neither changing the denominator to 40 model passes nor inventing accepted parts resolves it.

## Reviews are separate spending

The fresh person style review produced **14 actual scorecard outputs for 40 candidates**, plus one rejected pre-generation request exceeding the reference-path limit. The rejection is reported separately and is not an output. Environment retry review produced **5 actual scorecard calls**. The final environment model ledger records **56 successful outputs, zero tool errors = 12 superseded pilots + 38 fresh original candidates + 6 final calm-ground candidates**. All six calm candidates pass the model, separately from the original cohort. With 14 people scorecards and five retry scorecards, v2 adds **75 review outputs**. Historical 116 + 75 = **191 total review outputs**. Together with **115 production outputs and one historical negative-control output**, the two trials total **307 image outputs**. The single rejected pre-generation reference-count request remains excluded from output totals and explicitly recorded as an error attempt. The historical 116 review calls remain charged. Blind image-reading agents are evaluation work, not generated-image outputs; their elapsed time is not included in production-call timing.

## Observed timing, not service-time estimates

Eleven retry generation records at `trial2b/env-retries/*_generation.json` report **281.228 s summed wall intervals**, range **19.950–28.619 s/call**. These are observed tool waits, not billed GPU time. The five retry scorecard calls are separate from those generation intervals.

`trial2b/people-model/calls.json` has request timestamps for 13 of 14 outputs. Request-to-output-file-mtime brackets total **804.891174 s**, range **18.308093–122.364972 s**; the first request time is unknown. These overlapping brackets are not additive project elapsed time. The first-to-last output interval is not the full review duration because it excludes the first request. Filesystem output time is an observation proxy, not a server completion timestamp.

The frozen `trial2b/model-review/calls-ledger.json` contains all **56** UTC before-await/after-return brackets: **10464 s sum**, **122–332 s/call**, and **1556 s interval union**. Timestamps have one-second resolution. The union measures occupied logged intervals, not project duration. It is not added to the overlapping person or retry-review unions. Source identities are distinct across the 56 calls; the five retry records and 14 people records supply the other review outputs.

Ground/people/autotile generation logs preserve prompt and output identities but do not supply a complete start/end timing ledger here. Their unrecorded times remain **NOT_MEASURED**, not zero. Do not infer call duration from consecutive file mtimes. No total v2 wall time, isolated model latency or successful-production throughput is established.

Historical production interval sum was 4871.290 s and union 1555.080 s; historical all-category timed sum was 14459.780 s and union 3646.918 s (194/196 timed brackets). These historical brackets remain valid observations, but do not measure the revised workflow. Adding v2 generation-only waits to historical production gives a partial recorded sum of **5152.518 s**, with 25 incremental production calls lacking comparable intervals in this accounting. It is not the time for all 115 production calls.

`ENV_ACCEPTANCE_V2.json` now verifies combined acceptance **30/38 initially and 35/38 after retries**, with all 35 semantically eligible. C1 original-cohort spending is **70/35 = 2.000**; including calm replacement spending gives **82/35 = 2.343**. The latter charges replacement costs without claiming those separate candidates accepted. The final join records `promoted=true` for the 35 eligible original/retry assets. [PROJECTION.md](PROJECTION.md) keeps successful-output forecasting separate from attempted-call arithmetic.
