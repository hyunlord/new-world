# Daughter diagnosis (Pass 12 Part 0, PASS12_GOAL.md 5.3)

Sources: Pass 11's final `game_v17` 120-day probe runs (tuning seeds 0-31 `p11-7_final_v17_120day`, held-out 32-95 `p11-7_final_heldout_v17_120day`; the final binary reproduces the frozen R3/held-out runs on every shared key, Pass 11 Part 7) and the same 96 seeds recorded with Pass 11's final recorder `p11bin/wm_final` (schema 5, hourly) into `worldsim-wm-data/p12/diag/game_v17/`. `daughter_diag.py`; the full per-death and per-winter rows are in `daughter_diag.json`.

## Counts

- `deaths_in_daughters`: {"exposure": 622, "starvation": 20, "dehydration": 38}
- `a_starvation_in_daughters_with_bush_shortfall`: 20
- `a_starvation_in_daughters_without_region`: 0
- `a_starvation_total`: 20
- `a_starvation_food_min_72h`: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
- `b_dehydration_total`: 38
- `b_dehydration_region_water_distance`: [10, 14, 14, 14, 14, 15, 16, 16, 16, 16, 16, 16, 16, 16, 16, 17, 20, 21, 21, 21, 21, 23, 24, 24, 24, 24, 24, 25, 25, 25, 25, 25, 25, 25, 25, 26, 26, 28]
- `b_dehydration_water_distance_at_death`: [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 4, 4, 4, 4, 5, 5, 5, 6, 6, 8, 9, 11, 11, 12, 12, 14, 14, 15, 16, 16, 16, 16, 19, 20]
- `b_dehydration_off_region`: 0
- `c_daughters`: 119
- `c_daughters_off_region_centre`: 0
- `c_daughters_by_season`: {"spring": 117, "summer": 2}
- `c_daughters_with_bush_shortfall`: 69
- `c_region_water_distance_all_daughters`: [1, 1, 1, 2, 2, 3, 3, 4, 5, 5, 5, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 9, 9, 10, 10, 11, 11, 11, 12, 12, 12, 13, 13, 13, 14, 14, 14, 14, 14, 14, 14, 15, 15, 15, 15, 15, 15, 16, 16, 16, 16, 16, 16, 16, 17, 17, 17, 17, 17, 17, 17, 17, 17, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 19, 19, 19, 20, 21, 21, 22, 22, 22, 22, 23, 23, 23, 23, 23, 23, 24, 24, 24, 25, 25, 25, 25, 25, 25, 25, 26, 26, 26, 26, 26, 26, 26, 27, 27, 27, 28, 28, 30]
- `c_daughters_alive_28`: 97
- `exposure_deaths_in_daughters`: 622
- `exposure_in_daughters_dark_nights_of_3`: {"3": 422, "2": 165, "1": 31, "0": 4}
- `d_over_bound_winters`: 42
- `d_settlement_winters_with_members`: 626
- `d_settlement_winters_over_bound`: 45
- `d_settlement_winters_over_bound_daughters`: 45
- `d_max_settlement_winter_ratio`: 1.3

## Deaths by starvation, dehydration and exposure in daughters

