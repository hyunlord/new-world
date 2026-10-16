# Pass 4 W targets

| Target | Clause | Verdict | Requirement | Observed |
|---|---|---|---|---|
| W1 | twins_10 | PASS | 64 seeds, determinism checked | 64 |
| W1 | twins_120 | PASS | 32 seeds, determinism checked | 32 |
| W1 | legacy_pin | PASS | legacy_pin verified externally | true |
| W1 | v8_repro | PASS | v8_repro verified externally | true |
| W1 | v9_repro | PASS | v9_repro verified externally | true |
| W2 | day_10 | PASS | median 0, max <= 1 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 0.0} |
| W2 | day_120 | PASS | total median 0, max <= 2 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| W3 | day_10 | PASS | zero in every seed | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 0.0} |
| W3 | day_120 | PASS | total median 0, max <= 3 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| W4 | every_day | PASS | one settlement at every day sample from day 1 | {"max": 1.0, "median": 1.0, "min": 1.0, "n": 4480, "p10": 1.0, "p90": 1.0} |
| W5 | birth_by_day4 | PASS | births >= 1 by the day-4 checkpoint in every seed (10-day round) | {"max": 7.0, "median": 7.0, "min": 7.0, "n": 64, "p10": 7.0, "p90": 7.0} |
| W5 | alive_day10 | PASS | day-10 alive >= 30 in every seed | {"max": 42.0, "median": 42.0, "min": 42.0, "n": 64, "p10": 42.0, "p90": 42.0} |
| W5 | stagnation | FAIL | longest run of days 2..N with members <= sustainable-3 and no birth <= 4 in every seed | {"max": 6.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| W6 | old_age_by_day30 | PASS | >= 1 old-age death by day 30 in every seed | {"max": 13.0, "median": 9.0, "min": 5.0, "n": 32, "p10": 6.0, "p90": 12.0} |
| W6 | replacement | FAIL | births - deaths over days 30..120 >= -3 in every seed | {"max": 1.0, "median": 0.0, "min": -6.0, "n": 32, "p10": -1.0, "p90": 0.0} |
| W6 | alive_band | PASS | every day sample from day 30 has alive in [32, 44] | {"max": 42.0, "median": 42.0, "min": 34.0, "n": 2912, "p10": 41.0, "p90": 42.0} |
| W6 | child_share | PASS | day-120 child share median in [0.03, 0.20] | {"max": 0.1951219512195122, "median": 0.09523809523809523, "min": 0.023809523809523808, "n": 32, "p10": 0.047619047619047616, "p90": 0.17032520325203254} |
| W6 | elder_share | PASS | day-120 elder share median in [0.08, 0.35] | {"max": 0.3333333333333333, "median": 0.23809523809523808, "min": 0.12195121951219512, "n": 32, "p10": 0.19047619047619047, "p90": 0.31631823461091757} |
| W7 | autumn_store | PASS | median autumn-closing stock >= median 0.8 x days_per_season x max(0, need - winter supply) | {"paired_margin": {"max": 534.7199915885925, "median": 488.47199234962466, "min": 78.47199234962463, "n": 128, "p10": 379.0455918169022, "p90": 516.8327917027473}, "required": {"max": 161.2800084114075, "median": 150.52800765037537, "min": 118.27200536727906, "n": 128, "p10": 139.77600688934328, "p90": 161.2800084114075}, "stock": {"max": 696.0, "median": 634.5, "min": 229.0, "n": 128, "p10": 537.1, "p90": 671.6}} |
| W7 | hoard_cap | PASS | max stock - 3 x max target <= 0 in every seed | {"max": -652.8000428080559, "median": -682.8000428080559, "min": -981.8000428080559, "n": 32, "p10": -791.3000428080559, "p90": -667.4000428080559} |
| W8 | noon_idle | FAIL | noon idle share median <= 0.45 over spring-autumn days (all days without seasons) | {"max": 0.9285714285714286, "median": 0.8333333333333334, "min": 0.7142857142857143, "n": 32, "p10": 0.7752777777777778, "p90": 0.8809523809523809} |
| W8 | forage_trips | FAIL | forage trips per day median >= 10 in summer and autumn (all days without seasons) | {"max": 108.0, "median": 0.0, "min": 0.0, "n": 1856, "p10": 0.0, "p90": 90.0} |
| W9 | night_in_shelter | PASS | median >= 0.70 | {"max": 1.0, "median": 1.0, "min": 0.9047619047619048, "n": 32, "p10": 0.9523809523809523, "p90": 1.0} |
| W9 | evening_near_hearth | FAIL | median >= 0.50 and every seed >= 0.30 | {"max": 0.8333333333333334, "median": 0.5638586956521738, "min": 0.2571428571428571, "n": 32, "p10": 0.35944055944055947, "p90": 0.7985714285714286} |
| W10 | home_distance | FAIL | day-10 median <= 7.0, every seed <= 10 | {"max": 11.119047619047619, "median": 5.273809523809524, "min": 2.380952380952381, "n": 64, "p10": 3.585714285714286, "p90": 7.376190476190477} |
| W10 | t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.002238249191743347, "median": 0.0007429420505200594, "min": 0.0, "p10": 0.00024764735017335313, "p90": 0.0014858841010401188} |
| W10 | social_10 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.7205052505699017, "median": 0.6457640420045592, "min": 0.5620538884113756, "n": 64, "p10": 0.6098882618931948, "p90": 0.6738537966623439} |
| W10 | t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.010823021679759938, "median": 0.00861180556522696, "min": 0.006507848195168007, "p10": 0.007164743224943397, "p90": 0.009807860654549935} |
| W10 | social_120 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.8596695965408148, "median": 0.8288936905968275, "min": 0.7937439300744578, "n": 32, "p10": 0.8018894771278392, "p90": 0.8486407180723375} |
| W11 | day10 | PASS | median ms/agent-tick <= 1.5 x same-host Pass 3 game_v9 | {"limit": 0.0011675054515673492, "max": 0.0010815967234284625, "median": 0.0009614779336962399, "min": 0.0009105460675743542, "n": 64, "p10": 0.0009277931175803756, "p90": 0.001007898896321484} |
| W11 | day120 | PASS | median ms/agent-tick <= 1.5 x same-host Pass 3 game_v9 | {"limit": 0.0013239339685035822, "max": 0.0014816488826367943, "median": 0.0012053934275281016, "min": 0.0011062691246008034, "n": 32, "p10": 0.0011169367528761605, "p90": 0.0014221078226841532} |
