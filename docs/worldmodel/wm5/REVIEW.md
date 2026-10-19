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
