# WM-3 — pre-registration (GOAL.md 8)

Committed before any test-role number of a WM-3 model exists. Every test evaluation of a WM-3 run runs from this
commit (`wm3/_setup/prereg_sha` on the DGX, read by `finals.sh`) and writes its file once (refusing to overwrite).
Where this file and GOAL.md disagree, GOAL.md wins; nothing here changes after the commit except clearly marked
report-only amendments.

## 1. Audit decision (Part 1)

`K1.decision` (AUDIT.md, `runs/audit/`): **A1 and A3 clean** — 0 structural mismatches in 3,592 train + val cases,
4 behavioural ratios outside [0.5, 1.5] (0.11 %, below 0.5 %), 0 harness violations in 3 × 306 val cases. WM-2's
wrong-signed one-step response is the trained models' (A2: food jump at the cast row predicted +6 / −12 / +18 for a
+238 gift, −20 to −67 for a `rot_wood` that does not touch food). No erratum; Part 2 unchanged.

## 2. Runs (GOAL.md 9.1)

Each run fine-tunes a WM-2 M1b-act checkpoint (`best.pt` of `wm2/wm2_m1bact_v19_s<s>`, sha256 in DECISIONS K0.3) as an
`M1TFH` (`wm3/model.py`) with `wm3/finetune.py`, one at a time on the DGX, in this order:

| run | init | `--seed` | batch (per step) | `pair` | `hazard` |
|---|---|---|---|---|---|
| `wm3_full_s0` | `wm2_m1bact_v19_s0` | 0 | 16 plain + 16 cast + 16 partners (48) | 1.0 | 10.0 |
| `wm3_ctrl_s0` | `wm2_m1bact_v19_s0` | 0 | 32 plain | 0 | 0 |
| `wm3_os_s0` | `wm2_m1bact_v19_s0` | 0 | 16 plain + 16 cast (32) | 0 | 0 |
| `wm3_full_s1` | `wm2_m1bact_v19_s1` | 1 | as `wm3_full_s0` | 1.0 | 10.0 |
| `wm3_full_s2` | `wm2_m1bact_v19_s2` | 2 | as `wm3_full_s0` | 1.0 | 10.0 |

Common: AdamW lr 1e-4, weight decay 1e-4, grad clip 1.0, rollout horizon k = 168 (fixed), `--max-steps 4000`,
`--eval-every EVAL_EVERY` (section 4), `--patience 6`, `--max-seconds` = min(14,400, deadline − now − 3,600) (deadline
2026-10-17 23:19:46 KST plus recorded pauses; no run starts with less than 3,600 s); every other hyper-parameter is the
init run's `config.json` (d_model 256, 4 layers, 4 heads, context 168, warmup 168, window 336, `tf_l1` 1.0, `v_weight`
5.0, `clamp_nonneg` 1, `roll_starts` 1, `roll_min_context` 167, `roll_weight` 1.0, `val_batch` 512). Streams: plain
windows `default_rng([seed, 0])`, cast windows `default_rng([seed, 1])`, plain-window rollout positions
`default_rng([seed, 2])`; `torch.manual_seed(seed)` before the hazard layer is created. The control and `os` runs use
the same class with `hazard = 0` (their `haz` layer is never trained), so the three `_s0` runs differ only in batch
composition and loss weights. The control is step-matched, not compute-matched (32 sequences per step vs 48). There is
no tuning of these weights. Drop order under the time rule: `wm3_full_s2` → `wm3_os_s0` → `wm3_full_s1`.

## 3. Definitions (GOAL.md 7.2, 7.3, 7.6)

- **Case**: (train world, arm, replicate r); arm member and partner (`none_r` of the same world) equal through the cast
  row c (asserted at load). **Cast window**: a train case drawn uniformly (all four arms, both replicates), rows
  c − 167 … c + 169 of the arm member (W = 336), so the action sits at window position p = 167, the last teacher-forced
  row of the rollout start; its partner window has the same rows of the partner.
- **Loss** = WM-2's teacher-forced NLL + fire BCE + `tf_l1` × weighted L1 and rollout loss over every window (plain,
  cast, partner; plain windows' rollout starts drawn as WM-2 draws them, cast and partner windows' at p)
  + `pair` × pair loss + `hazard` × hazard loss. **Pair loss**: for each cast window and its partner, rolled 168 steps
  from p from the same state, the mean over steps i = 1 … 168 and continuous features of
  |(cur_A,i − cur_F,i) − (true_A,i − true_F,i)| / `stepper.std` × the feature weights (`v_weight` 5 on the V
  variables, rescaled to mean 1); fire excluded. **Hazard loss**: BCE with logits of the hazard head at every
  teacher-forced row t of every window against y_t = 1[`g.fissions_total`[t + 1] − `g.fissions_total`[t] ≥ 0.5]; no
  positive weight.
