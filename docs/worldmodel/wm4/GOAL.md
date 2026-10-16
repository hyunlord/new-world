# GOAL: WorldSim — WM-4: the benchmark as a paper needs it — a confounded observational split, the field's methods (CRN, Causal Transformer) on both splits, and the open ablations of WM-3

Repository `hyunlord/new-world`. Base: `codex/worldmodel-2026-10-wip` = `codex/worldmodel-2026-10` = `1bfb6ae0fc28ad62a3fde87d9bc202f087cc1ad0` (WM-3 final; draft PR #512).
This run is executed by **Claude Code on the Mac** in the WM clone, unattended; all recording, training and evaluation run **on the DGX Spark over ssh**. Another run (an art pass, Codex) may use the Mac: keep the Mac light (edits, small unit tests, git) and never take `/Users/rexxa/work/.mac_quiet.lock`.

WM-2 and WM-3 established, on dataset v2 (every arm cast on every world by a fixed rule, paired branches, a noise floor): an autoregressive world model ignores the god's interventions for reasons that are not a bug; training on paired branches teaches the first week's effects without hurting factual accuracy but only ties the analog baseline (V2 TIE), and no method learns the 30-day sign reversal or the split. Two holes remain before this is a benchmark paper: (1) **the field's own methods** — time-varying treatment-effect estimators, Counterfactual Recurrent Network (CRN, Bica et al., ICLR 2020) and Causal Transformer (CT, Melnychuk et al., ICML 2022) — are missing; (2) their reason to exist is **confounded treatment assignment**, which dataset v2 does not have. WM-4 records a confounded observational split on the same frozen world, runs CRN and CT on both splits next to the M1 models and observational baselines, and closes WM-3's two open items (ablation seeds, the hazard head's calibration).

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-wm1
git fetch origin && git checkout codex/worldmodel-2026-10-wip
git merge --ff-only origin/codex/worldmodel-2026-10-wip   # 1bfb6ae0
mkdir -p docs/worldmodel/wm4
cp ~/Downloads/claude_goal_worldsim_wm4.md docs/worldmodel/wm4/GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/worldmodel/wm4/GOAL.md>
```
Hooks stay neutralized in this clone (`.claude/settings.json` = `{}`, `--skip-worktree`); never edit, stage or commit it and never clear the flag.

**This document is `docs/worldmodel/wm4/GOAL.md`.** Commit it unchanged in Part 0. Where it disagrees with earlier WM goal documents, this one wins. Re-read the relevant section at the start of every Part and after every compaction or pause. The owner drives the run with `/goal`: a small model reads only the transcript; the section 13 report, the O/B/A/H table, commit hashes and `git ls-remote` heads must appear in your final message.

## 0.2 Overrides of CLAUDE.md
As WM-3's 0.2: no `install_hooks.sh`; never `lead/main`/`main`; no harness pipeline (4.5 is the gate); built-in `general-purpose` subagents only; a blocking hook other than `/goal` → record, push, report. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decisions in `docs/worldmodel/wm4/DECISIONS.md` (ids `L0.1`, …); times in `docs/worldmodel/wm4/RUNLOG.md`.
2. **Do not end your turn between Parts or while DGX jobs run.** Poll every 10–20 minutes with short calls; do other Parts' work meanwhile.
3. Review loops: two rounds maximum. Push after every Part.
4. On failure: work around, note it, continue. A negative result is valid; a missing number is not.
5. **Time rule:** past **40 hours** of wall time from the start (pauses excluded and recorded), start no new training; let running jobs end through their own `--max-seconds`; evaluate what exists; push; go to Part 7.
6. **Drop order:** the second seed of CT-v2 and CRN-v2 → CT-v2 and CRN-v2 → the second seed of every obs-trained model → the hazard recalibration (9.4) → the ablation seeds (9.3). Never drop Parts 0–3, the observational split, one seed each of M1b-obs, CRN-obs and CT-obs, their evaluations, or Part 7.
7. **DGX jobs are detached** with WM-3's watchdog and stack-dump-on-stall (`finetune.py`'s 600 s evaluation guard, `cc18dc3b`); one training job at a time. **Mac sleep:** if the Mac sleeps (WM-3 lost ≈ 2 h to a closed lid, `pmset` "Clamshell Sleep"), record the pause; DGX jobs continue.

---

# ⚠️ 1. MACHINES
- **Mac** (`/Users/rexxa/work/new-world-wm1`): edit, commit, push, unit tests on tiny synthetic inputs. No recording, training, evaluation or cargo builds. Never touch `/Users/rexxa/work/new-world-pass4`, `/Users/rexxa/work/new-world-art-prod`, `~/github/new-world-wt/lead` or other checkouts.
- **DGX** (`ssh hyunlord@100.70.109.50`; throwaway `ssh -F` config or the full address; never edit `~/.ssh/config`): code worktrees under `/home/hyunlord/work/wm1-runs/wm4/_code/<commit>` (WM-3's `code.sh` pattern with its lock); runs `/home/hyunlord/work/wm1-runs/wm4/<run>/`; venv `/home/hyunlord/work/wm1-venv`; recorder `/home/hyunlord/work/wm1-runs/_build/src_wm2` (the `game_v19` recorder of WM-2; **do not rebuild**); data: new `/home/hyunlord/work/worldsim-wm-data/obs/`; `v2/` and every other data directory **read-only**; WM-2 / WM-3 runs read-only (`wm2_m1bact_v19_s{0,1,2}`, `wm3_full_s{0,1,2}`, `wm3_ctrl_s0`, `wm3_os_s0`). Forbidden: `~/github/worldsim-training`, ComfyUI (8188), dev servers 3100/3200, other checkouts, processes you did not start.
- **Throughput (WM-2/3, measured):** M1b-act 1.54–1.58 s/step alone; WM-3 full 2.1–2.9 s/step; recording ≈ 2 s per episode at 8 workers; other users load the DGX (load average 1–63 in WM-3): record it, do not wait for it. Recordings `--workers 8` when no training runs, 4 while one does, `nice -n 10`; every recording sets `WORLDSIM_GIT_SHA=f5373ab2b7679c4a0d3a5c86b256971c86f93d41` (the recorder's source, as WM-2).
- **Network:** `pip install` works; cloning a public GitHub repository for reference code is allowed (record URL, commit, license).

---

# 2. GIT
- Commits to `codex/worldmodel-2026-10-wip`; `codex/worldmodel-2026-10` moves only by fast-forward to a WIP commit whose gate (4.5) is green on both machines. Never force-push. `git remote -v && git branch --show-current` before every push; `git ls-remote` after. No merges.
- Commit messages `worldmodel(wm4-N): …`. Stage explicit paths only.
- **May change:** the new `tools/worldmodel/wm4/**` and `docs/worldmodel/wm4/**`; one line at the top of `docs/worldmodel/wm3/WM3.md` (Part 6). Everything else is **read-only** (import and wrap; never edit). Third-party code, if used (6.4), lives under `tools/worldmodel/wm4/third_party/<name>/` with its license file and a `SOURCE.md` (URL, commit, files taken, every change).
- PR #512 stays draft; after the green fast-forward add a comment with the O/B/A/H table.

---

# 3. CONTEXT

## 3.1 What WM-3 left open (planner's review)
- **Audit:** structural mismatches 0 / 3,592, harness violations 0; the models see the action and answer with 3–37 % of the recorded one-hour jump. Cast windows alone (`os`) equal the control; the pair construction (`full`) is what helped.
- **V table (test, 12 cells):** V1 30 d `wm3_full` 0.511 vs `zero` 0.487 NONE; V2 7 d 0.496 vs `an5_arm` 0.485 TIE (post hoc: −0.096 vs WM-2, −0.091 vs control); V3 split shift TIE; V4 split timing NONE (expected day 34–49 days too early); V6 no factual harm at 168 h (24 h 1–3 % worse).
- **Open:** (a) the ablation has one seed each of `ctrl` and `os`; (b) the hazard head was never selected for (checkpoint selection read only `val_eff` and `val_t2`), so its probabilities are uncalibrated (predicted split within 30 d 0.76–0.99 vs observed 0.41; Brier 0.36–0.58 vs climatology 0.24); (c) the 30-day sign reversal (e.g. `sp240L` food: −90 at 7 d, +106 at 30 d) is outside the 168-step pair loss.

## 3.2 Why a confounded split
In dataset v2 every arm is cast on every world at a rule-fixed time: treatment assignment does not depend on the world's state, so methods whose contribution is correcting time-varying confounding (CRN's balancing representation via gradient reversal; CT's counterfactual domain confusion) have nothing to correct, and a reviewer will ask for the setting they were built for. The standard evaluation of these methods (the tumour-growth simulator with confounding strength γ) uses **observational training data whose treatments depend on the state** and **counterfactual truth at test**. The game's recorder can produce exactly that: a world's trajectory up to a cast is identical with and without the cast (WM-2 6.2), so a policy that reads the factual trajectory and casts once is a real sequential policy for a single decision.

## 3.3 Questions
- **Q1 (primary):** on the observational split, do CRN and CT predict intervention effects better than observational baselines and than an M1 model trained on the same data?
- **Q2:** how much does counterfactual training data (v2) help: WM-3 `full` (trained on paired branches) vs the obs-trained models on the obs test?
- **Q3:** on dataset v2, where do CRN and CT stand against WM-3's V table?
- **Q4 (closing WM-3):** with three seeds of each, is the 7-day gain the pair construction's? Does a selected-for hazard head forecast the split?

---

# 4. RULES
## 4.1 Comparability
WM-2/WM-3 harness, definitions and bootstrap (world-level paired, B = 2,000, 95 %, `default_rng(0)`) everywhere they apply. **Effect scales for the obs split:** per (outcome, lag), scale = max(the v2 arm's train std — `gf240` ↔ `gf240E`, `sp240` ↔ `sp240L`, `rw120` ↔ `rw120E` — , the v2 train std pooled over every arm's train cases for that (outcome, lag)), computed from `effects_base.json` `cases.train` (the floor stops near-constant v2 cells — e.g. `gf240E|alive|7d` 0.086, nonzero in 0.4 % of v2 cases — from letting one death dominate a 12-cell score); the per-cell table and the unfloored variant are reported as secondary; both frozen in METRICS part b. Multi-seed baselines (O3, A1) use `wm2.effects.average_errors` of their seeds. WM-2/3 committed numbers must not change; their tests stay green.
## 4.2 Seeds and roles
Observational split, `game_v19`: **train 46000–46399, val 46400–46479, test 46480–46559** (never used: `grep -rhoE '"seed": ?4[67][0-9]{3}\b' docs/worldmodel | sort -u` must be empty, and on the DGX `find /home/hyunlord/work/worldsim-wm-data -maxdepth 3 -name '*_s46[0-5][0-9][0-9]*' | wc -l` must be 0; else shift by +1,000 and record).
## 4.3 No leakage
Inputs exactly WM-2's lists (74 action-aware / 67 action-blind); hidden fields never inputs (unit test). In the obs split, **training and model selection see only the observed trajectory of each train/val world** (one episode per world); counterfactual members of val worlds are used **only** to choose among the observational baselines (6.3) — disclosed as conservative for the models; test counterfactuals only in evaluation. The policy's propensities are known (recorded) and may be used by the IPW baseline and analyses, not as model inputs.
## 4.4 Reproducibility
As WM-3 (config, code commit, manifest sha, seed, hyper-parameters, device, torch version, stop reason; GPU reproducibility not measured).
## 4.5 The gate
Mac: `python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'` (recorder-dependent tests may skip). DGX: the same with **zero skips**. `git diff --name-only 1bfb6ae0..HEAD` lists only section 2's paths.

---

# 5. PART 0 — Setup
Start time, `uname -a`, DGX `nproc`, `nvidia-smi`, `free -h`, `df -h`, `uptime`. Commit `GOAL.md`; create `DECISIONS.md`, `RUNLOG.md`. On the DGX: sha256 of the recorder binary and of every WM-2/WM-3 checkpoint used, equal to their recorded values. Push.

---

# 6. PART 1 — The observational split (DGX)

## 6.1 The policy (`tools/worldmodel/wm4/policy.py`, unit-tested)
- **Decision rows:** 06:00 of display days 8 … 60, tick = 1,440 d − 1,560 (day 11 → 14,280 = v2's fork tick; every tick a multiple of 60).
- **State read at a decision row** (from the world's factual episode, rows ≤ the decision row only): f = summed `s.food_stock` / max(1, `g.alive`) (food per person), w = summed `s.wood_stock` / max(1, `g.alive`).
- **Terciles:** q_f1, q_f2 (33rd / 67th percentiles of f) and q_w2 (67th of w) over every decision row of the **train** worlds' factual episodes; frozen in the first `METRICS.md` commit (8, part a) before any treated episode is recorded.
- **Daily cast probabilities** (a world casts at most once; after its cast no further decisions):
  | arm (kind, amount) | f < q_f1 | q_f1 ≤ f < q_f2 | f ≥ q_f2 |
  |---|---|---|---|
  | `gf240` (gift_food 240) | 0.030 | 0.008 | 0.002 |
  | `sp240` (spoil_food 240) | 0.002 | 0.008 | 0.030 |
  | `rw120` (rot_wood 120) | 0.012 if w ≥ q_w2 else 0.004 (all f) | | |
  Gifts go to hungry worlds, spoils to fed ones — the confounding a god-player would show. Every arm has positive probability in every state (overlap; weights up to 500 — report the effective sample size). The draw at a decision row: u = `default_rng([seed, day])`'s first uniform; cast `gf240` if u < p_g, else `sp240` if u < p_g + p_s, else `rw120` if u < p_g + p_s + p_r, else none. A decision row with `g.alive` = 0 or no settlement is not a decision (p = 0, recorded). Expected (mean daily hazard ≈ 0.033 over 53 rows): ≈ 0.17–0.19 of worlds never cast; arm shares among casts ≈ 40 % gift, 40 % spoil, 20 % rot (report the realized values).
- **Propensity record** per world: every decision row's (p_g, p_s, p_r), the chosen arm and its row (`policy.json`).

## 6.2 Recording
1. **Factual** `--scene game_v19 --seeds 46000..46560 --days 120 --every 60` (no schedule) into `obs/factual/`; verify episodes (`wm_data.verify_episode`).
2. Terciles (6.1) from the train factual episodes → `METRICS.md` part a (8) committed and pushed **before** step 3.
3. **Policy:** run it on every world's factual episode → `obs/policy/policy.json` and, for every casting world, one schedule file `obs/schedules/obs/s<seed>_obs.json` in `schedules_p14.py`'s format (a list of `{"tick", "kind", "settlement": "home", "amount"}`; "home" = the lowest settlement id).
4. **Observed episodes** (all 560 worlds): `--schedule-dir obs/schedules/obs --include-factual` (no fork; `--schedule-dir` without `--include-factual` errors on a seed without a file) into `obs/observed/`. Each world's `_none` episode must equal `obs/factual` array for array (a free determinism check); the observed episode of a casting world is its `_obs` recording, of a non-casting world its factual one. Check: rows ≤ the cast row equal the factual episode's (`global.npy` in full, settlements per WM-2 6.2's rule).
5. **Counterfactual families (val and test worlds):** reference row c_w = the policy's cast row, or for a world that never cast, the 06:00 row of day 30 (tick 41,640). Schedules for `gf240`, `sp240`, `rw120` at c_w in `obs/schedules/fam/s<seed>/`; **record each world separately** with `--seeds s..s+1 --fork-tick c_w --replicates 1 --schedule-dir obs/schedules/fam/s<seed>/ --include-factual --days 120 --every 60` into `obs/families/{val,test}/flat/`. This yields `game_v19_s<seed>_{none,gf240,sp240,rw120}_r0` with `noise_fork = {tick: c_w, replicate: 0}` — the names `wm_data.load_family` and `families_v2.parse_member` require (without a fork the members have no `_r<k>` and neither loads them); replicate 0 is a no-op (`sim-engine/src/lib.rs` ≈ L996–999), so the arrays equal an unforked recording. Then per-world directories with `families_v2.link`; prefix check per family. Compare an observed episode with a family member by the per-array sha256 in `meta["arrays"]`, not `meta.json` (`noise_fork`, `episode_id`, `schedule` differ). `families_v2.entry` / `build_manifest` assert fork 14,280 and replicates {0, 1}: wm4 writes its own manifest.
6. **Manifest** `docs/worldmodel/wm4/obs_manifest.json` (scene, seed, role, `split` — `obs_train` / `obs_val` for the observed episodes, `obs_test` = each test world's factual episode (for factual T2), `obsfam_<arm>` for family members — `schedule_id` (read by `data.load_split`), path, per-array sha256, rows, cast tick, applied amount, propensity file sha) and `DATASET_OBS.md`: counts, realized arm shares per f-tercile (the confounding made visible), never-cast share, effective sample sizes, and the per-arm true effect distribution on **val** families at 7 and 30 days (the test distribution is added only after METRICS part b is pushed).
Expected ≈ 560 factual + ≈ 1,020 observed (incl. `_none`) + 160 × 4 family members ≈ 2,200 episodes, ≈ 1.5 h at 8 workers. Commit, push.

---

# 7. PART 2 — Harness, baselines and the two methods (`tools/worldmodel/wm4/`)

## 7.1 Registry and cases
- `common.DATASETS`-style entries for `obs_v19` (train = observed train episodes; val = observed val episodes; test unused by training) and the existing `v2_v19`.
- **Obs test cases:** (world, arm, c_w) for the three arms; truth = arm member − `none` member at c_w + 168 and + 720 rows for the four WM-2 outcomes (food, alive, deaths, settlements alive) and the restricted split day shift; 12 cells per lag as W3 (4 outcomes × 3 arms). Censoring and exclusion of c + 30 d past the last row as WM-2.
- Loading: `wm_data.load_family` + `wm2.effects.family_cases` on `obs/families/{val,test}/*` (`ef.family_dirs` / `ef.load_cases` are hard-coded to v2), then each `Case`'s arm is rewritten to the v2 key (`gf240` → `gf240E`, `sp240` → `sp240L`, `rw120` → `rw120E`) with `dataclasses.replace`, because `effects3.predictions` uses `v2.ARM_SPEC[arm]` and `cell_errors` reads `stds[(arm, o, lag)]`; on-disk ids stay the obs names and every table labels arms with them. Effect evaluation for M1 models calls `wm3.effects3.predictions` directly (`effects3._load` refuses datasets other than `v2_v19`).
- `match5` and every an5 distance on the obs split use inv_std from the observed **train** episodes (`bench_v0.split_std`), not v2's.

## 7.2 Observational baselines (effects predicted directly; chosen per lag on **val families** by the 12-cell average)
- `zero`.
Stack every decision row c (days 8–60) with c + h ≤ 2,880. Treated T_a = {(w, c): w cast a at c}. Controls C = {(k, c): k cast at no decision row j < c + h} (not before c, not at c, c + 24, …, c + h − 24; a cast exactly at row c + h is allowed — that row is recorded before its cast). Δy = y(c + h) − y(c).
- `naive` = mean over T_a of Δy − mean over C of Δy.
- `ipw` = Σ_{T_a} Δy / p_a(c) ÷ Σ_{T_a} 1 / p_a(c) − Σ_C Δy / π_kc ÷ Σ_C 1 / π_kc, with π_kc = ∏_{decision rows j, c ≤ j < c + h} (1 − p_tot,k(j)) from `policy.json`. Estimand: the mean over risk-set pairs (w, c) of E[Δy_a − Δy_none]; the unit test asserts it on a synthetic confounded policy. Report the effective sample size (Σw)² / Σw² per arm.
- `match5`: for a query (world, a, c): the 5 nearest members of T_a (an5's 24-row distance at their cast rows) and the 5 nearest members of C at a row with the same display day as c; predicted effect = mean treated Δy − mean control Δy.

## 7.3 M1 models
- **M1b-obs:** WM-2's M1b-act recipe trained **from scratch on `obs_v19` train**, selection on val factual T2 24 h (the observational practice), seeds 0, 1, `--max-steps 8000`. Run through `tools/worldmodel/wm4/m1_obs.py`, which registers `c.DATASETS["obs_v19"] = c.Dataset("obs_v19", REPO / "docs/worldmodel/wm4/obs_manifest.json", {"train": ("obs_train", "train"), "val": ("obs_val", "val"), "test": ("obs_test", "test")}, ("train",), ("train",), ("val",), ("test",), None, None, None, ("T1", "T2", "T2_any"), True, None, c.V17_EXTRA)` (no edit of `wm1/common.py`; the argparse choices are built inside `main()`) and then calls `m1_tf.main([...])` with WM-2's flags from `wm2/launch/queue.sh`: `--variant sev-blind_action-aware --window 336 --context 168 --warmup 168 --ramp-steps 1500 --eval-every 150 --patience 12 --batch 32 --clamp-nonneg 1 --k-max 168 --roll-min-context 167 --roll-starts 1 --tf-l1 1.0 --threads 8 --v-weight 5.0 --val-batch 512`. Every process that loads an M1b-obs run registers the dataset first (`g.load_predictor` calls `c.dataset(config["dataset"])`).
- **References (no training):** WM-2 `wm2_m1bact_v19_s{0,1,2}` and WM-3 `wm3_full_s{0,1,2}` evaluated on the obs test (they were trained on v2's counterfactual data, same game, other worlds).

## 7.4 CRN and CT (daily resolution)
- **Daily tokens:** an episode is sampled every 24 rows at a clock phase φ. On the obs split every cast is at 06:00, so φ = 06:00 for training and evaluation (decision rows are tokens); on v2 casts are at 06:00 (`L`) or 18:00 (`E`), so training draws φ uniformly per sequence and evaluation uses the cast row's phase; both choices recorded in METHODS.md. The cast row and c + 7 d / c + 30 d are tokens. Token = the 74 inputs at that row with the action part replaced by the actions summed over the token's 24 rows (the cast falls in exactly one token).
- **Roles:** outcomes = the continuous state features (predicted as normalized daily changes and fed back); covariates = the calendar features at the token row (history only); treatments = a 4-class one-hot {none, gift, spoil, rot} plus amount / `ACTION_SCALE`; no static features. Alive, deaths and settlements alive are read from the fed-back outcomes.
- **Targets:** the next token's change of the continuous state features, normalized by train stds of the daily change (as WM-1's hourly normalizer, recomputed for daily steps); rollouts of 7 and 30 tokens with the planned treatments (the arm at the first token, none after).
- **CRN** (Bica et al. 2020): LSTM encoder producing a representation Φ_t trained with a gradient-reversal treatment classifier (λ schedule 2/(1 + e^(−10p)) − 1, p = training progress); outcome head on [Φ_t, treatment_t]; a decoder LSTM initialized from Φ for τ-step prediction given future treatments; teacher forcing then the paper's two-stage training (encoder, then decoder).
- **CT** (Melnychuk et al. 2022): transformer over the token sequence with separate input projections for covariates, treatments and outcomes, the counterfactual domain confusion loss (a treatment classifier trained to predict, the representation trained toward a uniform prediction), autoregressive multi-step prediction.
- **Source:** adapt the authors' public implementation (`github.com/Valentyn1997/CausalTransformer`, which contains CT and CRN; read its LICENSE at the vendored commit and record it) — vendor **plain-PyTorch modules only** (no Lightning, Hydra or MLflow) under `third_party/` (2.1) — or reimplement following the papers; **every deviation** (daily tokens, treatment classes, outcome vector, horizons, any architecture or training change) listed in `docs/worldmodel/wm4/METHODS.md`. **Never `pip install` into `/home/hyunlord/work/wm1-venv`** (it would move torch/numpy under WM-2/3's runs); an extra package goes into a separate venv created with `--system-site-packages` and installed with `pip install --no-deps`. Hyper-parameters: the paper's defaults for its synthetic benchmark, hidden size 128 (CRN) / d_model 128, 2 layers, 4 heads (CT); one tuning grid **on `obs_v19` only**, val factual loss only, at most 4 configurations per method, each capped at 20 minutes (the grid's time counts against the method's 1.5 h), chosen before any test number; the v2 runs reuse the obs-selected configuration.
- **Runs:** `crn_obs_s{0,1}`, `ct_obs_s{0,1}` on `obs_v19`; `crn_v2_s{0,1}`, `ct_v2_s{0,1}` on `v2_v19` (every member of the train families, as M1b-act); selection on val factual loss (`v19_fact` val only for v2).
- **Prediction contract** shared with the baselines: for a case (world, arm, c), predicted effect = rollout with the arm − rollout with none from the same observed history; outcomes read at the 7- and 30-token marks.

## 7.5 Tests
`tools/worldmodel/wm4/test_wm4.py`: the policy (probabilities per tercile, single cast, determinism, overlap), schedule rows (format, multiples of 60), the prefix check, every baseline on a synthetic world with a known effect (zero bias when the policy is randomized; `naive` biased and `ipw` unbiased under a synthetic confounded policy — a test that would fail if the weights were wrong), daily tokenization (cast token, phase alignment), CRN/CT forward and gradient-reversal sign on tiny inputs, the prediction contract. Commit, push.

---

# 8. PART 3 — Pre-registration
`docs/worldmodel/wm4/METRICS.md` in two commits: **part a** before step 6.2.3 (the policy, its frozen terciles, roles, seeds — so the policy cannot be tuned after any outcome is seen); **part b** before any test-role number of a WM-4 model exists (selection rules — models on val factual loss, baselines on val families; the runs of 9.1 with their hyper-parameters or tuning grids; the rules below). "CI", "every seed" and the W1 rule (KEEP / WEAK / TIE / NONE) as WM-3. Cells: 12 per lag (4 outcomes × 3 arms; the split shift is not one of them). In O3 the model side is `wm3_full`; in A1 the model side is `wm3_full` and the baseline is `wm3_ctrl` or `wm3_os` (seed-averaged errors, 4.1).

| # | rule |
|---|---|
| O1 | **obs, 7 d (primary):** each of CRN-obs, CT-obs, M1b-obs (every seed) vs the observational baseline best on val families at 7 d: the W1 rule. |
| O2 | **obs, 30 d:** the same at 30 d. |
| O3 | **value of counterfactual training (obs test, 7 d and 30 d):** WM-3 `wm3_full` (3 seeds) vs each obs-trained model (CRN-obs, CT-obs, M1b-obs; their factual losses are at different resolutions, so no pick among them): the W1 rule, one verdict per pair. |
| O4 | **confounding visible (reported):** `naive` vs `ipw` vs `zero` on obs test, per arm. |
| B1 | **v2, 7 d:** CRN-v2 and CT-v2 vs `an5_arm` (WM-3's V2 baseline) and vs `wm3_full`: the W1 rule. |
| B2 | **v2, 30 d:** CRN-v2 and CT-v2 vs `zero`: the W1 rule. |
| A1 | **ablation, three seeds (v2 test, 7 d and 30 d):** `wm3_full` vs `wm3_ctrl` and `wm3_full` vs `wm3_os`, each with seeds 0–2 (WM-3's s0 runs plus the new s1, s2 of 9.3): the W1 rule. |
| H1 | **hazard, selected (v2 test):** recalibrated heads (9.4) — split timing (WM-3 V4's task) vs `ridge_day`, and the split shift (V3's) vs `an5_arm`: the W1 rule; Brier and the 5-bin calibration vs climatology reported. |

---

# 9. PART 4 — Training and evaluation (DGX, one training job at a time)
1. **Order:** the GPU is free while Parts 1–2 are coded, so **9.3 then 9.4 start right after Part 0** (they need neither the obs split nor new code beyond 9.4's script; their test evaluations wait for METRICS part b). Then `m1b_obs_s0`, `crn_obs_s0`, `ct_obs_s0`, `m1b_obs_s1`, `crn_obs_s1`, `ct_obs_s1`, `crn_v2_s0`, `ct_v2_s0`, `crn_v2_s1`, `ct_v2_s1`. Before the first of each kind, a 100-step cost check (s/step, GPU memory, an evaluation's wall time); every run `--max-seconds` = min(its budget, deadline − now − 3,600). Budgets: M1b-obs 8,000 steps (≈ 3.5 h); CRN and CT at daily resolution ≤ 1.5 h each including their grid.
2. Evaluate each run once on test (effects at 7 and 30 days; factual T2 168 h for the M1 models) only after Part 3's commit is pushed; WM-2/WM-3 references on the obs test likewise.
3. **Ablation seeds (v2):** `wm3_ctrl_s1`, `wm3_ctrl_s2`, `wm3_os_s1`, `wm3_os_s2` with WM-3's exact recipe (`wm3/finetune.py`, inits `wm2_m1bact_v19_s1/s2`), selection and evaluation unchanged.
4. **Hazard recalibration** (`tools/worldmodel/wm4/recal_hazard.py`; retraining the head on the same teacher-forced target would change little — its miscalibration shows on 720-step rollouts, whose states differ): for `wm3_full_s{0,1,2}`, Platt-recalibrate h' = σ(a · logit h + b), fitting (a, b) by maximum likelihood of "split within 30 days" (1 − ∏(1 − h'_k) against the truth) on frozen-trunk 720-step zero-action rollouts from every **train**-family `none_r0` W3 cast row (`effects3.rollout_jobs_h`). Write the result into the head (`haz.weight ← a · W`, `haz.bias ← a · b0 + b`; `wm3.model.HAZ_KEYS`) in a new run directory `wm4_haz_s<k>` holding a copy of the run's `config.json` and `val_metrics.json`, so `effects3` split / calib / model work unchanged. Keep it only if its val Brier (`effects3.calibration` on val families) is below the original's (recorded either way). Then V3/V4's evaluations with WM-3's code; report test calibration for the original heads too (WM-3 reported only val).
5. Copy each run's small files (never checkpoints) into `docs/worldmodel/wm4/runs/<run>/`.

# 10. PART 5 — Judge
`tools/worldmodel/wm4/judge4.py`: every rule of Part 3 from the committed evaluation files; per-arm, per-outcome, per-lag tables with every model and baseline side by side; `docs/worldmodel/wm4/runs/judge4/judge.md` + `judge.json`. Commit, push.

# 11. PART 6 — Analysis
`docs/worldmodel/wm4/WM4.md` (English): the obs split (policy, confounding made visible, overlap); the O/B/A/H table; the effect tables; the value of counterfactual data; where CRN and CT stand on both splits; the closed ablation and the hazard head; and **"The paper"** — a one-page outline for a datasets-and-benchmarks submission: the benchmark (v2 interventional split with paired branches and noise floor, the obs split with known propensities), the tasks and metrics, the results table of every method, the claims the evidence supports and those it does not, and the limitations. One line at the top of `docs/worldmodel/wm3/WM3.md`: "Followed by docs/worldmodel/wm4/WM4.md (observational split; CRN and CT; ablation seeds; hazard recalibration)."

# 12. PART 7 — Review, gate, report
1. Independent review (fresh `general-purpose` subagent, two rounds maximum): the policy and its confounding (arm shares by tercile), prefix identity of observed and family episodes, leakage (no counterfactual member read by training or model selection), the baselines' formulas on one case by hand (`ipw` weights especially), the CRN/CT deviations list against the papers, the third-party license, pre-registration order (METRICS.md commit time vs every test output's time on the DGX), the verdicts. Findings in `docs/worldmodel/wm4/REVIEW.md`.
2. Gate (4.5) on both machines; fast-forward `codex/worldmodel-2026-10` if green; PR #512 comment.
3. `docs/worldmodel/wm4/REPORT.md` in Korean (section 13 format) and the same text in the final message.

---

# 13. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
### 구현 내용
(Part별 파일과 핵심 구조)
### 구현 방법
(정책과 교란, 관측 분할 녹화와 가족, 관측 기준선(naive·ipw·match5), CRN·CT 이식과 편차, 일 단위 토큰, 사전 등록, 절제 시드, 위험도 재보정)
### 기능 설명
연구 관점에서 무엇이 확인되었는가? 논문이 주장할 수 있는 것과 없는 것은?
### 변경된 파일 목록
(커밋별)
### 측정 결과
O1~O4·B1~B2·A1·H1 표(신뢰구간), 관측 분할의 교란(삼분위별 갈래 비율)·무투입 비율, 갈래·결과·지평별 효과 표(모든 방법과 기준선), 반사실 학습 데이터의 가치, 위험도 Brier·보정 전후, 실행별 종료 이유·최적 스텝·시간
### 환경
### 독립 리뷰
### 자율 결정 목록
### 실행 기록
### 확인된 제한사항
### 게이트 결과
### 정직한 판단
교란이 있는 관측 데이터에서 CRN·CT는 단순 기준선과 M1보다 나은가? 반사실 데이터(짝 갈래)는 얼마나 가치가 있는가? 데이터셋 v2에서 이 분야 방법들은 어디에 서는가? WM-3의 7일 개선은 짝 구성 덕인가(세 시드)? 논문은 지금 낼 수 있는가, 무엇이 남았는가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #512 comment URL, the DGX paths of the obs data and every run, and the time per Part.
