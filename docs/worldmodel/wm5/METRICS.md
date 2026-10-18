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