- **Expected restricted split day** from reference row c (hazard models): h_k = the hazard probability at the position
  whose mean change produces row c + k; S_0 = 1, P_k = S_{k−1} h_k, S_k = S_{k−1}(1 − h_k);
  E = Σ_{k=1}^{K} P_k · day(c + k) + S_K · day(2,880), K = 2,880 − c, day = `v2.day_of_row`. **Split shift** =
  E_arm − E_factual (both rolled from row c of `none_r`, the arm with the arm's action at c). WM-2's event-based day of
  the same rollouts is reported beside it. **Split timing**: E from row 574 (tick 34,440) on the test worlds with
  `g.fissions_total` = 0 there (replicate 0 of `none`, factual actions).

## 4. Checkpoint selection (GOAL.md 7.5)

Evaluations at step 0 (the init) and every EVAL_EVERY steps, with `round_counts = False`:
`val_t2` = WM-2's validation T2 24 h (`m1_gru.val_score`, 06:00 starts, val group = replicate-0 members of the 80 val
families, batched 512); `val_eff` = the mean over 24 cells (4 outcomes × `sp240L`, `gf240E`, `rw120E` × {c + 7 d,
c + 30 d}) of the scaled effect MAE (`wm2.effects` scales from `effects_base.json`) on the 233 val cases of replicate 0
of those arms, from one 720-row rollout per branch. The step-0 `val_t2` must equal the init's `best_val_t2_24`
(0.0608 / 0.0603 / 0.0589) within 1e-3, else the run stops. **Eligible**: `val_t2` ≤ 1.05 × the step-0 `val_t2`.
**Kept** (`best.pt`): the eligible checkpoint with the lowest `val_eff`, the init included; patience counts evaluations
without a new best eligible `val_eff`. If the init is kept (`selected: init`), its V1 / V2 / V5 / V6 predictions are
WM-2's model with an untrained hazard head; for V3 / V4 that seed is "hazard head untrained" (section 5). Output
rounding of counts: `m1_gru.choose_rounding` on the kept checkpoint (val), as WM-2. "Not converged" = the kept step is
in the last 10 % of the steps run and the run did not stop on patience.

EVAL_EVERY = **250** (cost check, section 7; never changed during a run).

## 5. Rules (test families; "CI" = world-level paired bootstrap 95 % of model − baseline, B = 2,000, `default_rng(0)`)

**The W1 rule**: KEEP if the seed mean ≤ 0.8 × the baseline, every seed below the baseline and CI upper < 0; WEAK if
CI upper < 0 (not KEEP); TIE if the CI contains 0; NONE if CI lower > 0. A configuration with one trained seed reaches
WEAK at most.

