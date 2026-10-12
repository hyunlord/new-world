# Pass 9 game_v15 tuning (vs v12 tuning)

Values: median / p10 / p90 / min / max. Y9 is reported only. Severity weights (350, 400, 250) (summary.json rules block).

| Target | Clause | Verdict | Requirement | Observed |
|---|---|---|---|---|
| X1 | twins_10 | PASS | every seed of the set (64), determinism checked | {"range": {"end_exclusive": 64, "start": 0}, "seeds": 64} |
| X1 | twins_120 | PASS | every seed of the set (32 or 64), determinism checked | {"range": {"end_exclusive": 32, "start": 0}, "seeds": 32} |
| X1 | legacy_pin | FAIL | legacy_pin verified externally | false |
| X1 | v8_repro | FAIL | v8_repro verified externally | false |
| X1 | v9_repro | FAIL | v9_repro verified externally | false |
| X1 | v10_repro | FAIL | v10_repro verified externally | false |
| X1 | v11_repro | FAIL | v11_repro verified externally | false |
| X1 | v12_repro | FAIL | v12_repro verified externally | false |
| X1 | v13_repro | FAIL | v13_repro verified externally | false |
| X2 | Y2_deaths_starvation_10 | PASS | zero in every seed at day 10 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 0.0} |
| X2 | Y2_deaths_starvation_120 | PASS | total median 0 and max <= 2 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X2 | Y2_deaths_dehydration_10 | PASS | zero in every seed at day 10 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 0.0} |
| X2 | Y2_deaths_dehydration_120 | PASS | total median 0 and max <= 2 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| X2 | Y3_one_settlement | PASS | failing seeds <= reference's + 1 (day samples with settlements != 1 (every seed: 0)) | {"failing": {}, "n": 96, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y3_birth_by_day4 | PASS | failing seeds <= reference's + 1 (births by day 4 >= 1 (10-day round)) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y3_alive_day10 | PASS | failing seeds <= reference's + 1 (day-10 alive >= 30) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y3_stagnation | PASS | failing seeds <= reference's + 1 (longest stagnant run <= 4 days) | {"failing": {"22": 8.0}, "n": 32, "n_failing": 1, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y3_old_age_by_day30 | PASS | failing seeds <= reference's + 1 (>= 1 old-age death by day 30) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y3_replacement | PASS | failing seeds <= reference's + 1 (births - deaths over days 30..120 >= -3) | {"failing": {"22": -7.0}, "n": 32, "n_failing": 1, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y3_alive_band | PASS | failing seeds <= reference's + 1 (alive in [32, 44] at every day sample from day 30 ([min, max])) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y7_winter_lit_every_seed | PASS | failing seeds <= reference's + 1 (share of winter nights lit at 02:00 >= 0.50) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y7_other_lit_every_seed | PASS | failing seeds <= reference's + 1 (outside winter, 02:00 lit share >= 0.85) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y8_winter_cold_every_seed | PASS | failing seeds <= reference's + 1 (winter cold members / members in the cold window <= 0.50) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y10_evening_near_hearth_every_seed | PASS | failing seeds <= reference's + 1 (per-seed median evening near-hearth share >= 0.35) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y11_home_distance_every_seed | PASS | failing seeds <= reference's + 1 (day-10 home distance <= 13) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y11_social_10_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y12_day10 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y11_social_120_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y12_day120 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y3_child_share | PASS | day-120 child share median in [0.03, 0.20] (W6) | {"max": 0.19047619047619047, "median": 0.11904761904761904, "min": 0.0, "n": 32, "p10": 0.052142857142857144, "p90": 0.14634146341463414} |
| X2 | Y3_elder_share | PASS | day-120 elder share median in [0.08, 0.35] (W6) | {"max": 0.37142857142857144, "median": 0.2682926829268293, "min": 0.19047619047619047, "n": 32, "p10": 0.225, "p90": 0.30952380952380953} |
| X3 | a_r2 | PASS | severity-only R2 of the dark-night count <= 0.75 (undefined fails) | {"n": 128, "r2": 0.593647990275793} |
| X3 | b_spread_within_severity | PASS | normal and harsh winters each: the most common dark-night count <= 0.75 of them | {"harsh": {"histogram": {"0": 1, "1": 2, "2": 6, "3": 13, "4": 4}, "mode_share": 0.5, "n": 26}, "normal": {"histogram": {"0": 15, "1": 19, "2": 13, "3": 3}, "mode_share": 0.38, "n": 50}} |
| X3 | c_no_dark_night | PASS | share of seed-winters with no dark night in [0.15, 0.60] | {"n": 128, "share": 0.484375} |
| X3 | d_two_or_more | PASS | share of seed-winters with >= 2 dark nights in [0.10, 0.50] | {"n": 128, "share": 0.3203125} |
| X3 | e_first_night_spread | PASS | first dark night on >= 4 distinct nights, <= 0.50 of them on the seventh | {"distinct_first_nights": 4, "histogram": {"4": 4, "5": 19, "6": 19, "7": 24}, "n": 66, "seventh_share": 0.36363636363636365} |
| X3 | f_severity_shares | PASS | each severity's share of the seed-winters within +-0.10 of its weight | {"shares": {"harsh": 0.203125, "mild": 0.40625, "normal": 0.390625}, "weights": {"harsh": 250, "mild": 350, "normal": 400}} |
| X4 | retarget_share | PASS | winter wood_retargets / winter wood trips, median over seed-winters <= 0.10 | {"max": 0.1875, "median": 0.04499494438827098, "min": 0.0, "n": 128, "p10": 0.0, "p90": 0.10282935455349249} |
| X4 | normal_gathered | PASS | median winter wood gathered over normal-severity seed-winters >= 40.5 | {"floor": 40.5, "max": 58.0, "median": 48.0, "min": 38.0, "n": 50, "p10": 41.0, "p90": 54.0} |
| X5 | Y4_store_share | PASS | store meals / all meals per seed-day from day 3, median >= 0.60 | {"max": 1.0, "median": 0.9896907216494846, "min": 0.5164835164835165, "n": 3776, "p10": 0.9489795918367347, "p90": 1.0} |
| X5 | Y4_every_seed | PASS | each seed's store share from day 3 >= 0.40 | {"max": 0.994816376735196, "median": 0.9871248957888836, "min": 0.9256279610261912, "n": 32, "p10": 0.9702857744518072, "p90": 0.9932989241849498} |
| X5 | Y4_winter | PASS | winter seed-day store share median >= 0.40 | {"max": 1.0, "median": 1.0, "min": 0.7790697674418605, "n": 896, "p10": 0.9797979797979798, "p90": 1.0} |
| X5 | Y5_autumn_wood | PASS | autumn wood gathered per day median >= 1.5 x summer's (seed-days) | {"autumn": {"max": 149.0, "median": 49.5, "min": 0.0, "n": 896, "p10": 0.0, "p90": 129.0}, "summer": {"max": 49.0, "median": 0.0, "min": 0.0, "n": 928, "p10": 0.0, "p90": 36.0}} |
| X5 | Y5_autumn_work | PASS | autumn adult work share at 12:00 median >= v11's autumn value - 0.05 | {"floor": 0.40784549731918157, "max": 0.7941176470588235, "median": 0.6277257095805483, "min": 0.34834558823529416, "n": 32, "p10": 0.4718183548886447, "p90": 0.7416891891891891} |
| X5 | Y6_trip_days | PASS | days with wood trips per winter (of 7), median over seed-winters >= 4 | {"max": 7.0, "median": 5.0, "min": 1.0, "n": 128, "p10": 2.0, "p90": 7.0} |
| X5 | Y6_trips | PASS | wood trips per winter, median over seed-winters >= 20 | {"max": 67.0, "median": 47.0, "min": 22.0, "n": 128, "p10": 35.7, "p90": 57.0} |
| X5 | Y7_winter_lit_median | PASS | per seed share of winter nights lit at 02:00: median in [0.70, 0.97] | {"max": 0.9642857142857143, "median": 0.8571428571428571, "min": 0.6428571428571429, "n": 32, "p10": 0.7857142857142857, "p90": 0.9285714285714286} |
| X5 | Y7_other_lit_median | PASS | outside winter, 02:00 lit share: median >= 0.95 | {"max": 1.0, "median": 1.0, "min": 1.0, "n": 32, "p10": 1.0, "p90": 1.0} |
| X5 | Y8_winter_cold_median | PASS | winter cold members / members in the cold window: median <= 0.25 | {"max": 0.3471127210739399, "median": 0.16066868005296475, "min": 0.034908471690080885, "n": 32, "p10": 0.07389688372227798, "p90": 0.22264362300505325} |
| X5 | Y8_winter_food | PASS | food store at each winter's last periodic sample, median >= 0.8 x v11's | {"floor": 202.8, "max": 293.0, "median": 240.0, "min": 7.0, "n": 128, "p10": 181.4, "p90": 268.0} |
| X5 | Y10_night_in_shelter | PASS | median >= 0.80 | {"max": 1.0, "median": 1.0, "min": 0.9285714285714286, "n": 32, "p10": 0.9547038327526132, "p90": 1.0} |
| X5 | Y10_evening_near_hearth_median | PASS | evening near-hearth share median >= 0.55 | {"max": 0.8518518518518519, "median": 0.6348148148148148, "min": 0.375, "n": 32, "p10": 0.39588815789473686, "p90": 0.7925809822361547} |
| X5 | Y11_home_distance_median | PASS | day-10 home distance median <= 8.1 | {"max": 10.976190476190476, "median": 7.357142857142858, "min": 4.619047619047619, "n": 64, "p10": 5.983333333333333, "p90": 8.904761904761905} |
| X5 | Y11_t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.0029717682020802376, "median": 0.0011165070847096823, "min": 0.00024764735017335313, "p10": 0.00032194155522535925, "p90": 0.0027241208519068845} |
| X5 | Y11_social_10_median | PASS | social completions per agent-day median >= 0.40 | {"max": 0.6865923358006184, "median": 0.6254355505586322, "min": 0.5650435367539893, "n": 64, "p10": 0.5889669598301455, "p90": 0.6658804278077373} |
| X5 | Y11_t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.012810140237324703, "median": 0.008711220511073375, "min": 0.006269645452394633, "p10": 0.007649658733253948, "p90": 0.011563381618654438} |
| X5 | Y11_social_120_median | PASS | social completions per agent-day median >= 0.40 | {"max": 0.8582128293298021, "median": 0.8133386080775111, "min": 0.7677141103666381, "n": 32, "p10": 0.7854579699893618, "p90": 0.8409830496342008} |
| X6 | receipt | FAIL | every Pass 7 intervention test passes and 8.1's Z6 for v14 | false |
| X7 | receipt | FAIL | schema 2 byte identity, hidden columns match the probe, v0 sample loads and Part 0.4 reproduced it, dataset v1 manifest verified | false |
| X8 | day10 | FAIL | median ms/agent-tick <= 1.2 x same-host game_v14, recorder off | {"limit": null, "max": 0.0012472369611094339, "median": 0.0011516034312355665, "min": 0.0010787173809647433, "n": 64, "p10": 0.001124473090268474, "p90": 0.001200553171352819} |
| X8 | day120 | FAIL | median ms/agent-tick <= 1.2 x same-host game_v14, recorder off | {"limit": null, "max": 0.001677244906732276, "median": 0.001492017331856421, "min": 0.001412033125205143, "n": 32, "p10": 0.0014312155543895332, "p90": 0.0016360384553514974} |
| Y9 | work_1200 | FAIL | adult work share at 12:00 median >= 0.35 (spring-autumn) | {"max": 0.6944444444444444, "median": 0.3076923076923077, "min": 0.03571428571428571, "n": 32, "p10": 0.16999999999999998, "p90": 0.45732758620689656} |
| Y9 | work_1500 | PASS | adult work share at 15:00 median >= 0.25 (spring-autumn) | {"max": 0.6666666666666666, "median": 0.42708333333333337, "min": 0.14814814814814814, "n": 32, "p10": 0.26916666666666667, "p90": 0.5317241379310346} |
| Y9 | idle_1200 | FAIL | idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"max": 0.9024390243902439, "median": 0.6339285714285714, "min": 0.3333333333333333, "n": 32, "p10": 0.5047619047619047, "p90": 0.7357142857142858} |

