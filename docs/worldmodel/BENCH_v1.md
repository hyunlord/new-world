# WM-Bench v1

Second benchmark for WorldSim world models, on dataset v1 (`DATASET_v1.md`, `game_v14`: winters whose severity, nightly burn and daily deadwood fall are hidden). Produced by `tools/worldmodel/bench_v1.py` (numpy only). Numbers are nMAE = MAE / standard deviation of the variable over the split's scored rows (lower is better) unless stated.

**Inputs are observable fields only** (`wm_data.Episode.observable`): `winter_severity`, `night_burn_per_hour`, `winter_fall_today`, `wood_target` and `latent.npy` are never inputs. Rows marked *reference* read the hidden severity (an oracle); they are not numbers to beat. For these linear baselines they are not an upper bound either: a severity one-hot only shifts a linear model's hourly change, and the tables show it adds nothing measurable (see *Reading the numbers*).

Episodes: {"A_obs": 128, "B_act": 64, "C_cf": 192, "OOD_v13": 32, "OOD_long": 16, "A_train": 96, "B_train": 48}. Ridge λ chosen on validation: {"ridge": 100.0, "ridge_action": 100.0, "ridge_severity_oracle": 100.0}.

Variables V: `food_stock`, `wood_stock`, `fire_lit`, `cold_members`, `alive`, `deadwood_total`, `food_on_bushes_total`, `adults_working`.

## T1 — next hour

Predict V at row t+1 from rows up to t. Train: A train (action-aware: A + B train).

### A_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.046 | 0.027 | 0.167 | 0.032 | 0.011 | 0.044 | 0.294 | 0.639 | 0.157 |
| daily_naive | 0.183 | 0.316 | 0.033 | 0.107 | 0.120 | 0.358 | 0.318 | 0.445 | 0.235 |
| ridge | 0.022 | 0.016 | 0.023 | 0.106 | 0.021 | 0.060 | 0.174 | 0.390 | 0.101 |
| ridge_action | 0.022 | 0.016 | 0.022 | 0.105 | 0.021 | 0.060 | 0.174 | 0.389 | 0.101 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.022 | 0.016 | 0.023 | 0.106 | 0.021 | 0.060 | 0.174 | 0.390 | 0.101 |

### B_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.049 | 0.027 | 0.166 | 0.032 | 0.011 | 0.043 | 0.323 | 0.641 | 0.161 |
| daily_naive | 0.199 | 0.311 | 0.053 | 0.113 | 0.117 | 0.364 | 0.348 | 0.440 | 0.243 |
| ridge | 0.024 | 0.016 | 0.029 | 0.135 | 0.021 | 0.062 | 0.188 | 0.386 | 0.108 |
| ridge_action | 0.024 | 0.016 | 0.029 | 0.132 | 0.021 | 0.061 | 0.186 | 0.384 | 0.107 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.024 | 0.016 | 0.030 | 0.135 | 0.021 | 0.061 | 0.188 | 0.385 | 0.108 |

### OOD_v13

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.046 | 0.027 | 0.166 | 0.026 | 0.011 | 0.043 | 0.276 | 0.665 | 0.157 |
| daily_naive | 0.191 | 0.323 | 0.025 | 0.101 | 0.121 | 0.377 | 0.288 | 0.431 | 0.232 |
| ridge | 0.022 | 0.016 | 0.019 | 0.109 | 0.021 | 0.062 | 0.171 | 0.398 | 0.102 |
| ridge_action | 0.022 | 0.016 | 0.019 | 0.109 | 0.021 | 0.062 | 0.171 | 0.398 | 0.102 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.022 | 0.016 | 0.020 | 0.109 | 0.021 | 0.061 | 0.172 | 0.397 | 0.102 |

### OOD_long

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.044 | 0.027 | 0.165 | 0.031 | 0.015 | 0.043 | 0.289 | 0.671 | 0.161 |
| daily_naive | 0.180 | 0.321 | 0.032 | 0.097 | 0.134 | 0.364 | 0.284 | 0.415 | 0.228 |
| ridge | 0.021 | 0.016 | 0.023 | 0.104 | 0.029 | 0.062 | 0.173 | 0.384 | 0.101 |
| ridge_action | 0.021 | 0.016 | 0.023 | 0.103 | 0.028 | 0.061 | 0.172 | 0.383 | 0.101 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.021 | 0.016 | 0.024 | 0.104 | 0.029 | 0.061 | 0.173 | 0.383 | 0.101 |

