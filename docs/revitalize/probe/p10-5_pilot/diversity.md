# Pass 10 diversity pilot

Train seeds 20000..20095 (calendar statistics fitted here), test seeds 20096..20111 (every number below). WM-1.1's definitions (`calcore_wm11.py`): R²_abs / R²_year against the train calendar means, η² = between-seed share of the variance, divergence day over test pairs, spread = std over seeds at 06:00 / split std, T2 nMAE at every-hour starts.

## η²

| Variable | game_v15 | game_v16_neutral | game_v16 |
|---|---:|---:|---:|
| food_stock | 0.153 | 0.112 | 0.500 |
| wood_stock | 0.055 | 0.056 | 0.063 |
| fire_lit | 0.041 | 0.042 | 0.040 |
| cold_members | 0.586 | 0.603 | 0.582 |
| alive | 0.021 | 0.030 | 0.978 |
| deadwood_total | 0.030 | 0.030 | 0.168 |
| food_on_bushes_total | 0.124 | 0.154 | 0.503 |
| adults_working | 0.304 | 0.327 | 0.484 |

## R²_abs

| Variable | game_v15 | game_v16_neutral | game_v16 |
|---|---:|---:|---:|
| food_stock | 0.845 | 0.885 | 0.474 |
| wood_stock | 0.942 | 0.941 | 0.934 |
| fire_lit | 0.956 | 0.955 | 0.957 |
| cold_members | 0.359 | 0.361 | 0.354 |
| alive | 0.978 | 0.967 | -0.043 |
| deadwood_total | 0.968 | 0.968 | 0.827 |
| food_on_bushes_total | 0.872 | 0.839 | 0.468 |
| adults_working | 0.676 | 0.653 | 0.474 |

## R²_year

| Variable | game_v15 | game_v16_neutral | game_v16 |
|---|---:|---:|---:|
| food_stock | 0.820 | 0.842 | 0.436 |
| wood_stock | 0.943 | 0.941 | 0.933 |
| fire_lit | 0.957 | 0.956 | 0.958 |
| cold_members | 0.377 | 0.372 | 0.368 |
| alive | 0.157 | 0.164 | -0.061 |
| deadwood_total | 0.968 | 0.968 | 0.827 |
| food_on_bushes_total | 0.870 | 0.832 | 0.466 |
| adults_working | 0.673 | 0.649 | 0.472 |

## H7 (η² ≥ 0.30 and R²_abs ≤ 0.70 for food, wood, alive)

| Scene | Verdict | food_stock | wood_stock | alive |
|---|---|---|---|---|
| game_v15 | FAIL | η² 0.153, R²_abs 0.845 | η² 0.055, R²_abs 0.942 | η² 0.021, R²_abs 0.978 |
| game_v16_neutral | FAIL | η² 0.112, R²_abs 0.885 | η² 0.056, R²_abs 0.941 | η² 0.030, R²_abs 0.967 |
| game_v16 | FAIL | η² 0.500, R²_abs 0.474 | η² 0.063, R²_abs 0.934 | η² 0.978, R²_abs -0.043 |

## Divergence day (test pairs: min / median / max; never)

| Variable | game_v15 | game_v16_neutral | game_v16 |
|---|---|---|---|
| (any) | 1 / 1.0 / 1; 0 | 1 / 1.0 / 1; 0 | 1 / 1.0 / 1; 0 |
| food_stock | 1 / 1.0 / 1; 0 | 1 / 1.0 / 1; 0 | 1 / 1.0 / 1; 0 |
| wood_stock | 1 / 1.0 / 1; 0 | 1 / 1.0 / 1; 0 | 1 / 1.0 / 1; 0 |
| fire_lit | 26 / 28.0 / 56; 0 | 26 / 28.0 / 85; 0 | 27 / 28.0 / 54; 0 |
| cold_members | 26 / 28.0 / 82; 0 | 26 / 28.0 / 111; 0 | 27 / 28.0 / 54; 0 |
| alive | 2 / 18.0 / 27; 0 | 2 / 18.0 / 28; 0 | 1 / 1.0 / 13; 0 |
| deadwood_total | 1 / 1.0 / 1; 0 | 1 / 1.0 / 1; 0 | 1 / 1.0 / 1; 0 |
| food_on_bushes_total | 1 / 1.0 / 1; 0 | 1 / 1.0 / 1; 0 | 1 / 1.0 / 1; 0 |
| adults_working | 1 / 1.0 / 1; 0 | 1 / 1.0 / 1; 0 | 1 / 1.0 / 1; 0 |

## Between-seed spread at 06:00 (std over test seeds / split std)

**game_v15**

