# WM-1.2 metrics and pre-registration

Pre-registration of GOAL_1_2.md section 8. Written on 2026-10-12 after the validation runs below and **before any
test-role number of a new model exists**: no `metrics.json` of `m1tf_p10v16_r3_s*` or `m1tf_p10v15_r3_s0`, no probe
output, and no test-role number of any baseline on the pilot. M1b seeds 0 and 1 of `game_v16` are training on the
DGX at the time of writing (launched 21:54 from code `da379c17`; validation only).

- Parent code commit that computes every test number below: **`ef880949`** (`climatology.py --dataset`,
  `evaluate.py`'s `tasks=` / `bias_removed=`, `data.py`'s schema-4 table, `m1_gru.py final` reading the dataset from
  `config.json`, `probe_world.py`, `judge_1_2.py`). The test jobs run from the commit that adds this file; it changes
  nothing under `tools/`. Any later change to those files before the test numbers exist gets a dated amendment at
  the end of this file.
- Validation runs (no test role read): `calendar_val_p10_v16_da379c17`, `calendar_val_p10_v15_da379c17`,
  `calendar_val_p10_v16n_da379c17` (10–11 s each; JSON copies in `runs/`). `da379c17` and `ef880949` differ only by
  `M1TF.hidden`, `probe_world.py`, `judge_1_2.py` and their tests (no change to anything the validation runs read).

## 1. Data and roles

`docs/worldmodel/wm1/p10_pilot_manifest.json` (sha256 `d1d0a847…`): `p10/pilot/{game_v15,game_v16_neutral,game_v16}`,
112 episodes per scene (seeds 20000–20111, 120 days, 2,881 rows, recorder schema 4, recorded by `worldsim_wm` built
from `58119582`; every episode's sha256 verified on the DGX). Roles (GOAL_1_2.md 4.2): **train 20000–20079 (80),
val 20080–20095 (16), test 20096–20111 (16)** for every scene. Datasets: `p10_v16` (`game_v16`, hidden world
parameters), `p10_v15` (`game_v15`, the control: no world variety), `p10_v16n` (`game_v16_neutral`, baselines only).

Inputs (all models and baselines): observable fields only. The model's input list (65 fields, asserted exactly in
`test_pilot_harness.py`): the 33 v0 severity-blind state columns, `d.deaths_exposure` (hourly change),
`g.injured_members`, and the 30 calendar features (hour one-hot, season one-hot, day of season / 7). Never inputs:
`winter_severity`, `night_burn_per_hour`, `winter_fall_today`, `wood_target`, `latent.npy`, `lifespan_days`,
`meta.json`'s `world_variety`. The pilot has no interventions; the models are action-blind.

## 2. Scores

The WM-1 harness (`evaluate.evaluate`, tasks T1, T2, T2_any; no T3, T3b, T4 on the pilot). V = food_stock,
wood_stock, fire_lit, cold_members, alive, deadwood_total, food_on_bushes_total, adults_working. nMAE = MAE / the
std of each variable over rows >= 24 of the scored split (`bench_v0.split_std` = `bench_v1.split_std`); the mean is
over the 8 variables. T2 at the horizon (24 h, 168 h) from **every-hour starts** (every hour of one-based days 3, 10,
17, …: `bench_v1.rollout_starts(n, h, True)`; T2_any, **primary**) and from **06:00 starts** (T2, reported).

**Bias-removed score** (GOAL_1_2.md 7.2): per episode and variable, e = prediction − truth at each scored start;
subtract the mean of e over that episode's scored starts; nMAE of the de-meaned error with the same split std. It
removes a predictor's static per-world offset and keeps what it gets wrong about how the world moves. (It subtracts
the mean, not the median, so for a skewed error it can exceed the raw score; persistence's does.)

## 3. Calendar family and selection rule

