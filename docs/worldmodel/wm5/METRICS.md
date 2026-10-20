# WM-5 — pre-registration (GOAL.md 8)

Two commits. **Part a** (this section) is committed before round 1 of the obs2 split is recorded (GOAL.md 6.2.2), so
the policy, the roles and the reference-row rule cannot be tuned after any treated episode exists. **Part b** is
committed before any test-role number of a WM-5 model or baseline exists. Where this file and GOAL.md disagree, GOAL.md
wins; nothing here changes after its commit except clearly marked report-only amendments.

## Part a — the obs2 split (GOAL.md 4.2, 6.1, 6.2)

**Worlds and roles** (`game_v19`, the frozen WM-2 recorder `_build/src_wm2`, `WORLDSIM_GIT_SHA`
`f5373ab2b7679c4a0d3a5c86b256971c86f93d41`): train seeds **47000–48199** (1,200), val **48200–48399** (200), test
**48400–48639** (240), unused before this run (DECISIONS L0.4). 120 display days, one row per 60 ticks (2,881 rows).
Round 0 (the factual episodes of all 1,640 worlds, no schedule) was recorded before this commit; it carries no
intervention.

**Decision rows**: WM-4's — 06:00 of display days 8 … 60, tick = 1,440 d − 1,560, row = tick / 60 = 166, 190, …,
1,414 (53 rows).

**State at a decision row** (`wm4.obs_split.state_of`, observable columns only): f = Σ `s.food_stock` / max(1,
`g.alive`), w = Σ `s.wood_stock` / max(1, `g.alive`), read from the episode recorded **with every earlier cast** of the
world (the rounds below), rows <= the decision row only. A row with `g.alive` = 0 or `g.settlements_alive` = 0 is not a
decision (p = 0, recorded as `no_decision`).

**Terciles**: WM-4's frozen values, imported from `docs/worldmodel/wm4/METRICS.md` part a (`policy2.frozen_terciles`;
never recomputed): q_f1 = 8.079227234753551, q_f2 = 12.1, q_w2 = 3.111944444444449.

**Daily cast probabilities** (`tools/worldmodel/wm5/policy2.py`): WM-4's table × 2, so the confounding ratios are WM-4's.

| arm (kind, amount) | f < q_f1 | q_f1 ≤ f < q_f2 | f ≥ q_f2 |
|---|---|---|---|
| `gf240` (gift_food 240) | 0.060 | 0.016 | 0.004 |
| `sp240` (spoil_food 240) | 0.004 | 0.016 | 0.060 |
| `rw120` (rot_wood 120) | 0.024 if w ≥ q_w2 else 0.008 (every f) | | |

**Multi-cast rules**: a world casts **at most 5 times**; after a cast on day d the next decision is on day d + 8 or
later (rows of days d + 1 … d + 7 have p = 0, recorded as `refractory`), so no second cast falls inside (c, c + 7 d];
rows after a 5th cast have p = 0 (`cap`).

