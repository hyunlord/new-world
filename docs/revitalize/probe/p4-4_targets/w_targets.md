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
| W5 | stagnation | PASS | longest run of days 2..N with members <= sustainable-3 and no birth <= 4 in every seed | {"max": 1.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| W6 | old_age_by_day30 | PASS | >= 1 old-age death by day 30 in every seed | {"max": 13.0, "median": 9.0, "min": 5.0, "n": 32, "p10": 6.0, "p90": 12.0} |
| W6 | replacement | PASS | births - deaths over days 30..120 >= -3 in every seed | {"max": 1.0, "median": 0.0, "min": -3.0, "n": 32, "p10": -1.0, "p90": 0.0} |
| W6 | alive_band | PASS | every day sample from day 30 has alive in [32, 44] | {"max": 42.0, "median": 42.0, "min": 37.0, "n": 2912, "p10": 41.0, "p90": 42.0} |
| W6 | child_share | PASS | day-120 child share median in [0.03, 0.20] | {"max": 0.1951219512195122, "median": 0.09523809523809523, "min": 0.023809523809523808, "n": 32, "p10": 0.047619047619047616, "p90": 0.17032520325203254} |
| W6 | elder_share | PASS | day-120 elder share median in [0.08, 0.35] | {"max": 0.38095238095238093, "median": 0.2619047619047619, "min": 0.17073170731707318, "n": 32, "p10": 0.21428571428571427, "p90": 0.3333333333333333} |
| W7 | autumn_store | PASS | median autumn-closing stock >= median 0.8 x days_per_season x max(0, need - winter supply) | {"paired_margin": {"max": 540.0959919691086, "median": 433.65999253988264, "min": 192.47199234962463, "n": 128, "p10": 314.17199234962465, "p90": 505.92239250183104}, "required": {"max": 161.2800084114075, "median": 150.52800765037537, "min": 118.27200536727906, "n": 128, "p10": 139.77600688934328, "p90": 161.2800084114075}, "stock": {"max": 696.0, "median": 586.5, "min": 343.0, "n": 128, "p10": 458.90000000000003, "p90": 659.0}} |
| W7 | hoard_cap | PASS | max stock - 3 x max target <= 0 in every seed | {"max": -352.40003830194473, "median": -457.90003830194473, "min": -668.4000383019447, "n": 32, "p10": -553.8000383019447, "p90": -369.7000383019447} |
| W8 | noon_idle | FAIL | noon idle share median <= 0.45 over spring-autumn days (all days without seasons) | {"max": 0.95, "median": 0.8086080586080586, "min": 0.6904761904761905, "n": 32, "p10": 0.7392857142857143, "p90": 0.9045296167247386} |
| W8 | forage_trips | FAIL | forage trips per day median >= 10 in summer and autumn (all days without seasons) | {"max": 88.0, "median": 0.0, "min": 0.0, "n": 1856, "p10": 0.0, "p90": 74.0} |
| W9 | night_in_shelter | PASS | median >= 0.70 | {"max": 1.0, "median": 1.0, "min": 0.926829268292683, "n": 32, "p10": 0.954112554112554, "p90": 1.0} |
| W9 | evening_near_hearth | FAIL | median >= 0.50 and every seed >= 0.30 | {"max": 0.8148148148148148, "median": 0.5712962962962963, "min": 0.2727272727272727, "n": 32, "p10": 0.346875, "p90": 0.7769230769230769} |
| W10 | home_distance | FAIL | day-10 median <= 7.0, every seed <= 10 | {"max": 11.023809523809524, "median": 7.011904761904762, "min": 4.976190476190476, "n": 64, "p10": 5.602380952380952, "p90": 8.595238095238095} |
| W10 | t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.002228826151560178, "median": 0.000866765725606736, "min": 0.0, "p10": 0.00024764735017335313, "p90": 0.001733531451213472} |
| W10 | social_10 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.6995777121716058, "median": 0.6512937036491746, "min": 0.6009193168653537, "n": 64, "p10": 0.6236406442692178, "p90": 0.6807429276131395} |
| W10 | t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.010502714184564551, "median": 0.008660161358459786, "min": 0.007013994267408531, "p10": 0.0072495272937139164, "p90": 0.01026362179416978} |
| W10 | social_120 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.8637170051874288, "median": 0.8350652139257995, "min": 0.8044751515669268, "n": 32, "p10": 0.811269126803727, "p90": 0.8532954624368686} |
| W11 | day10 | PASS | median ms/agent-tick <= 1.5 x same-host Pass 3 game_v9 | {"limit": 0.0011675054515673492, "max": 0.0010678417209578467, "median": 0.0010025302722656108, "min": 0.0009532533166411268, "n": 64, "p10": 0.0009718835494599921, "p90": 0.001031006044280014} |
| W11 | day120 | PASS | median ms/agent-tick <= 1.5 x same-host Pass 3 game_v9 | {"limit": 0.0013239339685035822, "max": 0.002278541186805405, "median": 0.0011981725015421183, "min": 0.001115890086099302, "n": 32, "p10": 0.0011274633409907077, "p90": 0.002072015220914973} |
