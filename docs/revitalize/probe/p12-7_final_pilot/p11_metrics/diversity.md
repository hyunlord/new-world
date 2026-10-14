# Pass 11 diversity pilot

Train 24000..24096, test 24096..24112 (fresh seeds; `diversity_p11.py`). eta2 / dyn-eta2 / R2_abs on the test split; the reference rows re-score Pass 10's pilot recordings read-only.

| Variable | game_v16 | game_v17 | game_v18_nofission | game_v18 |
|---|---|---|---|---|
| food_stock | 0.332 / 0.195 / 0.651 | 0.472 / 0.502 / 0.475 | 0.311 / 0.185 / 0.672 | 0.532 / 0.534 / 0.457 |
| wood_stock | 0.069 / 0.069 / 0.929 | 0.190 / 0.198 / 0.786 | 0.068 / 0.069 / 0.930 | 0.236 / 0.240 / 0.749 |
| fire_lit | 0.046 / 0.049 / 0.952 | 0.045 / 0.049 / 0.953 | 0.045 / 0.048 / 0.953 | 0.043 / 0.046 / 0.955 |
| cold_members | 0.593 / 0.593 / 0.376 | 0.536 / 0.536 / 0.428 | 0.566 / 0.567 / 0.395 | 0.579 / 0.580 / 0.391 |
| alive | 0.900 / 0.998 / -0.102 | 0.735 / 0.785 / 0.184 | 0.899 / 0.998 / -0.106 | 0.741 / 0.756 / 0.230 |
| deadwood_total | 0.207 / 0.107 / 0.781 | 0.330 / 0.147 / 0.669 | 0.358 / 0.132 / 0.638 | 0.351 / 0.150 / 0.644 |
| food_on_bushes_total | 0.342 / 0.409 / 0.631 | 0.640 / 0.424 / 0.337 | 0.835 / 0.410 / 0.155 | 0.751 / 0.508 / 0.232 |
| adults_working | 0.474 / 0.474 / 0.491 | 0.438 / 0.439 / 0.528 | 0.473 / 0.476 / 0.492 | 0.480 / 0.479 / 0.488 |

## T2 nMAE at 24 h -> 168 h (test split std, every-hour starts)

- game_v16 food_stock: persistence 0.157 -> 0.954, apfit_abs 0.078 -> 0.281, cal_abs 0.458, an5 0.285
- game_v16 wood_stock: persistence 0.287 -> 1.186, apfit_abs 0.204 -> 0.186, cal_abs 0.187, an5 0.188
- game_v16 alive: persistence 0.043 -> 0.154, apfit_abs 0.038 -> 0.118, cal_abs 0.829, an5 0.233
- game_v17 food_stock: persistence 0.134 -> 0.779, apfit_abs 0.076 -> 0.298, cal_abs 0.592, an5 0.258
- game_v17 wood_stock: persistence 0.219 -> 0.949, apfit_abs 0.155 -> 0.258, cal_abs 0.311, an5 0.193
- game_v17 alive: persistence 0.032 -> 0.149, apfit_abs 0.031 -> 0.137, cal_abs 0.758, an5 0.246
- game_v18_nofission food_stock: persistence 0.162 -> 0.968, apfit_abs 0.081 -> 0.283, cal_abs 0.453, an5 0.300
- game_v18_nofission wood_stock: persistence 0.285 -> 1.193, apfit_abs 0.199 -> 0.183, cal_abs 0.184, an5 0.191
- game_v18_nofission alive: persistence 0.041 -> 0.148, apfit_abs 0.035 -> 0.111, cal_abs 0.831, an5 0.306
- game_v18 food_stock: persistence 0.129 -> 0.733, apfit_abs 0.072 -> 0.307, cal_abs 0.563, an5 0.251
- game_v18 wood_stock: persistence 0.225 -> 0.977, apfit_abs 0.169 -> 0.293, cal_abs 0.366, an5 0.235
- game_v18 alive: persistence 0.026 -> 0.139, apfit_abs 0.029 -> 0.160, cal_abs 0.709, an5 0.257

## Settlements alive at days 30 / 60 / 90 / 118 (test seeds)

| Scene | Mean | Variance | Distribution (count: seeds) |
|---|---|---|---|
| game_v16 | [1.0, 1.0, 1.0, 1.0] | [0.0, 0.0, 0.0, 0.0] | [{'1.0': 16}, {'1.0': 16}, {'1.0': 16}, {'1.0': 16}] |
| game_v17 | [1.5, 1.75, 2.25, 2.25] | [0.25, 0.188, 0.688, 0.688] | [{'1.0': 8, '2.0': 8}, {'1.0': 4, '2.0': 12}, {'1.0': 4, '2.0': 4, '3.0': 8}, {'1.0': 4, '2.0': 4, '3.0': 8}] |
| game_v18_nofission | [1.0, 1.0, 1.0, 1.0] | [0.0, 0.0, 0.0, 0.0] | [{'1.0': 16}, {'1.0': 16}, {'1.0': 16}, {'1.0': 16}] |
| game_v18 | [1.062, 1.25, 1.562, 2.062] | [0.059, 0.188, 0.371, 0.559] | [{'1.0': 15, '2.0': 1}, {'1.0': 12, '2.0': 4}, {'1.0': 8, '2.0': 7, '3.0': 1}, {'1.0': 3, '2.0': 10, '3.0': 2, '4.0': 1}] |

## Attribution (delta eta2 / delta dyn-eta2)


## Deaths per episode by day 120 (mean)

- game_v16: deaths_starvation 0.00, deaths_dehydration 0.00, deaths_combat 0.00, deaths_old_age 66.92, deaths_exposure 2.53
- game_v17: deaths_starvation 0.19, deaths_dehydration 0.19, deaths_combat 0.00, deaths_old_age 77.24, deaths_exposure 8.34
- game_v18_nofission: deaths_starvation 0.00, deaths_dehydration 0.00, deaths_combat 0.00, deaths_old_age 66.85, deaths_exposure 1.25
- game_v18: deaths_starvation 0.00, deaths_dehydration 0.00, deaths_combat 0.00, deaths_old_age 79.32, deaths_exposure 2.41

**J7 (game_v17): FAIL** — food_stock: PASS, wood_stock: FAIL, alive: PASS, dyn_alive: PASS, dyn_food_stock: PASS
