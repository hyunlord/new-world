# Pass 12 Part 0: UBS on Pass 11 pilot (read-only)

Root `/Users/rexxa/work/worldsim-wm-data/p11/pilot`, seeds 22000..22112 (all episodes); dyn-eta2 (Pass 11, continuity) on the test split 22096..22112. `diversity_p12.py`; lambda by LOO MSE from [0.1, 1.0, 10.0, 100.0, 1000.0].

## Mean over days 60/90/118: R2_static / UBS (dyn-eta2)

| Variable | game_v16 | game_v17_nofission | game_v17 |
|---|---|---|---|
| food_stock | 0.547 / 0.299 (0.298) | 0.639 / 0.267 (0.434) | 0.275 / 0.516 (0.694) |
| wood_stock | 0.073 / 0.084 (0.069) | 0.081 / 0.081 (0.069) | 0.078 / 0.207 (0.276) |
| alive | 0.907 / 0.309 (0.998) | 0.901 / 0.319 (0.998) | 0.371 / 0.881 (0.933) |
| deadwood_total | 0.959 / 0.103 (0.097) | 0.994 / 0.053 (0.116) | 0.987 / 0.076 (0.124) |
| food_on_bushes_total | 0.379 / 0.221 (0.442) | 0.967 / 0.163 (0.467) | 0.696 / 0.434 (0.576) |
| adults_working | 0.431 / 0.176 (0.475) | 0.369 / 0.196 (0.493) | 0.308 / 0.244 (0.491) |

## Per day: R2_static / UBS

### game_v16 (n 112, features 11)

| Variable | day 30 | day 60 | day 90 | day 118 |
|---|---|---|---|---|
| food_stock | 0.519 / 0.302 | 0.475 / 0.314 | 0.593 / 0.288 | 0.573 / 0.294 |
| wood_stock | 0.171 / 0.090 | -0.002 / 0.080 | 0.237 / 0.087 | -0.016 / 0.083 |
| alive | 0.901 / 0.322 | 0.904 / 0.315 | 0.909 / 0.308 | 0.908 / 0.305 |
| deadwood_total | 0.851 / 0.194 | 0.956 / 0.106 | 0.953 / 0.110 | 0.968 / 0.092 |
| food_on_bushes_total | 0.169 / 0.178 | 0.409 / 0.223 | 0.290 / 0.206 | 0.437 / 0.234 |
| adults_working | 0.535 / 0.180 | 0.511 / 0.153 | 0.401 / 0.207 | 0.381 / 0.168 |

### game_v17_nofission (n 112, features 14)

| Variable | day 30 | day 60 | day 90 | day 118 |
|---|---|---|---|---|
| food_stock | 0.616 / 0.295 | 0.593 / 0.261 | 0.714 / 0.264 | 0.610 / 0.276 |
| wood_stock | 0.174 / 0.089 | -0.025 / 0.086 | 0.124 / 0.080 | 0.145 / 0.076 |
| alive | 0.893 / 0.335 | 0.895 / 0.329 | 0.904 / 0.317 | 0.903 / 0.312 |
| deadwood_total | 0.980 / 0.091 | 0.993 / 0.057 | 0.995 / 0.049 | 0.993 / 0.054 |
| food_on_bushes_total | 0.960 / 0.172 | 0.962 / 0.173 | 0.967 / 0.164 | 0.971 / 0.152 |
| adults_working | 0.514 / 0.197 | 0.410 / 0.182 | 0.457 / 0.192 | 0.238 / 0.215 |

### game_v17 (n 112, features 14)

| Variable | day 30 | day 60 | day 90 | day 118 |
|---|---|---|---|---|
| food_stock | 0.567 / 0.202 | 0.242 / 0.477 | 0.313 / 0.455 | 0.271 / 0.618 |
| wood_stock | 0.141 / 0.126 | -0.010 / 0.176 | 0.165 / 0.181 | 0.080 / 0.266 |
| alive | 0.876 / 0.163 | 0.409 / 0.708 | 0.375 / 0.863 | 0.328 / 1.070 |
| deadwood_total | 0.964 / 0.123 | 0.988 / 0.073 | 0.986 / 0.080 | 0.987 / 0.075 |
| food_on_bushes_total | 0.916 / 0.240 | 0.806 / 0.358 | 0.736 / 0.406 | 0.545 / 0.538 |
| adults_working | 0.455 / 0.183 | 0.288 / 0.230 | 0.356 / 0.224 | 0.281 / 0.279 |

## game_v17 by whether the world split (ridge refit per subset)

| Subset | n | food_stock | wood_stock | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---|---|---|---|---|---|---|
| split | 71 | 0.190 / 0.515 | 0.048 / 0.166 | 0.022 / 0.980 | 0.988 / 0.074 | 0.646 / 0.388 | -0.015 / 0.248 |
| no_split | 41 | 0.708 / 0.275 | 0.017 / 0.083 | 0.953 / 0.222 | 0.991 / 0.059 | 0.963 / 0.173 | 0.430 / 0.212 |

## Paired bootstrap game_v17_nofission - game_v16 (delta mean UBS, 90 % interval, 1000 resamples)

- food_stock: delta -0.032, [-0.065, 0.006]
- wood_stock: delta -0.003, [-0.010, 0.003]
- alive: delta 0.010, [0.001, 0.028] (above 0)
- deadwood_total: delta -0.049, [-0.059, -0.042]
- food_on_bushes_total: delta -0.058, [-0.085, -0.022]
- adults_working: delta 0.020, [0.004, 0.044] (above 0)

## Paired bootstrap game_v17 - game_v17_nofission (delta mean UBS, 90 % interval, 1000 resamples)

- food_stock: delta 0.249, [0.212, 0.319] (above 0)
- wood_stock: delta 0.127, [0.112, 0.149] (above 0)
- alive: delta 0.561, [0.483, 0.683] (above 0)
- deadwood_total: delta 0.023, [0.016, 0.032] (above 0)
- food_on_bushes_total: delta 0.271, [0.224, 0.348] (above 0)
- adults_working: delta 0.048, [0.023, 0.078] (above 0)

