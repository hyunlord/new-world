# Fork metrics (game_v18, seeds 12000–12031, R = 4, fork tick 28800)

## G0a — placebo identity

- amount-0 placebos identical to the factual of the same replicate: 224 / 224
- replicate 0 identical to the unforked run: 224 / 224
- **PASS**

## G0b — variance reduction from pairing

VRF = V_unpaired / V_paired, variances over replicates within a seed, averaged over seeds; 95 % interval from a cluster bootstrap over seeds. The unpaired arm still shares worldgen, weather and the whole prefix up to the fork, so VRF measures the value of pairing the post-fork agent noise only.

Noisy seeds: seeds whose 4 replicates do not all give the same value, in the factual (f), the variant (v) and both; where the factual is constant within a seed, the paired and unpaired differences have the same variance and that seed contributes VRF 1. VRF (both noisy) restricts V_p and V_u to the seeds noisy in both arms (reported only). Bootstrap resamples with no agent noise at all have no VRF and are left out of the interval; their count is shown in brackets after it.

| variant | target | VRF | 95 % CI | V_paired | V_unpaired | noisy seeds f / v / both | VRF (both noisy) | mean effect | sign stability | median std | median abs seed mean | G0b |
|---|---|---:|---|---:|---:|---|---:|---:|---:|---:|---:|---|
| gift_wood | dark_nights | 1 | [1, 1] (10 quiet) | 0.0547 | 0.0547 | 4 / 1 / 0 | – | -0.68 | 0.938 | 0 | 1 | fail |
| gift_wood | cold_member_hours | 1.88 | [1.2, 3.24] | 309 | 582 | 15 / 8 / 8 | 3.14 | -223 | 0.969 | 0 | 243 | pass |
| gift_wood | food_store | 1.91 | [1.18, 3.15] | 50.9 | 96.9 | 32 / 32 / 32 | 1.91 | 12.4 | 0.562 | 5.41 | 14 | reported |
| gift_wood | deaths | 1.48 | [1, 14.3] (3 quiet) | 0.13 | 0.193 | 4 / 4 / 2 | ∞ | -0.375 | 0.938 | 0 | 0 | reported |
| gift_wood | births | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | -0.25 | 1 | 0 | 0 | reported |
| rot_wood | dark_nights | 0.795 | [0.489, 1] | 0.102 | 0.0807 | 4 / 5 / 2 | 0.636 | 1.13 | 0.969 | 0 | 1 | fail |
| rot_wood | cold_member_hours | 1.58 | [1.14, 2.4] | 484 | 766 | 15 / 24 / 15 | 1.81 | 321 | 0.938 | 16.8 | 317 | pass |
| rot_wood | food_store | 1.65 | [0.911, 2.86] | 48.5 | 80.3 | 32 / 32 / 32 | 1.65 | -18.1 | 0.844 | 5.7 | 19.4 | reported |
| rot_wood | deaths | 1.23 | [1, 2.85] (1 quiet) | 0.276 | 0.339 | 4 / 9 / 4 | 1.3 | 1.14 | 0.906 | 0 | 0.125 | reported |
| rot_wood | births | 1 | [1, 1] (30 quiet) | 0.0312 | 0.0312 | 0 / 4 / 0 | – | 0.453 | 0.906 | 0 | 0 | reported |
| rain | dark_nights | 1 | [1, 1] | 0.0807 | 0.0807 | 4 / 5 / 1 | 1 | 1.76 | 0.969 | 0 | 2 | fail |
| rain | cold_member_hours | 1.27 | [0.901, 1.74] | 490 | 620 | 15 / 23 / 10 | 1.61 | 501 | 1 | 18.4 | 519 | fail |
| rain | food_store | 3.61 | [2.12, 5.96] | 25.8 | 93.3 | 32 / 32 / 32 | 3.61 | -23.9 | 0.969 | 4.61 | 25 | reported |
| rain | deaths | 1.88 | [0.758, 3.33] | 0.331 | 0.622 | 4 / 11 / 4 | 2.45 | 1.16 | 0.906 | 0 | 0.125 | reported |
| rain | births | 1 | [1, 1] (1 quiet) | 0.0495 | 0.0495 | 0 / 6 / 0 | – | 0.383 | 0.938 | 0 | 0 | reported |
| cold_snap | dark_nights | 1.5 | [1, 2.18] (1 quiet) | 0.0833 | 0.125 | 4 / 5 / 2 | 2.07 | 0.406 | 0.812 | 0 | 0 | fail |
| cold_snap | cold_member_hours | 1.07 | [0.814, 1.6] | 751 | 801 | 15 / 15 / 10 | 1.13 | 147 | 0.875 | 14.9 | 102 | fail |
| cold_snap | food_store | 9.15 | [4.8, 16.4] | 11.2 | 103 | 32 / 32 / 32 | 9.15 | -7.22 | 0.719 | 1.96 | 5.38 | reported |
| cold_snap | deaths | 2.33 | [1, 4.27] (1 quiet) | 0.188 | 0.438 | 4 / 6 / 3 | 4.56 | 0.141 | 0.844 | 0 | 0 | reported |
| cold_snap | births | 1 | [1, 1] (87 quiet) | 0.026 | 0.026 | 0 / 3 / 0 | – | 0.0469 | 0.938 | 0 | 0 | reported |
| windfall | dark_nights | 0.758 | [0.429, 1] (6 quiet) | 0.0859 | 0.0651 | 4 / 3 / 1 | 0.273 | -0.258 | 0.812 | 0 | 0 | fail |
| windfall | cold_member_hours | 1.47 | [1.07, 2.09] | 391 | 573 | 15 / 11 / 8 | 2.16 | -86.2 | 0.906 | 16.5 | 84.4 | pass |
| windfall | food_store | 2.39 | [1.47, 3.97] | 33 | 79 | 32 / 32 / 32 | 2.39 | 4.86 | 0.438 | 4.68 | 3.88 | reported |
| windfall | deaths | 0.273 | [0.121, ∞] (31 quiet) | 0.258 | 0.0703 | 4 / 3 / 3 | 0.242 | -0.148 | 0.906 | 0 | 0 | reported |
| windfall | births | 1 | [1, 1] (261 quiet) | 0.0182 | 0.0182 | 0 / 2 / 0 | – | -0.102 | 0.969 | 0 | 0 | reported |
| gift1 | dark_nights | 7.67 | [1, ∞] (29 quiet) | 0.0156 | 0.12 | 4 / 3 / 3 | 14.3 | 0 | 0.938 | 0 | 0 | fail |
| gift1 | cold_member_hours | 39.7 | [17, ∞] | 21.5 | 853 | 15 / 14 / 14 | 65.2 | -0.945 | 0.906 | 0 | 0 | pass |
| gift1 | food_store | 5.61 | [3.36, 9.24] | 17.8 | 99.6 | 32 / 32 / 32 | 5.61 | 0.219 | 0.0625 | 3.23 | 0.875 | reported |
| gift1 | deaths | 14.1 | [11.2, ∞] (31 quiet) | 0.0286 | 0.404 | 4 / 4 / 4 | 14.1 | 0.00781 | 0.969 | 0 | 0 | reported |
| gift1 | births | – | [–, –] | 0 | 0 | 0 / 0 / 0 | – | 0 | 1 | 0 | 0 | reported |

