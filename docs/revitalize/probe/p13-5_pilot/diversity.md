# Pass 11 diversity pilot

Train 26000..26048, test 26048..26064 (fresh seeds; `diversity_p11.py`). eta2 / dyn-eta2 / R2_abs on the test split; the reference rows re-score Pass 10's pilot recordings read-only.

| Variable | game_v16 | game_v17 | game_v18 | game_v19 |
|---|---|---|---|---|
| food_stock | 0.395 / 0.319 / 0.597 | 0.511 / 0.580 / 0.467 | 0.632 / 0.656 / 0.358 | 0.624 / 0.658 / 0.373 |
| wood_stock | 0.076 / 0.077 / 0.921 | 0.236 / 0.248 / 0.748 | 0.293 / 0.301 / 0.694 | 0.317 / 0.328 / 0.681 |
| fire_lit | 0.046 / 0.050 / 0.952 | 0.044 / 0.048 / 0.953 | 0.043 / 0.046 / 0.955 | 0.043 / 0.046 / 0.955 |
| cold_members | 0.590 / 0.591 / 0.383 | 0.560 / 0.560 / 0.410 | 0.627 / 0.627 / 0.343 | 0.628 / 0.629 / 0.351 |
| alive | 0.962 / 0.999 / 0.036 | 0.826 / 0.833 / 0.153 | 0.817 / 0.820 / 0.173 | 0.821 / 0.826 / 0.175 |
| deadwood_total | 0.162 / 0.090 / 0.834 | 0.245 / 0.108 / 0.753 | 0.286 / 0.124 / 0.712 | 0.282 / 0.128 / 0.717 |
| food_on_bushes_total | 0.386 / 0.392 / 0.590 | 0.710 / 0.486 / 0.263 | 0.816 / 0.577 / 0.169 | 0.804 / 0.595 / 0.186 |
| adults_working | 0.454 / 0.452 / 0.511 | 0.440 / 0.433 / 0.532 | 0.487 / 0.489 / 0.477 | 0.492 / 0.493 / 0.477 |

## T2 nMAE at 24 h -> 168 h (test split std, every-hour starts)

- game_v16 food_stock: persistence 0.157 -> 0.925, apfit_abs 0.088 -> 0.316, cal_abs 0.500, an5 0.272
- game_v16 wood_stock: persistence 0.284 -> 1.193, apfit_abs 0.189 -> 0.204, cal_abs 0.205, an5 0.189
- game_v16 alive: persistence 0.031 -> 0.101, apfit_abs 0.031 -> 0.089, cal_abs 0.783, an5 0.251
- game_v17 food_stock: persistence 0.134 -> 0.764, apfit_abs 0.079 -> 0.306, cal_abs 0.545, an5 0.253
- game_v17 wood_stock: persistence 0.217 -> 0.958, apfit_abs 0.149 -> 0.303, cal_abs 0.365, an5 0.232
- game_v17 alive: persistence 0.025 -> 0.128, apfit_abs 0.026 -> 0.126, cal_abs 0.742, an5 0.267
- game_v18 food_stock: persistence 0.105 -> 0.600, apfit_abs 0.068 -> 0.298, cal_abs 0.581, an5 0.258
- game_v18 wood_stock: persistence 0.204 -> 0.969, apfit_abs 0.148 -> 0.311, cal_abs 0.397, an5 0.211
- game_v18 alive: persistence 0.021 -> 0.102, apfit_abs 0.025 -> 0.133, cal_abs 0.720, an5 0.266
- game_v19 food_stock: persistence 0.106 -> 0.592, apfit_abs 0.071 -> 0.286, cal_abs 0.585, an5 0.233
- game_v19 wood_stock: persistence 0.201 -> 0.954, apfit_abs 0.137 -> 0.317, cal_abs 0.405, an5 0.202
- game_v19 alive: persistence 0.018 -> 0.098, apfit_abs 0.022 -> 0.125, cal_abs 0.713, an5 0.236

## Settlements alive at days 30 / 60 / 90 / 118 (test seeds)

| Scene | Mean | Variance | Distribution (count: seeds) |
|---|---|---|---|
| game_v16 | [1.0, 1.0, 1.0, 1.0] | [0.0, 0.0, 0.0, 0.0] | [{'1.0': 16}, {'1.0': 16}, {'1.0': 16}, {'1.0': 16}] |
| game_v17 | [1.375, 1.688, 1.938, 2.0] | [0.234, 0.215, 0.684, 0.625] | [{'1.0': 10, '2.0': 6}, {'1.0': 5, '2.0': 11}, {'1.0': 6, '2.0': 5, '3.0': 5}, {'1.0': 5, '2.0': 6, '3.0': 5}] |
| game_v18 | [1.125, 1.562, 1.75, 1.875] | [0.109, 0.371, 0.562, 0.484] | [{'1.0': 14, '2.0': 2}, {'1.0': 8, '2.0': 7, '3.0': 1}, {'1.0': 7, '2.0': 6, '3.0': 3}, {'1.0': 5, '2.0': 8, '3.0': 3}] |
| game_v19 | [1.0, 1.375, 1.812, 1.938] | [0.0, 0.234, 0.652, 1.059] | [{'1.0': 16}, {'1.0': 10, '2.0': 6}, {'1.0': 7, '2.0': 5, '3.0': 4}, {'1.0': 7, '2.0': 5, '3.0': 2, '4.0': 2}] |

## Attribution (delta eta2 / delta dyn-eta2)


## Deaths per episode by day 120 (mean)

- game_v16: deaths_starvation 0.00, deaths_dehydration 0.00, deaths_combat 0.00, deaths_old_age 65.09, deaths_exposure 2.59
- game_v17: deaths_starvation 0.06, deaths_dehydration 0.30, deaths_combat 0.00, deaths_old_age 72.44, deaths_exposure 8.64
- game_v18: deaths_starvation 0.00, deaths_dehydration 0.00, deaths_combat 0.00, deaths_old_age 77.19, deaths_exposure 2.11
- game_v19: deaths_starvation 0.00, deaths_dehydration 0.00, deaths_combat 0.00, deaths_old_age 77.86, deaths_exposure 1.19

**J7 (game_v17): FAIL** — food_stock: PASS, wood_stock: FAIL, alive: PASS, dyn_alive: PASS, dyn_food_stock: PASS
