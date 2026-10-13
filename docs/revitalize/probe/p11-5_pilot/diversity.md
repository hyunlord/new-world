# Pass 11 diversity pilot

Train 22000..22096, test 22096..22112 (fresh seeds; `diversity_p11.py`). eta2 / dyn-eta2 / R2_abs on the test split; the reference rows re-score Pass 10's pilot recordings read-only.

| Variable | game_v15 (Pass 10 pilot) | game_v16 (Pass 10 pilot) | game_v16 | game_v17_nofission | game_v17 |
|---|---|---|---|---|---|
| food_stock | 0.153 / 0.130 / 0.845 | 0.500 / 0.320 / 0.474 | 0.411 / 0.298 / 0.573 | 0.437 / 0.434 / 0.548 | 0.672 / 0.694 / 0.309 |
| wood_stock | 0.055 / 0.057 / 0.942 | 0.063 / 0.065 / 0.934 | 0.068 / 0.069 / 0.929 | 0.068 / 0.069 / 0.929 | 0.269 / 0.276 / 0.709 |
| fire_lit | 0.041 / 0.044 / 0.956 | 0.040 / 0.043 / 0.957 | 0.044 / 0.048 / 0.952 | 0.045 / 0.048 / 0.951 | 0.045 / 0.048 / 0.951 |
| cold_members | 0.586 / 0.586 / 0.359 | 0.582 / 0.582 / 0.354 | 0.598 / 0.599 / 0.358 | 0.598 / 0.598 / 0.361 | 0.615 / 0.616 / 0.338 |
| alive | 0.021 / 0.790 / 0.978 | 0.978 / 0.999 / -0.043 | 0.953 / 0.998 / 0.004 | 0.953 / 0.998 / 0.003 | 0.908 / 0.933 / 0.079 |
| deadwood_total | 0.030 / 0.030 / 0.968 | 0.168 / 0.087 / 0.827 | 0.187 / 0.097 / 0.809 | 0.318 / 0.116 / 0.681 | 0.340 / 0.124 / 0.658 |
| food_on_bushes_total | 0.124 / 0.133 / 0.872 | 0.503 / 0.492 / 0.468 | 0.375 / 0.442 / 0.606 | 0.826 / 0.467 / 0.164 | 0.802 / 0.576 / 0.169 |
| adults_working | 0.304 / 0.299 / 0.676 | 0.484 / 0.490 / 0.474 | 0.474 / 0.475 / 0.491 | 0.490 / 0.493 / 0.472 | 0.492 / 0.491 / 0.478 |

## T2 nMAE at 24 h -> 168 h (test split std, every-hour starts)

- game_v15 (Pass 10 pilot) food_stock: persistence 0.169 -> 1.042, apfit_abs 0.060 -> 0.150, cal_abs 0.270, an5 0.135
- game_v15 (Pass 10 pilot) wood_stock: persistence 0.258 -> 1.223, apfit_abs 0.174 -> 0.160, cal_abs 0.160, an5 0.168
- game_v15 (Pass 10 pilot) alive: persistence 0.115 -> 0.453, apfit_abs 0.060 -> 0.068, cal_abs 0.068, an5 0.069
- game_v16 (Pass 10 pilot) food_stock: persistence 0.142 -> 0.823, apfit_abs 0.090 -> 0.327, cal_abs 0.584, an5 0.216
- game_v16 (Pass 10 pilot) wood_stock: persistence 0.276 -> 1.174, apfit_abs 0.175 -> 0.200, cal_abs 0.202, an5 0.178
- game_v16 (Pass 10 pilot) alive: persistence 0.024 -> 0.080, apfit_abs 0.026 -> 0.089, cal_abs 0.861, an5 0.146
- game_v16 food_stock: persistence 0.149 -> 0.904, apfit_abs 0.082 -> 0.307, cal_abs 0.523, an5 0.225
- game_v16 wood_stock: persistence 0.276 -> 1.196, apfit_abs 0.194 -> 0.195, cal_abs 0.195, an5 0.187
- game_v16 alive: persistence 0.027 -> 0.103, apfit_abs 0.026 -> 0.094, cal_abs 0.932, an5 0.179
- game_v17_nofission food_stock: persistence 0.154 -> 0.880, apfit_abs 0.093 -> 0.336, cal_abs 0.542, an5 0.247
- game_v17_nofission wood_stock: persistence 0.279 -> 1.194, apfit_abs 0.203 -> 0.184, cal_abs 0.186, an5 0.198
- game_v17_nofission alive: persistence 0.027 -> 0.104, apfit_abs 0.027 -> 0.096, cal_abs 0.933, an5 0.202
- game_v17 food_stock: persistence 0.114 -> 0.651, apfit_abs 0.077 -> 0.345, cal_abs 0.630, an5 0.228
- game_v17 wood_stock: persistence 0.221 -> 0.974, apfit_abs 0.179 -> 0.295, cal_abs 0.375, an5 0.214
- game_v17 alive: persistence 0.024 -> 0.117, apfit_abs 0.026 -> 0.133, cal_abs 0.767, an5 0.199