| Set | Seed | Cause | Stage | Day (season) | Settlement | Founded day (season) | At leader tile | Region | Bushes placed/requested | Deadwood placed/requested | Water dist (region / at death) | Founders | Members at death | 72 h: food min, wood min, fire lit share, dark nights of 3 |
|---|---:|---|---|---|---:|---|---|---:|---|---|---|---:|---:|---|
| v17_120day | 0 | exposure | elder | 84 (winter) | 2 | 30 (spring) | False | 0 | 6/11 | 13/13 | 10 / 12 | 14 | 18 | 115, 0, 0.11, 3 |
| v17_120day | 0 | exposure | elder | 84 (winter) | 2 | 30 (spring) | False | 0 | 6/11 | 13/13 | 10 / 12 | 14 | 17 | 115, 0, 0.10, 3 |
| v17_120day | 0 | exposure | elder | 84 (winter) | 2 | 30 (spring) | False | 0 | 6/11 | 13/13 | 10 / 2 | 14 | 17 | 110, 0, 0.08, 3 |
| v17_120day | 0 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 6/11 | 13/13 | 10 / 12 | 14 | 17 | 94, 0, 0.01, 2 |
| v17_120day | 1 | exposure | child | 57 (spring) | 2 | 29 (spring) | False | 0 | 10/10 | 8/8 | 23 / 21 | 11 | 29 | 221, 0, 0.29, 1 |
| v17_120day | 1 | exposure | child | 57 (spring) | 2 | 29 (spring) | False | 0 | 10/10 | 8/8 | 23 / 21 | 11 | 29 | 221, 0, 0.25, 2 |
| v17_120day | 2 | exposure | adult | 57 (spring) | 2 | 29 (spring) | False | 1 | 2/7 | 8/8 | 9 / 11 | 14 | 10 | 0, 0, 0.16, 3 |
| v17_120day | 2 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 1 | 2/7 | 8/8 | 9 / 11 | 14 | 9 | 0, 0, 0.15, 3 |
| v17_120day | 2 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 1 | 2/7 | 8/8 | 9 / 11 | 14 | 8 | 0, 0, 0.14, 3 |
| v17_120day | 2 | exposure | adult | 57 (spring) | 2 | 29 (spring) | False | 1 | 2/7 | 8/8 | 9 / 11 | 14 | 8 | 0, 0, 0.14, 3 |
| v17_120day | 2 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 1 | 2/7 | 8/8 | 9 / 11 | 14 | 6 | 37, 0, 0.14, 3 |
| v17_120day | 4 | exposure | child | 56 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 13/13 | 17 / 15 | 14 | 30 | 237, 0, 0.21, 2 |
| v17_120day | 4 | exposure | elder | 56 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 13/13 | 17 / 7 | 14 | 30 | 230, 0, 0.16, 2 |
| v17_120day | 4 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 13/13 | 17 / 15 | 14 | 30 | 202, 0, 0.00, 3 |
| v17_120day | 4 | exposure | adult | 57 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 13/13 | 17 / 7 | 14 | 30 | 202, 0, 0.00, 3 |
| v17_120day | 7 | starvation | elder | 55 (winter) | 2 | 29 (spring) | False | 0 | 2/9 | 13/13 | 2 / 6 | 14 | 12 | 0, 59, 0.55, 0 |
| v17_120day | 7 | exposure | adult | 57 (spring) | 2 | 29 (spring) | False | 0 | 2/9 | 13/13 | 2 / 0 | 14 | 11 | 0, 0, 0.51, 1 |
| v17_120day | 7 | exposure | adult | 57 (spring) | 2 | 29 (spring) | False | 0 | 2/9 | 13/13 | 2 / 1 | 14 | 11 | 0, 0, 0.51, 1 |
| v17_120day | 7 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 2/9 | 13/13 | 2 / 3 | 14 | 9 | 0, 0, 0.47, 1 |
| v17_120day | 8 | exposure | child | 56 (winter) | 2 | 29 (spring) | False | 0 | 6/9 | 11/11 | 18 / 20 | 14 | 23 | 186, 0, 0.29, 2 |
| v17_120day | 8 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 6/9 | 11/11 | 18 / 19 | 14 | 23 | 172, 0, 0.12, 3 |
| v17_120day | 8 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 6/9 | 11/11 | 18 / 19 | 14 | 23 | 172, 0, 0.12, 3 |
| v17_120day | 8 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 6/9 | 11/11 | 18 / 19 | 14 | 21 | 172, 0, 0.11, 3 |
| v17_120day | 8 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 6/9 | 11/11 | 18 / 19 | 14 | 21 | 172, 0, 0.11, 3 |
| v17_120day | 9 | exposure | elder | 55 (winter) | 2 | 30 (spring) | False | 0 | 0/8 | 11/11 | 15 / 3 | 14 | 12 | 0, 0, 0.26, 1 |
| v17_120day | 9 | exposure | adult | 55 (winter) | 2 | 30 (spring) | False | 0 | 0/8 | 11/11 | 15 / 9 | 14 | 12 | 0, 0, 0.26, 1 |
| v17_120day | 9 | exposure | adult | 55 (winter) | 2 | 30 (spring) | False | 0 | 0/8 | 11/11 | 15 / 2 | 14 | 10 | 0, 0, 0.25, 1 |
| v17_120day | 9 | exposure | elder | 55 (winter) | 2 | 30 (spring) | False | 0 | 0/8 | 11/11 | 15 / 0 | 14 | 9 | 0, 0, 0.22, 2 |
| v17_120day | 9 | exposure | adult | 55 (winter) | 2 | 30 (spring) | False | 0 | 0/8 | 11/11 | 15 / 0 | 14 | 8 | 0, 0, 0.19, 2 |
| v17_120day | 9 | exposure | adult | 56 (winter) | 2 | 30 (spring) | False | 0 | 0/8 | 11/11 | 15 / 10 | 14 | 7 | 0, 0, 0.10, 2 |
| v17_120day | 9 | exposure | adult | 56 (winter) | 2 | 30 (spring) | False | 0 | 0/8 | 11/11 | 15 / 10 | 14 | 6 | 0, 0, 0.08, 2 |
| v17_120day | 9 | exposure | adult | 56 (winter) | 2 | 30 (spring) | False | 0 | 0/8 | 11/11 | 15 / 10 | 14 | 5 | 0, 0, 0.03, 3 |
| v17_120day | 9 | exposure | elder | 56 (winter) | 2 | 30 (spring) | False | 0 | 0/8 | 11/11 | 15 / 4 | 14 | 4 | 0, 0, 0.01, 3 |
| v17_120day | 9 | exposure | adult | 56 (winter) | 2 | 30 (spring) | False | 0 | 0/8 | 11/11 | 15 / 10 | 14 | 4 | 0, 0, 0.01, 3 |
| v17_120day | 9 | exposure | elder | 57 (spring) | 2 | 30 (spring) | False | 0 | 0/8 | 11/11 | 15 / 13 | 14 | 2 | 0, 0, 0.01, 2 |
| v17_120day | 9 | exposure | adult | 57 (spring) | 2 | 30 (spring) | False | 0 | 0/8 | 11/11 | 15 / 2 | 14 | 1 | 0, 0, 0.01, 3 |
| v17_120day | 9 | starvation | adult | 104 (autumn) | 3 | 87 (spring) | False | 0 | 0/8 | 11/11 | 15 / 1 | 14 | 14 | 0, 97, 0.55, 0 |
| v17_120day | 9 | dehydration | adult | 106 (winter) | 3 | 87 (spring) | False | 0 | 0/8 | 11/11 | 15 / 1 | 14 | 13 | 0, 162, 0.53, 0 |
| v17_120day | 9 | starvation | adult | 111 (winter) | 3 | 87 (spring) | False | 0 | 0/8 | 11/11 | 15 / 4 | 14 | 11 | 0, 38, 0.55, 0 |
| v17_120day | 9 | exposure | elder | 112 (winter) | 3 | 87 (spring) | False | 0 | 0/8 | 11/11 | 15 / 0 | 14 | 10 | 0, 2, 0.53, 0 |
| v17_120day | 9 | exposure | elder | 112 (winter) | 3 | 87 (spring) | False | 0 | 0/8 | 11/11 | 15 / 6 | 14 | 9 | 0, 0, 0.52, 0 |
| v17_120day | 9 | starvation | elder | 112 (winter) | 3 | 87 (spring) | False | 0 | 0/8 | 11/11 | 15 / 12 | 14 | 8 | 0, 0, 0.51, 1 |
| v17_120day | 9 | exposure | elder | 112 (winter) | 3 | 87 (spring) | False | 0 | 0/8 | 11/11 | 15 / 1 | 14 | 7 | 0, 0, 0.48, 1 |
| v17_120day | 9 | starvation | adult | 112 (winter) | 3 | 87 (spring) | False | 0 | 0/8 | 11/11 | 15 / 4 | 14 | 7 | 0, 0, 0.48, 1 |
| v17_120day | 9 | exposure | elder | 112 (winter) | 3 | 87 (spring) | False | 0 | 0/8 | 11/11 | 15 / 14 | 14 | 5 | 0, 0, 0.44, 1 |
| v17_120day | 9 | exposure | elder | 113 (spring) | 3 | 87 (spring) | False | 0 | 0/8 | 11/11 | 15 / 18 | 14 | 4 | 0, 0, 0.40, 2 |
| v17_120day | 9 | exposure | adult | 113 (spring) | 3 | 87 (spring) | False | 0 | 0/8 | 11/11 | 15 / 18 | 14 | 3 | 0, 0, 0.37, 2 |
| v17_120day | 9 | exposure | elder | 113 (spring) | 3 | 87 (spring) | False | 0 | 0/8 | 11/11 | 15 / 10 | 14 | 2 | 0, 0, 0.36, 2 |
| v17_120day | 11 | starvation | elder | 54 (winter) | 2 | 29 (spring) | False | 0 | 0/7 | 9/10 | 11 / 7 | 14 | 8 | 0, 0, 0.41, 0 |
| v17_120day | 11 | exposure | elder | 54 (winter) | 2 | 29 (spring) | False | 0 | 0/7 | 9/10 | 11 / 1 | 14 | 7 | 0, 0, 0.38, 1 |
| v17_120day | 11 | starvation | adult | 54 (winter) | 2 | 29 (spring) | False | 0 | 0/7 | 9/10 | 11 / 8 | 14 | 6 | 0, 0, 0.37, 1 |
| v17_120day | 11 | starvation | elder | 54 (winter) | 2 | 29 (spring) | False | 0 | 0/7 | 9/10 | 11 / 2 | 14 | 6 | 0, 0, 0.37, 1 |
| v17_120day | 11 | starvation | elder | 54 (winter) | 2 | 29 (spring) | False | 0 | 0/7 | 9/10 | 11 / 10 | 14 | 4 | 0, 0, 0.36, 1 |
| v17_120day | 11 | exposure | elder | 55 (winter) | 2 | 29 (spring) | False | 0 | 0/7 | 9/10 | 11 / 7 | 14 | 3 | 0, 0, 0.25, 1 |
| v17_120day | 11 | exposure | elder | 55 (winter) | 2 | 29 (spring) | False | 0 | 0/7 | 9/10 | 11 / 1 | 14 | 2 | 0, 0, 0.22, 2 |
| v17_120day | 11 | starvation | elder | 111 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 9/10 | 11 / 13 | 14 | 4 | 0, 0, 0.33, 1 |
| v17_120day | 11 | exposure | elder | 111 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 9/10 | 11 / 6 | 14 | 3 | 0, 0, 0.32, 1 |
| v17_120day | 11 | exposure | elder | 111 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 9/10 | 11 / 8 | 14 | 3 | 0, 0, 0.32, 1 |
| v17_120day | 12 | exposure | adult | 57 (spring) | 2 | 30 (spring) | False | 0 | 11/11 | 14/14 | 6 / 4 | 14 | 31 | 229, 0, 0.10, 3 |
| v17_120day | 12 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 11/11 | 14/14 | 6 / 8 | 14 | 31 | 237, 0, 0.05, 3 |
| v17_120day | 12 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 11/11 | 14/14 | 6 / 8 | 14 | 31 | 237, 0, 0.05, 3 |
| v17_120day | 12 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 11/11 | 14/14 | 6 / 8 | 14 | 30 | 229, 0, 0.04, 3 |
| v17_120day | 12 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 11/11 | 14/14 | 6 / 7 | 14 | 30 | 229, 0, 0.04, 3 |
| v17_120day | 12 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 11/11 | 14/14 | 6 / 4 | 14 | 30 | 229, 0, 0.04, 3 |
| v17_120day | 14 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 1 | 4/7 | 14/14 | 23 / 6 | 12 | 16 | 118, 0, 0.15, 3 |
| v17_120day | 14 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 1 | 4/7 | 14/14 | 23 / 25 | 12 | 16 | 118, 0, 0.14, 3 |
| v17_120day | 14 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 1 | 4/7 | 14/14 | 23 / 25 | 12 | 15 | 118, 0, 0.12, 3 |
| v17_120day | 14 | exposure | adult | 57 (spring) | 2 | 29 (spring) | False | 1 | 4/7 | 14/14 | 23 / 10 | 12 | 15 | 118, 0, 0.12, 3 |
| v17_120day | 14 | exposure | child | 85 (spring) | 2 | 29 (spring) | False | 1 | 4/7 | 14/14 | 23 / 25 | 12 | 16 | 112, 0, 0.32, 2 |
| v17_120day | 15 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 30 | 13 | 38 | 288, 0, 0.19, 2 |
| v17_120day | 15 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 9 | 13 | 38 | 288, 0, 0.19, 2 |
| v17_120day | 15 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 5 | 13 | 36 | 288, 0, 0.16, 3 |
| v17_120day | 15 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 15 | 13 | 36 | 288, 0, 0.16, 3 |
| v17_120day | 15 | exposure | child | 84 (winter) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 13 | 13 | 36 | 288, 0, 0.16, 3 |
| v17_120day | 15 | exposure | adult | 84 (winter) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 13 | 13 | 33 | 283, 0, 0.14, 3 |
| v17_120day | 15 | exposure | child | 84 (winter) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 13 | 13 | 32 | 277, 0, 0.12, 3 |
| v17_120day | 15 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 23 | 13 | 32 | 261, 0, 0.05, 3 |
| v17_120day | 15 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 27 | 13 | 31 | 261, 0, 0.05, 2 |
| v17_120day | 15 | exposure | adult | 85 (spring) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 10 | 13 | 31 | 261, 0, 0.05, 2 |
| v17_120day | 15 | exposure | adult | 85 (spring) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 13 | 13 | 30 | 254, 0, 0.05, 3 |
| v17_120day | 15 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 13 | 13 | 29 | 254, 0, 0.05, 3 |
| v17_120day | 15 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 6 | 13 | 28 | 251, 0, 0.05, 3 |
| v17_120day | 15 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 15 | 13 | 27 | 246, 0, 0.05, 3 |
| v17_120day | 15 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 3 | 13 | 27 | 246, 0, 0.05, 3 |
| v17_120day | 15 | dehydration | adult | 110 (winter) | 3 | 98 (summer) | False | 1 | 2/7 | 11/11 | 28 / 5 | 13 | 13 | 105, 0, 0.22, 2 |
| v17_120day | 15 | exposure | adult | 111 (winter) | 3 | 98 (summer) | False | 1 | 2/7 | 11/11 | 28 / 30 | 13 | 12 | 102, 0, 0.14, 2 |
| v17_120day | 15 | exposure | adult | 111 (winter) | 3 | 98 (summer) | False | 1 | 2/7 | 11/11 | 28 / 26 | 13 | 11 | 99, 0, 0.10, 3 |
| v17_120day | 15 | exposure | adult | 111 (winter) | 3 | 98 (summer) | False | 1 | 2/7 | 11/11 | 28 / 18 | 13 | 10 | 99, 0, 0.08, 3 |
| v17_120day | 15 | exposure | adult | 111 (winter) | 3 | 98 (summer) | False | 1 | 2/7 | 11/11 | 28 / 12 | 13 | 9 | 91, 0, 0.00, 3 |
| v17_120day | 15 | exposure | adult | 112 (winter) | 3 | 98 (summer) | False | 1 | 2/7 | 11/11 | 28 / 26 | 13 | 8 | 87, 0, 0.00, 2 |
| v17_120day | 15 | exposure | adult | 112 (winter) | 3 | 98 (summer) | False | 1 | 2/7 | 11/11 | 28 / 3 | 13 | 8 | 87, 0, 0.00, 2 |
| v17_120day | 15 | exposure | elder | 112 (winter) | 3 | 98 (summer) | False | 1 | 2/7 | 11/11 | 28 / 26 | 13 | 7 | 77, 0, 0.00, 3 |
| v17_120day | 15 | exposure | adult | 112 (winter) | 3 | 98 (summer) | False | 1 | 2/7 | 11/11 | 28 / 26 | 13 | 6 | 74, 0, 0.00, 3 |
| v17_120day | 15 | exposure | adult | 112 (winter) | 3 | 98 (summer) | False | 1 | 2/7 | 11/11 | 28 / 26 | 13 | 6 | 74, 0, 0.00, 3 |
| v17_120day | 15 | exposure | adult | 113 (spring) | 3 | 98 (summer) | False | 1 | 2/7 | 11/11 | 28 / 13 | 13 | 5 | 59, 0, 0.00, 2 |
| v17_120day | 15 | exposure | adult | 113 (spring) | 3 | 98 (summer) | False | 1 | 2/7 | 11/11 | 28 / 13 | 13 | 5 | 48, 0, 0.00, 3 |
| v17_120day | 15 | exposure | child | 113 (spring) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 / 13 | 13 | 38 | 277, 0, 0.21, 2 |
| v17_120day | 16 | exposure | adult | 83 (winter) | 2 | 29 (spring) | False | 1 | 0/8 | 14/14 | 21 / 19 | 14 | 6 | 42, 0, 0.29, 2 |
| v17_120day | 16 | exposure | adult | 85 (spring) | 2 | 29 (spring) | False | 1 | 0/8 | 14/14 | 21 / 19 | 14 | 6 | 39, 0, 0.00, 2 |
| v17_120day | 16 | exposure | child | 85 (spring) | 2 | 29 (spring) | False | 1 | 0/8 | 14/14 | 21 / 16 | 14 | 6 | 35, 0, 0.00, 3 |
| v17_120day | 16 | exposure | adult | 85 (spring) | 2 | 29 (spring) | False | 1 | 0/8 | 14/14 | 21 / 19 | 14 | 5 | 35, 0, 0.00, 3 |
| v17_120day | 18 | exposure | elder | 56 (winter) | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 22 / 20 | 13 | 10 | 40, 0, 0.15, 3 |
| v17_120day | 18 | exposure | elder | 56 (winter) | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 22 / 20 | 13 | 9 | 39, 0, 0.14, 3 |
| v17_120day | 18 | exposure | elder | 56 (winter) | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 22 / 20 | 13 | 9 | 39, 0, 0.14, 3 |
| v17_120day | 18 | exposure | adult | 56 (winter) | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 22 / 20 | 13 | 9 | 39, 0, 0.14, 3 |
| v17_120day | 18 | exposure | elder | 56 (winter) | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 22 / 13 | 13 | 6 | 35, 0, 0.11, 3 |
| v17_120day | 18 | exposure | elder | 56 (winter) | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 22 / 20 | 13 | 6 | 35, 0, 0.11, 3 |
| v17_120day | 18 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 22 / 20 | 13 | 6 | 8, 0, 0.04, 3 |
| v17_120day | 18 | exposure | adult | 83 (winter) | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 22 / 16 | 13 | 6 | 23, 0, 0.16, 3 |
| v17_120day | 18 | exposure | adult | 85 (spring) | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 22 / 20 | 13 | 4 | 0, 0, 0.14, 3 |
| v17_120day | 18 | exposure | adult | 85 (spring) | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 22 / 19 | 13 | 4 | 0, 0, 0.14, 3 |
| v17_120day | 19 | dehydration | adult | 35 (spring) | 2 | 29 (spring) | False | 0 | 0/9 | 12/12 | 16 / 12 | 12 | 12 | 0, 18, 0.53, 0 |
| v17_120day | 19 | dehydration | adult | 39 (summer) | 2 | 29 (spring) | False | 0 | 0/9 | 12/12 | 16 / 16 | 12 | 11 | 0, 0, 0.14, 3 |
| v17_120day | 19 | exposure | elder | 55 (winter) | 2 | 29 (spring) | False | 0 | 0/9 | 12/12 | 16 / 16 | 12 | 7 | 0, 0, 0.36, 1 |
| v17_120day | 19 | exposure | elder | 56 (winter) | 2 | 29 (spring) | False | 0 | 0/9 | 12/12 | 16 / 7 | 12 | 6 | 0, 0, 0.29, 2 |
| v17_120day | 19 | exposure | elder | 56 (winter) | 2 | 29 (spring) | False | 0 | 0/9 | 12/12 | 16 / 6 | 12 | 5 | 0, 0, 0.27, 2 |
| v17_120day | 19 | dehydration | elder | 56 (winter) | 2 | 29 (spring) | False | 0 | 0/9 | 12/12 | 16 / 14 | 12 | 4 | 0, 0, 0.23, 2 |
| v17_120day | 19 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 0/9 | 12/12 | 16 / 13 | 12 | 2 | 0, 0, 0.16, 2 |
| v17_120day | 19 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 0/9 | 12/12 | 16 / 14 | 12 | 1 | 0, 0, 0.11, 3 |
| v17_120day | 19 | dehydration | adult | 99 (autumn) | 3 | 85 (spring) | False | 0 | 0/9 | 12/12 | 16 / 16 | 12 | 12 | 0, 7, 0.53, 0 |
| v17_120day | 19 | dehydration | adult | 99 (autumn) | 3 | 85 (spring) | False | 0 | 0/9 | 12/12 | 16 / 16 | 12 | 11 | 0, 7, 0.53, 0 |
| v17_120day | 19 | dehydration | elder | 99 (autumn) | 3 | 85 (spring) | False | 0 | 0/9 | 12/12 | 16 / 9 | 12 | 10 | 0, 4, 0.55, 0 |
| v17_120day | 19 | dehydration | adult | 101 (autumn) | 3 | 85 (spring) | False | 0 | 0/9 | 12/12 | 16 / 16 | 12 | 9 | 0, 0, 0.26, 2 |
| v17_120day | 19 | dehydration | adult | 102 (autumn) | 3 | 85 (spring) | False | 0 | 0/9 | 12/12 | 16 / 2 | 12 | 8 | 0, 0, 0.07, 3 |
| v17_120day | 19 | dehydration | adult | 103 (autumn) | 3 | 85 (spring) | False | 0 | 0/9 | 12/12 | 16 / 3 | 12 | 6 | 0, 0, 0.01, 3 |
| v17_120day | 19 | exposure | elder | 107 (winter) | 3 | 85 (spring) | False | 0 | 0/9 | 12/12 | 16 / 17 | 12 | 5 | 0, 0, 0.11, 3 |
| v17_120day | 19 | exposure | adult | 107 (winter) | 3 | 85 (spring) | False | 0 | 0/9 | 12/12 | 16 / 17 | 12 | 4 | 0, 0, 0.11, 3 |
| v17_120day | 19 | exposure | adult | 108 (winter) | 3 | 85 (spring) | False | 0 | 0/9 | 12/12 | 16 / 10 | 12 | 2 | 0, 0, 0.10, 3 |
| v17_120day | 23 | exposure | elder | 108 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 11/11 | 28 / 22 | 13 | 10 | 46, 0, 0.04, 3 |
| v17_120day | 23 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 11/11 | 28 / 26 | 13 | 8 | 36, 0, 0.01, 2 |
| v17_120day | 23 | exposure | elder | 109 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 11/11 | 28 / 26 | 13 | 7 | 33, 0, 0.01, 3 |
| v17_120day | 23 | exposure | elder | 110 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 11/11 | 28 / 23 | 13 | 6 | 22, 0, 0.01, 3 |
| v17_120day | 23 | exposure | elder | 110 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 11/11 | 28 / 26 | 13 | 5 | 22, 0, 0.01, 3 |
| v17_120day | 23 | exposure | elder | 111 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 11/11 | 28 / 22 | 13 | 4 | 7, 0, 0.00, 2 |
| v17_120day | 23 | exposure | child | 112 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 11/11 | 28 / 26 | 13 | 3 | 0, 0, 0.00, 3 |
| v17_120day | 23 | exposure | elder | 112 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 11/11 | 28 / 26 | 13 | 2 | 0, 0, 0.00, 3 |
| v17_120day | 23 | exposure | child | 112 (winter) | 2 | 35 (spring) | False | 1 | 6/7 | 11/11 | 6 / 5 | 14 | 26 | 193, 0, 0.21, 2 |
| v17_120day | 23 | exposure | elder | 112 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 11/11 | 28 / 26 | 13 | 1 | 0, 0, 0.00, 3 |
| v17_120day | 24 | exposure | adult | 110 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 8 | 12 | 50 | 337, 0, 0.30, 1 |
| v17_120day | 24 | exposure | child | 111 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 17 | 12 | 50 | 337, 0, 0.25, 2 |
| v17_120day | 24 | exposure | adult | 111 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 6 | 12 | 49 | 332, 0, 0.22, 2 |
| v17_120day | 24 | exposure | child | 111 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 17 | 12 | 49 | 332, 0, 0.22, 2 |
| v17_120day | 24 | exposure | adult | 111 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 19 | 12 | 47 | 332, 0, 0.21, 2 |
| v17_120day | 24 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 18 | 12 | 48 | 299, 0, 0.10, 2 |
| v17_120day | 24 | exposure | elder | 112 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 17 | 12 | 47 | 299, 0, 0.07, 3 |
| v17_120day | 24 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 20 | 12 | 47 | 299, 0, 0.07, 3 |
| v17_120day | 24 | exposure | child | 112 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 17 | 12 | 47 | 299, 0, 0.07, 3 |
| v17_120day | 24 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 15 | 12 | 44 | 299, 0, 0.05, 3 |
| v17_120day | 24 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 19 | 12 | 44 | 299, 0, 0.05, 3 |
| v17_120day | 24 | exposure | child | 112 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 17 | 12 | 44 | 299, 0, 0.05, 3 |
| v17_120day | 24 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 20 | 12 | 41 | 299, 0, 0.04, 3 |
| v17_120day | 24 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 7 | 12 | 40 | 299, 0, 0.03, 3 |
| v17_120day | 24 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 19 | 12 | 40 | 299, 0, 0.03, 3 |
| v17_120day | 24 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 21 | 12 | 40 | 299, 0, 0.03, 3 |
| v17_120day | 24 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 11 | 12 | 39 | 282, 0, 0.00, 2 |
| v17_120day | 24 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 14 | 12 | 38 | 282, 0, 0.00, 3 |
| v17_120day | 24 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 19 | 12 | 38 | 282, 0, 0.00, 3 |
| v17_120day | 24 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 17 | 12 | 36 | 282, 0, 0.00, 3 |
| v17_120day | 24 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 10 | 12 | 36 | 282, 0, 0.00, 3 |
| v17_120day | 24 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 19 | 12 | 36 | 282, 0, 0.00, 3 |
| v17_120day | 24 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 17 | 12 | 33 | 282, 0, 0.00, 3 |
| v17_120day | 24 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 17 | 12 | 33 | 282, 0, 0.00, 3 |
| v17_120day | 24 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 17 | 12 | 33 | 282, 0, 0.00, 3 |
| v17_120day | 24 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 3 | 12 | 33 | 282, 0, 0.00, 3 |
| v17_120day | 24 | exposure | child | 113 (spring) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 / 21 | 12 | 33 | 282, 0, 0.00, 3 |
| v17_120day | 25 | exposure | elder | 109 (winter) | 3 | 85 (spring) | False | 2 | 0/7 | 2/14 | 18 / 16 | 14 | 11 | 15, 0, 0.00, 3 |
| v17_120day | 25 | exposure | adult | 110 (winter) | 3 | 85 (spring) | False | 2 | 0/7 | 2/14 | 18 / 16 | 14 | 10 | 7, 0, 0.00, 3 |
| v17_120day | 25 | exposure | adult | 110 (winter) | 3 | 85 (spring) | False | 2 | 0/7 | 2/14 | 18 / 16 | 14 | 10 | 7, 0, 0.00, 3 |
| v17_120day | 25 | exposure | elder | 110 (winter) | 3 | 85 (spring) | False | 2 | 0/7 | 2/14 | 18 / 16 | 14 | 8 | 7, 0, 0.00, 3 |
| v17_120day | 25 | exposure | adult | 110 (winter) | 3 | 85 (spring) | False | 2 | 0/7 | 2/14 | 18 / 19 | 14 | 8 | 7, 0, 0.00, 3 |
| v17_120day | 25 | exposure | elder | 111 (winter) | 3 | 85 (spring) | False | 2 | 0/7 | 2/14 | 18 / 16 | 14 | 6 | 7, 0, 0.00, 3 |
| v17_120day | 25 | exposure | adult | 111 (winter) | 3 | 85 (spring) | False | 2 | 0/7 | 2/14 | 18 / 16 | 14 | 6 | 7, 0, 0.00, 3 |
| v17_120day | 25 | exposure | elder | 111 (winter) | 3 | 85 (spring) | False | 2 | 0/7 | 2/14 | 18 / 16 | 14 | 4 | 7, 0, 0.00, 3 |
| v17_120day | 25 | exposure | elder | 111 (winter) | 3 | 85 (spring) | False | 2 | 0/7 | 2/14 | 18 / 16 | 14 | 3 | 7, 0, 0.00, 3 |
| v17_120day | 25 | exposure | elder | 112 (winter) | 3 | 85 (spring) | False | 2 | 0/7 | 2/14 | 18 / 16 | 14 | 2 | 6, 0, 0.00, 3 |
| v17_120day | 26 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 4 / 2 | 14 | 12 | 22, 0, 0.11, 3 |
| v17_120day | 26 | exposure | child | 85 (spring) | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 4 / 2 | 14 | 8 | 7, 0, 0.03, 3 |
| v17_120day | 27 | exposure | child | 84 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 30 | 204, 0, 0.32, 1 |
| v17_120day | 27 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 30 | 197, 0, 0.27, 2 |
| v17_120day | 27 | exposure | adult | 85 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 20 | 14 | 30 | 178, 0, 0.14, 2 |
| v17_120day | 27 | exposure | adult | 85 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 20 | 14 | 30 | 178, 0, 0.14, 2 |
| v17_120day | 27 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 19 | 14 | 29 | 175, 0, 0.12, 3 |
| v17_120day | 27 | exposure | adult | 85 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 17 | 14 | 29 | 175, 0, 0.12, 3 |
| v17_120day | 27 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 27 | 173, 0, 0.11, 3 |
| v17_120day | 27 | exposure | adult | 85 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 26 | 170, 0, 0.10, 3 |
| v17_120day | 27 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 8 | 14 | 25 | 169, 0, 0.08, 3 |
| v17_120day | 27 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 19 | 14 | 25 | 169, 0, 0.08, 3 |
| v17_120day | 27 | exposure | adult | 85 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 20 | 14 | 25 | 169, 0, 0.08, 3 |
| v17_120day | 27 | exposure | adult | 85 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 25 | 169, 0, 0.08, 3 |
| v17_120day | 27 | exposure | child | 111 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 30 | 228, 0, 0.33, 2 |
| v17_120day | 27 | exposure | adult | 111 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 30 | 228, 0, 0.32, 2 |
| v17_120day | 27 | exposure | child | 111 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 29 | 228, 0, 0.30, 2 |
| v17_120day | 27 | exposure | elder | 112 (winter) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 14 | 13 | 26 | 180, 0, 0.22, 2 |
| v17_120day | 27 | exposure | adult | 112 (winter) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 14 | 13 | 26 | 180, 0, 0.19, 3 |
| v17_120day | 27 | exposure | elder | 112 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 29 | 212, 0, 0.14, 3 |
| v17_120day | 27 | exposure | elder | 112 (winter) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 5 | 13 | 25 | 180, 0, 0.18, 3 |
| v17_120day | 27 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 29 | 212, 0, 0.14, 3 |
| v17_120day | 27 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 29 | 212, 0, 0.14, 3 |
| v17_120day | 27 | exposure | adult | 112 (winter) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 15 | 13 | 25 | 180, 0, 0.18, 3 |
| v17_120day | 27 | exposure | adult | 112 (winter) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 19 | 13 | 23 | 180, 0, 0.16, 3 |
| v17_120day | 27 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 20 | 14 | 26 | 208, 0, 0.12, 3 |
| v17_120day | 27 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 20 | 14 | 26 | 208, 0, 0.12, 3 |
| v17_120day | 27 | exposure | elder | 112 (winter) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 8 | 13 | 22 | 180, 0, 0.15, 3 |
| v17_120day | 27 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 24 | 203, 0, 0.11, 3 |
| v17_120day | 27 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 20 | 14 | 24 | 203, 0, 0.11, 3 |
| v17_120day | 27 | dehydration | adult | 112 (winter) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 5 | 14 | 23 | 182, 0, 0.07, 3 |
| v17_120day | 27 | exposure | adult | 112 (winter) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 12 | 13 | 22 | 170, 0, 0.05, 3 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 3 | 13 | 21 | 170, 0, 0.04, 2 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 22 | 181, 0, 0.00, 2 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 1 | 14 | 21 | 181, 0, 0.00, 3 |
| v17_120day | 27 | exposure | child | 113 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 20 | 14 | 21 | 181, 0, 0.00, 3 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 19 | 13 | 21 | 165, 0, 0.03, 3 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 20 | 14 | 20 | 172, 0, 0.00, 3 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 20 | 13 | 20 | 162, 0, 0.03, 3 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 3 | 14 | 19 | 170, 0, 0.00, 3 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 19 | 14 | 19 | 170, 0, 0.00, 3 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 19 | 13 | 20 | 162, 0, 0.03, 3 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 19 | 13 | 20 | 162, 0, 0.03, 3 |
| v17_120day | 27 | exposure | child | 113 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 20 | 14 | 19 | 170, 0, 0.00, 3 |
| v17_120day | 27 | exposure | elder | 113 (spring) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 8 | 13 | 17 | 161, 0, 0.03, 3 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 18 | 14 | 16 | 169, 0, 0.00, 3 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 20 | 13 | 17 | 161, 0, 0.03, 3 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 / 9 | 14 | 16 | 169, 0, 0.00, 3 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 11 | 13 | 17 | 161, 0, 0.03, 3 |
| v17_120day | 27 | exposure | adult | 113 (spring) | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 / 4 | 13 | 17 | 161, 0, 0.03, 3 |
| v17_120day | 28 | dehydration | adult | 107 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 13/13 | 14 / 2 | 12 | 12 | 0, 0, 0.52, 0 |
| v17_120day | 28 | dehydration | adult | 107 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 13/13 | 14 / 5 | 12 | 11 | 0, 0, 0.49, 0 |
| v17_120day | 28 | exposure | adult | 108 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 13/13 | 14 / 16 | 12 | 10 | 0, 0, 0.44, 1 |
| v17_120day | 28 | dehydration | adult | 108 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 13/13 | 14 / 11 | 12 | 8 | 0, 0, 0.36, 1 |
| v17_120day | 28 | dehydration | adult | 108 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 13/13 | 14 / 11 | 12 | 8 | 0, 0, 0.36, 1 |
| v17_120day | 28 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 13/13 | 14 / 5 | 12 | 6 | 0, 0, 0.29, 1 |
| v17_120day | 28 | exposure | elder | 109 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 13/13 | 14 / 16 | 12 | 5 | 0, 0, 0.27, 2 |
| v17_120day | 28 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 13/13 | 14 / 1 | 12 | 5 | 0, 0, 0.27, 2 |
| v17_120day | 28 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 13/13 | 14 / 13 | 12 | 3 | 0, 0, 0.25, 2 |
| v17_120day | 28 | starvation | adult | 109 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 13/13 | 14 / 12 | 12 | 2 | 0, 0, 0.21, 2 |
| v17_120day | 28 | exposure | adult | 110 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 13/13 | 14 / 12 | 12 | 1 | 0, 0, 0.12, 2 |
| v17_120day | 28 | exposure | child | 113 (spring) | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 16 / 18 | 14 | 42 | 273, 0, 0.34, 1 |
| v17_120day | 31 | dehydration | adult | 111 (winter) | 3 | 85 (spring) | False | 2 | 9/9 | 9/9 | 23 / 6 | 11 | 39 | 313, 26, 0.53, 0 |
| v17_120day | 31 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 2 | 9/9 | 9/9 | 23 / 20 | 11 | 39 | 310, 0, 0.53, 0 |
| heldout_v17_120day | 37 | starvation | elder | 118 (spring) | 3 | 85 (spring) | False | 1 | 0/10 | 10/10 | 7 / 6 | 13 | 5 | 0, 0, 0.55, 0 |
| heldout_v17_120day | 38 | exposure | elder | 57 (spring) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 25 | 14 | 26 | 173, 0, 0.21, 2 |
| heldout_v17_120day | 38 | exposure | elder | 57 (spring) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 25 | 14 | 25 | 169, 0, 0.19, 2 |
| heldout_v17_120day | 38 | exposure | elder | 57 (spring) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 22 | 14 | 24 | 162, 0, 0.16, 2 |
| heldout_v17_120day | 38 | exposure | elder | 57 (spring) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 5 | 14 | 24 | 162, 0, 0.16, 2 |
| heldout_v17_120day | 38 | exposure | adult | 57 (spring) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 4 | 14 | 24 | 162, 0, 0.16, 2 |
| heldout_v17_120day | 38 | exposure | elder | 83 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 20 | 14 | 26 | 211, 0, 0.18, 2 |
| heldout_v17_120day | 38 | exposure | elder | 83 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 24 | 14 | 26 | 211, 0, 0.18, 2 |
| heldout_v17_120day | 38 | exposure | adult | 83 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 17 | 14 | 26 | 211, 0, 0.18, 2 |
| heldout_v17_120day | 38 | exposure | elder | 83 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 14 | 14 | 23 | 209, 0, 0.16, 3 |
| heldout_v17_120day | 38 | exposure | elder | 83 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 26 | 14 | 23 | 209, 0, 0.16, 3 |
| heldout_v17_120day | 38 | exposure | elder | 83 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 25 | 14 | 21 | 205, 0, 0.15, 3 |
| heldout_v17_120day | 38 | exposure | elder | 83 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 25 | 14 | 21 | 205, 0, 0.15, 3 |
| heldout_v17_120day | 38 | exposure | adult | 83 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 25 | 14 | 19 | 202, 0, 0.14, 3 |
| heldout_v17_120day | 38 | exposure | elder | 83 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 16 | 14 | 19 | 173, 0, 0.04, 3 |
| heldout_v17_120day | 38 | exposure | elder | 83 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 28 | 14 | 18 | 172, 0, 0.03, 3 |
| heldout_v17_120day | 38 | exposure | elder | 84 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 25 | 14 | 18 | 161, 0, 0.01, 2 |
| heldout_v17_120day | 38 | exposure | adult | 84 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 10 | 14 | 18 | 161, 0, 0.01, 2 |
| heldout_v17_120day | 38 | exposure | elder | 84 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 9 | 14 | 16 | 160, 0, 0.01, 2 |
| heldout_v17_120day | 38 | exposure | adult | 84 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 20 | 14 | 16 | 160, 0, 0.01, 2 |
| heldout_v17_120day | 38 | exposure | adult | 84 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 3 | 14 | 14 | 160, 0, 0.01, 3 |
| heldout_v17_120day | 38 | exposure | adult | 84 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 25 | 14 | 13 | 160, 0, 0.01, 3 |
| heldout_v17_120day | 38 | exposure | elder | 84 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 25 | 14 | 12 | 158, 0, 0.01, 3 |
| heldout_v17_120day | 38 | exposure | adult | 84 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 6 | 14 | 11 | 156, 0, 0.01, 3 |
| heldout_v17_120day | 38 | exposure | child | 84 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 25 | 14 | 11 | 156, 0, 0.01, 3 |
| heldout_v17_120day | 38 | dehydration | elder | 84 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 1 | 14 | 10 | 138, 0, 0.00, 3 |
| heldout_v17_120day | 38 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 5 | 14 | 10 | 127, 0, 0.00, 2 |
| heldout_v17_120day | 38 | exposure | adult | 85 (spring) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 26 | 14 | 9 | 126, 0, 0.00, 3 |
| heldout_v17_120day | 38 | exposure | adult | 85 (spring) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 0 | 14 | 8 | 124, 0, 0.00, 3 |
| heldout_v17_120day | 38 | exposure | child | 85 (spring) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 25 | 14 | 8 | 124, 0, 0.00, 3 |
| heldout_v17_120day | 38 | exposure | child | 112 (winter) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 28 | 14 | 26 | 171, 0, 0.36, 2 |
| heldout_v17_120day | 38 | exposure | adult | 113 (spring) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 14 | 14 | 26 | 142, 0, 0.23, 3 |
| heldout_v17_120day | 38 | exposure | adult | 113 (spring) | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 / 1 | 14 | 26 | 129, 0, 0.22, 3 |
| heldout_v17_120day | 39 | exposure | child | 85 (spring) | 2 | 29 (spring) | False | 0 | 8/8 | 8/8 | 7 / 5 | 14 | 30 | 206, 0, 0.32, 1 |
| heldout_v17_120day | 39 | exposure | child | 111 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 17 | 12 | 23 | 168, 0, 0.30, 1 |
| heldout_v17_120day | 39 | exposure | elder | 111 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 24 | 12 | 22 | 168, 0, 0.23, 2 |
| heldout_v17_120day | 39 | exposure | elder | 112 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 24 | 12 | 22 | 140, 0, 0.10, 3 |
| heldout_v17_120day | 39 | exposure | elder | 112 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 17 | 12 | 22 | 140, 0, 0.10, 3 |
| heldout_v17_120day | 39 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 24 | 12 | 22 | 140, 0, 0.10, 3 |
| heldout_v17_120day | 39 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 24 | 12 | 22 | 140, 0, 0.10, 3 |
| heldout_v17_120day | 39 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 25 | 12 | 22 | 140, 0, 0.10, 3 |
| heldout_v17_120day | 39 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 24 | 12 | 17 | 139, 0, 0.07, 3 |
| heldout_v17_120day | 39 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 25 | 12 | 17 | 139, 0, 0.07, 3 |
| heldout_v17_120day | 39 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 24 | 12 | 17 | 139, 0, 0.07, 3 |
| heldout_v17_120day | 39 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 4 | 12 | 14 | 138, 0, 0.05, 3 |
| heldout_v17_120day | 39 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 24 | 12 | 14 | 138, 0, 0.05, 3 |
| heldout_v17_120day | 39 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 10 | 12 | 14 | 138, 0, 0.05, 3 |
| heldout_v17_120day | 39 | exposure | child | 112 (winter) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 24 | 12 | 14 | 138, 0, 0.05, 3 |
| heldout_v17_120day | 39 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 2 | 12 | 12 | 107, 0, 0.00, 2 |
| heldout_v17_120day | 39 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 19 | 12 | 11 | 104, 0, 0.00, 3 |
| heldout_v17_120day | 39 | exposure | child | 113 (spring) | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 / 25 | 12 | 11 | 104, 0, 0.00, 3 |
| heldout_v17_120day | 39 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 0 | 8/8 | 8/8 | 7 / 2 | 14 | 30 | 178, 0, 0.00, 3 |
| heldout_v17_120day | 39 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 0 | 8/8 | 8/8 | 7 / 5 | 14 | 29 | 177, 0, 0.00, 3 |
| heldout_v17_120day | 39 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 8/8 | 8/8 | 7 / 6 | 14 | 29 | 177, 0, 0.00, 3 |
| heldout_v17_120day | 39 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 8/8 | 8/8 | 7 / 6 | 14 | 29 | 177, 0, 0.00, 3 |
| heldout_v17_120day | 39 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 8/8 | 8/8 | 7 / 7 | 14 | 29 | 177, 0, 0.00, 3 |
| heldout_v17_120day | 44 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 0 | 3/7 | 10/10 | 18 / 14 | 14 | 10 | 65, 0, 0.07, 3 |
| heldout_v17_120day | 44 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 0 | 3/7 | 10/10 | 18 / 11 | 14 | 10 | 65, 0, 0.07, 3 |
| heldout_v17_120day | 44 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 3/7 | 10/10 | 18 / 16 | 14 | 8 | 65, 0, 0.05, 3 |
| heldout_v17_120day | 44 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 3/7 | 10/10 | 18 / 16 | 14 | 7 | 65, 0, 0.05, 3 |
| heldout_v17_120day | 45 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 0/11 | 8/8 | 17 / 19 | 14 | 12 | 58, 0, 0.14, 2 |
| heldout_v17_120day | 45 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 0/11 | 8/8 | 17 / 19 | 14 | 11 | 57, 0, 0.12, 3 |
| heldout_v17_120day | 45 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 0/11 | 8/8 | 17 / 19 | 14 | 10 | 54, 0, 0.11, 3 |
| heldout_v17_120day | 45 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 0/11 | 8/8 | 17 / 18 | 14 | 10 | 54, 0, 0.11, 3 |
| heldout_v17_120day | 45 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 0/11 | 8/8 | 17 / 12 | 14 | 8 | 54, 0, 0.10, 3 |
| heldout_v17_120day | 45 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 0/11 | 8/8 | 17 / 2 | 14 | 7 | 53, 0, 0.08, 3 |
| heldout_v17_120day | 45 | exposure | elder | 81 (winter) | 2 | 29 (spring) | False | 0 | 0/11 | 8/8 | 17 / 19 | 14 | 2 | 0, 0, 0.00, 3 |
| heldout_v17_120day | 45 | exposure | adult | 113 (spring) | 3 | 88 (spring) | False | 0 | 0/11 | 8/8 | 17 / 17 | 14 | 12 | 76, 0, 0.04, 3 |
| heldout_v17_120day | 47 | exposure | child | 83 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 28 | 12 | 29 | 218, 0, 0.26, 1 |
| heldout_v17_120day | 47 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 13 | 12 | 29 | 201, 0, 0.08, 2 |
| heldout_v17_120day | 47 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 25 | 12 | 29 | 201, 0, 0.08, 2 |
| heldout_v17_120day | 47 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 25 | 12 | 29 | 201, 0, 0.08, 2 |
| heldout_v17_120day | 47 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 25 | 12 | 27 | 197, 0, 0.07, 3 |
| heldout_v17_120day | 47 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 24 | 12 | 26 | 193, 0, 0.05, 3 |
| heldout_v17_120day | 47 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 24 | 12 | 26 | 193, 0, 0.05, 3 |
| heldout_v17_120day | 47 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 22 | 12 | 26 | 193, 0, 0.05, 3 |
| heldout_v17_120day | 47 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 24 | 12 | 26 | 193, 0, 0.05, 3 |
| heldout_v17_120day | 47 | exposure | adult | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 22 | 12 | 26 | 193, 0, 0.05, 3 |
| heldout_v17_120day | 47 | exposure | adult | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 24 | 12 | 26 | 193, 0, 0.05, 3 |
| heldout_v17_120day | 47 | exposure | adult | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 24 | 12 | 26 | 193, 0, 0.05, 3 |
| heldout_v17_120day | 47 | exposure | adult | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 28 | 12 | 26 | 193, 0, 0.05, 3 |
| heldout_v17_120day | 47 | exposure | adult | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 28 | 12 | 26 | 193, 0, 0.05, 3 |
| heldout_v17_120day | 47 | exposure | adult | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 12 | 12 | 17 | 191, 0, 0.04, 3 |
| heldout_v17_120day | 47 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 20 | 12 | 16 | 190, 0, 0.03, 3 |
| heldout_v17_120day | 47 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 24 | 12 | 16 | 190, 0, 0.03, 3 |
| heldout_v17_120day | 47 | exposure | adult | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 24 | 12 | 16 | 190, 0, 0.03, 3 |
| heldout_v17_120day | 47 | exposure | adult | 84 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 7 | 12 | 16 | 190, 0, 0.03, 3 |
| heldout_v17_120day | 47 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 11 | 12 | 13 | 162, 0, 0.00, 2 |
| heldout_v17_120day | 47 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 10 | 12 | 13 | 150, 0, 0.00, 3 |
| heldout_v17_120day | 47 | exposure | child | 85 (spring) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 25 | 12 | 12 | 149, 0, 0.00, 3 |
| heldout_v17_120day | 47 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 24 | 12 | 11 | 149, 0, 0.00, 3 |
| heldout_v17_120day | 47 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 19 | 12 | 29 | 226, 0, 0.34, 1 |
| heldout_v17_120day | 47 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 2 | 12 | 29 | 207, 0, 0.19, 2 |
| heldout_v17_120day | 47 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 / 16 | 12 | 29 | 207, 0, 0.19, 2 |
| heldout_v17_120day | 48 | exposure | elder | 53 (winter) | 2 | 29 (spring) | False | 0 | 1/7 | 12/12 | 25 / 1 | 13 | 8 | 0, 0, 0.11, 3 |
| heldout_v17_120day | 48 | exposure | elder | 53 (winter) | 2 | 29 (spring) | False | 0 | 1/7 | 12/12 | 25 / 26 | 13 | 7 | 0, 0, 0.11, 3 |
| heldout_v17_120day | 48 | exposure | elder | 53 (winter) | 2 | 29 (spring) | False | 0 | 1/7 | 12/12 | 25 / 26 | 13 | 6 | 0, 0, 0.11, 3 |
| heldout_v17_120day | 48 | dehydration | elder | 53 (winter) | 2 | 29 (spring) | False | 0 | 1/7 | 12/12 | 25 / 8 | 13 | 5 | 0, 0, 0.11, 3 |
| heldout_v17_120day | 48 | exposure | adult | 53 (winter) | 2 | 29 (spring) | False | 0 | 1/7 | 12/12 | 25 / 3 | 13 | 4 | 0, 0, 0.05, 3 |
| heldout_v17_120day | 48 | exposure | elder | 54 (winter) | 2 | 29 (spring) | False | 0 | 1/7 | 12/12 | 25 / 26 | 13 | 3 | 0, 0, 0.05, 2 |
| heldout_v17_120day | 48 | dehydration | elder | 54 (winter) | 2 | 29 (spring) | False | 0 | 1/7 | 12/12 | 25 / 14 | 13 | 2 | 0, 0, 0.05, 3 |
| heldout_v17_120day | 48 | dehydration | elder | 54 (winter) | 2 | 29 (spring) | False | 0 | 1/7 | 12/12 | 25 / 4 | 13 | 1 | 0, 0, 0.05, 3 |
| heldout_v17_120day | 48 | starvation | adult | 108 (winter) | 3 | 86 (spring) | False | 0 | 1/7 | 12/12 | 25 / 5 | 13 | 10 | 0, 0, 0.05, 2 |
| heldout_v17_120day | 48 | exposure | adult | 108 (winter) | 3 | 86 (spring) | False | 0 | 1/7 | 12/12 | 25 / 1 | 13 | 9 | 0, 0, 0.05, 2 |
| heldout_v17_120day | 48 | exposure | adult | 108 (winter) | 3 | 86 (spring) | False | 0 | 1/7 | 12/12 | 25 / 0 | 13 | 8 | 0, 0, 0.05, 3 |
| heldout_v17_120day | 48 | exposure | elder | 108 (winter) | 3 | 86 (spring) | False | 0 | 1/7 | 12/12 | 25 / 18 | 13 | 7 | 0, 0, 0.05, 3 |
| heldout_v17_120day | 48 | exposure | elder | 108 (winter) | 3 | 86 (spring) | False | 0 | 1/7 | 12/12 | 25 / 9 | 13 | 6 | 0, 0, 0.05, 3 |
| heldout_v17_120day | 48 | exposure | adult | 108 (winter) | 3 | 86 (spring) | False | 0 | 1/7 | 12/12 | 25 / 3 | 13 | 6 | 0, 0, 0.05, 3 |
| heldout_v17_120day | 48 | dehydration | elder | 108 (winter) | 3 | 86 (spring) | False | 0 | 1/7 | 12/12 | 25 / 2 | 13 | 4 | 0, 0, 0.00, 3 |
| heldout_v17_120day | 48 | exposure | elder | 109 (winter) | 3 | 86 (spring) | False | 0 | 1/7 | 12/12 | 25 / 28 | 13 | 3 | 0, 0, 0.00, 2 |
| heldout_v17_120day | 48 | exposure | elder | 109 (winter) | 3 | 86 (spring) | False | 0 | 1/7 | 12/12 | 25 / 27 | 13 | 3 | 0, 0, 0.00, 2 |
| heldout_v17_120day | 48 | exposure | elder | 109 (winter) | 3 | 86 (spring) | False | 0 | 1/7 | 12/12 | 25 / 28 | 13 | 1 | 0, 0, 0.00, 3 |
| heldout_v17_120day | 51 | dehydration | adult | 112 (winter) | 2 | 33 (spring) | False | 0 | 9/9 | 9/9 | 10 / 4 | 14 | 31 | 242, 24, 0.55, 0 |
| heldout_v17_120day | 53 | exposure | adult | 57 (spring) | 2 | 31 (spring) | False | 0 | 10/10 | 13/13 | 17 / 15 | 12 | 38 | 279, 0, 0.23, 2 |
| heldout_v17_120day | 53 | exposure | child | 57 (spring) | 2 | 31 (spring) | False | 0 | 10/10 | 13/13 | 17 / 15 | 12 | 37 | 279, 0, 0.22, 2 |
| heldout_v17_120day | 53 | exposure | elder | 85 (spring) | 2 | 31 (spring) | False | 0 | 10/10 | 13/13 | 17 / 16 | 12 | 38 | 278, 0, 0.18, 2 |
| heldout_v17_120day | 53 | exposure | elder | 85 (spring) | 2 | 31 (spring) | False | 0 | 10/10 | 13/13 | 17 / 16 | 12 | 38 | 278, 0, 0.15, 3 |
| heldout_v17_120day | 53 | exposure | elder | 85 (spring) | 2 | 31 (spring) | False | 0 | 10/10 | 13/13 | 17 / 15 | 12 | 37 | 278, 0, 0.14, 3 |
| heldout_v17_120day | 53 | exposure | elder | 85 (spring) | 2 | 31 (spring) | False | 0 | 10/10 | 13/13 | 17 / 16 | 12 | 37 | 278, 0, 0.14, 3 |
| heldout_v17_120day | 53 | exposure | elder | 85 (spring) | 2 | 31 (spring) | False | 0 | 10/10 | 13/13 | 17 / 16 | 12 | 35 | 278, 0, 0.12, 3 |
| heldout_v17_120day | 53 | exposure | elder | 112 (winter) | 3 | 88 (spring) | False | 1 | 5/10 | 13/13 | 15 / 3 | 13 | 15 | 106, 0, 0.12, 3 |
| heldout_v17_120day | 53 | exposure | adult | 113 (spring) | 2 | 31 (spring) | False | 0 | 10/10 | 13/13 | 17 / 19 | 12 | 38 | 242, 0, 0.04, 3 |
| heldout_v17_120day | 53 | exposure | adult | 113 (spring) | 2 | 31 (spring) | False | 0 | 10/10 | 13/13 | 17 / 15 | 12 | 37 | 235, 0, 0.04, 3 |
| heldout_v17_120day | 53 | exposure | adult | 113 (spring) | 3 | 88 (spring) | False | 1 | 5/10 | 13/13 | 15 / 15 | 13 | 15 | 87, 0, 0.04, 3 |
| heldout_v17_120day | 53 | exposure | adult | 113 (spring) | 2 | 31 (spring) | False | 0 | 10/10 | 13/13 | 17 / 13 | 12 | 36 | 229, 0, 0.04, 3 |
| heldout_v17_120day | 53 | exposure | adult | 113 (spring) | 2 | 31 (spring) | False | 0 | 10/10 | 13/13 | 17 / 17 | 12 | 36 | 229, 0, 0.04, 3 |
| heldout_v17_120day | 54 | exposure | child | 55 (winter) | 2 | 29 (spring) | False | 0 | 10/10 | 12/12 | 8 / 6 | 13 | 39 | 290, 0, 0.32, 2 |
| heldout_v17_120day | 54 | exposure | child | 57 (spring) | 2 | 29 (spring) | False | 0 | 10/10 | 12/12 | 8 / 6 | 13 | 39 | 232, 0, 0.01, 2 |
| heldout_v17_120day | 54 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 10/10 | 12/12 | 8 / 5 | 13 | 39 | 212, 0, 0.01, 3 |
| heldout_v17_120day | 54 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 0 | 10/10 | 12/12 | 8 / 6 | 13 | 39 | 227, 0, 0.07, 3 |
| heldout_v17_120day | 54 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 0 | 10/10 | 12/12 | 8 / 6 | 13 | 38 | 210, 0, 0.05, 3 |
| heldout_v17_120day | 56 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 1 | 10/10 | 10/10 | 27 / 17 | 12 | 30 | 204, 0, 0.18, 2 |
| heldout_v17_120day | 56 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 1 | 10/10 | 10/10 | 27 / 14 | 12 | 30 | 204, 0, 0.18, 2 |
| heldout_v17_120day | 56 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 1 | 10/10 | 10/10 | 27 / 25 | 12 | 29 | 204, 0, 0.16, 3 |
| heldout_v17_120day | 56 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 1 | 10/10 | 10/10 | 27 / 25 | 12 | 29 | 204, 0, 0.16, 3 |
| heldout_v17_120day | 56 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 1 | 10/10 | 10/10 | 27 / 27 | 12 | 29 | 204, 0, 0.16, 3 |
| heldout_v17_120day | 56 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 1 | 10/10 | 10/10 | 27 / 21 | 12 | 26 | 204, 0, 0.15, 3 |
| heldout_v17_120day | 56 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 1 | 10/10 | 10/10 | 27 / 27 | 12 | 25 | 204, 0, 0.14, 3 |
| heldout_v17_120day | 56 | exposure | child | 85 (spring) | 2 | 30 (spring) | False | 1 | 10/10 | 10/10 | 27 / 25 | 12 | 25 | 204, 0, 0.14, 3 |
| heldout_v17_120day | 56 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 1 | 10/10 | 10/10 | 27 / 24 | 12 | 23 | 204, 0, 0.12, 3 |
| heldout_v17_120day | 56 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 1 | 10/10 | 10/10 | 27 / 21 | 12 | 23 | 204, 0, 0.12, 3 |
| heldout_v17_120day | 56 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 1 | 10/10 | 10/10 | 27 / 26 | 12 | 23 | 204, 0, 0.12, 3 |
| heldout_v17_120day | 58 | exposure | child | 57 (spring) | 2 | 29 (spring) | False | 0 | 7/7 | 13/13 | 18 / 16 | 13 | 21 | 124, 0, 0.30, 1 |
| heldout_v17_120day | 59 | exposure | adult | 108 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 8/8 | 25 / 11 | 14 | 8 | 0, 0, 0.27, 2 |
| heldout_v17_120day | 59 | exposure | elder | 108 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 8/8 | 25 / 23 | 14 | 7 | 0, 0, 0.25, 3 |
| heldout_v17_120day | 59 | exposure | adult | 108 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 8/8 | 25 / 13 | 14 | 7 | 0, 0, 0.25, 3 |
| heldout_v17_120day | 59 | exposure | adult | 108 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 8/8 | 25 / 1 | 14 | 5 | 0, 0, 0.23, 3 |
| heldout_v17_120day | 59 | exposure | elder | 108 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 8/8 | 25 / 9 | 14 | 4 | 0, 0, 0.22, 3 |
| heldout_v17_120day | 59 | dehydration | elder | 108 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 8/8 | 25 / 12 | 14 | 3 | 0, 0, 0.18, 3 |
| heldout_v17_120day | 59 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 8/8 | 25 / 1 | 14 | 2 | 0, 0, 0.11, 2 |
| heldout_v17_120day | 59 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 1 | 0/11 | 8/8 | 25 / 15 | 14 | 1 | 0, 0, 0.10, 3 |
| heldout_v17_120day | 60 | exposure | child | 85 (spring) | 2 | 30 (spring) | False | 1 | 7/7 | 10/10 | 18 / 17 | 14 | 34 | 243, 0, 0.25, 3 |
| heldout_v17_120day | 60 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 1 | 7/7 | 10/10 | 18 / 15 | 14 | 33 | 243, 0, 0.22, 3 |
| heldout_v17_120day | 60 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 1 | 7/7 | 10/10 | 18 / 5 | 14 | 33 | 243, 0, 0.22, 3 |
| heldout_v17_120day | 60 | exposure | child | 85 (spring) | 2 | 30 (spring) | False | 1 | 7/7 | 10/10 | 18 / 17 | 14 | 33 | 243, 0, 0.22, 3 |
| heldout_v17_120day | 65 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 5 / 1 | 13 | 27 | 198, 0, 0.00, 3 |
| heldout_v17_120day | 65 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 5 / 4 | 13 | 27 | 198, 0, 0.00, 3 |
| heldout_v17_120day | 65 | exposure | elder | 84 (winter) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 5 / 4 | 13 | 26 | 198, 0, 0.00, 3 |
| heldout_v17_120day | 65 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 5 / 5 | 13 | 26 | 187, 0, 0.00, 3 |
| heldout_v17_120day | 65 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 5 / 3 | 13 | 25 | 187, 0, 0.00, 3 |
| heldout_v17_120day | 66 | exposure | child | 112 (winter) | 2 | 29 (spring) | False | 1 | 10/10 | 11/11 | 7 / 6 | 13 | 36 | 231, 0, 0.21, 2 |
| heldout_v17_120day | 66 | exposure | child | 113 (spring) | 2 | 29 (spring) | False | 1 | 10/10 | 11/11 | 7 / 6 | 13 | 36 | 194, 0, 0.08, 3 |
| heldout_v17_120day | 66 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 0 | 9/10 | 11/11 | 17 / 1 | 13 | 39 | 258, 0, 0.14, 3 |
| heldout_v17_120day | 66 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 1 | 10/10 | 11/11 | 7 / 8 | 13 | 35 | 185, 0, 0.05, 3 |
| heldout_v17_120day | 66 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 0 | 9/10 | 11/11 | 17 / 17 | 13 | 39 | 244, 0, 0.12, 3 |
| heldout_v17_120day | 66 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 0 | 9/10 | 11/11 | 17 / 8 | 13 | 39 | 244, 0, 0.12, 3 |
| heldout_v17_120day | 66 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 0 | 9/10 | 11/11 | 17 / 11 | 13 | 37 | 234, 0, 0.11, 3 |
| heldout_v17_120day | 67 | exposure | elder | 111 (winter) | 2 | 29 (spring) | False | 0 | 4/11 | 11/11 | 24 / 22 | 12 | 15 | 115, 0, 0.33, 2 |
| heldout_v17_120day | 67 | exposure | elder | 111 (winter) | 2 | 29 (spring) | False | 0 | 4/11 | 11/11 | 24 / 1 | 12 | 14 | 115, 0, 0.32, 2 |
| heldout_v17_120day | 67 | exposure | elder | 112 (winter) | 2 | 29 (spring) | False | 0 | 4/11 | 11/11 | 24 / 6 | 12 | 15 | 102, 0, 0.21, 2 |
| heldout_v17_120day | 67 | exposure | child | 112 (winter) | 2 | 29 (spring) | False | 0 | 4/11 | 11/11 | 24 / 22 | 12 | 14 | 102, 0, 0.19, 2 |
| heldout_v17_120day | 67 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 0 | 4/11 | 11/11 | 24 / 21 | 12 | 13 | 98, 0, 0.15, 3 |
| heldout_v17_120day | 67 | exposure | elder | 112 (winter) | 2 | 29 (spring) | False | 0 | 4/11 | 11/11 | 24 / 22 | 12 | 12 | 98, 0, 0.14, 3 |
| heldout_v17_120day | 67 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 4/11 | 11/11 | 24 / 22 | 12 | 13 | 81, 0, 0.05, 3 |
| heldout_v17_120day | 67 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 4/11 | 11/11 | 24 / 22 | 12 | 13 | 81, 0, 0.05, 3 |
| heldout_v17_120day | 67 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 4/11 | 11/11 | 24 / 22 | 12 | 11 | 81, 0, 0.05, 3 |
| heldout_v17_120day | 67 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 4/11 | 11/11 | 24 / 22 | 12 | 11 | 81, 0, 0.05, 3 |
| heldout_v17_120day | 67 | exposure | child | 113 (spring) | 2 | 29 (spring) | False | 0 | 4/11 | 11/11 | 24 / 22 | 12 | 9 | 81, 0, 0.05, 3 |
| heldout_v17_120day | 68 | exposure | adult | 110 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 16 | 14 | 20 | 97, 0, 0.23, 2 |
| heldout_v17_120day | 68 | exposure | adult | 110 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 29 | 14 | 19 | 97, 0, 0.22, 2 |
| heldout_v17_120day | 68 | exposure | child | 110 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 25 | 14 | 19 | 97, 0, 0.22, 2 |
| heldout_v17_120day | 68 | exposure | adult | 111 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 13 | 14 | 19 | 75, 0, 0.12, 2 |
| heldout_v17_120day | 68 | exposure | adult | 111 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 2 | 14 | 18 | 75, 0, 0.11, 2 |
| heldout_v17_120day | 68 | exposure | adult | 111 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 29 | 14 | 17 | 75, 0, 0.10, 3 |
| heldout_v17_120day | 68 | exposure | adult | 111 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 8 | 14 | 17 | 75, 0, 0.10, 3 |
| heldout_v17_120day | 68 | exposure | adult | 111 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 24 | 14 | 17 | 75, 0, 0.10, 3 |
| heldout_v17_120day | 68 | exposure | adult | 111 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 29 | 14 | 14 | 74, 0, 0.08, 3 |
| heldout_v17_120day | 68 | exposure | adult | 111 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 12 | 14 | 14 | 74, 0, 0.08, 3 |
| heldout_v17_120day | 68 | exposure | adult | 111 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 29 | 14 | 12 | 74, 0, 0.07, 3 |
| heldout_v17_120day | 68 | exposure | adult | 111 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 26 | 14 | 11 | 71, 0, 0.07, 3 |
| heldout_v17_120day | 68 | exposure | child | 112 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 29 | 14 | 10 | 50, 0, 0.05, 2 |
| heldout_v17_120day | 68 | exposure | child | 112 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 29 | 14 | 10 | 50, 0, 0.05, 2 |
| heldout_v17_120day | 68 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 29 | 14 | 8 | 49, 0, 0.05, 3 |
| heldout_v17_120day | 68 | exposure | elder | 112 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 29 | 14 | 7 | 48, 0, 0.05, 3 |
| heldout_v17_120day | 68 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 20 | 14 | 7 | 48, 0, 0.05, 3 |
| heldout_v17_120day | 68 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 3 | 14 | 5 | 48, 0, 0.05, 3 |
| heldout_v17_120day | 68 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 28 | 14 | 6 | 28, 0, 0.04, 3 |
| heldout_v17_120day | 68 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 29 | 14 | 5 | 28, 0, 0.04, 3 |
| heldout_v17_120day | 68 | exposure | child | 113 (spring) | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 / 29 | 14 | 5 | 28, 0, 0.04, 3 |
| heldout_v17_120day | 71 | exposure | elder | 56 (winter) | 2 | 31 (spring) | False | 2 | 0/10 | 8/8 | 22 / 19 | 14 | 11 | 136, 0, 0.15, 3 |
| heldout_v17_120day | 71 | exposure | elder | 56 (winter) | 2 | 31 (spring) | False | 2 | 0/10 | 8/8 | 22 / 22 | 14 | 10 | 134, 0, 0.14, 3 |
| heldout_v17_120day | 71 | exposure | elder | 56 (winter) | 2 | 31 (spring) | False | 2 | 0/10 | 8/8 | 22 / 17 | 14 | 10 | 134, 0, 0.14, 3 |
| heldout_v17_120day | 71 | exposure | elder | 56 (winter) | 2 | 31 (spring) | False | 2 | 0/10 | 8/8 | 22 / 16 | 14 | 8 | 133, 0, 0.12, 3 |
| heldout_v17_120day | 71 | exposure | elder | 57 (spring) | 2 | 31 (spring) | False | 2 | 0/10 | 8/8 | 22 / 22 | 14 | 7 | 120, 0, 0.00, 3 |
| heldout_v17_120day | 71 | exposure | elder | 57 (spring) | 2 | 31 (spring) | False | 2 | 0/10 | 8/8 | 22 / 18 | 14 | 7 | 120, 0, 0.00, 3 |
| heldout_v17_120day | 71 | exposure | adult | 57 (spring) | 2 | 31 (spring) | False | 2 | 0/10 | 8/8 | 22 / 0 | 14 | 7 | 120, 0, 0.00, 3 |
| heldout_v17_120day | 71 | exposure | elder | 57 (spring) | 2 | 31 (spring) | False | 2 | 0/10 | 8/8 | 22 / 19 | 14 | 4 | 120, 0, 0.00, 3 |
| heldout_v17_120day | 71 | exposure | adult | 82 (winter) | 2 | 31 (spring) | False | 2 | 0/10 | 8/8 | 22 / 15 | 14 | 3 | 20, 0, 0.03, 2 |
| heldout_v17_120day | 71 | exposure | elder | 82 (winter) | 2 | 31 (spring) | False | 2 | 0/10 | 8/8 | 22 / 14 | 14 | 3 | 10, 0, 0.03, 3 |
| heldout_v17_120day | 71 | exposure | elder | 82 (winter) | 2 | 31 (spring) | False | 2 | 0/10 | 8/8 | 22 / 22 | 14 | 2 | 10, 0, 0.03, 3 |
| heldout_v17_120day | 71 | exposure | elder | 112 (winter) | 3 | 86 (spring) | False | 2 | 0/10 | 8/8 | 22 / 6 | 14 | 11 | 109, 0, 0.12, 2 |
| heldout_v17_120day | 71 | exposure | adult | 112 (winter) | 3 | 86 (spring) | False | 2 | 0/10 | 8/8 | 22 / 13 | 14 | 10 | 109, 0, 0.08, 3 |
| heldout_v17_120day | 71 | exposure | adult | 113 (spring) | 3 | 86 (spring) | False | 2 | 0/10 | 8/8 | 22 / 16 | 14 | 9 | 91, 0, 0.00, 2 |
| heldout_v17_120day | 71 | exposure | elder | 113 (spring) | 3 | 86 (spring) | False | 2 | 0/10 | 8/8 | 22 / 22 | 14 | 8 | 90, 0, 0.00, 3 |
| heldout_v17_120day | 71 | exposure | adult | 113 (spring) | 3 | 86 (spring) | False | 2 | 0/10 | 8/8 | 22 / 22 | 14 | 7 | 89, 0, 0.00, 3 |
| heldout_v17_120day | 71 | exposure | elder | 113 (spring) | 3 | 86 (spring) | False | 2 | 0/10 | 8/8 | 22 / 22 | 14 | 6 | 88, 0, 0.00, 3 |
| heldout_v17_120day | 71 | exposure | adult | 113 (spring) | 3 | 86 (spring) | False | 2 | 0/10 | 8/8 | 22 / 22 | 14 | 6 | 88, 0, 0.00, 3 |
| heldout_v17_120day | 71 | exposure | adult | 113 (spring) | 3 | 86 (spring) | False | 2 | 0/10 | 8/8 | 22 / 21 | 14 | 6 | 88, 0, 0.00, 3 |
| heldout_v17_120day | 72 | exposure | elder | 84 (winter) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 25 | 14 | 35 | 255, 0, 0.30, 1 |
| heldout_v17_120day | 72 | exposure | child | 84 (winter) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 25 | 14 | 35 | 240, 0, 0.27, 2 |
| heldout_v17_120day | 72 | exposure | adult | 84 (winter) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 24 | 14 | 35 | 192, 0, 0.16, 2 |
| heldout_v17_120day | 72 | exposure | adult | 84 (winter) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 1 | 14 | 34 | 192, 0, 0.15, 2 |
| heldout_v17_120day | 72 | exposure | adult | 85 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 4 | 14 | 33 | 192, 0, 0.12, 2 |
| heldout_v17_120day | 72 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 18 | 14 | 33 | 188, 0, 0.11, 3 |
| heldout_v17_120day | 72 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 11 | 14 | 33 | 188, 0, 0.11, 3 |
| heldout_v17_120day | 72 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 25 | 14 | 31 | 185, 0, 0.10, 3 |
| heldout_v17_120day | 72 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 28 | 14 | 31 | 185, 0, 0.10, 3 |
| heldout_v17_120day | 72 | exposure | adult | 85 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 4 | 14 | 31 | 185, 0, 0.10, 3 |
| heldout_v17_120day | 72 | exposure | adult | 85 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 28 | 14 | 31 | 185, 0, 0.10, 3 |
| heldout_v17_120day | 72 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 5 | 14 | 27 | 181, 0, 0.08, 3 |
| heldout_v17_120day | 72 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 21 | 14 | 27 | 181, 0, 0.08, 3 |
| heldout_v17_120day | 72 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 26 | 14 | 27 | 181, 0, 0.08, 3 |
| heldout_v17_120day | 72 | exposure | adult | 85 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 11 | 14 | 27 | 181, 0, 0.08, 3 |
| heldout_v17_120day | 72 | exposure | elder | 85 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 14 | 14 | 23 | 179, 0, 0.07, 3 |
| heldout_v17_120day | 72 | exposure | adult | 112 (winter) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 9 | 14 | 35 | 269, 0, 0.18, 2 |
| heldout_v17_120day | 72 | exposure | adult | 112 (winter) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 8 | 14 | 34 | 266, 0, 0.16, 2 |
| heldout_v17_120day | 72 | exposure | adult | 113 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 3 | 14 | 35 | 230, 0, 0.04, 2 |
| heldout_v17_120day | 72 | exposure | adult | 113 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 18 | 14 | 34 | 225, 0, 0.01, 3 |
| heldout_v17_120day | 72 | exposure | adult | 113 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 28 | 14 | 34 | 225, 0, 0.01, 3 |
| heldout_v17_120day | 72 | exposure | adult | 113 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 17 | 14 | 34 | 225, 0, 0.01, 3 |
| heldout_v17_120day | 72 | exposure | elder | 113 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 25 | 14 | 31 | 225, 0, 0.00, 3 |
| heldout_v17_120day | 72 | exposure | adult | 113 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 8 | 14 | 31 | 225, 0, 0.00, 3 |
| heldout_v17_120day | 72 | exposure | adult | 113 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 8 | 14 | 29 | 221, 0, 0.00, 3 |
| heldout_v17_120day | 72 | dehydration | adult | 113 (spring) | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 / 3 | 14 | 29 | 193, 0, 0.00, 3 |
| heldout_v17_120day | 76 | exposure | child | 85 (spring) | 2 | 29 (spring) | False | 0 | 7/7 | 11/14 | 15 / 15 | 14 | 21 | 134, 0, 0.32, 2 |
| heldout_v17_120day | 76 | exposure | elder | 85 (spring) | 2 | 29 (spring) | False | 0 | 7/7 | 11/14 | 15 / 13 | 14 | 21 | 116, 0, 0.27, 2 |
| heldout_v17_120day | 77 | exposure | elder | 53 (winter) | 2 | 30 (spring) | False | 0 | 0/11 | 14/14 | 25 / 23 | 13 | 9 | 0, 0, 0.18, 3 |
| heldout_v17_120day | 77 | exposure | adult | 53 (winter) | 2 | 30 (spring) | False | 0 | 0/11 | 14/14 | 25 / 23 | 13 | 8 | 0, 0, 0.16, 3 |
| heldout_v17_120day | 77 | dehydration | adult | 53 (winter) | 2 | 30 (spring) | False | 0 | 0/11 | 14/14 | 25 / 4 | 13 | 7 | 0, 0, 0.12, 3 |
| heldout_v17_120day | 77 | exposure | adult | 54 (winter) | 2 | 30 (spring) | False | 0 | 0/11 | 14/14 | 25 / 23 | 13 | 6 | 0, 0, 0.07, 2 |
| heldout_v17_120day | 77 | exposure | elder | 54 (winter) | 2 | 30 (spring) | False | 0 | 0/11 | 14/14 | 25 / 23 | 13 | 5 | 0, 0, 0.07, 3 |
| heldout_v17_120day | 77 | exposure | elder | 54 (winter) | 2 | 30 (spring) | False | 0 | 0/11 | 14/14 | 25 / 22 | 13 | 4 | 0, 0, 0.07, 3 |
| heldout_v17_120day | 77 | dehydration | elder | 54 (winter) | 2 | 30 (spring) | False | 0 | 0/11 | 14/14 | 25 / 1 | 13 | 2 | 0, 0, 0.07, 3 |
| heldout_v17_120day | 77 | exposure | elder | 55 (winter) | 2 | 30 (spring) | False | 0 | 0/11 | 14/14 | 25 / 23 | 13 | 2 | 0, 0, 0.05, 3 |
| heldout_v17_120day | 77 | exposure | adult | 55 (winter) | 2 | 30 (spring) | False | 0 | 0/11 | 14/14 | 25 / 18 | 13 | 1 | 0, 0, 0.05, 3 |
| heldout_v17_120day | 77 | exposure | elder | 111 (winter) | 3 | 87 (spring) | False | 0 | 0/11 | 14/14 | 25 / 16 | 13 | 10 | 0, 0, 0.26, 2 |
| heldout_v17_120day | 77 | exposure | elder | 111 (winter) | 3 | 87 (spring) | False | 0 | 0/11 | 14/14 | 25 / 22 | 13 | 9 | 0, 0, 0.25, 2 |
| heldout_v17_120day | 77 | exposure | elder | 112 (winter) | 3 | 87 (spring) | False | 0 | 0/11 | 14/14 | 25 / 21 | 13 | 8 | 0, 0, 0.23, 2 |
| heldout_v17_120day | 77 | exposure | adult | 112 (winter) | 3 | 87 (spring) | False | 0 | 0/11 | 14/14 | 25 / 22 | 13 | 7 | 0, 0, 0.19, 2 |
| heldout_v17_120day | 77 | exposure | elder | 112 (winter) | 3 | 87 (spring) | False | 0 | 0/11 | 14/14 | 25 / 22 | 13 | 6 | 0, 0, 0.18, 2 |
| heldout_v17_120day | 77 | exposure | adult | 112 (winter) | 3 | 87 (spring) | False | 0 | 0/11 | 14/14 | 25 / 26 | 13 | 6 | 0, 0, 0.18, 2 |
| heldout_v17_120day | 77 | dehydration | adult | 112 (winter) | 3 | 87 (spring) | False | 0 | 0/11 | 14/14 | 25 / 4 | 13 | 4 | 0, 0, 0.14, 2 |
| heldout_v17_120day | 77 | exposure | adult | 113 (spring) | 3 | 87 (spring) | False | 0 | 0/11 | 14/14 | 25 / 22 | 13 | 3 | 0, 0, 0.04, 2 |
| heldout_v17_120day | 77 | exposure | elder | 113 (spring) | 3 | 87 (spring) | False | 0 | 0/11 | 14/14 | 25 / 22 | 13 | 2 | 0, 0, 0.04, 2 |
| heldout_v17_120day | 77 | exposure | adult | 113 (spring) | 3 | 87 (spring) | False | 0 | 0/11 | 14/14 | 25 / 22 | 13 | 1 | 0, 0, 0.04, 2 |
| heldout_v17_120day | 78 | exposure | adult | 55 (winter) | 2 | 29 (spring) | False | 0 | 0/9 | 14/14 | 6 / 7 | 13 | 9 | 0, 0, 0.37, 1 |
| heldout_v17_120day | 78 | starvation | elder | 55 (winter) | 2 | 29 (spring) | False | 0 | 0/9 | 14/14 | 6 / 2 | 13 | 8 | 0, 0, 0.36, 1 |
| heldout_v17_120day | 78 | starvation | elder | 55 (winter) | 2 | 29 (spring) | False | 0 | 0/9 | 14/14 | 6 / 4 | 13 | 7 | 0, 0, 0.34, 1 |
| heldout_v17_120day | 78 | starvation | elder | 55 (winter) | 2 | 29 (spring) | False | 0 | 0/9 | 14/14 | 6 / 8 | 13 | 6 | 0, 0, 0.33, 1 |
| heldout_v17_120day | 78 | starvation | elder | 55 (winter) | 2 | 29 (spring) | False | 0 | 0/9 | 14/14 | 6 / 8 | 13 | 6 | 0, 0, 0.33, 1 |
| heldout_v17_120day | 78 | starvation | adult | 55 (winter) | 2 | 29 (spring) | False | 0 | 0/9 | 14/14 | 6 / 8 | 13 | 4 | 0, 0, 0.32, 1 |
| heldout_v17_120day | 78 | exposure | adult | 56 (winter) | 2 | 29 (spring) | False | 0 | 0/9 | 14/14 | 6 / 8 | 13 | 3 | 0, 0, 0.27, 2 |
| heldout_v17_120day | 78 | exposure | elder | 56 (winter) | 2 | 29 (spring) | False | 0 | 0/9 | 14/14 | 6 / 2 | 13 | 2 | 0, 0, 0.26, 2 |
| heldout_v17_120day | 78 | starvation | elder | 56 (winter) | 2 | 29 (spring) | False | 0 | 0/9 | 14/14 | 6 / 2 | 13 | 1 | 0, 0, 0.19, 2 |
| heldout_v17_120day | 78 | exposure | elder | 111 (winter) | 3 | 85 (spring) | False | 0 | 0/9 | 14/14 | 6 / 11 | 13 | 8 | 0, 0, 0.51, 0 |
| heldout_v17_120day | 78 | exposure | elder | 112 (winter) | 3 | 85 (spring) | False | 0 | 0/9 | 14/14 | 6 / 10 | 13 | 7 | 0, 0, 0.42, 1 |
| heldout_v17_120day | 78 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 0 | 0/9 | 14/14 | 6 / 10 | 13 | 7 | 0, 0, 0.42, 1 |
| heldout_v17_120day | 78 | exposure | elder | 113 (spring) | 3 | 85 (spring) | False | 0 | 0/9 | 14/14 | 6 / 10 | 13 | 5 | 0, 0, 0.30, 1 |
| heldout_v17_120day | 78 | exposure | elder | 113 (spring) | 3 | 85 (spring) | False | 0 | 0/9 | 14/14 | 6 / 10 | 13 | 5 | 0, 0, 0.30, 1 |
| heldout_v17_120day | 78 | exposure | elder | 113 (spring) | 3 | 85 (spring) | False | 0 | 0/9 | 14/14 | 6 / 10 | 13 | 3 | 0, 0, 0.29, 2 |
| heldout_v17_120day | 79 | exposure | child | 111 (winter) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 15 | 13 | 36 | 273, 0, 0.34, 1 |
| heldout_v17_120day | 79 | dehydration | adult | 111 (winter) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 1 | 13 | 36 | 267, 0, 0.27, 2 |
| heldout_v17_120day | 79 | dehydration | adult | 111 (winter) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 1 | 13 | 38 | 294, 0, 0.34, 1 |
| heldout_v17_120day | 79 | dehydration | adult | 111 (winter) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 2 | 13 | 36 | 294, 0, 0.34, 1 |
| heldout_v17_120day | 79 | dehydration | adult | 111 (winter) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 4 | 13 | 36 | 294, 0, 0.34, 1 |
| heldout_v17_120day | 79 | dehydration | adult | 111 (winter) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 19 | 13 | 35 | 294, 0, 0.34, 1 |
| heldout_v17_120day | 79 | exposure | adult | 111 (winter) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 16 | 13 | 36 | 249, 0, 0.22, 2 |
| heldout_v17_120day | 79 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 3 | 13 | 35 | 249, 0, 0.19, 2 |
| heldout_v17_120day | 79 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 4 | 13 | 35 | 249, 0, 0.19, 2 |
| heldout_v17_120day | 79 | exposure | adult | 112 (winter) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 22 | 13 | 34 | 272, 0, 0.26, 2 |
| heldout_v17_120day | 79 | exposure | adult | 112 (winter) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 23 | 13 | 34 | 272, 0, 0.26, 2 |
| heldout_v17_120day | 79 | exposure | adult | 112 (winter) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 18 | 13 | 32 | 266, 0, 0.25, 2 |
| heldout_v17_120day | 79 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 3 | 13 | 34 | 231, 0, 0.16, 3 |
| heldout_v17_120day | 79 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 17 | 13 | 33 | 226, 0, 0.15, 3 |
| heldout_v17_120day | 79 | exposure | adult | 112 (winter) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 22 | 13 | 31 | 264, 0, 0.23, 2 |
| heldout_v17_120day | 79 | exposure | elder | 112 (winter) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 19 | 13 | 30 | 258, 0, 0.22, 2 |
| heldout_v17_120day | 79 | exposure | elder | 112 (winter) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 19 | 13 | 32 | 221, 0, 0.14, 3 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 15 | 13 | 32 | 205, 0, 0.07, 2 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 5 | 13 | 32 | 205, 0, 0.07, 2 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 23 | 13 | 31 | 235, 0, 0.10, 2 |
| heldout_v17_120day | 79 | exposure | child | 113 (spring) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 19 | 13 | 30 | 205, 0, 0.07, 2 |
| heldout_v17_120day | 79 | exposure | elder | 113 (spring) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 19 | 13 | 30 | 232, 0, 0.08, 3 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 2 | 13 | 30 | 205, 0, 0.07, 3 |
| heldout_v17_120day | 79 | exposure | child | 113 (spring) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 23 | 13 | 30 | 232, 0, 0.08, 3 |
| heldout_v17_120day | 79 | exposure | elder | 113 (spring) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 19 | 13 | 28 | 231, 0, 0.07, 3 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 19 | 13 | 29 | 205, 0, 0.07, 3 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 22 | 13 | 28 | 231, 0, 0.07, 3 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 17 | 13 | 28 | 205, 0, 0.07, 3 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 21 | 13 | 26 | 229, 0, 0.05, 3 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 / 20 | 13 | 26 | 229, 0, 0.05, 3 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 2 | 13 | 28 | 205, 0, 0.07, 3 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 17 | 13 | 28 | 205, 0, 0.07, 3 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 17 | 13 | 28 | 205, 0, 0.07, 3 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 8 | 13 | 24 | 205, 0, 0.07, 3 |
| heldout_v17_120day | 79 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 / 17 | 13 | 24 | 205, 0, 0.07, 3 |
| heldout_v17_120day | 80 | exposure | elder | 54 (winter) | 2 | 32 (spring) | False | 0 | 0/7 | 9/9 | 23 / 24 | 14 | 11 | 14, 0, 0.15, 3 |
| heldout_v17_120day | 80 | exposure | adult | 55 (winter) | 2 | 32 (spring) | False | 0 | 0/7 | 9/9 | 23 / 25 | 14 | 10 | 3, 0, 0.05, 2 |
| heldout_v17_120day | 80 | exposure | elder | 55 (winter) | 2 | 32 (spring) | False | 0 | 0/7 | 9/9 | 23 / 17 | 14 | 9 | 3, 0, 0.05, 3 |
| heldout_v17_120day | 80 | exposure | adult | 55 (winter) | 2 | 32 (spring) | False | 0 | 0/7 | 9/9 | 23 / 24 | 14 | 9 | 3, 0, 0.05, 3 |
| heldout_v17_120day | 80 | exposure | adult | 55 (winter) | 2 | 32 (spring) | False | 0 | 0/7 | 9/9 | 23 / 8 | 14 | 9 | 3, 0, 0.05, 3 |
| heldout_v17_120day | 80 | exposure | elder | 55 (winter) | 2 | 32 (spring) | False | 0 | 0/7 | 9/9 | 23 / 24 | 14 | 6 | 3, 0, 0.05, 3 |
| heldout_v17_120day | 80 | exposure | adult | 55 (winter) | 2 | 32 (spring) | False | 0 | 0/7 | 9/9 | 23 / 24 | 14 | 6 | 3, 0, 0.05, 3 |
| heldout_v17_120day | 80 | exposure | adult | 55 (winter) | 2 | 32 (spring) | False | 0 | 0/7 | 9/9 | 23 / 24 | 14 | 6 | 3, 0, 0.05, 3 |
| heldout_v17_120day | 80 | exposure | elder | 57 (spring) | 2 | 32 (spring) | False | 0 | 0/7 | 9/9 | 23 / 14 | 14 | 3 | 0, 0, 0.04, 2 |
| heldout_v17_120day | 80 | exposure | elder | 57 (spring) | 2 | 32 (spring) | False | 0 | 0/7 | 9/9 | 23 / 13 | 14 | 2 | 0, 0, 0.04, 3 |
| heldout_v17_120day | 80 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 9/9 | 23 / 16 | 14 | 9 | 0, 0, 0.11, 2 |
| heldout_v17_120day | 80 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 9/9 | 23 / 24 | 14 | 8 | 0, 0, 0.08, 3 |
| heldout_v17_120day | 80 | exposure | elder | 109 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 9/9 | 23 / 3 | 14 | 7 | 0, 0, 0.07, 3 |
| heldout_v17_120day | 80 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 9/9 | 23 / 24 | 14 | 7 | 0, 0, 0.07, 3 |
| heldout_v17_120day | 80 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 9/9 | 23 / 23 | 14 | 7 | 0, 0, 0.07, 3 |
| heldout_v17_120day | 80 | exposure | elder | 110 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 9/9 | 23 / 24 | 14 | 5 | 0, 0, 0.01, 3 |
| heldout_v17_120day | 80 | exposure | adult | 110 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 9/9 | 23 / 24 | 14 | 4 | 0, 0, 0.01, 3 |
| heldout_v17_120day | 80 | exposure | elder | 111 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 9/9 | 23 / 15 | 14 | 3 | 0, 0, 0.00, 3 |
| heldout_v17_120day | 80 | exposure | child | 111 (winter) | 3 | 85 (spring) | False | 0 | 0/7 | 9/9 | 23 / 24 | 14 | 2 | 0, 0, 0.00, 3 |
| heldout_v17_120day | 84 | exposure | adult | 57 (spring) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 15 | 12 | 27 | 199, 0, 0.08, 3 |
| heldout_v17_120day | 84 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 16 | 12 | 26 | 199, 0, 0.07, 3 |
| heldout_v17_120day | 84 | exposure | adult | 57 (spring) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 15 | 12 | 26 | 199, 0, 0.07, 3 |
| heldout_v17_120day | 84 | exposure | child | 111 (winter) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 16 | 12 | 27 | 208, 0, 0.32, 2 |
| heldout_v17_120day | 84 | exposure | child | 111 (winter) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 16 | 12 | 48 | 352, 0, 0.33, 2 |
| heldout_v17_120day | 84 | exposure | adult | 111 (winter) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 1 | 12 | 48 | 352, 0, 0.32, 2 |
| heldout_v17_120day | 84 | exposure | elder | 111 (winter) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 16 | 12 | 26 | 202, 0, 0.27, 2 |
| heldout_v17_120day | 84 | exposure | elder | 112 (winter) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 18 | 12 | 48 | 352, 0, 0.16, 2 |
| heldout_v17_120day | 84 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 15 | 12 | 25 | 186, 0, 0.14, 3 |
| heldout_v17_120day | 84 | exposure | elder | 112 (winter) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 16 | 12 | 48 | 352, 0, 0.14, 3 |
| heldout_v17_120day | 84 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 17 | 12 | 48 | 352, 0, 0.14, 3 |
| heldout_v17_120day | 84 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 17 | 12 | 46 | 352, 0, 0.12, 3 |
| heldout_v17_120day | 84 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 17 | 12 | 46 | 352, 0, 0.12, 3 |
| heldout_v17_120day | 84 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 16 | 12 | 24 | 171, 0, 0.10, 3 |
| heldout_v17_120day | 84 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 7 | 12 | 44 | 352, 0, 0.11, 3 |
| heldout_v17_120day | 84 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 12 | 12 | 44 | 352, 0, 0.11, 3 |
| heldout_v17_120day | 84 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 16 | 12 | 44 | 352, 0, 0.11, 3 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 17 | 12 | 42 | 337, 0, 0.01, 2 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 9 | 12 | 41 | 337, 0, 0.01, 3 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 17 | 12 | 41 | 337, 0, 0.01, 3 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 10 | 12 | 41 | 337, 0, 0.01, 3 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 16 | 12 | 41 | 337, 0, 0.01, 3 |
| heldout_v17_120day | 84 | exposure | child | 113 (spring) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 13 | 12 | 24 | 153, 0, 0.03, 3 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 13 | 12 | 24 | 151, 0, 0.03, 3 |
| heldout_v17_120day | 84 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 16 | 12 | 23 | 147, 0, 0.03, 3 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 17 | 12 | 38 | 336, 0, 0.01, 3 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 7 | 12 | 23 | 147, 0, 0.03, 3 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 13 | 12 | 23 | 147, 0, 0.03, 3 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 18 | 12 | 38 | 336, 0, 0.01, 3 |
| heldout_v17_120day | 84 | exposure | child | 113 (spring) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 19 | 12 | 38 | 336, 0, 0.01, 3 |
| heldout_v17_120day | 84 | exposure | child | 113 (spring) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 13 | 12 | 23 | 147, 0, 0.03, 3 |
| heldout_v17_120day | 84 | exposure | child | 113 (spring) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 19 | 12 | 38 | 336, 0, 0.01, 3 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 18 | 12 | 34 | 331, 0, 0.01, 3 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 / 18 | 12 | 34 | 331, 0, 0.01, 3 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 15 | 12 | 19 | 145, 0, 0.03, 3 |
| heldout_v17_120day | 84 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 / 10 | 12 | 19 | 145, 0, 0.03, 3 |
| heldout_v17_120day | 85 | exposure | child | 80 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 17 | 14 | 10 | 63, 0, 0.33, 2 |
| heldout_v17_120day | 85 | exposure | child | 80 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 17 | 14 | 9 | 63, 0, 0.32, 2 |
| heldout_v17_120day | 85 | exposure | child | 81 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 17 | 14 | 10 | 54, 0, 0.19, 3 |
| heldout_v17_120day | 85 | exposure | adult | 81 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 13 | 14 | 10 | 38, 0, 0.11, 3 |
| heldout_v17_120day | 85 | exposure | elder | 82 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 0 | 14 | 9 | 38, 0, 0.10, 2 |
| heldout_v17_120day | 85 | exposure | adult | 82 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 17 | 14 | 9 | 27, 0, 0.08, 3 |
| heldout_v17_120day | 85 | exposure | adult | 82 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 9 | 14 | 8 | 27, 0, 0.08, 3 |
| heldout_v17_120day | 85 | exposure | child | 82 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 17 | 14 | 8 | 27, 0, 0.08, 3 |
| heldout_v17_120day | 85 | exposure | child | 82 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 17 | 14 | 6 | 25, 0, 0.08, 3 |
| heldout_v17_120day | 85 | exposure | child | 83 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 17 | 14 | 7 | 6, 0, 0.14, 3 |
| heldout_v17_120day | 85 | exposure | child | 84 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 17 | 14 | 6 | 2, 0, 0.08, 2 |
| heldout_v17_120day | 85 | exposure | child | 84 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 17 | 14 | 5 | 2, 0, 0.08, 3 |
| heldout_v17_120day | 85 | exposure | child | 85 (spring) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 17 | 14 | 4 | 2, 0, 0.07, 3 |
| heldout_v17_120day | 85 | dehydration | adult | 108 (winter) | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 / 6 | 13 | 13 | 0, 0, 0.22, 2 |
| heldout_v17_120day | 85 | dehydration | adult | 108 (winter) | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 / 15 | 13 | 12 | 0, 0, 0.22, 2 |
| heldout_v17_120day | 85 | dehydration | adult | 108 (winter) | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 / 20 | 13 | 11 | 0, 0, 0.22, 2 |
| heldout_v17_120day | 85 | dehydration | adult | 108 (winter) | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 / 4 | 13 | 10 | 0, 0, 0.22, 2 |
| heldout_v17_120day | 85 | exposure | elder | 108 (winter) | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 / 8 | 13 | 9 | 0, 0, 0.18, 2 |
| heldout_v17_120day | 85 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 / 1 | 13 | 8 | 0, 0, 0.15, 2 |
| heldout_v17_120day | 85 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 / 3 | 13 | 8 | 0, 0, 0.15, 2 |
| heldout_v17_120day | 85 | exposure | elder | 109 (winter) | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 / 2 | 13 | 6 | 0, 0, 0.12, 3 |
| heldout_v17_120day | 85 | exposure | elder | 109 (winter) | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 / 2 | 13 | 6 | 0, 0, 0.12, 3 |
| heldout_v17_120day | 85 | dehydration | adult | 109 (winter) | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 / 3 | 13 | 4 | 0, 0, 0.04, 3 |
| heldout_v17_120day | 85 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 / 1 | 13 | 3 | 0, 0, 0.00, 3 |
| heldout_v17_120day | 85 | exposure | adult | 110 (winter) | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 / 1 | 13 | 2 | 0, 0, 0.00, 2 |
| heldout_v17_120day | 85 | exposure | adult | 110 (winter) | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 / 11 | 13 | 1 | 0, 0, 0.00, 3 |
| heldout_v17_120day | 85 | exposure | elder | 112 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 3 | 14 | 10 | 71, 0, 0.18, 2 |
| heldout_v17_120day | 85 | exposure | adult | 112 (winter) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 5 | 14 | 10 | 72, 0, 0.08, 2 |
| heldout_v17_120day | 85 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 17 | 14 | 10 | 70, 0, 0.03, 3 |
| heldout_v17_120day | 85 | exposure | adult | 113 (spring) | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 / 17 | 14 | 9 | 70, 0, 0.01, 3 |
| heldout_v17_120day | 87 | exposure | elder | 52 (winter) | 2 | 29 (spring) | False | 0 | 1/8 | 8/8 | 26 / 27 | 14 | 11 | 0, 0, 0.25, 2 |
| heldout_v17_120day | 87 | exposure | adult | 53 (winter) | 2 | 29 (spring) | False | 0 | 1/8 | 8/8 | 26 / 26 | 14 | 10 | 0, 0, 0.11, 3 |
| heldout_v17_120day | 87 | exposure | elder | 53 (winter) | 2 | 29 (spring) | False | 0 | 1/8 | 8/8 | 26 / 22 | 14 | 9 | 0, 0, 0.08, 3 |
| heldout_v17_120day | 87 | exposure | adult | 53 (winter) | 2 | 29 (spring) | False | 0 | 1/8 | 8/8 | 26 / 27 | 14 | 9 | 0, 0, 0.08, 3 |
| heldout_v17_120day | 87 | exposure | adult | 53 (winter) | 2 | 29 (spring) | False | 0 | 1/8 | 8/8 | 26 / 27 | 14 | 9 | 0, 0, 0.08, 3 |
| heldout_v17_120day | 87 | exposure | adult | 54 (winter) | 2 | 29 (spring) | False | 0 | 1/8 | 8/8 | 26 / 22 | 14 | 6 | 0, 0, 0.00, 2 |
| heldout_v17_120day | 87 | exposure | adult | 54 (winter) | 2 | 29 (spring) | False | 0 | 1/8 | 8/8 | 26 / 18 | 14 | 5 | 0, 0, 0.00, 3 |
| heldout_v17_120day | 87 | exposure | elder | 54 (winter) | 2 | 29 (spring) | False | 0 | 1/8 | 8/8 | 26 / 26 | 14 | 4 | 0, 0, 0.00, 3 |
| heldout_v17_120day | 87 | exposure | elder | 55 (winter) | 2 | 29 (spring) | False | 0 | 1/8 | 8/8 | 26 / 19 | 14 | 3 | 0, 0, 0.00, 3 |
| heldout_v17_120day | 87 | exposure | elder | 55 (winter) | 2 | 29 (spring) | False | 0 | 1/8 | 8/8 | 26 / 19 | 14 | 3 | 0, 0, 0.00, 3 |
| heldout_v17_120day | 87 | exposure | elder | 55 (winter) | 2 | 29 (spring) | False | 0 | 1/8 | 8/8 | 26 / 12 | 14 | 1 | 0, 0, 0.00, 3 |
| heldout_v17_120day | 87 | exposure | elder | 108 (winter) | 3 | 85 (spring) | False | 0 | 1/8 | 8/8 | 26 / 20 | 14 | 9 | 0, 0, 0.12, 3 |
| heldout_v17_120day | 87 | starvation | adult | 109 (winter) | 3 | 85 (spring) | False | 0 | 1/8 | 8/8 | 26 / 11 | 14 | 8 | 0, 0, 0.12, 2 |
| heldout_v17_120day | 87 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 0 | 1/8 | 8/8 | 26 / 15 | 14 | 7 | 0, 0, 0.12, 3 |
| heldout_v17_120day | 87 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 0 | 1/8 | 8/8 | 26 / 28 | 14 | 6 | 0, 0, 0.12, 3 |
| heldout_v17_120day | 87 | exposure | adult | 109 (winter) | 3 | 85 (spring) | False | 0 | 1/8 | 8/8 | 26 / 24 | 14 | 6 | 0, 0, 0.12, 3 |
| heldout_v17_120day | 87 | exposure | adult | 110 (winter) | 3 | 85 (spring) | False | 0 | 1/8 | 8/8 | 26 / 15 | 14 | 4 | 0, 0, 0.11, 2 |
| heldout_v17_120day | 87 | exposure | adult | 110 (winter) | 3 | 85 (spring) | False | 0 | 1/8 | 8/8 | 26 / 24 | 14 | 3 | 0, 0, 0.12, 3 |
| heldout_v17_120day | 87 | exposure | adult | 110 (winter) | 3 | 85 (spring) | False | 0 | 1/8 | 8/8 | 26 / 2 | 14 | 2 | 0, 0, 0.12, 3 |
| heldout_v17_120day | 90 | exposure | child | 85 (spring) | 2 | 29 (spring) | False | 0 | 9/9 | 14/14 | 19 / 21 | 14 | 24 | 154, 0, 0.27, 2 |
| heldout_v17_120day | 90 | exposure | child | 85 (spring) | 2 | 29 (spring) | False | 0 | 9/9 | 14/14 | 19 / 21 | 14 | 24 | 154, 0, 0.26, 2 |
| heldout_v17_120day | 91 | exposure | adult | 57 (spring) | 2 | 29 (spring) | False | 0 | 10/10 | 13/13 | 18 / 16 | 14 | 35 | 272, 0, 0.22, 2 |
| heldout_v17_120day | 93 | exposure | adult | 57 (spring) | 2 | 29 (spring) | False | 0 | 2/7 | 6/11 | 12 / 11 | 14 | 13 | 79, 0, 0.12, 3 |
| heldout_v17_120day | 93 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 2/7 | 6/11 | 12 / 11 | 14 | 13 | 73, 0, 0.11, 3 |
| heldout_v17_120day | 93 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 2/7 | 6/11 | 12 / 11 | 14 | 13 | 73, 0, 0.11, 3 |
| heldout_v17_120day | 93 | exposure | elder | 57 (spring) | 2 | 29 (spring) | False | 0 | 2/7 | 6/11 | 12 / 11 | 14 | 11 | 71, 0, 0.10, 3 |
| heldout_v17_120day | 93 | exposure | adult | 57 (spring) | 2 | 29 (spring) | False | 0 | 2/7 | 6/11 | 12 / 11 | 14 | 11 | 71, 0, 0.10, 3 |
| heldout_v17_120day | 93 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 7/7 | 11/11 | 17 / 19 | 12 | 32 | 245, 0, 0.27, 2 |
| heldout_v17_120day | 93 | exposure | child | 112 (winter) | 3 | 85 (spring) | False | 1 | 7/7 | 11/11 | 17 / 19 | 12 | 31 | 245, 0, 0.25, 2 |
| heldout_v17_120day | 93 | exposure | child | 112 (winter) | 2 | 29 (spring) | False | 0 | 2/7 | 6/11 | 12 / 12 | 14 | 13 | 77, 0, 0.18, 2 |
| heldout_v17_120day | 93 | exposure | adult | 112 (winter) | 3 | 85 (spring) | False | 1 | 7/7 | 11/11 | 17 / 20 | 12 | 32 | 223, 0, 0.16, 2 |
| heldout_v17_120day | 93 | exposure | child | 113 (spring) | 3 | 85 (spring) | False | 1 | 7/7 | 11/11 | 17 / 19 | 12 | 31 | 223, 0, 0.11, 3 |
| heldout_v17_120day | 93 | exposure | elder | 113 (spring) | 3 | 85 (spring) | False | 1 | 7/7 | 11/11 | 17 / 5 | 12 | 30 | 221, 0, 0.10, 3 |
| heldout_v17_120day | 93 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 0 | 2/7 | 6/11 | 12 / 7 | 14 | 13 | 37, 0, 0.01, 3 |
| heldout_v17_120day | 93 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 7/7 | 11/11 | 17 / 1 | 12 | 29 | 217, 0, 0.08, 3 |
| heldout_v17_120day | 93 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 7/7 | 11/11 | 17 / 23 | 12 | 29 | 217, 0, 0.08, 3 |
| heldout_v17_120day | 93 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 7/7 | 11/11 | 17 / 18 | 12 | 29 | 217, 0, 0.08, 3 |
| heldout_v17_120day | 93 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 7/7 | 11/11 | 17 / 16 | 12 | 29 | 217, 0, 0.08, 3 |
| heldout_v17_120day | 93 | exposure | adult | 113 (spring) | 3 | 85 (spring) | False | 1 | 7/7 | 11/11 | 17 / 2 | 12 | 29 | 217, 0, 0.08, 3 |
| heldout_v17_120day | 93 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 0 | 2/7 | 6/11 | 12 / 11 | 14 | 12 | 37, 0, 0.01, 3 |
| heldout_v17_120day | 93 | exposure | elder | 113 (spring) | 2 | 29 (spring) | False | 0 | 2/7 | 6/11 | 12 / 2 | 14 | 12 | 37, 0, 0.01, 3 |

