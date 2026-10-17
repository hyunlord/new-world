# WM-1.3 metrics and pre-registration

Pre-registration of GOAL_1_3.md section 8. Written on 2026-10-13 after the validation runs below and **before any
test-role number of a new model exists**: no `metrics.json` of any `m1tf_w13*` run, of WM-1.2's runs on the new test
or on the WM-1.2-rule checkpoints, no probe output, and no test-role number of any baseline on the w13 datasets.
At the time of writing the first M1b run (`m1tf_w13v16_n400_s0`) is training on the DGX (launched 12:21:56 from code `b7a48fed`; validation only), one run at a time: the throughput check gave 1.50 s/step for one job alone and 3.27 s/step per job for two at once (ratio 2.17 ≥ 1.8, GOAL_1_3.md 9.1), so the queue runs serially in the order of section 4 and the drop order of GOAL_1_3.md 0.3.6 applies.

- Parent code commit that computes every test number below: **`b7a48fed`** (`data.py`'s extra state columns,
  `common.py`'s w13 registry and nested `train_seeds`, `evaluate.py`'s per-world T2 scores, `m1_gru.py`'s
  `ramp_steps`, WM-1.2 rule online and `final --ckpt / --eval-dataset / --out`, `climatology.py final --eval-dataset`,
  `bootstrap.py`, `probe_world.py`'s Y6 labels, `judge_1_3.py`). The test jobs run from the commit that adds this file
  (it changes nothing under `tools/`). Any later change to those files before the test numbers exist gets a dated
  amendment at the end of this file.
- Validation runs (no test role read): `calendar_val_<ds>_b7a48fed` for the four datasets (JSON copies in `runs/`).

## 1. Data and roles

`docs/worldmodel/wm1/w13_manifest.json` (sha256 `dba15cbb…`): `w13/game_v16/` and `w13/game_v17/`, 560 episodes
each (120 days, 2,881 rows, recorder schema 5, recorded by `worldsim_wm` built from the merge commit `8f6b10ed`
with `meta.git_sha` = `8f6b10ed…`; every episode verified by `wm_data.verify_episode` on the DGX). The recorder is
equivalent to Pass 10's on `game_v16` (DECISIONS G1.2). Roles (GOAL_1_3.md 4.2, fixed before recording):

| scene | train | val | test |
|---|---|---|---|
| `game_v16` | 30000–30399 (400) | 30400–30479 (80) | 30480–30559 (80) |
| `game_v17` | 32000–32399 (400) | 32400–32479 (80) | 32480–32559 (80) |

Datasets (`common.DATASETS`): `w13_v16_n80` (train 30000–30079), `w13_v16_n200` (30000–30199), `w13_v16_n400`
(30000–30399), `w13_v17_n400` (32000–32399); every `game_v16` size uses the same val and test worlds. The **legacy
test** is `p10_v16`'s test (16 worlds, seeds 20096–20111, WM-1.2's), report only.

Inputs: observable fields only. `game_v16` arm: exactly WM-1.2's 65 names in order (the 35 observable state columns
and the 30 calendar features). `game_v17` arm: the same 35 state columns, then `g.settlements_alive` and
`g.fissions_total` (raw levels; state features: normalized, predicted, rolled out), then the 30 calendar features =
67 names. Both asserted exactly in `test_w13.py`. Never inputs: `winter_severity`, `night_burn_per_hour`,
`winter_fall_today`, `wood_target`, `latent.npy`, `lifespan_days`, `meta.json`'s `world_variety` (including region
centres), and the settlement columns `parent_slot` and `founded_tick`. No interventions; the models are
action-blind.

## 2. Scores

