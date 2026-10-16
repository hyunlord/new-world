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
