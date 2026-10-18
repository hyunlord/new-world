# Pass 21 evaluation: game_v26 vs game_v25 and game_v24

Seeds 19400–19431 (32 worlds), 120 days, paired by seed; Pass 17's definitions and bootstrap (2,000 world resamples, 95 % percentile interval). Rules registered in METRICS.md `## Pass 21`. Multi-village world: ≥ 2 settlements alive at once in game_v26.

| # | rule | value | verdict |
|---|---|---|---|
| U1 | ≥ 0.6 of multi-village worlds deliver; failed-trip share ≤ 0.1 | 0.632 (12 of 19); failed 0.000 (0/221 porters) | PASS |
| U2 | ≥ 1 exchange trip over the 32 worlds | 0 exchange trips (0 porters); exchange pair-days 0 (0 in reach, not estranged) | FAIL |
| U3 | disputes ≥ 0.3, quarrels ≥ 0.15 of multi-village worlds; estranged at d120 ≤ 0.25; quarrels while estranged ≤ 0.2; ≥ 3 estrangements → ≥ 0.5 reconciled | dispute 0.684, quarrel 0.684, estranged 9 of 32 (0.281), while estranged 36 of 369 (0.098), reconciled 13 of 22 (0.591) | FAIL |
| U4 | daughter non-old-age deaths per multi-village world v26 − v24 ≤ 0 | 0.316 [0.000, 0.895] (12 vs 6) | FAIL |
| U4 | the same against game_v25 | -0.737 [-2.264, 0.105] (12 vs 26) | PASS |
| U4 | daughters alive at day 118 ≥ game_v24 | 24 vs 24 (game_v25 24) | PASS |
| U7 | Pass 20's T7 on game_v26 + reconciliations = chronicle + costed + uncosted = disputes | mismatched worlds 0, unbalanced trips 0, village changes 0 | PASS (C1–C4: chronicle.md) |

U4 overall: SKIPPED.

## Porters (U1, U2)

Trips 113 (113 delivered), porters 221, food moved 2090, wood 101, turned back 0, loads lost 0; departure causes {0: 209, 1: 12} (0 aid food, 1 aid wood, 2 exchange food, 3 exchange wood). In-reach food-aid pair-days: 120 open, 69 between estranged villages. game_v25 delivering worlds on these seeds: 9 of 19.

## Conflict (U3)

Disputes 3199 (1762 past the daily dispute cap), quarrels 369 (1726 disputes met the quarrel cap), estrangements 22, reconciliations 13, pair-days estranged 254. game_v25 on these seeds: disputes 1834, quarrels 511, estrangements 14, reconciliations 0.