## Winters over the 0.35 bound (world ratio or any settlement-winter ratio)

- v17_120day seed 9, winter from day 50 (severity 2): world 22/63 = 0.35, world dark nights 4, deadwood per spot at the start 0.04 (25 spots)
  - settlement 1: deaths 10 / members 50.0 = 0.20, dark nights 4, wood at the start 206.0
  - settlement 2 (daughter): deaths 12 / members 13.0 = 0.92, dark nights 4, wood at the start 192.0
- v17_120day seed 9, winter from day 106 (severity 0): world 7/62 = 0.11, world dark nights 2, deadwood per spot at the start 1.00 (25 spots)
  - settlement 1: deaths 0 / members 50.0 = 0.00, dark nights 1, wood at the start 195.0
  - settlement 3 (daughter): deaths 7 / members 12.0 = 0.58, dark nights 2, wood at the start 181.0
- v17_120day seed 15, winter from day 78 (severity 2): world 18/64 = 0.28, world dark nights 4, deadwood per spot at the start 0.51 (47 spots)
  - settlement 1: deaths 3 / members 26.0 = 0.12, dark nights 4, wood at the start 194.0
  - settlement 2 (daughter): deaths 15 / members 38.0 = 0.39, dark nights 4, wood at the start 212.0
- v17_120day seed 15, winter from day 106 (severity 2): world 12/77 = 0.16, world dark nights 5, deadwood per spot at the start 0.32 (47 spots)
  - settlement 1: deaths 0 / members 26.0 = 0.00, dark nights 3, wood at the start 177.0
  - settlement 2 (daughter): deaths 1 / members 38.0 = 0.03, dark nights 2, wood at the start 213.0
  - settlement 3 (daughter): deaths 11 / members 13.0 = 0.85, dark nights 5, wood at the start 77.0
