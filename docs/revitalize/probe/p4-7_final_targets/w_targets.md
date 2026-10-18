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
| W5 | stagnation | PASS | longest run of days 2..N with members <= sustainable-3 and no birth <= 4 in every seed | {"max": 2.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| W6 | old_age_by_day30 | PASS | >= 1 old-age death by day 30 in every seed | {"max": 13.0, "median": 9.0, "min": 5.0, "n": 32, "p10": 6.0, "p90": 12.0} |
| W6 | replacement | PASS | births - deaths over days 30..120 >= -3 in every seed | {"max": 1.0, "median": 0.0, "min": -3.0, "n": 32, "p10": -1.0, "p90": 0.0} |
| W6 | alive_band | PASS | every day sample from day 30 has alive in [32, 44] | {"max": 42.0, "median": 42.0, "min": 37.0, "n": 2912, "p10": 41.0, "p90": 42.0} |
| W6 | child_share | PASS | day-120 child share median in [0.03, 0.20] | {"max": 0.1951219512195122, "median": 0.09523809523809523, "min": 0.023809523809523808, "n": 32, "p10": 0.047619047619047616, "p90": 0.17032520325203254} |
| W6 | elder_share | PASS | day-120 elder share median in [0.08, 0.35] | {"max": 0.38095238095238093, "median": 0.2619047619047619, "min": 0.17073170731707318, "n": 32, "p10": 0.21428571428571427, "p90": 0.3333333333333333} |
| W7 | autumn_store | PASS | median autumn-closing stock >= median 0.8 x days_per_season x max(0, need - winter supply) | {"paired_margin": {"max": 539.0959919691086, "median": 435.47199234962466, "min": 172.47199234962463, "n": 128, "p10": 305.47199234962466, "p90": 511.546392121315}, "required": {"max": 161.2800084114075, "median": 150.52800765037537, "min": 118.27200536727906, "n": 128, "p10": 139.77600688934328, "p90": 161.2800084114075}, "stock": {"max": 695.0, "median": 581.0, "min": 323.0, "n": 128, "p10": 453.3, "p90": 668.2}} |
| W7 | hoard_cap | PASS | max stock - 3 x max target <= 0 in every seed | {"max": -359.40003830194473, "median": -455.40003830194473, "min": -650.4000383019447, "n": 32, "p10": -543.6000383019448, "p90": -374.40003830194473} |
| W8 | noon_idle | FAIL | noon idle share median <= 0.45 over spring-autumn days (all days without seasons) | {"max": 0.9285714285714286, "median": 0.8095238095238095, "min": 0.6904761904761905, "n": 32, "p10": 0.7392857142857143, "p90": 0.9023809523809524} |
| W8 | forage_trips | FAIL | forage trips per day median >= 10 in summer and autumn (all days without seasons) | {"max": 91.0, "median": 0.0, "min": 0.0, "n": 1824, "p10": 0.0, "p90": 74.0} |
| W9 | night_in_shelter | PASS | median >= 0.70 | {"max": 1.0, "median": 1.0, "min": 0.9285714285714286, "n": 32, "p10": 0.9547619047619047, "p90": 1.0} |
| W9 | evening_near_hearth | FAIL | median >= 0.50 and every seed >= 0.30 | {"max": 0.8125, "median": 0.5542114695340502, "min": 0.28125, "n": 32, "p10": 0.3479020979020979, "p90": 0.7664705882352941} |
| W10 | home_distance | FAIL | day-10 median <= 7.0, every seed <= 10 | {"max": 11.047619047619047, "median": 7.059523809523809, "min": 5.142857142857143, "n": 64, "p10": 5.60952380952381, "p90": 8.30952380952381} |
| W10 | t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.002228826151560178, "median": 0.0009905894006934125, "min": 0.0, "p10": 0.00024764735017335313, "p90": 0.0014858841010401188} |
| W10 | social_10 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.6995777121716058, "median": 0.6497988794778677, "min": 0.5919503718375126, "n": 64, "p10": 0.6190605621631492, "p90": 0.687619118801151} |
| W10 | t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.01070302887289521, "median": 0.008761626854127569, "min": 0.00649120736456981, "p10": 0.007297068196597303, "p90": 0.010346251636852594} |
| W10 | social_120 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.8631108642688742, "median": 0.8352795868609617, "min": 0.8040587363490594, "n": 32, "p10": 0.809812983626291, "p90": 0.8532382730457948} |
| W11 | day10 | PASS | median ms/agent-tick <= 1.5 x same-host Pass 3 game_v9 | {"limit": 0.0011702719737690642, "max": 0.0011365222604232895, "median": 0.000989339636730345, "min": 0.0009475426668493649, "n": 64, "p10": 0.0009604118589384244, "p90": 0.0010824883829324324} |
| W11 | day120 | PASS | median ms/agent-tick <= 1.5 x same-host Pass 3 game_v9 | {"limit": 0.0013351547707594372, "max": 0.001525288594164317, "median": 0.0012101778785801123, "min": 0.0011172803775054436, "n": 32, "p10": 0.001146923753129754, "p90": 0.001415879006219948} |