The WM-1.2 harness, unchanged in its definitions (`evaluate.evaluate`, tasks T1, T2, T2_any; WM-1.2's numbers on
`p10_*` and v0 unchanged: the golden tests and a recomputation of `test.an5.T2_any["168"].test.mean` = 0.2064778167503853
on `p10_v16` pass, the latter in the DGX gate). V = food_stock, wood_stock, fire_lit, cold_members, alive,
deadwood_total, food_on_bushes_total, adults_working. nMAE = MAE / the std of each variable over rows ≥ 24 of the
scored split; mean over the 8 variables. T2 at 24 h and 168 h from **every-hour starts** (primary) and **06:00 starts**
(reported). **Bias-removed** score as WM-1.2 (per world and variable, the mean error over that world's scored starts
is subtracted).

**Per-world score** (GOAL_1_3.md 7.3): for a predictor and task, a test world's score is its mean over the 8 V of the
per-variable nMAE (with the split std of the full test split; bias-removed: of that world's de-meaned errors). Every
test world has 2,881 rows and the same starts (asserted), so the mean of the per-world scores is the pooled score.
A model's per-world score is the mean of its seeds' per-world scores.

**CI** = the paired world-level bootstrap 95 % interval: the 80 test worlds resampled with replacement, B = 2,000,
`numpy.random.default_rng(0)` (the same draws for every comparison on the same world set), 95 % percentile interval
of the mean over worlds of (model − baseline) (`wm1/bootstrap.py`). "Every seed" means every trained seed of the
configuration that exists.

## 3. Calendar family and selection rule

The nine members of WM-1.1 (`calcore.py`), fit on each dataset's train worlds (N = 80, 200, 400; analogs search
those worlds), selected on the shared **val** (80 worlds) per task by the mean nMAE (bias-removed tasks: the
bias-removed mean): **`cal_best`** = the lowest of the nine (ties: member order); **`cal_best_fair`** = the best of
{`cal_year`, `ap1_year`, `apfit_year`}; **`cal_best_anom`** = `cal_best` of a bias-removed task. Never re-selected on
test, never refit on train + val. References evaluated beside them: persistence and the observable ridge (λ on val T1).

Validation scores, `w13_v16_n80` (bold `cal_best`, "(fair)" `cal_best_fair`; 80 train worlds, 80 val worlds, ridge λ = 100.0):

| member | T1 | T2_24 | T2_168 | T2any_24 | T2any_168 | T2br_24 | T2br_168 | T2anybr_24 | T2anybr_168 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `cal_abs` | 0.3722 | 0.3000 | 0.3087 | 0.3550 | 0.3644 | 0.1393 | **0.1244** | 0.2149 | 0.2101 |
| `cal_year` | 0.3787 | 0.3092 | 0.3101 | 0.3636 | 0.3661 | 0.1503 | 0.1345 (fair) | 0.2252 | 0.2199 (fair) |
| `ap1_abs` | 0.1038 | 0.0957 | 0.1840 | 0.1847 | 0.2793 | 0.0993 | 0.1869 | 0.1885 | 0.2826 |
| `ap1_year` | 0.1038 | 0.0976 (fair) | 0.2019 | 0.1853 | 0.2962 | 0.1010 | 0.2035 | 0.1892 | 0.2983 |
| `apfit_abs` | 0.0940 | 0.0985 | 0.1745 | **0.1680** | 0.2368 | 0.0907 | 0.1497 | **0.1679** | 0.2319 |
| `apfit_year` | 0.0940 (fair) | 0.1006 | 0.1911 (fair) | 0.1697 (fair) | 0.2529 (fair) | 0.0922 (fair) | 0.1629 | 0.1688 (fair) | 0.2448 |
| `an1` | 0.2087 | 0.1650 | 0.1900 | 0.2383 | 0.2730 | 0.1558 | 0.1770 | 0.2357 | 0.2689 |
| `an5` | 0.1790 | 0.1370 | 0.1587 | 0.1996 | **0.2276** | 0.1152 | 0.1294 | 0.1826 | **0.2065** |
| `an5_off` | **0.0897** | **0.0873** | **0.1439** | 0.1693 | 0.2325 | **0.0892** | 0.1449 | 0.1716 | 0.2340 |
| persistence (reference) | 0.1447 | 0.1255 | 0.4594 | 0.2079 | 0.5741 | 0.1353 | 0.4671 | 0.2179 | 0.5821 |
| ridge (reference) | 0.1101 | 0.1807 | 0.3325 | 0.2287 | 0.3287 | 0.1468 | 0.2205 | 0.2112 | 0.2803 |

