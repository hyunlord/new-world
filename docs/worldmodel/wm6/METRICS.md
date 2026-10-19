# WM-6 — pre-registration (GOAL.md 8)

Committed before any test-role number of a WM-6 model exists. At the commit time no WM-6 run has a test output
(`effects.json`, `effects_obs2.json`) under `wm1-runs/wm6/`, and the v2 test features (`_prep/test.npz`) are not
prepared; the trainings that run read the v2 train / val families and the v2x train families only. Every test
evaluation runs through `wm6/launch/finals6.sh`, which refuses to start before `_setup/prereg_sha` (this commit or a
later one that changes no rule) exists, and refuses to overwrite an output. Where this file and GOAL.md disagree,
GOAL.md wins; nothing here changes after its commit except clearly marked report-only amendments.

## 1. Test set, statistic, baselines (GOAL.md 4.1, 4.5)

- **Test set**: v2's 80 test worlds (40480–40559), WM-2's 604 test cases of **both** replicates (`effects_base.json`),
  scored with WM-5's v2 code unchanged: `wm2.effects.cell_errors` with WM-2's scales (`effects_base.json` `stds`; on
  v2, WM-5 used WM-2's scales, not WM-4's floored obs scales: `judge5.py`), the **12 cells per lag** (`food_stock`,
  `alive`, `deaths`, `settlements_alive` × `sp240L`, `gf240E`, `rw120E`), the statistic = the mean over the 12 cells of
  the scaled effect MAE (`wm2.effects.cell_stat`).
- **CI**: world-level paired bootstrap 95 % (B = 2,000, `default_rng(0)`, `wm2.effects.compare`); a multi-seed side
  uses seed-averaged errors (`wm2.effects.average_errors`).
- **W1 rule** (`wm2.effects.rule`): KEEP if the CI upper of model − baseline < 0, the seed mean <= 0.8 × the baseline
  and every seed below the baseline (two seeds or more); WEAK if the CI upper < 0 otherwise; TIE if the CI contains 0;
  NONE if the CI lower > 0.
- **Beside every number**: `zero` (0.612 at 7 d, 0.487 at 30 d), `an5_arm`, `wm3_full` (WM-3's seeds 0–2, committed
  predictions) and the **floor** (each case predicted by its family's other replicate: 0.187 / 0.267, WM-5 F1).
- **Closure** of a model at a lag = (`zero` − model) / (`zero` − floor) on that lag's 12 cells; point estimate from
  the full sample, 95 % CI from the same bootstrap draws (the ratio per draw, percentiles).

## 2. Models, N and seeds (GOAL.md 4.2, 7, 9.1)

- **Training sets**: replicate 0 only, nested: N = 400 (WM-2's 40000–40399), 800 (+ 50000–50399), 1,600
  (+ 50000–51199), 2,000 (+ 50000–51599). Validation: v2 val (40400–40479, both replicates), unchanged; model
  selection and early stopping use it only.
- **Models** (`tools/worldmodel/wm6/effects6.py`): `ridge_obs`, `ridge_priv` (WM-5's code per N); `mlp_obs`,
  `mlp_priv`; `seq_obs`, `seq_priv` (seeds 0, 1, 2); `gbm_obs`, `gbm_priv` (one run per N; droppable). Every `*_priv`
  model reads hidden state: **diagnostic**, never in a method ranking against an observable model.
- **The grid** (N = 400 only, seed 0): width {256, 512} × dropout {0.1, 0.3} for `mlp_obs` and `mlp_priv` separately,
  chosen by the lowest v2 val 24-cell average (both lags), frozen for every N; `seq_obs` / `seq_priv` use the choice of
  `mlp_obs` / `mlp_priv` for their head.
- **Kept checkpoint**: the epoch with the lowest v2 val 24-cell average (early stopping, patience 30, at most 400
  epochs, `--max-seconds` 3,600).
- **WM-5 reproduction** (4.1): WM-5's ridge on both replicates at N = 400 (`ridge --repro`) must reproduce WM-5's
  committed val and test predictions (unit test `TestWM5Reproduction`, and the judge); committed WM-2 … WM-5 numbers do
  not change.
- **Largest N** = the largest N at which `mlp_obs`, `mlp_priv` and `seq_obs` each have at least two seeds evaluated
  (planned: 2,000). A seed missing at some N (drop order, time rule) is reported; the seed-averaged model uses the seeds
  present.

## 3. Rules (GOAL.md 8; one verdict per pair and lag)

| # | rule |
|---|---|
| C1 | **Nonlinearity on observables (primary), v2 test 7 d, largest N.** The C1 model is the one of `mlp_obs` and `seq_obs` whose seed-averaged predictions have the lower **v2 val** 12-cell average at 7 d (kept checkpoints; ties: `mlp_obs`); it is compared with `ridge_obs` at the same N: W1 rule. |
| C2 | **The same at 30 d**: the model chosen by the v2 val 12-cell average at 30 d, vs `ridge_obs` at the same N, 30 d: W1 rule. |
| C3 | **Nonlinearity on the full state (diagnostic)**: `mlp_priv` vs `ridge_priv` at the largest N, 7 d and 30 d: W1 rule. |
| C4 | **Data, 7 d**: for each of `mlp_obs`, `seq_obs`, `mlp_priv`: the largest N vs N = 400 (both seed-averaged), W1 rule; and the **slope** of the 12-cell average against log2 N over every N evaluated (ordinary least squares on the seed-averaged model's statistic per N), point estimate on the full sample, 95 % CI from the bootstrap draws (reported). The 30-d slopes are reported too. |
| C5 | **Closure (reported)**: every model's closure at every N and lag, with its CI (the learning curves of error and closure against N). |
| C6 | **Observable vs hidden (reported)**: `mlp_priv` − `mlp_obs` and `seq_priv` − `seq_obs` at the largest N, 7 d and 30 d, Δ with CI. |

Reported, not judged: the per-cell tables; `gbm_*`; the obs2 transfer of `mlp_obs` and `seq_obs` at the largest N
(GOAL.md 9.3, beside WM-5's E1 row, WM-4's floored obs2 scales); each run's stop reason, epochs and time.

## 4. Pre-registered reading (GOAL.md 8, written before results, applied after)

Closures are point estimates at 7 d and the largest N; "a slope's CI" is the C4 slope's 95 % CI at 7 d. The readings
are checked in this order and the first that holds is the reading:

1. **"Learnable from observables"**: the C1 model's 7-day closure >= **0.5**, or its C4 slope's CI upper bound < 0 and
   its closure >= **0.35**.
2. **"Learnable only with hidden state"**: `mlp_priv`'s closure >= **0.5** while the observable closure (the C1
   model's) < **0.35**.
3. **"Not learnable at this scale"**: every closure < **0.35** (`mlp_obs`, `seq_obs`, `mlp_priv`, `seq_priv`, and
   `gbm_*` if run) and no C4 slope's CI excludes 0 (`mlp_obs`, `seq_obs`, `mlp_priv`).
4. **Otherwise: "partially learnable"**, with the numbers.

The paper's abstract, introduction and conclusion are updated only as the reading allows (GOAL.md 11.2).
