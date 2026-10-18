# WM-Bench v0

First benchmark for WorldSim world models, on dataset v0 (`DATASET_v0.md`). Settlement level; numbers are nMAE = MAE / standard deviation of the variable over the test split's scored rows (the T1 targets, rows 24 … T−1 of every episode; T2 uses the same denominator; lower is better) unless stated. The action-aware ridge encodes each intervention by its requested amount. Produced by `tools/worldmodel/bench_v0.py` (numpy only).

Episodes: {"A_obs": 128, "B_act": 64, "C_cf": 160, "OOD_v12": 32, "OOD_long": 16, "A_train": 96, "B_train": 48}. Ridge λ chosen on validation: 100.0 (action-aware 100.0).

Variables V: `food_stock`, `wood_stock`, `fire_lit`, `cold_members`, `alive`, `deadwood_total`, `food_on_bushes_total`, `adults_working`.

## T1 — next hour

Predict V at row t+1 (one hour later) from rows up to t. Train: A train (action-aware: A + B train).

### A_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.046 | 0.027 | 0.165 | 0.030 | 0.011 | 0.043 | 0.287 | 0.655 | 0.158 |
| daily_naive | 0.192 | 0.319 | 0.030 | 0.106 | 0.114 | 0.364 | 0.318 | 0.451 | 0.237 |
| ridge | 0.022 | 0.015 | 0.018 | 0.089 | 0.021 | 0.061 | 0.177 | 0.396 | 0.100 |
| ridge_action | 0.022 | 0.015 | 0.019 | 0.095 | 0.021 | 0.060 | 0.176 | 0.396 | 0.101 |

### B_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.048 | 0.027 | 0.166 | 0.030 | 0.011 | 0.042 | 0.292 | 0.674 | 0.161 |
| daily_naive | 0.198 | 0.327 | 0.041 | 0.131 | 0.122 | 0.365 | 0.309 | 0.451 | 0.243 |
| ridge | 0.023 | 0.016 | 0.019 | 0.112 | 0.021 | 0.061 | 0.175 | 0.393 | 0.103 |
| ridge_action | 0.023 | 0.016 | 0.021 | 0.115 | 0.021 | 0.061 | 0.174 | 0.392 | 0.103 |

### OOD_v12

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.047 | 0.027 | 0.168 | 0.038 | 0.011 | 0.043 | 0.296 | 0.646 | 0.159 |
| daily_naive | 0.194 | 0.331 | 0.055 | 0.188 | 0.121 | 0.362 | 0.313 | 0.446 | 0.251 |
| ridge | 0.023 | 0.015 | 0.034 | 0.097 | 0.022 | 0.075 | 0.182 | 0.402 | 0.106 |
| ridge_action | 0.023 | 0.016 | 0.034 | 0.102 | 0.022 | 0.076 | 0.182 | 0.404 | 0.107 |

### OOD_long

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.049 | 0.027 | 0.165 | 0.029 | 0.015 | 0.044 | 0.294 | 0.668 | 0.161 |
| daily_naive | 0.191 | 0.319 | 0.029 | 0.104 | 0.134 | 0.366 | 0.312 | 0.438 | 0.237 |
| ridge | 0.022 | 0.015 | 0.017 | 0.088 | 0.028 | 0.061 | 0.174 | 0.391 | 0.100 |
| ridge_action | 0.022 | 0.015 | 0.018 | 0.095 | 0.028 | 0.060 | 0.174 | 0.392 | 0.101 |

## T2 — rollout

From 06:00 of days 3, 10, 17, … of every test episode, roll the one-step model forward 24 h and 168 h; nMAE at the horizon. Daily naive equals persistence at whole-day horizons.

### 24 h — A_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.184 | 0.255 | 0.000 | 0.000 | 0.146 | 0.164 | 0.366 | 0.000 | 0.139 |
| daily_naive | 0.184 | 0.255 | 0.000 | 0.000 | 0.146 | 0.164 | 0.366 | 0.000 | 0.139 |
| ridge | 0.084 | 0.166 | 0.040 | 0.174 | 0.113 | 0.115 | 0.415 | 0.056 | 0.145 |
| ridge_action | 0.084 | 0.166 | 0.041 | 0.174 | 0.112 | 0.116 | 0.419 | 0.056 | 0.146 |

