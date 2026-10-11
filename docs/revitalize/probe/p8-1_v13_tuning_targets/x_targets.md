# p8-1_v13_tuning_targets

Values: median / p10 / p90 / min / max. Y9 is reported only. Severity weights (520, 280, 200) (summary.json rules block).

| Target | Clause | Verdict | Requirement | Observed |
|---|---|---|---|---|
| X1 | twins_10 | PASS | 64 seeds, determinism checked | 64 |
| X1 | twins_120 | PASS | 32 seeds, determinism checked | 32 |
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
| X2 | Y3_child_share | PASS | day-120 child share median in [0.03, 0.20] (W6) | {"max": 0.1951219512195122, "median": 0.10714285714285714, "min": 0.023809523809523808, "n": 32, "p10": 0.047619047619047616, "p90": 0.17032520325203254} |
| X2 | Y3_elder_share | PASS | day-120 elder share median in [0.08, 0.35] (W6) | {"max": 0.38095238095238093, "median": 0.2619047619047619, "min": 0.17073170731707318, "n": 32, "p10": 0.21428571428571427, "p90": 0.3333333333333333} |
| X3 | a_r2 | FAIL | severity-only R2 of the dark-night count <= 0.75 (undefined fails) | {"n": 128, "r2": 0.9429052399659941} |
| X3 | b_spread_within_severity | FAIL | normal and harsh winters each: the most common dark-night count <= 0.75 of them | {"harsh": {"histogram": {"2": 12, "3": 9}, "mode_share": 0.5714285714285714, "n": 21}, "normal": {"histogram": {"1": 36, "2": 1}, "mode_share": 0.972972972972973, "n": 37}} |
| X3 | c_no_dark_night | PASS | share of seed-winters with no dark night in [0.15, 0.60] | {"n": 128, "share": 0.546875} |
| X3 | d_two_or_more | PASS | share of seed-winters with >= 2 dark nights in [0.10, 0.50] | {"n": 128, "share": 0.171875} |
| X3 | e_first_night_spread | FAIL | first dark night on >= 4 distinct nights, <= 0.50 of them on the seventh | {"distinct_first_nights": 3, "histogram": {"5": 9, "6": 13, "7": 36}, "n": 58, "seventh_share": 0.6206896551724138} |
| X3 | f_severity_shares | PASS | each severity's share of the seed-winters within +-0.10 of its weight | {"shares": {"harsh": 0.1640625, "mild": 0.546875, "normal": 0.2890625}, "weights": {"harsh": 200, "mild": 520, "normal": 280}} |
| X4 | retarget_share | PASS | winter wood_retargets / winter wood trips, median over seed-winters <= 0.10 | {"max": 0.18518518518518517, "median": 0.05, "min": 0.0, "n": 128, "p10": 0.0, "p90": 0.10256410256410256} |
| X4 | normal_gathered | PASS | median winter wood gathered over normal-severity seed-winters >= 40.5 | {"floor": 40.5, "max": 51.0, "median": 45.0, "min": 45.0, "n": 37, "p10": 45.0, "p90": 51.0} |
| X5 | Y4_store_share | PASS | store meals / all meals per seed-day from day 3, median >= 0.60 | {"max": 1.0, "median": 0.9896907216494846, "min": 0.6666666666666666, "n": 3776, "p10": 0.9494949494949495, "p90": 1.0} |
| X5 | Y4_every_seed | PASS | each seed's store share from day 3 >= 0.40 | {"max": 0.9955818679862154, "median": 0.9864724282118317, "min": 0.9365779057214826, "n": 32, "p10": 0.969400946643086, "p90": 0.992731500731} |
| X5 | Y4_winter | PASS | winter seed-day store share median >= 0.40 | {"max": 1.0, "median": 1.0, "min": 0.9306930693069307, "n": 896, "p10": 0.98, "p90": 1.0} |
| X5 | Y5_autumn_wood | PASS | autumn wood gathered per day median >= 1.5 x summer's (seed-days) | {"autumn": {"max": 147.0, "median": 48.5, "min": 0.0, "n": 896, "p10": 0.0, "p90": 130.0}, "summer": {"max": 52.0, "median": 0.0, "min": 0.0, "n": 928, "p10": 0.0, "p90": 36.0}} |
| X5 | Y5_autumn_work | PASS | autumn adult work share at 12:00 median >= v11's autumn value - 0.05 | {"floor": 0.40784549731918157, "max": 0.8024642289348172, "median": 0.6436293436293437, "min": 0.2038123167155425, "n": 32, "p10": 0.4722169722169722, "p90": 0.7329084008970157} |
| X5 | Y6_trip_days | PASS | days with wood trips per winter (of 7), median over seed-winters >= 4 | {"max": 7.0, "median": 4.0, "min": 2.0, "n": 128, "p10": 3.0, "p90": 7.0} |
| X5 | Y6_trips | PASS | wood trips per winter, median over seed-winters >= 20 | {"max": 56.0, "median": 46.0, "min": 31.0, "n": 128, "p10": 38.0, "p90": 52.0} |
| X5 | Y7_winter_lit_median | PASS | per seed share of winter nights lit at 02:00: median in [0.70, 0.97] | {"max": 1.0, "median": 0.8928571428571429, "min": 0.7142857142857143, "n": 32, "p10": 0.825, "p90": 0.9642857142857143} |
| X5 | Y7_other_lit_median | PASS | outside winter, 02:00 lit share: median >= 0.95 | {"max": 1.0, "median": 1.0, "min": 1.0, "n": 32, "p10": 1.0, "p90": 1.0} |
| X5 | Y8_winter_cold_median | PASS | winter cold members / members in the cold window: median <= 0.25 | {"max": 0.3013025838137946, "median": 0.10284975784948577, "min": 0.0, "n": 32, "p10": 0.03576982053263656, "p90": 0.1866848798607451} |
| X5 | Y8_winter_food | PASS | food store at each winter's last periodic sample, median >= 0.8 x v11's | {"floor": 202.8, "max": 294.0, "median": 242.5, "min": 48.0, "n": 128, "p10": 186.1, "p90": 276.3} |
| X5 | Y10_night_in_shelter | PASS | median >= 0.80 | {"max": 1.0, "median": 1.0, "min": 0.9285714285714286, "n": 32, "p10": 0.9547619047619047, "p90": 1.0} |
| X5 | Y10_evening_near_hearth_median | PASS | evening near-hearth share median >= 0.55 | {"max": 0.8518518518518519, "median": 0.6428571428571429, "min": 0.35294117647058826, "n": 32, "p10": 0.41565703634669154, "p90": 0.790625} |
| X5 | Y11_home_distance_median | PASS | day-10 home distance median <= 8.1 | {"max": 11.285714285714286, "median": 7.619047619047619, "min": 4.785714285714286, "n": 64, "p10": 5.8238095238095235, "p90": 9.066666666666668} |
| X5 | Y11_t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.0047052996532937095, "median": 0.0011165070847096823, "min": 0.0, "p10": 0.0004952947003467063, "p90": 0.002484235582858368} |
| X5 | Y11_social_10_median | PASS | social completions per agent-day median >= 0.40 | {"max": 0.6935984154863784, "median": 0.6293320559036168, "min": 0.5634658813000177, "n": 64, "p10": 0.5899307383282918, "p90": 0.6613825227645975} |
| X5 | Y11_t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.013953488372093023, "median": 0.008588478819857893, "min": 0.005494694777456249, "p10": 0.00655793589440792, "p90": 0.01249411073539241} |
| X5 | Y11_social_120_median | PASS | social completions per agent-day median >= 0.40 | {"max": 0.8580506330821691, "median": 0.8151147562296444, "min": 0.7681493668737271, "n": 32, "p10": 0.7912702820601144, "p90": 0.8413026398497303} |
| X6 | receipt | FAIL | every Pass 7 intervention test passes and 8.1's Z6 for v14 | false |
| X7 | receipt | FAIL | schema 2 byte identity, hidden columns match the probe, v0 sample loads and Part 0.4 reproduced it, dataset v1 manifest verified | false |
| X8 | day10 | FAIL | median ms/agent-tick <= 1.2 x same-host game_v13, recorder off | {"limit": null, "max": 0.001555608827351957, "median": 0.0011111650105368035, "min": 0.0010498628852806428, "n": 64, "p10": 0.0010660945008329568, "p90": 0.0011397163323035252} |
| X8 | day120 | FAIL | median ms/agent-tick <= 1.2 x same-host game_v13, recorder off | {"limit": null, "max": 0.001712334614724795, "median": 0.0013830672315313188, "min": 0.0012374104190842603, "n": 32, "p10": 0.001315987126492762, "p90": 0.001568363778005729} |
| Y9 | work_1200 | FAIL | adult work share at 12:00 median >= 0.35 (spring-autumn) | {"max": 0.7407407407407407, "median": 0.32291666666666663, "min": 0.041666666666666664, "n": 32, "p10": 0.16724137931034483, "p90": 0.4434482758620691} |
| Y9 | work_1500 | PASS | adult work share at 15:00 median >= 0.25 (spring-autumn) | {"max": 0.68, "median": 0.40900735294117646, "min": 0.125, "n": 32, "p10": 0.3, "p90": 0.5} |
| Y9 | idle_1200 | FAIL | idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"max": 0.9, "median": 0.6547619047619048, "min": 0.325, "n": 32, "p10": 0.55, "p90": 0.7380952380952381} |

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
| spring | 0.392 / 0.221 / 0.536 / 0.0426 / 0.75 | 0.581 / 0.473 / 0.737 / 0.316 / 0.901 | 0.421 / 0.269 / 0.557 / 0.0833 / 0.686 | 0.587 / 0.476 / 0.713 / 0.342 / 0.786 |
| summer | 0.069 / 0.0295 / 0.259 / 0 / 0.5 | 0.771 / 0.629 / 0.87 / 0.462 / 0.929 | 0.397 / 0.333 / 0.533 / 0.226 / 0.692 | 0.549 / 0.455 / 0.664 / 0.359 / 0.69 |
| autumn | 0.644 / 0.472 / 0.733 / 0.204 / 0.802 | 0.369 / 0.311 / 0.5 / 0.229 / 0.667 | 0.388 / 0.194 / 0.475 / 0.0739 / 0.649 | 0.56 / 0.477 / 0.737 / 0.25 / 0.762 |
| winter | 0.099 / 0.0397 / 0.242 / 0.0312 / 0.307 | 0.815 / 0.738 / 0.881 / 0.619 / 0.905 | 0.036 / 0 / 0.365 / 0 / 0.661 | 0.845 / 0.658 / 0.905 / 0.405 / 0.928 |

