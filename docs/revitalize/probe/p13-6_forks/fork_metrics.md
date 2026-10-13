# Fork metrics (game_v19, seeds 12000–12031, R = 4, fork tick 28800)

## G0a — placebo identity

- amount-0 placebos identical to the factual of the same replicate: 224 / 224
- replicate 0 identical to the unforked run: 224 / 224
- **PASS**

## G0b — variance reduction from pairing

VRF = V_unpaired / V_paired, variances over replicates within a seed, averaged over seeds; 95 % interval from a cluster bootstrap over seeds. The unpaired arm still shares worldgen, weather and the whole prefix up to the fork, so VRF measures the value of pairing the post-fork agent noise only.

Noisy seeds: seeds whose 4 replicates do not all give the same value, in the factual (f), the variant (v) and both; where the factual is constant within a seed, the paired and unpaired differences have the same variance and that seed contributes VRF 1. VRF (both noisy) restricts V_p and V_u to the seeds noisy in both arms (reported only). Bootstrap resamples with no agent noise at all have no VRF and are left out of the interval; their count is shown in brackets after it.

| variant | target | VRF | 95 % CI | V_paired | V_unpaired | noisy seeds f / v / both | VRF (both noisy) | mean effect | sign stability | median std | median abs seed mean | G0b |
|---|---|---:|---|---:|---:|---|---:|---:|---:|---:|---:|---|
| gift_wood | dark_nights | 1 | [1, 1] (22 quiet) | 0.0495 | 0.0495 | 3 / 1 / 0 | – | -0.68 | 0.969 | 0 | 1 | fail |
| gift_wood | cold_member_hours | 1.26 | [1, 1.81] | 159 | 199 | 10 / 5 / 4 | 1.81 | -209 | 0.969 | 0 | 245 | fail |
| gift_wood | food_store | 1.58 | [0.998, 2.39] | 50.2 | 79.5 | 32 / 32 / 32 | 1.58 | 9.5 | 0.594 | 5.41 | 9.25 | reported |
| gift_wood | deaths | 0.547 | [0.314, 1] (24 quiet) | 0.138 | 0.0755 | 3 / 2 / 1 | 0.314 | -0.367 | 0.938 | 0 | 0 | reported |
| gift_wood | births | 1 | [1, 1] (262 quiet) | 0.0156 | 0.0156 | 1 / 1 / 0 | – | -0.156 | 0.938 | 0 | 0 | reported |
| rot_wood | dark_nights | 0.778 | [0.448, 1] (1 quiet) | 0.0938 | 0.0729 | 3 / 5 / 2 | 0.636 | 1.12 | 0.969 | 0 | 1 | fail |
| rot_wood | cold_member_hours | 1.62 | [1.15, 3.38] | 353 | 572 | 10 / 19 / 10 | 3.18 | 324 | 0.938 | 0.577 | 309 | pass |
| rot_wood | food_store | 1.75 | [1.09, 2.77] | 48.3 | 84.3 | 32 / 32 / 32 | 1.75 | -16.8 | 0.875 | 5.22 | 15.8 | reported |
| rot_wood | deaths | 0.698 | [0.484, 1.8] (1 quiet) | 0.276 | 0.193 | 3 / 7 / 3 | 0.628 | 0.844 | 0.969 | 0 | 0 | reported |
| rot_wood | births | 1.67 | [1, 2.52] (2 quiet) | 0.0625 | 0.104 | 1 / 6 / 1 | 3 | 0.25 | 0.875 | 0 | 0 | reported |
| rain | dark_nights | 1 | [1, 1] (1 quiet) | 0.0938 | 0.0938 | 3 / 5 / 1 | 1 | 1.69 | 0.969 | 0 | 2 | fail |
| rain | cold_member_hours | 0.78 | [0.545, 1.14] | 303 | 237 | 10 / 18 / 6 | 0.557 | 510 | 1 | 14.4 | 509 | fail |
| rain | food_store | 2.57 | [1.19, 5.65] | 32.3 | 82.8 | 32 / 32 / 32 | 2.57 | -22.9 | 0.906 | 3.91 | 24.4 | reported |
| rain | deaths | 1.05 | [0.82, 1.55] | 0.411 | 0.432 | 3 / 10 / 3 | 1.08 | 1.02 | 1 | 0 | 0 | reported |
| rain | births | 1.76 | [1, 2.45] (28 quiet) | 0.0547 | 0.0964 | 1 / 4 / 1 | 2.45 | 0.258 | 0.938 | 0 | 0 | reported |
| cold_snap | dark_nights | 1.64 | [1, 2.68] (5 quiet) | 0.0651 | 0.107 | 3 / 4 / 2 | 2.07 | 0.398 | 0.875 | 0 | 0 | fail |
| cold_snap | cold_member_hours | 2.18 | [1.32, 4.39] | 173 | 379 | 10 / 13 / 8 | 3.51 | 129 | 0.969 | 0 | 93.4 | pass |
| cold_snap | food_store | 13.6 | [6.54, 26.5] | 8.58 | 117 | 32 / 32 / 32 | 13.6 | -6.64 | 0.719 | 2.07 | 5.75 | reported |
| cold_snap | deaths | 5.36 | [1, 25] (2 quiet) | 0.0573 | 0.307 | 3 / 6 / 3 | 25 | 0.172 | 0.906 | 0 | 0 | reported |
| cold_snap | births | 1 | [1, 1] (729 quiet) | 0.00781 | 0.00781 | 1 / 0 / 0 | – | 0.0547 | 0.969 | 0 | 0 | reported |
| windfall | dark_nights | 0.704 | [0.273, 1] (24 quiet) | 0.0703 | 0.0495 | 3 / 2 / 1 | 0.273 | -0.289 | 0.875 | 0 | 0 | fail |
| windfall | cold_member_hours | 0.945 | [0.582, 1.39] | 244 | 231 | 10 / 7 / 4 | 0.859 | -80 | 0.906 | 0 | 36.2 | fail |
| windfall | food_store | 2.46 | [1.59, 3.76] | 31.4 | 77.1 | 32 / 32 / 32 | 2.46 | 4.02 | 0.406 | 4.86 | 4.62 | reported |
| windfall | deaths | 1.31 | [1, 1.42] (78 quiet) | 0.0677 | 0.0885 | 3 / 2 / 2 | 1.35 | -0.125 | 0.938 | 0 | 0 | reported |
| windfall | births | 1 | [1, 1] (729 quiet) | 0.00781 | 0.00781 | 1 / 0 / 0 | – | -0.0703 | 0.969 | 0 | 0 | reported |
| gift1 | dark_nights | 6.33 | [1, ∞] (80 quiet) | 0.0156 | 0.099 | 3 / 2 / 2 | 11.7 | 0 | 0.938 | 0 | 0 | fail |
| gift1 | cold_member_hours | 464 | [116, ∞] | 0.799 | 371 | 10 / 10 / 10 | 464 | 0.0234 | 0.969 | 0 | 0 | pass |
| gift1 | food_store | 3.75 | [1.99, 8.48] | 28 | 105 | 32 / 32 / 32 | 3.75 | -0.273 | 0.0625 | 3.13 | 0.75 | reported |
| gift1 | deaths | 7.55 | [1, ∞] (78 quiet) | 0.0286 | 0.216 | 3 / 2 / 2 | 10 | 0.0234 | 0.938 | 0 | 0 | reported |
| gift1 | births | ∞ | [∞, ∞] (729 quiet) | 0 | 0.0208 | 1 / 1 / 1 | ∞ | 0 | 1 | 0 | 0 | reported |

