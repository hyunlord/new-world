# Fork metrics (game_v16, seeds 12000–12031, R = 4, fork tick 28800)

## G0a — placebo identity

- amount-0 placebos identical to the factual of the same replicate: 224 / 224
- replicate 0 identical to the unforked run: 224 / 224
- **PASS**

## G0b — variance reduction from pairing

VRF = V_unpaired / V_paired, variances over replicates within a seed, averaged over seeds; 95 % interval from a cluster bootstrap over seeds. The unpaired arm still shares worldgen, weather and the whole prefix up to the fork, so VRF measures the value of pairing the post-fork agent noise only.

Noisy seeds: seeds whose 4 replicates do not all give the same value, in the factual (f), the variant (v) and both; where the factual is constant within a seed, the paired and unpaired differences have the same variance and that seed contributes VRF 1. VRF (both noisy) restricts V_p and V_u to the seeds noisy in both arms (reported only). Bootstrap resamples with no agent noise at all have no VRF and are left out of the interval; their count is shown in brackets after it.

| variant | target | VRF | 95 % CI | V_paired | V_unpaired | noisy seeds f / v / both | VRF (both noisy) | mean effect | sign stability | median std | median abs seed mean | G0b |
|---|---|---:|---|---:|---:|---|---:|---:|---:|---:|---:|---|
| gift_wood | dark_nights | 1 | [1, 1] (251 quiet) | 0.0156 | 0.0156 | 1 / 1 / 0 | – | -0.734 | 0.969 | 0 | 1 | fail |
| gift_wood | cold_member_hours | 2.43 | [1.48, 4.68] | 223 | 540 | 17 / 10 / 10 | 4.77 | -211 | 0.969 | 0 | 211 | pass |
| gift_wood | food_store | 2.28 | [1.21, 4.32] | 25.9 | 59.1 | 32 / 32 / 32 | 2.28 | 12.7 | 0.625 | 3.53 | 8.88 | reported |
| gift_wood | deaths | 1 | [1, 1] (29 quiet) | 0.0573 | 0.0573 | 2 / 2 / 0 | – | -0.625 | 0.969 | 0 | 0 | reported |
| gift_wood | births | 1 | [1, 1] (78 quiet) | 0.0495 | 0.0495 | 1 / 2 / 0 | – | -0.414 | 0.969 | 0 | 0 | reported |
| rot_wood | dark_nights | 1 | [1, 1] (83 quiet) | 0.0234 | 0.0234 | 1 / 2 / 0 | – | 1.12 | 1 | 0 | 1 | fail |
| rot_wood | cold_member_hours | 1.47 | [1.1, 2.33] | 376 | 551 | 17 / 24 / 16 | 1.93 | 314 | 1 | 2.33 | 342 | pass |
| rot_wood | food_store | 1.9 | [1.28, 2.89] | 39.2 | 74.6 | 32 / 32 / 32 | 1.9 | -17.9 | 0.844 | 4.34 | 19.5 | reported |
| rot_wood | deaths | 1 | [1, 1] (6 quiet) | 0.117 | 0.117 | 2 / 4 / 0 | – | 1.18 | 0.906 | 0 | 1 | reported |
| rot_wood | births | 1 | [1, 1] (3 quiet) | 0.102 | 0.102 | 1 / 6 / 1 | 1 | 0.57 | 0.875 | 0 | 0 | reported |
| rain | dark_nights | 1 | [1, 1] (267 quiet) | 0.0156 | 0.0156 | 1 / 1 / 0 | – | 0.266 | 0.938 | 0 | 1 | fail |
| rain | cold_member_hours | 2.34 | [1.57, 3.91] | 235 | 548 | 17 / 10 / 10 | 3.04 | 83.7 | 0.938 | 0.25 | 208 | pass |
| rain | food_store | 4.63 | [2.34, 10.4] | 18.4 | 85 | 32 / 32 / 32 | 4.63 | -0.141 | 0.906 | 2.36 | 11 | reported |
| rain | deaths | 1 | [1, 1] (25 quiet) | 0.0677 | 0.0677 | 2 / 2 / 0 | – | -0.594 | 0.969 | 0 | 0 | reported |
| rain | births | 1 | [1, 1] (25 quiet) | 0.0573 | 0.0573 | 1 / 3 / 0 | – | -0.391 | 0.969 | 0 | 0 | reported |
| cold_snap | dark_nights | 1 | [1, 1] (267 quiet) | 0.0156 | 0.0156 | 1 / 1 / 0 | – | 0.328 | 0.969 | 0 | 0 | fail |
| cold_snap | cold_member_hours | 1.5 | [0.997, 3.03] | 349 | 522 | 17 / 17 / 14 | 1.62 | 98.9 | 0.875 | 0.539 | 58.9 | fail |
| cold_snap | food_store | 16.2 | [6.62, 47.8] | 4.73 | 76.4 | 32 / 32 / 32 | 16.2 | -5.47 | 0.781 | 0.979 | 2.75 | reported |
| cold_snap | deaths | ∞ | [∞, ∞] (263 quiet) | 0 | 0.0417 | 2 / 2 / 2 | ∞ | 0.156 | 1 | 0 | 0 | reported |
| cold_snap | births | 11.7 | [1, ∞] (244 quiet) | 0.00781 | 0.0911 | 1 / 2 / 1 | ∞ | 0.117 | 0.969 | 0 | 0 | reported |
| windfall | dark_nights | 1 | [1, 1] (713 quiet) | 0.00781 | 0.00781 | 1 / 0 / 0 | – | -0.367 | 0.969 | 0 | 0 | fail |
| windfall | cold_member_hours | 2.58 | [1.48, 6.47] | 207 | 533 | 17 / 14 / 13 | 2.93 | -96.6 | 0.938 | 0 | 80.1 | pass |
| windfall | food_store | 4.04 | [2.12, 7.19] | 20.5 | 83 | 32 / 32 / 32 | 4.04 | 6.05 | 0.469 | 3.43 | 4.75 | reported |
| windfall | deaths | 3.67 | [1, ∞] (263 quiet) | 0.00781 | 0.0286 | 2 / 1 / 1 | ∞ | -0.164 | 1 | 0 | 0 | reported |
| windfall | births | 1 | [1, 1] (263 quiet) | 0.0417 | 0.0417 | 1 / 1 / 0 | – | -0.188 | 0.969 | 0 | 0 | reported |
| gift1 | dark_nights | 1 | [1, 1] (713 quiet) | 0.00781 | 0.00781 | 1 / 0 / 0 | – | 0.00781 | 0.969 | 0 | 0 | fail |
| gift1 | cold_member_hours | 4.67 | [1.77, ∞] | 131 | 612 | 17 / 17 / 17 | 4.67 | -0.0938 | 0.938 | 0 | 0 | pass |
| gift1 | food_store | 3.12 | [1.59, 7.73] | 23.4 | 73 | 32 / 32 / 32 | 3.12 | 0.359 | 0.156 | 2.22 | 1.25 | reported |
| gift1 | deaths | 3.67 | [1, ∞] (263 quiet) | 0.00781 | 0.0286 | 2 / 1 / 1 | ∞ | -0.00781 | 0.969 | 0 | 0 | reported |
| gift1 | births | 1 | [1, 1] (244 quiet) | 0.0391 | 0.0391 | 1 / 1 / 0 | – | -0.0234 | 0.938 | 0 | 0 | reported |