## T2 — rollout

From every hour of one-based days 3, 10, 17, … (24 starts per start day; `every_hour`) and, as in v0, from 06:00 only (`0600_only`), roll the one-step model forward 24 h and 168 h; nMAE at the horizon. The severity oracle reads the true hidden severity of every rollout row. At whole-day horizons `daily_naive` (the same hour of the last observed day) is the start row itself, so it equals `persistence` (kept, as in v0).

### 24 h — every_hour — A_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.162 | 0.256 | 0.000 | 0.001 | 0.121 | 0.232 | 0.330 | 0.420 | 0.190 |
| daily_naive | 0.162 | 0.256 | 0.000 | 0.001 | 0.121 | 0.232 | 0.330 | 0.420 | 0.190 |
| ridge | 0.092 | 0.182 | 0.047 | 0.162 | 0.091 | 0.155 | 0.379 | 0.397 | 0.188 |
| ridge_action | 0.091 | 0.182 | 0.046 | 0.160 | 0.091 | 0.156 | 0.379 | 0.397 | 0.188 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.092 | 0.180 | 0.048 | 0.167 | 0.091 | 0.151 | 0.379 | 0.396 | 0.188 |

### 24 h — every_hour — B_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.174 | 0.261 | 0.023 | 0.051 | 0.122 | 0.226 | 0.303 | 0.416 | 0.197 |
| daily_naive | 0.174 | 0.261 | 0.023 | 0.051 | 0.122 | 0.226 | 0.303 | 0.416 | 0.197 |
| ridge | 0.092 | 0.196 | 0.067 | 0.219 | 0.097 | 0.158 | 0.349 | 0.393 | 0.196 |
| ridge_action | 0.092 | 0.195 | 0.067 | 0.213 | 0.096 | 0.161 | 0.349 | 0.393 | 0.196 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.094 | 0.178 | 0.068 | 0.224 | 0.097 | 0.155 | 0.350 | 0.392 | 0.195 |

### 24 h — every_hour — OOD_v13

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.170 | 0.262 | 0.000 | 0.000 | 0.113 | 0.233 | 0.390 | 0.425 | 0.199 |
| daily_naive | 0.170 | 0.262 | 0.000 | 0.000 | 0.113 | 0.233 | 0.390 | 0.425 | 0.199 |
| ridge | 0.097 | 0.180 | 0.039 | 0.166 | 0.087 | 0.161 | 0.427 | 0.415 | 0.196 |
| ridge_action | 0.096 | 0.180 | 0.039 | 0.165 | 0.086 | 0.163 | 0.428 | 0.415 | 0.196 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.096 | 0.165 | 0.044 | 0.186 | 0.087 | 0.155 | 0.426 | 0.414 | 0.197 |

### 24 h — every_hour — OOD_long

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.160 | 0.258 | 0.001 | 0.002 | 0.120 | 0.228 | 0.355 | 0.407 | 0.191 |
| daily_naive | 0.160 | 0.258 | 0.001 | 0.002 | 0.120 | 0.228 | 0.355 | 0.407 | 0.191 |
| ridge | 0.091 | 0.202 | 0.044 | 0.151 | 0.119 | 0.159 | 0.408 | 0.388 | 0.195 |
| ridge_action | 0.091 | 0.201 | 0.044 | 0.150 | 0.118 | 0.160 | 0.408 | 0.388 | 0.195 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.092 | 0.191 | 0.049 | 0.168 | 0.119 | 0.156 | 0.408 | 0.388 | 0.196 |

### 24 h — 0600_only — A_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.171 | 0.260 | 0.000 | 0.000 | 0.141 | 0.179 | 0.321 | 0.000 | 0.134 |
| daily_naive | 0.171 | 0.260 | 0.000 | 0.000 | 0.141 | 0.179 | 0.321 | 0.000 | 0.134 |
| ridge | 0.091 | 0.191 | 0.043 | 0.176 | 0.108 | 0.113 | 0.401 | 0.061 | 0.148 |
| ridge_action | 0.090 | 0.190 | 0.042 | 0.174 | 0.109 | 0.115 | 0.399 | 0.060 | 0.147 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.092 | 0.191 | 0.046 | 0.188 | 0.108 | 0.111 | 0.400 | 0.058 | 0.149 |