Validation scores, `w13_v16_n200` (bold `cal_best`, "(fair)" `cal_best_fair`; 200 train worlds, 80 val worlds, ridge λ = 100.0):

| member | T1 | T2_24 | T2_168 | T2any_24 | T2any_168 | T2br_24 | T2br_168 | T2anybr_24 | T2anybr_168 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `cal_abs` | 0.3683 | 0.2977 | 0.3055 | 0.3517 | 0.3609 | 0.1400 | **0.1243** | 0.2145 | 0.2092 |
| `cal_year` | 0.3750 | 0.3068 | 0.3083 | 0.3605 | 0.3640 | 0.1506 | 0.1327 (fair) | 0.2250 | 0.2180 (fair) |
| `ap1_abs` | 0.1028 | 0.0954 | 0.1838 | 0.1829 | 0.2782 | 0.0998 | 0.1867 | 0.1873 | 0.2815 |
| `ap1_year` | 0.1031 | 0.0974 (fair) | 0.1999 | 0.1845 | 0.2939 | 0.1016 | 0.2015 | 0.1887 | 0.2959 |
| `apfit_abs` | 0.0932 | 0.0988 | 0.1735 | 0.1669 | 0.2351 | 0.0914 | 0.1493 | 0.1675 | 0.2307 |
| `apfit_year` | 0.0934 (fair) | 0.1011 | 0.1883 (fair) | 0.1691 (fair) | 0.2498 (fair) | 0.0929 (fair) | 0.1606 | 0.1690 (fair) | 0.2423 |
| `an1` | 0.1913 | 0.1526 | 0.1808 | 0.2245 | 0.2657 | 0.1470 | 0.1704 | 0.2236 | 0.2626 |
| `an5` | 0.1609 | 0.1269 | 0.1491 | 0.1873 | **0.2179** | 0.1092 | 0.1253 | 0.1745 | **0.2016** |
| `an5_off` | **0.0854** | **0.0837** | **0.1361** | **0.1616** | 0.2207 | **0.0858** | 0.1368 | **0.1641** | 0.2222 |
| persistence (reference) | 0.1447 | 0.1255 | 0.4594 | 0.2079 | 0.5741 | 0.1353 | 0.4671 | 0.2179 | 0.5821 |
| ridge (reference) | 0.1097 | 0.1794 | 0.3320 | 0.2279 | 0.3276 | 0.1464 | 0.2205 | 0.2109 | 0.2794 |

Validation scores, `w13_v16_n400` (bold `cal_best`, "(fair)" `cal_best_fair`; 400 train worlds, 80 val worlds, ridge λ = 100.0):

