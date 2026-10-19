# WM-5 — independent review (GOAL.md 12.1)

Two rounds by fresh `general-purpose` subagents (read-only). Each round's full text, then the responses.

## Round 1 (2026-10-19 ≈ 11:55–12:25 KST, HEAD `cc3f744c`)

**Reviewer:** a fresh `general-purpose` subagent. The review was read-only: nothing was edited, staged, committed or pushed, and no process was started or stopped on the DGX.

**Date:** 2026-10-19, about 12:10 KST. Repository HEAD `cc3f744c` on `codex/worldmodel-2026-10-wip`.

**State of the run at review time:**
- CT-obs2 `ct_obs2_s0` is training: 790 steps logged at 712.6 s.
- `crn_obs2_s1`, `ct_obs2_s1` and `m1b_obs2_s1` have not run yet.
- The paper exists only as `paper_src.md` (not rendered).
- `runs/judge5/` is not committed yet. I ran `judge5.py` into my scratch directory, from the committed files only, to get the numbers quoted below.

**How I checked:**
- Read GOAL §4, 6, 7, 8, 9, 12, plus METRICS, METHODS, DECISIONS, RUNLOG, DATASET_OBS2 and WM5.md.
- Read the code in `tools/worldmodel/wm5/`.
- Recomputed things independently on the DGX with small read-only Python scripts against `_code/f7cf620a` and `worldsim-wm-data/obs2/`.
- Recomputed things on the Mac from the committed JSON files.
- Unit tests: `python3 -m unittest tools/worldmodel/wm5/test_wm5.py` gives 29 tests OK, 1 skipped (`test_every_quoted_number_of_the_paper_is_in_its_source`: "paper not rendered yet").

---

### 1. Summary

**No BLOCKING finding.** I found no bug that makes a committed number in `docs/worldmodel/wm5/runs/` wrong. These all match my own recomputations or were confirmed by direct checks:
- the policy and its rounds;
- the families;
- the floor;
- the IPW weights;
- the step plan and α;
- the checkpoint bookkeeping;
- the readout rounding;
- leakage control;
- the pre-registration order.

The real problems are **interpretive**. Two are MAJOR and concern Q2 (is the benchmark discriminative?):
1. The headroom ladder's `zero − ridge_obs` rung mixes up "knowing the arm's mean effect" with "knowing the per-world effect from the observable state".
2. The ridge numbers on obs2 measure extrapolation outside v2's support, not what obs2's observable state carries.

There are five MINOR findings (one missing run file, missing DECISIONS entries, a wrong rounding of one number, an overstated sentence about the floor in the paper, and checks `fam-check` could add) and several NOTEs.

| # | Class | Finding |
|---|---|---|
| M1 | MAJOR | `ridge_obs` ties the arm-mean baseline on v2. The E1 win over `zero` at 7 d is the arm's mean effect, not per-world information. The 30-d loss to `zero` comes from the loss function: ridge fits squared error (a mean), the metric is absolute error (rewards a median). |
| M2 | MAJOR | Ridge on obs2 (E1 obs2, the ridge rungs of F2) is extrapolation outside the training support, so it cannot answer "does obs2's observable state carry the effect". |
| m1 | MINOR | `ct_v2_s1_effects/effects_best.json` exists on the DGX but is missing from the committed copy. |
| m2 | MINOR | DECISIONS L2.1, L2.2 and L2.4 are cited in `effects5.py` but do not exist in DECISIONS.md. |
| m3 | MINOR | WM5.md and RUNLOG say the IPW effective sample sizes at 30 d are "127–141"; the true value 126.46 rounds to 126. WM5.md also has hard-coded numbers that no test traces. |
| m4 | MINOR | The paper says the floor can be beaten "never [by] a predictor that lacks the state". The data contradict this cell by cell. |
| m5 | MINOR | `fam-check` could cheaply check more: row c itself, and `pre_r0` against the round-1 episode for every family. I ran both and both pass. |
| N1–N12 | NOTE | Legitimacy of L4.3, L4.4 and L0.6; the one-seed WEAK cap on a deterministic regression; spikes at the last checkpoint; run-name collisions; and others (§3). |

---

### 2. Findings

### M1 (MAJOR): what `zero − ridge_obs` on v2 actually measures