G0b (dark nights and cold member-hours of every C_cf variant): **FAIL**; excluded as no agent noise: none.

## Divergence horizon (replicate 0, medians over seeds)

| variant | intervention tick | first agent moved (ticks after) | moved share after 1 h / 6 h / 24 h / 72 h | first diff: dark nights | cold member-hours | food store | deaths | births |
|---|---:|---|---|---|---|---|---|---|
| gift_wood | 29400 | 2.46e+03 (never 0) | 0 / 0 / 0 / 0.386 | 7.62e+03 (never 12) | 7.5e+03 (never 13) | 4.08e+03 (never 0) | 9.36e+03 (never 23) | 1.09e+04 (never 21) |
| rot_wood | 29400 | 1.11e+03 (never 0) | 0 / 0 / 0.0644 / 0.334 | 6.42e+03 (never 3) | 6.33e+03 (never 4) | 2.94e+03 (never 0) | 9.36e+03 (never 16) | 9.96e+03 (never 15) |
| rain | 33720 | 1.08e+03 (never 0) | 0 / 0 / 0.479 / 0.633 | 60 (never 0) | 300 (never 0) | 480 (never 0) | 3.54e+03 (never 20) | 5.82e+03 (never 16) |
| cold_snap | 35160 | 2.88e+03 (never 2) | 0 / 0 / 0 / 0.492 | 1.98e+03 (never 7) | 1.98e+03 (never 9) | 2.55e+03 (never 2) | 4.26e+03 (never 28) | 8.34e+03 (never 24) |
| windfall | 31920 | 60 (never 0) | 0.153 / 0.279 / 0.164 / 0.478 | 5.1e+03 (never 12) | 5.1e+03 (never 14) | 1.68e+03 (never 0) | 8.16e+03 (never 25) | 8.34e+03 (never 24) |
| gift1 | 29400 | 5.28e+03 (never 3) | 0 / 0 / 0 / 0 | 8.85e+03 (never 28) | 1.05e+04 (never 30) | 60 (never 0) | 1.06e+04 (never 31) | 1.65e+04 (never 29) |