| member | T1 | T2_24 | T2_168 | T2any_24 | T2any_168 | T2br_24 | T2br_168 | T2anybr_24 | T2anybr_168 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `cal_abs` | 0.3684 | 0.2983 | 0.3053 | 0.3521 | 0.3610 | 0.1404 | 0.1240 | 0.2144 | 0.2090 |
| `cal_year` | 0.3752 | 0.3075 | 0.3079 | 0.3610 | 0.3640 | 0.1510 | 0.1322 (fair) | 0.2249 | 0.2177 (fair) |
| `ap1_abs` | 0.1028 | 0.0954 | 0.1828 | 0.1822 | 0.2772 | 0.1002 | 0.1858 | 0.1867 | 0.2806 |
| `ap1_year` | 0.1031 | 0.0974 (fair) | 0.1986 | 0.1840 | 0.2930 | 0.1022 | 0.2004 | 0.1884 | 0.2952 |
| `apfit_abs` | 0.0931 | 0.0989 | 0.1726 | 0.1668 | 0.2349 | 0.0918 | 0.1493 | 0.1673 | 0.2307 |
| `apfit_year` | 0.0934 (fair) | 0.1013 | 0.1874 (fair) | 0.1692 (fair) | 0.2498 (fair) | 0.0936 (fair) | 0.1606 | 0.1689 (fair) | 0.2424 |
| `an1` | 0.1790 | 0.1445 | 0.1755 | 0.2161 | 0.2592 | 0.1411 | 0.1673 | 0.2171 | 0.2580 |
| `an5` | 0.1460 | 0.1155 | 0.1409 | 0.1755 | **0.2095** | 0.1045 | **0.1231** | 0.1676 | **0.1984** |
| `an5_off` | **0.0827** | **0.0820** | **0.1306** | **0.1574** | 0.2139 | **0.0849** | 0.1314 | **0.1600** | 0.2155 |
| persistence (reference) | 0.1447 | 0.1255 | 0.4594 | 0.2079 | 0.5741 | 0.1353 | 0.4671 | 0.2179 | 0.5821 |
| ridge (reference) | 0.1096 | 0.1772 | 0.3275 | 0.2268 | 0.3255 | 0.1454 | 0.2204 | 0.2100 | 0.2804 |

Validation scores, `w13_v17_n400` (bold `cal_best`, "(fair)" `cal_best_fair`; 400 train worlds, 80 val worlds, ridge λ = 100.0):

| member | T1 | T2_24 | T2_168 | T2any_24 | T2any_168 | T2br_24 | T2br_168 | T2anybr_24 | T2anybr_168 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `cal_abs` | 0.4231 | 0.3564 | 0.3574 | 0.4070 | 0.4121 | 0.1743 | 0.1613 | 0.2421 | 0.2374 |
| `cal_year` | 0.4479 | 0.3793 | 0.3745 | 0.4325 | 0.4332 | 0.1946 | 0.1778 | 0.2647 | 0.2581 |
| `ap1_abs` | 0.0839 | 0.0665 | 0.1698 | 0.1354 | 0.2514 | 0.0706 | 0.1702 | 0.1386 | 0.2522 |
| `ap1_year` | 0.0850 | 0.0688 (fair) | 0.1848 | 0.1376 | 0.2696 | 0.0730 | 0.1848 | 0.1408 | 0.2699 |
| `apfit_abs` | 0.0759 | 0.0695 | 0.1590 | 0.1293 | 0.2272 | 0.0702 | 0.1540 | 0.1319 | 0.2252 |
| `apfit_year` | 0.0770 (fair) | 0.0719 | 0.1730 (fair) | 0.1317 (fair) | 0.2439 (fair) | 0.0726 (fair) | 0.1662 (fair) | 0.1342 (fair) | 0.2402 (fair) |
| `an1` | 0.1709 | 0.1257 | 0.1590 | 0.1918 | 0.2304 | 0.1209 | 0.1521 | 0.1894 | 0.2276 |
| `an5` | 0.1364 | 0.0984 | 0.1242 | 0.1530 | **0.1827** | 0.0878 | **0.1120** | 0.1436 | **0.1731** |
| `an5_off` | **0.0716** | **0.0590** | **0.1113** | **0.1253** | 0.1848 | **0.0618** | 0.1141 | **0.1272** | 0.1873 |
| persistence (reference) | 0.1236 | 0.0892 | 0.3745 | 0.1582 | 0.4801 | 0.0953 | 0.3781 | 0.1642 | 0.4847 |
| ridge (reference) | 0.0921 | 0.1459 | 0.3043 | 0.1887 | 0.3024 | 0.1229 | 0.2186 | 0.1786 | 0.2700 |