### 24 h — 0600_only — B_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.179 | 0.270 | 0.059 | 0.204 | 0.145 | 0.173 | 0.304 | 0.000 | 0.167 |
| daily_naive | 0.179 | 0.270 | 0.059 | 0.204 | 0.145 | 0.173 | 0.304 | 0.000 | 0.167 |
| ridge | 0.088 | 0.204 | 0.084 | 0.329 | 0.117 | 0.109 | 0.365 | 0.058 | 0.169 |
| ridge_action | 0.089 | 0.204 | 0.085 | 0.329 | 0.117 | 0.112 | 0.365 | 0.061 | 0.170 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.091 | 0.185 | 0.089 | 0.353 | 0.116 | 0.105 | 0.366 | 0.054 | 0.170 |

### 24 h — 0600_only — OOD_v13

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.188 | 0.266 | 0.000 | 0.000 | 0.139 | 0.157 | 0.338 | 0.000 | 0.136 |
| daily_naive | 0.188 | 0.266 | 0.000 | 0.000 | 0.139 | 0.157 | 0.338 | 0.000 | 0.136 |
| ridge | 0.081 | 0.188 | 0.034 | 0.191 | 0.108 | 0.113 | 0.414 | 0.053 | 0.148 |
| ridge_action | 0.080 | 0.187 | 0.034 | 0.191 | 0.108 | 0.116 | 0.412 | 0.052 | 0.148 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.081 | 0.172 | 0.038 | 0.198 | 0.108 | 0.105 | 0.413 | 0.051 | 0.146 |

### 24 h — 0600_only — OOD_long

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.175 | 0.261 | 0.000 | 0.000 | 0.146 | 0.169 | 0.314 | 0.000 | 0.133 |
| daily_naive | 0.175 | 0.261 | 0.000 | 0.000 | 0.146 | 0.169 | 0.314 | 0.000 | 0.133 |
| ridge | 0.081 | 0.207 | 0.038 | 0.162 | 0.141 | 0.112 | 0.401 | 0.054 | 0.150 |
| ridge_action | 0.081 | 0.206 | 0.037 | 0.161 | 0.141 | 0.114 | 0.399 | 0.054 | 0.149 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.082 | 0.197 | 0.045 | 0.186 | 0.141 | 0.111 | 0.400 | 0.053 | 0.152 |

### 168 h — every_hour — A_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 1.032 | 1.219 | 0.000 | 0.000 | 0.478 | 1.161 | 0.678 | 0.542 | 0.639 |
| daily_naive | 1.032 | 1.219 | 0.000 | 0.000 | 0.478 | 1.161 | 0.678 | 0.542 | 0.639 |
| ridge | 0.268 | 0.280 | 0.062 | 0.205 | 0.197 | 0.186 | 0.491 | 0.481 | 0.271 |
| ridge_action | 0.268 | 0.276 | 0.055 | 0.190 | 0.198 | 0.186 | 0.483 | 0.481 | 0.267 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.264 | 0.274 | 0.072 | 0.238 | 0.199 | 0.180 | 0.495 | 0.476 | 0.275 |

### 168 h — every_hour — B_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 1.114 | 1.234 | 0.026 | 0.084 | 0.460 | 1.163 | 0.612 | 0.527 | 0.653 |
| daily_naive | 1.114 | 1.234 | 0.026 | 0.084 | 0.460 | 1.163 | 0.612 | 0.527 | 0.653 |
| ridge | 0.254 | 0.287 | 0.071 | 0.239 | 0.181 | 0.201 | 0.500 | 0.447 | 0.272 |
| ridge_action | 0.221 | 0.267 | 0.078 | 0.242 | 0.182 | 0.197 | 0.487 | 0.447 | 0.265 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.249 | 0.277 | 0.084 | 0.277 | 0.181 | 0.196 | 0.507 | 0.442 | 0.277 |

### 168 h — every_hour — OOD_v13

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 1.082 | 1.238 | 0.000 | 0.000 | 0.458 | 1.144 | 0.841 | 0.544 | 0.663 |
| daily_naive | 1.082 | 1.238 | 0.000 | 0.000 | 0.458 | 1.144 | 0.841 | 0.544 | 0.663 |
| ridge | 0.266 | 0.260 | 0.050 | 0.214 | 0.177 | 0.201 | 0.473 | 0.493 | 0.267 |
| ridge_action | 0.269 | 0.256 | 0.046 | 0.202 | 0.178 | 0.202 | 0.468 | 0.494 | 0.264 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.264 | 0.240 | 0.064 | 0.265 | 0.179 | 0.188 | 0.472 | 0.487 | 0.270 |