**Evidence.** I recomputed from the committed `effects_base.json` (which holds WM-2's `arm_mean` and `an5_arm` predictions) and `ridge_*.json`, using `wm2.effects.compare` (world bootstrap, B = 2,000, `default_rng(0)`), on the v2 test, 12 cells:

| comparison | 7 d | 30 d |
|---|---|---|
| `ridge_obs` vs `arm_mean` | 0.480 vs 0.497, Δ −0.017 [−0.037, +0.002] (TIE) | 0.602 vs 0.619, Δ −0.017 [−0.035, +0.001] (TIE) |
| `ridge_obs` vs `an5_arm` | 0.480 vs 0.485, Δ −0.006 [−0.037, +0.025] (TIE) | 0.602 vs 0.594, Δ +0.007 [−0.021, +0.038] (TIE) |
| `ridge_priv` vs `arm_mean` | 0.434 vs 0.497, Δ −0.063 [−0.082, −0.043] | 0.590 vs 0.619, Δ −0.029 [−0.053, −0.006] |

**Why it matters.** E1 on v2 at 7 d is WEAK (`ridge_obs` 0.480 vs `zero` 0.612). Almost all of that gain sits in the food-store cells, where the mean effect is far from 0:
- `gf240E|food_stock|7d`: `zero` 1.560, ridge 0.598;
- `rw120E|food_stock|7d`: 1.478 vs 0.659;
- `sp240L|food_stock|7d`: 1.265 vs 0.758.

`arm_mean` captures the same gain. At 30 d, ridge (0.602) does worse than `zero` (0.487) for the same reason `arm_mean` (0.619) does. Ridge fits squared error and so predicts a mean. The metric is absolute error, which rewards a median, and with skewed, mostly-zero effects the median is near 0.

`paper_src.md` §6 says "a linear map of the **observable** pre-act state already beats `zero`" and "the gap … is partly reachable from the observable state on the interventional split". That over-reads the result: linearly, on v2, the observable state adds nothing measurable beyond the arm's mean, while the hidden state does add something.

**Fix (no rule change):**
- Report `ridge_obs` and `ridge_priv` against `arm_mean` / `an5_arm`, labelled post hoc, beside F1.
- In WM5.md and the paper, say that the observable rung of the ladder is entirely the arm-level mean, and that the per-world part is visible only with hidden state.
- Mention the squared-error vs absolute-error mismatch as the reason ridge loses to `zero` at 30 d.
- Optionally add a post hoc median (L1) regression as a diagnostic.

### M2 (MAJOR): ridge on obs2 measures extrapolation, not what obs2's observable state carries

**Evidence:**
- In the committed `effects_base.json`, every v2 **train** case of `gf240E` and of `rw120E` (800 each) sits at one cast row, 490. `sp240L` sits mostly at rows 262 and 670. Val and test use the same rows.
- obs2 test cast rows range from 166 to 1,702, with 61 distinct rows (`obs2_base.json`).
- Within the gift and rot cells, the calendar and timing features are therefore constant in training. Every obs2 case is outside the training support.
- λ chosen on v2 val, whose cast rows are the same as train, cannot detect this. λ = 0.01 (essentially unregularized least squares on 145 features) was chosen in four cells: `rw120E|alive|7d`, `rw120E|settlements_alive|7d`, `gf240E|settlements_alive|7d` and `sp240L|deaths|7d` (`ridge_obs.json` `lambdas`).
- The resulting obs2 per-cell errors:
  - `rw120|alive|7d`: `ridge_obs` 1.535 vs `zero` 0.036;
  - `rw120|deaths|7d`: 0.802 vs 0.129;
  - `gf240|food_stock|7d`: 1.024 vs 0.436.
- Twelve-cell averages: `ridge_obs` 0.537 / 0.514 vs `zero` 0.217 / 0.188 (E1 obs2 NONE at both lags).

**Why it matters.** E1 on obs2, and the `zero − ridge_obs` and `ridge_obs − ridge_priv` rungs of F2, are a test of transfer under strong covariate shift. They cannot be read as "the observable obs2 state does not carry the effect", nor used to split observable from hidden state on obs2. The paper's §6 sentence ("obs2's acts fall at other times of the year …") is correct but should be the headline, not an aside.

**Fix:**
- In judge.md, WM5.md and the paper, label E1-obs2 and the F2 ridge rungs explicitly as v2→obs2 transfer.
- Report the shift, post hoc: for example the share of obs2 test feature values outside the v2 train range per cell, or z-scores.
- Do not draw an observable/hidden conclusion from F2.
- No rule changes.

### m1 (MINOR): one evaluation file missing from the committed copy

`docs/worldmodel/wm5/runs/ct_v2_s1_effects/` contains only `effects_best.start` and an empty `effects_best.out`. On the DGX the evaluation finished: `effects_best.json` written 06:18:29, exit 0, sha256 `5f18e607…`. The copy was fetched at 06:17, before it ended (commit `cc148a06`).

Effect: judge5's secondary table has no `ct_v2_s1|best|*` row. Primary R3 is unaffected.

I compared sha256 of every other run JSON between `docs/…/runs/` and the DGX: all equal, except the running `ct_obs2_s0/config.json`, which is not yet copied.

**Fix:** fetch `effects_best.{json,end,exit,out}` again, commit, re-run judge5.

### m2 (MINOR): decisions cited in code but not recorded

`effects5.py` cites "DECISIONS L2.1" (L18: the risk set excludes rows with p_tot(c) = 0), "L2.2" (L290: match5 control row for reference rows past day 60) and "L2.4" (L580: the split shift is not predicted by M1 on obs2). DECISIONS.md has no L2.x entries.

L2.2 changes the `match5` predictions for c2 = c1 + 14 days after day 60. L2.1 is a positivity choice: rows that can never be treated are excluded from controls. Both are autonomous decisions that GOAL 0.3.1 requires to be recorded.

**Fix:** add L2.1–L2.4 with their reasons.

### m3 (MINOR): one number rounded wrongly, and untraced numbers in WM5.md

The IPW effective sample size of `sp240` treated units at 30 d is 126.46 (`describe.json` and `obs2_base.json`; the two files agree exactly). WM5.md §2 and RUNLOG say "127–141"; it should be 126–141. The paper's placeholder `#0` renders 126.

WM5.md §2, §3 and §8 hard-code many numbers that the `paper_numbers` test does not trace. I spot-checked: the §8 D2 intervals, and "1.409 → 1.549" and "4.187 → 4.183", all match the judge.

**Fix:** correct to 126. Either use placeholders in WM5.md or state that WM5.md is checked by hand.

### m4 (MINOR): the paper overstates what the floor guarantees

`paper_src.md` §3.4 says: "a perfect predictor could score below it, never a predictor that lacks the state."

The floor predicts each case by one other noise realization. For sparse count cells its error can exceed that of `zero`:
- v2 `gf240E|deaths|30d`: `zero` 0.424 < floor 0.694;
- v2 `sp240L|deaths|30d`: 0.441 < 0.604;
- obs2 `gf240|deaths|30d`: 0.052 < 0.094;
- obs2 `rw120|food_stock|30d`: 0.134 < 0.158;
- v2 deaths cells at 30 d, averaged: `zero` 0.578 vs floor 0.539.

The aggregate claim (zero − floor > 0) does hold at every split and lag:
- v2: +0.425 [+0.374, +0.483] at 7 d, +0.220 [+0.114, +0.327] at 30 d;
- obs2: +0.159 [+0.144, +0.175] at 7 d, +0.094 [+0.070, +0.116] at 30 d.

The v2 floor also equals √2 × WM-2's spread exactly (0.187 = √2 × 0.132; 0.267 = √2 × 0.189), which independently confirms its computation.

**Fix:** remove "never a predictor that lacks the state". Say the floor is roughly √2 times the noise-limited error for continuous symmetric noise and can be worse than `zero` for sparse counts. Optionally show the floor / √2 (that is, WM-2's spread) beside it. GOAL's word "ceiling" should not be used for it in the paper.

### m5 (MINOR): checks `fam-check` could add

The corrected `fam-check` compares members with the observed episode on rows < c, and `none_r0` with the round-0 factual. Two cheap, stronger checks are possible, and I ran both:
- **`pre_r0` against the round-1 episode, all arrays, every family with a non-empty `pre`.** Every non-empty `pre` is exactly the world's first act, so `pre_r0` must equal `rounds/r1/…_obs`. Result: **419 / 419 identical** (sha256 from `meta["arrays"]`). This also confirms that replicate 0 is a true no-op in every family, against 3 sampled worlds in the run's own unforked check.
- **Rows ≤ c, including the fork row, for 24 test families (12 worlds × c1, c2), every member and replicate.** Result: 0 differences. In the same families I recomputed every truth (arm_k − pre_k at c + 168 / + 720, four outcomes, k = 0, 1): **1,152 / 1,152 equal** to `obs2_base.json`.

**Fix:** add both checks to `fam-check` / DATASET_OBS2, or cite this review.

---

### 3. NOTEs

**N1. L4.3 (obs2 evaluations from `f7cf620a`) is legitimate.**
- `git diff 6e9e88ed f7cf620a -- tools/` changes only `obs2.py` (the L1.5 checker), `test_wm5.py`, launch scripts, and the new `judge5.py`, `figures.py` and `dataset_doc.py`.
- `effects5.py`, `causal5.py`, `m1_obs2.py` and `policy2.py` are unchanged.
- `judge5.py` was committed at 21:46:02, before the first WM-5 test number (22:20:45). Its later changes (`4d051e14`, `bcdc3c59`, `cc3f744c`) add only reporting blocks: `main_table`, `cost_block`, `compute_block`.
- Every output's `code_commit` is `6e9e88ed` (v2, D2) or `f7cf620a` (obs2), as claimed.

**N2. L4.4 (CT-obs2 cost check repeated) is acceptable.**
- The reason for discarding the first two checks is objective (our own concurrent evaluations). The third check's result was accepted unconditionally.
- All three are disclosed: 1.894 s/step → plan 7,349; 1.833 → 7,735; 0.910 → 11,850.
- The aborted `ct_obs2_s0` attempts read no evaluation: their `train_log.jsonl` files are empty.
- The actual training corroborates the third check: 790 steps in 712.6 s including one 14.1-s evaluation, about 0.90 s/step.
- Recommendation: add this corroboration to L4.4.

**N3. L0.6 counts an SSH stall as a "pause".** GOAL 0.3.5 and 0.3.8 have Mac sleep in mind. During the stall the DGX trained on and only control was lost. The decision is defensible but a judgement call; it moves the training deadline by 2.15 h. It matters only if the deadline becomes binding.

**N4. The one-seed rule caps a deterministic regression at WEAK.** E1 v2 7 d `ridge_obs` has ratio 0.783 ≤ 0.8 and CI upper bound < 0. That is the size of a KEEP, but the W1 rule caps a single seed at WEAK, and a ridge fit is deterministic. The same applies to `m1b_obs2` against `wm3_full` in E2. Report this next to the verdicts; do not change the rule.

**N5. Last-checkpoint spikes.**
- `crn_obs2_s0`'s final decoder `val_roll` is 1.372, against 0.631–0.729 at the preceding eight evaluations. Its test result: last 0.770 vs best 0.709 at 30 d.
- CT-v2 last `val_roll` 1.769 / 1.957 vs best 1.560 / 1.508, with best at steps 790 and 1,580, where α is 0.32 and 0.58 of α_max.
- The CRN-obs2 decoder `val_roll` peaked at 61.9 (step 14,347) under α on real progress.
- The primary checkpoint follows the authors and is pre-registered. Show the `val_roll` trajectories in the paper's appendix.

**N6. Run-name collision between WM-4 and WM-5.** Both have `crn_v2_s0`, `ct_v2_s0` and so on. WM5.md §8's "`crn_v2_s0` 30 d 4.187 → 4.183" refers to WM-4's run (its deaths cells), while WM-5's own `crn_v2_s0` happens to score 4.187 on all 12 cells at 30 d. Prefix the names (`wm4:`/`wm5:`) in the text, as `main_table` already does.

**N7.** The judge field `cases_floor` = 2,880 counts both replicates. The obs2 floor is scored on the 1,440 r0 cases, correctly, because `cell_errors` iterates over the r0 test cases. Rename the field or add a note.

**N8.** `ridge_priv` includes every hidden array column (3 global, 7 latent, 3 settlement columns as sum and max) and the 11 numeric `world_variety` values. It omits `world_variety.regions` (region centres and water distances), which WM-1's `HIDDEN_V2` lists as `regions`. A minor gap in completeness; disclose it.

**N9.** Untracked OMC tool directories `docs/worldmodel/wm5/runs/.omc/` and `runs/record/.omc/` exist. Never stage them. judge5 skips them because they hold no `config.json`.

**N10.** The DGX gate (GOAL 4.5) requires zero skips. The paper-number test skips until `paper.md` is rendered, so render it before the gate.

**N11.** Evaluations shared the GPU with trainings (for example, the M1 reference evaluations beside `crn_obs2_s0`). This affects wall times only; R4 holds for every completed run.

**N12.** `describe` (04:02) used the val families before `fam-check` passed (10:27). No data changed afterwards and the val use is allowed. The `fam-check` rows show 100 member interventions with `applied` = 0, and many spoils applied below 240 (limited by the stock). Worth one line in the datasheet.

---

### 4. Checked and found correct

### 4.1 Multi-cast policy (GOAL 6.1) and recording rounds (6.2.3–6.2.4)

**Code (`policy2.py`):**
- Probabilities are WM-4's × 2: (0.060, 0.016, 0.004), (0.004, 0.016, 0.060), rot 0.024 / 0.008. `policy.json` carries the same values.
- Terciles are read from WM-4's METRICS (q_f1 8.079227234753551, q_f2 12.1, q_w2 3.111944444444449).
- Status priority is cap → refractory (`day < last + 8`) → no_decision → decision.
- The draw is `default_rng([s, d, 2])`; the choice is WM-4's `choose`.

**Independent re-implementation on the DGX.** I re-implemented the policy from scratch and ran it on the observed episodes, read with `wm4.obs_split.state_of`, for four worlds:

| world | role | acts | result |
|---|---|---|---|
| 47183 | train | 5 (days 8, 22, 36, 48, 58) | every row's status, (p_g, p_s, p_r), arm and act equal `policy.json` |
| 48201 | val | 3 | same |
| 48406 | test | 4 | same |
| 48430 | test | 0 | same |

**Decisions read from the right episode.** For each of these worlds and each round k, running the policy on the round-(k−1) episode finds exactly act k (same day and arm), and finds none after the last act. Each round's episode carries exactly the first k acts. Each round-k episode equals round k−1 on rows ≤ the new act's row (`compare_prefix` returns []) and differs after it. The act row itself is a pre-act row.

**Bookkeeping in `obs2.py`:**
- `round_schedules`, `round_check` and `casts_before` use the correct round indices: the prior acts for round k are those recorded in rounds 1 … k−1.
- `round_check` compares rows ≤ the new act's row.
- `finalize` maps n acts → round n.

**Over all 1,640 worlds (DGX):**
- `refs` follow the c1/c2 rule with 0 errors.
- Days d+1 … d+7 after every act are refractory or cap.
- Status counts: decision 61,179, refractory 25,587, cap 154, no_decision 0.
- Max c2 + 720 = 2,446 ≤ 2,880, so no censoring.

### 4.2 Families, replicates, and the L1.5 correction

**Schedules and replicates:**
- Family schedules are `pre` (acts at rows < c) plus the arm at c, with nothing after.
- Val replicates = 1, test = 2 (`REPLICATES`, `fork_ok` in all 880 families).
- Manifest: `obs2fam_*` for val r0 = 200 per (member, ref); for test r0 and r1 = 240 each. Total 7,080 episodes.

**The L1.5 correction is right.** In the first attempt's 417 failing families, the only differences are in `none_r0` / `none_r1` (228 + 189); every one had a non-empty `pre`, with `casts_ok`, `fork_ok` and `members_ok` all true. The `none` member is not a family member: it is not used by `members()` (the loader maps `pre` → `pre_r` or `none_r`), nor by the manifest (`member_id` none only when `pre` is empty).

**Nothing that should be checked is left unchecked.** The `pre` and arm members are still compared with the observed episode. Stronger checks (m5) also pass.

### 4.3 The floor

**obs2:** every one of the 1,440 r0 test cases' floor predictions equals the r1 twin's truth for the same (seed, arm, ref), on the same cast row: 11,520 / 11,520 values checked. The floor is scored with the same `fl` scales and the same r0 cases as every other predictor (`restrict` to `keep`).

**v2:** the other replicate on WM-2's 604 cases with WM-2's stds. It equals √2 × WM-2's spread exactly (§2, m4).

### 4.4 Leakage

- `obs2_v19` reads only the `obs2_train`, `obs2_val` and `obs2_test` splits. Configs show `train_groups` ['train'] and `val_groups` ['val'].
- The observed-only manifest (sha `beea2130…`, used by every obs2 training) equals the full manifest's 1,640 observed entries exactly. No family member appears in a train or val split.
- `causal5.load_data` and the val roll use only the observed train and val episodes; normalization is fitted on train.
- Ridge trains on v2 train families, chooses λ on v2 val, and uses no obs2 label.
- `ridge_obs.json` `feature_names` lists 145 features (37 state × 3, 30 calendar, 3 act counts, days since the last act). Every state name is in the recorder's `observable_fields`. No hidden name appears.
- `ridge_priv`'s first 145 features are identical to `ridge_obs`'s; it adds 27 hidden features.
- The state and input names of a v2 `none_r0` and an obs2 `pre_r0` episode are identical (37 / 74), so feature vectors align.
- The act-count and days-since-last-act features are always 0 on v2, so their ridge weights are exactly 0.

### 4.5 IPW weights by hand (GOAL 7.2), real obs2 train data

World 47015 has acts `rw120`@358, `gf240`@718 and `gf240`@982.

**Treated unit:** `gf240` at row 982, h = 720.
- p_a = 0.016.
- The window (c, c + 720) holds 7 refractory rows and 11 decision rows; ∏(1 − p_tot) = 0.494729.
- Weight by hand 1 / (0.016 × 0.494729) = **126.331714**; `effects5.stack` gives **126.331714**.
- At 7 d: 62.5 = 1 / p_a, because every row inside is refractory.
- An earlier act lies in the history.

**Control unit:** row 1174, h = 720, 11 rows in [c, c + 720).
- Product 0.494729.
- Weight by hand **2.021307**; `stack` gives **2.021307**.

**Supporting checks:**
- The Hájek mean (`wmean`) and effective sample size (Σw)² / Σw² are correct.
- `describe.json` (04:02) and `obs2_base.json` (10:29) give identical estimates and effective sample sizes, so the computation is reproducible.

### 4.6 Step plan, α, checkpoints, rounding

**Training reached (`val_metrics.json`, `train_log.jsonl`):**

| runs | steps reached | α at the last step | evaluations per stage |
|---|---|---|---|
| `crn_v2_s0`, `crn_v2_s1`, `crn_obs2_s0` | 15,700 + 53,800 / 15,700 + 53,800 | 0.99991 α_max | 15 |
| `ct_v2_s0`, `ct_v2_s1` | 11,850 / 11,850 | 0.0099991 (α_max 0.01) | 15 |

- Every stop reason is "planned". `plan.reduced` is false everywhere, and the cost-check predictions (3,694 / 11,998 / 1,275 / 11,006 s) are below the budgets. **R4 holds.**
- α = `alpha_at(step / P)`: the logged α at each evaluation equals the formula, for example 0.32146 at decoder step 3,587 and 0.00321 at CT step 790.

**Checkpoints:**
- CT: `model.pt` is the EMA state at the final evaluation; `model_best.pt` is the EMA state at the lowest `val_roll`.
- CRN: the last encoder with the last decoder, or with the best decoder; `load()` uses these correctly.

**Rounding:** `readout` rounds the food-store and alive levels at token n − 1 (row c + lag) and the cumulative deaths (deaths at c + summed daily deaths). Settlements alive are not rounded. Rounding never enters the feedback loop. This is equivalent to `effects3._chunk_h` L149–150 (`np.round` of the V features and of the cumulative cold hours and deaths), using the same round-half-to-even.

**Re-score check (D2):** the unrounded re-score reproduces WM-4's committed 12-cell numbers, for example `crn_obs_s0` 0.464 / 0.686 and `ct_obs_s1` 0.346.

**M1 on obs2:** warm-up and context are 168 rows < 192 (the minimum gap between acts), so D24 holds.

### 4.7 Pre-registration order

**Commit times** (Mac and DGX clocks agree within 1 s):
- METRICS part a: `2a97b4cd`, 21:24:24.
- First round-1 file: `schedules/r1/s47000_obs.json` at 22:08:43.
- METRICS part b: `6e9e88ed`, 21:39:47.
- The DGX worktree `_code/6e9e88ed` was created by `git fetch origin` at 21:40:41, so the commit had been pushed before any test run.

**Every test-role output on the DGX, by mtime** (`effects*.json`, `rescored*`, `obs2_base`, `ridge_*`, `metrics.json`):

| outputs | written |
|---|---|
| `rescored_*` | 21:40:45 – 21:42:23 |
| v2 effects | 22:20:45, 02:07:55, 02:54:56, 06:17:46 – 06:18:29 |
| `obs2_base`, `ridge_obs`, `ridge_priv` | 10:29:44, 10:30:00, 10:30:16 |
| obs2 references, `m1b_obs2_s0`, `crn_obs2_s0` | 10:36 – 11:45 |

All are after part b. All obs2 outputs follow `f7cf620a` (10:28:55; worktree fetched 10:29:23) and `fam-check` (10:27:52).

The finals log has 58 lines with no failure and no retry. RUNLOG times match the files within seconds; the re-score end is listed as 21:42:15 against the last file at 21:42:23, which is trivial.

`describe` (04:02) used train worlds and val families only, which is allowed.

### 4.8 Other

- `judge5` scoring keeps every rung of the ladder on the same cases.
- Multi-seed sides use `average_errors`, which averages per-case absolute errors, so `main_table`'s mean over seeds is valid.
- The val selection picks `zero` at both lags (val: `zero` 0.221 / 0.171, `ipw` 0.301 / 0.389).
- The obs2 M1 references use 720-row rollouts and the `pre` r0 member (`n_cases` 1,440, the same `obs2_base` sha everywhere).

---

### 5. Provisional verdicts

These come from committed files only, with my local judge5 run. They are incomplete: `ct_obs2` and the second seeds of `crn_obs2` and `m1b_obs2` are missing.

| rule | result (Δ = model − baseline, 95 % CI) |
|---|---|
| R1 obs2 7 d vs `zero` (0.217) | `crn_obs2` (s0) 0.403, +0.185 [+0.168, +0.203] **NONE**; `m1b_obs2` (s0) 0.215, −0.002 [−0.005, +0.001] **TIE** |
| R2 obs2 30 d vs `zero` (0.188) | `crn_obs2` 0.770 **NONE**; `m1b_obs2` 0.194, +0.005 [+0.003, +0.007] **NONE** |
| R3 v2 | `crn_v2` 1.649 / 3.645, `ct_v2` 1.243 / 1.920 (7 d / 30 d): **NONE** against both `zero` (0.612 / 0.487) and `an5_arm` (0.485 / 0.594) at both lags |
| R4 | all five completed CRN / CT runs: "trained as the authors" |
| E1 | v2 7 d WEAK (0.480 vs 0.612); v2 30 d NONE; obs2 NONE at both lags (see M1, M2) |
| D1 | `m1b_obs2` vs `m1b_obs` **WEAK** at 7 d (−0.005 [−0.007, −0.002]) and at 30 d (−0.004 [−0.007, −0.001]); `ct_obs2` pending |
| D2 | every interval contains 0 (WM5.md §8 numbers confirmed) |

---

### 6. For round 2

1. Re-check M1, M2 and m4 in the final WM5.md and `paper.md` wording, and the added post hoc rows.
2. Check that m1 is fetched and judge5 re-run; that DECISIONS L2.x (m2) and the 126 correction (m3) are in.
3. Repeat §4.6 (step plan, α, checkpoints) and §4.7 (timestamps) for `ct_obs2_s0`, `crn_obs2_s1`, `ct_obs2_s1` and `m1b_obs2_s1` once they finish, including whether any run hits the 18,000-s safety cap.
4. The paper: every number against `paper_numbers.json`, every reference against `REFERENCES_CHECK.md`, every claim against the final rule table (GOAL §12.1).
### Responses to round 1 (DECISIONS L7.1)

| # | response |
|---|---|
| M1 | Post hoc rows added to the judge (`posthoc/v2_vs_arm_level`): `ridge_obs` ties `arm_mean` and `an5_arm` at both lags; `ridge_priv` beats them at 7 d (WEAK). WM5.md §6 and the paper §6 now say the observable rung on v2 is the arm-level mean, the per-world part is visible only with hidden state, and why ridge loses at 30 d (squared-error fit, absolute-error metric). No median regression was added (not pre-registered; optional). |
| M2 | Judge `posthoc/support`: v2 gift / rot training cases sit at one cast row (490), obs2 test cases at 61 rows, 0 % of obs2 gift / rot cases on a v2 training row (spoils 20.4 %). E1-obs2 and the F2 ridge rungs are labelled v2→obs2 transfer in WM5.md and the paper; no observable / hidden conclusion is drawn from F2. |
| m1 | `ct_v2_s1_effects/effects_best.*` fetched and committed; the judge re-run. |
| m2 | DECISIONS L2.1–L2.5 recorded. |
| m3 | 126–141 in WM5.md and RUNLOG. WM5.md is checked by hand against the judge (only the paper is traced by `paper_numbers.json`). |
| m4 | The paper (§3.4) no longer claims a state-blind predictor can never beat the floor: it is ≈ √2 × the noise-limited error for continuous symmetric noise and can be worse than `zero` in sparse count cells; it is a reference point, not a ceiling. |
| m5 | Cited in DATASET_OBS2.md (the reviewer's 419 / 419 and 24-family checks); `fam-check` itself unchanged. |
| N2 | L4.4 corroboration: `ct_obs2_s0` ran 790 steps in 712.6 s (≈ 0.90 s/step), as the third cost check measured. |
| N4 | Reported beside E1 / E2 in WM5.md (a deterministic one-run side is capped at WEAK by the W1 rule). |
| N5 | The paper appendix shows the validation curves (figure `training_curves`), and WM5.md reports last vs best. |
| N6 | WM-4 run names prefixed `wm4:` in WM5.md. |
| N7 | Judge field renamed `floor_predictions_all_replicates`. |
| N8 | DECISIONS L2.5. |
| N9 | `.omc/` directories never staged. |
| N10 | The paper is rendered before the gate. |
| N12 | DATASET_OBS2.md notes the applied amounts. |

## Round 2 (2026-10-19, 21:51–22:09 KST (the reviewer's own header said "about 22:00–23:00"; corrected from the launch time and duration), HEAD `3d27cc07`)

**Reviewer:** a fresh `general-purpose` subagent. The review was read-only: nothing was edited, staged, committed or pushed, and no process was started, stopped or modified on the DGX. Scratch outputs went to the session scratchpad, outside the repository. During the review `git status` started showing `docs/worldmodel/wm5/REPORT.md` and `RUNLOG.md` as modified. The main session made those edits, not this review, and they are not reviewed here.

**State at review time:**
- All trainings and evaluations have finished.
- `runs/judge5/`, `paper.md`, `paper_numbers.json` and `figures/` are committed.

**How I checked:**
- Read GOAL §4, 8, 9, 11 and 12; REVIEW round 1 and its responses; WM5.md, DECISIONS, RUNLOG and DATASET_OBS2.
- Read the whole of `paper.md`, plus `paper_src.md`, `paper_numbers.json`, `references.bib` and `REFERENCES_CHECK.md`.
- Read `judge5.py`, `figures.py` and `wm2/effects.py` (`compare`, `cell_errors`, `cell_stat`, `rule`).
- Recomputed from the committed run files with small Python scripts. On the DGX I used only read-only `stat`, `sha256sum`, `git rev-parse`, `git status` and `cat`.

### 1. Summary

**No BLOCKING finding.** Everything mechanical reproduces exactly:
- The judge is reproduced byte for byte.
- `paper.md`, `paper_numbers.json` and all five figure PNGs are reproduced byte for byte.
- 5 verdicts recomputed independently all match.
- The new runs reached their plans, with α at 0.99991 α_max at the last step and no run hitting the safety cap.
- Every new test output comes after METRICS part b and from the commits it claims.
- Every citation key is in `references.bib` and in `REFERENCES_CHECK.md`.

The problems are in the **paper's and WM5.md's wording beyond the rule table**:
- Two MAJOR: a post hoc 30-day-reversal claim in the abstract that only half the data supports, and an untested mechanism for that reversal stated as fact. WM-3's review had already required the mechanism to be hedged.
- Nine MINOR: scope and post hoc labels, overgeneralizations, M1b-obs2 not converged and not disclosed, the M2 follow-up incomplete in `judge.md`, the page count, the CRN attribution, seed labels.
- Eight NOTEs.

| # | Class | Finding |
|---|---|---|
| MJ1 | MAJOR | Abstract (iv) and §6: "the learned dynamics models reproduce at most a fraction of the delayed sign reversal". This is post hoc and not labelled. It is shown only for spoils. For gifts, `crn_obs2_s1` reproduces the 30-day mean reversal almost fully. |
| MJ2 | MAJOR | Paper §6 states the reversal's mechanism ("through an earlier split and a second village gathering"; "a split triggered by hidden pressure") as fact. Nothing in WM-5 tests it. WM-3's review (item 7) required the cause to be hedged, and WM-3 had proposed a different mechanism. |
| m1 | MINOR | Scope and post hoc labels in the abstract, introduction and §2: "on every split" (obs has no floor); "reaches only the act's mean effect" (post hoc, v2 only); the E2 sentence's scope; "factual and counterfactual accuracy can diverge sharply" is not shown in the paper. |
| m2 | MINOR | Q3's headline ("more observational data does not help" / "No.") against D1's WEAK for M1b at both lags. The CT comparison in D1 is confounded with the checkpoint, and the paper's Q3 does not say so. M1b-obs2 did not converge (kept step 7,950 of 8,000, both seeds), and neither WM5.md nor the paper says so. |
| m3 | MINOR | "Secondary checkpoints are better at 30 d" and "validation loss is lowest early and rises" are overgeneralized. `crn_obs2_s1`'s best checkpoint is worse; CT-obs2's `val_roll` is flat; CRN's best steps are at 47–80 % of the plan. |
| m4 | MINOR | §6: IPW "gets the mean effects nearly right". The gift estimate is 2.1 × the truth and rot has the wrong sign (rot is not quoted). |
| m5 | MINOR | The M2 follow-up is incomplete: the `posthoc` block is not rendered in `judge.md`; the E1-obs2 / Table 1 obs2 ridge rows are not labelled as transfer. Paper §4.2 lists act-count features that have zero weight and says "every hidden field" against DECISIONS L2.5. |
| m6 | MINOR | WM5.md §7: "every obs-trained model … stay at or above `zero`" is literally false (0.215 and 0.216 < 0.217). |
| m7 | MINOR | The paper's main text is about 10–11 pages against GOAL's ≈ 9; WM5.md claims ≈ 9. |
| m8 | MINOR | CRN "as its authors train it": the schedule comes from the CT authors' re-implementation, not from Bica et al.'s code. The paper cites `melnychuk2022ct` for "the authors' public implementation of both methods". |
| m9 | MINOR | Seed-0-only numbers are not labelled as seed 0: paper §6 v2 deaths cells 14.51 / 4.99; the training-curve caption. |
| N1–N8 | NOTE | §3. |

### 2. Findings

#### MJ1 (MAJOR): the 30-day-reversal claim is post hoc, selective and partly contradicted

**Where.**
- Abstract (iv): "the learned dynamics models reproduce at most a fraction of the delayed sign reversal of food effects at one month".
- Paper §6, "The 30-day sign reversal".
- WM5.md §9: "nothing learns the 30-day reversal in full (CT-obs2 gets its sign at a fraction of its size)".

**Evidence.** Mean predictions on the obs2 test, from `judge.json` `obs2/tables/*/{cell}/mean_pred`. No rule covers these numbers.

| act, horizon | truth | predictions |
|---|---|---|
| spoil, 7 d / 30 d | −78.0 / +32.8 | CRN s0 −47.3, s1 −22.0; M1b-obs2 −0.5 / −1.4; `wm3_full` −13.7 / −4.9 / −7.9; CT +12.6 / +23.4 (all at 30 d) |
| gift, 7 d / 30 d | +21.7 / −17.3 | `crn_obs2_s1` +65.9 → **−15.9** (92 % of the 30-day mean, right sign); WM-4 `ref_crn_obs_s0` +68.9 → **−15.3**; `ct_obs2_s0` +45.5 → −6.6 (right sign); `crn_obs2_s0` +12.2 and `ct_obs2_s1` +5.2 (wrong sign) |

The spoil column supports the claim for every seed. The gift column contradicts "at most a fraction" for one CRN seed. A right mean is also not per-case accuracy: `crn_obs2_s1`'s `gf240|food_stock|30d` MAE is 0.822 against `zero`'s 0.279.

**Why it matters.** GOAL 11.2 says claims are limited to what the rule table supports, and "post hoc" must be said where it applies. This is a headline claim of the abstract, it is not in any rule, and it is not labelled.

**Fix:**
- Label it post hoc everywhere.
- Restrict it to spoils, or report both acts and both seeds.
- Say that a mean prediction can carry the right sign while the per-case error stays far above `zero`.
- Or drop it from the abstract.

#### MJ2 (MAJOR): an untested mechanism stated as fact

**Where.** Paper §6:
- "In the simulator a spoil empties the store for a week and, through an earlier split and a second village gathering, leaves more food a month later".
- "Rolling the dynamics forward does not reproduce the delayed mechanism (a split triggered by hidden pressure) that produces the reversal."

**Evidence:**
- No WM-5 file analyses the mechanism. `grep` for "second village", "earlier split" and "hidden pressure" finds only `paper_src.md`.
- WM-3 (WM3.md §2) proposed a *different* cause ("presumably through the agents' foraging … not inspected").
- WM-3's REVIEW item 7 (MINOR, fixed) required that "the cause of the 30-day reversal" be "hedged as a hypothesis everywhere".
- The obs2 truths (spoil 30 d: `settlements_alive` +0.12, `alive` +4.10) fit a split story but do not establish it.

**Fix:** state it as a hypothesis ("one possible route, not tested here, is …"), or remove the mechanism from both sentences.

#### m1 (MINOR): scope and post hoc labels in the abstract, introduction and related work

- **Abstract (i) and introduction findings.** "An effect floor … lies far below predicting no effect on every split"; "the gap … is large on every split". The obs split has no replicates and no floor (Table 1: "—"). Say "on both splits with noise replicates (v2, obs2)".
- **Abstract (iv).** "A linear map of the observable pre-act state reaches only the act's mean effect":
  - the comparison with `arm_mean` / `an5_arm` is post hoc;
  - it holds on v2 only, since on obs2 the regression does not transfer;
  - the "(post hoc)" in the parenthesis attaches only to the hidden-state clause.
  Write "on the interventional split (post hoc)".
- **Introduction findings, last sentence.** "Among the learned models compared with it (E2) only the world model trained on paired branches ties or beats it" is true on v2 only. On obs2, E2 gives KEEP for `crn_obs2`, `ct_obs2` and `m1b_obs2`. Add "on v2".
- **§2.** "Our benchmark … shows that factual accuracy and counterfactual accuracy can diverge sharply." The paper reports no factual-accuracy result; M1b-obs2's factual T2 is in `judge.json` `factual_obs2` but is not quoted. Either add the evidence (traced) or soften the sentence.
- **§6, two further causal explanations stated without a test.** Ridge loses at 30 d because "a squared-error fit predicts a mean while the metric rewards a median"; and `ipw` falls behind at 30 d because the survival weights grow. Both are plausible. Mark them as explanations ("likely", "post hoc") or test them, for example with an L1 ridge as a post hoc diagnostic.

#### m2 (MINOR): Q3 ("more observational data") is stated more strongly than D1, and M1b-obs2's non-convergence is not disclosed

**Headline against D1.**
- Paper §5 heading: "Q3: more observational data does not help". WM5.md §7 heading: "**No.**"
- D1 gives M1b-obs2 vs M1b-obs **WEAK at both lags**: −0.003 [−0.005, −0.001] and −0.004 [−0.006, −0.001]. The paper quotes only the 7-day value.
- WM5.md §1 says "barely helps", which is the accurate phrasing.

**The CT comparison is confounded.** CT-obs2 (authors' schedule, last checkpoint) is compared with WM-4's CT-obs (early best-val checkpoint), so the CT side of D1 mixes data size with schedule and checkpoint. WM5.md §7 says this; the paper's Q3 does not.

**Non-convergence.** Both M1b-obs2 seeds stopped at `max_steps` with the kept step at 7,950 of 8,000:
- `config.json` `best_step` 7950, `stop` "max_steps";
- the val T2 was still falling at the last evaluation;
- RUNLOG says "not converged by WM-3's definition".

WM-4 disclosed the same condition for `m1b_obs_s1` (WM4.md L98). WM5.md §3 / §7 and the paper do not. With three times the worlds and the same 8,000-step cap, each world is seen less often, which bears directly on Q3.

**Fix:**
- Use "barely helps (M1b-obs2: WEAK at both lags, ≈ 1.5–2 %)".
- State the CT confound in the paper.
- Disclose "not converged (kept step in the last 10 %)" in WM5.md §3 and §7, the paper's Q3 and §7, and appendix B.

#### m3 (MINOR): last vs best, and the validation curves, are overgeneralized

**"Secondary checkpoints are better at 30 days"** (WM5.md §5). This holds for 7 of 8 runs. For `crn_obs2_s1` the best checkpoint is worse: 30 d 1.125 vs last 0.790; 7 d 0.456 vs 0.401 (`judge.json` `obs2/secondary`).

**"Validation factual loss is lowest early and rises as the balancing weight reaches its maximum"** (WM5.md §5; paper §6 "their validation factual loss is lowest early"):

| run | `val_roll` (`train_log.jsonl`) | pattern |
|---|---|---|
| `ct_obs2_s0` | 0.546 at step 790, then 0.549–0.715, last 0.616 | flat within noise |
| `ct_obs2_s1` | 0.527 at step 790, then 0.556–0.761, last 0.563 | flat within noise |
| CRN decoders | best steps 25,107 / 32,280 (obs2), 32,280 / 43,040 (v2) of 53,800 | 47–80 % of the plan, not "early" |
| `crn_v2_s0` | 1.54–1.61 throughout | flat |

**Fix:** "for CT the best checkpoint is the first or second evaluation; for CRN it is in the second half of the plan; `crn_obs2_s1`'s secondary checkpoint is worse". Restrict "rises" to CT-v2.

#### m4 (MINOR): IPW "gets the mean effects nearly right" (paper §6)

| act, 7 d | IPW (pooled, train) | truth (obs2 test) |
|---|---|---|
| gift | +45.6 | +21.7 (2.1 ×) |
| spoil | −76.7 | −78.0 |
| rot (not quoted) | +4.1 | −4.8 (wrong sign; naive −100.2) |

WM5.md's "recovers the mean effects much better than `naive`" is accurate.

**Fix:** use that wording; quote all three acts or say "the spoil's mean". Note also that the pooled train estimate and the test-case mean are different populations.

#### m5 (MINOR): the M2 follow-up is incomplete, and §4.2 describes features inaccurately

**`judge.md`.**
- `judge.json` has `posthoc.{v2_vs_arm_level, support}`, but `judge.md` renders neither. Round 1's fix asked for the transfer label in `judge.md`, WM5.md and the paper.
- `judge.md`'s E1 obs2 rows carry no transfer note.
- WM5.md says "every number below comes from `runs/judge5/judge.md` / `judge.json`", so the post hoc numbers can be found only in the JSON.

**Paper rows.** Table 1's `ridge_obs` / `ridge_priv` obs2 cells and the §5.3 rows E1-obs2 / E2-obs2 are not marked "v2→obs2 transfer". Only the text after the table says so.

**Paper §4.2.**
- It lists "(obs2 only) the number of earlier acts per kind and the days since the last act" as features. These are constant 0 on v2 training, so their ridge weights are exactly 0 (round 1 §4.4) and they cannot affect any obs2 prediction.
- It says `ridge_priv` "adds every hidden field". DECISIONS L2.5 says `world_variety.regions` is excluded.

**Fix:**
- Render the `posthoc` block in `judge.md` with "post hoc" labels.
- Add "(transfer)" to those table cells or rows.
- In §4.2: "… are included but are constant on v2, so they carry zero weight"; and "every hidden field except the region geometry".

#### m6 (MINOR): WM5.md §7 is literally false

"Every obs-trained model and WM-2's / WM-3's references stay at or above `zero` on obs2 (M1b-act 0.215–0.225 …)". At 7 days, `m1b_obs2` scores 0.216 (s0 0.215) and `ref_wm2_m1bact_v19_s2` 0.215, against `zero` 0.217.

**Fix:** "none beats `zero` (M1b-obs2: TIE)".

#### m7 (MINOR): the paper is longer than GOAL's ≈ 9 pages

The main text (Abstract to §7) has:
- about 4,900 words of prose (6,200 including tables);
- three tables with 64 table lines in total, including a 38-row verdict table, plus the long R4 and D2 paragraphs;
- three figures, one of them a 2 × 2 panel.

In a NeurIPS-style layout that is about 10–11 pages before references. WM5.md §1 and §11 say "≈ 9 pages of content".

**Fix:**
- Move the §5.3 verdict table, keeping R1–R3 in the text, and the R4 / D2 lists to an appendix.
- Or correct the stated length.

#### m8 (MINOR): CRN attribution

The paper says (§4.4): "We use the authors' public implementation of both methods [@melnychuk2022ct]"; the headings and the abstract say "trained on their authors' full optimizer-step schedules". WM-4 METHODS (L5–6) is accurate: the repository is Melnychuk et al.'s CausalTransformer, which "contains both methods". The CRN step counts (15,700 + 53,800) are derived from that repository's CRN configuration, not from Bica et al.'s own code.

**Fix:** "the CT authors' public implementation, which includes CRN; CRN's schedule is the one that implementation uses", in §4.4, Appendix A and WM5.md §3.

#### m9 (MINOR): seed-0 numbers are not labelled

- Paper §6, "v2 30-day deaths cells 14.51 for CRN, 4.99 for CT". These are `crn_v2_s0` and `ct_v2_s0`; s1 gives 10.24 and 5.47. WM5.md gives both seeds.
- Figure 4 (`training_curves`) plots one run per panel without saying which seed.
- WM5.md §10, "the CRN-obs2 decoder's last evaluation spiked (1.372 …)", is s0 only; s1's last is 0.787 against a best of 0.769.

**Fix:** label the seed, or seed-average.

### 3. NOTEs

**N1.** The response table says the N2 corroboration was added to L4.4 ("790 steps in 712.6 s"). It is not in DECISIONS L4.4, RUNLOG or WM5.md: `grep 712` finds nothing outside REVIEW.md. The final runs do corroborate the third check: 11,850 steps in 10,665 s and 10,047 s, including 15 evaluations of about 14 s, that is 0.84–0.89 s/step against the measured 0.910. Add one line.

**N2.** WM5.md §1 says "28 verified references". The bib has 28 entries, of which 25 are cited. The three uncited ones (`ha2018recurrent`, `vaswani2017attention`, `hochreiter1997lstm`) are disclosed as uncited in REFERENCES_CHECK.md.

**N3.** DECISIONS headings are out of order:
- L1.4, L1.5, L0.6, L4.3 and L4.4 sit under "Part 3 / 5";
- L0.7 sits under "Part 7".

The content is fine; the placement makes entries hard to find.

**N4.** M1b-obs2's cost check (`_cost_m1b_obs2`: 613 steps, `stop` "time", 1,431 s including evaluations) is in `judge.md`'s Runs table. It is missing from WM5.md §3's list of cost checks and from paper Appendix B.

**N5.** The M1 trainer also writes `best_wm12rule.pt`, the WM-1/2 rule checkpoint (m1b_obs2_s1: 19:00:17, against `best.pt` at 21:13:05). Effects and factual T2 use `best.pt` (`metrics.json` `checkpoint`), as WM-4 did. It may be worth one line in METHODS.

**N6.** Appendix A is titled "D1–D24" but lists a subset. It does point to both METHODS files for the full list.

**N7.** Datasheet: v2 and obs give episode counts (5,316 + 560; 2,219); obs2 does not. Add 7,080 manifest episodes plus the round episodes.

**N8.** The introduction's earlier-study claims ("ignore the god's acts"; "teaches the first week's effects only") rest on unpublished earlier studies. Either tie them to Table 1's rows (M1b, `wm3_full`) or describe them as part of this benchmark's history.

### 4. Round-1 follow-ups

| item | claimed | verified |
|---|---|---|
| M1 | judge `posthoc/v2_vs_arm_level`; WM5.md and paper §6 wording | **Yes.** `judge.json` `posthoc` holds the ridge_obs / ridge_priv vs arm_mean / an5_arm comparisons with the figures quoted. WM5.md §6 and paper §6 say the observable rung is the arm-level mean, that hidden state adds per-world information, and give the squared-error vs absolute-error reason. Not rendered in `judge.md` (m5). |
| M2 | `posthoc/support`; E1-obs2 and F2 ridge rungs labelled as transfer | **Yes, partly.** `support`: gf240 / rw120 v2 train rows [490], obs2 61 distinct rows, 0 %; sp240 11 rows, 20.4 %. WM5.md §1, §6 and §10 and paper §6 label it. Not in `judge.md` or the paper's table rows (m5). No feature-level shift statistic, but the cast-row support suffices. |
| m1 | `ct_v2_s1_effects/effects_best.json` fetched | **Yes.** sha256 `5f18e607…` equals the DGX file; `judge.md` has `ct_v2_s1|best|*`. |
| m2 | DECISIONS L2.1–L2.5 | **Yes.** Present, with reasons. |
| m3 | 126–141 in WM5.md and RUNLOG | **Yes.** Also the paper's traced `126` / `141` (`describe/pooled/{sp240,rw120}/30d/ess_treated`). |
| m4 | §3.4 no longer claims the floor is unbeatable | **Yes.** §3.4 and §7 say ≈ √2, possibly worse than `zero` in sparse count cells, "not a hard ceiling". |
| m5 | cited in DATASET_OBS2 | **Yes.** L41 cites 419 / 419 and the 24 families / 1,152 truths. |
| N2 | L4.4 corroboration | **Not present** (N1 above). |
| N4 | W1 cap noted | **Yes.** WM5.md §6 and paper §5.3. |
| N5 | curves in the paper; last vs best in WM5.md | **Yes.** Figure 4 and WM5.md §5 / §10 (with m3 and m9's caveats). |
| N6 | `wm4:` prefixes | **Yes.** WM5.md §8 and §10. |
| N7 | field renamed | **Yes.** `floor_predictions_all_replicates` (`c5ecf7b7`). |
| N8 | L2.5 | **Yes.** The paper's wording still says "every hidden field" (m5). |
| N10 | paper rendered | **Yes.** `test_wm5`: 29 tests, OK, 0 skipped on the Mac. |
| N12 | applied amounts in the datasheet | **Yes.** DATASET_OBS2 L41. |

The `judge5.py` change after round 1 (`cc3f744c..c5ecf7b7`) only renames one field and adds `posthoc_block`. It does not touch any rule computation.

### 5. Runs finished after round 1

All of `ct_obs2_s0`, `crn_obs2_s1`, `ct_obs2_s1`, `m1b_obs2_s0` and `m1b_obs2_s1` were checked. Sources are `val_metrics.json`, `train_log.jsonl`, `config.json` and `train.{start,end,exit}`.

| run | reached / planned | α at last step / α_max | evals | stop | seconds | budget / safety cap | best (`val_roll`) step |
|---|---|---|---|---|---|---|---|
| `ct_obs2_s0` | 11,850 / 11,850 | 0.0099991 / 0.01 | 15 | planned | 10,665 | 14,400 / 18,000 | 790 |
| `ct_obs2_s1` | 11,850 / 11,850 | 0.0099991 / 0.01 | 15 | planned | 10,047 | 14,400 / 18,000 | 790 |
| `crn_obs2_s1` | 15,700 + 53,800 | 0.99991 / 1 | 15 + 15 | planned, planned | 1,848 | 5,400 / 6,750 | 32,280 |
| `m1b_obs2_s0` | 8,000 (max) | — | every 150 | `max_steps` | 12,992 | 16,200 | kept 7,950 |
| `m1b_obs2_s1` | 8,000 (max) | — | every 150 | `max_steps` | 11,964 | 16,200 | kept 7,950 |

- **α schedule.** Every logged α equals `alpha_at((step − 1) / P)` exactly (maximum deviation 0.0). This is consistent with "set before every step"; against `step / P` the deviation is at most 2.9e-4.
- **Plans.** `plan.reduced` is false everywhere; `planned_equal_authors` is true.
- **Safety cap.** No run hit it. **R4 holds** for all eight CRN / CT runs; the judge's R4 table matches.
- **Last vs best.** `model.pt` and `model_best.pt` differ on the DGX (sha256 prefixes `5b3c…` / `ff95…`, `da84…` / `5e70…`, `2385…` / `f501…`). The judge's last and best rows differ accordingly.
- **Reproducibility fields.**
  - CRN / CT configs: `code_commit` 835a8670 (obs2) or 38bea0c5 (v2), `manifest_sha256` beea2130 (observed-only, L1.4) or 4182f75a, seed, hyper, device cuda, torch 2.14.1+cu130, budget, safety cap, plan, `val_episodes` 200 (not sampled; one evaluation takes about 14 s, under 60 s).
  - M1b-obs2: `code_commit` 835a8670, `manifest_sha256` beea2130, seed, hyper (`max_steps` 8000, `max_seconds` 16200), `train_groups` ["train"], `val_groups` ["val"], device, torch, numpy, python, `stop` "max_steps", `best_step` 7950.
- **One training at a time.** Each training's start equals the previous one's end. Evaluations overlapped trainings only, which is allowed.
- **Timestamps and commits** (DGX `stat`, KST):
  - `ct_obs2_s0_effects` 14:52:24 / 14:53:39; `crn_obs2_s1_effects` 15:21:57 / 15:22:16; `ct_obs2_s1_effects` 18:12:18 / 18:14:18; `m1b_obs2_s0_effects` 11:29:14 with factual T2 at 11:44:52; `m1b_obs2_s1_effects` 21:35:19 with factual T2 at 21:49:53.
  - All are after part b (`6e9e88ed`, 2026-10-18 21:39:47) and after `f7cf620a` (2026-10-19 10:28:55).
  - Each effects file has `code_commit` `f7cf620a`; the factual T2 files have `eval_code_commit` `f7cf620a` and training commit 835a8670.
  - The DGX worktrees `_code/{f7cf620a,835a8670,6e9e88ed}` are at those commits and clean.
  - `effects5.py`, `causal5.py` and `m1_obs2.py` are unchanged since before part b (`effects5.py` at 21:37:34, the others at 21:29:36). 835a8670 is a descendant of 6e9e88ed.
  - Committed copies of four effects files have the same sha256 as on the DGX.
- **L4.4 (CT-obs2 cost check repeated).** The run files' start and end times confirm the stated cause:
  - check 1 (11:00:51–11:06:51) overlapped `ref_wm3_full_s2_obs2` (11:00:08–11:08:17);
  - check 2 (11:10:51–11:16:52) overlapped `ref_m1b_obs_s0_obs2`, `ref_crn_obs_s0` and `ref_ct_obs_s0`;
  - check 3 (11:45:52–11:51:52) started after the last evaluation of ours ended (`crn_obs2_s0` best, 11:45:09).
  - The final run times corroborate check 3 (N1). The repetition is objective and disclosed, and is acceptable.

### 6. Verdicts

**Judge reproduced.** `python3 tools/worldmodel/wm5/judge5.py --out <scratch>` produces a `judge.md` identical to the committed one (`diff`: none) and a `judge.json` identical byte for byte (`cmp`).

**WM5.md §4 matches the judge.** WM5.md §4's 38-row rule table is identical, line for line, to `judge.md` lines 3–42. Paper §5.3's table and Table 2 have the same values.

**Independent recomputation.** I wrote my own scaled-error computation from `effects_base.json` / `obs2_base.json` (r0 cases, floored scales) and the raw prediction files, with my own world bootstrap (`default_rng(0).integers`, B = 2,000), and also called `wm2.effects.compare`:

| rule | recomputed | judge |
|---|---|---|
| R3 v2 30 d `ct_v2` vs `zero` | 1.920 (1.950, 1.889), +1.433 [+1.299, +1.565], NONE | same |
| R3 v2 7 d `ct_v2` vs `an5_arm` | +0.758 [+0.662, +0.854], NONE | same |
| E1 obs2 7 d `ridge_obs` vs `zero` | 0.537 vs 0.217, +0.319 [+0.302, +0.335], NONE | same |
| D1 obs2 30 d `m1b_obs2` vs `m1b_obs` | 0.194 vs 0.197, −0.004 [−0.006, −0.001], WEAK | same |
| R1 obs2 7 d `ct_obs2` vs `zero` | 0.311 (0.304, 0.317), +0.093 [+0.077, +0.110], NONE | same |

Validation-family selection is `zero` at both lags (val `zero` 0.221 / 0.171).

### 7. The paper

**Rendering and tracing.**
- `figures.py check` passes: all 432 entries format to their source value.
- Re-rendering `paper_src.md` into scratch reproduces `paper.md` and `paper_numbers.json` byte for byte. Every quoted text is therefore at its placeholder's position (the `check` test alone only tests that the text occurs somewhere).
- Re-running `figures` reproduces all five PNGs byte for byte.

**Placeholders spot-checked for the right key and meaning** (all correct):
- 396 / 445 (`sp240` / `rw120` 7 d ESS); 126 / 141; 44 / 55 (WM-4 judge);
- 1,388 (largest treated weight: sp240 30 d);
- +78.8 / −121.1, −9.2, +63.3, +60.9, +12.6 / +23.4, −13.7 (`mean_pred` cells);
- the cost checks; the appendix B rows (seconds and best steps);
- 22.3 h, which is the sum of the ten trainings' seconds (80,453 s);
- 40,085 s, which equals 3,042 + 6,140 + 4,619 + 2,637 + 954 + 144 + 6,613 + 15,936.

**Table 1 seed averages** checked: M1b obs2 0.221, v2 0.592 / 0.501; `wm3_full` obs2 0.239; M1b-obs obs2 0.219; CRN / CT time-capped obs2 0.519 / 0.317.

**Hard-coded numbers (not traced).** All are design constants or earlier studies' dataset facts, as DECISIONS L6.2 states:
- the splits (560 / 400 / 80 / 80; 1,640 / 1,200 / 200 / 240);
- fork day 11 (DATASET_V2: tick 14,280 = display day 11, 06:00); 604 and 240 test cases;
- 2,881 rows, 39 and 14 columns; 5,316 + 560 and 2,219 episodes (WM-2 METRICS / WM-4 WM4.md);
- policy probabilities and terciles 8.08 / 12.1 / 3.11; the step plans, budgets and hyper-parameters;
- Appendix E's "20 CPU cores, 119 GiB, load averages 2–36, 29 GiB" (RUNLOG / L0.5).

I found no hard-coded *result*.

**Citations.**
- 25 keys are cited; all are in `references.bib` and in REFERENCES_CHECK.md.
- No `[CITATION NEEDED]` is used.
- Hájek 1971 was dropped as unverifiable, and that is disclosed.

**Placeholders and URLs.**
- `[AUTHORS]` (title block and datasheet), `[VENUE]`, `[OWNER: release plan]` (release section and datasheet) and `[OWNER]` (funding) are present.
- No release URL is claimed. The only URL-like reference is the authors' existing repository (`github.com/Valentyn1997/CausalTransformer` at `c49a24fa`), taken from WM-4 METHODS.

**Claims against the rule table.**

Correct:
- Abstract (ii) and (iii);
- the introduction's "every pre-registered comparison … is NONE";
- "further behind … (post hoc)";
- the Q1 and Q4 texts;
- §6 ladder and E1 v2 / post hoc text;
- §5.3 E2-obs2 caveat; R4 and D2 paragraphs; §7 limitations.

Problems: MJ1, MJ2, m1–m5, m8, m9.

**Page count:** about 10–11 pages of main text (m7).

### 8. WM5.md against the judge

More than 25 numbers were checked; all equal the judge. They include:
- §1:
  - acts 3,966, 2.42, 3.2 %, 60.0 %, 80.9 %, 72.9 %; ESS 396–445 / 126–141;
  - Q1's 0.402 / 0.311 / 0.780 / 0.483 / 1.649 / 1.243 / 3.645 / 1.920;
  - M1b-obs2 −0.001 [−0.003, +0.001] and +0.005; floors 0.187 / 0.267 / 0.058 / 0.095;
  - D2 range −0.006 … +0.018.
- §2: 39.9 / 39.9 / 20.1 %; 1,163 / 1,174 / 572; +21.7 / −17.3 / −78.0 / +32.8; std 41.2.
- §3: the cost checks (0.097 / 0.040 → 3,694 s; 0.998 → 11,998 s; 0.044 / 0.011 → 1,275 s; 0.910 → 11,006 s); all ten table rows (seconds, best steps, kept 7,950 with 0.0622 / 0.0620).
- §4: the whole table (identical).
- §5:
  - +78.8 / −121.1, +128.2 / −189.0;
  - deaths cells 14.5 / 10.2 / 5.0 / 5.5 vs 0.58; other nine cells 1.33 / 1.19 / 0.82 / 0.82 vs 0.72;
  - WM-4 1.474 / 1.547 / 1.007 / 1.374; secondary 0.346 / 0.502, 0.709 / 0.770.
- §6: the ladder (identical); post hoc −0.017 [−0.037, +0.002], −0.006 [−0.037, +0.025], −0.063 [−0.082, −0.043], −0.051 [−0.080, −0.022]; 0.247; λ = 0.01 in exactly four cells (`ridge_obs.json` `lambdas`); 1.535 vs 0.036.
- §7: D1 values; references 0.215–0.225, 0.218–0.257, 0.218 / 0.221.
- §8: every D2 interval; 1.409 → 1.549; 4.187 → 4.183.
- §9: 0.425 / 0.220 / 0.159 / 0.094.

Claims against the rule table are right, apart from m2 (§7 heading), m3 (§5 generalizations), m6 (§7 "at or above"), MJ1 (§9 reversal) and m8.

### 9. Also checked and found correct

- **Unit tests:** `python3 -m unittest tools/worldmodel/wm5/test_wm5.py` gives 29 tests, OK, 0 skipped. The paper-number test now runs.
- **Paths:** `git diff --name-only 64d633ff..HEAD` lists only `tools/worldmodel/wm5/**`, `docs/worldmodel/wm5/**`, `docs/worldmodel/paper/**` and `docs/worldmodel/wm4/WM4.md`. The WM4.md diff is exactly the GOAL 11.1 line under the title.
- **Hygiene:** no `.omc` file and no checkpoint is tracked under `runs/`.
- **Re-scoring commits:** v2 effects have `code_commit` `6e9e88ed`; obs2 effects, references, `obs2_base` and ridge have `f7cf620a`.
- **RUNLOG and DECISIONS times** match the run files' start / end epochs within seconds. The second pause (L0.7) does not affect any start: the last training started at 18:08:56 on 2026-10-19, before the deadline.

### 10. Recommended before the gate

**Required (MAJOR):**
1. MJ1 and MJ2: re-word the abstract (iv) and §6's reversal paragraph — post hoc label, both acts and seeds, mechanism hedged or removed.

**Recommended (MINOR):**
2. Apply m1–m9: mostly wording, plus rendering `posthoc` in `judge.md` and disclosing M1b-obs2's non-convergence.
3. Then re-render `paper.md`, re-run `figures.py check` and the unit tests, and re-run `judge5.py` if `judge5.py`'s rendering changes.

**Optional:**
4. N1 (one line in L4.4), N2, N4 and N7.

### Responses to round 2 (DECISIONS L7.2)

| # | response |
|---|---|
| MJ1 | Removed from the abstract. Paper §6 and WM5.md §9: labelled post hoc; spoils and gifts, both seeds; "a right mean is not a right per-world prediction" with the 0.822 vs 0.279 cell. |
| MJ2 | The mechanism is no longer asserted: "We do not analyse why; a delayed change in settlement splits is one possible route, not tested here." |
| m1 | "Both splits with noise replicates (v2, obs2)"; abstract (iv) "on the interventional split … (both post hoc)"; introduction "on that split"; §2's divergence claim removed; §6's two explanations "likely, post hoc". |
| m2 | Q3 heading "barely helps" in the paper and WM5.md, WEAK at both lags with both CIs; CT confound stated in the paper; non-convergence disclosed (WM5.md §1, §3, §7; paper Q3, §7, Appendix B). |
| m3 | WM5.md §5 / §10 and paper §6 state the curves per method and the `crn_obs2_s1` exception (seven of eight secondary checkpoints better at 30 d). |
| m4 | "Much better than `naive` for two of the three acts"; rot quoted with its wrong sign; populations differ (pooled train vs test cases). |
| m5 | `judge.md` renders the post hoc block and labels obs2 E1 / E2 as transfer; paper Table 1 and Appendix F rows labelled; §4.2 features corrected. |
| m6 | WM5.md §7: "none … beats `zero` (M1b-obs2: TIE)". |
| m7 | Verdict table, R4 and D2 lists moved to Appendix F; main text ≈ 5,200 words of prose, two tables, three figures: ≈ 9–10 pages; WM5.md says so. |
| m8 | "The CT authors' public implementation, which includes CRN; CRN's schedule is the one that implementation uses" in §4.4, the abstract, the contributions, Q1 and WM5.md §1 / §3 / §5. |
| m9 | v2 deaths cells for both seeds; Figure 4 titles and caption say seed 0; WM5.md §10's spike is s0 (s1's last 0.787 vs best 0.769). |
| N1 | Added to L4.4 (0.84–0.89 s/step). |
| N2 | WM5.md: "28 verified references, 25 of them cited". |
| N3 | Left in place (append-only log); L7.2 records it. |
| N4 | WM5.md §3 and paper Appendix B. |
| N5 | L7.2: `best_wm12rule.pt` is not read. |
| N6 | Appendix A titled "selected; full list D1–D24 in METHODS". |
| N7 | Datasheet: 7,080 manifest episodes. |
| N8 | Introduction ties the earlier findings to Table 1's M1b rows and `wm3_full`. |
