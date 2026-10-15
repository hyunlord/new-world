# WM-2 metrics and pre-registration

Pre-registration of GOAL.md section 8, written **before any test-role number of a new model exists**: at the time of
this commit no `metrics.json`, `effects.json`, `split.json` or `floor.json` of any WM-2 run exists, and no test-role
number of any baseline on dataset v2 exists. At the time of writing the first run (`wm2_m1bact_v19_s0`) is training on the DGX (launched 2026-10-15 02:14:09 KST from code `a34c8396`; it reads the train and validation roles only) and the noise-floor families are being recorded beside it (4 workers).

- Code that computes every test number below: the commit that adds this file (recorded on the DGX in `wm2/_setup/prereg_sha`).
  It changes nothing under `tools/` relative to the training code commit `a34c8396`. Any later change to the scoring
  code before the test numbers exist gets a dated amendment at the end of this file.
- Validation-only numbers that exist (no test role read): `calendar_val_v19`, `calendar_val_v16` (calendar family,
  persistence and ridge on validation; copies in `runs/`), the cost check (`runs/cost/`), the training runs' validation logs.

## 1. Data and roles

`docs/worldmodel/wm2/v2_manifest.json` (sha256 `4182f75a…`, 5,876 episodes: 5,316 family members — 560 worlds, 489 with the two `sp…L` arms — and 560 control episodes) — the `game_v19` families and the `game_v16` control — and
`v2_floor_manifest.json` (the noise-floor families, recorded beside the first training; DECISIONS H1.1). Recorder
`worldsim_wm` built at the merge commit `26b9af1f` (identical to Pass 14's Stage A episodes, Part 0), every episode
verified, `meta.git_sha` = `26b9af1f…`. Roles (GOAL.md 4.2, fixed before recording):

| scene | train | val | test |
|---|---|---|---|
| `game_v19` | 40000–40399 | 40400–40479 | 40480–40559 |
| `game_v16` (control, factual) | 42000–42399 | 42400–42479 | 42480–42559 |

A `game_v19` **family** = one world: the factual `none` and the arms `sp240L`, `sp120L`, `gf240E`, `rw120E`
(`schedules_p14.py`, cast times from the world's factual probe; worlds without `T_low` lack the `sp…L` arms), each as
replicates 0 and 1 forked at tick 14,280. Members share rows <= row(14,280); arm r shares rows <= row(c) with `none_r`
(checked for every family, `families_v2.py check`).

Datasets (`common.DATASETS`): **`v2_v19`** — train: every member of every train family; validation (early stopping, the
rounding choice): the replicate-0 members (factual and arms) of the val families; test: every member of the test
families, read through `wm2/effects.py`. **`v2_v19_fact`** — replicate 0 of `none` per role (every factual baseline fit,
W1, W2). **`v2_v16`** — the control (factual only).

Inputs (asserted in `test_wm2.py`): `game_v19` = WM-1.3's `game_v17` list exactly (67: the 35 observable state columns,
`g.settlements_alive`, `g.fissions_total`, the 30 calendar features), plus the 7 action columns (`data.ACTION_NAMES`,
the requested amount at the cast row / the kind's grid maximum) for the action-aware variant (74). `game_v16` = WM-1.2's
65. Never inputs: `fission_pressure`, `fission_threshold`, `world_variety`, `latent.npy`, `depart_tick`, the settlement
slot columns (`parent_slot`, `founded_tick`), `winter_severity` and the other hidden columns of `data.HIDDEN_V2`.

## 2. Scores shared with WM-1.3

T2 (raw and bias-removed), every-hour starts (primary) and 06:00 starts, nMAE = MAE / the std of each of the 8 V over
rows >= 24 of the scored split, mean over the 8 V; per-world scores; `cal_best` / `cal_best_anom` validation-selected per
task among the nine calendar members (fit on the dataset's train worlds); `apfit_abs`, `an5`, persistence, the
observable ridge (λ on validation T1) — all exactly WM-1.3's code (`evaluate.py`, `climatology.py`, `calcore.py`).
**CI** = the world-level paired bootstrap 95 % percentile interval (test worlds resampled with replacement, B = 2,000,
`numpy.random.default_rng(0)`; `wm1/bootstrap.py`; for the effect statistics `wm2/effects.compare`, which recomputes
the cell-averaged statistic per draw from per-world sums, both replicates of a world moving together).

## 3. The noise floor (GOAL.md 7.3, `wm2/floor.py`)

For each `game_v19` (and `game_v16`) test world, start s ∈ {20,040 (day 15, 06:00), 56,040 (day 40), 99,240 (day 70)}:
a factual family forked at s with R = 4 replicates. For h ∈ {24, 168} hours and each V variable j: replicate values
y_r at row(s) + h; floor error of replicate r = |y_r − mean of the other replicates| × √((R − 1)/R) (uncorrected value
recorded too) / the test split std of j (of `v2_v19_fact` / `v2_v16` test). **Floor score** = mean over worlds, starts,
variables, replicates. It estimates the error of the conditional-*mean* predictor — exactly in expectation only for
Gaussian noise (an approximation for integer columns such as `alive`); the MAE-optimal predictor is the conditional
median, so the true MAE floor is **at or below** this estimate and **headroom** (a predictor's score − the floor) is a
**lower bound**. **Floor-matched score**: each predictor rolled from row(s) of replicate 0 (the factual) for h hours,
|prediction − y_0| / std, averaged the same way. Headroom = `cal_best` (validation-selected for `T2any_h`) − floor, with
the paired world bootstrap interval.

## 4. Effects (GOAL.md 7.4–7.5, `wm2/effects.py`)

A **case** = (test world, arm, replicate r), cast tick c (`interventions.json`, a multiple of 60). The predictor starts
at row(c) of `none_r` and rolls the factual (no action) and the arm (the arm's requested amount at row(c), entering the
transition c → c + 60) to the last row (row 2,880). **True effect** = arm_r − none_r; **predicted effect** = predicted arm
− predicted factual from the same start. Outcomes at c + 7 days and c + 30 days (rows + 168, + 720): `food_stock`
(summed store), `alive`, cumulative deaths (the five causes `deaths_starvation`, `deaths_dehydration`, `deaths_combat`,
`deaths_old_age`, `deaths_exposure`), `g.settlements_alive`; a case whose c + 30 days is past the last row is excluded
from the 30-day outcomes (counted per arm). **Split event** after a reference row = the first later row at which
`g.fissions_total` exceeds its reference-row value by >= 0.5 (truth: integer series; prediction: the continuous rolled-out
state). **Restricted split day** = min(display day of the split event, display day of the last row = 121; DECISIONS
H2.2), censored when there is none. **Split shift** = restricted split day (reference row(c)) of the arm − of the factual.

Metrics per arm, outcome and horizon: **effect MAE** = mean over cases of |predicted − true| / the std over train cases
of that arm's true effect (fallback: all arms' train cases; < 1e-9: cell dropped and recorded); **sign accuracy** over
cases with a true effect ≠ 0; split shift: MAE in days, **right-direction share** over cases with a true shift ≠ 0,
censored shares; **between-replicate effect spread** |e_0 − e_1| / √2 / std (it is not a floor: the two replicates start
the cast from different states, so it bounds the noise a predictor that sees each replicate's history must pay only
from above).

Baselines: `zero`; `arm_mean` (mean train effect at each lag; mean train shift); `an5_arm` (the 5 nearest train cases of
the same arm, at most one per world — the nearer replicate — by `an5`'s distance over the 24-row window of the 8 V ending
at the start row, scaled by the factual train split std, each candidate compared at its own cast row; the mean of their
true effects and shifts; DECISIONS H2.6). **One** baseline per rule, chosen on the **validation families** (both
replicates) by the rule's own statistic, never per cell.

Drift (diagnostic): T2 nMAE of the factual rollout at 7, 30 and 45 days after the start.

## 5. Split timing (GOAL.md 7.7, `wm2/split_task.py`)

Test worlds (replicate 0 of `none`) with `g.fissions_total` = 0 at tick 34,440 (display day 25, 06:00); target = the
restricted split day (reference = the start row). Predictors: M1-long and M1b-act (rollout to the last row); baselines
fit on the eligible train worlds: `median_day`, `an5_day` (5 nearest by `an5`'s distance at the start row), `ridge_day`
(ridge from the 24 × 8 window, λ from `bench_v0.LAMBDAS` on validation MAE); the best of the three on validation is the
W5 baseline. Metric: MAE in days, share within ± 7 days. θ (`fission_threshold`) is read only after every test
prediction exists (DECISIONS H2.7).

## 6. Models and runs (GOAL.md 7.6, 9.1)

- **M1b-act** (`m1_tf.py`, action-aware): `--variant sev-blind_action-aware --dataset v2_v19 --window 336 --context 168
  --warmup 168 --ramp-steps 1500 --eval-every 150 --patience 12 --batch 32 --clamp-nonneg 1 --k-max 168
  --roll-min-context 167 --roll-starts 1 --tf-l1 1.0 --threads 8 --v-weight 5.0 --max-steps 12000 --val-batch 512` (validation on every start, batched across episodes; DECISIONS H4.1).
- **M1-long** (`m1_gru.py --burn-in full`, action-aware): the same flags where they apply (hidden 256, 2 layers),
  `--burn-in full --max-steps 12000` (= min(12,000, ⌊0.95 × 21,600 / 0.407 s/step⌋) = 12,000; 0.407 s/step = the cost check's 0.396 s/step at k = 168 plus one validation pass of 1.6 s per 150 steps).
- **M1b (control)** (`m1_tf.py`, action-blind, `--dataset v2_v16`): the M1b-act flags with `--variant sev-blind_action-blind`.
- `--max-seconds` = min(21,600, deadline − now − 3,600) at each launch (the queue re-reads the deadline).
- Runs, in order: `wm2_m1bact_v19_s0`, `wm2_m1long_v19_s0`, `wm2_m1bact_v19_s1`, `wm2_m1long_v19_s1`,
  `wm2_m1bact_v19_s2`, `wm2_m1b_v16_s0`; drop order (GOAL.md 0.3.6): `wm2_m1b_v16_s0` → `wm2_m1long_v19_s1` →
  `wm2_m1bact_v19_s2`. "Not converged" = best step >= 0.9 × last step on a `max_steps` / `time` stop.
- Test evaluations of every trained run, once, from this commit (`finals.sh`): v2_v19 runs — `m1_gru.py final
  --eval-dataset v2_v19_fact` (factual T2), `effects.py model`, `split_task.py model`; the control — `final` on
  `v2_v16`; all runs — the floor-matched scores (`floor.py`) after the floor families exist.

## 7. Rules

"CI" = world-level paired bootstrap 95 % interval of (model − baseline); the model side is the mean over its trained
seeds (per-world scores, or per-case errors for the effect statistics); "every seed" = every trained seed of the
configuration. **The W1 rule**: KEEP if the seed mean <= 0.8 × the baseline, every seed below the baseline and CI upper
< 0; WEAK if CI upper < 0 (not KEEP); TIE if the CI contains 0; NONE if CI lower > 0. A configuration with one trained
seed can reach WEAK at most, never KEEP.

| # | rule |
|---|---|
| W1 | **factual, raw**: `v2_v19_fact` test (replicate 0 of `none`, 80 worlds), T2 168 h every-hour starts: M1b-act vs `cal_best` (validation-selected for `T2any_168` on `v2_v19_fact`): the W1 rule. **W1b**: the same at 24 h (`T2any_24`). |
| W2 | **floor** (reported, no verdict): floor-matched scores of M1b-act, M1-long, `cal_best`, `an5`, persistence and the floor at 24 h and 168 h on `game_v19`; on `game_v16` the same for the v16 M1b, `cal_best`, `an5`, persistence; headroom = `cal_best` − floor with its interval. If headroom at 168 h < 0.02, the analysis says "factual forecasting is within 0.02 of the conditional-mean floor on game_v19" (not "saturated": headroom is a lower bound). |
| W3 | **effects — the primary claim**: test families, both replicates; the scaled effect MAE averaged over the four outcomes at c + 30 days and the arms `sp240L`, `gf240E`, `rw120E` (12 cells, equally weighted; a dropped cell is left out and recorded): M1b-act vs **one** baseline — the member of {`zero`, `arm_mean`, `an5_arm`} with the lowest value of this statistic on the validation families: the W1 rule. **W3-long**: the same for M1-long. The between-replicate spread of the same cells is reported beside it. |
| W4 | **split shift** (`sp240L` and `gf240E`, restricted days, both replicates; the two arms' shift MAEs in days averaged): M1b-act vs the one member of {`zero`, `arm_mean`, `an5_arm`} best on validation by the same average: the W1 rule; right-direction shares and censored shares reported (M1-long reported as W4-long, no extra claim). |
| W5 | **memory** (split timing, restricted split-day MAE, eligible test worlds): **W5** = M1-long vs the one member of {`median_day`, `an5_day`, `ridge_day`} best on validation: the W1 rule. **W5b** = M1-long vs M1b-act (seed means): the W1 rule — the memory comparison proper. |
| W6 | **control** (`v2_v16` test, reported): M1b (action-blind) vs `cal_best` (`T2any_168` on `v2_v16`), raw T2 168 h — WM-1.3's Y1 replicated on fresh worlds (one seed: WEAK at most). |

The verdicts are computed by `wm2/judge_wm2.py` from the run files; nothing is re-selected on test.

## 8. Validation scores (no test role read)

`cal_best` per task is the bold entry; W1 uses `T2any_168` / `T2any_24`, W2's headroom the same keys, W6 `T2any_168` on `v2_v16`.

Validation scores, `v2_v19_fact` (bold `cal_best`, "(fair)" `cal_best_fair`; 400 train worlds, 80 val worlds, ridge λ = 100.0):

| member | T1 | T2_24 | T2_168 | T2any_24 | T2any_168 | T2br_24 | T2br_168 | T2anybr_24 | T2anybr_168 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `cal_abs` | 0.4237 | 0.3463 | 0.3472 | 0.4070 | 0.4119 | 0.1870 | 0.1745 | 0.2596 | 0.2581 |
| `cal_year` | 0.4612 | 0.3813 | 0.3756 | 0.4456 | 0.4453 | 0.2187 | 0.2012 | 0.2959 | 0.2904 |
| `ap1_abs` | 0.0883 | 0.0617 | 0.1609 | 0.1287 | 0.2478 | 0.0664 | 0.1611 | 0.1316 | 0.2487 |
| `ap1_year` | 0.0894 | 0.0630 (fair) | 0.1752 | 0.1301 | 0.2639 | 0.0677 (fair) | 0.1736 | 0.1330 | 0.2625 |
| `apfit_abs` | 0.0769 | 0.0642 | 0.1547 | 0.1222 | 0.2248 | 0.0663 | 0.1471 | 0.1256 | 0.2223 |
| `apfit_year` | 0.0779 (fair) | 0.0659 | 0.1674 (fair) | 0.1238 (fair) | 0.2390 (fair) | 0.0677 | 0.1579 (fair) | 0.1271 (fair) | 0.2342 (fair) |
| `an1` | 0.1666 | 0.1251 | 0.1458 | 0.1868 | 0.2216 | 0.1249 | 0.1366 | 0.1854 | 0.2187 |
| `an5` | 0.1336 | 0.0945 | 0.1159 | 0.1485 | 0.1791 | 0.0847 | 0.1015 | 0.1405 | **0.1699** |
| `an5_off` | **0.0728** | **0.0554** | **0.0995** | **0.1203** | **0.1780** | **0.0591** | **0.1012** | **0.1225** | 0.1807 |
| persistence (reference) | 0.1260 | 0.0809 | 0.3555 | 0.1481 | 0.4589 | 0.0886 | 0.3604 | 0.1544 | 0.4652 |
| ridge (reference) | 0.0924 | 0.1442 | 0.2918 | 0.1851 | 0.2941 | 0.1229 | 0.2030 | 0.1766 | 0.2626 |

Validation scores, `v2_v16` (bold `cal_best`, "(fair)" `cal_best_fair`; 400 train worlds, 80 val worlds, ridge λ = 100.0):

| member | T1 | T2_24 | T2_168 | T2any_24 | T2any_168 | T2br_24 | T2br_168 | T2anybr_24 | T2anybr_168 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `cal_abs` | 0.3646 | 0.2934 | 0.2997 | 0.3475 | 0.3574 | 0.1358 | **0.1180** | 0.2079 | 0.2021 |
| `cal_year` | 0.3707 | 0.3010 | 0.3005 | 0.3554 | 0.3587 | 0.1470 | 0.1293 (fair) | 0.2198 | 0.2142 (fair) |
| `ap1_abs` | 0.1026 | 0.0986 | 0.1804 | 0.1841 | 0.2751 | 0.1046 | 0.1830 | 0.1885 | 0.2781 |
| `ap1_year` | 0.1032 | 0.1004 (fair) | 0.1984 | 0.1865 | 0.2932 | 0.1060 | 0.1994 | 0.1907 | 0.2947 |
| `apfit_abs` | 0.0932 | 0.0995 | 0.1665 | 0.1665 | 0.2303 | 0.0945 | 0.1446 | 0.1673 | 0.2254 |
| `apfit_year` | 0.0937 (fair) | 0.1017 | 0.1833 (fair) | 0.1695 (fair) | 0.2470 (fair) | 0.0955 (fair) | 0.1581 | 0.1694 (fair) | 0.2392 |
| `an1` | 0.1771 | 0.1413 | 0.1765 | 0.2152 | 0.2591 | 0.1393 | 0.1700 | 0.2174 | 0.2598 |
| `an5` | 0.1419 | 0.1112 | 0.1377 | 0.1713 | **0.2056** | 0.1055 | 0.1239 | 0.1676 | **0.1987** |
| `an5_off` | **0.0832** | **0.0846** | **0.1305** | **0.1595** | 0.2135 | **0.0887** | 0.1307 | **0.1621** | 0.2151 |
| persistence (reference) | 0.1433 | 0.1302 | 0.4644 | 0.2134 | 0.5829 | 0.1403 | 0.4707 | 0.2228 | 0.5895 |
| ridge (reference) | 0.1103 | 0.1731 | 0.3099 | 0.2240 | 0.3149 | 0.1445 | 0.2180 | 0.2102 | 0.2793 |


## Amendment 1 (2026-10-15 20:25 KST, review round 1, R8)

After the pre-registration commit `b57ef7bb` the following files under `tools/` changed; none of them computes a test
score or a verdict differently: `wm2/judge_wm2.py` (`149e8fe7`, 14:31) adds two report-only fields per effect cell
(`mean_pred`, `mean_true`); `wm2/diag_fission.py` and `wm2/diag_effect_lag.py` are new validation-only diagnostics;
`wm2/launch/floor.sh` and `floor_post.sh` are launch scripts. Every test number was computed by code at `b57ef7bb`
(`floor.py`, `effects.py`, `split_task.py`, `m1_gru.py final`, `climatology.py final` ran from the `b57ef7bb` worktree).