### 168 h — every_hour — OOD_long

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.953 | 1.212 | 0.000 | 0.000 | 0.386 | 1.128 | 0.727 | 0.525 | 0.616 |
| daily_naive | 0.953 | 1.212 | 0.000 | 0.000 | 0.386 | 1.128 | 0.727 | 0.525 | 0.616 |
| ridge | 0.288 | 0.284 | 0.062 | 0.206 | 0.214 | 0.183 | 0.496 | 0.465 | 0.275 |
| ridge_action | 0.289 | 0.281 | 0.055 | 0.191 | 0.214 | 0.183 | 0.490 | 0.466 | 0.271 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.285 | 0.270 | 0.076 | 0.250 | 0.215 | 0.176 | 0.500 | 0.460 | 0.279 |

### 168 h — 0600_only — A_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 1.032 | 1.190 | 0.000 | 0.000 | 0.519 | 1.137 | 0.525 | 0.000 | 0.550 |
| daily_naive | 1.032 | 1.190 | 0.000 | 0.000 | 0.519 | 1.137 | 0.525 | 0.000 | 0.550 |
| ridge | 0.259 | 0.289 | 0.103 | 0.453 | 0.199 | 0.144 | 0.592 | 0.229 | 0.284 |
| ridge_action | 0.260 | 0.285 | 0.097 | 0.435 | 0.200 | 0.142 | 0.577 | 0.228 | 0.278 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.258 | 0.285 | 0.106 | 0.435 | 0.200 | 0.137 | 0.573 | 0.212 | 0.276 |

### 168 h — 0600_only — B_test

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 1.109 | 1.210 | 0.078 | 0.289 | 0.503 | 1.137 | 0.576 | 0.000 | 0.613 |
| daily_naive | 1.109 | 1.210 | 0.078 | 0.289 | 0.503 | 1.137 | 0.576 | 0.000 | 0.613 |
| ridge | 0.249 | 0.322 | 0.133 | 0.582 | 0.181 | 0.145 | 0.657 | 0.225 | 0.312 |
| ridge_action | 0.214 | 0.294 | 0.122 | 0.558 | 0.182 | 0.142 | 0.633 | 0.228 | 0.297 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.246 | 0.315 | 0.140 | 0.582 | 0.181 | 0.139 | 0.644 | 0.207 | 0.307 |

### 168 h — 0600_only — OOD_v13

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 1.070 | 1.186 | 0.000 | 0.000 | 0.496 | 1.132 | 0.453 | 0.000 | 0.542 |
| daily_naive | 1.070 | 1.186 | 0.000 | 0.000 | 0.496 | 1.132 | 0.453 | 0.000 | 0.542 |
| ridge | 0.237 | 0.280 | 0.088 | 0.497 | 0.180 | 0.135 | 0.516 | 0.221 | 0.269 |
| ridge_action | 0.239 | 0.275 | 0.084 | 0.482 | 0.180 | 0.133 | 0.501 | 0.220 | 0.264 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.234 | 0.259 | 0.089 | 0.449 | 0.182 | 0.124 | 0.488 | 0.206 | 0.254 |

### 168 h — 0600_only — OOD_long

| Baseline | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.947 | 1.177 | 0.000 | 0.000 | 0.422 | 1.104 | 0.463 | 0.000 | 0.514 |
| daily_naive | 0.947 | 1.177 | 0.000 | 0.000 | 0.422 | 1.104 | 0.463 | 0.000 | 0.514 |
| ridge | 0.270 | 0.287 | 0.100 | 0.438 | 0.224 | 0.141 | 0.581 | 0.219 | 0.282 |
| ridge_action | 0.270 | 0.284 | 0.094 | 0.420 | 0.224 | 0.139 | 0.566 | 0.218 | 0.277 |
| ridge_severity_oracle (reference: reads hidden severity) | 0.269 | 0.276 | 0.108 | 0.440 | 0.225 | 0.133 | 0.564 | 0.204 | 0.277 |

## T3 — dark night (arithmetic: 18:00 stock vs tonight's rate)

At 18:00 of every winter evening of A test: P(fire out at 02:00). The rule (stock < severity rate × 13) needs the hidden severity and is a reference; `logistic_observable` reads the stock, members, deadwood, night index, last night's stock drop and the deadwood change since yesterday; `logistic_severity_oracle` is v0's logistic. Fit on A train (2688 evenings, base rate 0.153).

