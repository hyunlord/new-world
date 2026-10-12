# Pass 9 fork metrics (game_v15, seeds 12000–12031, R = 8, fork tick 28800)

## G0a — placebo identity

- amount-0 placebos identical to the factual of the same replicate: 448 / 448
- replicate 0 identical to the unforked run: 224 / 224
- **PASS**

## G0b — variance reduction from pairing

VRF = V_unpaired / V_paired, variances over replicates within a seed, averaged over seeds; 95 % interval from a cluster bootstrap over seeds. The unpaired arm still shares worldgen, weather and the whole prefix up to the fork, so VRF measures the value of pairing the post-fork agent noise only.

Noisy seeds: seeds whose 8 replicates do not all give the same value, in the factual (f), the variant (v) and both; where the factual is constant within a seed, the paired and unpaired differences have the same variance and that seed contributes VRF 1. VRF (both noisy) restricts V_p and V_u to the seeds noisy in both arms (reported only). Bootstrap resamples with no agent noise at all have no VRF and are left out of the interval; their count is shown in brackets after it.

| variant | target | VRF | 95 % CI | V_paired | V_unpaired | noisy seeds f / v / both | VRF (both noisy) | mean effect | sign stability | median std | median abs seed mean | G0b |
|---|---|---:|---|---:|---:|---|---:|---:|---:|---:|---:|---|
| gift_wood | dark_nights | 1 | [1, 1] (30 quiet) | 0.0246 | 0.0246 | 3 / 2 / 1 | 1 | -0.82 | 0.938 | 0 | 1 | fail |
| gift_wood | cold_member_hours | 1.54 | [0.943, 3.07] | 348 | 535 | 12 / 6 / 5 | 2.78 | -282 | 1 | 0 | 336 | fail |
| gift_wood | food_store | 5.89 | [3.37, 12.7] | 13.1 | 77.2 | 32 / 32 / 32 | 5.89 | 13.2 | 0.781 | 2.29 | 14.5 | reported |
| gift_wood | deaths | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | 0 | 1 | 0 | 0 | reported |
| gift_wood | births | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | 0 | 1 | 0 | 0 | reported |
| rot_wood | dark_nights | 1.19 | [1, 1.7] | 0.048 | 0.0569 | 3 / 7 / 1 | 3.29 | 1.35 | 0.875 | 0 | 1 | fail |
| rot_wood | cold_member_hours | 1.81 | [1.16, 2.38] | 677 | 1.23e+03 | 12 / 16 / 9 | 2.34 | 434 | 1 | 14.7 | 423 | pass |
| rot_wood | food_store | 3.55 | [2.09, 7.23] | 21.3 | 75.6 | 32 / 32 / 32 | 3.55 | -19.3 | 0.875 | 3.27 | 20.6 | reported |
| rot_wood | deaths | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | 0 | 1 | 0 | 0 | reported |
| rot_wood | births | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | 0 | 1 | 0 | 0 | reported |
| rain | dark_nights | 1 | [1, 1] (96 quiet) | 0.0162 | 0.0162 | 3 / 1 / 1 | 1 | 0.23 | 0.938 | 0 | 1 | fail |
| rain | cold_member_hours | 1.69 | [1.09, 3.51] | 295 | 499 | 12 / 6 / 5 | 5.8 | 64.8 | 0.875 | 0 | 180 | pass |
| rain | food_store | 16.5 | [9.34, 30.9] | 4.69 | 77.3 | 32 / 32 / 32 | 16.5 | -2 | 0.906 | 1.32 | 10.4 | reported |
| rain | deaths | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | 0 | 1 | 0 | 0 | reported |
| rain | births | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | 0 | 1 | 0 | 0 | reported |
| cold_snap | dark_nights | 1 | [1, 1] (10 quiet) | 0.0273 | 0.0273 | 3 / 2 / 0 | – | 0.48 | 0.844 | 0 | 0.375 | fail |
| cold_snap | cold_member_hours | 1.63 | [1.14, 2.36] | 351 | 571 | 12 / 12 / 6 | 3.62 | 155 | 0.844 | 0 | 168 | pass |
| cold_snap | food_store | 24.9 | [12.3, 55.3] | 3.05 | 76 | 32 / 32 / 32 | 24.9 | -6.89 | 0.781 | 0.707 | 6.69 | reported |
| cold_snap | deaths | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | 0 | 1 | 0 | 0 | reported |
| cold_snap | births | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | 0 | 1 | 0 | 0 | reported |
| windfall | dark_nights | 1 | [1, 1] (3 quiet) | 0.0363 | 0.0363 | 3 / 3 / 0 | – | -0.309 | 0.875 | 0 | 0 | fail |
| windfall | cold_member_hours | 2.17 | [1.4, 3.36] | 242 | 525 | 12 / 8 / 8 | 2.57 | -125 | 1 | 0 | 126 | pass |
| windfall | food_store | 3.86 | [2.13, 8.54] | 16.8 | 65 | 32 / 32 / 32 | 3.86 | 6.88 | 0.5 | 2.61 | 7 | reported |
| windfall | deaths | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | 0 | 1 | 0 | 0 | reported |
| windfall | births | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | 0 | 1 | 0 | 0 | reported |
| gift1 | dark_nights | ∞ | [∞, ∞] (96 quiet) | 0 | 0.0357 | 3 / 3 / 3 | ∞ | 0 | 1 | 0 | 0 | pass |
| gift1 | cold_member_hours | 23.3 | [11.6, ∞] | 28.7 | 671 | 12 / 11 / 11 | 30.4 | 0.332 | 0.938 | 0 | 0 | pass |
| gift1 | food_store | 5.65 | [2.68, 14.3] | 12.1 | 68.5 | 32 / 32 / 32 | 5.65 | 0.34 | 0.219 | 2.13 | 0.875 | reported |
| gift1 | deaths | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | 0 | 1 | 0 | 0 | reported |
| gift1 | births | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | 0 | 1 | 0 | 0 | reported |

