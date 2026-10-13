# Fork metrics (game_v17, seeds 12000–12031, R = 4, fork tick 28800)

## G0a — placebo identity

- amount-0 placebos identical to the factual of the same replicate: 224 / 224
- replicate 0 identical to the unforked run: 224 / 224
- **PASS**

## G0b — variance reduction from pairing

VRF = V_unpaired / V_paired, variances over replicates within a seed, averaged over seeds; 95 % interval from a cluster bootstrap over seeds. The unpaired arm still shares worldgen, weather and the whole prefix up to the fork, so VRF measures the value of pairing the post-fork agent noise only.

Noisy seeds: seeds whose 4 replicates do not all give the same value, in the factual (f), the variant (v) and both; where the factual is constant within a seed, the paired and unpaired differences have the same variance and that seed contributes VRF 1. VRF (both noisy) restricts V_p and V_u to the seeds noisy in both arms (reported only). Bootstrap resamples with no agent noise at all have no VRF and are left out of the interval; their count is shown in brackets after it.

| variant | target | VRF | 95 % CI | V_paired | V_unpaired | noisy seeds f / v / both | VRF (both noisy) | mean effect | sign stability | median std | median abs seed mean | G0b |
|---|---|---:|---|---:|---:|---|---:|---:|---:|---:|---:|---|
| gift_wood | dark_nights | 1 | [1, 1] (246 quiet) | 0.0156 | 0.0156 | 1 / 1 / 0 | – | -0.797 | 0.969 | 0 | 1 | fail |
| gift_wood | cold_member_hours | 2.49 | [1.63, 3.36] | 238 | 594 | 14 / 9 / 9 | 3.03 | -225 | 0.969 | 0 | 250 | pass |
| gift_wood | food_store | 2.79 | [1.63, 4.68] | 77.6 | 217 | 32 / 32 / 32 | 2.79 | 10.6 | 0.5 | 7.99 | 9.12 | reported |
| gift_wood | deaths | 1.09 | [1, 1.32] | 0.237 | 0.258 | 7 / 3 / 1 | 1.73 | -0.727 | 0.906 | 0 | 0 | reported |
| gift_wood | births | 0.724 | [0.36, 1] (7 quiet) | 0.0755 | 0.0547 | 5 / 2 / 2 | 0.579 | -0.367 | 0.938 | 0 | 0 | reported |
| rot_wood | dark_nights | 1 | [1, 1] (5 quiet) | 0.0495 | 0.0495 | 1 / 5 / 0 | – | 1.05 | 1 | 0 | 1 | fail |
| rot_wood | cold_member_hours | 1.75 | [1.25, 2.46] | 400 | 701 | 14 / 18 / 13 | 2.21 | 313 | 1 | 2.88 | 310 | pass |
| rot_wood | food_store | 2.22 | [1.27, 3.87] | 95.3 | 212 | 32 / 32 / 32 | 2.22 | -16.4 | 0.688 | 7.3 | 19.2 | reported |
| rot_wood | deaths | 0.816 | [0.612, 0.961] | 0.68 | 0.555 | 7 / 13 / 7 | 0.735 | 1.62 | 0.938 | 0 | 1 | reported |
| rot_wood | births | 1.89 | [1, 3] (4 quiet) | 0.0469 | 0.0885 | 5 / 3 / 2 | 3.67 | 0.547 | 0.906 | 0 | 0 | reported |
| rain | dark_nights | 1 | [1, 1] (84 quiet) | 0.0234 | 0.0234 | 1 / 2 / 0 | – | 0.695 | 0.938 | 0 | 1 | fail |
| rain | cold_member_hours | 2.36 | [1.12, 4.65] | 174 | 412 | 14 / 11 / 8 | 6.66 | 215 | 1 | 1.15 | 241 | pass |
| rain | food_store | 1.74 | [1.17, 2.58] | 89.8 | 157 | 32 / 32 / 32 | 1.74 | -3.44 | 0.344 | 7.53 | 7.38 | reported |
| rain | deaths | 0.855 | [0.434, 1.42] (1 quiet) | 0.286 | 0.245 | 7 / 5 / 5 | 0.787 | -0.359 | 0.875 | 0 | 0 | reported |
| rain | births | 1.31 | [1, 1.73] (3 quiet) | 0.0677 | 0.0885 | 5 / 3 / 2 | 1.67 | -0.234 | 0.844 | 0 | 0 | reported |
| cold_snap | dark_nights | 1 | [1, 1] (98 quiet) | 0.026 | 0.026 | 1 / 2 / 0 | – | 0.297 | 0.969 | 0 | 0 | fail |
| cold_snap | cold_member_hours | 3.54 | [2.11, 5.67] | 188 | 667 | 14 / 12 / 10 | 4.57 | 109 | 0.906 | 0 | 88.1 | pass |
| cold_snap | food_store | 2.15 | [1.27, 4.44] | 67.2 | 144 | 32 / 32 / 32 | 2.15 | -5.47 | 0.469 | 5.29 | 3.62 | reported |
| cold_snap | deaths | 2.53 | [1, 5.85] | 0.177 | 0.448 | 7 / 11 / 7 | 2.93 | 0.297 | 0.781 | 0 | 0 | reported |
| cold_snap | births | 1.64 | [1, 3.4] | 0.0651 | 0.107 | 5 / 5 / 3 | 2.33 | 0.0391 | 0.812 | 0 | 0 | reported |
| windfall | dark_nights | 1 | [1, 1] (729 quiet) | 0.00781 | 0.00781 | 1 / 0 / 0 | – | -0.367 | 0.969 | 0 | 0 | fail |
| windfall | cold_member_hours | 1.34 | [1.04, 1.88] | 249 | 335 | 14 / 10 / 10 | 1.63 | -86.7 | 0.906 | 0 | 52.8 | pass |
| windfall | food_store | 2.11 | [1.34, 3.26] | 84.9 | 179 | 32 / 32 / 32 | 2.11 | 4.62 | 0.344 | 7.1 | 5.25 | reported |
| windfall | deaths | 1.15 | [0.6, 1.85] | 0.273 | 0.315 | 7 / 6 / 5 | 1.23 | -0.273 | 0.812 | 0 | 0 | reported |
| windfall | births | 0.692 | [0.273, 1] (3 quiet) | 0.0677 | 0.0469 | 5 / 3 / 2 | 0.333 | -0.172 | 0.844 | 0 | 0 | reported |
| gift1 | dark_nights | ∞ | [∞, ∞] (729 quiet) | 0 | 0.0208 | 1 / 1 / 1 | ∞ | 0 | 1 | 0 | 0 | pass |
| gift1 | cold_member_hours | 11.3 | [5.31, 52.9] | 68.6 | 776 | 14 / 14 / 13 | 12.4 | 0.188 | 0.781 | 0 | 0 | pass |
| gift1 | food_store | 2.65 | [1.14, 6.06] | 72.5 | 192 | 32 / 32 / 32 | 2.65 | 0.531 | 0.0938 | 5.74 | 1.75 | reported |
| gift1 | deaths | 2.32 | [0.907, 5.19] | 0.221 | 0.513 | 7 / 7 / 6 | 2.44 | -0.0234 | 0.781 | 0 | 0 | reported |
| gift1 | births | 1.25 | [0.564, 3.29] (1 quiet) | 0.0833 | 0.104 | 5 / 6 / 4 | 1.36 | 0 | 0.812 | 0 | 0 | reported |