| Baseline | Brier | AUC | Accuracy | Evenings | Dark |
|---|---:|---:|---:|---:|---:|
| logistic_observable | 0.020 | 0.993 | 0.975 | 448 | 64 |
| rule_severity_oracle (reference) | 0.036 | 0.979 | 0.964 | 448 | 64 |
| logistic_severity_oracle (reference) | 0.021 | 0.995 | 0.973 | 448 | 64 |

## T3b — winter outlook

At 06:00 of each winter's third day (one-based days 24, 52, 80, 108) of A test: how many of the five remaining nights go dark (fire out at 02:00)? Ridge features (observable): `wood_stock`, `deadwood_total`, `drop_last_night`, `drop_night_before`, `deadwood_change_1`, `deadwood_change_2`, `members` (λ 1.0 on A val). Train: 384 winters; train mean 1.070.

| Baseline | MAE | Exact accuracy | Winters |
|---|---:|---:|---:|
| train_mean | 0.995 | 0.203 | 64 |
| ridge | 0.600 | 0.516 | 64 |
| severity_oracle_reference | 0.609 | 0.547 | 64 |

Dark nights of the five by severity (A test, count of winters): {"0": {"0": 23, "1": 2, "2": 1, "3": 1, "4": 0, "5": 0}, "1": {"0": 8, "1": 8, "2": 8, "3": 2, "4": 0, "5": 0}, "2": {"0": 0, "1": 3, "2": 2, "3": 4, "4": 2, "5": 0}}

## T4 — counterfactual

On C_cf test seeds, each variant's effect (variant − factual) on the first winter: dark nights, cold member-hours over those nights, food store at day 29 06:00 (ranked), deaths up to then (reported only). nMAE = MAE / mean |true effect| over the test seeds (— when every effect is 0). No mean across units.

### gift_wood

| Baseline | Target | MAE | Sign accuracy | nMAE |
|---|---|---:|---:|---:|
| zero_effect | dark_nights | 0.833 | 0.292 | 1.000 |
| zero_effect | cold_member_hours | 276.083 | 0.250 | 1.000 |
| zero_effect | food_store | 14.958 | 0.167 | 1.000 |
| zero_effect | deaths (reported only) | 0.000 | 1.000 | — |
| val_mean_effect | dark_nights | 0.521 | 0.708 | 0.625 |
| val_mean_effect | cold_member_hours | 167.208 | 0.750 | 0.606 |
| val_mean_effect | food_store | 11.469 | 0.750 | 0.767 |
| val_mean_effect | deaths (reported only) | 0.000 | 1.000 | — |

### rot_wood

| Baseline | Target | MAE | Sign accuracy | nMAE |
|---|---|---:|---:|---:|
| zero_effect | dark_nights | 1.125 | 0.167 | 1.000 |
| zero_effect | cold_member_hours | 403.167 | 0.000 | 1.000 |
| zero_effect | food_store | 18.958 | 0.000 | 1.000 |
| zero_effect | deaths (reported only) | 0.000 | 1.000 | — |
| val_mean_effect | dark_nights | 0.531 | 0.833 | 0.472 |
| val_mean_effect | cold_member_hours | 147.833 | 1.000 | 0.367 |
| val_mean_effect | food_store | 10.333 | 0.958 | 0.545 |
| val_mean_effect | deaths (reported only) | 0.000 | 1.000 | — |

### rain

| Baseline | Target | MAE | Sign accuracy | nMAE |
|---|---|---:|---:|---:|
| zero_effect | dark_nights | 0.708 | 0.292 | 1.000 |
| zero_effect | cold_member_hours | 217.625 | 0.125 | 1.000 |
| zero_effect | food_store | 9.750 | 0.042 | 1.000 |
| zero_effect | deaths (reported only) | 0.000 | 1.000 | — |
| val_mean_effect | dark_nights | 0.771 | 0.375 | 1.088 |
| val_mean_effect | cold_member_hours | 214.125 | 0.542 | 0.984 |
| val_mean_effect | food_store | 9.812 | 0.542 | 1.006 |
| val_mean_effect | deaths (reported only) | 0.000 | 1.000 | — |

### cold_snap

| Baseline | Target | MAE | Sign accuracy | nMAE |
|---|---|---:|---:|---:|
| zero_effect | dark_nights | 0.292 | 0.750 | 1.000 |
| zero_effect | cold_member_hours | 134.292 | 0.333 | 1.000 |
| zero_effect | food_store | 6.167 | 0.292 | 1.000 |
| zero_effect | deaths (reported only) | 0.000 | 1.000 | — |
| val_mean_effect | dark_nights | 0.479 | 0.250 | 1.643 |
| val_mean_effect | cold_member_hours | 116.792 | 0.667 | 0.870 |
| val_mean_effect | food_store | 5.479 | 0.708 | 0.889 |
| val_mean_effect | deaths (reported only) | 0.000 | 1.000 | — |

