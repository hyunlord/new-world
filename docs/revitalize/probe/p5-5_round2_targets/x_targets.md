# Pass 5 X targets

| Target | Clause | Verdict | Requirement | Observed |
|---|---|---|---|---|
| X1 | twins_10 | PASS | 64 seeds, determinism checked | 64 |
| X1 | twins_120 | PASS | 32 seeds, determinism checked | 32 |
| X1 | legacy_pin | FAIL | legacy_pin verified externally | false |
| X1 | v8_repro | FAIL | v8_repro verified externally | false |
| X1 | v9_repro | FAIL | v9_repro verified externally | false |
| X1 | v10_repro | FAIL | v10_repro verified externally | false |
| X2 | deaths_starvation_10 | PASS | zero in every seed at day 10 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 0.0} |
| X2 | deaths_starvation_120 | PASS | total median 0 and max <= 2 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X2 | deaths_dehydration_10 | PASS | zero in every seed at day 10 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 0.0} |
| X2 | deaths_dehydration_120 | PASS | total median 0 and max <= 2 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X3 | one_settlement | PASS | one settlement at every day sample | {"max": 1.0, "median": 1.0, "min": 1.0, "n": 4480, "p10": 1.0, "p90": 1.0} |
| X3 | birth_by_day4 | PASS | a birth by day 4 in every seed (10-day round) | {"max": 7.0, "median": 7.0, "min": 7.0, "n": 64, "p10": 7.0, "p90": 7.0} |
| X3 | alive_day10 | PASS | day-10 alive >= 30 in every seed | {"max": 42.0, "median": 42.0, "min": 42.0, "n": 64, "p10": 42.0, "p90": 42.0} |
| X3 | stagnation | PASS | longest stagnant run <= 4 days in every seed | {"max": 4.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X3 | old_age_by_day30 | PASS | >= 1 old-age death by day 30 in every seed | {"max": 13.0, "median": 9.0, "min": 5.0, "n": 32, "p10": 6.0, "p90": 12.0} |
| X3 | replacement | PASS | births >= deaths - 3 over days 30..120 in every seed | {"max": 1.0, "median": 0.0, "min": -3.0, "n": 32, "p10": -1.0, "p90": 0.0} |
| X3 | alive_band | PASS | alive in [32, 44] at every day sample from day 30 | {"max": 42.0, "median": 42.0, "min": 35.0, "n": 2912, "p10": 41.0, "p90": 42.0} |
| X3 | child_share | PASS | day-120 child share median in [0.03, 0.20] (W6) | {"max": 0.1951219512195122, "median": 0.10714285714285714, "min": 0.0, "n": 32, "p10": 0.047619047619047616, "p90": 0.17032520325203254} |
| X3 | elder_share | PASS | day-120 elder share median in [0.08, 0.35] (W6) | {"max": 0.38095238095238093, "median": 0.2619047619047619, "min": 0.17073170731707318, "n": 32, "p10": 0.21428571428571427, "p90": 0.3406504065040651} |
| X4 | store_share | PASS | store meals / all meals per seed-day from day 3, median >= 0.60 | {"max": 1.0, "median": 0.9895833333333334, "min": 0.5591397849462365, "n": 3776, "p10": 0.9479166666666666, "p90": 1.0} |
| X4 | every_seed | PASS | each seed's store share from day 3 >= 0.40 | {"max": 0.9945215163029071, "median": 0.9846546897708728, "min": 0.9119970938152756, "n": 32, "p10": 0.9688325040427322, "p90": 0.9914358549080188} |
| X4 | winter | PASS | winter seed-day store share median >= 0.40 | {"max": 1.0, "median": 1.0, "min": 0.7692307692307693, "n": 896, "p10": 0.9801980198019802, "p90": 1.0} |
| X5 | spring | PASS | forage trips per day median >= 8 in spring | {"max": 77.0, "median": 34.0, "min": 0.0, "n": 1120, "p10": 12.0, "p90": 61.0} |
| X5 | summer | PASS | forage trips per day median >= 8 in summer | {"max": 94.0, "median": 64.0, "min": 0.0, "n": 928, "p10": 0.0, "p90": 76.30000000000007} |
| X5 | autumn | FAIL | forage trips per day median >= 8 in autumn | {"max": 83.0, "median": 0.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 14.5} |
| X5 | winter | FAIL | forage trips per day median >= 8 in winter | {"max": 57.0, "median": 0.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 28.0} |
| X6 | lit_at_0200 | PASS | share of 02:00 samples from day 3 with the fire lit: median >= 0.90, every seed >= 0.75 | {"max": 1.0, "median": 1.0, "min": 1.0, "n": 32, "p10": 1.0, "p90": 1.0} |
| X6 | autumn_wood | PASS | median last-autumn wood stock >= 0.8 x 7 x 39.0 | {"required": 218.40000000000003, "wood_stock": {"max": 348.0, "median": 336.0, "min": 305.0, "n": 128, "p10": 313.0, "p90": 341.0}} |
| X7 | winter_cold | PASS | winter cold members / members in the cold window: median <= 0.10, every seed <= 0.25 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X8 | work_1200 | PASS | adult work share at 12:00 median >= 0.35 (spring-autumn) | {"max": 0.72, "median": 0.4745670995670996, "min": 0.16666666666666666, "n": 32, "p10": 0.30508021390374335, "p90": 0.6308695652173913} |
| X8 | work_1500 | PASS | adult work share at 15:00 median >= 0.25 (spring-autumn) | {"max": 0.5454545454545454, "median": 0.349833147942158, "min": 0.13333333333333333, "n": 32, "p10": 0.21507936507936506, "p90": 0.458968058968059} |
| X8 | idle_1200 | PASS | idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"max": 0.8048780487804879, "median": 0.5238095238095238, "min": 0.3333333333333333, "n": 32, "p10": 0.42857142857142855, "p90": 0.6642857142857143} |
| X9 | night_in_shelter | PASS | median >= 0.80 | {"max": 1.0, "median": 1.0, "min": 0.9512195121951219, "n": 32, "p10": 0.9761904761904762, "p90": 1.0} |
| X9 | evening_near_hearth | FAIL | median >= 0.55 and every seed >= 0.35 | {"max": 0.8148148148148148, "median": 0.555984555984556, "min": 0.30303030303030304, "n": 32, "p10": 0.35371621621621624, "p90": 0.7828185328185329} |
| X10 | home_distance | PASS | day-10 median <= 8.1, every seed <= 13 | {"max": 10.523809523809524, "median": 7.095238095238095, "min": 4.880952380952381, "n": 64, "p10": 6.214285714285714, "p90": 8.971428571428572} |
| X10 | t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.0032194155522535907, "median": 0.0012382367508667657, "min": 0.0, "p10": 0.00024764735017335313, "p90": 0.002402179296681526} |
| X10 | social_10 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.6935984154863784, "median": 0.6188572069210359, "min": 0.5484800865845917, "n": 64, "p10": 0.5846142096389373, "p90": 0.6578847473342376} |
| X10 | t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.01475249527920151, "median": 0.008496861862886961, "min": 0.005878775592945469, "p10": 0.006734588107477056, "p90": 0.01216191555030326} |
| X10 | social_120 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.8580544918568019, "median": 0.8258507522834317, "min": 0.7827181225737321, "n": 32, "p10": 0.8020363532007976, "p90": 0.8426532306200107} |
| X11 | day10 | PASS | no gatherer with hunger or thirst >= 90 at any 120-tick sample | 0 |
| X11 | day120 | PASS | no gatherer with hunger or thirst >= 90 at any 120-tick sample | 0 |
| X12 | day10 | PASS | median ms/agent-tick <= 1.5 x same-host game_v10 | {"limit": 0.0014917052378016129, "max": 0.0011459976443192322, "median": 0.001076805556593629, "min": 0.0010254025974837265, "n": 64, "p10": 0.0010531724641688238, "p90": 0.0011161920925447646} |
| X12 | day120 | PASS | median ms/agent-tick <= 1.5 x same-host game_v10 | {"limit": 0.0019341307653969202, "max": 0.00154312494449891, "median": 0.0012785349543749653, "min": 0.0012253943736296913, "n": 32, "p10": 0.001236250948578792, "p90": 0.0014954373160302298} |