The nine members of WM-1.1 (`calcore.py`), fit on the dataset's **80 train episodes** (T_tr = 2,881 asserted,
every year position covered); `apfit_*` α by least squares on train; analogs search the 80 train episodes. Selection
on **val** (16 episodes), per task, by the mean nMAE (bias-removed tasks: the bias-removed mean): **`cal_best`** = the
lowest of the nine (ties: member order); **`cal_best_fair`** = the best of {`cal_year`, `ap1_year`, `apfit_year`};
**`cal_best_anom`** = `cal_best` of a bias-removed task. Never re-selected on test, never refit on train + val.
References evaluated beside them: persistence and the observable ridge (`bench_v1`'s `ridge`: lags 1, 2, 3, 6, 12,
24 of V, hour and season one-hots, no severity; λ = 100 on the val T1 mean nMAE for all three datasets).

Validation scores, `p10_v16` (bold `cal_best`, "(fair)" `cal_best_fair`):

| member | T1 | T2_24 | T2_168 | T2any_24 | T2any_168 | T2br_24 | T2br_168 | T2anybr_24 | T2anybr_168 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `cal_abs` | 0.3700 | 0.3075 | 0.3066 | 0.3539 | 0.3662 | 0.1530 | **0.1272** | 0.2135 | **0.2100** |
| `cal_year` | 0.3740 | 0.3131 | 0.3073 | 0.3590 | 0.3662 | 0.1606 | 0.1360 (fair) | 0.2215 | 0.2184 (fair) |
| `ap1_abs` | 0.1034 | 0.1024 | 0.1879 | 0.1833 | 0.2864 | 0.1087 | 0.1900 | 0.1884 | 0.2892 |
| `ap1_year` | 0.1034 | 0.1038 (fair) | 0.2028 | 0.1846 | 0.2994 | 0.1098 | 0.2056 | 0.1897 | 0.3024 |
| `apfit_abs` | 0.0939 | 0.1072 | 0.1742 | 0.1681 | 0.2431 | 0.1065 | 0.1567 | 0.1699 | 0.2375 |
| `apfit_year` | 0.0939 (fair) | 0.1092 | 0.1883 (fair) | 0.1707 (fair) | 0.2562 (fair) | 0.1072 (fair) | 0.1706 | 0.1719 (fair) | 0.2500 |
| `an1` | 0.2261 | 0.1921 | 0.2139 | 0.2520 | 0.2905 | 0.1889 | 0.2030 | 0.2508 | 0.2855 |
| `an5` | 0.1893 | 0.1564 | 0.1739 | 0.2084 | **0.2382** | 0.1437 | 0.1472 | 0.1966 | 0.2186 |
| `an5_off` | **0.0903** | **0.0913** | **0.1489** | **0.1650** | 0.2391 | **0.0972** | 0.1507 | **0.1687** | 0.2415 |
| persistence (reference) | 0.1444 | 0.1345 | 0.4606 | 0.2104 | 0.5760 | 0.1469 | 0.4686 | 0.2225 | 0.5842 |
| ridge (reference) | 0.1104 | 0.1823 | 0.3289 | 0.2295 | 0.3287 | 0.1624 | 0.2289 | 0.2157 | 0.2856 |

Validation scores, `p10_v15`:

