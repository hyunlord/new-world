# WM-2: dataset v2 on `game_v19`, the noise floor, and whether a learned world model predicts what the god's hand will do

Run of `docs/worldmodel/wm2/GOAL.md` on `codex/worldmodel-2026-10-wip` (base `ce40bf3c`, WM-1.3 final), after merging the
simulation's green head after Pass 14 (`f5373ab2` → merge `26b9af1f`: `game_v19`, recorder schema 6, the Pass 14 tools).
Pre-registration: `METRICS.md` (`b57ef7bb`, committed 2026-10-15 02:15:36 KST, before any test-role number). Every
number below comes from `tools/worldmodel/wm2/judge_wm2.py` (`runs/judge_wm2/judge.md`, `judge.json`) or the run files
under `runs/`; intervals are world-level paired bootstrap 95 % intervals over the 80 test worlds (B = 2,000,
`default_rng(0)`). Dataset: `DATASET_V2.md`. Decisions: `DECISIONS.md`. Timeline: `RUNLOG.md`.

## 1. Answer

- **Q1 — the floor (W2).** On `game_v19` factual worlds the conditional-mean noise floor (four replicates forked at
  days 15, 40 and 70, 06:00) is **0.008 at 24 h and 0.020 at 168 h** (mean nMAE of the 8 V). At exactly those starts
  the validation-selected calendar member (`an5_off`) scores 0.041 and 0.080: **headroom +0.032 [+0.029, +0.035] at 24 h
  and +0.059 [+0.051, +0.067] at 168 h** — a lower bound (the MAE floor is at or below the conditional-mean floor).
  Factual forecasting on `game_v19` is **not** within 0.02 of the floor. The floor families share the whole simulator
  state at the fork (hidden θ, `latent`, `world_variety`, agent state), so this is the floor of a forecaster that knows
  the full state: headroom bounds the gap to such a forecaster from below, not what a forecaster that sees only the
  observable history can reach (section 5 shows that one hidden quantity, θ, carries a large part of the split timing). On the `game_v16` control the floor is 0.017 / 0.034 and the headroom +0.054 [+0.049, +0.060] / +0.084 [+0.077, +0.091]: the control's floor is higher in nMAE units than `game_v19`'s (its post-fork noise is larger relative to its spread), and it is still far from saturated.