| Day | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 30 | 0.271 | 0.092 | 0.000 | 0.000 | 0.000 | 0.233 | 0.080 | 0.000 |
| 60 | 0.306 | 0.150 | 0.000 | 0.000 | 0.236 | 0.030 | 0.158 | 0.000 |
| 90 | 0.366 | 0.164 | 0.000 | 0.000 | 0.098 | 0.027 | 0.143 | 0.000 |
| 118 | 0.435 | 0.097 | 0.000 | 0.000 | 0.188 | 0.053 | 0.152 | 0.000 |

**game_v16_neutral**

| Day | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 30 | 0.274 | 0.060 | 0.000 | 0.000 | 0.118 | 0.276 | 0.135 | 0.000 |
| 60 | 0.276 | 0.153 | 0.000 | 0.000 | 0.272 | 0.052 | 0.212 | 0.000 |
| 90 | 0.280 | 0.148 | 0.000 | 0.000 | 0.161 | 0.037 | 0.115 | 0.000 |
| 118 | 0.397 | 0.154 | 0.000 | 0.000 | 0.256 | 0.018 | 0.131 | 0.000 |

**game_v16**

| Day | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 30 | 0.570 | 0.095 | 0.000 | 0.000 | 1.045 | 0.442 | 0.431 | 0.000 |
| 60 | 0.600 | 0.152 | 0.000 | 0.000 | 1.007 | 0.433 | 0.744 | 0.000 |
| 90 | 0.645 | 0.155 | 0.000 | 0.000 | 1.043 | 0.432 | 0.556 | 0.000 |
| 118 | 0.666 | 0.171 | 0.000 | 0.000 | 1.017 | 0.426 | 0.697 | 0.000 |

## T2 24 h nMAE (every-hour starts, test split std)

**game_v15**

| Member | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| cal_abs | 0.263 | 0.174 | 0.001 | 0.002 | 0.064 | 0.128 | 0.209 | 0.368 | 0.151 |
| apfit_abs | 0.060 | 0.174 | 0.001 | 0.002 | 0.060 | 0.126 | 0.204 | 0.358 | 0.123 |
| apfit_year | 0.067 | 0.175 | 0.001 | 0.002 | 0.094 | 0.125 | 0.206 | 0.361 | 0.129 |
| an5 | 0.099 | 0.147 | 0.001 | 0.002 | 0.060 | 0.079 | 0.179 | 0.329 | 0.112 |
| an5_off | 0.053 | 0.150 | 0.001 | 0.002 | 0.080 | 0.107 | 0.196 | 0.366 | 0.119 |
| persistence | 0.169 | 0.258 | 0.001 | 0.001 | 0.115 | 0.228 | 0.344 | 0.414 | 0.191 |

**game_v16_neutral**

| Member | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| cal_abs | 0.238 | 0.183 | 0.002 | 0.003 | 0.075 | 0.132 | 0.268 | 0.389 | 0.161 |
| apfit_abs | 0.062 | 0.179 | 0.002 | 0.003 | 0.065 | 0.127 | 0.250 | 0.380 | 0.134 |
| apfit_year | 0.068 | 0.180 | 0.002 | 0.003 | 0.094 | 0.127 | 0.252 | 0.382 | 0.138 |
| an5 | 0.095 | 0.142 | 0.001 | 0.001 | 0.065 | 0.079 | 0.224 | 0.353 | 0.120 |
| an5_off | 0.060 | 0.147 | 0.001 | 0.001 | 0.085 | 0.109 | 0.242 | 0.399 | 0.130 |
| persistence | 0.165 | 0.263 | 0.001 | 0.001 | 0.122 | 0.239 | 0.415 | 0.438 | 0.206 |

**game_v16**

| Member | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| cal_abs | 0.579 | 0.184 | 0.006 | 0.007 | 0.833 | 0.316 | 0.518 | 0.473 | 0.364 |
| apfit_abs | 0.090 | 0.175 | 0.005 | 0.007 | 0.026 | 0.179 | 0.406 | 0.450 | 0.167 |
| apfit_year | 0.092 | 0.176 | 0.005 | 0.007 | 0.035 | 0.179 | 0.408 | 0.453 | 0.169 |
| an5 | 0.167 | 0.150 | 0.003 | 0.005 | 0.128 | 0.188 | 0.387 | 0.427 | 0.182 |
| an5_off | 0.083 | 0.162 | 0.003 | 0.005 | 0.025 | 0.146 | 0.401 | 0.505 | 0.166 |
| persistence | 0.142 | 0.276 | 0.001 | 0.001 | 0.024 | 0.222 | 0.469 | 0.483 | 0.202 |

## T2 168 h nMAE (every-hour starts, test split std)

