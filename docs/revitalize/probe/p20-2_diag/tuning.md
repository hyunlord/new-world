# Pass 20 tuning round (7.3): game_v25 with aid_giver_days_permille 3000, aid_receiver_days_permille 2000, 16 worlds, 60 days

Pair-days are ordered (sender, receiver) pairs at the 08:00 decision. Conditions are counted before reach, estrangement and porter slots; `in reach` = within three quarters of the 08:00–15:00 window.

| seed | villages (max alive) | pair-days | in reach | aid food | aid wood | exchange | sent | porters | delivered | turned back | disputes | quarrels | estranged | reconciled | relation (first, min, max, last) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 19500 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | – |
| 19501 | 2 | 54 | 54 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1-2: [300, 300, 300, 300] |
| 19502 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | – |
| 19503 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | – |
| 19504 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | – |
| 19505 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | – |
| 19506 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | – |
| 19507 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | – |
| 19508 | 2 | 48 | 48 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 1 | 0 | 0 | 1-2: [300, 184, 300, 188] |
| 19509 | 3 | 60 | 60 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 55 | 9 | 1 | 0 | 1-2: [300, -987, 300, -975] |
| 19510 | 2 | 56 | 56 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1-2: [300, 300, 300, 300] |
| 19511 | 3 | 74 | 68 | 1 | 0 | 0 | 1 | 1 | 1 | 0 | 5 | 1 | 0 | 0 | 1-2: [300, 300, 340, 340]; 1-3: [151, 151, 153, 153]; 2-3: [150, 150, 150, 150] |
| 19512 | 2 | 56 | 56 | 0 | 1 | 0 | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 1-2: [300, 300, 340, 340] |
| 19513 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | – |
| 19514 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | – |
| 19515 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | – |

Totals: pair_days 348, in_reach 342, aid_food 5, aid_wood 1, exchange 0, aid_food_in_reach 5, aid_wood_in_reach 1, exchange_in_reach 0, sent 2, trips 2, porters 2, deliveries 2, turnbacks 0, disputes 63, quarrels 11, estranged 1, reconciled 0

Measures over all pair-days (sender side; quantiles 0/10/50/90/100 %): food days (permille) 19501: [2237, 2528, 4005, 6233, 6608]; 19508: [3151, 3296, 3865, 6168, 6600]; 19509: [1444, 2616, 3831, 5416, 6122]; 19510: [1765, 2553, 3793, 6012, 6286]; 19511: [1662, 2684, 3859, 6271, 6759]; 19512: [2247, 2565, 3852, 6024, 6388] — wood nights 19501: [0, 0, 3, 7, 8]; 19508: [0, 0, 1, 7, 8]; 19509: [0, 0, 1, 7, 8]; 19510: [0, 0, 1, 6, 7]; 19511: [0, 0, 1, 7, 8]; 19512: [0, 0, 1, 6, 7]

Multi-village worlds: 6 of 16. Share with ≥ 1 delivery 0.3333333333333333, with ≥ 1 dispute 0.5, with ≥ 1 quarrel 0.5. T1 within reach: False; T3 within reach: True.