- v17_120day seed 16, winter from day 78 (severity 2): world 9/43 = 0.21, world dark nights 4, deadwood per spot at the start 0.48 (64 spots)
  - settlement 1: deaths 5 / members 37.0 = 0.14, dark nights 3, wood at the start 196.0
  - settlement 2 (daughter): deaths 4 / members 6.0 = 0.67, dark nights 4, wood at the start 165.0
- v17_120day seed 18, winter from day 50 (severity 1): world 7/41 = 0.17, world dark nights 4, deadwood per spot at the start 0.71 (56 spots)
  - settlement 1: deaths 0 / members 31.0 = 0.00, dark nights 2, wood at the start 193.0
  - settlement 2 (daughter): deaths 7 / members 10.0 = 0.70, dark nights 4, wood at the start 128.0
- v17_120day seed 18, winter from day 78 (severity 0): world 3/37 = 0.08, world dark nights 6, deadwood per spot at the start 0.89 (56 spots)
  - settlement 1: deaths 0 / members 31.0 = 0.00, dark nights 0, wood at the start 195.0
  - settlement 2 (daughter): deaths 3 / members 6.0 = 0.50, dark nights 6, wood at the start 30.0
- v17_120day seed 19, winter from day 50 (severity 0): world 5/41 = 0.12, world dark nights 3, deadwood per spot at the start 0.92 (49 spots)
  - settlement 1: deaths 0 / members 33.0 = 0.00, dark nights 0, wood at the start 206.0
  - settlement 2 (daughter): deaths 5 / members 8.0 = 0.62, dark nights 3, wood at the start 104.0