### windfall

| Baseline | Target | MAE | Sign accuracy | nMAE |
|---|---|---:|---:|---:|
| zero_effect | dark_nights | 0.375 | 0.625 | 1.000 |
| zero_effect | cold_member_hours | 110.792 | 0.250 | 1.000 |
| zero_effect | food_store | 6.917 | 0.167 | 1.000 |
| zero_effect | deaths (reported only) | 0.000 | 1.000 | — |
| val_mean_effect | dark_nights | 0.469 | 0.375 | 1.250 |
| val_mean_effect | cold_member_hours | 69.271 | 0.708 | 0.625 |
| val_mean_effect | food_store | 5.604 | 0.708 | 0.810 |
| val_mean_effect | deaths (reported only) | 0.000 | 1.000 | — |

### Ground-truth effects (C_cf test)

| Variant | Target | Mean | Mean abs | Min | Max | Non-zero share |
|---|---|---:|---:|---:|---:|---:|
| gift_wood | dark_nights | -0.833 | 0.833 | -2.000 | 0.000 | 0.708 |
| gift_wood | cold_member_hours | -276.083 | 276.083 | -672.000 | 0.000 | 0.750 |
| gift_wood | food_store | 12.958 | 14.958 | -17.000 | 39.000 | 0.833 |
| gift_wood | deaths | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| gift_wood | any ranked target changed | | | | | 0.833 |
| rot_wood | dark_nights | 1.125 | 1.125 | 0.000 | 3.000 | 0.833 |
| rot_wood | cold_member_hours | 403.167 | 403.167 | 84.000 | 882.000 | 1.000 |
| rot_wood | food_store | -17.375 | 18.958 | -43.000 | 19.000 | 1.000 |
| rot_wood | deaths | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| rot_wood | any ranked target changed | | | | | 1.000 |
| rain | dark_nights | 0.042 | 0.708 | -1.000 | 1.000 | 0.708 |
| rain | cold_member_hours | 52.125 | 217.625 | -463.000 | 336.000 | 0.875 |
| rain | food_store | -1.167 | 9.750 | -18.000 | 23.000 | 0.958 |
| rain | deaths | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| rain | any ranked target changed | | | | | 0.958 |
| cold_snap | dark_nights | 0.292 | 0.292 | 0.000 | 2.000 | 0.250 |
| cold_snap | cold_member_hours | 134.292 | 134.292 | 0.000 | 504.000 | 0.667 |
| cold_snap | food_store | -6.167 | 6.167 | -24.000 | 0.000 | 0.708 |
| cold_snap | deaths | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| cold_snap | any ranked target changed | | | | | 0.708 |
| windfall | dark_nights | -0.375 | 0.375 | -1.000 | 0.000 | 0.375 |
| windfall | cold_member_hours | -107.292 | 110.792 | -252.000 | 42.000 | 0.750 |
| windfall | food_store | 5.583 | 6.917 | -6.000 | 22.000 | 0.833 |
| windfall | deaths | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| windfall | any ranked target changed | | | | | 0.875 |

## T5 — agent next hour

A agent episodes (test seeds [10112, 10113, 10114, 10115, 10116, 10117, 10118, 10119]): every agent alive at t+1, t and t−23 (one pair set for both baselines). Persistence predicts row t, `day_earlier` row t+1−24. nMAE = MAE / std of the true next value over those pairs.

| Baseline | State tag acc. | Target kind acc. | Position Chebyshev | Hunger nMAE | Thirst nMAE | Fatigue nMAE | Pairs |
|---|---:|---:|---:|---:|---:|---:|---:|
| persistence | 0.741 | 0.714 | 2.490 | 0.450 | 0.607 | 0.256 | 924554 |
| day_earlier | 0.757 | 0.738 | 4.749 | 1.223 | 0.696 | 0.172 | 924554 |

## Sanity checks

- Ridge beats persistence on T1 food and wood stock: {"A_test": [true, true], "B_test": [true, true], "OOD_v13": [true, true], "OOD_long": [true, true]}
- T3b ridge beats the train mean: True
- Every C_cf variant changes at least one ranked target in ≥ 25 % of test seeds: {"gift_wood": true, "rot_wood": true, "rain": true, "cold_snap": true, "windfall": true}
- The severity oracle's T3b MAE is not 0: True (MAE 0.609)

