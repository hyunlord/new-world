# Pass 20 evaluation: game_v25 vs game_v24

Seeds 19400–19431 (32 worlds), 120 days, paired by seed; Pass 17's definitions and bootstrap (2,000 world resamples, 95 % percentile interval). Rules registered in METRICS.md `## Pass 20`.

| # | rule | value | verdict |
|---|---|---|---|
| T1 | ≥ 0.6 of multi-village worlds deliver; failed-trip share ≤ 0.1 | 0.474 of 19 worlds; failed 0.000 (0/71 porters) | FAIL |
| T2a | daughter non-old-age deaths v25 − v24 ≤ 0 per multi-village world; daughters alive at day 118 ≥ v24 | 1.053 [0.000, 3.105] (26 vs 6); alive 24 vs 24 | FAIL |
| T2b | (reported) aided villages, region-matched, deaths v25 − v24 | -0.444 [-1.333, 0.000]; matched 9/9 | reported |
| T3 | ≥ 0.3 dispute, ≥ 0.15 quarrel; estranged share at day 120 ≤ 0.5 | dispute 0.684, quarrel 0.684, estranged 0.438 of 32 pairs | PASS |
| T4 | R2-alive: relative change within +/-0.10 | 56.844 vs 56.500 (stat 0.006 [0.000, 0.019]) | PASS |
| T4 | R2-deaths: non-old-age deaths difference <= +1.5/world | 1.906 vs 1.375 (stat 0.531 [0.000, 1.500]) | PASS |
| T4 | R2-shelters: completed shelters ratio >= 0.90 | 6.344 vs 6.344 (stat 1.000 [1.000, 1.000]) | PASS |
| T4 | R2-food: noon food relative change within +/-0.15 | 416.270 vs 420.826 (stat -0.011 [-0.030, 0.001]) | PASS |
| T4 | R2-wood: noon wood relative change within +/-0.15 | 108.611 vs 108.670 (stat -0.001 [-0.004, 0.003]) | PASS |
| T4 | R2-social: pooled adult social rate ratio >= 0.90 | 1.741 vs 1.735 (stat 1.003 [1.001, 1.005]) | PASS |
| T4 | F1 within ± 0.05 of game_v24 | 0.681 [0.641, 0.717] vs 0.675 [0.641, 0.703] (diff 0.0053) | PASS |
| T4 | F5 within ± 0.05 of game_v24 | 0.000 [0.000, 0.000] vs 0.000 [0.000, 0.000] (diff 0.0000) | PASS |
| T7 | tallies = chronicle entries; withdrawn = delivered + returned + lost; no village change on a trip | mismatched worlds 0, unbalanced trips 0, village changes 0 | PASS (C1–C4: see chronicle.md) |

## Porters (T1)

Trips 38, porters 71, delivered food 634, wood 34, turned back 0, loads lost 0 (0 units); departure causes {0: 67, 1: 4} (0 aid food, 1 aid wood, 2 exchange food, 3 exchange wood).

| seed | villages | trips | porters | delivered | turned back | food | wood | lost | disputes | quarrels | estranged | reconciled | daughter deaths v25/v24 | daughters alive d118 v25/v24 | pairs at end [a, b, value, estranged, base] |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 19400 | 2 | 1 | 2 | 2 | 0 | 20 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 1/1 | [[1, 2, 340, 0, 300]] |
| 19401 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 1/1 | [[1, 2, 300, 0, 300]] |
| 19402 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 178 | 56 | 1 | 0 | 0/0 | 1/1 | [[1, 2, -1000, 1, 300]] |
| 19403 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 1/1 | [[1, 2, 300, 0, 300]] |
| 19404 | 2 | 4 | 8 | 8 | 0 | 80 | 0 | 0 | 285 | 78 | 1 | 0 | 0/0 | 1/1 | [[1, 2, -1000, 1, 300]] |
| 19405 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [] |
| 19406 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [] |
| 19407 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 35 | 7 | 1 | 0 | 0/0 | 1/1 | [[1, 2, -304, 1, 300]] |
| 19408 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [[1, 2, 300, 0, 300]] |
| 19409 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 185 | 67 | 1 | 0 | 0/0 | 2/2 | [[1, 2, 300, 0, 300], [1, 3, -1000, 1, 300], [2, 3, 150, 0, 150]] |
| 19410 | 2 | 2 | 4 | 4 | 0 | 0 | 34 | 0 | 318 | 101 | 1 | 0 | 0/0 | 1/1 | [[1, 2, -1000, 1, 300]] |
| 19411 | 2 | 5 | 10 | 10 | 0 | 94 | 0 | 0 | 47 | 11 | 1 | 0 | 0/0 | 1/1 | [[1, 2, -515, 1, 300]] |
| 19412 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 1/1 | [[1, 2, 300, 0, 300]] |
| 19413 | 2 | 2 | 4 | 4 | 0 | 35 | 0 | 0 | 121 | 33 | 1 | 0 | 1/0 | 1/1 | [[1, 2, -1000, 1, 300]] |
| 19414 | 3 | 5 | 10 | 10 | 0 | 99 | 0 | 0 | 69 | 17 | 2 | 0 | 24/5 | 2/2 | [[1, 2, -353, 1, 300], [1, 3, -254, 1, 300], [2, 3, 150, 0, 150]] |
| 19415 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [] |
| 19416 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [] |
| 19417 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 2 | 0 | 0 | 0/0 | 2/2 | [[1, 2, 300, 0, 300], [1, 3, 68, 0, 300], [2, 3, 150, 0, 150]] |
| 19418 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [] |
| 19419 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [] |
| 19420 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 115 | 30 | 1 | 0 | 0/0 | 1/1 | [[1, 2, -1000, 1, 300]] |
| 19421 | 3 | 1 | 2 | 2 | 0 | 20 | 0 | 0 | 94 | 17 | 2 | 0 | 0/0 | 2/2 | [[1, 2, -274, 1, 300], [1, 3, -1000, 1, 300], [2, 3, 150, 0, 150]] |
| 19422 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [] |
| 19423 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [] |
| 19424 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [] |
| 19425 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 282 | 78 | 1 | 0 | 0/0 | 1/1 | [[1, 2, -1000, 1, 300]] |
| 19426 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [] |
| 19427 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [] |
| 19428 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [] |
| 19429 | 2 | 1 | 2 | 2 | 0 | 20 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 1/1 | [[1, 2, 340, 0, 300]] |
| 19430 | 4 | 17 | 29 | 29 | 0 | 266 | 0 | 0 | 101 | 14 | 1 | 0 | 1/1 | 3/3 | [[1, 2, 629, 0, 300], [1, 3, -649, 1, 300], [1, 4, 420, 0, 300], [2, 3, 150, 0, 150], [2, 4, 150, 0, 150], [3, 4, -1000, 0, 150]] |
| 19431 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 | 0/0 | [] |

## T7 totals (tally, ledger or chronicle)

- departed: 71 / 71
- delivered: 71 / 71
- turned_back: 0 / 0
- quarrels: 511 / 511
- estranged: 14 / 14
- reconciled: 0 / 0
- trip_records: 71 / 71
- withdrawn: 668 / 668
- delivered_units: 668 / 668
- returned_units: 0 / 0
- lost_units: 0 / 0
- units in flight at day 120: 0

T5 (identity) and T6 (performance) are judged outside this script.
