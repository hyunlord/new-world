# Pass 12 final: UBS on the pilot (final recorder)

Root `/Users/rexxa/work/worldsim-wm-data/p12/final/pilot`, seeds 24000..24112 (all episodes); dyn-eta2 (Pass 11, continuity) on the test split 24096..24112. `diversity_p12.py`; lambda by LOO MSE from [0.1, 1.0, 10.0, 100.0, 1000.0].

## Mean over days 60/90/118: R2_static / UBS (dyn-eta2)

| Variable | game_v16 | game_v17 | game_v18_nofission | game_v18 |
|---|---|---|---|---|
| food_stock | 0.596 / 0.295 (0.195) | 0.305 / 0.502 (0.502) | 0.633 / 0.271 (0.185) | 0.388 / 0.536 (0.534) |
| wood_stock | 0.049 / 0.084 (0.069) | 0.031 / 0.210 (0.198) | 0.044 / 0.084 (0.069) | 0.130 / 0.249 (0.240) |
| alive | 0.899 / 0.320 (0.998) | 0.442 / 0.801 (0.785) | 0.894 / 0.329 (0.998) | 0.445 / 0.837 (0.756) |
| deadwood_total | 0.959 / 0.107 (0.107) | 0.978 / 0.089 (0.147) | 0.995 / 0.043 (0.132) | 0.988 / 0.069 (0.150) |
| food_on_bushes_total | 0.488 / 0.236 (0.409) | 0.626 / 0.446 (0.424) | 0.979 / 0.130 (0.410) | 0.684 / 0.472 (0.508) |
| adults_working | 0.324 / 0.202 (0.474) | 0.272 / 0.244 (0.439) | 0.331 / 0.204 (0.476) | 0.373 / 0.240 (0.479) |

## Per day: R2_static / UBS

### game_v16 (n 112, features 11)

| Variable | day 30 | day 60 | day 90 | day 118 |
|---|---|---|---|---|
| food_stock | 0.609 / 0.288 | 0.580 / 0.284 | 0.647 / 0.296 | 0.562 / 0.306 |
| wood_stock | 0.301 / 0.093 | -0.008 / 0.081 | 0.063 / 0.092 | 0.091 / 0.079 |
| alive | 0.888 / 0.340 | 0.899 / 0.319 | 0.901 / 0.319 | 0.896 / 0.323 |
| deadwood_total | 0.857 / 0.191 | 0.964 / 0.102 | 0.958 / 0.110 | 0.953 / 0.110 |
| food_on_bushes_total | 0.365 / 0.171 | 0.478 / 0.220 | 0.520 / 0.238 | 0.466 / 0.249 |
| adults_working | 0.533 / 0.184 | 0.466 / 0.172 | 0.377 / 0.233 | 0.128 / 0.201 |

### game_v17 (n 112, features 14)

| Variable | day 30 | day 60 | day 90 | day 118 |
|---|---|---|---|---|
| food_stock | 0.638 / 0.194 | 0.293 / 0.469 | 0.348 / 0.437 | 0.273 / 0.601 |
| wood_stock | 0.074 / 0.136 | -0.015 / 0.178 | 0.073 / 0.190 | 0.036 / 0.263 |
| alive | 0.835 / 0.191 | 0.478 / 0.671 | 0.460 / 0.747 | 0.388 / 0.985 |
| deadwood_total | 0.946 / 0.137 | 0.980 / 0.085 | 0.981 / 0.082 | 0.973 / 0.100 |
| food_on_bushes_total | 0.904 / 0.250 | 0.759 / 0.383 | 0.633 / 0.432 | 0.485 / 0.523 |
| adults_working | 0.395 / 0.176 | 0.438 / 0.218 | 0.209 / 0.220 | 0.168 / 0.294 |

### game_v18_nofission (n 112, features 14)