Selection used by the Y rules (`cal_best` of a bias-removed task = `cal_best_anom`):

| dataset | task | `cal_best` | `cal_best_fair` |
|---|---|---|---|
| w13_v16_n80 | T2any_168 | `an5` | `apfit_year` |
| w13_v16_n80 | T2any_24 | `apfit_abs` | `apfit_year` |
| w13_v16_n80 | T2anybr_168 | `an5` | `cal_year` |
| w13_v16_n80 | T2anybr_24 | `apfit_abs` | `apfit_year` |
| w13_v16_n200 | T2any_168 | `an5` | `apfit_year` |
| w13_v16_n200 | T2any_24 | `an5_off` | `apfit_year` |
| w13_v16_n200 | T2anybr_168 | `an5` | `cal_year` |
| w13_v16_n200 | T2anybr_24 | `an5_off` | `apfit_year` |
| w13_v16_n400 | T2any_168 | `an5` | `apfit_year` |
| w13_v16_n400 | T2any_24 | `an5_off` | `apfit_year` |
| w13_v16_n400 | T2anybr_168 | `an5` | `cal_year` |
| w13_v16_n400 | T2anybr_24 | `an5_off` | `apfit_year` |
| w13_v17_n400 | T2any_168 | `an5` | `apfit_year` |
| w13_v17_n400 | T2any_24 | `an5_off` | `apfit_year` |
| w13_v17_n400 | T2anybr_168 | `an5` | `apfit_year` |
| w13_v17_n400 | T2anybr_24 | `an5_off` | `apfit_year` |

## 4. Configurations evaluated on test

Models (Part 4), each trained by `tools/worldmodel/wm1/w13_launch/slot.sh`:
`m1_tf.py train --dataset D --out /home/hyunlord/work/wm1-runs/<run> --variant sev-blind_action-blind --seed S
--batch 32 --clamp-nonneg 1 --eval-every 150 --k-max 168 --max-steps 9000 --ramp-steps 1500 --patience 12
--roll-min-context 167 --roll-starts 1 --tf-l1 1.0 --threads 8 --v-weight 5.0 --window 336 --max-seconds M`
(M = min(30000, the 36-hour deadline − launch time − 3,600)); r3 with the curriculum ramp held at WM-1.2's 1,500
steps, then training at k = 168 until patience (12 evaluations × 150 steps = 1,800 steps without improvement,
counted from step 1,500), 9,000 steps or M seconds. Early stopping and model selection on val T2 24 h at 06:00
starts. Queue: GOAL_1_3.md 9.1's pairs n400_s0 + n400_s1 → n400_s2 + v17_s0 → n80_s0 + n80_s1 → v17_s1 + n200_s0, run one at
a time in that order (throughput check above), so the droppable runs come last.

| run | dataset | seed | test evaluations (each once, `tools/worldmodel/wm1/w13_launch/finals.sh`) |
|---|---|---|---|
| `m1tf_w13v16_n400_s0`, `_s1`, `_s2` | `w13_v16_n400` | 0, 1, 2 | `final --run R`; `final --run R --ckpt best_wm12rule.pt --out R_short`; `final --run R --eval-dataset p10_v16 --out R_legacy` |
| `m1tf_w13v16_n80_s0`, `_s1` | `w13_v16_n80` | 0, 1 | `final --run R`; `--ckpt best_wm12rule.pt --out R_short` |
| `m1tf_w13v16_n200_s0` | `w13_v16_n200` | 0 | `final --run R`; `--ckpt best_wm12rule.pt --out R_short` |
| `m1tf_w13v17_n400_s0`, `_s1` | `w13_v17_n400` | 0, 1 | `final --run R`; probe of `_s0` (`probe_world.py --run R --out probe_R`) |
| WM-1.2's `m1tf_p10v16_r3_s0`, `_s1`, `_s2` | (trained on `p10_v16`) | 0, 1, 2 | `final --run R --eval-dataset w13_v16_n400 --out R_on_w13v16` |

