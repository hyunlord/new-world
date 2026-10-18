# Pass 13 pilot UBS on replicate 0 (reported only)

Root `/Users/rexxa/work/worldsim-wm-data/p13/ubs_view/pilot`, seeds 26000..26064 (all episodes); dyn-eta2 (Pass 11, continuity) on the test split None. `diversity_p12.py`; lambda by LOO MSE from [0.1, 1.0, 10.0, 100.0, 1000.0].

## Mean over days 60/90/118: R2_static / UBS (dyn-eta2)

| Variable | game_v16 | game_v17 | game_v18 | game_v19 |
|---|---|---|---|---|
| food_stock | 0.610 / 0.288 (—) | 0.306 / 0.461 (—) | 0.367 / 0.519 (—) | 0.237 / 0.553 (—) |
| wood_stock | 0.022 / 0.085 (—) | 0.112 / 0.193 (—) | 0.093 / 0.257 (—) | 0.095 / 0.237 (—) |
| alive | 0.884 / 0.343 (—) | 0.576 / 0.696 (—) | 0.447 / 0.820 (—) | 0.292 / 0.910 (—) |
| deadwood_total | 0.938 / 0.109 (—) | 0.981 / 0.085 (—) | 0.986 / 0.075 (—) | 0.982 / 0.084 (—) |
| food_on_bushes_total | 0.455 / 0.187 (—) | 0.669 / 0.439 (—) | 0.601 / 0.514 (—) | 0.528 / 0.526 (—) |
| adults_working | 0.382 / 0.180 (—) | 0.179 / 0.272 (—) | 0.228 / 0.253 (—) | 0.092 / 0.301 (—) |

## Per day: R2_static / UBS

### game_v16 (n 64, features 11)

| Variable | day 30 | day 60 | day 90 | day 118 |
|---|---|---|---|---|
| food_stock | 0.707 / 0.219 | 0.536 / 0.303 | 0.694 / 0.270 | 0.599 / 0.291 |
| wood_stock | -0.005 / 0.141 | -0.040 / 0.080 | -0.031 / 0.091 | 0.139 / 0.084 |
| alive | 0.873 / 0.364 | 0.894 / 0.328 | 0.876 / 0.355 | 0.883 / 0.347 |
| deadwood_total | 0.727 / 0.227 | 0.941 / 0.104 | 0.938 / 0.108 | 0.934 / 0.115 |
| food_on_bushes_total | 0.260 / 0.160 | 0.500 / 0.163 | 0.517 / 0.178 | 0.348 / 0.220 |
| adults_working | 0.312 / 0.209 | 0.452 / 0.159 | 0.377 / 0.213 | 0.317 / 0.168 |

### game_v17 (n 64, features 14)

| Variable | day 30 | day 60 | day 90 | day 118 |
|---|---|---|---|---|
| food_stock | 0.609 / 0.180 | 0.300 / 0.424 | 0.320 / 0.409 | 0.298 / 0.550 |
| wood_stock | 0.291 / 0.118 | 0.008 / 0.170 | 0.235 / 0.165 | 0.092 / 0.243 |
| alive | 0.856 / 0.180 | 0.548 / 0.609 | 0.618 / 0.631 | 0.563 / 0.848 |
| deadwood_total | 0.950 / 0.133 | 0.986 / 0.072 | 0.983 / 0.079 | 0.973 / 0.103 |
| food_on_bushes_total | 0.827 / 0.296 | 0.705 / 0.418 | 0.753 / 0.370 | 0.548 / 0.530 |
| adults_working | 0.414 / 0.191 | 0.037 / 0.276 | 0.282 / 0.207 | 0.216 / 0.331 |

### game_v18 (n 64, features 14)

| Variable | day 30 | day 60 | day 90 | day 118 |
|---|---|---|---|---|
| food_stock | 0.470 / 0.179 | 0.359 / 0.424 | 0.391 / 0.526 | 0.352 / 0.608 |
| wood_stock | -0.027 / 0.153 | 0.016 / 0.230 | 0.109 / 0.262 | 0.154 / 0.278 |
| alive | 0.348 / 0.425 | 0.381 / 0.689 | 0.461 / 0.836 | 0.500 / 0.936 |
| deadwood_total | 0.965 / 0.115 | 0.987 / 0.072 | 0.990 / 0.063 | 0.980 / 0.089 |
| food_on_bushes_total | 0.849 / 0.326 | 0.721 / 0.437 | 0.555 / 0.536 | 0.527 / 0.570 |
| adults_working | 0.404 / 0.172 | 0.096 / 0.269 | 0.237 / 0.237 | 0.352 / 0.253 |

### game_v19 (n 64, features 14)

| Variable | day 30 | day 60 | day 90 | day 118 |
|---|---|---|---|---|
| food_stock | 0.700 / 0.124 | 0.108 / 0.434 | 0.320 / 0.574 | 0.284 / 0.651 |
| wood_stock | -0.032 / 0.091 | -0.032 / 0.183 | 0.166 / 0.241 | 0.150 / 0.289 |
| alive | 0.874 / 0.113 | 0.192 / 0.763 | 0.317 / 0.939 | 0.366 / 1.028 |
| deadwood_total | 0.970 / 0.105 | 0.988 / 0.069 | 0.980 / 0.089 | 0.977 / 0.094 |
| food_on_bushes_total | 0.967 / 0.148 | 0.710 / 0.445 | 0.521 / 0.526 | 0.351 / 0.606 |
| adults_working | 0.370 / 0.173 | 0.148 / 0.273 | 0.092 / 0.318 | 0.034 / 0.312 |

## Paired bootstrap game_v19 - game_v17 (delta mean UBS, 90 % interval, 1000 resamples)

- food_stock: delta 0.092, [-0.013, 0.214]
- wood_stock: delta 0.045, [0.016, 0.092] (above 0)
- alive: delta 0.214, [0.002, 0.403] (above 0)
- deadwood_total: delta -0.001, [-0.021, 0.022]
- food_on_bushes_total: delta 0.086, [-0.039, 0.202]
- adults_working: delta 0.029, [-0.026, 0.074]

## Paired bootstrap game_v19 - game_v18 (delta mean UBS, 90 % interval, 1000 resamples)

- food_stock: delta 0.034, [-0.075, 0.152]
- wood_stock: delta -0.019, [-0.065, 0.027]
- alive: delta 0.090, [-0.073, 0.288]
- deadwood_total: delta 0.009, [-0.010, 0.034]
- food_on_bushes_total: delta 0.012, [-0.084, 0.122]
- adults_working: delta 0.048, [-0.004, 0.083]

