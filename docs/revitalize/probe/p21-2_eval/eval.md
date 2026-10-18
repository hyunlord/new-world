# Pass 21 evaluation: game_v26 vs game_v25 and game_v24

Seeds 19600–19631 (32 worlds), 120 days, paired by seed; Pass 17's definitions and bootstrap (2,000 world resamples, 95 % percentile interval). Rules registered in METRICS.md `## Pass 21`. Multi-village world: ≥ 2 settlements alive at once in game_v26.

| # | rule | value | verdict |
|---|---|---|---|
| U1 | ≥ 0.6 of multi-village worlds deliver; failed-trip share ≤ 0.1 | 0.684 (13 of 19); failed 0.000 (0/170 porters) | PASS |
| U2 | ≥ 1 exchange trip over the 32 worlds | 0 exchange trips (0 porters); exchange pair-days 0 (0 in reach, not estranged) | FAIL |
| U3 | disputes ≥ 0.3, quarrels ≥ 0.15 of multi-village worlds; estranged at d120 ≤ 0.25; quarrels while estranged ≤ 0.2; ≥ 3 estrangements → ≥ 0.5 reconciled | dispute 0.579, quarrel 0.526, estranged 13 of 45 (0.289), while estranged 51 of 322 (0.158), reconciled 8 of 21 (0.381) | FAIL |
| U4 | R2-alive: relative change within +/-0.10 (v26 vs v24) | 63.844 vs 63.875 (stat -0.000 [-0.016, 0.015]) | PASS |
| U4 | R2-deaths: non-old-age deaths difference <= +1.5/world (v26 vs v24) | 1.375 vs 1.094 (stat 0.281 [0.031, 0.562]) | PASS |
| U4 | R2-shelters: completed shelters ratio >= 0.90 (v26 vs v24) | 7.094 vs 7.031 (stat 1.009 [0.988, 1.042]) | PASS |
| U4 | R2-food: noon food relative change within +/-0.15 (v26 vs v24) | 486.387 vs 491.674 (stat -0.011 [-0.020, -0.002]) | PASS |
| U4 | R2-wood: noon wood relative change within +/-0.15 (v26 vs v24) | 120.053 vs 120.232 (stat -0.001 [-0.005, 0.001]) | PASS |
| U4 | R2-social: pooled adult social rate ratio >= 0.90 (v26 vs v24) | 1.731 vs 1.729 (stat 1.001 [0.999, 1.003]) | PASS |
| U4 | F1 within ± 0.05 of game_v24 | 0.671 [0.638, 0.701] vs 0.681 [0.647, 0.712] (diff -0.0102) | PASS |
| U4 | F5 within ± 0.05 of game_v24 | 0.000 [0.000, 0.000] vs 0.000 [0.000, 0.000] (diff 0.0000) | PASS |
| U4 | daughter non-old-age deaths per multi-village world v26 − v24 ≤ 0 | 0.158 [-0.158, 0.474] (12 vs 9) | FAIL |
| U4 | the same against game_v25 | -0.105 [-0.737, 0.368] (12 vs 14) | PASS |
| U4 | daughters alive at day 118 ≥ game_v24 | 30 vs 29 (game_v25 28) | PASS |
| U7 | Pass 20's T7 on game_v26 + reconciliations = chronicle + costed + uncosted = disputes | mismatched worlds 0, unbalanced trips 0, village changes 0 | PASS (C1–C4: chronicle.md) |

U4 overall: FAIL.

## Porters (U1, U2)

Trips 89 (89 delivered), porters 170, food moved 1510, wood 157, turned back 0, loads lost 0; departure causes {0: 151, 1: 19} (0 aid food, 1 aid wood, 2 exchange food, 3 exchange wood). In-reach food-aid pair-days: 87 open, 67 between estranged villages. game_v25 delivering worlds on these seeds: 11 of 19.

## Conflict (U3)

Disputes 2615 (1347 past the daily dispute cap), quarrels 322 (1402 disputes met the quarrel cap), estrangements 21, reconciliations 8, pair-days estranged 311. game_v25 on these seeds: disputes 2815, quarrels 864, estrangements 14, reconciliations 0.