Calendar family, persistence and ridge on test, once each: `climatology.py final --dataset D --selection
calendar_val_D_b7a48fed/calendar_val.json --out calendar_D_<sha8>` for the four datasets, and the legacy test
`climatology.py final --dataset w13_v16_n400 --eval-dataset p10_v16 --selection
calendar_val_w13_v16_n400_b7a48fed/calendar_val.json --out calendar_w13_v16_n400_on_p10_v16_<sha8>`.

A run that ends on `max_steps` or `time` with its best step in the last 10 % of its steps is "not converged" in
every table. A dropped run (GOAL_1_3.md 0.3.6) is reported as dropped; the rules then use the seeds that exist.

## 5. Rules

Computed by `judge_1_3.py` from the result files only.

| # | rule |
|---|---|
| Y1 | **`game_v16`, N = 400, raw.** M1b (`m1tf_w13v16_n400_s0..2`) vs `cal_best` = the member validation-selected at N = 400 for key `T2any_168` (`an5`), at T2 168 h every-hour starts: **KEEP** if the seed mean ≤ 0.8 × `cal_best`, every seed < `cal_best` and CI upper < 0; **WEAK** if CI upper < 0 but not KEEP; **TIE** if the CI contains 0; **NONE** if CI lower > 0. **Y1b**: the same at 24 h (`T2any_24`, `an5_off`). WEAK here needs an interval below 0, stricter than WM-1.2's point-estimate WEAK (Z1); every comparison with Z1 says so. The CI resamples worlds only; each seed's own paired CI is also reported. Reported beside Y1 with the same rule: N = 80 and N = 200 against their own `cal_best`. |
| Y2 | **`game_v16`, N = 400, bias-removed:** the Y1 rule on the bias-removed T2 168 h every-hour score against `cal_best_anom` (key `T2anybr_168`, `an5`). |
| Y3 | **data vs steps** (`game_v16` test, raw, 168 h every hour; CIs, no verdict): Δ_steps = M1b(long) − M1b(`best_wm12rule.pt`) on the two N = 80 runs (seed-averaged per world); Δ_data = M1b(N = 400, long, 3 seeds) − M1b(N = 80, long, 2 seeds); Δ_cal = `cal_best`(N = 400) − `cal_best`(N = 80) (each N's own selection). "Scale helps the model more than the calendar" if the CI of Δ_data − Δ_cal (per world) lies below 0. WM-1.2's three runs on the new test give `Δ_steps_xworld` = M1b(N = 80, long) − WM-1.2's M1b, labelled **confounded** (other training worlds, 16 val worlds, 3,000-step cap). |
| Y4 | **`game_v17`, N = 400:** M1b (`m1tf_w13v17_n400_s0`, `_s1`) vs `cal_best` (`T2any_168`, `an5`) raw and vs `cal_best_anom` (`T2anybr_168`, `an5`) bias-removed, the Y1 verdict rule. |
| Y5 | **learning curve** (reported): test score with CI vs N ∈ {80, 200, 400} for M1b and for `cal_best`, `an5`, `apfit_abs` (each fit on that N), raw and bias-removed, 168 h and 24 h every hour; per variable. |
| Y6 | **world identification on `game_v17`** (reported): WM-1.2's probe (`M1TF.hidden` at row 166, ridge with λ by val MSE, R² on test; `climate` accuracy) on `m1tf_w13v17_n400_s0` for every hidden parameter (`starting_agents`, `berry_bushes`, `berry_capacity`, `deadwood_spots`, `deadwood_capacity`, `forest_permille`, `food_regen_permille`, `wild_regions`, `region_bushes`, `region_deadwood`), plus labels read from observable columns after training: fission by day 120 (yes / no; accuracy against the train majority class), `settlements_alive` at row 2,832 (R²), and the first-fission day only if its test variance is above 0 — it is pinned to display day 29 in `game_v17`, so the calendar family can learn it. Beside the raw-row reference (37 columns). |

Also reported (no verdict): per-variable skill 1 − nMAE / baseline with intervals (same draws, ratio of means) for
food, alive, bushes, working adults (`game_v16`, N = 400; `game_v17` also `wood_stock`) at T2 168 h every hour, raw
against `cal_best` and `apfit_abs`, bias-removed against `cal_best_anom` and `apfit_abs`; the legacy test (the N =
400 models and the N = 400 family on WM-1.2's 16 test worlds, beside WM-1.2's numbers); the full raw and
bias-removed tables per dataset, size and horizon (members, `cal_best`, `cal_best_fair`, `cal_best_anom`,
`apfit_abs`, persistence, ridge, each seed, seed mean, with CIs).

## 6. Limitations known before the test numbers

- GPU reproducibility of M1b is not measured.
- 80 test worlds give a world-level interval; the starts within a world are not independent and are not resampled.
- The calendar family's selection at each N uses the shared 80 val worlds; the model's early stopping uses the same
  val worlds, so both sides are tuned on val identically.
- `game_v17`'s fission is pinned to display day 29 (Pass 11); its splits differ by world in whether and how often
  they happen, not when.

## Amendment 2026-10-13 22:25 (review round 1, before any `game_v17` model or probe number)

At the time of this amendment the test numbers that exist are the calendar family / persistence / ridge on the four
w13 datasets and the legacy test, WM-1.2's three runs on the new test, and `m1tf_w13v16_n400_s0` / `_s1` with their
`_short` and `_legacy` evaluations (all from `166eb737`). No `game_v17` model, no probe and no N = 80 / 200 model has
a test number. No rule, member, threshold, score definition or selection changes.

1. **Fission timing (corrects sections 5 Y6 and 6).** "Pinned to display day 29" is wrong for the recorded worlds:
   the first split falls on display days 29–36 (train: 155 of 243 splitting worlds on day 29, variance 1.50; test:
   34 of 53 on day 29, 18 on days 30–36, one world on day 85, variance 58.3). Fission is *eligible* from day 29; whether
   and how often a world splits varies, its date varies little. The Y6 first-fission probe runs (its test variance is
   above 0) and additionally reports the R² on the test worlds whose first split lies inside the train range of
   first-split days (`r2_test_inside_train_range`, `test_outside_train_range`; `probe_world.py`), so that the one
   day-85 world does not decide the number alone.
2. **Disclosure.** `w13_describe_b7a48fed` (GOAL_1_3.md 6.4: η², R²_abs, dyn-η² of the test split with Pass 11's
   `scene_block`, and the fission labels of every role) ran at 12:10–12:12, before this file's commit. It scores no
   forecaster of the Y rules and selects nothing; it is the data description the goal asks for.
3. **Convergence flag.** "Best step in the last 10 %" is read inclusively: a run that ended on `max_steps` or `time`
   is not converged when best step ≥ 0.9 × last step (DECISIONS G4.4). The flag is shown in the Y, Y5 and convergence
   tables (`judge_1_3.py`).
4. **Launcher.** `w13_launch/slot.sh` gained a watchdog after a hung CUDA context (DECISIONS G4.3); the training
   command and every hyper-parameter are unchanged.
5. **Code of the later test jobs.** The test evaluations still to run (`n400_s2`, the `game_v17` runs and the probe,
   the N = 80 and N = 200 runs) run from the commit that adds this amendment; its diff to `166eb737` under `tools/`
   is `probe_world.py` (the extra first-fission output above), `judge_1_3.py` (convergence flags and per-seed
   intervals in the markdown), `test_w13.py` and `w13_launch/slot.sh`. `evaluate.py`, `m1_gru.py`, `m1_tf.py`,
   `data.py`, `common.py`, `climatology.py` and `bootstrap.py` are byte-identical, so every model score is computed by
   the same code as before.