- **Q1 — factual forecasting (W1, W1b).** The action-aware transformer M1b-act (3 seeds) beats `an5_off` on raw T2
  every-hour starts of the 80 factual test worlds at both horizons with an interval below 0 — 168 h: 0.169 vs 0.178,
  −0.010 [−0.014, −0.006]; 24 h: 0.111 vs 0.123, −0.012 [−0.014, −0.010] — but far from the 20 % KEEP margin:
  **W1 WEAK, W1b WEAK** (every seed's own interval is below 0). At the three floor starts it does not beat
  `an5_off` (168 h 0.098–0.109 vs 0.080). The long-memory GRU (M1-long, 2 seeds) is clearly worse on factual T2
  (168 h 0.226, +0.048 [+0.044, +0.053] vs `an5_off`).
- **Q2 — effects, the primary claim (W3, W4).** Neither learned model predicts the effect of the god's acts better than
  predicting **no effect at all**: on the pre-registered statistic (scaled effect MAE at c + 30 days, 4 outcomes × 3
  arms) M1b-act 0.501 vs `zero` 0.487, +0.014 [+0.006, +0.027], **W3 NONE**; M1-long 0.574, +0.087 [+0.057, +0.121],
  **W3-long NONE**. On the split shift both models predict 0 days for every case (they never predict a split), so they
  equal the zero predictor: 12.21 days vs the validation-chosen `an5_arm` 12.49, −0.29 [−2.43, +1.97], **W4 TIE** (and
  W4-long TIE), with a right-direction share of 0.
- **Q3 — memory (W5, W5b).** No model ever predicts a split from day 25 within the 96 remaining days, so every model
  predicts the censoring day for every world: split-day MAE 40.1 days vs `ridge_day` 29.2, +10.9 [+3.7, +17.6],
  **W5 NONE**; M1-long and M1b-act make identical predictions, **W5b TIE** with a degenerate interval [0, 0]. Long
  memory does not let the model anticipate the split.
- **Control (W6).** The action-blind M1b trained on 400 fresh `game_v16` worlds (one seed) ties the validation-selected `an5` at 168 h: 0.210 vs 0.207, +0.002 [-0.003, +0.007], **W6 TIE**; at 24 h it beats `an5_off` (0.133 vs 0.155, -0.021 [-0.023, -0.019], the rule would give WEAK; reported). WM-1.3's Y1 (three seeds, `an5` 0.207, −0.013 [−0.019, −0.008], WEAK) is therefore not replicated at 168 h by this one seed on fresh worlds; the analog baseline scored the same on fresh worlds (0.207 both times).
- **Why the models fail on effects and splits (diagnostics, validation only, not pre-registered).** The models do
  not learn even the mechanical one-hour effect of an act: one hour after a gift of 240 the true store is +237, the
  models predict between +6 and −19; after a spoil of 240, −222 true against −13 to −45 predicted
  (`runs/diag/effect_lag.json`). Their rolled-out `fissions_total` never rises by more than 0.0006 (M1b-act) or 0.11
  (M1-long) above its start in 96 days, while 72 % of the eligible validation worlds split (a true rate of 4.4 × 10⁻⁴
  per hour; `runs/diag/fission_s0.json`). Cast rows are 0.03 % of the training rows and a split is a single +1 step
  in 2,880 rows: a mean (L1-weighted) next-hour predictor treats both as noise. The data contain the effects (the
  arm-mean and analog effect baselines predict the 7-day effects well, section 4); the training recipe does not
  extract them. (Diagnostics: `effect_lag` uses the four s0 / s1 runs on 40 validation worlds, `fission_s0` the two
  s0 runs on 79.) The action input is wired correctly — training and evaluation feed the requested amount at the cast
  row — yet the models do not even tell the kinds apart: one hour after rotted wood every model predicts a food drop
  (true food effect 0), and three of four predict one after a gift of 240 (true +237).

## W table (METRICS.md 7)

| # | comparison | model (seeds) | baseline | model − baseline [95 % CI] | each seed − baseline | verdict |
|---|---|---|---|---|---|---|
| W1 | W1 factual raw T2 168 h (`v2_v19_fact`) | 0.169 (0.172, 0.171, 0.164) | `an5_off` 0.178 | -0.010 [-0.014, -0.006] | -0.007 [-0.011, -0.003]; -0.008 [-0.012, -0.003]; -0.014 [-0.019, -0.010] | **WEAK** |
| W1b | W1b factual raw T2 24 h | 0.111 (0.113, 0.112, 0.108) | `an5_off` 0.123 | -0.012 [-0.014, -0.010] | -0.010 [-0.012, -0.007]; -0.011 [-0.013, -0.009]; -0.015 [-0.017, -0.013] | **WEAK** |
| W3 | W3 effects, M1b-act (scaled MAE, 30 d, 12 cells) | 0.501 (0.510, 0.491, 0.503) | `zero` 0.487 | +0.014 [+0.006, +0.027] | — | **NONE** |
| W3-long | W3-long effects, M1-long | 0.574 (0.519, 0.629) | `zero` 0.487 | +0.087 [+0.057, +0.121] | — | **NONE** |
| W4 | W4 split shift, M1b-act (days) | 12.21 (12.21, 12.21, 12.21) | `an5_arm` 12.49 | -0.29 [-2.43, +1.97] | — | **TIE** |
| W4-long | W4-long split shift, M1-long (reported) | 12.21 (12.21, 12.21) | `an5_arm` 12.49 | -0.29 [-2.43, +1.97] | — | **TIE** |
| W5 | W5 split timing, M1-long vs best day baseline (days) | 40.11 (40.11, 40.11) | `ridge_day` 29.21 | +10.90 [+3.72, +17.64] | +10.90 [+3.72, +17.64]; +10.90 [+3.72, +17.64] | **NONE** |
| W5b | W5b split timing, M1-long vs M1b-act | 40.11 (40.11, 40.11) | `M1b-act (seed mean)` 40.11 | +0.00 [+0.00, +0.00] | +0.00 [+0.00, +0.00]; +0.00 [+0.00, +0.00] | **TIE** |
| W6 | W6 control `game_v16` raw T2 168 h (reported) | 0.210 (0.210) | `an5` 0.207 | +0.002 [-0.003, +0.007] | +0.002 [-0.003, +0.007] | **TIE** |

Beside W3: the between-replicate effect spread of the same 12 cells is 0.189; the other baselines score `arm_mean` 0.619, `an5_arm` 0.594. Beside W4: spread 1.56 days; `zero` 12.21, `arm_mean` 12.30; right-direction share of every model 0.00 (no predicted shift); truth censored in the arm / factual: `sp240L` 0.27 / 0.37, `gf240E` 0.43 / 0.42; predictions censored 1.00 for every model.

The KEEP margin is 0.8 × the baseline; "every seed" uses the trained seeds (M1b-act 3, M1-long 2, control 1). A one-seed
configuration can reach WEAK at most.

## 2. Runs and convergence

| run | stop | best step / last | converged | s/step at k = 168 (incl. validation) | validation pass (s) | seconds | best val T2 24 h |
|---|---|---|---|---:|---:|---:|---:|
| `wm2_m1b_v16_s0` | patience | 8400 / 10200 | yes | 1.54 | 1.8 | 15637 | 0.0769 |
| `wm2_m1bact_v19_s0` | patience | 8550 / 10350 | yes | 1.58 | 7.5 | 15729 | 0.0608 |
| `wm2_m1bact_v19_s1` | patience | 9900 / 11700 | yes | 1.58 | 7.7 | 17966 | 0.0603 |
| `wm2_m1bact_v19_s2` | max_steps | 10950 / 12000 | no | 1.56 | 7.6 | 18288 | 0.0589 |
| `wm2_m1long_v19_s0` | max_steps | 10950 / 12000 | no | 0.41 | 1.8 | 5112 | 0.0774 |
| `wm2_m1long_v19_s1` | max_steps | 11700 / 12000 | no | 0.38 | 1.7 | 4841 | 0.0782 |

M1b-act s0 and s1 stopped on patience; s2 and both M1-long runs reached the 12,000-step cap with their best step in
the last 10 % ("not converged"). M1-long costs 0.40 s/step against M1b-act's 1.56 s/step, so it got a quarter of the
compute; its validation score was still falling (0.0774 / 0.0782 at the cap vs M1b-act's 0.059–0.061). The validation
pass was batched across episodes (`--val-batch 512`, DECISIONS H4.1); no start subsample was needed.

## 3. Factual forecasting and the floor (W1, W1b, W2)

Raw T2, every-hour starts, `v2_v19_fact` test (replicate 0 of `none`, 80 worlds):

| predictor | T2 24 h | T2 168 h |
|---|---:|---:|
| M1b-act (s0, s1, s2) | 0.111 (0.113, 0.112, 0.108) | 0.169 (0.172, 0.171, 0.164) |
| M1-long (s0, s1) | 0.141 | 0.226 |
| `an5_off` (`cal_best`, both horizons) | 0.123 | 0.178 |
| `an5` | 0.148 | 0.179 |
| `apfit_abs` | 0.128 | 0.232 |
| persistence | 0.150 | 0.463 |
| ridge | 0.189 | 0.302 |

Each M1b-act seed − `an5_off`: 168 h −0.007 [−0.011, −0.003], −0.008 [−0.012, −0.003], −0.015 [−0.019, −0.010];
24 h −0.010 [−0.012, −0.008], −0.011 [−0.013, −0.009], −0.015 [−0.017, −0.013]. The margin over the analog forecaster
(5 % at 168 h, 10 % at 24 h) is smaller than WM-1.3's on `game_v16` (6 % and 16 %), on worlds that split.

**Floor** (`runs/floor_v19/floor.json`; METRICS.md 3). The floor estimates the error of the predictor that knows the
mean over the noise after the start; exactly so only for Gaussian noise; the MAE floor is at or below it, so headroom is
a lower bound.

| scene | horizon | floor (uncorrected) | `cal_best` | headroom [95 % CI] | M1b-act / M1b seeds | M1-long seeds | `an5` | persistence |
|---|---|---|---|---|---|---|---|---|
| `game_v19` | 24 h | 0.008 (0.010) | `an5_off` 0.041 | +0.032 [+0.029, +0.035] | 0.051, 0.045, 0.046 | 0.070, 0.068 | 0.087 | 0.087 |
| `game_v19` | 168 h | 0.020 (0.023) | `an5_off` 0.080 | +0.059 [+0.051, +0.067] | 0.109, 0.101, 0.098 | 0.201, 0.218 | 0.107 | 0.378 |
| `game_v16` | 24 h | 0.017 (0.020) | `an5_off` 0.071 | +0.054 [+0.049, +0.060] | 0.069 | — | 0.109 | 0.167 |
| `game_v16` | 168 h | 0.034 (0.040) | `an5` 0.118 | +0.084 [+0.077, +0.091] | 0.111 | — | 0.118 | 0.480 |

Per variable (168 h, `game_v19`): the floor is 0.028 for the food store, 0.068 for wood, 0.057 for food on bushes,
0.009 for deadwood and 0 for the fire, cold members and working adults at the 06:00 targets; per start 0.022 (day 15),
0.018 (day 40), 0.020 (day 70). The noise after a fork is small compared with what the forecasters get wrong. The
floor-matched scores use three 06:00 starts per world, not every hour of every seventh day, so they are not comparable
with the T2 table above.

## 4. Effects (W3, W4)

Test families: 80 worlds, 604 cases (world × arm × replicate; `sp240L` / `sp120L` in the 71 worlds with a `T_low`).
Baselines selected on the 80 validation families: W3 `zero` (validation 0.510 vs `arm_mean` 0.633, `an5_arm` 0.644),
W4 `an5_arm` (10.39 days vs `arm_mean` 10.43, `zero` 12.46). Scaled effect MAE (the std of the arm's train effect) and
the mean true and predicted effect:

| arm | outcome | lag | true mean (std) | spread | `zero` | `arm_mean` | `an5_arm` | M1b-act s0 / s1 / s2 | M1-long s0 / s1 | mean predicted: M1b-act, M1-long |
|---|---|---|---|---:|---:|---:|---:|---|---|---|
| sp240L | food_stock | 7d | -89.72 (69.76) | 0.177 | 1.265 | 0.750 | 0.780 | 1.216 / 1.155 / 1.146 | 1.267 / 1.206 | -9.66, -4.14 |
| sp240L | food_stock | 30d | +106.42 (143.88) | 0.122 | 0.857 | 0.916 | 0.833 | 0.857 / 0.858 / 0.859 | 0.855 / 0.857 | +0.08, +0.61 |
| sp240L | alive | 7d | +5.23 (7.13) | 0.069 | 0.843 | 0.936 | 0.884 | 0.843 / 0.843 / 0.842 | 0.843 / 0.843 | +0.00, +0.00 |
| sp240L | alive | 30d | +9.13 (13.62) | 0.048 | 0.695 | 0.892 | 0.780 | 0.697 / 0.696 / 0.698 | 0.701 / 0.696 | -0.01, -0.05 |
| sp240L | deaths | 7d | +0.00 (0.00) | 0.000 | 0.000 | 0.000 | 0.000 | 0.040 / 0.020 / 0.000 | 0.060 / 0.159 | -0.01, +0.10 |
| sp240L | deaths | 30d | +0.01 (1.73) | 0.427 | 0.441 | 0.556 | 0.539 | 0.522 / 0.453 / 0.441 | 0.497 / 0.573 | -0.04, +0.09 |
| sp240L | settlements_alive | 7d | +0.33 (0.47) | 0.065 | 0.715 | 0.942 | 0.830 | 0.716 / 0.715 / 0.719 | 0.716 / 0.719 | +0.00, +0.00 |
| sp240L | settlements_alive | 30d | +0.25 (0.44) | 0.045 | 0.568 | 0.871 | 0.791 | 0.570 / 0.568 / 0.575 | 0.569 / 0.574 | -0.00, +0.00 |
| sp240L | split shift (days) | — | -20.0 (25.5) | 1.6 | 20.0 | 18.6 | 19.9 | 20.0 / 20.0 / 20.0 | 20.0 / 20.0 | 0.0, 0.0 |
| sp120L | food_stock | 7d | -37.04 (51.10) | 0.236 | 0.988 | 0.856 | 0.741 | 0.957 / 0.881 / 0.865 | 1.002 / 1.015 | -9.28, -0.90 |
| sp120L | food_stock | 30d | +59.84 (100.53) | 0.137 | 0.725 | 0.790 | 0.742 | 0.724 / 0.725 / 0.726 | 0.723 / 0.724 | +0.08, +0.65 |
| sp120L | alive | 7d | +2.70 (5.15) | 0.055 | 0.649 | 0.849 | 0.749 | 0.649 / 0.649 / 0.647 | 0.649 / 0.649 | +0.00, +0.00 |
| sp120L | alive | 30d | +4.72 (9.82) | 0.043 | 0.429 | 0.664 | 0.549 | 0.430 / 0.430 / 0.432 | 0.435 / 0.429 | -0.02, -0.04 |
| sp120L | deaths | 7d | +0.00 (0.00) | 0.000 | 0.000 | 0.000 | 0.000 | 0.040 / 0.020 / 0.000 | 0.033 / 0.113 | -0.01, +0.08 |
| sp120L | deaths | 30d | +0.06 (1.64) | 0.555 | 0.603 | 0.679 | 0.716 | 0.747 / 0.613 / 0.603 | 0.651 / 0.747 | -0.04, +0.08 |
| sp120L | settlements_alive | 7d | +0.15 (0.36) | 0.112 | 0.435 | 0.724 | 0.632 | 0.437 / 0.436 / 0.439 | 0.437 / 0.441 | -0.00, +0.00 |
| sp120L | settlements_alive | 30d | +0.15 (0.35) | 0.029 | 0.433 | 0.711 | 0.615 | 0.436 / 0.434 / 0.441 | 0.435 / 0.441 | -0.00, +0.00 |
| sp120L | split shift (days) | — | -10.3 (17.1) | 1.7 | 10.3 | 12.1 | 10.8 | 10.3 / 10.3 / 10.3 | 10.3 / 10.3 | 0.0, 0.0 |
| gf240E | food_stock | 7d | +40.42 (23.14) | 0.271 | 1.560 | 0.746 | 0.636 | 1.409 / 1.464 / 1.413 | 1.901 / 2.424 | +3.84, -15.70 |
| gf240E | food_stock | 30d | -29.19 (83.95) | 0.233 | 0.424 | 0.588 | 0.542 | 0.431 / 0.426 / 0.432 | 0.424 / 0.422 | +1.53, -0.58 |
| gf240E | alive | 7d | +0.01 (0.08) | 0.102 | 0.072 | 0.129 | 0.101 | 0.072 / 0.072 / 0.072 | 0.072 / 0.072 | +0.00, +0.00 |
| gf240E | alive | 30d | -2.92 (9.15) | 0.096 | 0.322 | 0.566 | 0.465 | 0.324 / 0.322 / 0.327 | 0.326 / 0.322 | +0.02, -0.02 |
| gf240E | deaths | 7d | -0.01 (0.08) | 0.125 | 0.089 | 0.159 | 0.124 | 0.443 / 0.266 / 0.443 | 0.797 / 3.456 | -0.02, -0.14 |
| gf240E | deaths | 30d | -0.04 (0.38) | 0.491 | 0.424 | 0.431 | 0.517 | 0.540 / 0.424 / 0.502 | 0.656 / 1.890 | -0.02, -0.17 |
| gf240E | settlements_alive | 7d | +0.00 (0.00) | 0.000 | 0.000 | 0.000 | 0.000 | 0.011 / 0.003 / 0.001 | 0.018 / 0.010 | +0.00, +0.00 |
| gf240E | settlements_alive | 30d | -0.09 (0.29) | 0.097 | 0.343 | 0.585 | 0.490 | 0.361 / 0.346 / 0.348 | 0.365 / 0.355 | +0.00, +0.00 |
| gf240E | split shift (days) | — | +4.4 (8.5) | 1.5 | 4.4 | 6.0 | 5.1 | 4.4 / 4.4 / 4.4 | 4.4 / 4.4 | 0.0, 0.0 |
| rw120E | food_stock | 7d | -29.73 (18.89) | 0.373 | 1.478 | 0.716 | 0.710 | 1.693 / 0.923 / 0.736 | 0.974 / 0.846 | -26.30, -27.95 |
| rw120E | food_stock | 30d | +20.12 (67.74) | 0.242 | 0.410 | 0.477 | 0.515 | 0.408 / 0.425 / 0.448 | 0.412 / 0.426 | -2.23, -1.35 |
| rw120E | alive | 7d | -0.61 (1.64) | 0.209 | 0.530 | 0.759 | 0.799 | 0.530 / 0.530 / 0.530 | 0.530 / 0.530 | +0.00, +0.00 |
| rw120E | alive | 30d | +1.44 (7.58) | 0.121 | 0.243 | 0.415 | 0.422 | 0.251 / 0.246 / 0.249 | 0.244 / 0.243 | -0.02, -0.00 |
| rw120E | deaths | 7d | +1.38 (2.41) | 0.198 | 0.793 | 0.824 | 0.955 | 0.807 / 0.793 / 0.800 | 0.836 / 0.886 | -0.01, -0.11 |
| rw120E | deaths | 30d | +1.96 (2.21) | 0.224 | 0.870 | 0.708 | 0.801 | 0.884 / 0.870 / 0.876 | 0.904 / 0.931 | -0.01, -0.10 |
| rw120E | settlements_alive | 7d | +0.00 (0.00) | 0.000 | 0.000 | 0.000 | 0.000 | 0.014 / 0.012 / 0.011 | 0.029 / 0.010 | +0.00, +0.01 |
| rw120E | settlements_alive | 30d | +0.04 (0.23) | 0.116 | 0.247 | 0.420 | 0.439 | 0.271 / 0.253 / 0.279 | 0.281 / 0.262 | -0.00, +0.01 |
| rw120E | split shift (days) | — | -2.2 (8.3) | 1.9 | 3.7 | 5.0 | 4.9 | 3.7 / 3.7 / 3.7 | 3.7 / 3.7 | 0.0, 0.0 |

(Scaled effect MAE; split shift in days. "true mean (std)" over the test cases; the scale of each cell is the std of the arm's **train** effect, so the `zero` column is mean |true| / train std.)

Readings:

- **At 30 days the effects are heavy-tailed and mostly zero** (a spoil changes the number of villages in a quarter of
  cases, a gift changes `alive` in a tenth), so predicting zero is the best of the three baselines on the
  pre-registered statistic, and the effect baselines that do predict the mean effect lose to it (`arm_mean` 0.619 vs
  `zero` 0.487 on the W3 cells, +0.132 [+0.090, +0.171], not pre-registered). The learned models predict effects near
  zero (mean predicted 30-day food effect of `sp240L` between −0.02 and +0.8 against a true +106) and land slightly above `zero`
  because their small spurious effects add error (e.g. M1-long predicts a drop in deaths after a gift: −0.05 and −0.29
  at 30 days against a true −0.04).
- **At 7 days the effects are large and regular** (a spoil of 240: −90 food; a gift: +40; rotted wood: +1.4 deaths),
  and the simple effect baselines capture them: on the same 12 cells at 7 days (not pre-registered) `an5_arm` 0.485,
  `arm_mean` 0.497, `zero` 0.612, M1b-act 0.592 (seeds 0.650, 0.566, 0.559): the model ties `zero` (−0.020 [−0.052,
  +0.023]) and loses to `arm_mean` (+0.095 [+0.044, +0.151]). It gets the sign of the 7-day food effect right in 88–97 %
  of the spoil and gift cases but predicts about a tenth of its size (−5 to −12 vs −90; +3 to +4 vs +40) and none of the
  rot-wood deaths.
- **Split shift.** The models never predict a split, so their predicted shift is 0 everywhere; the effect baselines are
  no better on test (`an5_arm` 12.49 days, `arm_mean` 12.30, `zero` 12.21) although `an5_arm` and `arm_mean` have the
  direction right in 0.87–1.00 of the `sp240L` and `gf240E` cases with a true shift. The between-replicate spread of the shift (1.6 days)
  is an order of magnitude below every predictor's error.
- **Drift** of the factual rollouts from the cast (T2 nMAE of the 8 V): M1b-act 0.12–0.13 at 7 days, 0.34–0.41 at 30
  days, 0.62–0.74 at 45 days; M1-long 0.15–0.16, 0.41–0.44, 0.65–0.75.

## 5. Split timing and theta (W5, W5b)

From day 25, 06:00, the 80 test worlds have not split yet (eligible 80 / 80; 393 / 400 train, 79 / 80 validation);
42.5 % of them do not split before the end (censored at day 121). `ridge_day` was chosen on validation (MAE 23.5 vs
`an5_day` 26.3, `median_day` 35.3).

| predictor | split-day MAE (days) | within ± 7 days | predicted censored |
|---|---:|---:|---:|
| `median_day` | 37.0 | 0.075 | — |
| `an5_day` | 28.8 | 0.225 | — |
| `ridge_day` | 29.2 | 0.088 | — |
| M1b-act (each seed) | 40.1 | 0.450 | 1.00 |
| M1-long (each seed) | 40.1 | 0.450 | 1.00 |

The models' 45 % within ± 7 days is the censored worlds (predicting "no split" is exactly right for them); on the
worlds that do split they are off by the full remaining time. **Theta** (read only after every prediction existed,
`runs/split_explain/`): the hidden threshold of the home village (mean 1,127, std 1,155 over the 80 test worlds)
correlates 0.49 with the true split day; adding it to a ridge on the 24-row window summary raises the leave-one-out R²
of every predictor's signed error by about 0.33 (`ridge_day` −0.07 → 0.27, `an5_day` −0.07 → 0.28, `median_day` and
every model 0.15 → 0.48). θ adds about a third of the variance of the signed split-day error that the window summary
does not explain (a share of variance, not of the MAE; λ chosen by the same leave-one-out error, so slightly
optimistic): the hidden threshold carries much of what the observable predictors get wrong.

## 6. What a paper can claim

**Supported.**

1. *A benchmark for counterfactual forecasting in an agent-based world*, with the evidence that it measures something:
   560 worlds of a frozen simulation (`game_v19`) recorded as paired intervention families (the factual and up to
   four interventions — 489 of 560 worlds have all four — × two noise replicates forked before the cast, identical
   histories up to the cast, checked for every family), a 560-world control without splits, and a noise-floor set
   (four replicates at three starts for every test world); the recorder reproduced 52 of 52 episodes of the simulation
   team's own intervention screen (2 worlds, Stage A group F) bit for bit (Part 0). The
   interventions have large, regular short-term effects (−90 food a week after a spoil, +40 after a gift, +1.4 deaths
   after rotted wood) and heavy-tailed month-scale effects through village splits (a spoil brings the split forward by
   20 days on average).
2. *Factual forecasting is not saturated on this world*: the best calendar baseline is at least 0.059 (168 h) and
   0.032 (24 h) above the conditional-mean floor (intervals in section 3), so there is room for better forecasters.
3. *On every-hour starts, a learned autoregressive world model is a slightly better factual forecaster than the
   strongest simple baseline* — 5 % (168 h) and 10 % (24 h) below the validation-selected analog forecaster with
   intervals below 0 on 80 fresh test worlds, every seed (W1, W1b WEAK) — the same direction as WM-1.3's `game_v16`
   result, with a smaller margin on worlds that split. Caveat: at the three 06:00 floor starts the same model is worse
   than `an5_off` (seed mean − `an5_off`, not pre-registered: 168 h +0.023 [+0.016, +0.031], 24 h +0.007 [+0.004,
   +0.009]), so the advantage depends on the start set.
4. *A negative result with a mechanism*: trained on 3,780 factual and intervention trajectories with the action as an
   input, the same model does **not** predict intervention effects better than predicting none (W3 NONE, interval
   [+0.006, +0.027]), loses at 7 days to the arm-mean effect (not pre-registered), predicts no splits at all (W4 TIE
   with the zero predictor, W5 NONE), and a GRU that reads the whole history does not change this (W3-long NONE, W5b
   TIE). The diagnostics show why: the models miss even the one-hour mechanical effect of an act, and their mean
   rollouts never accumulate a split.

**Not supported.**

- That a learned world model of this kind predicts what the god's acts will do (W3, W4) — neither at 30 days nor, on
  a side analysis, at 7 days against the arm-mean baseline.
- That long memory anticipates a village's split (W5, W5b): no model predicts any split; the comparison is
  degenerate, not informative about memory.
- That forecasting on `game_v19` is near its irreducible error (the headroom is large); and anything about the
  MAE-optimal floor (only the conditional-mean floor was estimated).
- Anything about KEEP-level factual gains (the margin is 5–10 %, not 20 %).
- That the model is a better factual forecaster at every start (it is worse at the 06:00 floor starts).
- That the headroom bounds what an observable-history forecaster can gain (the floor is a full-state floor).

**The next experiment each open claim needs.**

- *Effects*: make the action visible to the training signal — sample training windows around casts (or weight cast
  rows), add a loss on the paired difference arm_r − none_r from the shared prefix (both branches are in the data), or
  give the model the act's mechanical effect as a structured input; then re-run W3 / W4 on the same test families.
- *Splits and memory*: model the split as an event — a hazard head trained on the fission indicator with stochastic
  rollouts (ensembles of sampled trajectories) and score the expected restricted split day — and only then compare a
  long-memory model with a 7-day context (W5b); give M1-long the same wall-clock budget as M1b-act (it got a quarter).
- *Floor*: estimate the MAE floor with the conditional median (more replicates per start), and at every-hour starts.

## 7. Limitations

- One seed for the control (W6 can reach WEAK at most) and two for M1-long; M1-long got a quarter of M1b-act's compute
  (12,000 steps at 0.40 s/step) and both of its runs, and M1b-act s2, ended on the step cap with their best step in the
  last 10 %.
- The model input is the requested amount of an act, as GOAL prescribes, but 28 % of the `sp240L` casts were applied
  only in part (mean 0.951 of 240; in world 40500 replicate 0 a "240" spoil applied 120 and has exactly the `sp120L`
  truth), so the action input is a noisy proxy of what happened.
- The floor is the conditional-mean floor of a forecaster that knows the full simulator state at the fork, measured at
  three 06:00 starts with four replicates; the MAE floor and the floor of an observable-history forecaster were not
  measured.
- The effect statistic is dominated by heavy-tailed 30-day effects; the 7-day side analysis and the diagnostics are not
  pre-registered. The diagnostics use validation worlds and the s0 / s1 runs only.
- GPU reproducibility was not measured; the DGX was shared with other users' jobs (load average up to 37), which moved
  wall times but not results.