## v0 and v1 side by side

Best non-oracle baseline (mean nMAE over V for T1/T2). v0: dataset v0 (`game_v13`, severity an input); v1: dataset v1 (`game_v14`, observable inputs only). OOD_v12 (v0) and OOD_v13 (v1) are each pass's previous ruleset.

| Task | Split | v0 | v1 |
|---|---|---:|---:|
| T1 next hour | A_test (v0 A_test) | 0.100 (ridge) | 0.101 (ridge_action) |
| T1 next hour | B_test (v0 B_test) | 0.103 (ridge) | 0.107 (ridge_action) |
| T1 next hour | OOD_long (v0 OOD_long) | 0.100 (ridge) | 0.101 (ridge_action) |
| T1 next hour | OOD_v13 (v0 OOD_v12) | 0.106 (ridge) | 0.102 (ridge_action) |
| T2 24 h, 06:00 starts | A_test | 0.139 (daily_naive) | 0.134 (daily_naive); every hour 0.188 (ridge_action) |
| T2 24 h, 06:00 starts | B_test | 0.137 (daily_naive) | 0.167 (daily_naive); every hour 0.196 (ridge_action) |
| T2 24 h, 06:00 starts | OOD_long | 0.135 (daily_naive) | 0.133 (daily_naive); every hour 0.191 (daily_naive) |
| T2 24 h, 06:00 starts | OOD_v13 | 0.137 (daily_naive) | 0.136 (daily_naive); every hour 0.196 (ridge_action) |
| T2 168 h, 06:00 starts | A_test | 0.541 (ridge_action) | 0.278 (ridge_action); every hour 0.267 (ridge_action) |
| T2 168 h, 06:00 starts | B_test | 0.549 (daily_naive) | 0.297 (ridge_action); every hour 0.265 (ridge_action) |
| T2 168 h, 06:00 starts | OOD_long | 0.528 (ridge_action) | 0.277 (ridge_action); every hour 0.271 (ridge_action) |
| T2 168 h, 06:00 starts | OOD_v13 | 0.419 (ridge_action) | 0.264 (ridge_action); every hour 0.264 (ridge_action) |
| T3 Brier / AUC | A_test | logistic (severity input) 0.001 / 1.000 | observable 0.020 / 0.993; oracle 0.021 / 0.995 |
| T4 cold_snap dark_nights MAE | C_cf test | 0.333 | 0.292 |
| T4 cold_snap cold_member_hours MAE | C_cf test | 102.104 | 116.792 |
| T4 cold_snap food_store MAE | C_cf test | 5.333 | 5.479 |
| T4 gift_wood dark_nights MAE | C_cf test | 0.594 | 0.521 |
| T4 gift_wood cold_member_hours MAE | C_cf test | 161.635 | 167.208 |
| T4 gift_wood food_store MAE | C_cf test | 7.656 | 11.469 |
| T4 rain dark_nights MAE | C_cf test | 0.458 | 0.708 |
| T4 rain cold_member_hours MAE | C_cf test | 146.208 | 214.125 |
| T4 rain food_store MAE | C_cf test | 8.479 | 9.750 |
| T4 rot_wood dark_nights MAE | C_cf test | 0.625 | 0.531 |
| T4 rot_wood cold_member_hours MAE | C_cf test | 192.271 | 147.833 |
| T4 rot_wood food_store MAE | C_cf test | 9.042 | 10.333 |

## Numbers to beat (WM-1)

