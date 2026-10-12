# p8-4_round1_targets

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
| X2 | Y3_stagnation | PASS | failing seeds <= reference's + 1 (longest stagnant run <= 4 days) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y3_old_age_by_day30 | PASS | failing seeds <= reference's + 1 (>= 1 old-age death by day 30) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| X2 | Y3_replacement | PASS | failing seeds <= reference's + 1 (births - deaths over days 30..120 >= -3) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
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
| X2 | Y3_child_share | PASS | day-120 child share median in [0.03, 0.20] (W6) | {"max": 0.1951219512195122, "median": 0.10714285714285714, "min": 0.0, "n": 32, "p10": 0.047619047619047616, "p90": 0.17032520325203254} |
| X2 | Y3_elder_share | PASS | day-120 elder share median in [0.08, 0.35] (W6) | {"max": 0.38095238095238093, "median": 0.2619047619047619, "min": 0.17073170731707318, "n": 32, "p10": 0.21428571428571427, "p90": 0.3406504065040651} |
| X3 | a_r2 | PASS | severity-only R2 of the dark-night count <= 0.75 (undefined fails) | {"n": 128, "r2": 0.5300909842845327} |
| X3 | b_spread_within_severity | PASS | normal and harsh winters each: the most common dark-night count <= 0.75 of them | {"harsh": {"histogram": {"0": 1, "1": 4, "2": 4, "3": 13, "4": 4}, "mode_share": 0.5, "n": 26}, "normal": {"histogram": {"0": 17, "1": 20, "2": 9, "3": 3, "4": 1}, "mode_share": 0.4, "n": 50}} |
| X3 | c_no_dark_night | PASS | share of seed-winters with no dark night in [0.15, 0.60] | {"n": 128, "share": 0.484375} |
| X3 | d_two_or_more | PASS | share of seed-winters with >= 2 dark nights in [0.10, 0.50] | {"n": 128, "share": 0.28125} |
| X3 | e_first_night_spread | PASS | first dark night on >= 4 distinct nights, <= 0.50 of them on the seventh | {"distinct_first_nights": 4, "histogram": {"4": 5, "5": 18, "6": 15, "7": 28}, "n": 66, "seventh_share": 0.42424242424242425} |
| X3 | f_severity_shares | PASS | each severity's share of the seed-winters within +-0.10 of its weight | {"shares": {"harsh": 0.203125, "mild": 0.40625, "normal": 0.390625}, "weights": {"harsh": 250, "mild": 350, "normal": 400}} |
| X4 | retarget_share | PASS | winter wood_retargets / winter wood trips, median over seed-winters <= 0.10 | {"max": 0.19444444444444445, "median": 0.054563492063492064, "min": 0.0, "n": 128, "p10": 0.017153711279953243, "p90": 0.11876513317191283} |
| X4 | normal_gathered | PASS | median winter wood gathered over normal-severity seed-winters >= 40.5 | {"floor": 40.5, "max": 58.0, "median": 48.0, "min": 38.0, "n": 50, "p10": 43.0, "p90": 54.0} |
| X5 | Y4_store_share | PASS | store meals / all meals per seed-day from day 3, median >= 0.60 | {"max": 1.0, "median": 0.9896907216494846, "min": 0.6336633663366337, "n": 3776, "p10": 0.9489795918367347, "p90": 1.0} |
| X5 | Y4_every_seed | PASS | each seed's store share from day 3 >= 0.40 | {"max": 0.9952347334980586, "median": 0.9861915003609449, "min": 0.92429923227995, "n": 32, "p10": 0.9685273591720939, "p90": 0.9921878619040626} |
| X5 | Y4_winter | PASS | winter seed-day store share median >= 0.40 | {"max": 1.0, "median": 1.0, "min": 0.92, "n": 896, "p10": 0.9795918367346939, "p90": 1.0} |
| X5 | Y5_autumn_wood | PASS | autumn wood gathered per day median >= 1.5 x summer's (seed-days) | {"autumn": {"max": 146.0, "median": 50.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 131.0}, "summer": {"max": 50.0, "median": 0.0, "min": 0.0, "n": 928, "p10": 0.0, "p90": 36.0}} |
| X5 | Y5_autumn_work | PASS | autumn adult work share at 12:00 median >= v11's autumn value - 0.05 | {"floor": 0.40784549731918157, "max": 0.7746031746031746, "median": 0.6138793145372092, "min": 0.32291666666666663, "n": 32, "p10": 0.4661317567567567, "p90": 0.7240915285032933} |
| X5 | Y6_trip_days | PASS | days with wood trips per winter (of 7), median over seed-winters >= 4 | {"max": 7.0, "median": 5.0, "min": 1.0, "n": 128, "p10": 2.0, "p90": 7.0} |
| X5 | Y6_trips | PASS | wood trips per winter, median over seed-winters >= 20 | {"max": 71.0, "median": 48.0, "min": 21.0, "n": 128, "p10": 34.7, "p90": 56.0} |
| X5 | Y7_winter_lit_median | PASS | per seed share of winter nights lit at 02:00: median in [0.70, 0.97] | {"max": 0.9642857142857143, "median": 0.8571428571428571, "min": 0.6071428571428571, "n": 32, "p10": 0.7857142857142857, "p90": 0.9285714285714286} |
| X5 | Y7_other_lit_median | PASS | outside winter, 02:00 lit share: median >= 0.95 | {"max": 1.0, "median": 1.0, "min": 1.0, "n": 32, "p10": 1.0, "p90": 1.0} |
| X5 | Y8_winter_cold_median | PASS | winter cold members / members in the cold window: median <= 0.25 | {"max": 0.3833546189868029, "median": 0.1386153036367865, "min": 0.026848497762625188, "n": 32, "p10": 0.08129196901209926, "p90": 0.2214981129916078} |
| X5 | Y8_winter_food | PASS | food store at each winter's last periodic sample, median >= 0.8 x v11's | {"floor": 202.8, "max": 302.0, "median": 237.5, "min": 41.0, "n": 128, "p10": 183.7, "p90": 277.6} |
| X5 | Y10_night_in_shelter | PASS | median >= 0.80 | {"max": 1.0, "median": 1.0, "min": 0.9285714285714286, "n": 32, "p10": 0.9756678281068525, "p90": 1.0} |
| X5 | Y10_evening_near_hearth_median | PASS | evening near-hearth share median >= 0.55 | {"max": 0.8846153846153846, "median": 0.6483516483516484, "min": 0.3793103448275862, "n": 32, "p10": 0.42066259808195294, "p90": 0.7931034482758621} |
| X5 | Y11_home_distance_median | PASS | day-10 home distance median <= 8.1 | {"max": 11.285714285714286, "median": 7.619047619047619, "min": 4.785714285714286, "n": 64, "p10": 5.8238095238095235, "p90": 9.066666666666668} |
| X5 | Y11_t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.0047052996532937095, "median": 0.0011165070847096823, "min": 0.0, "p10": 0.0004952947003467063, "p90": 0.002484235582858368} |
| X5 | Y11_social_10_median | PASS | social completions per agent-day median >= 0.40 | {"max": 0.6935984154863784, "median": 0.6293320559036168, "min": 0.5634658813000177, "n": 64, "p10": 0.5899307383282918, "p90": 0.6613825227645975} |
| X5 | Y11_t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.013717559824738794, "median": 0.008531451258489866, "min": 0.005333333333333333, "p10": 0.00668403594781043, "p90": 0.012404399512737389} |
| X5 | Y11_social_120_median | PASS | social completions per agent-day median >= 0.40 | {"max": 0.8576458922175078, "median": 0.8151530364736453, "min": 0.7640944842941192, "n": 32, "p10": 0.7908853419935666, "p90": 0.8409386986612516} |
| X6 | receipt | FAIL | every Pass 7 intervention test passes and 8.1's Z6 for v14 | false |
| X7 | receipt | FAIL | schema 2 byte identity, hidden columns match the probe, v0 sample loads and Part 0.4 reproduced it, dataset v1 manifest verified | false |
| X8 | day10 | FAIL | median ms/agent-tick <= 1.2 x same-host game_v13, recorder off | {"limit": null, "max": 0.0011355443089634018, "median": 0.001098577076342628, "min": 0.0010585573514207047, "n": 64, "p10": 0.0010739658540777884, "p90": 0.0011197077852140506} |
| X8 | day120 | FAIL | median ms/agent-tick <= 1.2 x same-host game_v13, recorder off | {"limit": null, "max": 0.0015290070111685953, "median": 0.0013825446758759775, "min": 0.001293131112780546, "n": 32, "p10": 0.0013258410581192819, "p90": 0.0014947588589827145} |
| Y9 | work_1200 | FAIL | adult work share at 12:00 median >= 0.35 (spring-autumn) | {"max": 0.72, "median": 0.3077651515151515, "min": 0.037037037037037035, "n": 32, "p10": 0.16255259467040672, "p90": 0.4608391608391609} |
| Y9 | work_1500 | PASS | adult work share at 15:00 median >= 0.25 (spring-autumn) | {"max": 0.6666666666666666, "median": 0.41555977229601515, "min": 0.13333333333333333, "n": 32, "p10": 0.27730812013348166, "p90": 0.48640926640926646} |
| Y9 | idle_1200 | FAIL | idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"max": 0.8809523809523809, "median": 0.6666666666666666, "min": 0.34146341463414637, "n": 32, "p10": 0.5261904761904762, "p90": 0.7613240418118467} |