### 24 h — B_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.194 | 0.260 | 0.015 | 0.000 | 0.139 | 0.171 | 0.317 | 0.000 | 0.137 |
| daily_naive | 0.194 | 0.260 | 0.015 | 0.000 | 0.139 | 0.171 | 0.317 | 0.000 | 0.137 |
| ridge | 0.089 | 0.197 | 0.051 | 0.189 | 0.117 | 0.117 | 0.373 | 0.048 | 0.148 |
| ridge_action | 0.088 | 0.181 | 0.059 | 0.201 | 0.116 | 0.119 | 0.374 | 0.057 | 0.149 |

### 24 h — OOD_v12

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.189 | 0.240 | 0.000 | 0.000 | 0.147 | 0.191 | 0.327 | 0.000 | 0.137 |
| daily_naive | 0.189 | 0.240 | 0.000 | 0.000 | 0.147 | 0.191 | 0.327 | 0.000 | 0.137 |
| ridge | 0.080 | 0.256 | 0.142 | 0.410 | 0.135 | 0.291 | 0.288 | 0.067 | 0.209 |
| ridge_action | 0.080 | 0.278 | 0.136 | 0.344 | 0.134 | 0.311 | 0.291 | 0.065 | 0.205 |

### 24 h — OOD_long

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.183 | 0.259 | 0.000 | 0.000 | 0.178 | 0.156 | 0.303 | 0.000 | 0.135 |
| daily_naive | 0.183 | 0.259 | 0.000 | 0.000 | 0.178 | 0.156 | 0.303 | 0.000 | 0.135 |
| ridge | 0.088 | 0.169 | 0.038 | 0.177 | 0.165 | 0.103 | 0.378 | 0.054 | 0.147 |
| ridge_action | 0.089 | 0.169 | 0.040 | 0.179 | 0.163 | 0.104 | 0.384 | 0.055 | 0.148 |

### 168 h — A_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 1.089 | 1.192 | 0.000 | 0.000 | 0.482 | 1.133 | 0.536 | 0.000 | 0.554 |
| daily_naive | 1.089 | 1.192 | 0.000 | 0.000 | 0.482 | 1.133 | 0.536 | 0.000 | 0.554 |
| ridge | 0.596 | 0.436 | 0.472 | 1.373 | 0.205 | 0.510 | 0.656 | 0.211 | 0.557 |
| ridge_action | 0.584 | 0.534 | 0.434 | 1.153 | 0.210 | 0.549 | 0.653 | 0.209 | 0.541 |

### 168 h — B_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 1.085 | 1.214 | 0.000 | 0.000 | 0.504 | 1.124 | 0.462 | 0.000 | 0.549 |
| daily_naive | 1.085 | 1.214 | 0.000 | 0.000 | 0.504 | 1.124 | 0.462 | 0.000 | 0.549 |
| ridge | 0.584 | 0.439 | 0.461 | 1.507 | 0.211 | 0.522 | 0.671 | 0.206 | 0.575 |
| ridge_action | 0.580 | 0.533 | 0.419 | 1.259 | 0.217 | 0.563 | 0.665 | 0.205 | 0.555 |

### 168 h — OOD_v12

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 1.078 | 1.204 | 0.000 | 0.000 | 0.487 | 1.143 | 0.512 | 0.000 | 0.553 |
| daily_naive | 1.078 | 1.204 | 0.000 | 0.000 | 0.487 | 1.143 | 0.512 | 0.000 | 0.553 |
| ridge | 0.973 | 0.327 | 0.279 | 0.645 | 0.270 | 0.395 | 0.538 | 0.192 | 0.453 |
| ridge_action | 0.877 | 0.337 | 0.223 | 0.549 | 0.271 | 0.399 | 0.499 | 0.199 | 0.419 |