## Wood by season

| Season | Wood stock | Wood target | Gathered/day | Wood trips/day | Urgent trips/day | Lit at 02:00 | Cold member-hours/day | Food store | Forage trips/day |
|---|---|---|---|---|---|---:|---|---|---|
| spring | 28 / 15 / 47.3 / 0 / 63 | 39 / 39 / 39 / 39 / 39 | 28 / 0 / 48 / 0 / 68 | 14 / 0 / 26 / 0 / 40 | 0 / 0 / 17 / 0 / 40 | 1.000 | 0 / 0 / 0 / 0 / 0 | 187 / 63 / 233 / 0 / 271 | 56 / 14.9 / 86 / 0 / 106 |
| summer | 21 / 10 / 38.1 / 7 / 52 | 19.5 / 19.5 / 19.5 / 19.5 / 19.5 | 0 / 0 / 36 / 0 / 52 | 0 / 0 / 18 / 0 / 26 | 0 / 0 / 9.3 / 0 / 23 | 1.000 | 0 / 0 / 0 / 0 / 0 | 281 / 158 / 408 / 0 / 510 | 112 / 0 / 129 / 0 / 142 |
| autumn | 181 / 34 / 206 / 7 / 212 | 195 / 195 / 195 / 195 / 195 | 48.5 / 0 / 130 / 0 / 147 | 25 / 0 / 74 / 0 / 90 | 0 / 0 / 0 / 0 / 30 | 1.000 | 0 / 0 / 0 / 0 / 0 | 496 / 422 / 578 / 118 / 640 | 0 / 0 / 5.5 / 0 / 130 |
| winter | 106 / 22 / 196 / 0 / 212 | 104 / 104 / 208 / 104 / 208 | 5 / 0 / 20 / 0 / 22 | 5 / 0 / 20 / 0 / 28 | 0 / 0 / 4 / 0 / 7 | 0.901 | 0 / 0 / 168 / 0 / 336 | 320 / 216 / 537 / 25 / 649 | 0 / 0 / 22 / 0 / 85 |

