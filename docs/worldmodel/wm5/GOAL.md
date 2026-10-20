# GOAL: WorldSim — WM-5: a fair rematch and a paper draft — CRN and CT trained as their authors train them, a larger multi-cast observational split, a measured headroom for the benchmark, and the benchmark paper written

Repository `hyunlord/new-world`. Base: `codex/worldmodel-2026-10-wip` = `codex/worldmodel-2026-10` = `64d633ff01f6bc2e6969b3142b36c95a2a2073a1` (WM-4 final; draft PR #512).
This run is executed by **Claude Code on the Mac** in the WM clone, unattended. All recording, training and evaluation run **on the DGX Spark over ssh**. Simulation and art runs use the Mac: keep the Mac light (edits, small unit tests, git, web lookups for citations) and never take `/Users/rexxa/work/.mac_quiet.lock`.

**Where the research line stands after WM-2 … WM-4 (planner's review of WM-4):**
- An autoregressive world model ignores the god's interventions.
- Paired branches teach the first week's effects only: A1 WEAK, three seeds.
- On the confounded observational split, no trained model beats predicting no effect.
- The field's methods (CRN, CT) are at the bottom on both splits.

**Three objections a reviewer would raise, and WM-5 answers them:**
1. **"You did not train CRN and CT properly."**
   - CT's kept checkpoints have 4 % of the authors' optimizer steps and 20 % of α_max. α's ramp was tied to nominal epochs that the time budget never reached (WM-4 `METHODS.md` D7, D14).
   - Neither method rounded counts, while M1 does. Their death cells cost 1.4–4.2 against 0.3–0.6 for the other methods.
   - WM-4 kept the best-validation checkpoint; the authors keep the last.
2. **"Your observational split is too small for balancing to matter."** One cast per world, 400 train worlds, 320 casts in 48,000 tokens, IPW effective sample sizes 44–55. CRN and CT were built for many treatments per sequence.
3. **"Maybe nothing can beat zero — is the benchmark discriminative?"**
   - WM-2 measured a between-replicate effect spread of 0.189 on v2's 7-day cells against `zero` 0.487 (`WM2.md` L65), so headroom exists. It was never stated as the benchmark's ceiling.
   - Nobody tested whether the pre-cast **observable** state carries the per-world effect: a direct supervised effect regression.
   - Nobody tested how much of it lies in **hidden** state.

**What WM-5 does:**
- Retrains CRN and CT on their authors' schedule, with rounding.
- Records a second observational split `obs2`: three times the worlds, a sequential multi-cast policy, two reference rows per evaluation world, noise replicates on the test families.
- Measures the headroom ladder: floor ← privileged regression ← observable regression ← `zero`.
- Writes the paper draft.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-wm1
git fetch origin && git checkout codex/worldmodel-2026-10-wip
git merge --ff-only origin/codex/worldmodel-2026-10-wip   # 64d633ff
mkdir -p docs/worldmodel/wm5
cp ~/Downloads/claude_goal_worldsim_wm5.md docs/worldmodel/wm5/GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/worldmodel/wm5/GOAL.md>
```
Hooks stay neutralized in this clone (`.claude/settings.json` = `{}`, `--skip-worktree`). Never edit, stage or commit it, and never clear the flag.

**This document is `docs/worldmodel/wm5/GOAL.md`.**
- Commit it unchanged in Part 0. Where it disagrees with earlier WM goal documents, this one wins.
- Re-read the relevant section at the start of every Part and after every compaction or pause.
- The owner drives the run with `/goal`, and a small model reads only the transcript. Your final message must contain the section 13 report, the rule table of section 8, the commit hashes and the `git ls-remote` heads.

## 0.2 Overrides of CLAUDE.md
As WM-4's 0.2:
- no `install_hooks.sh`;
- never `lead/main` or `main`;
- no harness pipeline (4.5 is the gate);
- built-in `general-purpose` subagents only;
- a blocking hook other than `/goal` → record, push, report.

First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decisions go in `docs/worldmodel/wm5/DECISIONS.md` (ids `L0.1`, …); times go in `docs/worldmodel/wm5/RUNLOG.md`.
2. **Do not end your turn between Parts or while DGX jobs run.** Poll every 10–20 minutes with short calls; do other Parts' work meanwhile. The paper (Part 6) is written while GPU jobs run.
3. **Review loops:** two rounds maximum. Push after every Part.
4. **On failure:** work around, note it, continue. A negative result is valid; a missing number is not.
5. **Time rule:** past **46 hours** of wall time from the start (pauses excluded and recorded):
   - start no new training;
   - let running jobs end through their own caps;
   - evaluate what exists;
   - finish the paper with what exists, marking every missing number `[MISSING: reason]`;
   - push; go to Part 7.
6. **Drop order:**
   1. `m1b_obs2_s1`;
   2. the second seed of CT-obs2 and CRN-obs2;
   3. the second seed of CT-v2 and CRN-v2;
   4. `ridge_priv`;
   5. the obs2 test replicates (then F2 is NOT_RUN).

   **Never drop:** Parts 0–3; the obs2 split; one seed each of CRN-v2, CT-v2, CRN-obs2, CT-obs2 and M1b-obs2; `ridge_obs`; the v2 floor; the WM-4 re-scoring (9.5); their evaluations; the paper draft; Part 7.
7. **DGX jobs:** detached, with WM-3's watchdog and stack-dump-on-stall; one training job at a time.
8. **Mac sleep:** record the pause; DGX jobs continue.

---

# ⚠️ 1. MACHINES
- **Mac** (`/Users/rexxa/work/new-world-wm1`):
  - Allowed: edit, commit, push, unit tests on tiny synthetic inputs, web lookups for citations (Part 6).
  - Not allowed: recording, training, evaluation, cargo builds.
  - Never touch `/Users/rexxa/work/new-world-pass4`, `/Users/rexxa/work/new-world-art-prod`, `~/github/new-world-wt/lead` or any other checkout.
- **DGX** (`ssh hyunlord@100.70.109.50`; use a throwaway `ssh -F` config or the full address; never edit `~/.ssh/config`). As WM-4:
  - code worktrees: `/home/hyunlord/work/wm1-runs/wm5/_code/<commit>`;
  - runs: `/home/hyunlord/work/wm1-runs/wm5/<run>/`;
  - venv: `/home/hyunlord/work/wm1-venv`. **Never `pip install` into it.**
  - recorder: `/home/hyunlord/work/wm1-runs/_build/src_wm2`. **Do not rebuild.** Every recording sets `WORLDSIM_GIT_SHA=f5373ab2b7679c4a0d3a5c86b256971c86f93d41`.
  - new data: `/home/hyunlord/work/worldsim-wm-data/obs2/`;
  - read-only: `v2/`, `obs/`, and every earlier run (`wm2_*`, `wm3_*`, `wm4/*`).
- **Forbidden on the DGX:** `~/github/worldsim-training`, ComfyUI (8188), dev servers 3100/3200, other checkouts, processes you did not start.
- **Measured throughput (WM-2 … 4):**

  | job | cost |
  |---|---|
  | recording | ≈ 2 s per episode at 8 workers (`--workers 8` with no training running, 4 while one does, `nice -n 10`) |
  | M1b-act | 1.54–1.58 s/step |
  | CRN encoder / decoder | ≈ 0.069 / 0.029 s per step |
  | CT | ≈ 0.83 s per step (batch 128 doubled by the masked copy), 14.3 GiB GPU, `val_roll` ≈ 5.3 s per 80 episodes |

  Other users load the DGX: WM-4 saw load 2–56, a full swap and one `CUDA_ERROR_OUT_OF_MEMORY` at context creation. Record the load, do not wait for it. Retry a CUDA context failure up to three times, 5 minutes apart, and record each attempt.

---

# 2. GIT
- **Branches:** commit to `codex/worldmodel-2026-10-wip`. `codex/worldmodel-2026-10` moves only by fast-forward to a WIP commit whose gate (4.5) is green on both machines. Never force-push. Run `git remote -v && git branch --show-current` before every push and `git ls-remote` after. No merges.
- **Commits:** messages `worldmodel(wm5-N): …`. Stage explicit paths only.
- **May change:**
  - the new `tools/worldmodel/wm5/**`, `docs/worldmodel/wm5/**`, `docs/worldmodel/paper/**`;
  - one line at the top of `docs/worldmodel/wm4/WM4.md` (Part 6).
  - Everything else is **read-only**: import and wrap, never edit.
- **WM-4 code:** extend it by subclassing or wrapping inside `wm5/`, never by editing `wm4/`. The vendored CT/CRN modules stay as WM-4 vendored them; any further change lives in `wm5/` and is listed in `wm5/METHODS.md`.
- **PR #512:** stays draft. After the green fast-forward, add a comment with the rule table and a link to the paper draft.

---

# 3. CONTEXT AND QUESTIONS

## 3.1 Facts from WM-4 (verify each)
- **The kept checkpoints were early** (`METHODS.md` D14):
  - CT-obs s0/s1: 477 steps; CT-v2: 493 steps. The authors use 11,850; α reached 0.0019–0.0020 of 0.01.
  - CRN-v2 ran every nominal epoch (encoder 15,753, decoder 54,116 steps).
  - α schedule: `causal.alpha_at(ep / n_ep, …)` (L410, L456, L518) over nominal epochs computed by `epochs_for` to match the authors' steps. The time cap (`max_seconds`, L404–L552) stops CT long before.
- **WM-4 kept the lowest-`val_roll` checkpoint** (D8). CT's `val_roll` rose after its early best (0.273 → 0.40).
- **Rounding:** M1's readout rounds counts (`wm3/effects3.py` `_chunk_h` L149–150: the `v` features, cumulative cold hours, cumulative deaths when `round_counts`). CRN/CT do not (D13).
- **Configurations selected by WM-4's grid on `obs_v19`:** CRN config 3 (`enc_br` 64, `enc_fc` 128, `dec_br` 64, `dec_fc` 32, `enc_dropout` 0.1); CT config 2 (`br` 64, `fc` 128, `lr` 0.001). **WM-5 reuses them; no new grid.**
- **The WM-4 policy** (`wm4/policy.py`):
  - decision rows: 06:00 of days 8–60 (`DAYS`, `row_of_day`);
  - state f / w per person;
  - frozen terciles q_f1 8.0792, q_f2 12.1, q_w2 3.1119;
  - `P_GIFT` (0.030, 0.008, 0.002), `P_SPOIL` (0.002, 0.008, 0.030), rot 0.012 / 0.004;
  - one cast per world; the draw is `default_rng([seed, day])`.
- **WM-4 results:**
  - obs test 7 d: `zero` 0.218, M1b-obs 0.214, CT-obs 0.321, CRN-obs 0.497, `ipw` 0.391;
  - v2 7 d: `zero` 0.487, `an5_arm` 0.485, `wm3_full` 0.496.
  - Effects vary widely between worlds: gift 7-day food effect mean 18.6, std 43.5.

## 3.2 Questions
- **Q1 (primary):** trained as their authors train them (full schedule, last checkpoint, counts rounded), do CRN and CT beat `zero` and the observational baselines on a larger, multi-cast observational split? On v2?
- **Q2:** is the benchmark discriminative? Measure where `zero` sits against the floor (another noise replicate's effect), a direct effect regression from the observable pre-cast state, and the same regression with hidden state.
- **Q3:** does more observational data help? CT-obs2 and M1b-obs2 against WM-4's obs-trained models, on the same obs2 test.
- **Q4:** how much of WM-4's CRN/CT deficit was rounding? WM-4's runs re-scored with rounding.

---

# 4. RULES
## 4.1 Comparability
- **Harness:** WM-2/3/4's harness, definitions and bootstrap (world-level paired, B = 2,000, 95 %, `default_rng(0)`) everywhere they apply.
- **Effect cells and scales:** 12 per lag (4 outcomes × 3 arms; the split shift is excluded), with WM-4's floored scales (`wm4/effects4.py`), unchanged for obs2 and v2.
- **Arms:** obs arms map to v2 keys as WM-4 did (`gf240` → `gf240E`, `sp240` → `sp240L`, `rw120` → `rw120E`).
- **Multi-seed:** baselines and multi-seed models use `wm2.effects.average_errors`.
- **No retroactive changes:** WM-2/3/4 committed numbers must not change; their tests stay green.

## 4.2 Seeds and roles
- **obs2 (`game_v19`):** train **47000–48199** (1,200), val **48200–48399** (200), test **48400–48639** (240).
- **Unused check:**
  - `grep -rhoE '"seed": ?4[78][0-9]{3}\b' docs/worldmodel | sort -u` must be empty;
  - on the DGX, `find /home/hyunlord/work/worldsim-wm-data -maxdepth 4 -name '*_s4[78][0-9][0-9][0-9]*' | wc -l` must be 0.
  - Otherwise shift by +2,000 and record.

## 4.3 No leakage
- **Inputs:** exactly WM-2's lists (74 action-aware / 67 action-blind). Hidden fields are never inputs of any model or baseline except `ridge_priv`, which is labelled a diagnostic everywhere and never enters a method ranking.
- **obs2 training and model selection** see only each train/val world's observed episode. Val families choose among observational baselines only (disclosed). Test families are used only in evaluation.
- **Propensities:** the policy's are known; IPW and analyses may use them, models may not.
- **Direct regressions** (7.3) train on **v2 train families** (counterfactual labels, other worlds, same game, as `wm3_full`), choose λ on v2 val families, and are applied unchanged to the v2 and obs2 tests.

## 4.4 Reproducibility
As WM-4: config, code commit, manifest sha, seed, hyper-parameters, device, torch version, stop reason, planned and reached steps, and α at the kept checkpoint.

## 4.5 The gate
- **Mac:** `python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'` (recorder-dependent tests may skip).
- **DGX:** the same with **zero skips**.
- **Paths:** `git diff --name-only 64d633ff..HEAD` lists only section 2's paths.

---

# 5. PART 0 — Setup
- **Record:** start time, `uname -a`; on the DGX `nproc`, `nvidia-smi`, `free -h`, `df -h`, `uptime`.
- **Files:** commit `GOAL.md`; create `DECISIONS.md` and `RUNLOG.md`.
- **On the DGX:** sha256 of the recorder binary and of every earlier checkpoint used must equal their recorded values; run the seed check (4.2).
- Push.

---

# 6. PART 1 — The obs2 split (DGX)

## 6.1 The sequential multi-cast policy (`tools/worldmodel/wm5/policy2.py`, unit-tested; imports WM-4's frozen terciles, never recomputes them)
- **Decision rows** as WM-4: 06:00 of days 8–60. A world may cast **up to 5 times**.
- **Refractory period:** after a cast on day d, the next decision is on day d + 8 or later, so no second cast falls inside (c, c + 7 d]. Rows in the refractory window have p = 0, recorded.
- **Daily probabilities:** WM-4's table **× 2**, same tercile structure, so the confounding ratios are unchanged:

  | arm (kind, amount) | f < q_f1 | q_f1 ≤ f < q_f2 | f ≥ q_f2 |
  |---|---|---|---|
  | gift_food 240 | 0.060 | 0.016 | 0.004 |
  | spoil_food 240 | 0.004 | 0.016 | 0.060 |
  | rot_wood 120 | 0.024 if w ≥ q_w2 else 0.008 (all f) | | |

  Expected: ≈ 2–3 casts per world, ≈ 3 % of worlds never cast. Report the realized values.
- **The draw:** u = `default_rng([seed, day, 2])`'s first uniform (the `2` keeps it distinct from WM-4's stream); the choice as in WM-4's `choose`.
- **No decision** (p = 0, recorded) on a decision row with `g.alive` = 0 or no settlement.
- **The state is read from the observed trajectory:** after a cast, the next decisions read the episode recorded **with** that cast. Hence 6.2's rounds.
- **Propensity record** per world: every decision row's (p_g, p_s, p_r), every cast (arm, day, tick, row) and the round that produced it (`obs2/policy/policy.json`).

## 6.2 Recording in rounds
1. **Round 0 — factual.** `--scene game_v19 --seeds 47000..48640 --days 120 --every 60` into `obs2/rounds/r0/`; verify episodes (`wm_data.verify_episode`).
2. **METRICS.md part a** (section 8) committed and pushed before step 3.
3. **Round k ≥ 1.**
   - For every world, the policy reads its latest episode from the row after its last cast (or from day 8), stopping at the first cast or at day 60.
   - Worlds with a new cast get a schedule of all their casts so far, in `schedules_p14.py`'s format (`obs2/schedules/r<k>/s<seed>.json`), and are recorded into `obs2/rounds/r<k>/` with `--schedule-dir … ` for those seeds only, as WM-4 6.2.4 (with `--include-factual` if the recorder requires it for the call; record what it requires).
   - **Prefix check per world:** the new episode equals the previous round's episode on every row ≤ the new cast row (`global.npy` in full, settlements per WM-2 6.2's rule), and carries every earlier cast.
   - Repeat until a round adds no cast (at most 5 rounds).
4. **Observed episode** of a world = its last round's episode → `obs2/observed/` (symlinks); the manifest gives each world's final schedule.
5. **Families (val and test worlds; two reference rows each):**
   - **c1** = the world's first cast row, or day 30 06:00 if it never cast.
   - **c2** = its second cast row, or c1 + 14 days if it has none.
   - For each reference row c: a **`pre`** schedule = the world's casts at rows < c, and three arm schedules = `pre` + the arm at c (`gf240`, `sp240`, `rw120`, WM-4's amounts). Nothing after c.
   - **Val families:** `--fork-tick c --replicates 1`. **Test families:** `--replicates 2`. Replicate 0 is a no-op; replicate 1 injects the fork noise, as v2's floor families.
   - **One recorder call per (world, c)** into `obs2/families/{val,test}/c{1,2}/`. Find out how the recorder names members built from schedule files (WM-4 used `--include-factual` to get `none`; here the no-cast member is the `pre` schedule). Name and map the members in `wm5`'s own loader, and record the mapping.
   - **Checks:**
     - every member equals the observed episode on rows < c (per-array sha256 in `meta["arrays"]`, not `meta.json`);
     - every member carries the `pre` casts;
     - replicate 0 of `pre` equals a fresh unforked recording of the same schedule for 3 sampled worlds.
6. **Manifest and dataset note:**
   - **`docs/worldmodel/wm5/obs2_manifest.json`**, with WM-4's fields plus `ref` (c1/c2), `replicate`, the full schedule and the round.
   - **`DATASET_OBS2.md`:**
     - counts and casts per world;
     - arm shares per f-tercile (the confounding);
     - share of cast rows with an earlier cast in the history;
     - effective sample sizes (7.2);
     - the val families' true effect distribution per arm and lag. The test distribution is added only after part b.

**Expected volume:** ≈ 1,640 factual + ≈ 4,000 round episodes + val 200 × 2 × 4 + test 240 × 2 × 8 ≈ 10,000 episodes, ≈ 6 h at 8 workers. Record beside the v2 trainings of 9.1 at 4 workers. Commit, push.

---

# 7. PART 2 — Code (`tools/worldmodel/wm5/`)

## 7.1 Registry, cases, floors
- **Dataset:** `obs2_v19` registered as WM-4's `m1_obs.py` registers `obs_v19`, with train/val = observed episodes and test = observed test episodes for factual T2.
- **Obs2 test cases:** (world, ref, arm). Truth = arm r0 − `pre` r0 at c + 168 and c + 720 rows, for WM-2's four outcomes. Censoring as WM-2.
- **Arm mapping:** rewrite arms to v2 keys with `dataclasses.replace`, as WM-4 7.1.
- **Effect floor** (F1, F2): each case is predicted by the **other replicate's** true effect (arm r1 − `pre` r1 for an r0 case, and the reverse for an r1 case on v2). Scored with the same `cell_errors` and scales.
  - v2: its test families' replicates {0, 1}.
  - obs2: the test families' replicates (r0 truth is primary; r1 cases appear only in the floor).
  - This is the error of a predictor that knows the world's full state at the cast and only lacks the post-fork noise: the benchmark's ceiling.

## 7.2 Observational baselines on obs2 (multi-cast)
Risk set: every decision row c with c + h ≤ 2,880, for h = 168 and 720.
- **Treated T_a:** world w cast a at c and **no other cast in (c, c + h)**.
- **Controls C:** no cast in [c, c + h).
- **Weights** (history up to c is conditioned on; earlier casts are allowed in it):
  - treated: 1 / (p_a(c) · ∏_{j ∈ (c, c+h)} (1 − p_tot(j)));
  - controls: 1 / ∏_{j ∈ [c, c+h)} (1 − p_tot(j)).
  - Refractory rows contribute factor 1.
- **Estimators:**
  - `naive`: unweighted means;
  - `ipw`: Hájek-normalized difference;
  - `match5`: WM-4's rule, distances on obs2 train stds;
  - `zero`.
- **Effective sample size:** report it per arm and lag.
- **Unit test:** a synthetic multi-cast confounded policy with a known regime effect. `ipw` is unbiased; `naive` is biased; the test would fail if the post-cast survival factor were dropped.
- **Selection:** choose among the four per lag on obs2 val families (12-cell average).

## 7.3 Direct effect regressions (new)
- **`ridge_obs`:** one ridge regression per (arm, outcome, lag) cell.
  - **Target:** the true effect divided by the cell's scale.
  - **Features** (normalized by v2 train stds):
    - the 37 state features at c;
    - their means over rows c − 23 … c and c − 167 … c;
    - the 30 calendar features at c;
    - for obs2 cases, also the count of earlier casts per arm and the days since the last cast (0 on v2).
  - **λ** ∈ {0.01, 0.1, 1, 10, 100, 1,000}, chosen per cell on v2 val families. Closed form in numpy.
  - **Training data:** v2 train families, both replicates, every case.
- **`ridge_priv`** (diagnostic, droppable): the same plus the hidden fields at c, i.e. every recorded field that WM-4's leakage test (4.3) marks hidden, normalized. It bounds what the full pre-cast state predicts linearly.
- **Unit test:** recovers a planted linear effect; a feature-leakage test confirms `ridge_obs` uses no hidden field.

## 7.4 CRN and CT, as their authors train them (`wm5/causal5.py`, wrapping `wm4/causal.py`)
- **Step plan, not a time cap:**
  - Each stage trains for **planned steps** = the authors' optimizer steps: CRN encoder 15,700, CRN decoder 53,800, CT 11,850.
  - This is reduced only if a 300-s cost check on the actual dataset predicts more than the stage's budget (9.1). In that case: planned = floor((budget − evaluation allowance) / measured s per step), recorded.
  - The time cap becomes a safety net of 1.25 × budget. A run stopped by it is reported as such.
- **α schedule on real progress:** α = `alpha_at(step / planned_steps, α_max)`, so every completed run ends at α ≈ α_max. Assert in a unit test that a run of the planned length reaches ≥ 0.99 α_max.
- **Checkpoints:**
  - **Primary** = the authors' practice: the **last** step. For CT, its EMA weights (β 0.99); for CRN, the last encoder and decoder.
  - **Secondary:** the lowest-`val_roll` checkpoint (WM-4's rule), evaluated and reported.
- **Rounding at readout:** apply M1's rule to CRN/CT rollouts — round the `v` features, cumulative cold hours and cumulative deaths at the readout tokens (`round_counts` behaviour of `effects3._chunk_h` L149–150), not inside the feedback loop. Unrounded results are reported as secondary.
- **Configurations:** WM-4's selected configs (3.1); every other setting as WM-4's `METHODS.md`.
- **Daily tokens for obs2:** φ = 06:00, as `obs_v19`. A token's treatment carries every cast in its 24 rows, so earlier casts appear in the history.
- **`METHODS.md` (wm5):** D16 onward, each change against WM-4 with its reason.

## 7.5 Tests
`tools/worldmodel/wm5/test_wm5.py` covers:
- the policy: probabilities, refractory period, at most 5 casts, determinism, overlap;
- round schedules (all earlier casts kept; ticks multiples of 60) and the prefix check;
- the family schedules (`pre` + arm);
- the floor computation on a synthetic two-replicate family;
- 7.2's estimators under a synthetic multi-cast confounded policy;
- ridge recovery and leakage;
- α reaching α_max on a tiny planned run for both methods;
- last vs best checkpoint bookkeeping;
- rounding at readout.

Commit, push.

---

# 8. PART 3 — Pre-registration
`docs/worldmodel/wm5/METRICS.md` is committed in two parts:
- **Part a,** before round 1 (6.2.2): the policy, roles, seeds, the reference-row rule.
- **Part b,** before any test-role number of a WM-5 model or baseline: the runs and budgets of 9.1, selection rules, cases and scales, floors, the rules below.

"CI", "every seed" and the **W1 rule** (KEEP / WEAK / TIE / NONE) are as WM-3/4. "Primary" means last checkpoint, counts rounded. Each "vs" is one W1 verdict per pair; multi-seed sides use seed-averaged errors.

| # | rule |
|---|---|
| R1 | **obs2, 7 d (primary):** CRN-obs2, CT-obs2, M1b-obs2 vs the observational baseline best on obs2 val families at 7 d. |
| R2 | **obs2, 30 d:** the same at 30 d. |
| R3 | **v2, 7 d and 30 d:** CRN-v2, CT-v2 vs `zero` and vs `an5_arm`. |
| R4 | **training reached (must hold for R1–R3 to count as "trained as the authors"):** each primary run completed ≥ 90 % of its planned steps and its last checkpoint has α ≥ 0.9 α_max; the planned steps equal the authors' unless 7.4's cost rule reduced them (reported). A run failing R4 is reported under its verdicts as "under-trained". |
| F1 | **v2 headroom (reported with CI):** `zero` − floor, `zero` − `ridge_obs`, `ridge_obs` − `ridge_priv`, `ridge_priv` − floor, at 7 d and 30 d. |
| F2 | **obs2 headroom:** the same ladder on obs2 test (floor from its replicates). |
| E1 | **does the observable state carry the effect?** `ridge_obs` vs `zero` on v2 test and on obs2 test, 7 d and 30 d. |
| E2 | **models vs the regression:** each primary model vs `ridge_obs` on its test split, at 7 d and 30 d. Include `wm3_full` (three seeds) on v2 and obs2. |
| D1 | **more observational data (reported):** on obs2 test, CT-obs2 vs WM-4's `ct_obs` (best-val, unrounded as kept, and re-scored rounded) and M1b-obs2 vs WM-4's `m1b_obs`. |
| D2 | **rounding alone (reported):** WM-4's CRN/CT runs re-scored with rounding, on their own test splits (obs and v2), next to their WM-4 numbers. |
| S | **secondary (reported):** every model's best-val checkpoint and unrounded results; the per-cell tables. |

---

# 9. PART 4 — Training and evaluation (DGX, one training job at a time)
1. **Order and budgets.**
   - **First, while obs2 records:** `crn_v2_s0`, `ct_v2_s0`, `crn_v2_s1`, `ct_v2_s1`.
   - **Then, after obs2:** `m1b_obs2_s0`, `crn_obs2_s0`, `ct_obs2_s0`, `crn_obs2_s1`, `ct_obs2_s1`, `m1b_obs2_s1`.
   - **Budgets:** CRN ≤ 1.5 h per run (both stages; the authors' steps cost ≈ 45 min at WM-4's speed); CT ≤ 4 h per run (11,850 steps ≈ 2.7 h at 0.83 s/step, plus evaluations); M1b-obs2 8,000 steps ≤ 4.5 h.
   - **Cost check:** a 300-s check on the actual dataset before the first run of each kind; record s/step, GPU memory, an evaluation's wall time.
   - **`val_roll` evaluations:** at most 15 per stage, on at most 120 val episodes (a fixed sample, recorded) if the full val set makes one evaluation longer than 60 s.
2. **Direct regressions, floors and baselines:** on CPU or GPU whenever free. They need only v2 data and the obs2 families.
3. **Test evaluations:** each run once on test, only after part b is pushed — effects at 7 and 30 days, primary and secondary; factual T2 168 h for M1b-obs2.
4. **References on the obs2 test (no training):**
   - WM-2 `wm2_m1bact_v19_s{0,1,2}`, WM-3 `wm3_full_s{0,1,2}`;
   - WM-4's `m1b_obs_s{0,1}`, `crn_obs_s{0,1}`, `ct_obs_s{0,1}`, as kept.
5. **WM-4 re-scoring (D2):** WM-4's `crn_{obs,v2}_s{0,1}` and `ct_{obs,v2}_s{0,1}`, re-scored with rounding on their WM-4 test splits. WM-4's committed numbers stay as they are; these are new files labelled `rescored_rounded`.
6. **Run files:** copy each run's small files (never checkpoints) into `docs/worldmodel/wm5/runs/<run>/`.

# 10. PART 5 — Judge
- **`tools/worldmodel/wm5/judge5.py`:** every rule of Part 3 from the committed evaluation files, plus the headroom ladder per split and lag.
- **Per-cell tables:** every model and baseline side by side, including WM-2 … WM-4's numbers on their own splits.
- **Output:** `docs/worldmodel/wm5/runs/judge5/judge.md` + `judge.json`. Commit, push.

---

# 11. PART 6 — Analysis and the paper draft
1. **`docs/worldmodel/wm5/WM5.md`** (English):
   - obs2 (policy, rounds, confounding, multi-cast histories);
   - the rule table;
   - the headroom ladder;
   - CRN and CT trained as their authors train them, primary vs secondary;
   - the rounding effect alone;
   - the value of more observational data;
   - what changed relative to WM-4's conclusions.
   - **Add one line at the top of `docs/worldmodel/wm4/WM4.md`:** "Followed by docs/worldmodel/wm5/WM5.md (authors' training schedule for CRN/CT, multi-cast observational split obs2, headroom ladder) and the paper draft docs/worldmodel/paper/."
2. **The paper draft** `docs/worldmodel/paper/` (English):
   - `paper.md`: a full draft for a datasets-and-benchmarks style submission (≈ 9 pages of content plus appendix; the venue is the owner's choice and is left as `[VENUE]`).
   - `figures/`: generated by `tools/worldmodel/wm5/figures.py` from the committed judge JSON of WM-2 … WM-5, matplotlib only, PNG and PDF.
   - `references.bib`.

   **Sections of `paper.md`:**
   - **Abstract.**
   - **1 Introduction:** the question (can learned world models of an agent society predict the effects of a god's interventions?), the contributions.
   - **2 Related work:**
     - world models and learned simulators;
     - time-varying treatment-effect estimation (RMSN, CRN, CT, G-Net and others you can verify);
     - causal benchmarks and simulators (the tumour-growth simulator, semi-synthetic MIMIC benchmarks, causal RL environments you can verify);
     - agent-based social simulation.
   - **3 The benchmark:**
     - the simulator and scene;
     - interventions;
     - the three splits (v2 interventional with paired branches and replicates; obs with one cast; obs2 with a sequential multi-cast policy and known propensities);
     - the noise floor;
     - tasks, cells, scales, metrics, the W1 rule, pre-registration.
   - **4 Methods evaluated:** baselines, direct regressions, M1 world models, pair-trained M1, CRN, CT. Every adaptation goes in the appendix.
   - **5 Results:** one main table with every method on every split at 7 and 30 days; the headroom ladder figure; the per-cell figure.
   - **6 Analysis:**
     - why the methods fail where they fail;
     - confounding measured;
     - what the regressions say about observable vs hidden state;
     - the 30-day sign reversal.
   - **7 Limitations and broader impact.**
   - **Appendix:**
     - CRN/CT deviations D1–D20+;
     - hyper-parameters, budgets and reached steps;
     - the policies;
     - a datasheet for the dataset (Gebru et al. format);
     - compute.
   - **Data and code release:** a `[OWNER: release plan]` placeholder. Never claim a URL that does not exist.

   **Writing rules:**
   - Every number in the paper comes from a committed judge file. `figures.py` and a `paper_numbers.json` map each quoted number to its source file and key; a unit test checks that each quoted number appears in its source.
   - Claims are limited to what the rule table supports. Say "post hoc" where it applies.
   - **References:** add one only after verifying it on the web (arXiv, OpenReview, ACL Anthology, PMLR, publisher page): title, authors, venue, year, URL recorded in `docs/worldmodel/paper/REFERENCES_CHECK.md`. An unverifiable citation becomes `[CITATION NEEDED: topic]`, never an invented entry.
   - Authorship: `[AUTHORS]`.

---

# 12. PART 7 — Review, gate, report
1. **Independent review** — a fresh `general-purpose` subagent, two rounds maximum. Scope:
   - the multi-cast policy and its rounds (prefix checks, the state read from the observed trajectory);
   - family schedules (`pre` + arm) and replicates;
   - the floor computation;
   - leakage (no counterfactual member in obs2 training or model selection; no hidden field outside `ridge_priv`);
   - 7.2's weights by hand on one treated and one control case;
   - the step plan and α actually reached;
   - last vs best checkpoints;
   - rounding;
   - pre-registration order (METRICS commit times vs every test output's time on the DGX);
   - the verdicts;
   - **the paper:** every quoted number against `paper_numbers.json`, every reference against `REFERENCES_CHECK.md`, every claim against the rule table.

   Findings go in `docs/worldmodel/wm5/REVIEW.md`.
2. **Gate and publish:** gate (4.5) on both machines; fast-forward `codex/worldmodel-2026-10` if green; add the PR #512 comment.
3. **Report:** `docs/worldmodel/wm5/REPORT.md` in Korean (section 13 format), and the same text in the final message.

---

# 13. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
### 구현 내용
(Part별 파일과 핵심 구조)
### 구현 방법
(순차 다중 투입 정책과 녹화 라운드, 두 기준 행과 pre 일정, 노이즈 복제와 효과 바닥, 다중 투입 IPW, 직접 효과 회귀(관측·특권), CRN·CT의 계획 스텝·α 일정·마지막 체크포인트·반올림, 사전 등록, 논문 숫자 추적과 인용 검증)
### 기능 설명
연구 관점에서 무엇이 확인되었는가? 논문이 주장할 수 있는 것과 없는 것은?
### 변경된 파일 목록
(커밋별)
### 측정 결과
R1~R4·F1~F2·E1~E2·D1~D2 표(신뢰구간), 여유 사다리(바닥 ← ridge_priv ← ridge_obs ← zero, 분할·지평별), obs2의 교란(삼분위별 갈래 비율)·세계당 투입 수·이력 속 이전 투입 비율·유효 표본, 갈래·결과·지평별 효과 표(모든 방법), 실행별 계획/도달 스텝·α·종료 이유·시간, 주/보조 체크포인트 비교, 반올림 재채점
### 환경
### 독립 리뷰
### 자율 결정 목록
### 실행 기록
### 확인된 제한사항
### 게이트 결과
### 정직한 판단
저자 방식으로 학습한 CRN·CT는 효과 없음과 관측 기준선을 이기는가? 벤치마크는 판별력이 있는가(바닥과 zero 사이 여유, 관측 상태와 숨은 상태의 몫)? 관측 데이터를 늘리면 나아지는가? WM-4의 결론 중 바뀐 것은? 논문 초안은 제출 가능한 수준인가, 무엇이 남았는가?
```
The final message also includes:
- every commit hash with its Part;
- both branch heads, confirmed with `git ls-remote`;
- the PR #512 comment URL;
- the DGX paths of the obs2 data and every run;
- the paper draft's path and page estimate;
- the time per Part.
