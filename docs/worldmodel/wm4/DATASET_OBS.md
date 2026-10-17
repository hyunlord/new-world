# Dataset obs (WM-4): the confounded observational split of `game_v19`

Recorded by WM-4 Part 1 (GOAL.md 6) with WM-2's frozen recorder (`_build/src_wm2`, `worldsim_wm` `52b36541…`, `WORLDSIM_GIT_SHA` `f5373ab2…`) on the DGX into `/home/hyunlord/work/worldsim-wm-data/obs/`; the policy and its frozen terciles are METRICS.md part a (committed `bfa36208` before any treated episode). Manifest: `obs_manifest.json` (1,200 episodes, every one verified; paths relative to `WM_DATA_ROOT`); policy record `runs/record/policy.json` (sha256 `feef826c…`); checks `runs/record/`.

## Episodes

| directory | content | episodes |
|---|---|---|
| `obs/factual/` | every world 46000–46559, no intervention (120 days, a row per hour) | 560 |
| `obs/observed/` | every world with its policy schedule and `--include-factual`: `_none` (= the factual, array for array) and, for a casting world, `_obs` | 1,019 (560 + 459) |
| `obs/families/{val,test}/` | per val / test world, forked at c_w with replicate 0: `none` and the three arms at c_w | 640 (160 × 4) |