## Per-seed clauses

| Clause | Seeds | Failing seeds (value) |
|---|---:|---|
| Y3_one_settlement | 96 | none |
| Y3_birth_by_day4 | 64 | none |
| Y3_alive_day10 | 64 | none |
| Y3_stagnation | 32 | none |
| Y3_old_age_by_day30 | 32 | none |
| Y3_replacement | 32 | none |
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
| spring | 0.393 / 0.186 / 0.533 / 0.0323 / 0.74 | 0.603 / 0.452 / 0.743 / 0.319 / 0.894 | 0.455 / 0.241 / 0.571 / 0.0626 / 0.653 | 0.571 / 0.476 / 0.719 / 0.337 / 0.801 |
| summer | 0.0635 / 0.00303 / 0.281 / 0 / 0.556 | 0.786 / 0.626 / 0.874 / 0.488 / 0.905 | 0.4 / 0.303 / 0.518 / 0.207 / 0.71 | 0.548 / 0.475 / 0.667 / 0.357 / 0.762 |
| autumn | 0.614 / 0.466 / 0.724 / 0.323 / 0.775 | 0.404 / 0.299 / 0.499 / 0.238 / 0.643 | 0.366 / 0.159 / 0.458 / 0.123 / 0.639 | 0.589 / 0.476 / 0.714 / 0.286 / 0.75 |
| winter | 0.103 / 0.0319 / 0.254 / 0 / 0.283 | 0.806 / 0.727 / 0.893 / 0.643 / 0.905 | 0.211 / 0.0308 / 0.38 / 0 / 0.681 | 0.77 / 0.656 / 0.905 / 0.381 / 0.917 |