G0b (dark nights and cold member-hours of every C_cf variant): **FAIL**; excluded as no agent noise: none.

## v14 single forks against v15

| variant | target | mean over seeds of abs(d_v14 − mean_k d_v15) | v15 median within-seed std |
|---|---|---:|---:|
| gift_wood | dark_nights | 0.25 | 0 |
| gift_wood | cold_member_hours | 61.7 | 0 |
| gift_wood | food_store | 5.87 | 2.29 |
| gift_wood | deaths | 0 | 0 |
| rot_wood | dark_nights | 0.32 | 0 |
| rot_wood | cold_member_hours | 100 | 14.7 |
| rot_wood | food_store | 7.2 | 3.27 |
| rot_wood | deaths | 0 | 0 |
| rain | dark_nights | 0.293 | 0 |
| rain | cold_member_hours | 74.3 | 0 |
| rain | food_store | 5.04 | 1.32 |
| rain | deaths | 0 | 0 |
| cold_snap | dark_nights | 0.309 | 0 |
| cold_snap | cold_member_hours | 73.7 | 0 |
| cold_snap | food_store | 3.81 | 0.707 |
| cold_snap | deaths | 0 | 0 |
| windfall | dark_nights | 0.426 | 0 |
| windfall | cold_member_hours | 57.9 | 0 |
| windfall | food_store | 4.12 | 2.61 |
| windfall | deaths | 0 | 0 |

## Divergence horizon (replicate 0, medians over seeds)

| variant | intervention tick | first agent moved (ticks after) | moved share after 1 h / 6 h / 24 h / 72 h | first diff: dark nights | cold member-hours | food store | deaths | births |
|---|---:|---|---|---|---|---|---|---|
| gift_wood | 29400 | 2.46e+03 (never 0) | 0 / 0 / 0 / 0.548 | 7.83e+03 (never 8) | 7.83e+03 (never 8) | 3.96e+03 (never 0) | – (never 32) | 1.54e+04 (never 30) |
| rot_wood | 29400 | 1.05e+03 (never 0) | 0 / 0 / 0.107 / 0.417 | 6.36e+03 (never 0) | 6.36e+03 (never 0) | 2.85e+03 (never 0) | – (never 32) | 1.64e+04 (never 25) |
| rain | 33720 | 1.02e+03 (never 0) | 0 / 0 / 0.619 / 0.663 | 60 (never 0) | 300 (never 0) | 480 (never 0) | – (never 32) | 1.21e+04 (never 27) |
| cold_snap | 35160 | 2.55e+03 (never 0) | 0 / 0 / 0 / 0.643 | 2.04e+03 (never 2) | 1.98e+03 (never 3) | 1.89e+03 (never 0) | – (never 32) | 8.76e+03 (never 28) |
| windfall | 31920 | 60 (never 0) | 0.405 / 0.41 / 0.167 / 0.631 | 5.31e+03 (never 8) | 5.31e+03 (never 10) | 420 (never 0) | – (never 32) | 1.39e+04 (never 27) |
| gift1 | 29400 | 4.44e+03 (never 3) | 0 / 0 / 0 / 0 | 1.01e+04 (never 29) | – (never 32) | 60 (never 0) | – (never 32) | 1.64e+04 (never 27) |

## Newborn lineage keys (Y6, reported only)

Newborns born at or after the intervention in either arm (replicate 0), and how many carry a key present among the other arm's newborns.

| variant | newborns (either arm) | equal key in both | share |
|---|---:|---:|---:|
| gift_wood | 337 | 335 | 0.994 |
| rot_wood | 336 | 334 | 0.994 |
| rain | 273 | 273 | 1 |
| cold_snap | 252 | 248 | 0.984 |
| windfall | 307 | 307 | 1 |
| gift1 | 336 | 334 | 0.994 |