| member | T1 | T2_24 | T2_168 | T2any_24 | T2any_168 | T2br_24 | T2br_168 | T2anybr_24 | T2anybr_168 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `cal_abs` | 0.1675 | 0.0866 | 0.0964 | 0.1467 | 0.1597 | 0.0755 | 0.0806 | 0.1382 | 0.1500 |
| `cal_year` | 0.2188 | 0.1570 | 0.1412 | 0.2138 | 0.2021 | 0.1447 | 0.1324 (fair) | 0.2043 | 0.2001 (fair) |
| `ap1_abs` | 0.0860 | 0.0838 | 0.1148 | 0.1550 | 0.2006 | 0.0861 | 0.1142 | 0.1619 | 0.2006 |
| `ap1_year` | 0.0868 | 0.0967 | 0.2216 | 0.1659 | 0.3031 | 0.0982 | 0.2324 | 0.1725 | 0.3152 |
| `apfit_abs` | 0.0764 | 0.0711 | 0.0835 | 0.1239 | 0.1482 | 0.0713 | 0.0796 | 0.1265 | 0.1492 |
| `apfit_year` | 0.0774 (fair) | 0.0792 (fair) | 0.1302 (fair) | 0.1297 (fair) | 0.1920 (fair) | 0.0788 (fair) | 0.1340 | 0.1323 (fair) | 0.2006 |
| `an1` | 0.1049 | 0.0753 | 0.0934 | 0.1338 | 0.1625 | 0.0806 | 0.0998 | 0.1406 | 0.1724 |
| `an5` | 0.0906 | **0.0634** | **0.0745** | **0.1123** | **0.1367** | **0.0644** | **0.0756** | **0.1161** | **0.1412** |
| `an5_off` | **0.0692** | 0.0679 | 0.0855 | 0.1205 | 0.1596 | 0.0695 | 0.0874 | 0.1236 | 0.1626 |
| persistence (reference) | 0.1578 | 0.1366 | 0.5452 | 0.1962 | 0.6526 | 0.1534 | 0.5733 | 0.2150 | 0.6853 |
| ridge (reference) | 0.1018 | 0.1502 | 0.2535 | 0.1945 | 0.2552 | 0.1444 | 0.1798 | 0.1859 | 0.2404 |

Selection used by the Z rules:

| dataset | task | `cal_best` | `cal_best_fair` |
|---|---|---|---|
| p10_v16 | T2any_168 (Z1) | `an5` | `apfit_year` |
| p10_v16 | T2any_24 (Z1b) | `an5_off` | `apfit_year` |
| p10_v16 | T2anybr_168 (Z2: `cal_best_anom`) | `cal_abs` | `cal_year` |
| p10_v15 | T2any_168 (Z3) | `an5` | `apfit_year` |

(`p10_v16n` is selected the same way and reported only: `an5` for T2any_168 and T2anybr_168.)

## 4. Configurations evaluated on test