### 168 h — OOD_long

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 1.027 | 1.191 | 0.000 | 0.000 | 0.425 | 1.097 | 0.547 | 0.000 | 0.536 |
| daily_naive | 1.027 | 1.191 | 0.000 | 0.000 | 0.425 | 1.097 | 0.547 | 0.000 | 0.536 |
| ridge | 0.569 | 0.398 | 0.441 | 1.351 | 0.238 | 0.495 | 0.642 | 0.212 | 0.543 |
| ridge_action | 0.564 | 0.489 | 0.406 | 1.138 | 0.244 | 0.533 | 0.642 | 0.210 | 0.528 |

## T3 — dark night

At 18:00 of every winter evening of A test: P(fire out at 02:00). Rule: wood stock at 18:00 < tonight's burn (severity rate × 13). Logistic: wood / tonight's burn, members, deadwood total, severity one-hot, night index; fit on A train (2688 evenings, base rate 0.095).

| Baseline | Brier | AUC | Accuracy | Evenings | Dark |
|---|---:|---:|---:|---:|---:|
| rule | 0.033 | 0.981 | 0.967 | 448 | 54 |
| logistic | 0.001 | 1.000 | 0.998 | 448 | 54 |

## T4 — counterfactual

On C_cf test seeds, each variant's effect (variant − factual) on: dark nights of the first winter, cold member-hours over those nights, food store at day 29 06:00, deaths up to then.

### gift_wood

| Baseline | dark_nights MAE | dark_nights sign | cold_member_hours MAE | cold_member_hours sign | food_store MAE | food_store sign | deaths MAE | deaths sign |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| zero_effect | 0.750 | 0.375 | 223.292 | 0.375 | 9.875 | 0.375 | 0.000 | 1.000 |
| val_mean_effect | 0.594 | 0.625 | 161.635 | 0.625 | 7.656 | 0.625 | 0.000 | 1.000 |

### rot_wood

| Baseline | dark_nights MAE | dark_nights sign | cold_member_hours MAE | cold_member_hours sign | food_store MAE | food_store sign | deaths MAE | deaths sign |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| zero_effect | 0.792 | 0.375 | 277.333 | 0.375 | 12.750 | 0.167 | 0.000 | 1.000 |
| val_mean_effect | 0.625 | 0.625 | 192.271 | 0.625 | 9.042 | 0.750 | 0.000 | 1.000 |

### rain

| Baseline | dark_nights MAE | dark_nights sign | cold_member_hours MAE | cold_member_hours sign | food_store MAE | food_store sign | deaths MAE | deaths sign |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| zero_effect | 0.458 | 0.542 | 146.208 | 0.125 | 8.667 | 0.125 | 0.000 | 1.000 |
| val_mean_effect | 0.552 | 0.375 | 156.417 | 0.542 | 8.479 | 0.542 | 0.000 | 1.000 |

### cold_snap

| Baseline | dark_nights MAE | dark_nights sign | cold_member_hours MAE | cold_member_hours sign | food_store MAE | food_store sign | deaths MAE | deaths sign |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| zero_effect | 0.333 | 0.667 | 114.792 | 0.375 | 5.333 | 0.458 | 0.000 | 1.000 |
| val_mean_effect | 0.542 | 0.333 | 102.104 | 0.625 | 5.875 | 0.542 | 0.000 | 1.000 |

### Ground-truth effects (C_cf test)

| Variant | Target | Mean | Median | Min | Max | Non-zero share |
|---|---|---:|---:|---:|---:|---:|
| gift_wood | dark_nights | -0.750 | -1.000 | -2.000 | 0.000 | 0.625 |
| gift_wood | cold_member_hours | -223.292 | -314.500 | -540.000 | 0.000 | 0.625 |
| gift_wood | food_store | 9.875 | 12.500 | 0.000 | 30.000 | 0.625 |
| gift_wood | deaths | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| gift_wood | any target changed | | | | | 0.625 |
| rot_wood | dark_nights | 0.792 | 1.000 | 0.000 | 2.000 | 0.625 |
| rot_wood | cold_member_hours | 277.333 | 420.000 | 0.000 | 499.000 | 0.625 |
| rot_wood | food_store | -12.417 | -17.500 | -29.000 | 2.000 | 0.833 |
| rot_wood | deaths | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| rot_wood | any target changed | | | | | 0.833 |
| rain | dark_nights | 0.292 | 0.000 | -1.000 | 1.000 | 0.458 |
| rain | cold_member_hours | 119.208 | 18.000 | -78.000 | 336.000 | 0.875 |
| rain | food_store | -6.417 | -3.500 | -21.000 | 7.000 | 0.875 |
| rain | deaths | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| rain | any target changed | | | | | 0.958 |
| cold_snap | dark_nights | 0.333 | 0.000 | 0.000 | 1.000 | 0.333 |
| cold_snap | cold_member_hours | 114.792 | 105.000 | 0.000 | 293.000 | 0.625 |
| cold_snap | food_store | -5.333 | -3.000 | -19.000 | 0.000 | 0.542 |
| cold_snap | deaths | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| cold_snap | any target changed | | | | | 0.625 |

