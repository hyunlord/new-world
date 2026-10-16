# WM-4 — pre-registration (GOAL.md 8)

Two commits. **Part a** (this section) is committed before any treated episode of the observational split is
recorded (GOAL.md 6.2.3), so the policy cannot be tuned after an outcome is seen. **Part b** is committed before any
test-role number of a WM-4 model exists. Where this file and GOAL.md disagree, GOAL.md wins; nothing here changes after
its commit except clearly marked report-only amendments.

## Part a — the observational split's policy (GOAL.md 6.1)

**Worlds and roles** (`game_v19`, the frozen WM-2 recorder `_build/src_wm2`, `WORLDSIM_GIT_SHA`
`f5373ab2b7679c4a0d3a5c86b256971c86f93d41`): train seeds 46000–46399, val 46400–46479, test 46480–46559 (unused before
this run, DECISIONS L0.4). 120 display days, one row per 60 ticks (2,881 rows).

**Decision rows**: 06:00 of display days 8 … 60, tick = 1,440 d − 1,560 (day 11 → 14,280, v2's fork tick), row =
tick / 60 = 166, 190, …, 1,414 (53 rows, all at the 06:00 clock phase). A world casts at most once; after its cast it
makes no further decision.

**State at a decision row**, read from the world's factual episode, rows ≤ that row only (observable columns):
f = Σ `s.food_stock` / max(1, `g.alive`) (food per person), w = Σ `s.wood_stock` / max(1, `g.alive`). A row with
`g.alive` = 0 or `g.settlements_alive` = 0 is not a decision (p = 0, recorded).

**Terciles** (over every decision row of the 400 train worlds' factual episodes; `numpy.percentile`, linear):
q_f1 = 33rd and q_f2 = 67th percentile of f, q_w2 = 67th percentile of w. Frozen values (computed on the DGX by
`wm4/obs_split.py terciles` from `obs/factual/`; `wm4/obs_split.py policy` re-derives them and refuses to run if they
differ by more than 1e-9):

TERCILES_JSON: `{"q_f1": 8.079227234753551, "q_f2": 12.1, "q_w2": 3.111944444444449, "rows": 21200}`

(q_f1 = 8.0792 and q_f2 = 12.1000 food per person, q_w2 = 3.1119 wood per person; 21,200 decision rows = 400 worlds × 53, none excluded.)

**Daily cast probabilities** (`wm4/policy.py`):

| arm (kind, amount) | f < q_f1 | q_f1 ≤ f < q_f2 | f ≥ q_f2 |
|---|---|---|---|
| `gf240` (gift_food 240) | 0.030 | 0.008 | 0.002 |
| `sp240` (spoil_food 240) | 0.002 | 0.008 | 0.030 |
| `rw120` (rot_wood 120) | 0.012 if w ≥ q_w2 else 0.004 (every f) | | |

**Draw** at a decision row of seed s and day d: u = `numpy.random.default_rng([s, d]).random()` (the first uniform);
cast `gf240` if u < p_g, else `sp240` if u < p_g + p_s, else `rw120` if u < p_g + p_s + p_r, else none. Every arm has
positive probability in every decision state (overlap). The cast goes to "home" (the lowest settlement id at the cast
tick, as `schedules_p14.py`).

**Records**: `obs/policy/policy.json` — per world every decision row's (f, w, tercile, p_g, p_s, p_r, u) up to and
including its cast, and the cast (arm, day, tick, row) or none; one schedule file `obs/schedules/obs/s<seed>_obs.json`
per casting world. The propensities are known and may be used by the IPW baseline and by analyses, never as model
inputs.

**Reference row and families** (GOAL.md 6.2.5): c_w = the world's cast row, or for a world that never cast the 06:00
row of day 30 (tick 41,640, row 694). For every val and test world, a family forked at c_w with replicate 0 only: the
factual (`none`) and the three arms cast at c_w (`gf240`, `sp240`, `rw120`, the amounts above).

**Roles of the data** (GOAL.md 4.3): training and model selection see only the observed episode of each train / val
world (one per world: `_obs` of a casting world, the factual of a non-casting one). The val families' counterfactual
members are used only to choose among the observational baselines (disclosed as conservative for the models); the test
families only in evaluation.

**Expected** (GOAL.md 6.1, mean daily hazard ≈ 0.033 over 53 rows): ≈ 17–19 % of worlds never cast; arm shares among
casts ≈ 40 % gift, 40 % spoil, 20 % rot. The realized values are reported in `DATASET_OBS.md`, not judged.

## Part b — runs, selection and rules (GOAL.md 8)

Committed before any test-role number of a WM-4 model exists (`find` on the DGX at the commit time: no `effects.json`,
`split.json`, `metrics.json`, `calib_test.json` or `obs_base.json` under `wm1-runs/wm4/`). Every test evaluation runs
once from this commit (`wm4/_setup/prereg_sha`, read by `finals4.sh`) and refuses to overwrite its file.

### b.1 Runs (GOAL.md 9.1; one training at a time, `wm4/_setup/queue.sh`)

`--max-seconds` = min(budget, deadline − now − 3,600) (deadline 2026-10-18 15:13:31 KST plus recorded pauses).
Drop order under the time rule: GOAL.md 0.3.6.

| run | what | data | budget |
|---|---|---|---|
| `wm3_ctrl_s1`, `wm3_ctrl_s2`, `wm3_os_s1`, `wm3_os_s2` | WM-3's exact recipe (`wm3/finetune.py`; ctrl = 32 plain windows, os = 16 plain + 16 cast, no partner, pair 0, hazard 0; lr 1e-4, 4,000 steps, eval every 250, patience 6), inits `wm2_m1bact_v19_s1` / `_s2`, `--seed` 1 / 2; selection unchanged (lowest eligible `val_eff`, `val_t2` ≤ 1.05 × step 0) | v2 | 14,400 s |
| `wm4_haz_s0`, `_s1`, `_s2` | Platt recalibration of `wm3_full_s{0,1,2}`'s hazard head (`wm4/recal_hazard.py`; train-family rollouts; kept iff val Brier below the original's) — done, all three kept (DECISIONS L4.2) | v2 train + val | — |
| `m1b_obs_s0`, `m1b_obs_s1` | WM-2's M1b-act recipe from scratch (`wm4/m1_obs.py`: `--variant sev-blind_action-aware --window 336 --context 168 --warmup 168 --ramp-steps 1500 --eval-every 150 --patience 12 --batch 32 --clamp-nonneg 1 --k-max 168 --roll-min-context 167 --roll-starts 1 --tf-l1 1.0 --threads 8 --v-weight 5.0 --val-batch 512`, lr 3e-4 default), `--seed` 0 / 1, `--max-steps 8000`; selection on val factual T2 24 h (observed val episodes), WM-2's rounding choice | `obs_v19` | 16,200 s |
| `crn_obs_s0`, `ct_obs_s0` | the tuning grid (`wm4/causal.py grid`, `GRID`): 4 configurations per method, seed 0, each ≤ 1,200 s; the chosen member (lowest val factual loss `val_roll`) is the run | `obs_v19` | 4 × 1,200 s |
| `crn_obs_s1`, `ct_obs_s1` | the chosen configuration, seed 1 | `obs_v19` | 1,200 s |
| `crn_v2_s0`, `ct_v2_s0`, `crn_v2_s1`, `ct_v2_s1` | the obs-chosen configuration, seeds 0 / 1 | `v2_v19` (every train-family member; val `v19_fact`) | 2,400 s |

CRN / CT defaults (METHODS.md D10): CRN encoder hidden 128, BR 24, FC 96, dropout 0.2, lr 1e-3, batch 64; decoder
hidden 24, BR 24, FC 12, dropout 0.1, lr 1e-3, batch 1,024; gradient reversal α_max 1. CT d_model 128, 2 layers,
4 heads, BR 16, FC 48, dropout 0.1, lr 0.01, batch 128, domain confusion α 0.01, EMA β 0.99, relative positions (max
15). Grid: CRN {defaults; BR 64 / FC 128 and decoder BR 64 / FC 32; lr 1e-4 (both stages); BR 64 / FC 128, decoder
BR 64 / FC 32, encoder dropout 0.1}; CT {defaults; lr 1e-3; lr 1e-3 with BR 64 / FC 128; lr 1e-4 with BR 64 / FC 128}.
Epochs: the authors' number of optimizer steps (METHODS.md D7), each run capped by its budget. Before the first run of
each kind a cost check (≤ 100 steps or ≤ 300 s, train and val data only).

**Model selection uses validation factual loss only** (no counterfactual member is read by training or selection):
M1 — val T2 24 h on the observed val episodes; CRN — encoder one-step MSE, decoder `val_roll`; CT — `val_roll`
(METHODS.md D8: 30-token factual rollouts with the observed treatments from tokens 6, 18, 30, 42, 54 of every val
episode, evaluations at most every budget / 15 s). Dry runs of every training and evaluation path on train / val data
only (`wm4/_dry/`, `launch/dryrun.sh`) preceded this commit. **Baselines** are chosen per lag on the **val families** (the only use of val counterfactuals;
conservative for the models): the member of {`zero`, `naive`, `ipw`, `match5`} with the lowest 12-cell average.

### b.2 Cases, scales, statistic

- **Obs test**: 80 test worlds × 3 arms = 240 cases at c_w (replicate 0 only); 12 cells per lag (`food_stock`,
  `alive`, `deaths`, `settlements_alive` × `gf240`, `sp240`, `rw120`), the restricted split shift reported only.
  Scales: per (outcome, lag), max(the v2 partner arm's train std, the v2 train std pooled over every arm's train cases)
  (`effects_base.json` `cases.train`); the unfloored variant (the v2 arm's std) and the per-cell table are secondary.
- **v2 test**: WM-2's 604 test cases (both replicates) and WM-2's scales, 12 cells per lag (W3 arms), as WM-3.
- **Statistic**: the mean over the 12 cells of the scaled effect MAE; **CI** = world-level paired bootstrap 95 % of
  model − baseline (B = 2,000, `default_rng(0)`; `wm2.effects.compare`). A model with several seeds is scored with its
  seed-averaged errors; a multi-seed baseline (O3, A1) with `wm2.effects.average_errors` of its seeds.
- **The W1 rule**: KEEP if the seed mean ≤ 0.8 × the baseline, every seed below the baseline and CI upper < 0; WEAK if
  CI upper < 0 (not KEEP); TIE if the CI contains 0; NONE if CI lower > 0. One seed reaches WEAK at most.

### b.3 Rules

| # | rule |
|---|---|
| O1 | **obs, 7 d (primary)**: each of `crn_obs`, `ct_obs`, `m1b_obs` (every trained seed) vs the obs baseline chosen on the val families at 7 d: the W1 rule. |
| O2 | **obs, 30 d**: the same at 30 d (the baseline chosen at 30 d). |
| O3 | **value of counterfactual training (obs test, 7 d and 30 d)**: WM-3 `wm3_full` (seeds 0–2; trained on v2's paired branches, no obs data) vs each obs-trained model (seed-averaged), one verdict per pair and lag. |
| O4 | **confounding visible (reported)**: `naive`, `ipw` and `zero` on the obs test, per arm and per lag, with paired intervals; the pooled estimates, counts and effective sample sizes. |
| B1 | **v2, 7 d**: `crn_v2` and `ct_v2` vs `an5_arm` (WM-3's V2 baseline) and vs `wm3_full` (seeds 0–2, averaged): the W1 rule. |
| B2 | **v2, 30 d**: `crn_v2` and `ct_v2` vs `zero`: the W1 rule. |
| A1 | **ablation, three seeds (v2 test, 7 d and 30 d)**: `wm3_full` (seeds 0–2) vs `wm3_ctrl` (seeds 0–2, averaged) and vs `wm3_os` (seeds 0–2, averaged): the W1 rule. |
| H1 | **hazard, selected (v2 test)**: the recalibrated heads (`wm4_haz_s{0,1,2}`; a seed whose recalibration was not kept uses its original head) — split timing (WM-3 V4's task: expected day from row 574 on the 80 eligible test worlds) vs `ridge_day`, and the split shift (V3's: expected-day shift, per-world mean over the W4 cases) vs `an5_arm`: the W1 rule with `judge_wm2.pw_verdict` on per-world scores. Brier and the 5-bin calibration of P(split within 30 d) vs climatology reported on val and test, original and recalibrated heads. |

### b.4 Reported, not judged

Per-arm, per-outcome, per-lag effect tables with every model and baseline side by side (obs: WM-2 `wm2_m1bact_v19_s*`
and WM-3 `wm3_full_s*` evaluated on the obs test as references); mean predicted vs true effects; the unfloored-scale
variant of O1 / O2 and its own val selection; M1b-obs factual T2 24 h / 168 h on the obs test (`m1_obs.py final`);
CRN / CT factual 30-token rollout loss on the test factual episodes; per run the stop reason, best step / epoch, time,
the grid results. All produced by `wm4/judge4.py` from the files copied to `docs/worldmodel/wm4/runs/`.