## Settlements alive at days 30 / 60 / 90 / 118 (test seeds)

| Scene | Mean | Variance | Distribution (count: seeds) |
|---|---|---|---|
| game_v15 (Pass 10 pilot) | [1.0, 1.0, 1.0, 1.0] | [0.0, 0.0, 0.0, 0.0] | [{'1.0': 16}, {'1.0': 16}, {'1.0': 16}, {'1.0': 16}] |
| game_v16 (Pass 10 pilot) | [1.0, 1.0, 1.0, 1.0] | [0.0, 0.0, 0.0, 0.0] | [{'1.0': 16}, {'1.0': 16}, {'1.0': 16}, {'1.0': 16}] |
| game_v16 | [1.0, 1.0, 1.0, 1.0] | [0.0, 0.0, 0.0, 0.0] | [{'1.0': 16}, {'1.0': 16}, {'1.0': 16}, {'1.0': 16}] |
| game_v17_nofission | [1.0, 1.0, 1.0, 1.0] | [0.0, 0.0, 0.0, 0.0] | [{'1.0': 16}, {'1.0': 16}, {'1.0': 16}, {'1.0': 16}] |
| game_v17 | [1.188, 1.5, 1.688, 1.688] | [0.152, 0.25, 0.715, 0.715] | [{'1.0': 13, '2.0': 3}, {'1.0': 8, '2.0': 8}, {'1.0': 9, '2.0': 3, '3.0': 4}, {'1.0': 9, '2.0': 3, '3.0': 4}] |

## Attribution (delta eta2 / delta dyn-eta2)

- exposure v2 + rain + regions: food_stock 0.026 / 0.136, wood_stock 0.000 / -0.000, fire_lit 0.001 / 0.001, cold_members -0.001 / -0.001, alive 0.000 / 0.000, deadwood_total 0.131 / 0.019, food_on_bushes_total 0.451 / 0.025, adults_working 0.016 / 0.018
- fission: food_stock 0.235 / 0.260, wood_stock 0.201 / 0.207, fire_lit -0.000 / -0.000, cold_members 0.017 / 0.017, alive -0.045 / -0.065, deadwood_total 0.022 / 0.008, food_on_bushes_total -0.024 / 0.109, adults_working 0.002 / -0.002

## Deaths per episode by day 120 (mean)

- game_v15 (Pass 10 pilot): deaths_starvation 0.00, deaths_dehydration 0.00, deaths_combat 0.00, deaths_old_age 72.87, deaths_exposure 0.00
- game_v16 (Pass 10 pilot): deaths_starvation 0.00, deaths_dehydration 0.00, deaths_combat 0.00, deaths_old_age 64.67, deaths_exposure 2.32
- game_v16: deaths_starvation 0.00, deaths_dehydration 0.00, deaths_combat 0.00, deaths_old_age 66.07, deaths_exposure 2.46
- game_v17_nofission: deaths_starvation 0.00, deaths_dehydration 0.00, deaths_combat 0.00, deaths_old_age 65.43, deaths_exposure 2.89
- game_v17: deaths_starvation 0.18, deaths_dehydration 0.23, deaths_combat 0.00, deaths_old_age 73.59, deaths_exposure 8.19

**J7 (game_v17): FAIL** — food_stock: PASS, wood_stock: FAIL, alive: PASS, dyn_alive: PASS, dyn_food_stock: PASS