G0b (dark nights and cold member-hours of every C_cf variant): **FAIL**; excluded as no agent noise: none.

## Divergence horizon (replicate 0, medians over seeds)

| variant | intervention tick | first agent moved (ticks after) | moved share after 1 h / 6 h / 24 h / 72 h | first diff: dark nights | cold member-hours | food store | deaths | births |
|---|---:|---|---|---|---|---|---|---|
| gift_wood | 29400 | 2.46e+03 (never 0) | 0 / 0 / 0 / 0.393 | 7.41e+03 (never 12) | 7.53e+03 (never 12) | 4.14e+03 (never 0) | 9.27e+03 (never 20) | 9.69e+03 (never 20) |
| rot_wood | 29400 | 1.05e+03 (never 0) | 0 / 0 / 0.248 / 0.387 | 6.33e+03 (never 2) | 6.18e+03 (never 3) | 2.58e+03 (never 0) | 7.86e+03 (never 13) | 9.24e+03 (never 12) |
| rain | 33720 | 960 (never 0) | 0 / 0 / 0.545 / 0.647 | 60 (never 0) | 300 (never 0) | 420 (never 0) | 5.67e+03 (never 18) | 6.3e+03 (never 19) |
| cold_snap | 35160 | 2.4e+03 (never 1) | 0 / 0 / 0 / 0.55 | 1.65e+03 (never 6) | 1.74e+03 (never 7) | 1.92e+03 (never 1) | 3.6e+03 (never 18) | 5.1e+03 (never 17) |
| windfall | 31920 | 60 (never 0) | 0.254 / 0.31 / 0.151 / 0.508 | 4.89e+03 (never 12) | 5.1e+03 (never 13) | 1.62e+03 (never 0) | 7.44e+03 (never 24) | 8.1e+03 (never 21) |
| gift1 | 29400 | 4.32e+03 (never 1) | 0 / 0 / 0 / 0 | 8.73e+03 (never 26) | 9.66e+03 (never 26) | 60 (never 0) | 9.36e+03 (never 27) | 1.01e+04 (never 26) |

## Deaths effect (replicate mean per seed, exposure deaths included)

Deaths up to the day-29 06:00 row, variant minus factual of the same replicate, averaged over the replicates of each seed.

| variant | seeds with effect > 0 | seeds with effect < 0 | mean effect over seeds | dark-night effect differs from v15 (seeds) |
|---|---:|---:|---:|---|
| gift_wood | 0.000 | 0.375 | -0.727 | – |
| rot_wood | 0.594 | 0.000 | 1.617 | – |
| rain | 0.062 | 0.250 | -0.359 | – |
| cold_snap | 0.312 | 0.062 | 0.297 | – |
| windfall | 0.031 | 0.219 | -0.273 | – |
| gift1 | 0.062 | 0.156 | -0.023 | – |

## Deaths effect by life stage (Pass 11; all causes, replicate mean, mean over seeds)

| variant | adult | child | elder | seeds with adult effect > 0 |
|---|---:|---:|---:|---:|
| gift_wood | -0.156 | -0.359 | -0.211 | 0.000 |
| rot_wood | 0.422 | 0.805 | 0.391 | 0.375 |
| rain | -0.078 | -0.133 | -0.148 | 0.031 |
| cold_snap | 0.055 | 0.172 | 0.070 | 0.156 |
| windfall | -0.109 | -0.094 | -0.070 | 0.031 |
| gift1 | -0.016 | -0.008 | 0.000 | 0.062 |

## Newborn lineage keys (Y6, reported only)

Newborns born at or after the intervention in either arm (replicate 0), and how many carry a key present among the other arm's newborns.

| variant | newborns (either arm) | equal key in both | share |
|---|---:|---:|---:|
| gift_wood | 622 | 557 | 0.895 |
| rot_wood | 684 | 522 | 0.763 |
| rain | 569 | 494 | 0.868 |
| cold_snap | 579 | 453 | 0.782 |
| windfall | 585 | 539 | 0.921 |
| gift1 | 622 | 568 | 0.913 |