- v17_120day seed 19, winter from day 106 (severity 0): world 3/38 = 0.08, world dark nights 2, deadwood per spot at the start 0.90 (49 spots)
  - settlement 1: deaths 0 / members 33.0 = 0.00, dark nights 0, wood at the start 190.0
  - settlement 3 (daughter): deaths 3 / members 5.0 = 0.60, dark nights 2, wood at the start 5.0
- v17_120day seed 23, winter from day 106 (severity 2): world 10/70 = 0.14, world dark nights 7, deadwood per spot at the start 0.16 (38 spots)
  - settlement 1: deaths 0 / members 34.0 = 0.00, dark nights 3, wood at the start 183.0
  - settlement 2 (daughter): deaths 1 / members 26.0 = 0.04, dark nights 3, wood at the start 198.0
  - settlement 3 (daughter): deaths 9 / members 10.0 = 0.90, dark nights 6, wood at the start 3.0
- v17_120day seed 24, winter from day 106 (severity 2): world 35/76 = 0.46, world dark nights 4, deadwood per spot at the start 0.60 (65 spots)
  - settlement 1: deaths 8 / members 28.0 = 0.29, dark nights 4, wood at the start 195.0
  - settlement 2 (daughter): deaths 27 / members 48.0 = 0.56, dark nights 4, wood at the start 208.0