**Draw** at a decision row of seed s and day d: u = `numpy.random.default_rng([s, d, 2]).random()` (the first uniform;
the `2` keeps the stream distinct from WM-4's `default_rng([s, d])`); cast `gf240` if u < p_g, else `sp240` if
u < p_g + p_s, else `rw120` if u < p_g + p_s + p_r, else none (WM-4's `choose`). Every arm has positive probability on
every decision row (overlap). A cast goes to "home" (the lowest settlement id at the cast tick).

**Rounds** (GOAL.md 6.2.3): round k >= 1 — every world recorded in round k − 1 (all worlds for k = 1) runs the policy
on that episode from the row after its last cast (or day 8) up to its first new cast or day 60
(`obs2.py round-schedules`); a world with a new cast gets the schedule of **all** its casts so far
(`obs2/schedules/r<k>/s<seed>_obs.json`, `schedules_p14.py`'s format) and is recorded into `obs2/rounds/r<k>/`. Prefix
check per world: the new episode equals the previous round's on every row <= the new cast row (`global.npy` in full,
settlements per WM-2 6.2's rule) and carries every earlier cast. Rounds repeat until one adds no cast (at most 5
recording rounds). The **observed episode** of a world is its last round's episode (`obs2/observed/`). Replaying the
policy on the observed episode reproduces every round's casts and rows (`obs2.py finalize`; the propensity record
`obs2/policy/policy.json` is that replay: every decision row's (p_g, p_s, p_r) and status, every cast with its round).

**Reference rows** (GOAL.md 6.2.5; `policy2.ref_rows`): for every val and test world, **c1** = its first cast row, or
06:00 of day 30 (row 694) if it never cast; **c2** = its second cast row, or c1 + 14 days (+ 336 rows) if it has none.

**Families**: for each reference row c, a **`pre`** schedule = the world's casts at rows < c, and three arm schedules =
`pre` + the arm at c (`gf240`, `sp240`, `rw120`, the amounts above); nothing after c. One recorder call per (world, c)
with `--fork-tick tick(c)` — val worlds `--replicates 1`, test worlds `--replicates 2` (replicate 0 is a no-op;
replicate 1 injects the fork noise, as v2's floor families). The recorder requires `--include-factual` with a fork, so
every family also has a `none` member; when `pre` is empty the `pre` member is that `none` member (no `pre` file is
written). Checks: every member equals the observed episode on rows < c (per-array sha256 / WM-2 6.2's rule); every
member carries the `pre` casts (an arm member: `pre` + the arm at c only); `pre` replicate 0 equals a fresh unforked
recording of the same schedule for 3 sampled test worlds.

**Roles of the data** (GOAL.md 4.3): obs2 training and model selection see only each train / val world's observed
episode. The val families are used only to choose among observational baselines (disclosed); the test families only in
evaluation; test replicate 1 only in the effect floor. The propensities are known and may be used by IPW and analyses,
never as model inputs.

**Expected** (GOAL.md 6.1, not judged): ≈ 2–3 casts per world, ≈ 3 % of worlds never cast; realized values are reported
in `DATASET_OBS2.md`.

## Part b — runs, selection, cases and rules (GOAL.md 8, 9)

Committed before any test-role number of a WM-5 model or baseline exists (at the commit time no `effects*.json`,
`obs2_base.json`, `ridge_*.json`, `floor*.json` or `rescored*.json` exists under `wm1-runs/wm5/`; the trainings of
`crn_v2_s0` … are running and read train / val data only). Every test evaluation runs from this commit or a later
commit that changes no rule (`wm5/_setup/prereg_sha`, read by `finals5.sh`) and refuses to overwrite its file.

### b.1 Runs (GOAL.md 9.1; one training at a time, `wm5/_setup/queue.sh`; drop order GOAL.md 0.3.6)

| run | what | data | budget |
|---|---|---|---|
| `crn_v2_s0`, `crn_v2_s1` | CRN (`wm5/causal5.py`), WM-4's chosen configuration 3 (encoder BR 64, FC 128, dropout 0.1; decoder BR 64, FC 32; other settings WM-4 METHODS D10), planned steps = the authors' 15,700 (encoder) + 53,800 (decoder) unless the cost check reduces them (`_cost_crn_v2`: not reduced, 3,694 s predicted) | `v2_v19` (every train-family member; val `v19_fact`) | 5,400 s |
| `ct_v2_s0`, `ct_v2_s1` | CT, WM-4's configuration 2 (lr 1e-3, BR 64, FC 128; other settings D10), planned 11,850 steps unless `_cost_ct_v2` reduces them | `v2_v19` | 14,400 s |
| `m1b_obs2_s0`, `m1b_obs2_s1` | WM-2's M1b-act recipe from scratch (`wm5/m1_obs2.py` = WM-4's `m1_obs.py` flags), `--max-steps 8000`, selection on val factual T2 24 h (observed val episodes), WM-2's rounding choice | `obs2_v19` (observed train / val episodes) | 16,200 s |
| `crn_obs2_s0`, `crn_obs2_s1` | CRN as above (`_cost_crn_obs2`) | `obs2_v19` | 5,400 s |
| `ct_obs2_s0`, `ct_obs2_s1` | CT as above (`_cost_ct_obs2`) | `obs2_v19` | 14,400 s |

Before the first run of each kind (CRN-v2, CT-v2, M1b-obs2, CRN-obs2, CT-obs2) a 300-s cost check on the actual
dataset (train and val only): s / step, peak GPU memory, an evaluation's wall time. CRN / CT (GOAL.md 7.4, METHODS.md D16+): α = `alpha_at(step / planned, α_max)`; the safety cap is 1.25 × budget (a run
stopped by it is reported with `stop` "time"); at most 15 `val_roll` evaluations per stage on the whole val set (a fixed
120-episode sample only if one evaluation exceeds 60 s). **Primary checkpoint** = the last step (CT: its EMA weights; CRN:
last encoder and decoder), **counts rounded at readout**. **Secondary**: the lowest-`val_roll` checkpoint (CT: EMA
weights at that evaluation; CRN: the last encoder with its lowest-`val_roll` decoder) and the unrounded readouts. No
counterfactual member is read by training or selection.

### b.2 Cases, scales, statistic

- **v2 test**: WM-2's 604 test cases (both replicates, `effects_base.json`), WM-2's scales, 12 cells per lag
  (`food_stock`, `alive`, `deaths`, `settlements_alive` × `gf240E`, `sp240L`, `rw120E`).
- **obs2 test**: every test world × (c1, c2) × the three arms, replicate 0 (`effects5.world_cases`, rep codes 0 / 2);
  truth = arm r0 − `pre` r0 at c + 168 / + 720 rows; arms mapped to the v2 keys; WM-4's floored scales
  (`wm4.effects4.scales`), the unfloored variant secondary. Replicate 1 (rep codes 1 / 3) enters only the floor.
- **Floors** (F1, F2): each case predicted by the other replicate's true effect (v2: the family's other replicate, both
  replicates scored; obs2: r0 cases predicted by r1), scored with the same cells and scales.
- **Observational baselines** (obs2): `zero`, `naive`, `ipw`, `match5` (GOAL.md 7.2, `effects5.py`), chosen per lag on
  the obs2 val families (replicate 0) by the 12-cell average — the only use of val counterfactuals. On v2 the
  baselines are WM-2's (`zero`, `an5_arm`).
- **Direct regressions** (GOAL.md 7.3): `ridge_obs` and `ridge_priv` (diagnostic; never in a method ranking) trained on
  the v2 train families, λ per cell on the v2 val families, applied unchanged to the v2 and obs2 tests.
- **Statistic**: the mean over the 12 cells of the scaled effect MAE; **CI** = world-level paired bootstrap 95 % of
  model − baseline (B = 2,000, `default_rng(0)`, `wm2.effects.compare`); multi-seed sides use seed-averaged errors
  (`wm2.effects.average_errors`). **W1 rule**: KEEP if the seed mean <= 0.8 × the baseline, every seed below it and CI
  upper < 0; WEAK if CI upper < 0 (not KEEP); TIE if the CI contains 0; NONE if CI lower > 0. One seed reaches WEAK at
  most.
- **Model evaluations**: CRN / CT — the prediction contract of WM-4 (D13) at the cast row's phase from the case's `pre`
  member, rounded and unrounded readouts; M1 models — `wm3.effects3.predictions` with 720-row rollouts on obs2 (both
  lags; the split shift is not predicted there), WM-2's full rollouts on v2 (committed WM-2 / WM-3 numbers reused).

### b.3 Rules

"Primary" = last checkpoint, counts rounded (CRN / CT); M1b-obs2's checkpoint is its WM-2-rule selection with its
val-chosen rounding. Each "vs" is one W1 verdict per pair and lag.

| # | rule |
|---|---|
| R1 | **obs2, 7 d (primary):** `crn_obs2`, `ct_obs2`, `m1b_obs2` (each seed-averaged) vs the observational baseline chosen on obs2 val families at 7 d. |
| R2 | **obs2, 30 d:** the same at 30 d (the baseline chosen at 30 d). |
| R3 | **v2, 7 d and 30 d:** `crn_v2`, `ct_v2` vs `zero` and vs `an5_arm`. |
| R4 | **training reached** (must hold for R1–R3 to count as "trained as the authors"): each primary CRN / CT run completed >= 90 % of its planned steps in every stage and its last checkpoint has α >= 0.9 α_max; the planned steps equal the authors' unless the cost rule reduced them (reported). A run failing R4 is reported under its verdicts as "under-trained". |
| F1 | **v2 headroom (reported with CI):** `zero` − floor, `zero` − `ridge_obs`, `ridge_obs` − `ridge_priv`, `ridge_priv` − floor, at 7 d and 30 d. |
| F2 | **obs2 headroom:** the same ladder on the obs2 test (floor from its replicates). |
| E1 | **does the observable state carry the effect?** `ridge_obs` vs `zero` on the v2 test and on the obs2 test, 7 d and 30 d (the W1 rule). |
| E2 | **models vs the regression:** each primary model vs `ridge_obs` on its test split, 7 d and 30 d; with `wm3_full` (seeds 0–2) on v2 (committed WM-3 predictions) and on obs2. |
| D1 | **more observational data (reported):** on the obs2 test, `ct_obs2` vs WM-4's `ct_obs` (best-val as kept, unrounded; and re-scored rounded) and `m1b_obs2` vs WM-4's `m1b_obs`: the W1 rule. |
| D2 | **rounding alone (reported):** WM-4's `crn_{obs,v2}_s{0,1}` and `ct_{obs,v2}_s{0,1}` re-scored with rounding on their WM-4 test splits (obs: WM-4's 240 obs test cases and scales; v2: WM-2's), next to their committed WM-4 numbers (new files `rescored_rounded`; WM-4's numbers unchanged). |
| S | **secondary (reported):** every CRN / CT run's best-val checkpoint and unrounded readouts; the per-cell tables; the unfloored obs2 scales. |

### b.4 Reported, not judged

Per-arm, per-outcome, per-lag effect tables of every model and baseline side by side (obs2: WM-2 `wm2_m1bact_v19_s*`,
WM-3 `wm3_full_s*`, WM-4 `m1b_obs_s*`, `crn_obs_s*`, `ct_obs_s*` as references); per run the plan, reached steps, α at
the kept checkpoints, stop reason and time; the obs2 confounding (arm shares per f-tercile), casts per world, the share
of casts with an earlier cast in the history and the effective sample sizes; M1b-obs2 factual T2 24 h / 168 h on the
obs2 test. All produced by `wm5/judge5.py` from the files copied to `docs/worldmodel/wm5/runs/`.