## Per-seed clauses

| Clause | Seeds | Failing seeds (value) |
|---|---:|---|
| Y3_one_settlement | 96 | none |
| Y3_birth_by_day4 | 64 | none |
| Y3_alive_day10 | 64 | none |
| Y3_stagnation | 32 | 22 (8.0) |
| Y3_old_age_by_day30 | 32 | none |
| Y3_replacement | 32 | 22 (-7.0) |
| Y3_alive_band | 32 | none |
| Y7_winter_lit_every_seed | 32 | none |
| Y7_other_lit_every_seed | 32 | none |
| Y8_winter_cold_every_seed | 32 | none |
| Y10_evening_near_hearth_every_seed | 32 | none |
| Y11_home_distance_every_seed | 64 | none |
| Y11_social_10_every_seed | 64 | none |
| Y12_day10 | 64 | none |
| Y11_social_120_every_seed | 32 | none |
| Y12_day120 | 32 | none |

## Work by season

| Season | Adult work 12:00 | Idle 12:00 | Adult work 15:00 | Idle 15:00 |
|---|---|---|---|---|
| spring | 0.389 / 0.232 / 0.549 / 0.0354 / 0.759 | 0.585 / 0.477 / 0.69 / 0.312 / 0.905 | 0.431 / 0.241 / 0.626 / 0.119 / 0.679 | 0.577 / 0.458 / 0.712 / 0.325 / 0.762 |
| summer | 0.0702 / 0.0304 / 0.23 / 0 / 0.462 | 0.774 / 0.669 / 0.881 / 0.5 / 0.925 | 0.424 / 0.301 / 0.519 / 0.205 / 0.667 | 0.548 / 0.476 / 0.663 / 0.361 / 0.714 |
| autumn | 0.628 / 0.472 / 0.742 / 0.348 / 0.794 | 0.393 / 0.3 / 0.492 / 0.214 / 0.639 | 0.367 / 0.141 / 0.476 / 0.0616 / 0.65 | 0.575 / 0.465 / 0.748 / 0.25 / 0.845 |
| winter | 0.0951 / 0.0346 / 0.252 / 0.0147 / 0.271 | 0.815 / 0.738 / 0.891 / 0.627 / 0.917 | 0.161 / 0.0191 / 0.407 / 0 / 0.712 | 0.804 / 0.621 / 0.893 / 0.381 / 0.917 |