## Sanity checks

- Ridge beats persistence on T1 food and wood stock: {"A_test": [true, true], "B_test": [true, true], "OOD_v12": [true, true], "OOD_long": [true, true]}
- T3 logistic AUC ≥ 0.70: True (AUC 1.000)
- Every C_cf variant changes at least one target in ≥ 25 % of test seeds: {"gift_wood": true, "rot_wood": true, "rain": true, "cold_snap": true}

## How to beat this (WM-1)

- **Inputs.** Per row (one simulated hour): the eight settlement-level variables V above, the calendar (`minute_of_day`, `season`, `day_of_season`, `year`), `winter_severity`, and for B_act the interventions (`interventions.json`, applied after the row of their tick). Richer models may read every column of `global.npy` and `settlements.npy`, the agent tables of the `--agents` episodes and `latent.npy`; say which. `lifespan_days` is hidden ground truth: a model of what a player sees must not read it.
- **Splits.** Fit on the `train` role only (A_obs train; A + B train for action-aware models), choose hyper-parameters on `val`, report on `test`, OOD_v12 and OOD_long untouched. C_cf val may be used for T4 calibration, C_cf test only for the report.
- **Metrics.** As above: T1 and T2 per-variable nMAE (and the mean over V), T3 Brier / AUC / accuracy, T4 MAE and sign accuracy per target and variant.
- **Numbers to beat** (best baseline, mean nMAE over V unless stated):
  - T1 A_test: 0.100 (ridge)
  - T1 B_test: 0.103 (ridge)
  - T1 OOD_v12: 0.106 (ridge)
  - T1 OOD_long: 0.100 (ridge)
  - T2 24 h A_test: 0.139 (daily_naive)
  - T2 24 h B_test: 0.137 (daily_naive)
  - T2 24 h OOD_v12: 0.137 (daily_naive)
  - T2 24 h OOD_long: 0.135 (daily_naive)
  - T2 168 h A_test: 0.541 (ridge_action)
  - T2 168 h B_test: 0.549 (daily_naive)
  - T2 168 h OOD_v12: 0.419 (ridge_action)
  - T2 168 h OOD_long: 0.528 (ridge_action)
  - T3: Brier 0.001, AUC 1.000 (logistic / best AUC)
  - T4 gift_wood: mean MAE over the four targets 42.471 (val_mean_effect)
  - T4 rot_wood: mean MAE over the four targets 50.484 (val_mean_effect)
  - T4 rain: mean MAE over the four targets 38.833 (zero_effect)
  - T4 cold_snap: mean MAE over the four targets 27.130 (val_mean_effect)
- **T3 is nearly arithmetic in v0.** The fire burns only stockpile wood from 18:00, one hourly withdrawal at a time, so whether it is out at the 02:00 row follows from the 18:00 stock and the night's rate (eight withdrawals before that row); the logistic baseline learns the threshold (AUC 1.000) where the rule's whole-night threshold (13 hours) is miscalibrated. A world model must reproduce this; a harder dark-night task (a lead time of a day or more) belongs in v1.
- T2's daily-naive prediction equals persistence at whole-day horizons, and both are exact for `cold_members` and `adults_working` at 06:00 (both are always 0 then); the 24-hour 'numbers to beat' therefore favour persistence on those two variables.
- A model that predicts severity-dependent winters, the claim dynamics of wood gatherers and the response to an intervention (T4) should beat the ridge most where the ridge is closest to persistence: the fire and cold variables at 24 h and 168 h, and every T4 target.