- v17_120day seed 25, winter from day 106 (severity 2): world 11/82 = 0.13, world dark nights 7, deadwood per spot at the start 0.61 (49 spots)
  - settlement 1: deaths 1 / members 46.0 = 0.02, dark nights 2, wood at the start 208.0
  - settlement 2 (daughter): deaths 0 / members 24.0 = 0.00, dark nights 2, wood at the start 213.0
  - settlement 3 (daughter): deaths 10 / members 12.0 = 0.83, dark nights 6, wood at the start 2.0
- v17_120day seed 27, winter from day 78 (severity 2): world 15/66 = 0.23, world dark nights 3, deadwood per spot at the start 0.40 (52 spots)
  - settlement 1: deaths 3 / members 36.0 = 0.08, dark nights 3, wood at the start 186.0
  - settlement 2 (daughter): deaths 12 / members 30.0 = 0.40, dark nights 3, wood at the start 205.0
- v17_120day seed 27, winter from day 106 (severity 2): world 39/92 = 0.42, world dark nights 4, deadwood per spot at the start 0.19 (52 spots)
  - settlement 1: deaths 4 / members 36.0 = 0.11, dark nights 4, wood at the start 191.0
  - settlement 2 (daughter): deaths 19 / members 30.0 = 0.63, dark nights 4, wood at the start 193.0
  - settlement 3 (daughter): deaths 16 / members 26.0 = 0.62, dark nights 4, wood at the start 201.0
