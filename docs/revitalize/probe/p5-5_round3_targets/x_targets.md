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
| X3 | stagnation | PASS | longest stagnant run <= 4 days in every seed | {"max": 3.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X3 | old_age_by_day30 | PASS | >= 1 old-age death by day 30 in every seed | {"max": 13.0, "median": 9.0, "min": 5.0, "n": 32, "p10": 6.0, "p90": 12.0} |
| X3 | replacement | PASS | births >= deaths - 3 over days 30..120 in every seed | {"max": 1.0, "median": 0.0, "min": -3.0, "n": 32, "p10": -1.0, "p90": 0.0} |
| X3 | alive_band | PASS | alive in [32, 44] at every day sample from day 30 | {"max": 42.0, "median": 42.0, "min": 36.0, "n": 2912, "p10": 41.0, "p90": 42.0} |
| X3 | child_share | PASS | day-120 child share median in [0.03, 0.20] (W6) | {"max": 0.1951219512195122, "median": 0.10714285714285714, "min": 0.0, "n": 32, "p10": 0.047619047619047616, "p90": 0.17032520325203254} |
| X3 | elder_share | PASS | day-120 elder share median in [0.08, 0.35] (W6) | {"max": 0.38095238095238093, "median": 0.2619047619047619, "min": 0.17073170731707318, "n": 32, "p10": 0.21428571428571427, "p90": 0.3406504065040651} |
| X4 | store_share | PASS | store meals / all meals per seed-day from day 3, median >= 0.60 | {"max": 1.0, "median": 0.9896907216494846, "min": 0.6739130434782609, "n": 3776, "p10": 0.9485857784826857, "p90": 1.0} |
| X4 | every_seed | PASS | each seed's store share from day 3 >= 0.40 | {"max": 0.9948749668640099, "median": 0.9863515289146154, "min": 0.9374155785682126, "n": 32, "p10": 0.9675072960174392, "p90": 0.9920361384909742} |
| X4 | winter | PASS | winter seed-day store share median >= 0.40 | {"max": 1.0, "median": 1.0, "min": 0.9238095238095239, "n": 896, "p10": 0.9802950883323627, "p90": 1.0} |
| X5 | spring | PASS | forage trips per day median >= 8 in spring | {"max": 113.0, "median": 32.5, "min": 0.0, "n": 1120, "p10": 11.0, "p90": 81.10000000000002} |
| X5 | summer | PASS | forage trips per day median >= 8 in summer | {"max": 144.0, "median": 83.0, "min": 0.0, "n": 928, "p10": 0.0, "p90": 120.0} |
| X5 | autumn | FAIL | forage trips per day median >= 8 in autumn | {"max": 139.0, "median": 0.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 0.0} |
| X5 | winter | FAIL | forage trips per day median >= 8 in winter | {"max": 85.0, "median": 0.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 21.0} |
| X6 | lit_at_0200 | PASS | share of 02:00 samples from day 3 with the fire lit: median >= 0.90, every seed >= 0.75 | {"max": 1.0, "median": 1.0, "min": 1.0, "n": 32, "p10": 1.0, "p90": 1.0} |
| X6 | autumn_wood | PASS | median last-autumn wood stock >= 0.8 x 7 x 39.0 | {"required": 218.40000000000003, "wood_stock": {"max": 347.0, "median": 335.0, "min": 301.0, "n": 128, "p10": 308.0, "p90": 341.0}} |
| X7 | winter_cold | PASS | winter cold members / members in the cold window: median <= 0.10, every seed <= 0.25 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X8 | work_1200 | PASS | adult work share at 12:00 median >= 0.35 (spring-autumn) | {"max": 0.75, "median": 0.5, "min": 0.125, "n": 32, "p10": 0.3596638655462185, "p90": 0.6645161290322581} |
| X8 | work_1500 | PASS | adult work share at 15:00 median >= 0.25 (spring-autumn) | {"max": 0.6538461538461539, "median": 0.41640866873065013, "min": 0.1875, "n": 32, "p10": 0.2600358422939068, "p90": 0.5155172413793104} |
| X8 | idle_1200 | PASS | idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"max": 0.7666666666666667, "median": 0.5119047619047619, "min": 0.3, "n": 32, "p10": 0.39169570267131243, "p90": 0.5746428571428571} |
| X9 | night_in_shelter | PASS | median >= 0.80 | {"max": 1.0, "median": 1.0, "min": 0.9285714285714286, "n": 32, "p10": 0.9547619047619047, "p90": 1.0} |
| X9 | evening_near_hearth | FAIL | median >= 0.55 and every seed >= 0.35 | {"max": 0.8, "median": 0.5903225806451613, "min": 0.3333333333333333, "n": 32, "p10": 0.38504273504273506, "p90": 0.7644257703081232} |
| X10 | home_distance | PASS | day-10 median <= 8.1, every seed <= 13 | {"max": 10.69047619047619, "median": 7.476190476190476, "min": 4.261904761904762, "n": 64, "p10": 5.816666666666666, "p90": 9.05952380952381} |
| X10 | t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.004210004952947003, "median": 0.0009905894006934125, "min": 0.0, "p10": 0.00024764735017335313, "p90": 0.0027321827862858182} |
| X10 | social_10 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.6786501737733099, "median": 0.6082083667856879, "min": 0.5304971329260805, "n": 64, "p10": 0.5590839726354473, "p90": 0.6508625468055917} |
| X10 | t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.013926355543566226, "median": 0.008404076250200669, "min": 0.006339706851955166, "p10": 0.006767982730654014, "p90": 0.011548564413316592} |
| X10 | social_120 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.8507662540179007, "median": 0.8125397418958623, "min": 0.7659217114016434, "n": 32, "p10": 0.7900517398519988, "p90": 0.836434099663905} |
| X11 | day10 | PASS | no gatherer with hunger or thirst >= 90 at any 120-tick sample | 0 |
| X11 | day120 | PASS | no gatherer with hunger or thirst >= 90 at any 120-tick sample | 0 |
| X12 | day10 | PASS | median ms/agent-tick <= 1.5 x same-host game_v10 | {"limit": 0.0014784587755521453, "max": 0.0011410055350017185, "median": 0.0010903953431848443, "min": 0.0010130483458388386, "n": 64, "p10": 0.0010545425280132116, "p90": 0.0011126181543489508} |
| X12 | day120 | PASS | median ms/agent-tick <= 1.5 x same-host game_v10 | {"limit": 0.0017828544950750367, "max": 0.0020363741663308142, "median": 0.0013035898315934633, "min": 0.0012338011511124373, "n": 32, "p10": 0.0012404033822276067, "p90": 0.0018741482779280711} |