## Wood by season

| Season | Wood stock | Wood target | Gathered/day | Wood trips/day | Urgent trips/day | Lit at 02:00 | Cold member-hours/day | Food store | Forage trips/day |
|---|---|---|---|---|---|---:|---|---|---|
| spring | 25 / 13 / 43 / 0 / 87 | 39 / 39 / 39 / 39 / 39 | 30 / 0 / 50 / 0 / 67 | 15 / 0 / 28 / 0 / 38 | 4.5 / 0 / 19 / 0 / 34 | 1.000 | 0 / 0 / 0 / 0 / 0 | 180 / 69.7 / 232 / 0 / 282 | 61 / 15 / 86 / 0 / 101 |
| summer | 21 / 10 / 38.1 / 7 / 52 | 19.5 / 19.5 / 19.5 / 19.5 / 19.5 | 0 / 0 / 36 / 0 / 49 | 0 / 0 / 18 / 0 / 25 | 0 / 0 / 9.3 / 0 / 22 | 1.000 | 0 / 0 / 0 / 0 / 0 | 280 / 154 / 412 / 0 / 553 | 107 / 0 / 127 / 0 / 139 |
| autumn | 180 / 30 / 206 / 7 / 210 | 195 / 195 / 195 / 195 / 195 | 49.5 / 0 / 129 / 0 / 149 | 25 / 0 / 74 / 0 / 92 | 0 / 0 / 0 / 0 / 27 | 1.000 | 0 / 0 / 0 / 0 / 0 | 497 / 413 / 585 / 92 / 654 | 0 / 0 / 11.5 / 0 / 126 |
| winter | 105 / 0 / 195 / 0 / 212 | 156 / 104 / 208 / 104 / 208 | 5 / 0 / 20 / 0 / 22 | 5 / 0 / 21 / 0 / 28 | 0 / 0 / 5 / 0 / 10 | 0.854 | 0 / 0 / 335 / 0 / 336 | 316 / 216 / 531 / 0 / 651 | 0 / 0 / 24 / 0 / 84 |