- v17_120day seed 28, winter from day 106 (severity 1): world 7/89 = 0.08, world dark nights 3, deadwood per spot at the start 0.80 (65 spots)
  - settlement 1: deaths 0 / members 35.0 = 0.00, dark nights 1, wood at the start 199.0
  - settlement 2 (daughter): deaths 1 / members 42.0 = 0.02, dark nights 1, wood at the start 201.0
  - settlement 3 (daughter): deaths 6 / members 12.0 = 0.50, dark nights 2, wood at the start 30.0
- heldout_v17_120day seed 38, winter from day 78 (severity 2): world 27/74 = 0.36, world dark nights 5, deadwood per spot at the start 0.00 (28 spots)
  - settlement 1: deaths 4 / members 48.0 = 0.08, dark nights 3, wood at the start 196.0
  - settlement 2 (daughter): deaths 23 / members 26.0 = 0.88, dark nights 5, wood at the start 150.0
- heldout_v17_120day seed 39, winter from day 106 (severity 2): world 24/86 = 0.28, world dark nights 4, deadwood per spot at the start 0.03 (38 spots)
  - settlement 1: deaths 2 / members 33.0 = 0.06, dark nights 4, wood at the start 215.0
  - settlement 2 (daughter): deaths 5 / members 30.0 = 0.17, dark nights 4, wood at the start 182.0
  - settlement 3 (daughter): deaths 17 / members 23.0 = 0.74, dark nights 4, wood at the start 181.0
- heldout_v17_120day seed 44, winter from day 106 (severity 0): world 4/52 = 0.08, world dark nights 3, deadwood per spot at the start 1.00 (36 spots)
  - settlement 1: deaths 0 / members 42.0 = 0.00, dark nights 2, wood at the start 184.0
  - settlement 2 (daughter): deaths 4 / members 10.0 = 0.40, dark nights 3, wood at the start 180.0
- heldout_v17_120day seed 45, winter from day 50 (severity 2): world 6/49 = 0.12, world dark nights 3, deadwood per spot at the start 0.00 (22 spots)
  - settlement 1: deaths 0 / members 36.0 = 0.00, dark nights 3, wood at the start 202.0
  - settlement 2 (daughter): deaths 6 / members 13.0 = 0.46, dark nights 3, wood at the start 191.0
- heldout_v17_120day seed 47, winter from day 78 (severity 2): world 30/63 = 0.48, world dark nights 4, deadwood per spot at the start 0.00 (27 spots)
  - settlement 1: deaths 7 / members 34.0 = 0.21, dark nights 4, wood at the start 207.0
  - settlement 2 (daughter): deaths 23 / members 29.0 = 0.79, dark nights 4, wood at the start 192.0
- heldout_v17_120day seed 48, winter from day 50 (severity 1): world 5/46 = 0.11, world dark nights 5, deadwood per spot at the start 0.73 (41 spots)
  - settlement 1: deaths 0 / members 37.0 = 0.00, dark nights 1, wood at the start 200.0
  - settlement 2 (daughter): deaths 5 / members 9.0 = 0.56, dark nights 4, wood at the start 12.0
- heldout_v17_120day seed 48, winter from day 106 (severity 1): world 8/49 = 0.16, world dark nights 5, deadwood per spot at the start 0.93 (41 spots)
  - settlement 1: deaths 0 / members 37.0 = 0.00, dark nights 2, wood at the start 188.0
  - settlement 3 (daughter): deaths 8 / members 12.0 = 0.67, dark nights 3, wood at the start 2.0
- heldout_v17_120day seed 56, winter from day 78 (severity 2): world 11/63 = 0.17, world dark nights 3, deadwood per spot at the start 0.24 (41 spots)
  - settlement 1: deaths 0 / members 33.0 = 0.00, dark nights 2, wood at the start 198.0
  - settlement 2 (daughter): deaths 11 / members 30.0 = 0.37, dark nights 3, wood at the start 187.0
- heldout_v17_120day seed 59, winter from day 106 (severity 0): world 7/90 = 0.08, world dark nights 3, deadwood per spot at the start 0.87 (46 spots)
  - settlement 1: deaths 0 / members 39.0 = 0.00, dark nights 0, wood at the start 203.0
  - settlement 2 (daughter): deaths 0 / members 43.0 = 0.00, dark nights 0, wood at the start 190.0
  - settlement 3 (daughter): deaths 7 / members 8.0 = 0.88, dark nights 3, wood at the start 6.0
- heldout_v17_120day seed 67, winter from day 106 (severity 1): world 11/43 = 0.26, world dark nights 4, deadwood per spot at the start 0.62 (32 spots)
  - settlement 1: deaths 0 / members 28.0 = 0.00, dark nights 3, wood at the start 200.0
  - settlement 2 (daughter): deaths 11 / members 15.0 = 0.73, dark nights 4, wood at the start 162.0
- heldout_v17_120day seed 68, winter from day 106 (severity 1): world 21/89 = 0.24, world dark nights 5, deadwood per spot at the start 0.68 (40 spots)
  - settlement 1: deaths 0 / members 47.0 = 0.00, dark nights 2, wood at the start 172.0
  - settlement 2 (daughter): deaths 0 / members 22.0 = 0.00, dark nights 1, wood at the start 185.0
  - settlement 3 (daughter): deaths 21 / members 20.0 = 1.05, dark nights 5, wood at the start 54.0
- heldout_v17_120day seed 71, winter from day 50 (severity 1): world 10/54 = 0.19, world dark nights 4, deadwood per spot at the start 1.00 (50 spots)
  - settlement 1: deaths 2 / members 41.0 = 0.05, dark nights 3, wood at the start 186.0
  - settlement 2 (daughter): deaths 8 / members 13.0 = 0.62, dark nights 4, wood at the start 169.0
- heldout_v17_120day seed 71, winter from day 78 (severity 1): world 6/44 = 0.14, world dark nights 6, deadwood per spot at the start 0.94 (50 spots)
  - settlement 1: deaths 3 / members 41.0 = 0.07, dark nights 2, wood at the start 193.0
  - settlement 2 (daughter): deaths 3 / members 3.0 = 1.00, dark nights 4, wood at the start 3.0
- heldout_v17_120day seed 71, winter from day 106 (severity 2): world 9/55 = 0.16, world dark nights 4, deadwood per spot at the start 0.34 (50 spots)
  - settlement 1: deaths 1 / members 43.0 = 0.02, dark nights 4, wood at the start 191.0
  - settlement 3 (daughter): deaths 8 / members 12.0 = 0.67, dark nights 4, wood at the start 179.0
- heldout_v17_120day seed 72, winter from day 78 (severity 2): world 20/62 = 0.32, world dark nights 3, deadwood per spot at the start 0.00 (24 spots)
  - settlement 1: deaths 4 / members 28.0 = 0.14, dark nights 3, wood at the start 200.0
  - settlement 2 (daughter): deaths 16 / members 34.0 = 0.47, dark nights 3, wood at the start 197.0
- heldout_v17_120day seed 77, winter from day 50 (severity 1): world 7/46 = 0.15, world dark nights 7, deadwood per spot at the start 0.81 (42 spots)
  - settlement 1: deaths 0 / members 37.0 = 0.00, dark nights 2, wood at the start 192.0
  - settlement 2 (daughter): deaths 7 / members 9.0 = 0.78, dark nights 5, wood at the start 25.0
- heldout_v17_120day seed 77, winter from day 106 (severity 1): world 9/48 = 0.19, world dark nights 3, deadwood per spot at the start 0.83 (42 spots)
  - settlement 1: deaths 0 / members 37.0 = 0.00, dark nights 0, wood at the start 194.0
  - settlement 3 (daughter): deaths 9 / members 11.0 = 0.82, dark nights 3, wood at the start 69.0
- heldout_v17_120day seed 78, winter from day 106 (severity 1): world 6/43 = 0.14, world dark nights 2, deadwood per spot at the start 1.00 (32 spots)
  - settlement 1: deaths 0 / members 35.0 = 0.00, dark nights 2, wood at the start 198.0
  - settlement 3 (daughter): deaths 6 / members 8.0 = 0.75, dark nights 2, wood at the start 172.0
- heldout_v17_120day seed 79, winter from day 106 (severity 1): world 30/110 = 0.27, world dark nights 4, deadwood per spot at the start 1.00 (42 spots)
  - settlement 1: deaths 0 / members 36.0 = 0.00, dark nights 4, wood at the start 189.0
  - settlement 2 (daughter): deaths 12 / members 38.0 = 0.32, dark nights 3, wood at the start 197.0
  - settlement 3 (daughter): deaths 18 / members 36.0 = 0.50, dark nights 4, wood at the start 174.0
- heldout_v17_120day seed 80, winter from day 50 (severity 1): world 10/47 = 0.21, world dark nights 6, deadwood per spot at the start 0.59 (27 spots)
  - settlement 1: deaths 0 / members 36.0 = 0.00, dark nights 1, wood at the start 178.0
  - settlement 2 (daughter): deaths 10 / members 11.0 = 0.91, dark nights 6, wood at the start 48.0
- heldout_v17_120day seed 80, winter from day 106 (severity 2): world 9/46 = 0.20, world dark nights 7, deadwood per spot at the start 0.11 (27 spots)
  - settlement 1: deaths 0 / members 36.0 = 0.00, dark nights 3, wood at the start 185.0
  - settlement 3 (daughter): deaths 9 / members 10.0 = 0.90, dark nights 6, wood at the start 5.0
- heldout_v17_120day seed 84, winter from day 106 (severity 2): world 34/102 = 0.33, world dark nights 4, deadwood per spot at the start 0.30 (53 spots)
  - settlement 1: deaths 1 / members 29.0 = 0.03, dark nights 4, wood at the start 193.0
  - settlement 2 (daughter): deaths 12 / members 27.0 = 0.44, dark nights 4, wood at the start 209.0
  - settlement 3 (daughter): deaths 21 / members 46.0 = 0.46, dark nights 4, wood at the start 213.0
