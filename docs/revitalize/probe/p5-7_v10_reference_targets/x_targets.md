# Pass 5 X targets

| Target | Clause | Verdict | Requirement | Observed |
|---|---|---|---|---|
| X1 | twins_10 | PASS | 64 seeds, determinism checked | 64 |
| X1 | twins_120 | PASS | 32 seeds, determinism checked | 32 |
| X1 | legacy_pin | PASS | legacy_pin verified externally | true |
| X1 | v8_repro | PASS | v8_repro verified externally | true |
| X1 | v9_repro | PASS | v9_repro verified externally | true |
| X1 | v10_repro | PASS | v10_repro verified externally | true |
| X2 | deaths_starvation_10 | PASS | zero in every seed at day 10 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 0.0} |
| X2 | deaths_starvation_120 | PASS | total median 0 and max <= 2 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X2 | deaths_dehydration_10 | PASS | zero in every seed at day 10 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 0.0} |
| X2 | deaths_dehydration_120 | PASS | total median 0 and max <= 2 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X3 | one_settlement | PASS | one settlement at every day sample | {"max": 1.0, "median": 1.0, "min": 1.0, "n": 4480, "p10": 1.0, "p90": 1.0} |
| X3 | birth_by_day4 | PASS | a birth by day 4 in every seed (10-day round) | {"max": 7.0, "median": 7.0, "min": 7.0, "n": 64, "p10": 7.0, "p90": 7.0} |
| X3 | alive_day10 | PASS | day-10 alive >= 30 in every seed | {"max": 42.0, "median": 42.0, "min": 42.0, "n": 64, "p10": 42.0, "p90": 42.0} |
| X3 | stagnation | PASS | longest stagnant run <= 4 days in every seed | {"max": 2.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X3 | old_age_by_day30 | PASS | >= 1 old-age death by day 30 in every seed | {"max": 13.0, "median": 9.0, "min": 5.0, "n": 32, "p10": 6.0, "p90": 12.0} |
| X3 | replacement | PASS | births >= deaths - 3 over days 30..120 in every seed | {"max": 1.0, "median": 0.0, "min": -3.0, "n": 32, "p10": -1.0, "p90": 0.0} |
| X3 | alive_band | PASS | alive in [32, 44] at every day sample from day 30 | {"max": 42.0, "median": 42.0, "min": 37.0, "n": 2912, "p10": 41.0, "p90": 42.0} |
| X3 | child_share | PASS | day-120 child share median in [0.03, 0.20] (W6) | {"max": 0.1951219512195122, "median": 0.09523809523809523, "min": 0.023809523809523808, "n": 32, "p10": 0.047619047619047616, "p90": 0.17032520325203254} |
| X3 | elder_share | PASS | day-120 elder share median in [0.08, 0.35] (W6) | {"max": 0.38095238095238093, "median": 0.2619047619047619, "min": 0.17073170731707318, "n": 32, "p10": 0.21428571428571427, "p90": 0.3333333333333333} |
| X4 | store_share | PASS | store meals / all meals per seed-day from day 3, median >= 0.60 | {"max": 1.0, "median": 0.9029126213592233, "min": 0.5638297872340425, "n": 3776, "p10": 0.7795348837209303, "p90": 0.9690721649484536} |
| X4 | every_seed | PASS | each seed's store share from day 3 >= 0.40 | {"max": 0.9437455705173635, "median": 0.8904518915074437, "min": 0.801338688085676, "n": 32, "p10": 0.8371032871298499, "p90": 0.9344676263001879} |
| X4 | winter | PASS | winter seed-day store share median >= 0.40 | {"max": 1.0, "median": 0.887119193791266, "min": 0.5638297872340425, "n": 896, "p10": 0.7340425531914894, "p90": 0.9810422282120395} |
| X5 | spring | FAIL | forage trips per day median >= 8 in spring | {"max": 75.0, "median": 0.0, "min": 0.0, "n": 1120, "p10": 0.0, "p90": 51.0} |
| X5 | summer | PASS | forage trips per day median >= 8 in summer | {"max": 91.0, "median": 19.0, "min": 0.0, "n": 928, "p10": 0.0, "p90": 77.0} |
| X5 | autumn | FAIL | forage trips per day median >= 8 in autumn | {"max": 80.0, "median": 0.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 0.0} |
| X5 | winter | FAIL | forage trips per day median >= 8 in winter | {"max": 53.0, "median": 0.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 0.0} |
| X6 | lit_at_0200 | FAIL | share of 02:00 samples from day 3 with the fire lit: median >= 0.90, every seed >= 0.75 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X6 | autumn_wood | FAIL | median last-autumn wood stock >= 0.8 x 7 x 39.0 | {"required": 218.40000000000003, "wood_stock": {"max": 0.0, "median": 0.0, "min": 0.0, "n": 128, "p10": 0.0, "p90": 0.0}} |
| X7 | winter_cold | PASS | winter cold members / members in the cold window: median <= 0.10, every seed <= 0.25 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X8 | work_1200 | FAIL | adult work share at 12:00 median >= 0.35 (spring-autumn) | {"max": 0.14285714285714285, "median": 0.08285714285714285, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.125} |
| X8 | work_1500 | FAIL | adult work share at 15:00 median >= 0.25 (spring-autumn) | {"max": 0.3793103448275862, "median": 0.24190177638453503, "min": 0.06666666666666667, "n": 32, "p10": 0.13035714285714287, "p90": 0.3591351351351351} |
| X8 | idle_1200 | FAIL | idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"max": 0.9285714285714286, "median": 0.7738095238095237, "min": 0.6585365853658537, "n": 32, "p10": 0.7142857142857143, "p90": 0.8809523809523809} |
| X9 | night_in_shelter | PASS | median >= 0.80 | {"max": 1.0, "median": 1.0, "min": 0.9285714285714286, "n": 32, "p10": 0.9547619047619047, "p90": 1.0} |
| X9 | evening_near_hearth | FAIL | median >= 0.55 and every seed >= 0.35 | {"max": 0.8125, "median": 0.5542114695340502, "min": 0.28125, "n": 32, "p10": 0.3479020979020979, "p90": 0.7664705882352941} |
| X10 | home_distance | PASS | day-10 median <= 8.1, every seed <= 13 | {"max": 11.047619047619047, "median": 7.059523809523809, "min": 5.142857142857143, "n": 64, "p10": 5.60952380952381, "p90": 8.30952380952381} |
| X10 | t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.002228826151560178, "median": 0.0009905894006934125, "min": 0.0, "p10": 0.00024764735017335313, "p90": 0.0014858841010401188} |
| X10 | social_10 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.6995777121716058, "median": 0.6497988794778677, "min": 0.5919503718375126, "n": 64, "p10": 0.6190605621631492, "p90": 0.687619118801151} |
| X10 | t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.01070302887289521, "median": 0.008761626854127569, "min": 0.00649120736456981, "p10": 0.007297068196597303, "p90": 0.010346251636852594} |
| X10 | social_120 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.8631108642688742, "median": 0.8352795868609617, "min": 0.8040587363490594, "n": 32, "p10": 0.809812983626291, "p90": 0.8532382730457948} |
| X11 | day10 | PASS | no gatherer with hunger or thirst >= 90 at any 120-tick sample | 0 |
| X11 | day120 | PASS | no gatherer with hunger or thirst >= 90 at any 120-tick sample | 0 |
| X12 | day10 | PASS | median ms/agent-tick <= 1.5 x same-host game_v10 | {"limit": 0.0014784587755521453, "max": 0.0010488854923161861, "median": 0.0009856391837014301, "min": 0.0009359740481914696, "n": 64, "p10": 0.0009540200281525174, "p90": 0.0010128351025765045} |
| X12 | day120 | PASS | median ms/agent-tick <= 1.5 x same-host game_v10 | {"limit": 0.0017828544950750367, "max": 0.0017977792513947548, "median": 0.0012894205102646134, "min": 0.0011362999564475031, "n": 32, "p10": 0.001187666972151279, "p90": 0.0016704581353234793} |