Manifest splits: `obs_train` 400 and `obs_val` 80 (the observed episode of each world: `_obs` of a casting world, the factual otherwise), `obs_test` 80 (each test world's factual), `obsfam_{none,gf240,sp240,rw120}` 160 each (80 val + 80 test).

**Checks** (`runs/record/`): 560 factual episodes verified, 2,881 rows; every observed `_none` equals the factual by per-array sha256; every `_obs` equals its factual on rows ≤ the cast row (`global.npy` in full, `settlements.npy` by WM-2 6.2's rule) and has one record with the policy's kind, amount and tick; all 160 families pass WM-2's per-member prefix check, their `none_r0` equals the factual and a casting world's `_obs` equals its family member of the same arm (per-array sha256; the replicate-0 fork is a no-op).

## The policy and the confounding

Terciles (train factual decision rows, 21,200): q_f1 = 8.0792, q_f2 = 12.1000 food per person; q_w2 = 3.1119 wood per person. No decision row of days 8 … 60 was a non-decision (every world alive with a settlement).

| worlds | never cast | casts | `gf240` | `sp240` | `rw120` |
|---|---|---|---|---|---|
| all (560) | 101 (18.0 %) | 459 | 183 (39.9 %) | 186 (40.5 %) | 90 (19.6 %) |
| train (400) | 80 (20.0 %) | 320 | 129 (40.3 %) | 132 (41.2 %) | 59 (18.4 %) |
| val (80) | 4 (5.0 %) | 76 | 30 (39.5 %) | 26 (34.2 %) | 20 (26.3 %) |
| test (80) | 17 (21.2 %) | 63 | 24 (38.1 %) | 28 (44.4 %) | 11 (17.5 %) |

**Arm shares by the f-tercile of the cast row** (the confounding made visible: gifts go to hungry worlds, spoils to fed ones):

| f tercile at the cast row | casts | `gf240` | `sp240` | `rw120` | decision rows spent there | realized daily cast rate |
|---|---|---|---|---|---|---|
| f < q_f1 (hungry) | 148 | 85.8 % | 3.4 % | 10.8 % | 4,601 | 0.0322 |
| q_f1 ≤ f < q_f2 | 109 | 38.5 % | 30.3 % | 31.2 % | 4,945 | 0.0220 |
| f ≥ q_f2 (fed) | 202 | 6.9 % | 73.3 % | 19.8 % | 5,152 | 0.0392 |

GOAL.md 6.1's expectation (≈ 17–19 % never cast; ≈ 40 / 40 / 20 % among casts) is met: 18.0 % and 39.9 / 40.5 / 19.6 %.

**Applied amounts.** 456 of 459 casts changed the trajectory; the other 3 are `rw120` on a store with no wood (applied 0). The applied amount was below the request in 43 `rw120` and 4 `sp240` casts (the store held less); in the 160 val / test families at c_w, `gf240` applied 240 in all 160, `sp240` the full 240 in 118 and less in 42, `rw120` the full 120 in 49, less in 106 and nothing in 5 (`runs/record/fam_check.json`): a `rot_wood` 120 usually exceeds the store at c_w, so its effect is mostly "lose the whole wood store".

## Overlap and effective sample size (observed train worlds, `effects4.pooled_estimates`)

Risk-set units: every decision row c of a world that has not cast before c. Treated units carry weight 1 / p_a(c) (p ≥ 0.002, so ≤ 500); control units 1 / π, π = ∏ (1 − p_tot) over the decision rows in [c, c + h).

| arm | lag | treated | ESS treated | max weight | controls | ESS controls | max weight |
|---|---|---|---|---|---|---|---|
| gf240 | 7d | 129 | 44.3 | 500 | 8,441 | 8,419.3 | 1.37 |
| gf240 | 30d | 129 | 44.3 | 500 | 5,018 | 4,673.1 | 3.21 |
| sp240 | 7d | 132 | 54.9 | 500 | 8,441 | 8,419.3 | 1.37 |
| sp240 | 30d | 132 | 54.9 | 500 | 5,018 | 4,673.1 | 3.21 |
| rw120 | 7d | 59 | 46.7 | 250 | 8,441 | 8,419.3 | 1.37 |
| rw120 | 30d | 59 | 46.7 | 250 | 5,018 | 4,673.1 | 3.21 |

The treated ESS is about a third of the treated count: a few casts in the unlikely tercile (a gift to a fed world, p = 0.002) carry most of the weight.

## True effects on the val families (c_w + 7 d / + 30 d; mean, std, share of cases ≠ 0)

The test families' distribution is added after METRICS.md part b is pushed.

| arm | outcome | 7 d mean (std) [≠ 0] | 30 d mean (std) [≠ 0] |
|---|---|---|---|
| gf240 | food_stock | 32.00 (37.93) [99 %] | -15.93 (61.03) [98 %] |
| gf240 | alive | -0.19 (0.84) [6 %] | -2.94 (9.78) [11 %] |
| gf240 | deaths | 0.01 (0.11) [1 %] | -0.04 (0.53) [6 %] |
| gf240 | settlements_alive | -0.05 (0.22) [5 %] | -0.09 (0.28) [9 %] |
| gf240 | split shift (days, restricted) | 4.90 (9.40) [38 %] | |
| sp240 | food_stock | -73.80 (70.09) [100 %] | 36.16 (93.00) [99 %] |
| sp240 | alive | 1.15 (3.64) [18 %] | 4.39 (10.35) [24 %] |
| sp240 | deaths | 0.01 (0.11) [1 %] | 0.21 (1.22) [12 %] |
| sp240 | settlements_alive | 0.10 (0.30) [10 %] | 0.12 (0.33) [12 %] |
| sp240 | split shift (days, restricted) | -7.28 (13.73) [48 %] | |
| rw120 | food_stock | -8.00 (22.05) [92 %] | 2.99 (36.96) [95 %] |
| rw120 | alive | -0.05 (0.31) [2 %] | 0.79 (4.40) [6 %] |
| rw120 | deaths | 0.17 (0.75) [9 %] | 0.28 (0.94) [18 %] |
| rw120 | settlements_alive | 0.00 (0.00) [0 %] | 0.04 (0.19) [4 %] |
| rw120 | split shift (days, restricted) | -0.51 (5.18) [29 %] | |

Reference day of the val cases: mean 29.6, std 14.3 (cast days 8 … 60, never-cast worlds at day 30).

## Effect scales (GOAL.md 4.1)

Per (outcome, lag): max(the v2 partner arm's train std, the v2 train std pooled over every arm's train cases), from WM-2's `effects_base.json`. The floor binds where the v2 arm's own effect is nearly constant (e.g. `gf240E` `alive` 7 d: 0.086 → 4.046).

| cell | scale (floored) | v2 arm's std (unfloored)¹ |
|---|---|---|
| gf240|alive|30d | 11.222 | 9.074 |
| gf240|alive|7d | 4.046 | 0.086 |
| gf240|deaths|30d | 1.570 | 0.162 |
| gf240|deaths|7d | 1.062 | 0.071 |
| gf240|food_stock|30d | 106.282 | 85.417 |
| gf240|food_stock|7d | 72.858 | 25.949 |
| gf240|settlements_alive|30d | 0.352 | 0.273 |
| gf240|settlements_alive|7d | 0.309 | 0.309 |
| rw120|alive|30d | 11.222 | 7.571 |
| rw120|alive|7d | 4.046 | 1.143 |
| rw120|deaths|30d | 2.248 | 2.248 |
| rw120|deaths|7d | 1.742 | 1.742 |
| rw120|food_stock|30d | 106.282 | 71.252 |
| rw120|food_stock|7d | 72.858 | 20.851 |
| rw120|settlements_alive|30d | 0.352 | 0.228 |
| rw120|settlements_alive|7d | 0.309 | 0.309 |
| sp240|alive|30d | 13.157 | 13.157 |
| sp240|alive|7d | 6.204 | 6.204 |
| sp240|deaths|30d | 1.570 | 1.119 |
| sp240|deaths|7d | 1.062 | 1.062 |
| sp240|food_stock|30d | 131.739 | 131.739 |
| sp240|food_stock|7d | 77.338 | 77.338 |
| sp240|settlements_alive|30d | 0.447 | 0.447 |
| sp240|settlements_alive|7d | 0.463 | 0.463 |

¹ WM-2's `effects_base.json` `stds`: where the v2 arm's own train std is 0 (`gf240E` `settlements_alive` 7 d, `rw120E` `settlements_alive` 7 d, `sp240L` `deaths` 7 d) it holds WM-2's pooled fallback.

**Never-cast worlds by role vs the policy's expectation** (review round 1, from each world's factual states): train 80 vs 66.9 ± 7.5, val 4 vs 13.4 ± 3.3 (z ≈ −2.8), test 17 vs 13.4; every draw replays exactly from `default_rng([seed, day])`, so the low val count is chance (95 % of val worlds cast).

## True effects on the test families (added after METRICS.md part b, `956c8a05`)

| arm | outcome | 7 d mean (std) [≠ 0] | 30 d mean (std) [≠ 0] |
|---|---|---|---|
| gf240 | food_stock | 18.64 (43.46) [99 %] | -16.93 (50.93) [94 %] |
| gf240 | alive | -0.84 (2.48) [12 %] | -2.19 (8.05) [8 %] |
| gf240 | deaths | 0.00 (0.00) [0 %] | 0.03 (0.16) [2 %] |
| gf240 | settlements_alive | -0.04 (0.19) [4 %] | -0.06 (0.24) [6 %] |
| gf240 | split shift (days, restricted) | 3.73 (9.12) [39 %] | |
| sp240 | food_stock | -81.42 (63.59) [100 %] | 32.39 (84.17) [98 %] |
| sp240 | alive | 1.31 (4.21) [24 %] | 4.36 (10.32) [20 %] |
| sp240 | deaths | 0.00 (0.00) [0 %] | 0.07 (0.38) [5 %] |
| sp240 | settlements_alive | 0.07 (0.26) [8 %] | 0.14 (0.34) [14 %] |
| sp240 | split shift (days, restricted) | -9.01 (19.32) [50 %] | |
| rw120 | food_stock | -4.09 (17.54) [94 %] | 2.52 (28.46) [89 %] |
| rw120 | alive | -0.06 (0.73) [4 %] | 0.80 (4.85) [5 %] |
| rw120 | deaths | 0.20 (0.89) [6 %] | 0.23 (0.82) [9 %] |
| rw120 | settlements_alive | 0.00 (0.00) [0 %] | 0.01 (0.11) [1 %] |
| rw120 | split shift (days, restricted) | -1.19 (4.77) [21 %] | |

Reference day of the test cases: mean 29.3, std 13.6.