- heldout_v17_120day seed 85, winter from day 78 (severity 1): world 13/48 = 0.27, world dark nights 7, deadwood per spot at the start 0.72 (40 spots)
  - settlement 1: deaths 0 / members 38.0 = 0.00, dark nights 1, wood at the start 201.0
  - settlement 2 (daughter): deaths 13 / members 10.0 = 1.30, dark nights 7, wood at the start 16.0
- heldout_v17_120day seed 85, winter from day 106 (severity 2): world 16/61 = 0.26, world dark nights 7, deadwood per spot at the start 0.07 (40 spots)
  - settlement 1: deaths 4 / members 38.0 = 0.11, dark nights 3, wood at the start 189.0
  - settlement 2 (daughter): deaths 4 / members 10.0 = 0.40, dark nights 3, wood at the start 187.0
  - settlement 3 (daughter): deaths 8 / members 13.0 = 0.62, dark nights 4, wood at the start 11.0
- heldout_v17_120day seed 87, winter from day 50 (severity 2): world 13/58 = 0.22, world dark nights 7, deadwood per spot at the start 0.84 (31 spots)
  - settlement 1: deaths 2 / members 47.0 = 0.04, dark nights 3, wood at the start 210.0
  - settlement 2 (daughter): deaths 11 / members 11.0 = 1.00, dark nights 5, wood at the start 4.0
- heldout_v17_120day seed 87, winter from day 106 (severity 0): world 7/56 = 0.12, world dark nights 5, deadwood per spot at the start 0.77 (31 spots)
  - settlement 1: deaths 0 / members 47.0 = 0.00, dark nights 0, wood at the start 184.0
  - settlement 3 (daughter): deaths 7 / members 9.0 = 0.78, dark nights 5, wood at the start 3.0
- heldout_v17_120day seed 93, winter from day 50 (severity 2): world 5/46 = 0.11, world dark nights 3, deadwood per spot at the start 0.50 (46 spots)
  - settlement 1: deaths 0 / members 33.0 = 0.00, dark nights 2, wood at the start 207.0
  - settlement 2 (daughter): deaths 5 / members 13.0 = 0.38, dark nights 3, wood at the start 187.0

## Daughters

| Set | Seed | Sid | Founded day (season) | At leader | Region | Bushes | Deadwood | Water | Founders | Alive 28 d |
|---|---:|---:|---|---|---:|---|---|---:|---:|---|
| v17_120day | 0 | 2 | 30 (spring) | False | 0 | 6/11 | 13/13 | 10 | 14 | True |
| v17_120day | 1 | 2 | 29 (spring) | False | 0 | 10/10 | 8/8 | 23 | 11 | True |
| v17_120day | 2 | 2 | 29 (spring) | False | 1 | 2/7 | 8/8 | 9 | 14 | True |
| v17_120day | 2 | 3 | 86 (spring) | False | 0 | 7/7 | 8/8 | 1 | 14 | True |
| v17_120day | 4 | 2 | 29 (spring) | False | 1 | 9/9 | 13/13 | 17 | 14 | True |
| v17_120day | 4 | 3 | 85 (spring) | False | 0 | 9/9 | 13/13 | 18 | 14 | True |
| v17_120day | 6 | 2 | 30 (spring) | False | 0 | 11/11 | 13/13 | 18 | 13 | True |
| v17_120day | 6 | 3 | 85 (spring) | False | 2 | 6/11 | 13/13 | 1 | 11 | True |
| v17_120day | 7 | 2 | 29 (spring) | False | 0 | 2/9 | 13/13 | 2 | 14 | True |
| v17_120day | 8 | 2 | 29 (spring) | False | 0 | 6/9 | 11/11 | 18 | 14 | True |
| v17_120day | 8 | 3 | 85 (spring) | False | 1 | 9/9 | 11/11 | 9 | 13 | True |
| v17_120day | 9 | 2 | 30 (spring) | False | 0 | 0/8 | 11/11 | 15 | 14 | False |
| v17_120day | 9 | 3 | 87 (spring) | False | 0 | 0/8 | 11/11 | 15 | 14 | False |
| v17_120day | 10 | 2 | 29 (spring) | False | 0 | 7/11 | 10/10 | 5 | 14 | True |
| v17_120day | 11 | 2 | 29 (spring) | False | 0 | 0/7 | 9/10 | 11 | 14 | False |
| v17_120day | 11 | 3 | 85 (spring) | False | 0 | 0/7 | 9/10 | 11 | 14 | False |
| v17_120day | 12 | 2 | 30 (spring) | False | 0 | 11/11 | 14/14 | 6 | 14 | True |
| v17_120day | 13 | 2 | 29 (spring) | False | 1 | 5/8 | 8/8 | 12 | 13 | True |
| v17_120day | 13 | 3 | 85 (spring) | False | 0 | 5/8 | 8/8 | 27 | 13 | True |
| v17_120day | 14 | 2 | 29 (spring) | False | 1 | 4/7 | 14/14 | 23 | 12 | True |
| v17_120day | 15 | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 15 | 13 | True |
| v17_120day | 15 | 3 | 98 (summer) | False | 1 | 2/7 | 11/11 | 28 | 13 | None |
| v17_120day | 16 | 2 | 29 (spring) | False | 1 | 0/8 | 14/14 | 21 | 14 | True |
| v17_120day | 16 | 3 | 87 (spring) | False | 2 | 0/8 | 12/14 | 17 | 13 | True |
| v17_120day | 17 | 2 | 29 (spring) | False | 1 | 9/9 | 14/14 | 16 | 14 | True |
| v17_120day | 17 | 3 | 85 (spring) | False | 0 | 1/9 | 14/14 | 3 | 14 | True |
| v17_120day | 18 | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 22 | 13 | True |
| v17_120day | 18 | 3 | 94 (summer) | False | 1 | 0/11 | 13/13 | 22 | 13 | None |
| v17_120day | 19 | 2 | 29 (spring) | False | 0 | 0/9 | 12/12 | 16 | 12 | False |
| v17_120day | 19 | 3 | 85 (spring) | False | 0 | 0/9 | 12/12 | 16 | 12 | False |
| v17_120day | 21 | 2 | 29 (spring) | False | 0 | 7/11 | 13/14 | 14 | 14 | True |
| v17_120day | 23 | 2 | 35 (spring) | False | 1 | 6/7 | 11/11 | 6 | 14 | True |
| v17_120day | 23 | 3 | 85 (spring) | False | 0 | 0/7 | 11/11 | 28 | 13 | False |
| v17_120day | 24 | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 19 | 12 | True |
| v17_120day | 25 | 2 | 29 (spring) | False | 1 | 7/7 | 14/14 | 9 | 14 | True |
| v17_120day | 25 | 3 | 85 (spring) | False | 2 | 0/7 | 2/14 | 18 | 14 | False |
| v17_120day | 26 | 2 | 29 (spring) | False | 1 | 0/11 | 13/13 | 4 | 14 | True |
| v17_120day | 26 | 3 | 85 (spring) | False | 0 | 11/11 | 13/13 | 16 | 14 | True |
| v17_120day | 27 | 2 | 29 (spring) | False | 1 | 9/9 | 10/10 | 20 | 14 | True |
| v17_120day | 27 | 3 | 87 (spring) | False | 2 | 9/9 | 10/10 | 18 | 13 | True |
| v17_120day | 28 | 2 | 29 (spring) | False | 0 | 11/11 | 13/13 | 16 | 14 | True |
| v17_120day | 28 | 3 | 85 (spring) | False | 1 | 0/11 | 13/13 | 14 | 12 | False |
| v17_120day | 31 | 2 | 29 (spring) | False | 1 | 0/9 | 9/9 | 16 | 14 | True |
| v17_120day | 31 | 3 | 85 (spring) | False | 2 | 9/9 | 9/9 | 23 | 11 | True |
| heldout_v17_120day | 37 | 2 | 30 (spring) | False | 0 | 10/10 | 10/10 | 12 | 13 | True |
| heldout_v17_120day | 37 | 3 | 85 (spring) | False | 1 | 0/10 | 10/10 | 7 | 13 | True |
| heldout_v17_120day | 38 | 2 | 30 (spring) | False | 0 | 7/7 | 8/8 | 26 | 14 | True |
| heldout_v17_120day | 39 | 2 | 29 (spring) | False | 0 | 8/8 | 8/8 | 7 | 14 | True |
| heldout_v17_120day | 39 | 3 | 85 (spring) | False | 1 | 8/8 | 8/8 | 26 | 12 | True |
| heldout_v17_120day | 41 | 2 | 29 (spring) | False | 0 | 0/8 | 11/11 | 7 | 14 | True |
| heldout_v17_120day | 41 | 3 | 85 (spring) | False | 0 | 0/8 | 11/11 | 7 | 14 | True |
| heldout_v17_120day | 43 | 2 | 29 (spring) | False | 2 | 11/11 | 10/10 | 5 | 14 | True |
| heldout_v17_120day | 43 | 3 | 85 (spring) | False | 0 | 11/11 | 10/10 | 26 | 14 | True |
| heldout_v17_120day | 44 | 2 | 29 (spring) | False | 0 | 3/7 | 10/10 | 18 | 14 | True |
| heldout_v17_120day | 45 | 2 | 29 (spring) | False | 0 | 0/11 | 8/8 | 17 | 14 | True |
| heldout_v17_120day | 45 | 3 | 88 (spring) | False | 0 | 0/11 | 8/8 | 17 | 14 | True |
| heldout_v17_120day | 46 | 2 | 29 (spring) | False | 0 | 9/9 | 10/10 | 13 | 12 | True |
| heldout_v17_120day | 47 | 2 | 29 (spring) | False | 0 | 10/10 | 10/10 | 26 | 12 | True |
| heldout_v17_120day | 48 | 2 | 29 (spring) | False | 0 | 1/7 | 12/12 | 25 | 13 | False |
| heldout_v17_120day | 48 | 3 | 86 (spring) | False | 0 | 1/7 | 12/12 | 25 | 13 | False |
| heldout_v17_120day | 49 | 2 | 29 (spring) | False | 0 | 1/7 | 12/12 | 3 | 14 | True |
| heldout_v17_120day | 51 | 2 | 33 (spring) | False | 0 | 9/9 | 9/9 | 10 | 14 | True |
| heldout_v17_120day | 51 | 3 | 85 (spring) | False | 1 | 6/9 | 9/9 | 25 | 14 | True |
| heldout_v17_120day | 53 | 2 | 31 (spring) | False | 0 | 10/10 | 13/13 | 17 | 12 | True |
| heldout_v17_120day | 53 | 3 | 88 (spring) | False | 1 | 5/10 | 13/13 | 15 | 13 | True |
| heldout_v17_120day | 54 | 2 | 29 (spring) | False | 0 | 10/10 | 12/12 | 8 | 13 | True |
| heldout_v17_120day | 56 | 2 | 30 (spring) | False | 1 | 10/10 | 10/10 | 27 | 12 | True |
| heldout_v17_120day | 56 | 3 | 85 (spring) | False | 0 | 10/10 | 10/10 | 16 | 12 | True |
| heldout_v17_120day | 58 | 2 | 29 (spring) | False | 0 | 7/7 | 13/13 | 18 | 13 | True |
| heldout_v17_120day | 59 | 2 | 29 (spring) | False | 2 | 11/11 | 8/8 | 14 | 14 | True |
| heldout_v17_120day | 59 | 3 | 85 (spring) | False | 1 | 0/11 | 8/8 | 25 | 14 | False |
| heldout_v17_120day | 60 | 2 | 30 (spring) | False | 1 | 7/7 | 10/10 | 18 | 14 | True |
| heldout_v17_120day | 60 | 3 | 85 (spring) | False | 0 | 7/7 | 10/10 | 6 | 13 | True |
| heldout_v17_120day | 61 | 2 | 30 (spring) | False | 0 | 7/7 | 11/11 | 9 | 11 | True |
| heldout_v17_120day | 62 | 2 | 29 (spring) | False | 1 | 9/9 | 8/8 | 6 | 14 | True |
| heldout_v17_120day | 62 | 3 | 85 (spring) | False | 2 | 9/9 | 8/8 | 14 | 13 | True |
| heldout_v17_120day | 65 | 2 | 29 (spring) | False | 0 | 7/7 | 11/11 | 5 | 13 | True |
| heldout_v17_120day | 65 | 3 | 88 (spring) | False | 1 | 7/7 | 11/11 | 24 | 13 | True |
| heldout_v17_120day | 66 | 2 | 29 (spring) | False | 1 | 10/10 | 11/11 | 7 | 13 | True |
| heldout_v17_120day | 66 | 3 | 85 (spring) | False | 0 | 9/10 | 11/11 | 17 | 13 | True |
| heldout_v17_120day | 67 | 2 | 29 (spring) | False | 0 | 4/11 | 11/11 | 24 | 12 | True |
| heldout_v17_120day | 68 | 2 | 29 (spring) | False | 0 | 6/11 | 12/12 | 15 | 14 | True |
| heldout_v17_120day | 68 | 3 | 85 (spring) | False | 1 | 6/11 | 12/12 | 27 | 14 | True |
| heldout_v17_120day | 69 | 2 | 29 (spring) | False | 0 | 10/10 | 13/13 | 25 | 13 | True |
| heldout_v17_120day | 69 | 3 | 85 (spring) | False | 1 | 10/10 | 13/13 | 13 | 11 | True |
| heldout_v17_120day | 70 | 2 | 29 (spring) | False | 1 | 4/9 | 8/8 | 14 | 14 | True |
| heldout_v17_120day | 70 | 3 | 88 (spring) | False | 0 | 9/9 | 8/8 | 30 | 12 | True |
| heldout_v17_120day | 71 | 2 | 31 (spring) | False | 2 | 0/10 | 8/8 | 22 | 14 | True |
| heldout_v17_120day | 71 | 3 | 86 (spring) | False | 2 | 0/10 | 8/8 | 22 | 14 | True |
| heldout_v17_120day | 72 | 2 | 30 (spring) | False | 0 | 6/8 | 8/8 | 26 | 14 | True |
| heldout_v17_120day | 73 | 2 | 31 (spring) | False | 0 | 8/11 | 12/12 | 2 | 14 | True |
| heldout_v17_120day | 74 | 2 | 29 (spring) | False | 0 | 10/10 | 12/12 | 13 | 12 | True |
| heldout_v17_120day | 75 | 2 | 29 (spring) | False | 0 | 0/8 | 11/11 | 23 | 12 | True |
| heldout_v17_120day | 76 | 2 | 29 (spring) | False | 0 | 7/7 | 11/14 | 15 | 14 | True |
| heldout_v17_120day | 77 | 2 | 30 (spring) | False | 0 | 0/11 | 14/14 | 25 | 13 | False |
| heldout_v17_120day | 77 | 3 | 87 (spring) | False | 0 | 0/11 | 14/14 | 25 | 13 | False |
| heldout_v17_120day | 78 | 2 | 29 (spring) | False | 0 | 0/9 | 14/14 | 6 | 13 | False |
| heldout_v17_120day | 78 | 3 | 85 (spring) | False | 0 | 0/9 | 14/14 | 6 | 13 | True |
| heldout_v17_120day | 79 | 2 | 30 (spring) | False | 0 | 10/10 | 14/14 | 21 | 13 | True |
| heldout_v17_120day | 79 | 3 | 85 (spring) | False | 1 | 9/10 | 14/14 | 17 | 13 | True |
| heldout_v17_120day | 80 | 2 | 32 (spring) | False | 0 | 0/7 | 9/9 | 23 | 14 | False |
| heldout_v17_120day | 80 | 3 | 85 (spring) | False | 0 | 0/7 | 9/9 | 23 | 14 | False |
| heldout_v17_120day | 83 | 2 | 29 (spring) | False | 0 | 2/11 | 8/8 | 11 | 14 | True |
| heldout_v17_120day | 84 | 2 | 29 (spring) | False | 0 | 7/11 | 13/13 | 14 | 12 | True |
| heldout_v17_120day | 84 | 3 | 85 (spring) | False | 1 | 11/11 | 13/13 | 18 | 12 | True |
| heldout_v17_120day | 85 | 2 | 29 (spring) | False | 1 | 1/11 | 11/11 | 19 | 14 | True |
| heldout_v17_120day | 85 | 3 | 85 (spring) | False | 0 | 0/11 | 11/11 | 24 | 13 | False |
| heldout_v17_120day | 87 | 2 | 29 (spring) | False | 0 | 1/8 | 8/8 | 26 | 14 | False |
| heldout_v17_120day | 87 | 3 | 85 (spring) | False | 0 | 1/8 | 8/8 | 26 | 14 | False |
| heldout_v17_120day | 89 | 2 | 29 (spring) | False | 1 | 8/8 | 13/13 | 8 | 13 | True |
| heldout_v17_120day | 89 | 3 | 85 (spring) | False | 0 | 7/8 | 13/13 | 8 | 12 | True |
| heldout_v17_120day | 90 | 2 | 29 (spring) | False | 0 | 9/9 | 14/14 | 19 | 14 | True |
| heldout_v17_120day | 91 | 2 | 29 (spring) | False | 0 | 10/10 | 13/13 | 18 | 14 | True |
| heldout_v17_120day | 91 | 3 | 85 (spring) | False | 1 | 6/10 | 13/13 | 17 | 12 | True |
| heldout_v17_120day | 93 | 2 | 29 (spring) | False | 0 | 2/7 | 6/11 | 12 | 14 | True |
| heldout_v17_120day | 93 | 3 | 85 (spring) | False | 1 | 7/7 | 11/11 | 17 | 12 | True |
| heldout_v17_120day | 94 | 2 | 29 (spring) | False | 0 | 5/7 | 10/10 | 14 | 13 | True |
| heldout_v17_120day | 94 | 3 | 86 (spring) | False | 2 | 5/7 | 10/10 | 9 | 11 | True |
| heldout_v17_120day | 95 | 2 | 30 (spring) | False | 1 | 7/10 | 9/9 | 1 | 14 | True |
