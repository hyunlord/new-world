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
| W6 | replacement | PASS | births - deaths over days 30..120 >= -3 in every seed | {"max": 1.0, "median": 0.0, "min": -3.0, "n": 32, "p10": -1.0, "p90": 0.0} |
| W6 | alive_band | PASS | every day sample from day 30 has alive in [32, 44] | {"max": 42.0, "median": 42.0, "min": 37.0, "n": 2912, "p10": 41.0, "p90": 42.0} |
| W6 | child_share | PASS | day-120 child share median in [0.03, 0.20] | {"max": 0.1951219512195122, "median": 0.09523809523809523, "min": 0.023809523809523808, "n": 32, "p10": 0.047619047619047616, "p90": 0.17032520325203254} |
| W6 | elder_share | PASS | day-120 elder share median in [0.08, 0.35] | {"max": 0.3333333333333333, "median": 0.23809523809523808, "min": 0.12195121951219512, "n": 32, "p10": 0.19047619047619047, "p90": 0.31631823461091757} |
| W7 | autumn_store | PASS | median autumn-closing stock >= median 0.8 x days_per_season x max(0, need - winter supply) | {"paired_margin": {"max": 462.0959919691086, "median": 422.15999253988264, "min": 97.47199234962463, "n": 128, "p10": 362.98319185495376, "p90": 444.84639212131503}, "required": {"max": 161.2800084114075, "median": 150.52800765037537, "min": 118.27200536727906, "n": 128, "p10": 139.77600688934328, "p90": 161.2800084114075}, "stock": {"max": 619.0, "median": 572.0, "min": 248.0, "n": 128, "p10": 500.7, "p90": 600.0}} |
| W7 | hoard_cap | PASS | max stock - 3 x max target <= 0 in every seed | {"max": -580.6000405550003, "median": -603.6000405550003, "min": -849.6000405550003, "n": 32, "p10": -656.1000405550003, "p90": -588.7000405550003} |
| W8 | noon_idle | FAIL | noon idle share median <= 0.45 over spring-autumn days (all days without seasons) | {"max": 0.9285714285714286, "median": 0.8214285714285714, "min": 0.7142857142857143, "n": 32, "p10": 0.7637630662020906, "p90": 0.8809523809523809} |
| W8 | forage_trips | FAIL | forage trips per day median >= 10 in summer and autumn (all days without seasons) | {"max": 131.0, "median": 0.0, "min": 0.0, "n": 1856, "p10": 0.0, "p90": 113.0} |
| W9 | night_in_shelter | PASS | median >= 0.70 | {"max": 1.0, "median": 1.0, "min": 0.9047619047619048, "n": 32, "p10": 0.9539170506912442, "p90": 1.0} |
| W9 | evening_near_hearth | FAIL | median >= 0.50 and every seed >= 0.30 | {"max": 0.84375, "median": 0.53, "min": 0.2571428571428571, "n": 32, "p10": 0.3342857142857143, "p90": 0.7640973630831642} |
| W10 | home_distance | FAIL | day-10 median <= 7.0, every seed <= 10 | {"max": 10.880952380952381, "median": 5.25, "min": 2.761904761904762, "n": 64, "p10": 3.4285714285714284, "p90": 7.5738095238095235} |
| W10 | t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.0037462537462537465, "median": 0.0007429420505200594, "min": 0.0, "p10": 0.00032194155522535925, "p90": 0.0016592372461614667} |
| W10 | social_10 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.7205052505699017, "median": 0.6442692178332523, "min": 0.5351470533278523, "n": 64, "p10": 0.602713105870922, "p90": 0.6867222242983669} |
| W10 | t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.01180789335679123, "median": 0.008495209944846288, "min": 0.006828182691821355, "p10": 0.007167542075256083, "p90": 0.010222959734335613} |
| W10 | social_120 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.86169426964991, "median": 0.8296278430791737, "min": 0.7814057007005741, "n": 32, "p10": 0.8036870598561234, "p90": 0.8516000620553429} |
| W11 | day10 | PASS | median ms/agent-tick <= 1.5 x same-host Pass 3 game_v9 | {"limit": 0.0011675054515673492, "max": 0.0012346011622257928, "median": 0.0009948113602665414, "min": 0.0009451751996213082, "n": 64, "p10": 0.0009612375780111347, "p90": 0.0011319673818985048} |
| W11 | day120 | PASS | median ms/agent-tick <= 1.5 x same-host Pass 3 game_v9 | {"limit": 0.0013239339685035822, "max": 0.001590994538650776, "median": 0.0012677845586151221, "min": 0.0011501856100402956, "n": 32, "p10": 0.0011704505432641639, "p90": 0.0015222851572230052} |