G0b (dark nights and cold member-hours of every C_cf variant): **FAIL**; excluded as no agent noise: none.

## Divergence horizon (replicate 0, medians over seeds)

| variant | intervention tick | first agent moved (ticks after) | moved share after 1 h / 6 h / 24 h / 72 h | first diff: dark nights | cold member-hours | food store | deaths | births |
|---|---:|---|---|---|---|---|---|---|
| gift_wood | 29400 | 2.46e+03 (never 0) | 0 / 0 / 0 / 0.481 | 7.74e+03 (never 11) | 7.74e+03 (never 11) | 3.36e+03 (never 0) | 9.3e+03 (never 21) | 9.42e+03 (never 21) |
| rot_wood | 29400 | 1.17e+03 (never 0) | 0 / 0 / 0.0429 / 0.382 | 6.87e+03 (never 2) | 6.42e+03 (never 3) | 2.64e+03 (never 0) | 9.24e+03 (never 15) | 1.03e+04 (never 14) |
| rain | 33720 | 1.08e+03 (never 0) | 0 / 0 / 0.463 / 0.609 | 60 (never 0) | 300 (never 0) | 480 (never 0) | 4.92e+03 (never 17) | 6.36e+03 (never 15) |
| cold_snap | 35160 | 2.4e+03 (never 3) | 0 / 0 / 0 / 0.531 | 1.98e+03 (never 5) | 2.04e+03 (never 9) | 1.98e+03 (never 3) | 3.54e+03 (never 28) | 9.36e+03 (never 26) |
| windfall | 31920 | 60 (never 0) | 0.223 / 0.308 / 0.13 / 0.457 | 5.22e+03 (never 11) | 5.22e+03 (never 11) | 930 (never 0) | 6.78e+03 (never 24) | 7.62e+03 (never 27) |
| gift1 | 29400 | 4.17e+03 (never 2) | 0 / 0 / 0 / 0 | 1.04e+04 (never 27) | 1.04e+04 (never 29) | 60 (never 0) | 9.36e+03 (never 31) | 1.49e+04 (never 29) |

## Deaths effect (replicate mean per seed, exposure deaths included)

Deaths up to the day-29 06:00 row, variant minus factual of the same replicate, averaged over the replicates of each seed.

| variant | seeds with effect > 0 | seeds with effect < 0 | mean effect over seeds | dark-night effect differs from v15 (seeds) |
|---|---:|---:|---:|---|
| gift_wood | 0.000 | 0.281 | -0.375 | – |
| rot_wood | 0.500 | 0.000 | 1.141 | – |
| rain | 0.500 | 0.000 | 1.164 | – |
| cold_snap | 0.188 | 0.000 | 0.141 | – |
| windfall | 0.000 | 0.125 | -0.148 | – |
| gift1 | 0.031 | 0.000 | 0.008 | – |

## Deaths effect by life stage (Pass 11; all causes, replicate mean, mean over seeds)

| variant | adult | child | elder | seeds with adult effect > 0 |
|---|---:|---:|---:|---:|
| gift_wood | -0.062 | -0.305 | -0.008 | 0.000 |
| rot_wood | 0.547 | 0.562 | 0.031 | 0.312 |
| rain | 0.602 | 0.500 | 0.062 | 0.406 |
| cold_snap | 0.078 | 0.047 | 0.016 | 0.125 |
| windfall | -0.023 | -0.125 | 0.000 | 0.000 |
| gift1 | 0.016 | 0.000 | -0.008 | 0.031 |

## Newborn lineage keys (Y6, reported only)

Newborns born at or after the intervention in either arm (replicate 0), and how many carry a key present among the other arm's newborns.

| variant | newborns (either arm) | equal key in both | share |
|---|---:|---:|---:|
| gift_wood | 558 | 545 | 0.977 |
| rot_wood | 601 | 539 | 0.897 |
| rain | 475 | 416 | 0.876 |
| cold_snap | 404 | 388 | 0.96 |
| windfall | 483 | 478 | 0.99 |
| gift1 | 559 | 558 | 0.998 |