| seed | villages | trips (delivered) | porters | food | wood | exchange trips | failed | disputes (uncosted) | quarrels (capped, while estranged) | estranged | reconciled | daughter deaths v26/v25/v24 | daughters alive d118 v26/v25/v24 | pairs at end [a, b, value, estranged, base] | v25 trips delivered |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 19400 | 2 | 1 (1) | 2 | 20 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 1/1/1 | [[1, 2, 333, 0, 300]] | 1 |
| 19401 | 2 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 1/1/1 | [[1, 2, 300, 0, 300]] | 0 |
| 19402 | 2 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 182 (103) | 19 (101, 0) | 2 | 1 | 0/0/0 | 1/1/1 | [[1, 2, -234, 1, 300]] | 0 |
| 19403 | 2 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 1/1/1 | [[1, 2, 300, 0, 300]] | 0 |
| 19404 | 2 | 6 (6) | 12 | 120 | 0 | 0 | 0 | 266 (150) | 31 (147, 13) | 1 | 0 | 0/0/0 | 1/1/1 | [[1, 2, -458, 1, 300]] | 4 |
| 19405 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19406 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19407 | 2 | 3 (3) | 6 | 20 | 33 | 0 | 0 | 198 (69) | 26 (77, 0) | 2 | 1 | 0/0/0 | 1/1/1 | [[1, 2, -301, 1, 300]] | 0 |
| 19408 | 2 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [[1, 2, 300, 0, 300]] | 0 |
| 19409 | 3 | 3 (3) | 5 | 50 | 0 | 0 | 0 | 113 (49) | 17 (52, 1) | 1 | 0 | 0/0/0 | 2/2/2 | [[1, 2, 335, 0, 300], [1, 3, -340, 1, 300], [2, 3, 150, 0, 150]] | 0 |
| 19410 | 2 | 17 (17) | 34 | 300 | 34 | 0 | 0 | 289 (149) | 37 (168, 0) | 2 | 2 | 0/0/0 | 1/1/1 | [[1, 2, -180, 0, 300]] | 2 |
| 19411 | 2 | 10 (10) | 19 | 190 | 0 | 0 | 0 | 153 (24) | 29 (32, 0) | 1 | 1 | 0/0/0 | 1/1/1 | [[1, 2, -313, 0, 300]] | 5 |
| 19412 | 2 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 1/1/1 | [[1, 2, 300, 0, 300]] | 0 |
| 19413 | 2 | 30 (30) | 59 | 570 | 18 | 0 | 0 | 538 (361) | 50 (337, 1) | 2 | 2 | 0/1/0 | 1/1/1 | [[1, 2, -145, 0, 300]] | 2 |
| 19414 | 3 | 7 (7) | 13 | 130 | 0 | 0 | 0 | 571 (358) | 57 (349, 3) | 5 | 4 | 10/24/5 | 2/2/2 | [[1, 2, -217, 1, 300], [1, 3, -232, 0, 300], [2, 3, 150, 0, 150]] | 5 |
| 19415 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19416 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19417 | 3 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 4 (0) | 2 (1, 0) | 0 | 0 | 0/0/0 | 2/2/2 | [[1, 2, 300, 0, 300], [1, 3, 204, 0, 300], [2, 3, 150, 0, 150]] | 0 |
| 19418 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19419 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19420 | 2 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 142 (66) | 21 (71, 6) | 1 | 0 | 0/0/0 | 1/1/1 | [[1, 2, -454, 1, 300]] | 0 |
| 19421 | 3 | 2 (2) | 3 | 30 | 0 | 0 | 0 | 347 (216) | 35 (200, 0) | 3 | 1 | 0/0/0 | 2/2/2 | [[1, 2, -251, 1, 300], [1, 3, -301, 1, 300], [2, 3, 150, 0, 150]] | 1 |
| 19422 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19423 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19424 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19425 | 2 | 7 (7) | 14 | 120 | 16 | 0 | 0 | 303 (168) | 36 (139, 12) | 2 | 1 | 0/0/0 | 1/1/1 | [[1, 2, -415, 1, 300]] | 0 |
| 19426 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19427 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19428 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19429 | 2 | 2 (2) | 4 | 40 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 1/1/1 | [[1, 2, 378, 0, 300]] | 1 |
| 19430 | 4 | 25 (25) | 50 | 500 | 0 | 0 | 0 | 93 (49) | 9 (52, 0) | 0 | 0 | 2/1/1 | 3/3/3 | [[1, 2, 497, 0, 300], [1, 3, 323, 0, 300], [1, 4, 368, 0, 300], [2, 3, 150, 0, 150], [2, 4, 150, 0, 150], [3, 4, 23, 0, 150]] | 17 |
| 19431 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |

## U7 totals (tally / ledger, log or chronicle)

- departed: 221 / 221
- delivered: 221 / 221
- turned_back: 0 / 0
- quarrels: 369 / 369
- estranged: 22 / 22
- reconciled: 13 / 13
- trip_records: 221 / 221
- withdrawn: 2191 / 2191
- delivered_units: 2191 / 2191
- returned_units: 0 / 0
- lost_units: 0 / 0
- disputes_logged: 3199 / 3199
- costed_plus_uncosted: 3199 / 3199
- quarrels_logged: 369 / 369
- pair_reconciliations: 13 / 13
- pair_estrangements: 22 / 22
- units in flight at day 120: 0
- worlds whose relations are exactly the living pairs with values in range: 32 of 32

U5 (identity) and U6 (performance) are judged outside this script.