- **Inputs.** Observable fields only (`Episode.observable` of each array, `interventions.json` for actions). A model that reads `winter_severity`, `night_burn_per_hour`, `winter_fall_today`, `wood_target`, `latent.npy` or `lifespan_days` is an oracle and must say so.
- **Splits.** Fit on `train` (A_obs; A + B for action-aware), choose on `val`, report on `test`, OOD_v13 and OOD_long untouched; C_cf val may calibrate T4.
- **Best non-oracle baseline:**
  - T1 A_test: mean nMAE 0.101 (ridge_action)
  - T1 B_test: mean nMAE 0.107 (ridge_action)
  - T1 OOD_v13: mean nMAE 0.102 (ridge_action)
  - T1 OOD_long: mean nMAE 0.101 (ridge_action)
  - T2 24 h every-hour starts A_test: 0.188 (ridge_action)
  - T2 24 h every-hour starts B_test: 0.196 (ridge_action)
  - T2 24 h every-hour starts OOD_v13: 0.196 (ridge_action)
  - T2 24 h every-hour starts OOD_long: 0.191 (daily_naive)
  - T2 168 h every-hour starts A_test: 0.267 (ridge_action)
  - T2 168 h every-hour starts B_test: 0.265 (ridge_action)
  - T2 168 h every-hour starts OOD_v13: 0.264 (ridge_action)
  - T2 168 h every-hour starts OOD_long: 0.271 (ridge_action)
  - T3 (observable): Brier 0.020, AUC 0.993
  - T3b: MAE 0.600, exact accuracy 0.516 (ridge); the severity oracle reference reaches MAE 0.609
  - T4 gift_wood: dark_nights MAE 0.521 (nMAE 0.625, val_mean_effect); cold_member_hours MAE 167.208 (nMAE 0.606, val_mean_effect); food_store MAE 11.469 (nMAE 0.767, val_mean_effect)
  - T4 rot_wood: dark_nights MAE 0.531 (nMAE 0.472, val_mean_effect); cold_member_hours MAE 147.833 (nMAE 0.367, val_mean_effect); food_store MAE 10.333 (nMAE 0.545, val_mean_effect)
  - T4 rain: dark_nights MAE 0.708 (nMAE 1.000, zero_effect); cold_member_hours MAE 214.125 (nMAE 0.984, val_mean_effect); food_store MAE 9.750 (nMAE 1.000, zero_effect)
  - T4 cold_snap: dark_nights MAE 0.292 (nMAE 1.000, zero_effect); cold_member_hours MAE 116.792 (nMAE 0.870, val_mean_effect); food_store MAE 5.479 (nMAE 0.889, val_mean_effect)
  - T4 windfall: dark_nights MAE 0.375 (nMAE 1.000, zero_effect); cold_member_hours MAE 69.271 (nMAE 0.625, val_mean_effect); food_store MAE 5.604 (nMAE 0.810, val_mean_effect)
  - T5 (persistence): state tag 0.741, target kind 0.714, Chebyshev 2.490, hunger/thirst/fatigue nMAE 0.450 / 0.607 / 0.256

## Reading the numbers

- **T3b needs the winter inferred.** The observable ridge reaches MAE 0.600 against the train mean's 0.995, level with the severity oracle (0.609). Ablation (same splits and λ grid): without the two stock-drop features 0.605, stock and deadwood level and changes alone 0.601, the two stock drops alone 0.737. The signal a linear model finds is in the stock and the deadwood (whose daily fall is the other drawn quantity), not in last nights' burning. A model should beat the ridge and the oracle.
- **T3 stays nearly arithmetic**: the 18:00 stock alone gives AUC 0.994 (observable logistic 0.993; without last night's drop 0.994). The fire burns only stockpile wood from 18:00, so the 02:00 state follows mostly from the 18:00 stock. Kept for comparison with v0 and labelled so.
- **T2 at 168 h** (A test, every-hour starts): ridge 0.271, action ridge 0.267, severity oracle 0.275 (reads each rollout row's true severity), persistence 0.639. The oracle is worse than the plain ridge in 4 of 4 test splits: knowing the severity buys a linear model nothing here.
  v0's ridge read the severity as an input and held the start row's value through the rollout; its 168 h A-test number (06:00 starts) was 0.557 (persistence 0.554). In a first v1 run the oracle was built the same way and scored 0.643 at 06:00 starts (persistence 0.550) and 0.691 at every-hour starts (persistence 0.639), first-run numbers recorded in METRICS: holding a seasonal label constant across a season boundary pushes a rollout off its training distribution, a likely reason v0's 168 h ridge stayed near persistence (not re-measured on v0, whose bench is frozen).
- Every-hour starts make T2 at 24 h harder than v0's 06:00-only protocol (A test best 0.188 against 0.134): 06:00 falls in the quiet early-morning hours (`adults_working` is 0 from 04:00 to 06:00, and at A test's 06:00 start rows `cold_members` is also 0), while the late-morning hours are the busiest.
- At whole-day horizons `daily_naive` equals `persistence` (v0's definition, kept for comparability).
- T3's oracle rule reads tonight's rate as `--burn` × 13 (`2,3,4`, `game_v14()`'s severity burns, given on the command line because the recorder's meta has no rules block); it must be updated if those burns change.
