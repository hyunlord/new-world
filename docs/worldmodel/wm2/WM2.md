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
  observable history can reach (section 5 shows that one hidden quantity, θ, carries a large part of the split timing). @@W2V16@@
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
- **Control (W6).** @@W6@@
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

@@WTABLE@@

The KEEP margin is 0.8 × the baseline; "every seed" uses the trained seeds (M1b-act 3, M1-long 2, control 1). A one-seed
configuration can reach WEAK at most.

## 2. Runs and convergence

@@RUNS@@

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

@@FLOORTABLE@@

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

@@EFFTABLE@@

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

@@V16NOTE@@
