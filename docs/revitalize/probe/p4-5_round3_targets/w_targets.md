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
| W5 | stagnation | FAIL | longest run of days 2..N with members <= sustainable-3 and no birth <= 4 in every seed | {"max": 8.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| W6 | old_age_by_day30 | PASS | >= 1 old-age death by day 30 in every seed | {"max": 13.0, "median": 9.0, "min": 5.0, "n": 32, "p10": 6.0, "p90": 12.0} |
| W6 | replacement | PASS | births - deaths over days 30..120 >= -3 in every seed | {"max": 1.0, "median": 0.0, "min": -3.0, "n": 32, "p10": -1.0, "p90": 0.0} |
| W6 | alive_band | PASS | every day sample from day 30 has alive in [32, 44] | {"max": 42.0, "median": 42.0, "min": 35.0, "n": 2912, "p10": 41.0, "p90": 42.0} |
| W6 | child_share | PASS | day-120 child share median in [0.03, 0.20] | {"max": 0.1951219512195122, "median": 0.09523809523809523, "min": 0.023809523809523808, "n": 32, "p10": 0.047619047619047616, "p90": 0.17032520325203254} |
| W6 | elder_share | PASS | day-120 elder share median in [0.08, 0.35] | {"max": 0.38095238095238093, "median": 0.2619047619047619, "min": 0.17073170731707318, "n": 32, "p10": 0.21428571428571427, "p90": 0.3406504065040651} |
| W7 | autumn_store | PASS | median autumn-closing stock >= median 0.8 x days_per_season x max(0, need - winter supply) | {"paired_margin": {"max": 542.0959919691086, "median": 415.09999349117277, "min": 58.847992730140675, "n": 128, "p10": 269.15839193105694, "p90": 502.3351926159859}, "required": {"max": 161.2800084114075, "median": 150.52800765037537, "min": 118.27200536727906, "n": 128, "p10": 139.77600688934328, "p90": 161.2800084114075}, "stock": {"max": 699.0, "median": 562.0, "min": 204.0, "n": 128, "p10": 425.6, "p90": 653.3}} |
| W7 | hoard_cap | PASS | max stock - 3 x max target <= 0 in every seed | {"max": -351.40003830194473, "median": -481.90003830194473, "min": -733.4000383019447, "n": 32, "p10": -583.3000383019447, "p90": -378.8000383019447} |
| W8 | noon_idle | FAIL | noon idle share median <= 0.45 over spring-autumn days (all days without seasons) | {"max": 0.9285714285714286, "median": 0.81265664160401, "min": 0.7380952380952381, "n": 32, "p10": 0.7404761904761905, "p90": 0.8809523809523809} |
| W8 | forage_trips | FAIL | forage trips per day median >= 10 in summer and autumn (all days without seasons) | {"max": 74.0, "median": 0.0, "min": 0.0, "n": 1856, "p10": 0.0, "p90": 63.0} |
| W9 | night_in_shelter | PASS | median >= 0.70 | {"max": 1.0, "median": 1.0, "min": 0.9047619047619048, "n": 32, "p10": 0.951335656213705, "p90": 1.0} |
| W9 | evening_near_hearth | FAIL | median >= 0.50 and every seed >= 0.30 | {"max": 0.8181818181818182, "median": 0.5706845238095238, "min": 0.28125, "n": 32, "p10": 0.32510518934081345, "p90": 0.7764705882352941} |
| W10 | home_distance | FAIL | day-10 median <= 7.0, every seed <= 10 | {"max": 10.928571428571429, "median": 5.05952380952381, "min": 2.7857142857142856, "n": 64, "p10": 3.5047619047619047, "p90": 6.771428571428571} |
| W10 | t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.0024764735017335313, "median": 0.0007429420505200594, "min": 0.0, "p10": 0.00024764735017335313, "p90": 0.0014858841010401188} |
| W10 | social_10 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.7025673605142195, "median": 0.6547329870324002, "min": 0.6009193168653537, "n": 64, "p10": 0.632595912373785, "p90": 0.6858253297955827} |
| W10 | t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.011042179439630467, "median": 0.008313570095143283, "min": 0.0064200550290431065, "p10": 0.006890090802775391, "p90": 0.009917963972489296} |
| W10 | social_120 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.8657687892980516, "median": 0.8338730763714787, "min": 0.8071109212667242, "n": 32, "p10": 0.8157575844989603, "p90": 0.856309092793381} |
| W11 | day10 | PASS | median ms/agent-tick <= 1.5 x same-host Pass 3 game_v9 | {"limit": 0.0011675054515673492, "max": 0.001040486858004156, "median": 0.000988439733672161, "min": 0.0009265551154128827, "n": 64, "p10": 0.0009607127101577443, "p90": 0.0010220321347472953} |
| W11 | day120 | PASS | median ms/agent-tick <= 1.5 x same-host Pass 3 game_v9 | {"limit": 0.0013239339685035822, "max": 0.0015181409813340941, "median": 0.0011974037792575785, "min": 0.0011373992368475003, "n": 32, "p10": 0.001147456446358008, "p90": 0.0014665332970911218} |