| Variable | day 30 | day 60 | day 90 | day 118 |
|---|---|---|---|---|
| food_stock | 0.619 / 0.279 | 0.583 / 0.267 | 0.713 / 0.267 | 0.604 / 0.280 |
| wood_stock | 0.282 / 0.091 | 0.016 / 0.082 | 0.143 / 0.090 | -0.026 / 0.080 |
| alive | 0.891 / 0.334 | 0.895 / 0.326 | 0.895 / 0.329 | 0.891 / 0.330 |
| deadwood_total | 0.982 / 0.086 | 0.996 / 0.042 | 0.995 / 0.044 | 0.995 / 0.044 |
| food_on_bushes_total | 0.973 / 0.145 | 0.978 / 0.131 | 0.978 / 0.133 | 0.980 / 0.126 |
| adults_working | 0.586 / 0.167 | 0.423 / 0.191 | 0.304 / 0.235 | 0.266 / 0.186 |

### game_v18 (n 112, features 14)

| Variable | day 30 | day 60 | day 90 | day 118 |
|---|---|---|---|---|
| food_stock | 0.415 / 0.230 | 0.361 / 0.441 | 0.433 / 0.493 | 0.371 / 0.674 |
| wood_stock | 0.061 / 0.149 | 0.021 / 0.202 | 0.161 / 0.250 | 0.208 / 0.294 |
| alive | 0.401 / 0.445 | 0.395 / 0.714 | 0.470 / 0.792 | 0.471 / 1.005 |
| deadwood_total | 0.963 / 0.121 | 0.991 / 0.060 | 0.990 / 0.065 | 0.983 / 0.082 |
| food_on_bushes_total | 0.900 / 0.268 | 0.792 / 0.392 | 0.728 / 0.454 | 0.533 / 0.571 |
| adults_working | 0.509 / 0.171 | 0.395 / 0.227 | 0.384 / 0.228 | 0.340 / 0.265 |

## game_v18 by whether the world split (ridge refit per subset)

| Subset | n | food_stock | wood_stock | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---|---|---|---|---|---|---|
| split | 66 | 0.200 / 0.516 | 0.104 / 0.185 | 0.140 / 0.811 | 0.985 / 0.077 | 0.693 / 0.394 | 0.089 / 0.226 |
| no_split | 46 | 0.647 / 0.268 | 0.105 / 0.081 | 0.878 / 0.362 | 0.994 / 0.046 | 0.984 / 0.115 | 0.228 / 0.226 |

## Paired bootstrap game_v18 - game_v17 (delta mean UBS, 90 % interval, 1000 resamples)

- food_stock: delta 0.034, [-0.063, 0.140]
- wood_stock: delta 0.038, [0.010, 0.076] (above 0)
- alive: delta 0.036, [-0.102, 0.163]
- deadwood_total: delta -0.020, [-0.033, -0.009]
- food_on_bushes_total: delta 0.026, [-0.046, 0.097]
- adults_working: delta -0.004, [-0.036, 0.029]

## Paired bootstrap game_v18_nofission - game_v17 (delta mean UBS, 90 % interval, 1000 resamples)

- food_stock: delta -0.231, [-0.307, -0.182]
- wood_stock: delta -0.126, [-0.144, -0.112]
- alive: delta -0.472, [-0.616, -0.385]
- deadwood_total: delta -0.046, [-0.057, -0.039]
- food_on_bushes_total: delta -0.316, [-0.395, -0.264]
- adults_working: delta -0.040, [-0.069, -0.013]

## Paired bootstrap game_v18 - game_v18_nofission (delta mean UBS, 90 % interval, 1000 resamples)

- food_stock: delta 0.265, [0.187, 0.375] (above 0)
- wood_stock: delta 0.165, [0.141, 0.203] (above 0)
- alive: delta 0.509, [0.432, 0.630] (above 0)
- deadwood_total: delta 0.026, [0.019, 0.036] (above 0)
- food_on_bushes_total: delta 0.342, [0.295, 0.430] (above 0)
- adults_working: delta 0.036, [0.010, 0.065] (above 0)