## Winters

One row per complete seed-winter (Pass 7's columns). Night burn = mean `night_burn_per_hour` at the winter's 02:00 samples; fall = mean `winter_fall_today` at its seven 08:00 checkpoints.

| Seed | Year | Severity | Wood at start | Nights out | First night out | Trip-days | Wood trips | Urgent trips | Gathered | Re-targets | Re-target share | Cold member-hours | Food store at end | Night burn | Fall |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 0 | 1 | 177 | 2 | 5 | 6 | 52 | 21 | 51 | 1 | 0.019 | 798 | 239 | 3.00 | 5.86 |
| 0 | 1 | 0 | 203 | 0 | 0 | 1 | 23 | 0 | 22 | 1 | 0.043 | 0 | 248 | 2.14 | 7.57 |
| 0 | 2 | 2 | 176 | 4 | 4 | 7 | 49 | 19 | 49 | 0 | 0.000 | 1344 | 245 | 4.57 | 4.43 |
| 0 | 3 | 1 | 201 | 0 | 0 | 5 | 47 | 6 | 46 | 1 | 0.021 | 0 | 257 | 2.71 | 6.14 |
| 1 | 0 | 0 | 205 | 0 | 0 | 3 | 41 | 0 | 37 | 5 | 0.122 | 0 | 241 | 2.14 | 8.71 |
| 1 | 1 | 1 | 206 | 1 | 7 | 5 | 52 | 9 | 50 | 3 | 0.058 | 336 | 258 | 3.29 | 7.00 |
| 1 | 2 | 0 | 203 | 0 | 0 | 2 | 34 | 0 | 30 | 2 | 0.059 | 0 | 258 | 1.71 | 7.86 |
| 1 | 3 | 0 | 179 | 0 | 0 | 3 | 39 | 0 | 36 | 2 | 0.051 | 0 | 256 | 1.86 | 7.14 |
| 2 | 0 | 1 | 208 | 1 | 7 | 4 | 43 | 9 | 41 | 1 | 0.023 | 336 | 203 | 3.00 | 6.86 |
| 2 | 1 | 1 | 180 | 2 | 6 | 6 | 59 | 19 | 55 | 4 | 0.068 | 713 | 180 | 3.29 | 6.57 |
| 2 | 2 | 1 | 201 | 0 | 0 | 5 | 48 | 8 | 46 | 1 | 0.021 | 0 | 201 | 2.71 | 6.00 |
| 2 | 3 | 1 | 210 | 2 | 6 | 5 | 41 | 9 | 39 | 2 | 0.049 | 546 | 173 | 3.29 | 4.86 |
| 3 | 0 | 0 | 207 | 0 | 0 | 3 | 45 | 0 | 40 | 5 | 0.111 | 0 | 262 | 2.57 | 9.14 |
| 3 | 1 | 1 | 207 | 2 | 6 | 5 | 48 | 13 | 46 | 2 | 0.042 | 670 | 264 | 3.86 | 5.86 |
| 3 | 2 | 0 | 205 | 0 | 0 | 3 | 37 | 0 | 37 | 0 | 0.000 | 0 | 281 | 2.00 | 8.14 |
| 3 | 3 | 0 | 205 | 0 | 0 | 2 | 33 | 0 | 31 | 3 | 0.091 | 0 | 259 | 2.43 | 8.57 |
| 4 | 0 | 0 | 202 | 0 | 0 | 4 | 59 | 0 | 55 | 2 | 0.034 | 0 | 271 | 2.29 | 7.86 |
| 4 | 1 | 2 | 207 | 3 | 5 | 7 | 49 | 11 | 46 | 3 | 0.061 | 1008 | 242 | 4.29 | 4.29 |
| 4 | 2 | 0 | 205 | 0 | 0 | 2 | 40 | 0 | 38 | 3 | 0.075 | 0 | 293 | 1.71 | 9.00 |
| 4 | 3 | 0 | 206 | 0 | 0 | 4 | 52 | 10 | 51 | 4 | 0.077 | 0 | 277 | 2.71 | 8.71 |
| 5 | 0 | 1 | 177 | 0 | 0 | 5 | 53 | 0 | 48 | 5 | 0.094 | 0 | 211 | 2.43 | 6.57 |
| 5 | 1 | 2 | 199 | 2 | 6 | 7 | 41 | 11 | 41 | 1 | 0.024 | 672 | 196 | 4.00 | 3.71 |
| 5 | 2 | 0 | 182 | 1 | 7 | 4 | 52 | 18 | 49 | 3 | 0.058 | 294 | 244 | 2.71 | 9.29 |
| 5 | 3 | 2 | 203 | 3 | 5 | 7 | 46 | 14 | 43 | 4 | 0.087 | 1008 | 185 | 4.57 | 3.71 |
| 6 | 0 | 2 | 210 | 3 | 5 | 6 | 46 | 18 | 44 | 1 | 0.022 | 1174 | 176 | 4.57 | 4.43 |
| 6 | 1 | 1 | 204 | 1 | 7 | 5 | 48 | 5 | 46 | 1 | 0.021 | 290 | 225 | 3.00 | 6.29 |
| 6 | 2 | 1 | 212 | 2 | 6 | 5 | 42 | 9 | 40 | 0 | 0.000 | 672 | 218 | 3.29 | 5.57 |
| 6 | 3 | 1 | 174 | 0 | 0 | 6 | 53 | 4 | 52 | 2 | 0.038 | 168 | 214 | 2.71 | 6.29 |
| 7 | 0 | 0 | 205 | 0 | 0 | 4 | 53 | 19 | 47 | 7 | 0.132 | 0 | 256 | 2.71 | 8.14 |
| 7 | 1 | 0 | 203 | 0 | 0 | 3 | 52 | 0 | 50 | 4 | 0.077 | 0 | 243 | 2.71 | 9.00 |
| 7 | 2 | 2 | 179 | 2 | 6 | 7 | 57 | 20 | 49 | 4 | 0.070 | 672 | 260 | 3.14 | 4.71 |
| 7 | 3 | 0 | 194 | 2 | 6 | 4 | 48 | 17 | 45 | 4 | 0.083 | 630 | 207 | 3.43 | 8.43 |
| 8 | 0 | 0 | 204 | 0 | 0 | 4 | 46 | 0 | 46 | 1 | 0.022 | 0 | 278 | 2.14 | 8.43 |
| 8 | 1 | 2 | 189 | 2 | 6 | 7 | 51 | 16 | 50 | 1 | 0.020 | 713 | 241 | 3.71 | 4.71 |
| 8 | 2 | 0 | 176 | 0 | 0 | 3 | 40 | 5 | 39 | 1 | 0.025 | 165 | 249 | 2.57 | 8.14 |
| 8 | 3 | 0 | 207 | 0 | 0 | 1 | 22 | 0 | 21 | 3 | 0.136 | 0 | 245 | 1.57 | 8.29 |
| 9 | 0 | 1 | 173 | 2 | 6 | 6 | 52 | 13 | 49 | 2 | 0.038 | 503 | 241 | 2.71 | 5.43 |
| 9 | 1 | 1 | 181 | 3 | 5 | 6 | 53 | 18 | 51 | 1 | 0.019 | 1007 | 198 | 3.57 | 6.00 |
| 9 | 2 | 1 | 208 | 0 | 0 | 6 | 59 | 0 | 54 | 4 | 0.068 | 0 | 232 | 2.71 | 6.29 |
| 9 | 3 | 0 | 204 | 0 | 0 | 4 | 49 | 9 | 46 | 3 | 0.061 | 168 | 252 | 2.86 | 8.00 |
| 10 | 0 | 1 | 180 | 0 | 0 | 6 | 58 | 8 | 52 | 6 | 0.103 | 0 | 253 | 2.43 | 5.71 |
| 10 | 1 | 2 | 203 | 3 | 5 | 7 | 49 | 11 | 46 | 3 | 0.061 | 918 | 217 | 4.57 | 4.14 |
| 10 | 2 | 0 | 204 | 0 | 0 | 3 | 41 | 0 | 38 | 3 | 0.073 | 0 | 244 | 2.00 | 8.43 |
| 10 | 3 | 0 | 201 | 0 | 0 | 3 | 39 | 8 | 37 | 4 | 0.103 | 168 | 216 | 2.71 | 7.57 |
| 11 | 0 | 0 | 193 | 0 | 0 | 2 | 35 | 0 | 31 | 4 | 0.114 | 0 | 270 | 2.29 | 8.43 |
| 11 | 1 | 2 | 174 | 4 | 4 | 7 | 44 | 11 | 41 | 3 | 0.068 | 1259 | 208 | 4.57 | 3.57 |
| 11 | 2 | 0 | 175 | 0 | 0 | 4 | 45 | 0 | 43 | 1 | 0.022 | 0 | 248 | 2.00 | 7.14 |
| 11 | 3 | 0 | 204 | 0 | 0 | 2 | 32 | 0 | 32 | 0 | 0.000 | 0 | 240 | 2.29 | 8.29 |
| 12 | 0 | 0 | 204 | 0 | 0 | 3 | 38 | 4 | 35 | 4 | 0.105 | 0 | 258 | 2.43 | 7.43 |
| 12 | 1 | 2 | 203 | 3 | 5 | 7 | 58 | 17 | 56 | 4 | 0.069 | 923 | 235 | 4.86 | 5.71 |
| 12 | 2 | 2 | 204 | 2 | 6 | 7 | 50 | 14 | 47 | 3 | 0.060 | 588 | 236 | 3.43 | 4.14 |
| 12 | 3 | 0 | 204 | 0 | 0 | 4 | 43 | 0 | 42 | 4 | 0.093 | 126 | 222 | 2.86 | 6.71 |
| 13 | 0 | 0 | 170 | 1 | 7 | 5 | 56 | 17 | 56 | 0 | 0.000 | 378 | 215 | 2.71 | 8.57 |
| 13 | 1 | 1 | 200 | 0 | 0 | 6 | 60 | 9 | 58 | 1 | 0.017 | 168 | 180 | 3.00 | 7.29 |
| 13 | 2 | 1 | 182 | 1 | 7 | 6 | 51 | 11 | 50 | 2 | 0.039 | 378 | 211 | 2.71 | 6.00 |
| 13 | 3 | 0 | 179 | 0 | 0 | 2 | 39 | 0 | 39 | 0 | 0.000 | 0 | 182 | 1.86 | 7.29 |
| 14 | 0 | 1 | 202 | 0 | 0 | 5 | 49 | 0 | 47 | 2 | 0.041 | 0 | 263 | 2.14 | 6.00 |
| 14 | 1 | 1 | 205 | 2 | 6 | 6 | 50 | 17 | 48 | 1 | 0.020 | 672 | 256 | 3.29 | 5.14 |
| 14 | 2 | 2 | 197 | 3 | 5 | 7 | 47 | 13 | 43 | 3 | 0.064 | 924 | 219 | 4.00 | 3.57 |
| 14 | 3 | 0 | 195 | 0 | 0 | 4 | 44 | 5 | 42 | 1 | 0.023 | 0 | 264 | 2.43 | 7.86 |
| 15 | 0 | 1 | 205 | 1 | 7 | 5 | 42 | 4 | 41 | 2 | 0.048 | 336 | 240 | 3.29 | 5.14 |
| 15 | 1 | 0 | 180 | 0 | 0 | 2 | 39 | 0 | 37 | 4 | 0.103 | 0 | 257 | 2.14 | 7.86 |
| 15 | 2 | 1 | 203 | 2 | 5 | 5 | 52 | 22 | 49 | 2 | 0.038 | 839 | 230 | 3.57 | 6.86 |
| 15 | 3 | 1 | 204 | 1 | 7 | 4 | 46 | 5 | 43 | 3 | 0.065 | 336 | 263 | 3.00 | 5.14 |
| 16 | 0 | 0 | 204 | 0 | 0 | 1 | 23 | 0 | 20 | 3 | 0.130 | 0 | 268 | 1.86 | 6.29 |
| 16 | 1 | 0 | 205 | 0 | 0 | 3 | 37 | 0 | 37 | 0 | 0.000 | 0 | 276 | 2.43 | 7.57 |
| 16 | 2 | 2 | 206 | 3 | 5 | 7 | 46 | 9 | 45 | 0 | 0.000 | 1006 | 227 | 4.00 | 4.14 |
| 16 | 3 | 2 | 197 | 0 | 0 | 7 | 43 | 4 | 40 | 2 | 0.047 | 126 | 272 | 2.86 | 3.29 |
| 17 | 0 | 0 | 203 | 0 | 0 | 2 | 32 | 0 | 31 | 6 | 0.188 | 0 | 238 | 2.00 | 8.71 |
| 17 | 1 | 1 | 203 | 1 | 7 | 5 | 47 | 5 | 44 | 3 | 0.064 | 336 | 269 | 3.00 | 5.43 |
| 17 | 2 | 0 | 173 | 0 | 0 | 4 | 47 | 0 | 43 | 4 | 0.085 | 0 | 274 | 2.29 | 7.86 |
| 17 | 3 | 0 | 209 | 0 | 0 | 2 | 31 | 0 | 31 | 2 | 0.065 | 0 | 258 | 2.14 | 8.14 |
| 18 | 0 | 0 | 206 | 0 | 0 | 2 | 31 | 0 | 29 | 3 | 0.097 | 0 | 230 | 2.00 | 7.86 |
| 18 | 1 | 2 | 207 | 3 | 5 | 7 | 53 | 15 | 50 | 4 | 0.075 | 1008 | 228 | 4.57 | 4.86 |
| 18 | 2 | 0 | 204 | 0 | 0 | 3 | 47 | 0 | 46 | 4 | 0.085 | 0 | 214 | 1.86 | 7.86 |
| 18 | 3 | 1 | 202 | 1 | 7 | 6 | 58 | 19 | 52 | 4 | 0.069 | 504 | 250 | 3.00 | 5.86 |
| 19 | 0 | 0 | 182 | 0 | 0 | 3 | 44 | 16 | 37 | 6 | 0.136 | 0 | 237 | 2.43 | 8.14 |
| 19 | 1 | 0 | 205 | 0 | 0 | 2 | 30 | 0 | 30 | 1 | 0.033 | 0 | 217 | 2.00 | 9.00 |
| 19 | 2 | 2 | 210 | 1 | 7 | 6 | 38 | 4 | 36 | 2 | 0.053 | 287 | 216 | 2.86 | 2.86 |
| 19 | 3 | 0 | 211 | 0 | 0 | 3 | 40 | 0 | 36 | 3 | 0.075 | 0 | 200 | 2.14 | 7.29 |
| 20 | 0 | 1 | 176 | 3 | 5 | 6 | 57 | 23 | 53 | 4 | 0.070 | 882 | 253 | 3.57 | 6.29 |
| 20 | 1 | 2 | 205 | 1 | 7 | 7 | 48 | 4 | 48 | 1 | 0.021 | 210 | 258 | 3.14 | 4.71 |
| 20 | 2 | 1 | 206 | 0 | 0 | 5 | 49 | 0 | 47 | 2 | 0.041 | 0 | 282 | 2.14 | 6.71 |
| 20 | 3 | 1 | 176 | 2 | 5 | 6 | 53 | 19 | 52 | 1 | 0.019 | 714 | 249 | 3.00 | 6.14 |
| 21 | 0 | 1 | 206 | 1 | 7 | 6 | 52 | 5 | 51 | 0 | 0.000 | 210 | 161 | 3.00 | 6.00 |
| 21 | 1 | 1 | 177 | 1 | 7 | 5 | 42 | 5 | 41 | 1 | 0.024 | 210 | 126 | 2.71 | 5.14 |
| 21 | 2 | 1 | 177 | 1 | 7 | 6 | 52 | 5 | 51 | 1 | 0.019 | 336 | 133 | 3.00 | 5.57 |
| 21 | 3 | 1 | 187 | 1 | 7 | 6 | 53 | 7 | 52 | 1 | 0.019 | 210 | 125 | 2.71 | 5.71 |
| 22 | 0 | 2 | 197 | 4 | 4 | 7 | 46 | 18 | 44 | 1 | 0.022 | 1301 | 77 | 4.57 | 3.71 |
| 22 | 1 | 1 | 193 | 1 | 7 | 5 | 46 | 12 | 45 | 4 | 0.087 | 336 | 74 | 3.00 | 6.43 |
| 22 | 2 | 2 | 197 | 3 | 5 | 7 | 50 | 11 | 49 | 1 | 0.020 | 960 | 52 | 4.57 | 4.71 |
| 22 | 3 | 1 | 195 | 2 | 6 | 5 | 42 | 10 | 40 | 1 | 0.024 | 577 | 7 | 3.57 | 4.71 |
| 23 | 0 | 1 | 201 | 0 | 0 | 6 | 57 | 0 | 54 | 1 | 0.018 | 0 | 256 | 2.71 | 6.57 |
| 23 | 1 | 0 | 199 | 0 | 0 | 1 | 23 | 0 | 22 | 1 | 0.043 | 0 | 257 | 1.57 | 8.57 |
| 23 | 2 | 1 | 172 | 0 | 0 | 6 | 59 | 6 | 55 | 4 | 0.068 | 0 | 288 | 2.43 | 6.43 |
| 23 | 3 | 2 | 204 | 3 | 5 | 7 | 57 | 19 | 53 | 2 | 0.035 | 1007 | 235 | 4.86 | 5.29 |
| 24 | 0 | 1 | 175 | 0 | 0 | 4 | 47 | 7 | 43 | 4 | 0.085 | 0 | 253 | 2.43 | 5.57 |
| 24 | 1 | 1 | 179 | 0 | 0 | 5 | 45 | 0 | 45 | 0 | 0.000 | 0 | 237 | 2.14 | 6.14 |
| 24 | 2 | 1 | 203 | 0 | 0 | 5 | 52 | 0 | 48 | 3 | 0.058 | 0 | 238 | 2.14 | 6.43 |
| 24 | 3 | 2 | 179 | 4 | 4 | 7 | 39 | 13 | 39 | 0 | 0.000 | 1512 | 216 | 5.43 | 3.14 |
| 25 | 0 | 0 | 204 | 0 | 0 | 4 | 51 | 7 | 45 | 7 | 0.137 | 0 | 241 | 2.71 | 7.29 |
| 25 | 1 | 0 | 205 | 1 | 7 | 3 | 40 | 9 | 38 | 3 | 0.075 | 294 | 237 | 3.00 | 8.14 |
| 25 | 2 | 1 | 179 | 2 | 6 | 6 | 67 | 16 | 58 | 8 | 0.119 | 420 | 224 | 3.00 | 7.00 |
| 25 | 3 | 2 | 205 | 2 | 6 | 7 | 49 | 11 | 48 | 4 | 0.082 | 671 | 213 | 4.00 | 4.57 |
| 26 | 0 | 0 | 179 | 0 | 0 | 3 | 38 | 0 | 37 | 1 | 0.026 | 0 | 257 | 1.57 | 7.43 |
| 26 | 1 | 2 | 205 | 3 | 5 | 7 | 43 | 8 | 41 | 0 | 0.000 | 1008 | 235 | 4.57 | 3.43 |
| 26 | 2 | 1 | 177 | 1 | 7 | 6 | 53 | 13 | 51 | 2 | 0.038 | 459 | 242 | 3.00 | 6.00 |
| 26 | 3 | 0 | 208 | 0 | 0 | 3 | 40 | 0 | 38 | 2 | 0.050 | 0 | 258 | 2.00 | 8.57 |
| 27 | 0 | 2 | 176 | 2 | 6 | 7 | 48 | 14 | 48 | 1 | 0.021 | 586 | 226 | 3.14 | 4.57 |
| 27 | 1 | 0 | 177 | 0 | 0 | 3 | 36 | 0 | 36 | 0 | 0.000 | 0 | 242 | 1.86 | 7.29 |
| 27 | 2 | 2 | 174 | 3 | 5 | 7 | 54 | 22 | 51 | 3 | 0.056 | 1007 | 231 | 4.00 | 5.00 |
| 27 | 3 | 2 | 207 | 3 | 5 | 7 | 46 | 15 | 45 | 1 | 0.022 | 1007 | 222 | 4.29 | 3.86 |
| 28 | 0 | 0 | 204 | 2 | 6 | 5 | 54 | 14 | 50 | 4 | 0.074 | 546 | 262 | 3.14 | 7.43 |
| 28 | 1 | 1 | 207 | 2 | 6 | 5 | 51 | 9 | 43 | 3 | 0.059 | 714 | 254 | 3.29 | 6.00 |
| 28 | 2 | 0 | 199 | 0 | 0 | 4 | 57 | 9 | 50 | 6 | 0.105 | 0 | 268 | 2.71 | 8.71 |
| 28 | 3 | 1 | 207 | 1 | 7 | 5 | 52 | 5 | 48 | 5 | 0.096 | 378 | 246 | 3.29 | 6.29 |
| 29 | 0 | 1 | 200 | 0 | 0 | 5 | 45 | 11 | 44 | 0 | 0.000 | 84 | 231 | 2.71 | 5.71 |
| 29 | 1 | 0 | 206 | 0 | 0 | 3 | 39 | 8 | 38 | 1 | 0.026 | 42 | 229 | 2.71 | 8.14 |
| 29 | 2 | 1 | 198 | 1 | 6 | 5 | 50 | 14 | 48 | 1 | 0.020 | 462 | 206 | 3.00 | 5.86 |
| 29 | 3 | 1 | 180 | 1 | 7 | 6 | 54 | 5 | 52 | 2 | 0.037 | 378 | 241 | 3.29 | 6.29 |
| 30 | 0 | 1 | 199 | 2 | 6 | 5 | 46 | 14 | 44 | 1 | 0.022 | 630 | 174 | 3.29 | 5.71 |
| 30 | 1 | 0 | 206 | 0 | 0 | 4 | 53 | 0 | 51 | 2 | 0.038 | 0 | 210 | 2.71 | 7.86 |
| 30 | 2 | 0 | 177 | 1 | 7 | 5 | 61 | 20 | 59 | 0 | 0.000 | 584 | 193 | 3.00 | 8.86 |
| 30 | 3 | 1 | 181 | 1 | 7 | 6 | 51 | 4 | 49 | 1 | 0.020 | 378 | 201 | 2.71 | 6.14 |
| 31 | 0 | 0 | 206 | 0 | 0 | 1 | 25 | 0 | 22 | 2 | 0.080 | 0 | 258 | 1.71 | 7.14 |
| 31 | 1 | 1 | 202 | 3 | 5 | 5 | 46 | 20 | 44 | 1 | 0.022 | 966 | 197 | 3.86 | 5.29 |
| 31 | 2 | 0 | 179 | 0 | 0 | 3 | 37 | 0 | 35 | 0 | 0.000 | 0 | 249 | 2.00 | 8.29 |
| 31 | 3 | 1 | 208 | 1 | 7 | 4 | 39 | 5 | 38 | 1 | 0.026 | 336 | 249 | 3.29 | 6.29 |

Severity-only R2 of the dark-night count: 0.594.

## Winters by severity

| Severity | Seed-winters | Dark nights (count: winters) | Nights out | First night out | Wood trips | Gathered | Re-target share | Cold member-hours | Food store at end | Night burn | Fall |
|---|---:|---|---|---|---|---|---|---|---|---|---|
| mild | 52 | {"0": 46, "1": 4, "2": 2} | 0 / 0 / 0.9 / 0 / 2 | 0 / 0 / 5.4 / 0 / 7 | 40 / 30.1 / 53 / 22 / 61 | 38 / 29.1 / 50 / 20 / 59 | 0.0736 / 0 / 0.13 / 0 / 0.188 | 0 / 0 / 281 / 0 / 630 | 249 / 214 / 274 / 182 / 293 | 2.29 / 1.73 / 2.84 / 1.57 / 3.43 | 8.14 / 7.29 / 8.84 / 6.29 / 9.29 |
| normal | 50 | {"0": 15, "1": 19, "2": 13, "3": 3} | 1 / 0 / 2 / 0 / 3 | 6 / 0 / 7 / 0 / 7 | 51 / 42 / 58.1 / 39 / 67 | 48 / 41 / 54 / 38 / 58 | 0.0377 / 0.0175 / 0.0853 / 0 / 0.119 | 336 / 0 / 722 / 0 / 1.01e+03 | 234 / 158 / 263 / 7 / 288 | 3 / 2.43 / 3.57 / 2.14 / 3.86 | 6 / 5.14 / 6.73 / 4.71 / 7.29 |
| harsh | 26 | {"0": 1, "1": 2, "2": 6, "3": 13, "4": 4} | 3 / 1.5 / 4 / 0 / 4 | 5 / 4 / 6 / 0 / 7 | 48 / 42 / 55.5 / 38 / 58 | 46 / 40.5 / 50.5 / 36 / 56 | 0.0408 / 0 / 0.0728 / 0 / 0.087 | 983 / 436 / 1.28e+03 / 126 / 1.51e+03 | 226 / 180 / 252 / 52 / 272 | 4.29 / 3.14 / 4.71 / 2.86 / 5.43 | 4.21 / 3.36 / 4.93 / 2.86 / 5.71 |