## Wood by season

| Season | Wood stock | Wood target | Gathered/day | Wood trips/day | Urgent trips/day | Lit at 02:00 | Cold member-hours/day | Food store | Forage trips/day |
|---|---|---|---|---|---|---:|---|---|---|
| spring | 26 / 13 / 44 / 0 / 83 | 39 / 39 / 39 / 39 / 39 | 29.5 / 0 / 50 / 0 / 68 | 15 / 0 / 28 / 0 / 41 | 0 / 0 / 19 / 0 / 40 | 1.000 | 0 / 0 / 0 / 0 / 0 | 179 / 67 / 235 / 0 / 269 | 61 / 15 / 87 / 0 / 110 |
| summer | 21 / 10 / 37 / 7 / 52 | 19.5 / 19.5 / 19.5 / 19.5 / 19.5 | 0 / 0 / 36 / 0 / 50 | 0 / 0 / 18 / 0 / 25 | 0 / 0 / 9 / 0 / 23 | 1.000 | 0 / 0 / 0 / 0 / 0 | 280 / 155 / 411 / 0 / 513 | 111 / 0 / 129 / 0 / 147 |
| autumn | 181 / 33 / 206 / 7 / 210 | 195 / 195 / 195 / 195 / 195 | 50 / 0 / 131 / 0 / 146 | 25 / 0 / 75 / 0 / 89 | 0 / 0 / 0 / 0 / 35 | 1.000 | 0 / 0 / 0 / 0 / 0 | 494 / 414 / 585 / 124 / 650 | 0 / 0 / 15 / 0 / 124 |
| winter | 104 / 0 / 192 / 0 / 210 | 156 / 104 / 208 / 104 / 208 | 5 / 0 / 20 / 0 / 22 | 5 / 0 / 21 / 0 / 27 | 0 / 0 / 5 / 0 / 10 | 0.857 | 0 / 0 / 334 / 0 / 336 | 319 / 212 / 530 / 19 / 639 | 0 / 0 / 25.5 / 0 / 86 |

## Winters

