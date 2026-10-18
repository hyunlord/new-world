# Pass 21 diagnostic (7.1)

Per run: seeds and days of the probe, multi-village worlds (≥ 2 villages alive at once in that run), disputes per pair-day before and while estranged (lambda, from the measurement-only dispute log; a pair-day is estranged when the pair was estranged after that day's 08:00 decision), quarrels, estrangements and reconciliations, aid / exchange conditions in reach, trips and daughter deaths. The model columns replay the planner's model (brief 3.3) at each pair's measured lambda with the run's rule.

| run | multi-village | delivering share | dispute share | quarrel share | lambda before | lambda while | contact while/before | quarrel share of disputes | estrangements | reconciled | estranged at d120 | quarrels while estranged | uncosted disputes | capped quarrels | aid-food pair-days (not estranged) | exchange pair-days (not estranged) | trips (delivered) | exchange porters | daughter deaths | daughters alive d118 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| game_v25 (game_v25) | 7 of 16 | 0.714 | 0.714 | 0.714 | 0.088 (1036 pair-days) | 2.410 (166) | 27.433 (estranged pairs: 3.400) | 0.291 | 4 | 0 (0.000) | 4 of 20 (0.200) | 119 of 146 (0.815) | 0 of 498 | 0 | 36 (15) | 0 (0) | 27 (19) | 0 | 15 | 13 |
| game_v26 (game_v26) | 7 of 16 | 0.571 | 0.714 | 0.714 | 0.561 (1140 pair-days) | 0.820 (61) | 1.460 (estranged pairs: 0.135) | 0.112 | 4 | 3 (0.750) | 1 of 20 (0.050) | 10 of 81 (0.123) | 408 of 700 | 400 | 10 (10) | 0 (0) | 26 (14) | 0 | 15 | 13 |

## game_v25 (game_v25): worlds and pairs

Lambda before estrangement by pair (≥ 3 pair-days, sorted): [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.31, 0.33, 0.88, 1.12, 1.89]

| seed | villages | pair | base | pair-days (before / while) | disputes before / while | lambda before / while | quarrels (while estranged) | estranged at (day) | reconciled at (day) | estranged at end | relation every 10 days |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 19500 | 1 | – | | | | | | | | | |
| 19501 | 3 | 1–2 | 300 | 87 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 300 340 340 340 380 380 |
| 19501 | 3 | 1–3 | 300 | 55 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 300 300 300 |
| 19501 | 3 | 2–3 | 150 | 55 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 150 150 150 150 150 |
| 19502 | 1 | – | | | | | | | | | |
| 19503 | 1 | – | | | | | | | | | |
| 19504 | 1 | – | | | | | | | | | |
| 19505 | 1 | – | | | | | | | | | |
| 19506 | 1 | – | | | | | | | | | |
| 19507 | 1 | – | | | | | | | | | |
| 19508 | 2 | 1–2 | 300 | 77 / 7 | 24 / 10 | 0.31 / 1.43 | 11 (4) | [113] | [] | True | 300 189 189 -48 -128 -98 -200 -178 |
| 19509 | 3 | 1–2 | 300 | 18 / 72 | 34 / 64 | 1.89 / 0.89 | 23 (17) | [48] | [] | True | 300 270 -753 -983 -875 -851 -975 -915 -985 |
| 19509 | 3 | 1–3 | 0 | 60 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 0 0 0 40 40 40 |
| 19509 | 3 | 2–3 | 300 | 60 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 300 300 300 300 |
| 19510 | 3 | 1–2 | 300 | 88 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 300 380 380 380 380 380 |
| 19510 | 3 | 1–3 | 300 | 16 / 40 | 14 / 254 | 0.88 / 6.35 | 81 (76) | [80] | [] | True | 340 -374 -987 -973 -987 |
| 19510 | 3 | 2–3 | 150 | 56 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 150 150 150 150 150 |
| 19511 | 4 | 1–2 | 300 | 91 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 300 340 380 380 455 445 435 |
| 19511 | 4 | 1–3 | 300 | 16 / 47 | 18 / 72 | 1.12 / 1.53 | 28 (22) | [73] | [] | True | 154 6 -568 -973 -983 -984 |
| 19511 | 4 | 1–4 | 0 | 29 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 0 0 |
| 19511 | 4 | 2–3 | 150 | 63 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 150 150 150 150 150 150 |
| 19511 | 4 | 2–4 | 0 | 29 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 0 0 |
| 19511 | 4 | 3–4 | 300 | 29 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 |
| 19512 | 3 | 1–2 | 300 | 88 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 340 340 380 380 380 380 380 |
| 19512 | 3 | 1–3 | 300 | 58 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 300 300 300 |
| 19512 | 3 | 2–3 | 150 | 58 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 150 150 150 150 150 |
| 19513 | 1 | – | | | | | | | | | |
| 19514 | 2 | 1–2 | 300 | 3 / 0 | 1 / 0 | 0.33 / n/a | 3 (0) | [] | [] | False | – |
| 19515 | 1 | – | | | | | | | | | |

Model check (19 pairs with ≥ 10 pair-days; pairs without a dispute and without an estrangement are omitted from the table, the model predicts nothing for them either): estrangements per pair model 0.195 vs measured 0.211; estranged at the end model 0.195 vs measured 0.211.

| seed | pair | lambda before | contact while estranged (model input) | pair-days | estrangements measured / model | reconciled measured / model share | estranged at end measured / model |
|---|---|---|---|---|---|---|---|
| 19508 | 1–2 | 0.31 | 4.58 | 84 | 1 / 0.70 | 0 / 0.00 | 1 / 0.70 |
| 19509 | 1–2 | 1.89 | 0.47 | 90 | 1 / 1.00 | 0 / 0.00 | 1 / 1.00 |
| 19510 | 1–3 | 0.88 | 7.26 | 56 | 1 / 1.00 | 0 / 0.00 | 1 / 1.00 |
| 19511 | 1–3 | 1.12 | 1.36 | 63 | 1 / 1.00 | 0 / 0.00 | 1 / 1.00 |

## game_v26 (game_v26): worlds and pairs

Lambda before estrangement by pair (≥ 3 pair-days, sorted): [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.35, 1.51, 6.0, 6.11]

| seed | villages | pair | base | pair-days (before / while) | disputes before / while | lambda before / while | quarrels (while estranged) | estranged at (day) | reconciled at (day) | estranged at end | relation every 10 days |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 19500 | 1 | – | | | | | | | | | |
| 19501 | 3 | 1–2 | 300 | 87 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 300 334 333 333 365 355 |
| 19501 | 3 | 1–3 | 300 | 55 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 300 300 300 |
| 19501 | 3 | 2–3 | 150 | 55 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 150 150 150 150 150 |
| 19502 | 1 | – | | | | | | | | | |
| 19503 | 1 | – | | | | | | | | | |
| 19504 | 1 | – | | | | | | | | | |
| 19505 | 1 | – | | | | | | | | | |
| 19506 | 1 | – | | | | | | | | | |
| 19507 | 1 | – | | | | | | | | | |
| 19508 | 2 | 1–2 | 300 | 84 / 0 | 29 / 0 | 0.35 / n/a | 9 (0) | [] | [] | False | 300 259 264 178 169 194 168 188 |
| 19509 | 3 | 1–2 | 300 | 56 / 34 | 342 / 8 | 6.11 / 0.24 | 27 (1) | [62, 83, 108] | [73, 95, 119] | False | 300 285 103 -240 -188 -216 -210 -111 -287 |
| 19509 | 3 | 1–3 | 0 | 58 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 0 0 0 0 0 |
| 19509 | 3 | 2–3 | 300 | 58 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 300 300 300 |
| 19510 | 3 | 1–2 | 300 | 88 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 300 369 357 347 337 333 |
| 19510 | 3 | 1–3 | 300 | 29 / 27 | 174 / 42 | 6.00 / 1.56 | 25 (10) | [93] | [] | True | 338 215 -197 -384 -306 |
| 19510 | 3 | 2–3 | 150 | 56 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 150 150 150 150 150 |
| 19511 | 4 | 1–2 | 300 | 91 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 300 336 333 333 333 333 333 |
| 19511 | 4 | 1–3 | 300 | 63 / 0 | 95 / 0 | 1.51 / n/a | 19 (0) | [] | [] | False | 249 156 61 14 -90 -51 |
| 19511 | 4 | 1–4 | 0 | 30 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 0 0 0 |
| 19511 | 4 | 2–3 | 150 | 63 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 150 150 150 150 150 150 |
| 19511 | 4 | 2–4 | 0 | 30 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 0 0 0 |
| 19511 | 4 | 3–4 | 300 | 30 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 300 |
| 19512 | 3 | 1–2 | 300 | 88 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 333 333 360 350 340 333 333 |
| 19512 | 3 | 1–3 | 300 | 58 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 300 300 300 337 333 |
| 19512 | 3 | 2–3 | 150 | 58 / 0 | 0 / 0 | 0.00 / n/a | 0 (0) | [] | [] | False | 150 150 150 150 150 |
| 19513 | 1 | – | | | | | | | | | |
| 19514 | 2 | 1–2 | 300 | 3 / 0 | 0 / 0 | 0.00 / n/a | 1 (0) | [] | [] | False | – |
| 19515 | 1 | – | | | | | | | | | |

Model check (19 pairs with ≥ 10 pair-days; pairs without a dispute and without an estrangement are omitted from the table, the model predicts nothing for them either): estrangements per pair model 0.232 vs measured 0.211; estranged at the end model 0.075 vs measured 0.053.

| seed | pair | lambda before | contact while estranged (model input) | pair-days | estrangements measured / model | reconciled measured / model share | estranged at end measured / model |
|---|---|---|---|---|---|---|---|
| 19508 | 1–2 | 0.35 | 0.14 | 84 | 0 / 0.00 | 0 / n/a | 0 / 0.00 |
| 19509 | 1–2 | 6.11 | 0.04 | 90 | 3 / 3.34 | 3 / 0.83 | 0 / 0.56 |
| 19510 | 1–3 | 6.00 | 0.26 | 56 | 1 / 1.07 | 0 / 0.19 | 1 / 0.87 |
| 19511 | 1–3 | 1.51 | 0.14 | 63 | 0 / 0.00 | 0 / 1.00 | 0 / 0.00 |
