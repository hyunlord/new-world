# Pass 12 Part 0: UBS on Pass 10 pilot (read-only)

Root `/Users/rexxa/work/worldsim-wm-data/p10/pilot`, seeds 20000..20112 (all episodes); dyn-eta2 (Pass 11, continuity) on the test split 20096..20112. `diversity_p12.py`; lambda by LOO MSE from [0.1, 1.0, 10.0, 100.0, 1000.0].

## Mean over days 60/90/118: R2_static / UBS (dyn-eta2)

| Variable | game_v15 | game_v16 |
|---|---|---|
| food_stock | 0.568 / 0.246 (0.130) | 0.688 / 0.299 (0.320) |
| wood_stock | 0.014 / 0.088 (0.057) | 0.085 / 0.088 (0.065) |
| alive | -0.003 / 0.171 (0.790) | 0.945 / 0.240 (0.999) |
| deadwood_total | -0.016 / 0.100 (0.030) | 0.960 / 0.105 (0.087) |
| food_on_bushes_total | 0.558 / 0.073 (0.133) | 0.372 / 0.250 (0.492) |
| adults_working | 0.402 / 0.120 (0.299) | 0.456 / 0.182 (0.490) |

## Per day: R2_static / UBS

### game_v15 (n 112, features 1)

| Variable | day 30 | day 60 | day 90 | day 118 |
|---|---|---|---|---|
| food_stock | 0.492 / 0.229 | 0.516 / 0.241 | 0.621 / 0.240 | 0.566 / 0.256 |
| wood_stock | 0.037 / 0.086 | -0.004 / 0.090 | 0.000 / 0.087 | 0.046 / 0.086 |
| alive | -0.020 / 0.065 | -0.018 / 0.175 | 0.028 / 0.220 | -0.019 / 0.118 |
| deadwood_total | -0.019 / 0.185 | -0.020 / 0.085 | -0.010 / 0.110 | -0.019 / 0.106 |
| food_on_bushes_total | 0.630 / 0.060 | 0.617 / 0.072 | 0.482 / 0.081 | 0.576 / 0.066 |
| adults_working | 0.452 / 0.114 | 0.430 / 0.109 | 0.352 / 0.140 | 0.423 / 0.110 |

### game_v16 (n 112, features 11)

| Variable | day 30 | day 60 | day 90 | day 118 |
|---|---|---|---|---|
| food_stock | 0.677 / 0.285 | 0.681 / 0.292 | 0.696 / 0.301 | 0.688 / 0.304 |
| wood_stock | 0.112 / 0.117 | -0.027 / 0.081 | 0.248 / 0.105 | 0.035 / 0.077 |
| alive | 0.937 / 0.259 | 0.942 / 0.246 | 0.946 / 0.239 | 0.947 / 0.234 |
| deadwood_total | 0.861 / 0.186 | 0.962 / 0.100 | 0.963 / 0.101 | 0.955 / 0.113 |
| food_on_bushes_total | 0.176 / 0.194 | 0.369 / 0.248 | 0.333 / 0.250 | 0.415 / 0.253 |
| adults_working | 0.634 / 0.171 | 0.511 / 0.162 | 0.501 / 0.197 | 0.356 / 0.186 |

## Paired bootstrap game_v16 - game_v15 (delta mean UBS, 90 % interval, 1000 resamples)

- food_stock: delta 0.053, [0.022, 0.112] (above 0)
- wood_stock: delta 0.000, [-0.010, 0.013]
- alive: delta 0.068, [0.026, 0.143] (above 0)
- deadwood_total: delta 0.005, [0.000, 0.019] (above 0)
- food_on_bushes_total: delta 0.177, [0.145, 0.232] (above 0)
- adults_working: delta 0.062, [0.053, 0.084] (above 0)