One row per complete seed-winter (Pass 7's columns). Night burn = mean `night_burn_per_hour` at the winter's 02:00 samples; fall = mean `winter_fall_today` at its seven 08:00 checkpoints.

| Seed | Year | Severity | Wood at start | Nights out | First night out | Trip-days | Wood trips | Urgent trips | Gathered | Re-targets | Re-target share | Cold member-hours | Food store at end | Night burn | Fall |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 0 | 1 | 183 | 3 | 5 | 6 | 53 | 17 | 51 | 2 | 0.038 | 840 | 225 | 3.00 | 5.86 |
| 0 | 1 | 0 | 203 | 0 | 0 | 1 | 23 | 0 | 22 | 0 | 0.000 | 0 | 228 | 2.14 | 7.57 |
| 0 | 2 | 2 | 201 | 3 | 5 | 7 | 53 | 19 | 49 | 4 | 0.075 | 1007 | 235 | 4.57 | 4.43 |
| 0 | 3 | 1 | 204 | 0 | 0 | 5 | 47 | 5 | 46 | 1 | 0.021 | 0 | 272 | 2.71 | 6.14 |
| 1 | 0 | 0 | 199 | 0 | 0 | 3 | 41 | 0 | 37 | 3 | 0.073 | 0 | 253 | 2.14 | 8.71 |
| 1 | 1 | 1 | 170 | 1 | 7 | 6 | 58 | 15 | 58 | 1 | 0.017 | 504 | 266 | 3.29 | 7.00 |
| 1 | 2 | 0 | 206 | 0 | 0 | 2 | 31 | 0 | 30 | 2 | 0.065 | 0 | 244 | 1.71 | 7.86 |
| 1 | 3 | 0 | 201 | 0 | 0 | 2 | 30 | 0 | 29 | 2 | 0.067 | 0 | 263 | 1.86 | 7.14 |
| 2 | 0 | 1 | 179 | 1 | 7 | 5 | 49 | 8 | 48 | 3 | 0.061 | 420 | 199 | 3.00 | 6.86 |
| 2 | 1 | 1 | 208 | 1 | 6 | 5 | 52 | 13 | 49 | 3 | 0.058 | 545 | 183 | 3.29 | 6.57 |
| 2 | 2 | 1 | 205 | 0 | 0 | 5 | 48 | 5 | 46 | 3 | 0.062 | 0 | 229 | 2.71 | 6.00 |
| 2 | 3 | 1 | 174 | 2 | 6 | 6 | 49 | 14 | 44 | 3 | 0.061 | 798 | 169 | 3.29 | 4.86 |
| 3 | 0 | 0 | 208 | 0 | 0 | 2 | 34 | 0 | 30 | 5 | 0.147 | 0 | 242 | 2.57 | 9.14 |
| 3 | 1 | 1 | 175 | 2 | 6 | 5 | 49 | 20 | 46 | 4 | 0.082 | 840 | 253 | 3.86 | 5.86 |
| 3 | 2 | 0 | 206 | 0 | 0 | 3 | 41 | 0 | 37 | 3 | 0.073 | 0 | 251 | 2.00 | 8.14 |
| 3 | 3 | 0 | 207 | 0 | 0 | 2 | 35 | 0 | 31 | 5 | 0.143 | 0 | 259 | 2.43 | 8.57 |
| 4 | 0 | 0 | 175 | 0 | 0 | 5 | 57 | 0 | 55 | 1 | 0.018 | 0 | 249 | 2.29 | 7.86 |
| 4 | 1 | 2 | 179 | 3 | 5 | 7 | 47 | 15 | 46 | 1 | 0.021 | 1176 | 244 | 4.29 | 4.29 |
| 4 | 2 | 0 | 177 | 0 | 0 | 4 | 50 | 0 | 48 | 1 | 0.020 | 0 | 269 | 1.71 | 9.00 |
| 4 | 3 | 0 | 208 | 0 | 0 | 3 | 42 | 10 | 41 | 2 | 0.048 | 0 | 277 | 2.71 | 8.71 |
| 5 | 0 | 1 | 180 | 0 | 0 | 5 | 52 | 0 | 48 | 6 | 0.115 | 0 | 192 | 2.43 | 6.57 |
| 5 | 1 | 2 | 203 | 2 | 6 | 7 | 43 | 11 | 41 | 2 | 0.047 | 671 | 184 | 4.00 | 3.71 |
| 5 | 2 | 0 | 205 | 0 | 0 | 3 | 42 | 10 | 40 | 2 | 0.048 | 0 | 184 | 2.71 | 9.29 |
| 5 | 3 | 2 | 176 | 4 | 4 | 7 | 49 | 17 | 43 | 4 | 0.082 | 1259 | 164 | 4.57 | 3.71 |
| 6 | 0 | 2 | 176 | 4 | 4 | 7 | 50 | 17 | 48 | 4 | 0.080 | 1305 | 159 | 4.57 | 4.43 |
| 6 | 1 | 1 | 208 | 1 | 7 | 4 | 41 | 5 | 39 | 1 | 0.024 | 336 | 232 | 3.00 | 6.29 |
| 6 | 2 | 1 | 177 | 3 | 5 | 6 | 48 | 13 | 47 | 0 | 0.000 | 920 | 185 | 3.29 | 5.57 |
| 6 | 3 | 1 | 204 | 0 | 0 | 5 | 46 | 0 | 46 | 0 | 0.000 | 0 | 232 | 2.71 | 6.29 |
| 7 | 0 | 0 | 205 | 0 | 0 | 4 | 50 | 19 | 47 | 3 | 0.060 | 0 | 241 | 2.71 | 8.14 |
| 7 | 1 | 0 | 203 | 0 | 0 | 3 | 53 | 0 | 50 | 3 | 0.057 | 0 | 298 | 2.71 | 9.00 |
| 7 | 2 | 2 | 206 | 1 | 7 | 7 | 52 | 10 | 49 | 3 | 0.058 | 416 | 236 | 3.14 | 4.71 |
| 7 | 3 | 0 | 201 | 2 | 6 | 4 | 49 | 16 | 45 | 4 | 0.082 | 546 | 267 | 3.43 | 8.43 |
| 8 | 0 | 0 | 197 | 0 | 0 | 4 | 47 | 0 | 46 | 0 | 0.000 | 0 | 228 | 2.14 | 8.43 |
| 8 | 1 | 2 | 208 | 2 | 6 | 6 | 46 | 10 | 46 | 0 | 0.000 | 672 | 262 | 3.71 | 4.71 |
| 8 | 2 | 0 | 175 | 0 | 0 | 3 | 40 | 6 | 39 | 0 | 0.000 | 166 | 260 | 2.57 | 8.14 |
| 8 | 3 | 0 | 179 | 0 | 0 | 2 | 41 | 0 | 40 | 3 | 0.073 | 0 | 302 | 1.57 | 8.29 |
| 9 | 0 | 1 | 177 | 1 | 7 | 6 | 50 | 13 | 49 | 1 | 0.020 | 462 | 238 | 2.71 | 5.43 |
| 9 | 1 | 1 | 174 | 4 | 4 | 6 | 53 | 24 | 51 | 2 | 0.038 | 1091 | 196 | 3.57 | 6.00 |
| 9 | 2 | 1 | 207 | 0 | 0 | 6 | 56 | 0 | 54 | 2 | 0.036 | 0 | 254 | 2.71 | 6.29 |
| 9 | 3 | 0 | 208 | 1 | 7 | 3 | 42 | 9 | 39 | 3 | 0.071 | 252 | 213 | 2.86 | 8.00 |
| 10 | 0 | 1 | 202 | 0 | 0 | 5 | 49 | 6 | 47 | 1 | 0.020 | 0 | 202 | 2.43 | 5.71 |
| 10 | 1 | 2 | 203 | 3 | 5 | 7 | 46 | 11 | 46 | 2 | 0.043 | 924 | 210 | 4.57 | 4.14 |
| 10 | 2 | 0 | 177 | 0 | 0 | 4 | 49 | 0 | 47 | 2 | 0.041 | 0 | 200 | 2.00 | 8.43 |
| 10 | 3 | 0 | 180 | 1 | 7 | 4 | 55 | 9 | 52 | 4 | 0.073 | 294 | 249 | 2.71 | 7.57 |
| 11 | 0 | 0 | 202 | 0 | 0 | 2 | 34 | 0 | 31 | 3 | 0.088 | 0 | 246 | 2.29 | 8.43 |
| 11 | 1 | 2 | 202 | 3 | 5 | 7 | 44 | 11 | 41 | 3 | 0.068 | 1008 | 197 | 4.57 | 3.57 |
| 11 | 2 | 0 | 207 | 0 | 0 | 3 | 39 | 0 | 35 | 2 | 0.051 | 0 | 246 | 2.00 | 7.14 |
| 11 | 3 | 0 | 200 | 0 | 0 | 2 | 33 | 0 | 32 | 0 | 0.000 | 0 | 237 | 2.29 | 8.29 |
| 12 | 0 | 0 | 177 | 0 | 0 | 4 | 44 | 14 | 41 | 4 | 0.091 | 84 | 260 | 2.43 | 7.43 |
| 12 | 1 | 2 | 208 | 3 | 5 | 6 | 58 | 18 | 50 | 7 | 0.121 | 924 | 244 | 4.86 | 5.71 |
| 12 | 2 | 2 | 178 | 3 | 5 | 7 | 54 | 15 | 47 | 4 | 0.074 | 1007 | 237 | 3.43 | 4.14 |
| 12 | 3 | 0 | 206 | 0 | 0 | 4 | 43 | 0 | 42 | 3 | 0.070 | 84 | 274 | 2.86 | 6.71 |
| 13 | 0 | 0 | 173 | 1 | 7 | 5 | 57 | 18 | 56 | 1 | 0.018 | 336 | 189 | 2.71 | 8.57 |
| 13 | 1 | 1 | 177 | 1 | 7 | 6 | 59 | 14 | 58 | 1 | 0.017 | 420 | 195 | 3.00 | 7.29 |
| 13 | 2 | 1 | 209 | 0 | 0 | 6 | 52 | 5 | 50 | 1 | 0.019 | 0 | 180 | 2.71 | 6.00 |
| 13 | 3 | 0 | 179 | 0 | 0 | 2 | 41 | 0 | 39 | 2 | 0.049 | 0 | 198 | 1.86 | 7.29 |
| 14 | 0 | 1 | 206 | 0 | 0 | 5 | 48 | 0 | 47 | 1 | 0.021 | 0 | 284 | 2.14 | 6.00 |
| 14 | 1 | 1 | 205 | 2 | 6 | 6 | 52 | 17 | 48 | 3 | 0.058 | 671 | 266 | 3.29 | 5.14 |
| 14 | 2 | 2 | 204 | 3 | 5 | 7 | 46 | 12 | 43 | 2 | 0.043 | 882 | 244 | 4.00 | 3.57 |
| 14 | 3 | 0 | 195 | 0 | 0 | 4 | 44 | 5 | 42 | 2 | 0.045 | 0 | 251 | 2.43 | 7.86 |
| 15 | 0 | 1 | 172 | 2 | 6 | 6 | 51 | 15 | 47 | 2 | 0.039 | 588 | 225 | 3.29 | 5.14 |
| 15 | 1 | 0 | 182 | 0 | 0 | 2 | 29 | 0 | 29 | 0 | 0.000 | 0 | 281 | 2.14 | 7.86 |
| 15 | 2 | 1 | 204 | 2 | 5 | 5 | 53 | 22 | 49 | 4 | 0.075 | 832 | 229 | 3.57 | 6.86 |
| 15 | 3 | 1 | 205 | 1 | 7 | 4 | 46 | 5 | 43 | 4 | 0.087 | 336 | 279 | 3.00 | 5.14 |
| 16 | 0 | 0 | 179 | 0 | 0 | 2 | 36 | 0 | 35 | 1 | 0.028 | 0 | 242 | 1.86 | 6.29 |
| 16 | 1 | 0 | 195 | 0 | 0 | 3 | 39 | 0 | 37 | 3 | 0.077 | 0 | 270 | 2.43 | 7.57 |
| 16 | 2 | 2 | 206 | 3 | 5 | 7 | 48 | 9 | 45 | 3 | 0.062 | 1007 | 224 | 4.00 | 4.14 |
| 16 | 3 | 2 | 208 | 0 | 0 | 6 | 42 | 0 | 36 | 5 | 0.119 | 84 | 285 | 2.86 | 3.29 |
| 17 | 0 | 0 | 203 | 0 | 0 | 2 | 32 | 0 | 31 | 6 | 0.188 | 0 | 227 | 2.00 | 8.71 |
| 17 | 1 | 1 | 208 | 1 | 7 | 5 | 50 | 5 | 44 | 5 | 0.100 | 252 | 276 | 3.00 | 5.43 |
| 17 | 2 | 0 | 201 | 0 | 0 | 3 | 38 | 0 | 35 | 5 | 0.132 | 0 | 248 | 2.29 | 7.86 |
| 17 | 3 | 0 | 173 | 0 | 0 | 3 | 44 | 0 | 39 | 7 | 0.159 | 0 | 265 | 2.14 | 8.14 |
| 18 | 0 | 0 | 203 | 0 | 0 | 2 | 30 | 0 | 29 | 1 | 0.033 | 0 | 228 | 2.00 | 7.86 |
| 18 | 1 | 2 | 201 | 3 | 5 | 7 | 52 | 21 | 50 | 4 | 0.077 | 1008 | 226 | 4.57 | 4.86 |
| 18 | 2 | 0 | 205 | 0 | 0 | 3 | 50 | 0 | 46 | 6 | 0.120 | 0 | 229 | 1.86 | 7.86 |
| 18 | 3 | 1 | 204 | 1 | 7 | 6 | 60 | 21 | 52 | 7 | 0.117 | 461 | 242 | 3.00 | 5.86 |
| 19 | 0 | 0 | 205 | 0 | 0 | 3 | 42 | 0 | 37 | 5 | 0.119 | 0 | 226 | 2.43 | 8.14 |
| 19 | 1 | 0 | 202 | 0 | 0 | 2 | 36 | 0 | 30 | 7 | 0.194 | 0 | 246 | 2.00 | 9.00 |
| 19 | 2 | 2 | 179 | 1 | 7 | 7 | 40 | 5 | 38 | 2 | 0.050 | 504 | 228 | 2.86 | 2.86 |
| 19 | 3 | 0 | 208 | 0 | 0 | 3 | 39 | 0 | 36 | 1 | 0.026 | 0 | 215 | 2.14 | 7.29 |
| 20 | 0 | 1 | 200 | 1 | 7 | 6 | 55 | 22 | 53 | 0 | 0.000 | 504 | 253 | 3.57 | 6.29 |
| 20 | 1 | 2 | 175 | 1 | 7 | 7 | 49 | 8 | 48 | 1 | 0.020 | 336 | 297 | 3.14 | 4.71 |
| 20 | 2 | 1 | 202 | 0 | 0 | 5 | 49 | 0 | 47 | 2 | 0.041 | 0 | 287 | 2.14 | 6.71 |
| 20 | 3 | 1 | 178 | 2 | 5 | 6 | 56 | 21 | 52 | 3 | 0.054 | 713 | 238 | 3.00 | 6.14 |
| 21 | 0 | 1 | 198 | 1 | 7 | 6 | 54 | 5 | 51 | 3 | 0.056 | 293 | 92 | 3.00 | 6.00 |
| 21 | 1 | 1 | 204 | 0 | 0 | 4 | 41 | 0 | 41 | 1 | 0.024 | 0 | 151 | 2.71 | 5.14 |
| 21 | 2 | 1 | 189 | 1 | 7 | 6 | 53 | 5 | 51 | 2 | 0.038 | 228 | 134 | 3.00 | 5.57 |
| 21 | 3 | 1 | 175 | 1 | 7 | 6 | 59 | 7 | 52 | 7 | 0.119 | 294 | 154 | 2.71 | 5.71 |
| 22 | 0 | 2 | 184 | 4 | 4 | 7 | 45 | 19 | 44 | 1 | 0.022 | 1338 | 51 | 4.57 | 3.71 |
| 22 | 1 | 1 | 171 | 2 | 6 | 5 | 45 | 12 | 45 | 1 | 0.022 | 628 | 53 | 3.00 | 6.43 |
| 22 | 2 | 2 | 169 | 3 | 5 | 7 | 52 | 11 | 49 | 3 | 0.058 | 1008 | 48 | 4.57 | 4.71 |
| 22 | 3 | 1 | 190 | 2 | 6 | 6 | 45 | 9 | 45 | 3 | 0.067 | 546 | 41 | 3.57 | 4.71 |
| 23 | 0 | 1 | 205 | 0 | 0 | 6 | 61 | 0 | 54 | 6 | 0.098 | 0 | 254 | 2.71 | 6.57 |
| 23 | 1 | 0 | 204 | 0 | 0 | 1 | 22 | 0 | 22 | 2 | 0.091 | 0 | 288 | 1.57 | 8.57 |
| 23 | 2 | 1 | 203 | 0 | 0 | 6 | 59 | 0 | 55 | 3 | 0.051 | 0 | 269 | 2.43 | 6.43 |
| 23 | 3 | 2 | 201 | 3 | 5 | 7 | 57 | 16 | 53 | 4 | 0.070 | 1007 | 232 | 4.86 | 5.29 |
| 24 | 0 | 1 | 177 | 0 | 0 | 4 | 45 | 7 | 43 | 5 | 0.111 | 0 | 282 | 2.43 | 5.57 |
| 24 | 1 | 1 | 176 | 0 | 0 | 5 | 45 | 0 | 45 | 0 | 0.000 | 0 | 282 | 2.14 | 6.14 |
| 24 | 2 | 1 | 172 | 0 | 0 | 6 | 54 | 0 | 53 | 1 | 0.019 | 0 | 271 | 2.14 | 6.43 |
| 24 | 3 | 2 | 203 | 4 | 4 | 7 | 40 | 10 | 39 | 1 | 0.025 | 1344 | 220 | 5.43 | 3.14 |
| 25 | 0 | 0 | 207 | 0 | 0 | 4 | 53 | 8 | 45 | 8 | 0.151 | 0 | 238 | 2.71 | 7.29 |
| 25 | 1 | 0 | 206 | 1 | 7 | 3 | 41 | 10 | 38 | 1 | 0.024 | 294 | 207 | 3.00 | 8.14 |
| 25 | 2 | 1 | 206 | 1 | 7 | 6 | 71 | 8 | 58 | 12 | 0.169 | 210 | 233 | 3.00 | 7.00 |
| 25 | 3 | 2 | 205 | 2 | 6 | 7 | 51 | 11 | 48 | 3 | 0.059 | 671 | 200 | 4.00 | 4.57 |
| 26 | 0 | 0 | 206 | 0 | 0 | 1 | 21 | 0 | 20 | 3 | 0.143 | 0 | 294 | 1.57 | 7.43 |
| 26 | 1 | 2 | 208 | 3 | 5 | 6 | 37 | 11 | 36 | 1 | 0.027 | 1008 | 240 | 4.57 | 3.43 |
| 26 | 2 | 1 | 179 | 1 | 7 | 6 | 54 | 15 | 51 | 3 | 0.056 | 420 | 293 | 3.00 | 6.00 |
| 26 | 3 | 0 | 204 | 0 | 0 | 4 | 52 | 0 | 47 | 4 | 0.077 | 0 | 263 | 2.00 | 8.57 |
| 27 | 0 | 2 | 181 | 1 | 7 | 7 | 48 | 14 | 48 | 0 | 0.000 | 500 | 220 | 3.14 | 4.57 |
| 27 | 1 | 0 | 177 | 0 | 0 | 3 | 37 | 0 | 36 | 1 | 0.027 | 0 | 263 | 1.86 | 7.29 |
| 27 | 2 | 2 | 208 | 2 | 6 | 6 | 50 | 17 | 46 | 4 | 0.080 | 840 | 232 | 4.00 | 5.00 |
| 27 | 3 | 2 | 207 | 3 | 5 | 7 | 45 | 15 | 45 | 1 | 0.022 | 988 | 229 | 4.29 | 3.86 |
| 28 | 0 | 0 | 203 | 2 | 6 | 5 | 55 | 16 | 50 | 6 | 0.109 | 586 | 208 | 3.14 | 7.43 |
| 28 | 1 | 1 | 200 | 2 | 6 | 5 | 49 | 9 | 43 | 4 | 0.082 | 756 | 251 | 3.29 | 6.00 |
| 28 | 2 | 0 | 199 | 0 | 0 | 4 | 56 | 9 | 50 | 4 | 0.071 | 0 | 258 | 2.71 | 8.71 |
| 28 | 3 | 1 | 210 | 1 | 7 | 4 | 42 | 5 | 40 | 4 | 0.095 | 420 | 268 | 3.29 | 6.29 |
| 29 | 0 | 1 | 203 | 0 | 0 | 5 | 48 | 7 | 44 | 3 | 0.062 | 0 | 239 | 2.71 | 5.71 |
| 29 | 1 | 0 | 197 | 1 | 7 | 3 | 38 | 14 | 38 | 1 | 0.026 | 210 | 242 | 2.71 | 8.14 |
| 29 | 2 | 1 | 205 | 0 | 0 | 5 | 50 | 15 | 48 | 2 | 0.040 | 336 | 218 | 3.00 | 5.86 |
| 29 | 3 | 1 | 205 | 1 | 7 | 6 | 52 | 5 | 52 | 0 | 0.000 | 336 | 240 | 3.29 | 6.29 |
| 30 | 0 | 1 | 208 | 1 | 6 | 5 | 46 | 14 | 44 | 2 | 0.043 | 588 | 202 | 3.29 | 5.71 |
| 30 | 1 | 0 | 210 | 0 | 0 | 4 | 44 | 0 | 42 | 2 | 0.045 | 0 | 231 | 2.71 | 7.86 |
| 30 | 2 | 0 | 176 | 1 | 7 | 5 | 61 | 18 | 59 | 3 | 0.049 | 588 | 211 | 3.00 | 8.86 |
| 30 | 3 | 1 | 181 | 1 | 7 | 6 | 50 | 4 | 49 | 2 | 0.040 | 336 | 195 | 2.71 | 6.14 |
| 31 | 0 | 0 | 201 | 0 | 0 | 1 | 22 | 0 | 22 | 2 | 0.091 | 0 | 217 | 1.71 | 7.14 |
| 31 | 1 | 1 | 207 | 3 | 5 | 5 | 46 | 19 | 44 | 2 | 0.043 | 924 | 199 | 3.86 | 5.29 |
| 31 | 2 | 0 | 206 | 0 | 0 | 2 | 29 | 0 | 28 | 1 | 0.034 | 0 | 231 | 2.00 | 8.29 |
| 31 | 3 | 1 | 210 | 1 | 7 | 4 | 42 | 5 | 38 | 1 | 0.024 | 336 | 253 | 3.29 | 6.29 |

Severity-only R2 of the dark-night count: 0.530.

## Winters by severity

| Severity | Seed-winters | Dark nights (count: winters) | Nights out | First night out | Wood trips | Gathered | Re-target share | Cold member-hours | Food store at end | Night burn | Fall |
|---|---:|---|---|---|---|---|---|---|---|---|---|
| mild | 52 | {"0": 44, "1": 6, "2": 2} | 0 / 0 / 1 / 0 / 2 | 0 / 0 / 6.9 / 0 / 7 | 41 / 29.1 / 54.8 / 21 / 61 | 38.5 / 29 / 50 / 20 / 59 | 0.0656 / 0.0175 / 0.143 / 0 / 0.194 | 0 / 0 / 290 / 0 / 588 | 246 / 208 / 277 / 184 / 302 | 2.29 / 1.73 / 2.84 / 1.57 / 3.43 | 8.14 / 7.29 / 8.84 / 6.29 / 9.29 |
| normal | 50 | {"0": 17, "1": 20, "2": 9, "3": 3, "4": 1} | 1 / 0 / 2 / 0 / 4 | 6 / 0 / 7 / 0 / 7 | 50 / 45 / 59 / 41 / 71 | 48 / 43 / 54 / 38 / 58 | 0.0421 / 0.0153 / 0.101 / 0 / 0.169 | 336 / 0 / 833 / 0 / 1.09e+03 | 232 / 154 / 279 / 41 / 293 | 3 / 2.43 / 3.57 / 2.14 / 3.86 | 6 / 5.14 / 6.73 / 4.71 / 7.29 |
| harsh | 26 | {"0": 1, "1": 4, "2": 4, "3": 13, "4": 4} | 3 / 1 / 4 / 0 / 4 | 5 / 4 / 7 / 0 / 7 | 48 / 41 / 53.5 / 37 / 58 | 46 / 38.5 / 49.5 / 36 / 53 | 0.0577 / 0.0208 / 0.0808 / 0 / 0.121 | 998 / 458 / 1.28e+03 / 84 / 1.34e+03 | 228 / 162 / 253 / 48 / 297 | 4.29 / 3.14 / 4.71 / 2.86 / 5.43 | 4.21 / 3.36 / 4.93 / 2.86 / 5.71 |
