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
| X2 | deaths_starvation_120 | PASS | total median 0 and max <= 2 | {"max": 2.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X2 | deaths_dehydration_10 | PASS | zero in every seed at day 10 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 0.0} |
| X2 | deaths_dehydration_120 | PASS | total median 0 and max <= 2 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X3 | one_settlement | PASS | one settlement at every day sample | {"max": 1.0, "median": 1.0, "min": 1.0, "n": 4480, "p10": 1.0, "p90": 1.0} |
| X3 | birth_by_day4 | PASS | a birth by day 4 in every seed (10-day round) | {"max": 7.0, "median": 7.0, "min": 7.0, "n": 64, "p10": 7.0, "p90": 7.0} |
| X3 | alive_day10 | PASS | day-10 alive >= 30 in every seed | {"max": 42.0, "median": 42.0, "min": 41.0, "n": 64, "p10": 42.0, "p90": 42.0} |
| X3 | stagnation | FAIL | longest stagnant run <= 4 days in every seed | {"max": 9.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X3 | old_age_by_day30 | PASS | >= 1 old-age death by day 30 in every seed | {"max": 13.0, "median": 9.0, "min": 5.0, "n": 32, "p10": 6.0, "p90": 12.0} |
| X3 | replacement | PASS | births >= deaths - 3 over days 30..120 in every seed | {"max": 1.0, "median": 0.0, "min": -3.0, "n": 32, "p10": -1.0, "p90": 0.0} |
| X3 | alive_band | PASS | alive in [32, 44] at every day sample from day 30 | {"max": 42.0, "median": 42.0, "min": 33.0, "n": 2912, "p10": 41.0, "p90": 42.0} |
| X3 | child_share | PASS | day-120 child share median in [0.03, 0.20] (W6) | {"max": 0.1951219512195122, "median": 0.09523809523809523, "min": 0.0, "n": 32, "p10": 0.023809523809523808, "p90": 0.17032520325203254} |
| X3 | elder_share | PASS | day-120 elder share median in [0.08, 0.35] (W6) | {"max": 0.42857142857142855, "median": 0.2619047619047619, "min": 0.17073170731707318, "n": 32, "p10": 0.21428571428571427, "p90": 0.36572528883183575} |
| X4 | store_share | PASS | store meals / all meals per seed-day from day 3, median >= 0.60 | {"max": 1.0, "median": 0.9904761904761905, "min": 0.14942528735632185, "n": 3776, "p10": 0.96, "p90": 1.0} |
| X4 | every_seed | PASS | each seed's store share from day 3 >= 0.40 | {"max": 0.9966586703221384, "median": 0.9896335605071528, "min": 0.8575200291332847, "n": 32, "p10": 0.9767384191148153, "p90": 0.9951953977532076} |
| X4 | winter | PASS | winter seed-day store share median >= 0.40 | {"max": 1.0, "median": 1.0, "min": 0.14942528735632185, "n": 896, "p10": 0.9896907216494846, "p90": 1.0} |
| X5 | spring | PASS | forage trips per day median >= 8 in spring | {"max": 73.0, "median": 41.0, "min": 0.0, "n": 1120, "p10": 12.900000000000006, "p90": 60.0} |
| X5 | summer | PASS | forage trips per day median >= 8 in summer | {"max": 87.0, "median": 64.0, "min": 0.0, "n": 928, "p10": 16.0, "p90": 77.0} |
| X5 | autumn | FAIL | forage trips per day median >= 8 in autumn | {"max": 82.0, "median": 0.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 28.0} |
| X5 | winter | FAIL | forage trips per day median >= 8 in winter | {"max": 58.0, "median": 0.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 31.5} |
| X6 | lit_at_0200 | PASS | share of 02:00 samples from day 3 with the fire lit: median >= 0.90, every seed >= 0.75 | {"max": 1.0, "median": 1.0, "min": 1.0, "n": 32, "p10": 1.0, "p90": 1.0} |
| X6 | autumn_wood | PASS | median last-autumn wood stock >= 0.8 x 7 x 39.0 | {"required": 218.40000000000003, "wood_stock": {"max": 347.0, "median": 334.5, "min": 302.0, "n": 128, "p10": 312.0, "p90": 341.0}} |
| X7 | winter_cold | PASS | winter cold members / members in the cold window: median <= 0.10, every seed <= 0.25 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X8 | work_1200 | PASS | adult work share at 12:00 median >= 0.35 (spring-autumn) | {"max": 0.7272727272727273, "median": 0.4583333333333333, "min": 0.125, "n": 32, "p10": 0.3161904761904762, "p90": 0.5763736263736263} |
| X8 | work_1500 | PASS | adult work share at 15:00 median >= 0.25 (spring-autumn) | {"max": 0.6451612903225806, "median": 0.3484848484848485, "min": 0.11764705882352941, "n": 32, "p10": 0.21995192307692307, "p90": 0.4441176470588235} |
| X8 | idle_1200 | PASS | idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"max": 0.8571428571428571, "median": 0.5238095238095238, "min": 0.34285714285714286, "n": 32, "p10": 0.4309523809523809, "p90": 0.6326364692218351} |
| X9 | night_in_shelter | PASS | median >= 0.80 | {"max": 1.0, "median": 1.0, "min": 0.9459459459459459, "n": 32, "p10": 0.9756678281068525, "p90": 1.0} |
| X9 | evening_near_hearth | FAIL | median >= 0.55 and every seed >= 0.35 | {"max": 0.8421052631578947, "median": 0.5554435483870968, "min": 0.2692307692307692, "n": 32, "p10": 0.3554848966613673, "p90": 0.775} |
| X10 | home_distance | PASS | day-10 median <= 8.1, every seed <= 13 | {"max": 10.666666666666666, "median": 6.892857142857142, "min": 5.309523809523809, "n": 64, "p10": 5.576190476190476, "p90": 8.321428571428571} |
| X10 | t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.003467062902426944, "median": 0.0012382367508667657, "min": 0.0, "p10": 0.00024764735017335313, "p90": 0.0026531532530512376} |
| X10 | social_10 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.6965880638289921, "median": 0.6283502571900773, "min": 0.5544744044707621, "n": 64, "p10": 0.5847045985257849, "p90": 0.6579014523291042} |
| X10 | t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.013808102776794295, "median": 0.008588149275643775, "min": 0.003373209069411601, "p10": 0.006677431528585085, "p90": 0.011756982539985623} |
| X10 | social_120 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.8602796094237481, "median": 0.8244011855164111, "min": 0.7899669323255566, "n": 32, "p10": 0.8021389831335517, "p90": 0.8443973154077361} |
| X11 | day10 | PASS | no gatherer with hunger or thirst >= 90 at any 120-tick sample | 0 |
| X11 | day120 | PASS | no gatherer with hunger or thirst >= 90 at any 120-tick sample | 0 |
| X12 | day10 | PASS | median ms/agent-tick <= 1.5 x same-host game_v10 | {"limit": 0.0014917052378016129, "max": 0.0014395489969737535, "median": 0.0011443531816244417, "min": 0.001033224909720996, "n": 64, "p10": 0.0010712654797484485, "p90": 0.0012638245109501858} |
| X12 | day120 | PASS | median ms/agent-tick <= 1.5 x same-host game_v10 | {"limit": 0.0019341307653969202, "max": 0.0016633539050746507, "median": 0.0013090284935527358, "min": 0.0012086066721833388, "n": 32, "p10": 0.0012427881501239444, "p90": 0.0016123042883596727} |