| # | rule |
|---|---|
| V1 | **effects, primary** — W3 exactly as WM-2: 12 cells (`food_stock`, `alive`, `deaths`, `settlements_alive` × `sp240L`, `gf240E`, `rw120E`) at c + 30 days, test, both replicates (WM-2's 604 test cases), the mean over cells of the scaled effect MAE: `wm3_full` (every trained seed, seed-averaged errors) vs `zero` (WM-2's validation choice, not re-selected): the W1 rule (`wm2.effects.compare` / `rule`). |
| V2 | **effects at 7 days** — the same 12 cells at c + 7 days: `wm3_full` vs **`an5_arm`** (the lowest 7-day average on WM-2's validation predictions, both replicates: `an5_arm` 0.489, `arm_mean` 0.499, `zero` 0.647; asserted by `test_wm3.V2BaselineTests`): the W1 rule (`compare` / `rule`). |
| V3 | **split shift** — WM-2's W4 cases (`sp240L`, `gf240E`, restricted days, both replicates): `wm3_full`'s expected-day shift vs `an5_arm` (WM-2's validation choice): the W1 rule with `judge_wm2.pw_verdict` on per-world scores, a world's score = the mean over its W4 cases of \|predicted − true shift\| (days). Right-direction share reported; WM-2's cell-averaged comparison (`compare`) and the event-based shifts reported beside it. |
| V4 | **split timing** — WM-2's split-timing task (80 eligible test worlds): `wm3_full`'s expected day vs `ridge_day` (WM-2's validation choice, test MAE 29.21): the W1 rule with `pw_verdict` on the per-world absolute errors. |
| V5 | **ablation** (one seed each, WEAK at most; reported with intervals) on V1's cells with `compare`: `wm3_full_s0` vs `wm3_ctrl_s0` (the objective beyond more training under the same selection); `wm3_os_s0` vs `wm3_ctrl_s0` (cast windows alone); `wm3_full_s0` vs `wm3_os_s0` (pair loss and hazard beyond cast windows). |
| V6 | **factual guard** (reported) — raw T2 168 h every-hour on `v2_v19_fact` test (`wm3/final_fact.py` = `m1_gru.final --eval-dataset v2_v19_fact`), each WM-3 run vs the WM-2 checkpoint it started from (WM-2's committed `_fact/metrics.json`), `pw_verdict` on `bootstrap.world_scores` of `T2_any.168.test`. If a `wm3_full` seed is worse by more than 0.005 (CI lower > 0.005), WM3.md says the effect gain cost factual accuracy. |

**Untrained hazard head.** A `wm3_full` seed whose kept checkpoint is the init has an untrained hazard head: for V3 and
V4 it is left out of the seed mean and the verdict is capped at WEAK (KEEP needs every seed); with no trained-hazard
seed V3 / V4 are NONE. `wm3_ctrl` and `wm3_os` carry an untrained hazard layer and are scored as non-hazard models
(event-based split day); V3 and V4 apply to `wm3_full` only. A dropped run is absent from every rule; the verdicts use
the trained seeds.

## 6. Reported, not judged (Part 5)

Per-arm, per-outcome and per-lag effect tables (WM-2's layout, WM-2's three M1b-act seeds and the three baselines
beside); mean predicted vs true effect per cell; the lag curve (`wm2.diag_effect_lag`, first 40 validation families,
lags 1, 2, 6, 24, 72, 168, 720 rows) for every WM-3 run beside WM-2's `effect_lag.json`; hazard calibration of
`wm3_full` on validation (every val replicate-0 cast row: the factual rollout's 1 − S_720 against the observed split
event within 720 rows, 5 quantile bins, Brier score vs the observed rate); per run the stop reason, kept step, wall
time, s/step and the `val_eff` / `val_t2` trajectory; drift of the factual rollouts. All produced by
`wm3/judge3.py` from the files copied to `docs/worldmodel/wm3/runs/`.

## 7. Cost check (GOAL.md 9.2; train and val only)

Two checks from `wm2_m1bact_v19_s0` (`--seed 0`, lr 1e-4), alone on the GPU, train and validation data only
(`wm3/_cost/`, `wm3/_cost_2/` on the DGX; no test family read):

| check | code | run | steps | s/step | peak GPU | `val_t2` eval | `val_eff` eval |
|---|---|---|---|---|---|---|---|
| 1 | `bbae1a0c` | `wm3_full` | 100 | 2.31 (steps 1–100; 2.13 over 50–100) | 31,195 MiB | 7.4 s | 64.1 s (per-family rollouts) |
| 1 | `bbae1a0c` | `wm3_ctrl` | 100 | 1.57 | 21,299 MiB | 8.0 s | 68.5 s (per-family rollouts) |
| 2 | `f0ebe7e5` | `wm3_full` | 50 | (as check 1) | 31,451 MiB | 7.6–7.9 s | 10.8–11.4 s (pooled, DECISIONS K2.4) |

One evaluation with the pooled `val_eff` ≈ 19 s = 3.4 % of 250 `wm3_full` steps (≈ 550 s) and 4.8 % of 250 `wm3_ctrl`
steps (≈ 390 s), under 15 %: **EVAL_EVERY = 250** (GOAL.md 7.5's rule does not raise it). Step-0 checks: `val_t2`
0.0608 = the init's `best_val_t2_24` 0.0608; the init's `val_eff` 0.6149 (both paths). Hazard positives in the sampled
`wm3_full` batches: 2.8 × 10⁻⁴ per teacher-forced row (200 batches; train data 3.2 × 10⁻⁴, ≈ 0.93 fissions per episode).
Expected durations: `wm3_full` ≈ 4,000 × 2.2 s + 17 × 19 s ≈ 2.5 h; `wm3_ctrl` / `wm3_os` (32 sequences) ≈ 1.8 h; the five
runs ≈ 11 h if none stops early.
