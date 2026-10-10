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
| X3 | stagnation | FAIL | longest stagnant run <= 4 days in every seed | {"max": 5.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X3 | old_age_by_day30 | PASS | >= 1 old-age death by day 30 in every seed | {"max": 13.0, "median": 9.0, "min": 5.0, "n": 32, "p10": 6.0, "p90": 12.0} |
| X3 | replacement | PASS | births >= deaths - 3 over days 30..120 in every seed | {"max": 1.0, "median": 0.0, "min": -3.0, "n": 32, "p10": -1.0, "p90": 0.0} |
| X3 | alive_band | PASS | alive in [32, 44] at every day sample from day 30 | {"max": 42.0, "median": 42.0, "min": 35.0, "n": 2912, "p10": 41.0, "p90": 42.0} |
| X3 | child_share | PASS | day-120 child share median in [0.03, 0.20] (W6) | {"max": 0.1951219512195122, "median": 0.10714285714285714, "min": 0.0, "n": 32, "p10": 0.047619047619047616, "p90": 0.17032520325203254} |
| X3 | elder_share | PASS | day-120 elder share median in [0.08, 0.35] (W6) | {"max": 0.38095238095238093, "median": 0.2619047619047619, "min": 0.17073170731707318, "n": 32, "p10": 0.21428571428571427, "p90": 0.3406504065040651} |
| X4 | store_share | PASS | store meals / all meals per seed-day from day 3, median >= 0.60 | {"max": 1.0, "median": 0.9895833333333334, "min": 0.5517241379310345, "n": 3776, "p10": 0.9479166666666666, "p90": 1.0} |
| X4 | every_seed | PASS | each seed's store share from day 3 >= 0.40 | {"max": 0.9938140685754684, "median": 0.9853065789774471, "min": 0.9157798999545248, "n": 32, "p10": 0.9692199154891522, "p90": 0.9914220559223749} |
| X4 | winter | PASS | winter seed-day store share median >= 0.40 | {"max": 1.0, "median": 1.0, "min": 0.5517241379310345, "n": 896, "p10": 0.98, "p90": 1.0} |
| X5 | spring | PASS | forage trips per day median >= 8 in spring | {"max": 70.0, "median": 33.0, "min": 0.0, "n": 1120, "p10": 11.0, "p90": 60.0} |
| X5 | summer | PASS | forage trips per day median >= 8 in summer | {"max": 94.0, "median": 64.0, "min": 0.0, "n": 928, "p10": 0.0, "p90": 77.0} |
| X5 | autumn | FAIL | forage trips per day median >= 8 in autumn | {"max": 85.0, "median": 0.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 15.0} |
| X5 | winter | FAIL | forage trips per day median >= 8 in winter | {"max": 57.0, "median": 0.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 23.0} |
| X6 | lit_at_0200 | PASS | share of 02:00 samples from day 3 with the fire lit: median >= 0.90, every seed >= 0.75 | {"max": 1.0, "median": 1.0, "min": 1.0, "n": 32, "p10": 1.0, "p90": 1.0} |
| X6 | autumn_wood | PASS | median last-autumn wood stock >= 0.8 x 7 x 39.0 | {"required": 218.40000000000003, "wood_stock": {"max": 348.0, "median": 336.5, "min": 300.0, "n": 128, "p10": 315.0, "p90": 341.0}} |
| X7 | winter_cold | PASS | winter cold members / members in the cold window: median <= 0.10, every seed <= 0.25 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X8 | work_1200 | PASS | adult work share at 12:00 median >= 0.35 (spring-autumn) | {"max": 0.7391304347826086, "median": 0.4696691176470588, "min": 0.16666666666666666, "n": 32, "p10": 0.2946236559139785, "p90": 0.6122189638318671} |
| X8 | work_1500 | PASS | adult work share at 15:00 median >= 0.25 (spring-autumn) | {"max": 0.5135135135135135, "median": 0.35446859903381644, "min": 0.11764705882352941, "n": 32, "p10": 0.2059837728194726, "p90": 0.43324324324324326} |
| X8 | idle_1200 | PASS | idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"max": 0.8125, "median": 0.5060975609756098, "min": 0.325, "n": 32, "p10": 0.42857142857142855, "p90": 0.6236111111111111} |
| X9 | night_in_shelter | PASS | median >= 0.80 | {"max": 1.0, "median": 1.0, "min": 0.9523809523809523, "n": 32, "p10": 0.9761904761904762, "p90": 1.0} |
| X9 | evening_near_hearth | FAIL | median >= 0.55 and every seed >= 0.35 | {"max": 0.8064516129032258, "median": 0.5691287878787878, "min": 0.2857142857142857, "n": 32, "p10": 0.37573529411764706, "p90": 0.763235294117647} |
| X10 | home_distance | PASS | day-10 median <= 8.1, every seed <= 13 | {"max": 10.357142857142858, "median": 6.928571428571429, "min": 5.357142857142857, "n": 64, "p10": 6.14047619047619, "p90": 8.971428571428572} |
| X10 | t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.0032194155522535907, "median": 0.0012382367508667657, "min": 0.0, "p10": 0.00032194155522535925, "p90": 0.002649826646854879} |
| X10 | social_10 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.6935984154863784, "median": 0.6220642107117956, "min": 0.5271719008058176, "n": 64, "p10": 0.5895148295614026, "p90": 0.6594332900561665} |
| X10 | t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.014449015376315079, "median": 0.008509123556037789, "min": 0.006322847365769772, "p10": 0.0065614340575226035, "p90": 0.012446370958941256} |
| X10 | social_120 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.8588601148114919, "median": 0.8276578673544495, "min": 0.7916165051074503, "n": 32, "p10": 0.8016262567946125, "p90": 0.841985165955873} |
| X11 | day10 | PASS | no gatherer with hunger or thirst >= 90 at any 120-tick sample | 0 |
| X11 | day120 | PASS | no gatherer with hunger or thirst >= 90 at any 120-tick sample | 0 |
| X12 | day10 | PASS | median ms/agent-tick <= 1.5 x same-host game_v10 | {"limit": 0.0014784587755521453, "max": 0.002085189666197461, "median": 0.0010812697760216765, "min": 0.0010261410810866853, "n": 64, "p10": 0.0010478937495688118, "p90": 0.0014665804289097518} |
| X12 | day120 | PASS | median ms/agent-tick <= 1.5 x same-host game_v10 | {"limit": 0.0017828544950750367, "max": 0.0013271220951135356, "median": 0.0012614540398252381, "min": 0.0012064513242095673, "n": 32, "p10": 0.0012299773087227348, "p90": 0.001297135879448037} |