**game_v15**

| Member | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| cal_abs | 0.270 | 0.160 | 0.000 | 0.000 | 0.068 | 0.140 | 0.321 | 0.362 | 0.165 |
| apfit_abs | 0.150 | 0.160 | 0.000 | 0.000 | 0.068 | 0.140 | 0.326 | 0.355 | 0.150 |
| apfit_year | 0.190 | 0.160 | 0.000 | 0.000 | 0.365 | 0.140 | 0.327 | 0.357 | 0.192 |
| an5 | 0.135 | 0.168 | 0.000 | 0.000 | 0.069 | 0.146 | 0.227 | 0.353 | 0.137 |
| an5_off | 0.127 | 0.201 | 0.000 | 0.000 | 0.105 | 0.169 | 0.267 | 0.418 | 0.161 |
| persistence | 1.042 | 1.223 | 0.000 | 0.000 | 0.453 | 1.171 | 0.769 | 0.530 | 0.648 |

**game_v16_neutral**

| Member | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| cal_abs | 0.249 | 0.168 | 0.000 | 0.000 | 0.081 | 0.144 | 0.331 | 0.377 | 0.169 |
| apfit_abs | 0.169 | 0.168 | 0.000 | 0.000 | 0.081 | 0.144 | 0.336 | 0.370 | 0.159 |
| apfit_year | 0.212 | 0.168 | 0.000 | 0.000 | 0.360 | 0.144 | 0.354 | 0.375 | 0.202 |
| an5 | 0.142 | 0.172 | 0.000 | 0.000 | 0.078 | 0.147 | 0.283 | 0.370 | 0.149 |
| an5_off | 0.147 | 0.201 | 0.000 | 0.000 | 0.125 | 0.169 | 0.325 | 0.437 | 0.176 |
| persistence | 1.117 | 1.226 | 0.000 | 0.000 | 0.475 | 1.174 | 0.796 | 0.537 | 0.666 |

**game_v16**

| Member | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| cal_abs | 0.584 | 0.202 | 0.003 | 0.001 | 0.861 | 0.371 | 0.539 | 0.457 | 0.377 |
| apfit_abs | 0.327 | 0.200 | 0.003 | 0.001 | 0.089 | 0.331 | 0.508 | 0.446 | 0.238 |
| apfit_year | 0.358 | 0.202 | 0.002 | 0.001 | 0.160 | 0.331 | 0.508 | 0.446 | 0.251 |
| an5 | 0.216 | 0.178 | 0.000 | 0.000 | 0.146 | 0.272 | 0.420 | 0.421 | 0.207 |
| an5_off | 0.237 | 0.227 | 0.000 | 0.000 | 0.057 | 0.255 | 0.497 | 0.524 | 0.225 |
| persistence | 0.823 | 1.174 | 0.000 | 0.000 | 0.080 | 1.094 | 0.665 | 0.570 | 0.551 |

## Attribution (difference of the metric)

**exposure**

| Variable | Δη² | ΔR²_abs | ΔR²_year |
|---|---:|---:|---:|
| food_stock | -0.041 | 0.041 | 0.022 |
| wood_stock | 0.002 | -0.001 | -0.002 |
| fire_lit | 0.001 | -0.000 | -0.001 |
| cold_members | 0.018 | 0.002 | -0.006 |
| alive | 0.010 | -0.011 | 0.008 |
| deadwood_total | 0.000 | 0.000 | -0.000 |
| food_on_bushes_total | 0.030 | -0.032 | -0.038 |
| adults_working | 0.023 | -0.024 | -0.024 |

**variety**

| Variable | Δη² | ΔR²_abs | ΔR²_year |
|---|---:|---:|---:|
| food_stock | 0.388 | -0.411 | -0.407 |
| wood_stock | 0.007 | -0.007 | -0.008 |
| fire_lit | -0.002 | 0.002 | 0.002 |
| cold_members | -0.021 | -0.007 | -0.003 |
| alive | 0.947 | -1.010 | -0.226 |
| deadwood_total | 0.138 | -0.141 | -0.141 |
| food_on_bushes_total | 0.349 | -0.371 | -0.366 |
| adults_working | 0.157 | -0.179 | -0.177 |

## Deaths (mean per episode by day 120, all 112 seeds)

| Scene | Starvation | Dehydration | Old age | Exposure | All |
|---|---:|---:|---:|---:|---:|
| game_v15 | 0.00 | 0.00 | 72.87 | 0.00 | 72.87 |
| game_v16_neutral | 0.00 | 0.00 | 70.74 | 2.00 | 72.74 |
| game_v16 | 0.00 | 0.00 | 64.67 | 2.32 | 66.99 |