G0b (dark nights and cold member-hours of every C_cf variant): **FAIL**; excluded as no agent noise: none.

## Divergence horizon (replicate 0, medians over seeds)

| variant | intervention tick | first agent moved (ticks after) | moved share after 1 h / 6 h / 24 h / 72 h | first diff: dark nights | cold member-hours | food store | deaths | births |
|---|---:|---|---|---|---|---|---|---|
| gift_wood | 29400 | 2.46e+03 (never 0) | 0 / 0 / 0 / 0.367 | 7.32e+03 (never 13) | 7.5e+03 (never 13) | 4.14e+03 (never 0) | 9.24e+03 (never 20) | 9.42e+03 (never 18) |
| rot_wood | 29400 | 1.08e+03 (never 0) | 0 / 0 / 0.227 / 0.467 | 6.87e+03 (never 2) | 6.93e+03 (never 2) | 2.52e+03 (never 0) | 7.92e+03 (never 12) | 9.06e+03 (never 12) |
| rain | 33720 | 960 (never 0) | 0 / 0 / 0.566 / 0.6 | 60 (never 0) | 300 (never 0) | 480 (never 0) | 4.92e+03 (never 20) | 5.1e+03 (never 17) |
| cold_snap | 35160 | 2.4e+03 (never 2) | 0 / 0 / 0 / 0.477 | 2.04e+03 (never 4) | 2.16e+03 (never 7) | 1.98e+03 (never 2) | 3.54e+03 (never 23) | 4.47e+03 (never 26) |
| windfall | 31920 | 60 (never 0) | 0.267 / 0.343 / 0.178 / 0.493 | 4.8e+03 (never 13) | 5.01e+03 (never 14) | 1.59e+03 (never 0) | 5.4e+03 (never 25) | 6.9e+03 (never 25) |
| gift1 | 29400 | 4.35e+03 (never 2) | 0 / 0 / 0 / 0 | 8.76e+03 (never 27) | 9.3e+03 (never 31) | 60 (never 0) | – (never 32) | 1.49e+04 (never 29) |

## Deaths effect (replicate mean per seed, exposure deaths included)

Deaths up to the day-29 06:00 row, variant minus factual of the same replicate, averaged over the replicates of each seed.

| variant | seeds with effect > 0 | seeds with effect < 0 | mean effect over seeds | dark-night effect differs from v15 (seeds) |
|---|---:|---:|---:|---|
| gift_wood | 0.000 | 0.281 | -0.625 | 0.562 (18 / 32; v16 -0.734, v15 -0.820) |
| rot_wood | 0.562 | 0.000 | 1.180 | 0.562 (18 / 32; v16 1.117, v15 1.352) |
| rain | 0.000 | 0.281 | -0.594 | 0.594 (19 / 32; v16 0.266, v15 0.230) |
| cold_snap | 0.062 | 0.000 | 0.156 | 0.562 (18 / 32; v16 0.328, v15 0.480) |
| windfall | 0.000 | 0.125 | -0.164 | 0.531 (17 / 32; v16 -0.367, v15 -0.309) |
| gift1 | 0.000 | 0.031 | -0.008 | 0.031 (1 / 32; v16 0.008, v15 0.000) |

## Newborn lineage keys (Y6, reported only)

Newborns born at or after the intervention in either arm (replicate 0), and how many carry a key present among the other arm's newborns.

| variant | newborns (either arm) | equal key in both | share |
|---|---:|---:|---:|
| gift_wood | 374 | 353 | 0.944 |
| rot_wood | 412 | 373 | 0.905 |
| rain | 315 | 294 | 0.933 |
| cold_snap | 300 | 295 | 0.983 |
| windfall | 347 | 342 | 0.986 |
| gift1 | 374 | 374 | 1 |