## Deaths effect (replicate mean per seed, exposure deaths included)

Deaths up to the day-29 06:00 row, variant minus factual of the same replicate, averaged over the replicates of each seed.

| variant | seeds with effect > 0 | seeds with effect < 0 | mean effect over seeds | dark-night effect differs from v15 (seeds) |
|---|---:|---:|---:|---|
| gift_wood | 0.000 | 0.281 | -0.367 | – |
| rot_wood | 0.438 | 0.000 | 0.844 | – |
| rain | 0.375 | 0.000 | 1.016 | – |
| cold_snap | 0.156 | 0.000 | 0.172 | – |
| windfall | 0.031 | 0.125 | -0.125 | – |
| gift1 | 0.031 | 0.031 | 0.023 | – |

## Deaths effect by life stage (Pass 11; all causes, replicate mean, mean over seeds)

| variant | adult | child | elder | seeds with adult effect > 0 |
|---|---:|---:|---:|---:|
| gift_wood | -0.078 | -0.289 | 0.000 | 0.000 |
| rot_wood | 0.516 | 0.289 | 0.039 | 0.281 |
| rain | 0.734 | 0.227 | 0.055 | 0.344 |
| cold_snap | 0.125 | 0.039 | 0.008 | 0.094 |
| windfall | -0.039 | -0.086 | 0.000 | 0.031 |
| gift1 | 0.016 | 0.000 | 0.008 | 0.031 |

## Newborn lineage keys (Y6, reported only)

Newborns born at or after the intervention in either arm (replicate 0), and how many carry a key present among the other arm's newborns.

| variant | newborns (either arm) | equal key in both | share |
|---|---:|---:|---:|
| gift_wood | 415 | 377 | 0.908 |
| rot_wood | 449 | 392 | 0.873 |
| rain | 396 | 321 | 0.811 |
| cold_snap | 332 | 311 | 0.937 |
| windfall | 375 | 362 | 0.965 |
| gift1 | 407 | 405 | 0.995 |