## Winters

One row per complete seed-winter (Pass 7's columns). Night burn = mean `night_burn_per_hour` at the winter's 02:00 samples; fall = mean `winter_fall_today` at its seven 08:00 checkpoints.

| Seed | Year | Severity | Wood at start | Nights out | First night out | Trip-days | Wood trips | Urgent trips | Gathered | Re-targets | Re-target share | Cold member-hours | Food store at end | Night burn | Fall |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 0 | 0 | 183 | 0 | 0 | 3 | 38 | 0 | 38 | 1 | 0.026 | 0 | 259 | 2.00 | 8.00 |
| 0 | 1 | 0 | 202 | 0 | 0 | 3 | 39 | 0 | 38 | 1 | 0.026 | 0 | 245 | 2.00 | 8.00 |
| 0 | 2 | 1 | 176 | 1 | 7 | 6 | 53 | 13 | 51 | 1 | 0.019 | 504 | 253 | 3.00 | 6.00 |
| 0 | 3 | 0 | 200 | 0 | 0 | 3 | 38 | 0 | 38 | 0 | 0.000 | 0 | 270 | 2.00 | 8.00 |
| 1 | 0 | 0 | 199 | 0 | 0 | 3 | 41 | 0 | 38 | 4 | 0.098 | 0 | 251 | 2.00 | 8.00 |
| 1 | 1 | 1 | 178 | 1 | 7 | 6 | 55 | 14 | 51 | 3 | 0.055 | 462 | 250 | 3.00 | 6.00 |
| 1 | 2 | 0 | 199 | 0 | 0 | 3 | 39 | 0 | 38 | 4 | 0.103 | 0 | 248 | 2.00 | 8.00 |
| 1 | 3 | 0 | 206 | 0 | 0 | 3 | 39 | 0 | 38 | 2 | 0.051 | 0 | 271 | 2.00 | 8.00 |
| 2 | 0 | 1 | 179 | 1 | 7 | 6 | 54 | 12 | 51 | 3 | 0.056 | 462 | 217 | 3.00 | 6.00 |
| 2 | 1 | 1 | 179 | 1 | 7 | 6 | 56 | 12 | 51 | 4 | 0.071 | 459 | 193 | 3.00 | 6.00 |
| 2 | 2 | 1 | 181 | 1 | 7 | 6 | 56 | 13 | 51 | 5 | 0.089 | 420 | 228 | 3.00 | 6.00 |
| 2 | 3 | 0 | 200 | 0 | 0 | 3 | 40 | 0 | 38 | 2 | 0.050 | 0 | 197 | 2.00 | 8.00 |
| 3 | 0 | 0 | 208 | 0 | 0 | 2 | 34 | 0 | 30 | 5 | 0.147 | 0 | 241 | 2.00 | 8.00 |
| 3 | 1 | 1 | 206 | 1 | 7 | 5 | 47 | 6 | 45 | 3 | 0.064 | 294 | 272 | 3.00 | 6.00 |
| 3 | 2 | 0 | 201 | 0 | 0 | 3 | 40 | 0 | 38 | 3 | 0.075 | 0 | 242 | 2.00 | 8.00 |
| 3 | 3 | 0 | 208 | 0 | 0 | 2 | 33 | 0 | 30 | 3 | 0.091 | 0 | 268 | 2.00 | 8.00 |
| 4 | 0 | 0 | 175 | 0 | 0 | 4 | 48 | 0 | 46 | 1 | 0.021 | 0 | 249 | 2.00 | 8.00 |
| 4 | 1 | 2 | 205 | 2 | 6 | 7 | 47 | 13 | 45 | 2 | 0.043 | 840 | 265 | 4.00 | 4.00 |
| 4 | 2 | 0 | 176 | 0 | 0 | 4 | 48 | 0 | 46 | 1 | 0.021 | 0 | 262 | 2.00 | 8.00 |
| 4 | 3 | 0 | 203 | 0 | 0 | 3 | 39 | 0 | 38 | 4 | 0.103 | 0 | 255 | 2.00 | 8.00 |
| 5 | 0 | 0 | 180 | 0 | 0 | 4 | 47 | 0 | 46 | 3 | 0.064 | 0 | 187 | 2.00 | 8.00 |
| 5 | 1 | 2 | 206 | 2 | 6 | 7 | 47 | 13 | 45 | 4 | 0.085 | 839 | 184 | 4.00 | 4.00 |
| 5 | 2 | 0 | 195 | 0 | 0 | 3 | 39 | 0 | 38 | 0 | 0.000 | 0 | 204 | 2.00 | 8.00 |
| 5 | 3 | 2 | 205 | 2 | 6 | 7 | 48 | 12 | 45 | 2 | 0.042 | 840 | 189 | 4.00 | 4.00 |
| 6 | 0 | 2 | 176 | 3 | 5 | 7 | 47 | 16 | 45 | 4 | 0.085 | 969 | 175 | 4.00 | 4.00 |
| 6 | 1 | 0 | 181 | 0 | 0 | 4 | 48 | 0 | 46 | 2 | 0.042 | 0 | 234 | 2.00 | 8.00 |
| 6 | 2 | 0 | 202 | 0 | 0 | 3 | 40 | 0 | 38 | 4 | 0.100 | 0 | 224 | 2.00 | 8.00 |
| 6 | 3 | 1 | 179 | 1 | 7 | 6 | 55 | 12 | 51 | 4 | 0.073 | 462 | 236 | 3.00 | 6.00 |
| 7 | 0 | 0 | 205 | 0 | 0 | 3 | 41 | 0 | 38 | 3 | 0.073 | 0 | 242 | 2.00 | 8.00 |
| 7 | 1 | 0 | 204 | 0 | 0 | 3 | 41 | 0 | 38 | 6 | 0.146 | 0 | 277 | 2.00 | 8.00 |
| 7 | 2 | 2 | 202 | 3 | 5 | 7 | 47 | 12 | 45 | 3 | 0.064 | 867 | 238 | 4.00 | 4.00 |
| 7 | 3 | 0 | 200 | 0 | 0 | 3 | 39 | 0 | 38 | 3 | 0.077 | 0 | 271 | 2.00 | 8.00 |
| 8 | 0 | 0 | 197 | 0 | 0 | 3 | 38 | 0 | 38 | 1 | 0.026 | 0 | 226 | 2.00 | 8.00 |
| 8 | 1 | 2 | 177 | 3 | 5 | 7 | 45 | 15 | 45 | 0 | 0.000 | 1008 | 267 | 4.00 | 4.00 |
| 8 | 2 | 0 | 203 | 0 | 0 | 3 | 38 | 0 | 38 | 0 | 0.000 | 0 | 255 | 2.00 | 8.00 |
| 8 | 3 | 0 | 201 | 0 | 0 | 3 | 40 | 0 | 38 | 1 | 0.025 | 0 | 294 | 2.00 | 8.00 |
| 9 | 0 | 0 | 177 | 0 | 0 | 4 | 49 | 0 | 46 | 2 | 0.041 | 0 | 258 | 2.00 | 8.00 |
| 9 | 1 | 1 | 200 | 1 | 7 | 5 | 47 | 12 | 45 | 1 | 0.021 | 336 | 246 | 3.00 | 6.00 |
| 9 | 2 | 1 | 207 | 1 | 7 | 5 | 46 | 6 | 45 | 1 | 0.022 | 252 | 246 | 3.00 | 6.00 |
| 9 | 3 | 0 | 204 | 0 | 0 | 3 | 40 | 0 | 38 | 3 | 0.075 | 0 | 232 | 2.00 | 8.00 |
| 10 | 0 | 1 | 202 | 1 | 7 | 5 | 48 | 6 | 45 | 3 | 0.062 | 335 | 201 | 3.00 | 6.00 |
| 10 | 1 | 2 | 176 | 3 | 5 | 7 | 46 | 15 | 45 | 1 | 0.022 | 1008 | 214 | 4.00 | 4.00 |
| 10 | 2 | 0 | 177 | 0 | 0 | 4 | 47 | 0 | 46 | 2 | 0.043 | 0 | 206 | 2.00 | 8.00 |
| 10 | 3 | 0 | 208 | 0 | 0 | 2 | 31 | 0 | 30 | 2 | 0.065 | 0 | 230 | 2.00 | 8.00 |
| 11 | 0 | 0 | 202 | 0 | 0 | 3 | 38 | 0 | 38 | 1 | 0.026 | 0 | 245 | 2.00 | 8.00 |
| 11 | 1 | 2 | 193 | 3 | 5 | 7 | 47 | 12 | 45 | 1 | 0.021 | 966 | 236 | 4.00 | 4.00 |
| 11 | 2 | 0 | 203 | 0 | 0 | 3 | 38 | 0 | 38 | 1 | 0.026 | 0 | 262 | 2.00 | 8.00 |
| 11 | 3 | 0 | 207 | 0 | 0 | 3 | 38 | 0 | 38 | 1 | 0.026 | 0 | 262 | 2.00 | 8.00 |
| 12 | 0 | 0 | 177 | 0 | 0 | 4 | 49 | 0 | 46 | 5 | 0.102 | 0 | 262 | 2.00 | 8.00 |
| 12 | 1 | 2 | 200 | 3 | 5 | 7 | 52 | 12 | 45 | 5 | 0.096 | 882 | 250 | 4.00 | 4.00 |
| 12 | 2 | 2 | 203 | 2 | 6 | 7 | 50 | 13 | 45 | 5 | 0.100 | 840 | 246 | 4.00 | 4.00 |
| 12 | 3 | 0 | 205 | 0 | 0 | 3 | 38 | 0 | 38 | 3 | 0.079 | 0 | 280 | 2.00 | 8.00 |
| 13 | 0 | 0 | 173 | 0 | 0 | 4 | 50 | 0 | 46 | 4 | 0.080 | 0 | 183 | 2.00 | 8.00 |
| 13 | 1 | 0 | 206 | 0 | 0 | 3 | 39 | 0 | 38 | 0 | 0.000 | 0 | 193 | 2.00 | 8.00 |
| 13 | 2 | 1 | 204 | 1 | 7 | 5 | 46 | 6 | 45 | 1 | 0.022 | 294 | 178 | 3.00 | 6.00 |
| 13 | 3 | 0 | 180 | 0 | 0 | 4 | 47 | 0 | 46 | 1 | 0.021 | 0 | 193 | 2.00 | 8.00 |
| 14 | 0 | 0 | 206 | 0 | 0 | 3 | 40 | 0 | 38 | 4 | 0.100 | 0 | 284 | 2.00 | 8.00 |
| 14 | 1 | 1 | 206 | 1 | 7 | 5 | 49 | 7 | 45 | 4 | 0.082 | 293 | 285 | 3.00 | 6.00 |
| 14 | 2 | 2 | 204 | 2 | 6 | 7 | 50 | 13 | 45 | 3 | 0.060 | 840 | 236 | 4.00 | 4.00 |
| 14 | 3 | 0 | 205 | 0 | 0 | 3 | 40 | 0 | 38 | 0 | 0.000 | 0 | 253 | 2.00 | 8.00 |
| 15 | 0 | 0 | 172 | 0 | 0 | 4 | 48 | 0 | 46 | 4 | 0.083 | 0 | 234 | 2.00 | 8.00 |
| 15 | 1 | 0 | 180 | 0 | 0 | 4 | 48 | 0 | 46 | 5 | 0.104 | 0 | 268 | 2.00 | 8.00 |
| 15 | 2 | 1 | 178 | 1 | 7 | 6 | 53 | 12 | 51 | 2 | 0.038 | 462 | 258 | 3.00 | 6.00 |
| 15 | 3 | 1 | 185 | 1 | 7 | 6 | 52 | 12 | 51 | 0 | 0.000 | 378 | 279 | 3.00 | 6.00 |
| 16 | 0 | 0 | 179 | 0 | 0 | 4 | 48 | 0 | 46 | 0 | 0.000 | 0 | 242 | 2.00 | 8.00 |
| 16 | 1 | 0 | 202 | 0 | 0 | 3 | 38 | 0 | 38 | 2 | 0.053 | 0 | 290 | 2.00 | 8.00 |
| 16 | 2 | 2 | 204 | 2 | 6 | 7 | 46 | 13 | 45 | 2 | 0.043 | 839 | 227 | 4.00 | 4.00 |
| 16 | 3 | 2 | 205 | 2 | 6 | 7 | 45 | 12 | 45 | 1 | 0.022 | 840 | 264 | 4.00 | 4.00 |
| 17 | 0 | 0 | 203 | 0 | 0 | 3 | 40 | 0 | 38 | 3 | 0.075 | 0 | 227 | 2.00 | 8.00 |
| 17 | 1 | 0 | 177 | 0 | 0 | 4 | 49 | 0 | 46 | 2 | 0.041 | 0 | 276 | 2.00 | 8.00 |
| 17 | 2 | 0 | 206 | 0 | 0 | 3 | 41 | 0 | 38 | 3 | 0.073 | 0 | 273 | 2.00 | 8.00 |
| 17 | 3 | 0 | 204 | 0 | 0 | 3 | 42 | 0 | 38 | 5 | 0.119 | 0 | 269 | 2.00 | 8.00 |
| 18 | 0 | 0 | 203 | 0 | 0 | 3 | 40 | 0 | 38 | 3 | 0.075 | 0 | 232 | 2.00 | 8.00 |
| 18 | 1 | 1 | 206 | 1 | 7 | 5 | 54 | 6 | 45 | 10 | 0.185 | 294 | 262 | 3.00 | 6.00 |
| 18 | 2 | 0 | 199 | 0 | 0 | 3 | 43 | 0 | 38 | 3 | 0.070 | 0 | 234 | 2.00 | 8.00 |
| 18 | 3 | 0 | 203 | 0 | 0 | 3 | 44 | 0 | 38 | 5 | 0.114 | 0 | 260 | 2.00 | 8.00 |
| 19 | 0 | 0 | 205 | 0 | 0 | 3 | 39 | 0 | 38 | 3 | 0.077 | 0 | 224 | 2.00 | 8.00 |
| 19 | 1 | 0 | 206 | 0 | 0 | 3 | 42 | 0 | 38 | 2 | 0.048 | 0 | 247 | 2.00 | 8.00 |
| 19 | 2 | 2 | 206 | 2 | 6 | 7 | 47 | 12 | 45 | 3 | 0.064 | 840 | 206 | 4.00 | 4.00 |
| 19 | 3 | 0 | 212 | 0 | 0 | 2 | 33 | 0 | 30 | 2 | 0.061 | 0 | 226 | 2.00 | 8.00 |
| 20 | 0 | 1 | 200 | 1 | 7 | 5 | 46 | 12 | 45 | 1 | 0.022 | 336 | 260 | 3.00 | 6.00 |
| 20 | 1 | 2 | 206 | 2 | 6 | 7 | 45 | 12 | 45 | 0 | 0.000 | 840 | 263 | 4.00 | 4.00 |
| 20 | 2 | 1 | 198 | 1 | 7 | 5 | 45 | 11 | 45 | 0 | 0.000 | 336 | 283 | 3.00 | 6.00 |
| 20 | 3 | 1 | 202 | 1 | 7 | 5 | 48 | 6 | 45 | 1 | 0.021 | 336 | 268 | 3.00 | 6.00 |
| 21 | 0 | 0 | 198 | 0 | 0 | 3 | 39 | 0 | 38 | 1 | 0.026 | 0 | 91 | 2.00 | 8.00 |
| 21 | 1 | 1 | 176 | 1 | 7 | 6 | 54 | 12 | 51 | 2 | 0.037 | 503 | 105 | 3.00 | 6.00 |
| 21 | 2 | 1 | 169 | 2 | 6 | 6 | 52 | 13 | 51 | 1 | 0.019 | 557 | 86 | 3.00 | 6.00 |
| 21 | 3 | 1 | 192 | 1 | 7 | 6 | 56 | 11 | 51 | 4 | 0.071 | 336 | 154 | 3.00 | 6.00 |
| 22 | 0 | 1 | 184 | 1 | 7 | 6 | 54 | 13 | 51 | 2 | 0.037 | 374 | 94 | 3.00 | 6.00 |
| 22 | 1 | 0 | 206 | 0 | 0 | 3 | 38 | 0 | 38 | 1 | 0.026 | 0 | 58 | 2.00 | 8.00 |
| 22 | 2 | 1 | 196 | 1 | 7 | 5 | 49 | 12 | 45 | 5 | 0.102 | 320 | 49 | 3.00 | 6.00 |
| 22 | 3 | 0 | 193 | 0 | 0 | 3 | 38 | 0 | 38 | 0 | 0.000 | 0 | 48 | 2.00 | 8.00 |
| 23 | 0 | 1 | 205 | 1 | 7 | 5 | 48 | 7 | 45 | 3 | 0.062 | 294 | 252 | 3.00 | 6.00 |
| 23 | 1 | 0 | 210 | 0 | 0 | 2 | 32 | 0 | 30 | 4 | 0.125 | 0 | 283 | 2.00 | 8.00 |
| 23 | 2 | 0 | 204 | 0 | 0 | 3 | 39 | 0 | 38 | 4 | 0.103 | 0 | 267 | 2.00 | 8.00 |
| 23 | 3 | 1 | 207 | 1 | 7 | 5 | 49 | 6 | 45 | 4 | 0.082 | 251 | 259 | 3.00 | 6.00 |
| 24 | 0 | 1 | 177 | 1 | 7 | 6 | 56 | 13 | 51 | 3 | 0.054 | 461 | 277 | 3.00 | 6.00 |
| 24 | 1 | 1 | 200 | 1 | 7 | 5 | 47 | 11 | 45 | 1 | 0.021 | 336 | 264 | 3.00 | 6.00 |
| 24 | 2 | 1 | 179 | 1 | 7 | 6 | 52 | 12 | 51 | 1 | 0.019 | 462 | 278 | 3.00 | 6.00 |
| 24 | 3 | 2 | 203 | 2 | 6 | 7 | 48 | 13 | 45 | 4 | 0.083 | 840 | 240 | 4.00 | 4.00 |
| 25 | 0 | 0 | 207 | 0 | 0 | 3 | 40 | 0 | 38 | 2 | 0.050 | 0 | 236 | 2.00 | 8.00 |
| 25 | 1 | 0 | 207 | 0 | 0 | 3 | 40 | 0 | 38 | 5 | 0.125 | 0 | 211 | 2.00 | 8.00 |
| 25 | 2 | 1 | 208 | 1 | 7 | 5 | 50 | 7 | 45 | 3 | 0.060 | 252 | 229 | 3.00 | 6.00 |
| 25 | 3 | 2 | 201 | 3 | 5 | 7 | 48 | 12 | 45 | 2 | 0.042 | 881 | 176 | 4.00 | 4.00 |
| 26 | 0 | 0 | 206 | 0 | 0 | 3 | 39 | 0 | 38 | 1 | 0.026 | 0 | 293 | 2.00 | 8.00 |
| 26 | 1 | 2 | 207 | 2 | 6 | 7 | 46 | 12 | 45 | 1 | 0.022 | 798 | 259 | 4.00 | 4.00 |
| 26 | 2 | 1 | 210 | 1 | 7 | 5 | 47 | 6 | 45 | 1 | 0.021 | 210 | 275 | 3.00 | 6.00 |
| 26 | 3 | 0 | 170 | 0 | 0 | 4 | 49 | 0 | 46 | 2 | 0.041 | 0 | 266 | 2.00 | 8.00 |
| 27 | 0 | 2 | 181 | 3 | 5 | 7 | 45 | 12 | 45 | 0 | 0.000 | 994 | 198 | 4.00 | 4.00 |
| 27 | 1 | 0 | 205 | 0 | 0 | 3 | 38 | 0 | 38 | 0 | 0.000 | 0 | 253 | 2.00 | 8.00 |
| 27 | 2 | 2 | 180 | 3 | 5 | 7 | 46 | 12 | 45 | 1 | 0.022 | 1008 | 222 | 4.00 | 4.00 |
| 27 | 3 | 2 | 203 | 2 | 6 | 7 | 46 | 12 | 45 | 1 | 0.022 | 819 | 234 | 4.00 | 4.00 |
| 28 | 0 | 0 | 203 | 0 | 0 | 3 | 41 | 0 | 38 | 4 | 0.098 | 0 | 216 | 2.00 | 8.00 |
| 28 | 1 | 1 | 208 | 1 | 7 | 5 | 52 | 7 | 45 | 7 | 0.135 | 252 | 262 | 3.00 | 6.00 |
| 28 | 2 | 0 | 201 | 0 | 0 | 3 | 39 | 0 | 38 | 3 | 0.077 | 0 | 234 | 2.00 | 8.00 |
| 28 | 3 | 0 | 205 | 0 | 0 | 3 | 40 | 0 | 38 | 5 | 0.125 | 0 | 279 | 2.00 | 8.00 |
| 29 | 0 | 1 | 203 | 1 | 7 | 5 | 48 | 7 | 45 | 2 | 0.042 | 336 | 240 | 3.00 | 6.00 |
| 29 | 1 | 0 | 206 | 0 | 0 | 3 | 39 | 0 | 38 | 2 | 0.051 | 0 | 241 | 2.00 | 8.00 |
| 29 | 2 | 1 | 202 | 1 | 7 | 5 | 47 | 6 | 45 | 1 | 0.021 | 336 | 243 | 3.00 | 6.00 |
| 29 | 3 | 1 | 199 | 1 | 7 | 5 | 47 | 11 | 45 | 1 | 0.021 | 336 | 252 | 3.00 | 6.00 |
| 30 | 0 | 1 | 208 | 1 | 7 | 5 | 47 | 6 | 45 | 2 | 0.043 | 252 | 221 | 3.00 | 6.00 |
| 30 | 1 | 0 | 206 | 0 | 0 | 3 | 41 | 0 | 38 | 1 | 0.024 | 0 | 216 | 2.00 | 8.00 |
| 30 | 2 | 0 | 204 | 0 | 0 | 3 | 42 | 0 | 38 | 3 | 0.071 | 0 | 233 | 2.00 | 8.00 |
| 30 | 3 | 1 | 207 | 1 | 7 | 5 | 47 | 6 | 45 | 1 | 0.021 | 252 | 202 | 3.00 | 6.00 |
| 31 | 0 | 0 | 201 | 0 | 0 | 3 | 40 | 0 | 38 | 1 | 0.025 | 0 | 217 | 2.00 | 8.00 |
| 31 | 1 | 0 | 203 | 0 | 0 | 3 | 39 | 0 | 38 | 0 | 0.000 | 0 | 242 | 2.00 | 8.00 |
| 31 | 2 | 0 | 182 | 0 | 0 | 3 | 40 | 0 | 38 | 5 | 0.125 | 0 | 236 | 2.00 | 8.00 |
| 31 | 3 | 1 | 205 | 1 | 7 | 5 | 50 | 6 | 45 | 5 | 0.100 | 294 | 244 | 3.00 | 6.00 |

Severity-only R2 of the dark-night count: 0.943.

## Winters by severity

| Severity | Seed-winters | Dark nights (count: winters) | Nights out | First night out | Wood trips | Gathered | Re-target share | Cold member-hours | Food store at end | Night burn | Fall |
|---|---:|---|---|---|---|---|---|---|---|---|---|
| mild | 70 | {"0": 70} | 0 / 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 / 0 | 40 / 38 / 48 / 31 / 50 | 38 / 38 / 46 / 30 / 46 | 0.0566 / 0 / 0.114 / 0 / 0.147 | 0 / 0 / 0 / 0 / 0 | 244 / 197 / 277 / 48 / 294 | 2 / 2 / 2 / 2 / 2 | 8 / 8 / 8 / 8 / 8 |
| normal | 37 | {"1": 36, "2": 1} | 1 / 1 / 1 / 1 / 2 | 7 / 7 / 7 / 6 / 7 | 49 / 46.6 / 55.4 / 45 / 56 | 45 / 45 / 51 / 45 / 51 | 0.0417 / 0.0192 / 0.0936 / 0 / 0.185 | 336 / 252 / 462 / 210 / 557 | 246 / 134 / 277 / 49 / 285 | 3 / 3 / 3 / 3 / 3 | 6 / 6 / 6 / 6 / 6 |
| harsh | 21 | {"2": 12, "3": 9} | 2 / 2 / 3 / 2 / 3 | 6 / 5 / 6 / 5 / 6 | 47 / 45 / 50 / 45 / 52 | 45 / 45 / 45 / 45 / 45 | 0.0417 / 0 / 0.0851 / 0 / 0.1 | 840 / 839 / 1.01e+03 / 798 / 1.01e+03 | 236 / 184 / 264 / 175 / 267 | 4 / 4 / 4 / 4 / 4 | 4 / 4 / 4 / 4 / 4 |