| seed | villages | trips (delivered) | porters | food | wood | exchange trips | failed | disputes (uncosted) | quarrels (capped, while estranged) | estranged | reconciled | daughter deaths v26/v25/v24 | daughters alive d118 v26/v25/v24 | pairs at end [a, b, value, estranged, base] | v25 trips delivered |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 19600 | 3 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 216 (129) | 19 (119, 4) | 1 | 0 | 0/0/0 | 2/2/2 | [[1, 2, -185, 1, 300], [1, 3, -15, 0, 0], [2, 3, 300, 0, 300]] | 0 |
| 19601 | 2 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 1/1/1 | [[1, 2, 300, 0, 300]] | 0 |
| 19602 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19603 | 4 | 9 (9) | 18 | 120 | 57 | 0 | 0 | 488 (248) | 63 (239, 13) | 5 | 2 | 1/0/0 | 3/3/3 | [[1, 2, -179, 1, 300], [1, 3, -268, 1, 300], [1, 4, -294, 1, 0], [2, 3, 187, 0, 150], [2, 4, 33, 0, 0], [3, 4, 300, 0, 300]] | 3 |
| 19604 | 2 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 143 (79) | 16 (64, 2) | 1 | 0 | 0/0/0 | 1/1/1 | [[1, 2, -409, 1, 300]] | 0 |
| 19605 | 2 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 168 (27) | 35 (53, 20) | 1 | 0 | 0/5/1 | 1/1/1 | [[1, 2, -401, 1, 300]] | 0 |
| 19606 | 4 | 1 (1) | 1 | 0 | 10 | 0 | 0 | 257 (105) | 37 (126, 7) | 4 | 2 | 1/0/0 | 3/3/3 | [[1, 2, 300, 0, 300], [1, 3, -419, 1, 0], [1, 4, 300, 0, 300], [2, 3, -235, 1, 300], [2, 4, 150, 0, 150], [3, 4, 0, 0, 0]] | 2 |
| 19607 | 3 | 16 (16) | 30 | 300 | 0 | 0 | 0 | 469 (317) | 40 (338, 3) | 2 | 1 | 0/0/0 | 2/1/2 | [[1, 2, -325, 1, 300], [1, 3, 79, 0, 0], [2, 3, 300, 0, 300]] | 0 |
| 19608 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19609 | 2 | 1 (1) | 2 | 0 | 15 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 1/1/1 | [[1, 2, 333, 0, 300]] | 1 |
| 19610 | 2 | 18 (18) | 34 | 340 | 0 | 0 | 0 | 237 (90) | 36 (107, 0) | 2 | 1 | 0/1/1 | 1/1/1 | [[1, 2, -218, 1, 300]] | 5 |
| 19611 | 2 | 2 (2) | 4 | 20 | 16 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 1/1/1 | [[1, 2, 362, 0, 300]] | 1 |
| 19612 | 2 | 2 (2) | 4 | 20 | 15 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 1/1/1 | [[1, 2, 334, 0, 300]] | 1 |
| 19613 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19614 | 2 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 1/1/1 | [[1, 2, 300, 0, 300]] | 0 |
| 19615 | 2 | 1 (1) | 2 | 0 | 17 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 6/6/5 | 1/1/1 | [[1, 2, 333, 0, 300]] | 1 |
| 19616 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19617 | 2 | 1 (1) | 2 | 0 | 13 | 0 | 0 | 158 (73) | 22 (65, 2) | 1 | 0 | 0/0/0 | 1/1/1 | [[1, 2, -341, 1, 300]] | 0 |
| 19618 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19619 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19620 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19621 | 4 | 9 (9) | 18 | 180 | 0 | 0 | 0 | 269 (178) | 26 (182, 0) | 1 | 1 | 1/1/1 | 3/2/2 | [[1, 2, 409, 0, 300], [1, 3, -337, 0, 300], [1, 4, 0, 0, 0], [2, 3, 150, 0, 150], [2, 4, 0, 0, 0], [3, 4, 300, 0, 300]] | 8 |
| 19622 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19623 | 4 | 26 (26) | 49 | 490 | 0 | 0 | 0 | 207 (101) | 28 (109, 0) | 3 | 1 | 2/0/0 | 3/3/3 | [[1, 2, 629, 0, 300], [1, 3, -284, 1, 0], [1, 4, 325, 0, 300], [2, 3, -305, 1, 300], [2, 4, 150, 0, 150], [3, 4, 0, 0, 0]] | 26 |
| 19624 | 3 | 2 (2) | 4 | 40 | 0 | 0 | 0 | 3 (0) | 0 (0, 0) | 0 | 0 | 1/1/1 | 2/2/2 | [[1, 2, 361, 0, 300], [1, 3, 285, 0, 300], [2, 3, 150, 0, 150]] | 5 |
| 19625 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19626 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19627 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19628 | 2 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 1/1/1 | [[1, 2, 300, 0, 300]] | 0 |
| 19629 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19630 | 1 | 0 (0) | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 0/0/0 | [] | 0 |
| 19631 | 2 | 1 (1) | 2 | 0 | 14 | 0 | 0 | 0 (0) | 0 (0, 0) | 0 | 0 | 0/0/0 | 1/1/1 | [[1, 2, 333, 0, 300]] | 1 |

## U7 totals (tally / ledger, log or chronicle)

- departed: 170 / 170
- delivered: 170 / 170
- turned_back: 0 / 0
- quarrels: 322 / 322
- estranged: 21 / 21
- reconciled: 8 / 8
- trip_records: 170 / 170
- withdrawn: 1667 / 1667
- delivered_units: 1667 / 1667
- returned_units: 0 / 0
- lost_units: 0 / 0
- disputes_logged: 2615 / 2615
- costed_plus_uncosted: 2615 / 2615
- quarrels_logged: 322 / 322
- pair_reconciliations: 8 / 8
- pair_estrangements: 21 / 21
- units in flight at day 120: 0
- worlds whose relations are exactly the living pairs with values in range: 32 of 32

U5 (identity) and U6 (performance) are judged outside this script.