Each once, by `python3 tools/worldmodel/wm1/m1_gru.py final --run /home/hyunlord/work/wm1-runs/<run>` (the dataset
comes from the run's `config.json`) from the commit that adds this file; then `probe_world.py --run <run> --out
/home/hyunlord/work/wm1-runs/probe_<run>` for the three `game_v16` seeds:

| run | command (`m1_tf.py train …`) |
|---|---|
| `m1tf_p10v16_r3_s0`, `_s1`, `_s2` | `--dataset p10_v16 --out /home/hyunlord/work/wm1-runs/m1tf_p10v16_r3_s<S> --variant sev-blind_action-blind --seed S --batch 32 --clamp-nonneg 1 --eval-every 150 --k-max 168 --max-steps 3000 --roll-min-context 167 --roll-starts 1 --tf-l1 1.0 --threads 8 --v-weight 5.0 --window 336` |
| `m1tf_p10v15_r3_s0` | the same with `--dataset p10_v15`, S = 0 |

WM-1's recipe r3 for M1b; the only difference from WM-1's main configuration besides the data is
`sev-blind_action-blind` (the pilot has no interventions). Early stopping on the validation T2 24 h at 06:00 starts
(the recipe's rule, patience 5 evaluations after the curriculum's half). Each run records its `stop` reason. A run
whose final validation T2 168 h (every-hour starts) is worse than persistence's validation number (p10_v16 0.5760,
p10_v15 0.6526) is recorded as such and kept; nothing is tuned on test.

The calendar family and the references on test, once each, from the same commit:
`climatology.py final --dataset p10_v16 --out /home/hyunlord/work/wm1-runs/calendar_p10_v16_<sha8> --selection
calendar_val_p10_v16_da379c17/calendar_val.json` (and `p10_v15`, `p10_v16n`).

## 5. Rules

Model score = the mean over the 3 seeds of each seed's mean nMAE on `game_v16` test (16 episodes); `cal` = the test
score of the val-selected member of the same task. Computed by `judge_1_2.py` from the result files only.

| # | rule |
|---|---|
| Z1 | **v16, raw scores.** M1b (3 seeds) vs `cal_best` (`an5`) at T2 168 h every-hour starts, test: **KEEP** if the 3-seed mean ≤ 0.8 × `cal_best` and every seed < `cal_best`; **WEAK** if the mean < `cal_best`; **NONE** otherwise. Same at 24 h against `an5_off` (reported as **Z1b**). |
| Z2 | **v16, bias-removed scores.** The same rule on the bias-removed T2 168 h every-hour score against `cal_best_anom` (`cal_abs`, bias-removed). Z1 KEEP with Z2 NONE means the model's edge is identifying the world's level, not forecasting its movement. |
| Z3 | **control.** The v15 model (seed 0) vs `cal_best` (`an5`) on the v15 pilot, raw, 168 h every-hour starts; same thresholds with the single seed as mean and every seed. Expected NONE, as in WM-1.1; reported. |
| Z4 | **per variable.** Skill 1 − nMAE / baseline of the M1b seed mean for food (`food_stock`), alive, bushes (`food_on_bushes_total`) and working adults (`adults_working`) at T2 168 h every-hour starts: raw against `cal_best` (`an5`) and `apfit_abs`; bias-removed against `cal_best_anom` (`cal_abs`) and `apfit_abs` (both bias-removed). Reported. |
| Z5 | **world identification probe.** For each M1b v16 seed: `M1TF.hidden` (the final layer-normed state at the last position) at row 166 = 06:00 of day 8 (window rows −1..166, row −1 masked) for every episode; per hidden parameter of `world_variety` (`starting_agents`, `berry_bushes`, `berry_capacity`, `deadwood_spots`, `deadwood_capacity`, `forest_permille`, `food_regen_permille`) a ridge (`bench_v0.Ridge`, standardized inputs) fit on the 80 train episodes, λ ∈ {0.01, 0.1, 1, 10, 100, 1000, 10000} by val MSE, **R² on test** (1 − SSE / sum of squares around the test mean); `climate` (0–2): one-vs-rest ridge on one-hot targets (λ by val one-hot MSE), **accuracy of the argmax on test** (beside the train-majority accuracy). Reference: the same ridges from the raw observable state row at s (35 columns). Labels are read from `meta.json` by the probe only, after training. Reported. `starting_agents` equals `alive` at row 0, so the reference recovers it almost trivially. |

If a seed is dropped (GOAL_1_2.md 0.3.6: the third v16 seed is the last item of the drop order), Z1, Z1b, Z2 and Z4
use the seeds that exist and say so; KEEP then needs every existing seed below `cal`.

## 6. Limitations known before the test numbers

- 80 training worlds (Pass 10 fit its calendar means on 96), 16 test worlds: the test score of one seed is an average
  over 16 worlds × 384 every-hour starts at 168 h, but world-level effects have only 16 independent draws.
- GPU reproducibility of M1b is not measured (as in WM-1.1).
- R² on 16 test worlds is noisy; no confidence interval is pre-registered.

## Amendment 2026-10-12 (review round 1, before any model test number)

`judge_1_2.py` took one commit sha for both calendar run directories; the validation runs are `…_da379c17` and the
test runs `…_1a7dfdb6`, so it now takes `--val-sha` and `--test-sha` (and inserts its own `sys.path`). No rule,
member, threshold or score definition changed. `families.py` also checks `settlement_ids.npy` in its prefix check
(not used by any Z rule). At the time of this amendment no `metrics.json` of an M1b run and no probe output exists.
