# GOAL: WorldSim — WM-3: why the world model ignores the god's hand, and whether the paired branches teach it

Repository `hyunlord/new-world`. Base: `codex/worldmodel-2026-10-wip` = `codex/worldmodel-2026-10` = `3cde2e9198995850f413e0c2496173584b590884` (WM-2 final; draft PR #512).
This run is executed by **Claude Code on the Mac** in the WM clone, unattended; every training and evaluation job runs **on the DGX Spark over ssh**. A simulation run (Claude Code, `new-world-pass4`) and an art run (Codex, `new-world-art-prod`) may use the Mac at the same time and measure times there: keep the Mac light (edits, small unit tests, git).

WM-2 found that the action-aware transformer (M1b-act) predicts the effect of the god's interventions no better than "no effect" (W3 NONE: 0.501 vs `zero` 0.487) and never predicts a split. Its own diagnostic is stranger than "too rare to learn": one hour after a gift of 240 food the true effect is +237, the models predict +6 or −12; one hour after rotting 120 wood (which does not touch food) they predict −20 to −67 food. A one-step response with the wrong sign is either a bug or a training problem, and the two need different next steps. WM-3 first decides which (Part 1), then tests one pre-registered idea: train on the dataset's unique asset — **paired branches that share their noise** (an arm member and the factual member of the same replicate are identical up to the cast and differ afterwards only through the intervention) — with windows that contain the cast, a loss on the paired difference, and a split-hazard head.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-wm1
git fetch origin && git checkout codex/worldmodel-2026-10-wip
git merge --ff-only origin/codex/worldmodel-2026-10-wip   # 3cde2e91
mkdir -p docs/worldmodel/wm3
cp ~/Downloads/claude_goal_worldsim_wm3.md docs/worldmodel/wm3/GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/worldmodel/wm3/GOAL.md>
```
Hooks stay neutralized in this clone (`.claude/settings.json` = `{}`, `--skip-worktree`); never edit, stage or commit it and never clear the flag.

**This document is `docs/worldmodel/wm3/GOAL.md`.** Commit it unchanged in Part 0. Where it disagrees with the WM-1/WM-2 goal documents, this one wins. Re-read the relevant section at the start of every Part and after every compaction or pause. The owner drives the run with `/goal`: a small model reads only the transcript; the section 13 report, the V table, commit hashes and `git ls-remote` heads must appear in your final message.

## 0.2 Overrides of CLAUDE.md
As WM-2's 0.2: no `install_hooks.sh`; never `lead/main`/`main`; no harness pipeline (4.5 is the gate); built-in `general-purpose` subagents only; a blocking hook other than `/goal` → record, push, report. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decisions in `docs/worldmodel/wm3/DECISIONS.md` (ids `K0.1`, …); times in `docs/worldmodel/wm3/RUNLOG.md`.
2. **Do not end your turn between Parts or while DGX jobs run.** Poll every 10–20 minutes with short calls; do other Parts' work meanwhile.
3. Review loops: two rounds maximum. Push after every Part.
4. On failure: work around, note it, continue. A negative result is valid; a missing number is not.
5. **Time rule:** past **36 hours** of wall time from the start (pauses excluded and recorded), start no new training; let running jobs end through their own `--max-seconds`; evaluate what exists; push; go to Part 7.
6. **Drop order:** `wm3_full_s2` → `wm3_os_s0` → `wm3_full_s1`. Never drop Part 1, `wm3_full_s0`, `wm3_ctrl_s0`, the test evaluations of what was trained, or Part 7.
7. **DGX jobs are detached** (`setsid nohup … > log 2>&1 &`, PID and `.exit` recorded), with WM-2's 30-minute silent-log watchdog. **Connection loss:** poll every 10 minutes for up to 2 hours while doing Mac-side work; never call a run lost before checking it after reconnecting.

---

# ⚠️ 1. MACHINES
- **Mac** (`/Users/rexxa/work/new-world-wm1`): edit, commit, push, unit tests on tiny synthetic inputs. No training, evaluation or cargo builds. Never touch `/Users/rexxa/work/new-world-pass4`, `/Users/rexxa/work/new-world-art-prod`, `~/github/new-world-wt/lead` or other checkouts, and never take `/Users/rexxa/work/.mac_quiet.lock` (you run nothing timed on the Mac).
- **DGX** (`ssh hyunlord@100.70.109.50`; throwaway `ssh -F` config or the full address; never edit `~/.ssh/config`): code clone `/home/hyunlord/work/new-world-wm1` detached at a pushed WIP commit before every job (`WM3_CODE_COMMIT` recorded); runs `/home/hyunlord/work/wm1-runs/wm3/<run>/`; venv `/home/hyunlord/work/wm1-venv`; data `/home/hyunlord/work/worldsim-wm-data/v2/` (**read-only**: WM-3 records nothing); WM-2 runs `/home/hyunlord/work/wm1-runs/wm2/` (**read-only**: the three `wm2_m1bact_v19_s{0,1,2}/best.pt` are the initial checkpoints; `effects_base/`, `split_base/` are reused). Build clone `_build/src_wm2` is used only by the gate. Forbidden: `~/github/worldsim-training`, ComfyUI (8188), dev servers 3100/3200, other checkouts, processes you did not start.
- **Throughput (WM-2, measured):** M1b-act 1.54–1.58 s/step alone (GPU 21–23 GB); a batched validation pass 7.6 s. **Train one model at a time.** `nvidia-smi`, `free -h`, `df -h`, `uptime` before every job; other users load the DGX (load average 2–37 in WM-2): record it, do not wait for it.

---

# 2. GIT
- Commits to `codex/worldmodel-2026-10-wip`; `codex/worldmodel-2026-10` moves only by fast-forward to a WIP commit whose gate (4.5) is green on both machines. Never force-push. `git remote -v && git branch --show-current` before every push; `git ls-remote` after. No merges.
- Commit messages `worldmodel(wm3-N): …`. Stage explicit paths only.
- **May change:** the new `tools/worldmodel/wm3/**` and `docs/worldmodel/wm3/**`; one line at the top of `docs/worldmodel/wm2/WM2.md` (Part 6). Everything under `tools/worldmodel/wm1/**`, `tools/worldmodel/wm2/**` and every other path is **read-only**: WM-3 imports and wraps that code; it never edits it. Exception, only through Part 1's erratum branch (6.4): a bug fix in `tools/worldmodel/wm2/**` or `wm1/**`, with a test that fails before and passes after, recorded in `docs/worldmodel/wm3/ERRATUM_WM2.md`.
- PR #512 stays draft; after the green fast-forward add a comment with the V table.

---

# 3. CONTEXT (planner's review of WM-2)

## 3.1 What WM-2 established
- Process was clean: METRICS.md (`b57ef7bb`, 02:15) precedes every test number; its one amendment is report-only.
- **Factual:** M1b-act beats `an5_off` by 5–10 % (W1 WEAK); headroom to the full-state floor at 168 h is +0.059 (benchmark not saturated).
- **Effects:** W3 (12 cells: 4 outcomes × `sp240L`, `gf240E`, `rw120E` at c + 30 days, scaled effect MAE) — M1b-act 0.501 (s0 0.510, s1 0.491, s2 0.503) vs `zero` 0.487 (chosen on validation: `zero` 0.510, `arm_mean` 0.633, `an5_arm` 0.644): NONE. Between-replicate effect spread of the same cells 0.189. At 7 days `arm_mean` beats the model by +0.095 [+0.044, +0.151] (not pre-registered).
- **Split:** no model ever predicts a split (rolled `g.fissions_total` rises ≤ 0.11 in 96 days); W4 TIE, W5 NONE, W5b a degenerate [0, 0].
- **Diagnostic (`docs/worldmodel/wm2/runs/diag/effect_lag.json`, the first 40 validation families (`diag_effect_lag.py --worlds 40`), lags 1, 2, 6, 24, 72, 168, 720 rows):** true food effect at lag 1: `sp240L` −221.7, `sp120L` −122.2, `gf240E` +237.4, `rw120E` 0.0. Predicted at lag 1: `sp240L` −23.8 / −44.7 (M1b-act s0 / s1), `gf240E` +6.3 / −12.2, `rw120E` −19.7 / −66.8.

## 3.2 What the planner checked in the code (so you do not repeat it, but do verify it in Part 1)
- **Recorder** (`rust/crates/sim-test/src/worldmodel/export.rs:1-6, 345-356` on the simulation line): a row is taken at every tick multiple of 60 *before* that tick's `apply_due`, so the action at row c changes row c + 1.
- **Training actions** (`bench_v1.episode_arrays:96-100`): the requested amount at the row whose `tick` equals the intervention's tick, divided by `data.ACTION_SCALE` (food 120, wood 80) in `data.prepare`; `dtarget[t]` = state[t + 1] − state[t]. So the training pair (action at c, jump at c → c + 1) is aligned.
- **Evaluation actions** (`wm2/effects.py:439-448`): the arm job gets `act[cast_row, kind] = amount` on the factual episode `none_r`; `wm2/rollout_ext.py:97-103` (transformer) encodes rows start − 167 … start including row start's action, and the first step uses that output. Aligned too.
- **Where the signal is thin:** a window of 336 rows contains a cast in ≈ 10 % of draws (336 / 2,545 window starts × 2,980 / 3,780 arm episodes), and that one row carries 1/336 of a window's teacher-forced loss; early stopping reads validation T2 24 h on the val factual and arm-r0 members (`common.py:113`), whose weekly 06:00 starts (`bench_v0.py:212-223`: days 3, 10, 17, …) almost never have a cast (days 12, 21, 29 or 57) inside their 24 h, so it barely rewards an effect.
- **Paired noise:** a family's arm member and factual member of the same replicate r share the post-fork noise stream; they are identical up to row(c) (WM-2 6.2 checked this for every family) and the simulation is deterministic, so arm_r − none_r after c is the intervention's effect on that trajectory, not effect plus independent noise. No other training signal in the dataset isolates the effect this cleanly.

## 3.3 The questions
- **Q0 (audit):** is the wrong-signed one-step response a bug (data, harness) or a property of the trained models?
- **Q1 (primary):** does fine-tuning M1b-act on cast windows with a paired-difference loss predict intervention effects better than the best simple baseline (W3 as in WM-2)?
- **Q2:** does a split-hazard head turn "never predicts a split" into a useful split-shift and split-timing prediction?
- **Q3 (ablation):** is any gain the objective's, or would more training with WM-2's objective under the same effect-aware checkpoint selection give it too?

Not in WM-3 (WM-4): literature baselines (CRN, Bica et al. 2020; Causal Transformer, Melnychuk et al. 2022). Their contribution is balancing representations against time-varying confounding; in dataset v2 every arm is cast on every world by a fixed rule, so there is no treatment-selection bias for them to correct. WM-4 adds an observational split with confounded assignment, where they belong. State this in WM3.md.

---

# 4. RULES

## 4.1 Comparability
WM-2's harness, definitions, roles, test cases (`effects_base.json`: 604 test cases) and bootstrap (world-level paired, B = 2,000, 95 %, `default_rng(0)`) everywhere they apply. WM-2's committed numbers must not change: its tests stay green and no WM-2 file changes (section 2).

## 4.2 Roles and seeds
Dataset v2 roles unchanged: `game_v19` train 40000–40399, val 40400–40479, test 40480–40559. Training reads train families only; checkpoint selection reads val families only; test is read once per run after the pre-registration (Part 3) is pushed.

## 4.3 No leakage
Inputs exactly WM-2's action-aware list (74 = 67 state and calendar + 7 actions; `runs/wm2_m1bact_v19_s0/config.json` `input_names`); a unit test asserts it and the absence of every hidden field. The hazard head's target `g.fissions_total` is an observable input column already. The cast row, arm and replicate of a training window come from the train families' `interventions.json` (observable), never from test.

## 4.4 Reproducibility
As WM-2 (`config.json` with code commit, init checkpoint sha256, manifest sha, seed, every hyper-parameter, device, torch version; `train_log.jsonl`; `val_metrics.json`; stop reason). GPU reproducibility not measured.

## 4.5 The gate
Mac: `python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'` (recorder-dependent tests may skip). DGX: the same discovery in the venv **with zero skips**, `WM_BIN`/`PROBE_BIN` pointing at `_build/src_wm2`, `WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data`. `git diff --name-only 3cde2e91..HEAD` lists only the paths of section 2. `tools/worldmodel/wm3/__init__.py` mirrors `wm2/__init__.py` (puts `tools/worldmodel` on `sys.path`) so discovery finds `wm3/` tests.

---

# 5. PART 0 — Setup
1. Record start time, `uname -a` of both machines, the DGX's `nproc`, `nvidia-smi`, `free -h`, `swapon --show`, `df -h`, `uptime`. Commit `GOAL.md`; create `DECISIONS.md`, `RUNLOG.md`.
2. On the DGX: sha256 of the three WM-2 `best.pt` and of `wm2/effects_base/effects_base.json`; the latter must equal the committed `docs/worldmodel/wm2/runs/effects_base/effects_base.json` (else stop and record). Push.

---

# 6. PART 1 — Audit (DGX, no training; train and val only)

Write `tools/worldmodel/wm3/audit.py`. Every number in this Part comes from train or val families; no test family is opened.

Members: `EpisodeData.eid` is the arm name only (`schedule_id`); the replicate comes from `ep.split` (`v19_fact` / `v19_arm_r0` → 0, `v19_none_r1` / `v19_arm_r1` → 1); a case's partner is the member of the same seed with `eid == "none"` and the same replicate.

1. **A1 data.** For every train and val case (`wm2.effects.load_cases` gives seed, arm, replicate, cast tick and row). **Structural checks:** in the arm member's `d.prepare` output the action matrix is non-zero exactly at the cast row and in the arm's kind column, with value requested / `ACTION_SCALE`; the partner's action matrix is all zero; rows ≤ cast row of `x_state` are equal between arm and partner; the arm − partner stock difference at row c is exactly 0 (off-by-one check). **Behavioural check:** the arm − partner difference of the stock at row c + 1 (summed `s.food_stock` for food arms, summed `s.wood_stock` for `rot_wood`) divided by s_kind × applied, with s_kind = +1 for `gift_food` and −1 for `spoil_food` and `rot_wood` (`interventions.json` records `applied` as a positive amount for every kind), lies in [0.5, 1.5] — not equality: the hour after the cast lets behaviour move it a little (WM-2's validation mean for a 240 gift is +237.4). Report counts of every kind of mismatch and the distribution of the ratio per arm.
2. **A2 one-step response of the WM-2 checkpoints.** For each of the three M1b-act checkpoints and each train and val case: the teacher-forced predicted mean change of the 7 continuous V variables at the cast row (`model.run` on the 168-row window ending at the cast row, built exactly as `rollout_ext` builds the transformer warm-up: rows c − 167 … c, `valid` = row ≥ 0), μ × `norm.dstd` in raw units, and the fire probability separately; for the arm and the partner. Report per arm and role: mean true arm − partner jump, mean predicted arm − partner jump, their correlation over cases, and the share of cases with the right sign.
3. **A3 harness consistency.** For the val cases of replicate 0, with `pred.round_counts = False`: the rollout's lag-1 difference (cur_A,1 − cur_F,1) from `wm2.rollout_ext.rollout_jobs` (one partner job and one arm job per case, horizon 1, actions as `effects.model_predictions` builds them) is compared, per continuous V, with `Stepper.next_state(s_c, μ_A, fire_A) − Stepper.next_state(s_c, μ_F, fire_F)` built from A2's outputs (the same clamp to the train range); tolerance 1e-4 × max(1, |s_c|) + 1e-3 (float32). Also a unit test on a synthetic stub model (it must satisfy `run`'s kv/valid contract and expose `blocks` so `rollout_ext._is_tf` takes the transformer path; its food μ = food action input × 120 / dstd[food], negative for `spoil_food`) where the harness returns exactly the stub's jump at lag 1.
4. **Decision (recorded in DECISIONS.md as `K1.decision`, with the numbers):**
   - **Any structural A1 mismatch, or any A3 violation:** a bug.
   - **Behavioural A1 ratio outside [0.5, 1.5] in more than 0.5 % of cases:** inspect 5 such cases by hand (rows c, c + 1, c + 2 of both members, and `interventions.json`); a bug only if the stock jump appears at c + 2 or the recorded applied amount disagrees with the jump; otherwise record it as behaviour and treat A1 as clean.
   - **A bug:** Find it, fix it in place (section 2's exception) with a failing-then-passing test, recompute WM-2's affected test evaluations with the fixed code into `docs/worldmodel/wm3/erratum/` (WM-2's committed files stay as they were), write `ERRATUM_WM2.md` (what was wrong, which W rows change and by how much), then continue with Part 2 on the fixed code.
   - **A1 and A3 clean:** the wrong response is the trained models' (A2 says how wrong at one step); continue with Part 2 unchanged.
5. Commit `docs/worldmodel/wm3/AUDIT.md` (the tables and the decision) and `runs/audit/*.json`. Push.

---

# 7. PART 2 — The paired fine-tuning harness (`tools/worldmodel/wm3/`)

Definitions: a **case** is (train world, arm, replicate r) as in `wm2.effects`; its **arm member** and **partner** (the factual member `none_r` of the same world) are identical through the cast row c (asserted once per case at load, A1). Window length W = 336, context 168; the **cast position** p = 167 (the last teacher-forced row of a rollout start; WM-2's runs use `--roll-min-context 167`, so their rollout starts are rows 167–168 of a window).

1. **Model with a hazard head** (`wm3/model.py`): `M1TFH(m1_tf.M1TF)` adds `self.haz = nn.Linear(d_model, 1)` (initialized after `torch.manual_seed(seed)`) and `run_h(x, cache=None, valid=None)` → `(mu, logvar, fire_logit, haz_logit, kv)`, computed from the same final layer-normed `z` as the other heads. `run()` is overridden to return `run_h()` without `haz_logit`, so every WM-1/WM-2 caller works unchanged; a test asserts that `M1TFH` loaded with a WM-2 state dict (strict, except the two `haz.*` keys, asserted to be the only missing ones) returns `mu`, `logvar`, `fire` bit-identical to `M1TF` on a fixed input (CPU). **Loading for evaluation:** every existing entry point (`effects.py model`, `split_task.py model`, `m1_gru.final`, `diag_effect_lag.py`) goes through `g.load_predictor` → `kind_of(config["model"])`, which cannot build `M1TFH`. So importing `wm3.model` registers `g.KINDS["m1_tfh"] = g.Kind("m1_tfh", m1_tf.DEFAULTS, <build M1TFH>, <loss>, m1_tf._predictor)` (in the module object `wm1.m1_gru`); every WM-3 run (full, ctrl, os) writes a `config.json` with `model: "m1_tfh"`, `dataset: "v2_v19"`, `variant`, `input_names`, `normalizer`, `hyper` (d_model, layers, heads, context, warmup, clamp_nonneg, threads and every WM-3 setting), `round_counts`, `best_step`, `stop`, `selected`; every WM-3 evaluation script imports `wm3.model` before `g.load_predictor`, and `wm3/final_fact.py` imports it and calls `g.final` with `--eval-dataset v2_v19_fact` (the factual T2 168 h of V6). A test round-trips a tiny `m1_tfh` run directory through `g.load_predictor`.
2. **Cast-window sampler** (`wm3/sampling.py`): a batch is `n_plain` ordinary windows (`d.sample_windows`, unchanged, drawn from `np.random.default_rng([seed, 0])`) plus `n_cast` cast windows. A cast window: draw a train case uniformly (all four arms, both replicates; `np.random.default_rng([seed, 1])`), start s = c − p (cast rows span 262–1,390 in dataset v2, so 95 ≤ s and s + 337 ≤ 2,881 always) (assert s ≥ 0 and s + W + 1 ≤ rows), the arm member's rows s … s + W; with the pair loss also the partner's same rows. Tested: the action is at window position p; partner and arm rows equal at positions ≤ p.
3. **Loss** (`wm3/losses.py`), built from `m1_tf.losses_tf`'s arithmetic with the rollout start positions passed in explicitly (plain windows: drawn as WM-2 draws them; cast windows and partners: exactly p). A test asserts that with `n_cast = 0`, `pair = 0`, `hazard = 0` and the same draws it returns `losses_tf`'s values (CPU, `torch.equal` on the scalar). Terms:
   - WM-2's teacher-forced NLL + BCE + `tf_l1` × weighted L1 and rollout loss, over every window (plain, cast and partner), unchanged in form and weights.
   - **Pair loss** (weight `pair`): for each cast window and its partner, the two k-step rollouts from position p start from the same state (asserted); with cur_A, cur_F the stepped states and true_A, true_F the recorded ones, the loss is the mean over steps i = 1 … k and continuous features of |(cur_A,i − cur_F,i) − (true_A,i − true_F,i)| / `stepper.std[stepper.cont]` × `stepper.feature_weights(v_weight)` (the indexing of `losses_tf`, `m1_tf.py:147`). The fire feature is excluded.
   - **Hazard loss** (weight `hazard`): target y_t = 1 if `g.fissions_total`[t + 1] − `g.fissions_total`[t] ≥ 0.5, else 0, for every teacher-forced row of every window; plain `binary_cross_entropy_with_logits` (no positive weight, so the head stays calibrated); report the positive rate of the train windows (expected ≈ 1–3 per 2,881 rows: the head learns, but the trunk gets little hazard gradient — say so in WM3.md).
4. **Fine-tuning loop** (`wm3/finetune.py`, its own loop that imports WM-1/WM-2 pieces; `m1_gru.train` is not called or changed): load the init run's `config.json` and `best.pt`; rebuild the normalizer from the same train groups and assert it equals the init run's stored normalizer to 1e-12 (else stop); AdamW lr `lr`, weight decay 1e-4, grad clip 1.0; the rollout horizon fixed at k = 168 (the init runs ended their curriculum at `k_max` 168); `--max-steps`, `--eval-every`, `--patience`, `--max-seconds` (the clock starts before data loading, as WM-2's). A newly created `haz` layer is the only parameter not loaded from the init. Train and val families are loaded once at start and kept in memory.
5. **Effect-aware checkpoint selection** (the same for every WM-3 run, including the control):
   - `val_eff` = the scaled effect MAE (`wm2.effects` scales `stds`, from `effects_base.json`) averaged equally over 24 cells — 4 outcomes × the 3 W3 arms × {c + 7 days, c + 30 days} — on the **val** cases of **replicate 0**, with rollouts of 720 rows (both lags read from one rollout), through `effects3.predictions` (item 6) on the pre-loaded val families.
   - `val_t2` = WM-2's validation T2 24 h (`m1_gru.val_score` with `val_batch` 512, as the init runs).
   - During fine-tuning both use `round_counts = False`; test evaluations use the `choose_rounding` result.
   - At step 0 compute both for the init checkpoint and record them; the step-0 `val_t2` with the init's own rounding must equal the init's `best_val_t2_24` (`val_metrics.json`: 0.0608 / 0.0603 / 0.0589 for s0 / s1 / s2) within 1e-3, else stop and record. A checkpoint is **eligible** if its `val_t2` ≤ 1.05 × the init's; among eligible checkpoints (the init included), keep the lowest `val_eff`. Patience counts evaluations without a new best eligible `val_eff`. `eval_every` is fixed before Part 3 by the cost check (9.2: if one evaluation exceeds 15 % of the training time between evaluations, the smallest multiple of 50 that brings it under) and never changes during a run.
   - **If the init is selected**, record `selected: init`: its V1/V2/V5/V6 predictions are WM-2's; for V3/V4 its hazard head is untrained, so that seed is reported as "hazard head untrained" and counts as a NONE seed there.
   - Output rounding of counts: `m1_gru.choose_rounding` on the selected checkpoint, as WM-2.
6. **Effect and split evaluation** (`wm3/effects3.py`):
   - `effects3.predictions(pred, families, cases, horizon=None)` takes pre-loaded families of **any role** (`wm2.effects.model_predictions` hard-codes `family_dirs(root, "test")`, `effects.py:430`, and reloads and re-verifies the families on every call); `horizon=None` = LAST_ROW − row, as WM-2. For models without a hazard head it must reproduce `wm2.effects.model_predictions` exactly: a test monkeypatches `wm2.effects.family_dirs` to return two **val** family directories, calls both on the same predictor and cases, and compares every number with `==` (no unit test reads a test family).
   - `wm3_ctrl` and `wm3_os` carry an untrained `haz` layer and are scored as non-hazard models (WM-2's event-based split day); V3 and V4 apply to `wm3_full` only.
   - For a hazard model, the rollout also returns the hazard probability h_k of every step k (the head's output at the position whose mean change produces row c + k). The **expected restricted split day** from reference row c: S_0 = 1; P_k = S_{k−1} h_k; S_k = S_{k−1}(1 − h_k); E = Σ_{k=1}^{K} P_k · day(c + k) + S_K · day(LAST_ROW), with K = LAST_ROW − c and day(row) = `v2.day_of_row(row)` (the same mapping as `v2.restricted_split_day`). The split shift = E_arm − E_factual. Report WM-2's event-based day for the same rollouts beside it.
   - Split timing (WM-2 7.7: test worlds with `g.fissions_total` = 0 at tick 34,440, replicate 0 of `none`): E from row 574 (= 34,440 / 60). Baselines from WM-2's committed `split_base`. A test on a synthetic hazard sequence (constant h → geometric expectation by hand; h = 1 at step 5 → day(c + 5)).
   - Test cases are WM-2's 604 (`effects_base.json` `cases.test`); outputs are written once per run (refuse to overwrite).
7. **Judge** (`wm3/judge3.py`): every V rule of Part 3 from the committed evaluation files, using `wm2.effects.compare` / `rule` for intervals and verdicts; `judge.md` and `judge.json`.
8. Tests for all of the above in `wm3/test_wm3.py`. Commit, push.

---

# 8. PART 3 — Pre-registration
Commit `docs/worldmodel/wm3/METRICS.md` **before any test-role number of a WM-3 model exists**: the audit decision, the runs (9.1) and their hyper-parameters, the selection rule (7.5), the definitions of 7.3 and 7.6, and the rules below. "CI" = world-level paired bootstrap 95 % interval of (model − baseline), as WM-2. **The W1 rule**: KEEP if the seed mean ≤ 0.8 × the baseline, every seed below the baseline and CI upper < 0; WEAK if CI upper < 0 (not KEEP); TIE if the CI contains 0; NONE if CI lower > 0. A configuration with one trained seed reaches WEAK at most.

| # | rule |
|---|---|
| V1 | **effects, primary** — W3 exactly as WM-2 (12 cells at c + 30 days, test, both replicates): `wm3_full` (every trained seed) vs `zero` (WM-2's validation choice, not re-selected): the W1 rule. |
| V2 | **effects at 7 days** — the same 12 cells at c + 7 days: `wm3_full` vs **`an5_arm`** — the lowest 7-day average on WM-2's validation predictions (`effects_base.json` `val_predictions`, both replicates: `an5_arm` 0.489, `arm_mean` 0.499, `zero` 0.647; recompute and assert in Part 2): the W1 rule. |
| V3 | **split shift** — WM-2's W4 (`sp240L`, `gf240E`, restricted days, both replicates), `wm3_full`'s expected-day shift (7.6) vs `an5_arm` (WM-2's validation choice): the W1 rule. Right-direction share reported. |
| V4 | **split timing** — WM-2's split-timing task, `wm3_full`'s expected day vs `ridge_day` (WM-2's validation choice, test MAE 29.21): the W1 rule. |
| V5 | **ablation** (reported with intervals, one seed each, WEAK at most): on V1's cells, `wm3_full_s0` vs `wm3_ctrl_s0` (the objective beyond more training under the same selection); `wm3_os_s0` vs `wm3_ctrl_s0` (cast windows alone); `wm3_full_s0` vs `wm3_os_s0` (pair loss and hazard beyond cast windows). |
| V6 | **factual guard** (reported): raw T2 168 h every-hour on `v2_v19_fact` test (`wm3/final_fact.py`), each WM-3 run vs the WM-2 checkpoint it started from, with intervals. If a `wm3_full` seed is worse by more than 0.005 (CI lower > 0.005), WM3.md says the effect gain cost factual accuracy. |

V1, V2 and V5 use `wm2.effects.compare` / `rule` on cell errors; V3, V4 and V6 use `judge_wm2.pw_verdict` on per-world scores (for V6 `bootstrap.world_scores` of `T2_any.168.test`), as WM-2's judge does.

---

# 9. PART 4 — Fine-tuning (DGX, one at a time)
1. **Runs** (order; each starts from a WM-2 checkpoint, `--seed` = the init's seed for the plain-window stream):

   | run | init | batch | pair | hazard |
   |---|---|---|---|---|
   | `wm3_full_s0` | `wm2_m1bact_v19_s0` | 16 plain + 16 cast (+ 16 partners) | 1.0 | 10.0 |
   | `wm3_ctrl_s0` | `wm2_m1bact_v19_s0` | 32 plain | 0 | 0 |
   | `wm3_os_s0` | `wm2_m1bact_v19_s0` | 16 plain + 16 cast | 0 | 0 |
   | `wm3_full_s1` | `wm2_m1bact_v19_s1` | as `wm3_full_s0` | 1.0 | 10.0 |
   | `wm3_full_s2` | `wm2_m1bact_v19_s2` | as `wm3_full_s0` | 1.0 | 10.0 |

   Common: lr 1e-4, `--max-steps 4000`, `--eval-every 250` (subject to 7.5's cost rule), `--patience 6`, every other hyper-parameter as the init run's `config.json` (`--tf-l1 1.0 --v-weight 5.0 --clamp-nonneg 1 --context 168 --warmup 168 --window 336 --val-batch 512`), `--max-seconds` = min(14,400, deadline − now − 3,600). The control and `os` runs use the same model class (`M1TFH`) with `hazard = 0`, so the three `_s0` runs differ only in batch composition and loss weights. These weights are fixed here; there is no tuning. The control is step-matched, not compute-matched (32 sequences per step vs 48 for `full`): say so in WM3.md.
2. **Before the Part 3 commit**, a 100-step cost check of `wm3_full` and `wm3_ctrl` (train and val only: s/step, GPU memory, one evaluation's wall time) fixes `eval_every` (7.5), which METRICS.md then records. Expected: `wm3_full` ≈ 2.3–2.4 s/step, ≈ 2.6 h for 4,000 steps; `ctrl` and `os` ≈ 1.6–1.7 h; one `val_eff` evaluation 15–30 s on cached families; the five runs ≈ 11–12 h, test evaluations ≈ 2 h. At a high DGX load s/step may approach 3.5 and `wm3_full` may stop on `time`: record it; do not extend `--max-seconds`.
3. Record stop reason, best step, wall time; "not converged" if the best step is in the last 10 % of steps. Evaluate (`effects3`, split timing, factual T2 168 h via `m1_gru.final`-equivalent on `v2_v19_fact` test) only after section 8's commit is pushed.
4. Copy each run's small files (never checkpoints) into `docs/worldmodel/wm3/runs/<run>/`.

# 10. PART 5 — Evaluation
All V comparisons with intervals; per-arm, per-outcome and per-lag effect tables (WM-2's layout, with WM-2's M1b-act seeds and the baselines beside them); the lag curve of `docs/worldmodel/wm2/runs/diag/effect_lag.json` recomputed for every WM-3 run on the same first 40 validation families (`--worlds 40` or the identical selection; one-step response, the thing A2 measured); hazard calibration of `wm3_full` on validation: for every val r0 cast row, the factual (zero-action) rollout's predicted probability of a split within 30 days, 1 − S_720, against the observed `v2.split_event_row(none_r fissions, c) ≤ c + 720`, in 5 quantile bins of the prediction. Commit, push.

# 11. PART 6 — Analysis
`docs/worldmodel/wm3/WM3.md` (English): the audit; the V table; the effect tables; the lag curves before and after; split shift and timing with the hazard head; the ablation; the factual guard; the design caveats (the control is step-matched, not compute-matched; training rollouts attend to every teacher-forced key ≤ the start plus the appended tokens, up to 335, while inference trims to 167 — inherited from WM-2's `losses_tf`; the hazard gradient into the trunk is small); and **"What a paper can claim"** — the strongest honest claim, the comparisons that back it, what it does not support, and the next experiment for each open claim (including WM-4: CRN / Causal Transformer on a confounded split). One line at the top of `docs/worldmodel/wm2/WM2.md`: "Followed by docs/worldmodel/wm3/WM3.md (audit of the one-step response; paired fine-tuning)."

# 12. PART 7 — Review, gate, report
1. Independent review (fresh `general-purpose` subagent, two rounds maximum): the audit's logic and numbers (one case by hand: its action row, its jump, the A2 prediction, the A3 lag-1); no test read before METRICS.md (its commit time vs every test output's time on the DGX); the pair loss on one cast window by hand (shared start state, differenced target); the expected-day formula on one case by hand; selection used validation only; V verdicts follow the rules. Findings in `docs/worldmodel/wm3/REVIEW.md`.
2. Gate (4.5) on both machines; fast-forward `codex/worldmodel-2026-10` if green; PR #512 comment.
3. `docs/worldmodel/wm3/REPORT.md` in Korean (section 13 format) and the same text in the final message.

---

# 13. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
감사(A1 데이터·A2 1스텝 응답·A3 하네스 일치), 위험도 헤드 모델, 투입 창 표집, 짝 차이 손실, 위험도 손실, 미세조정 루프, 효과 인지 체크포인트 선택, 기대 분열일, 사전 등록

### 기능 설명
연구 관점에서 무엇이 확인되었는가? WM-2의 실패는 버그였나 학습 문제였나? 논문이 주장할 수 있는 것과 없는 것은?

### 변경된 파일 목록
(커밋별)

### 측정 결과
감사 표(갈래별 참/예측 1스텝 점프, 부호 정답 비율, 불일치 수), V1~V6 표(신뢰구간), 갈래·결과·지평별 효과 표(WM-2 모델과 기준선 나란히), 1스텝 지연 곡선 전후, 분열 이동·분열 시점(기대일과 사건일), 위험도 보정, 실행별 종료 이유·최적 스텝·시간·val_eff 궤적

### 환경
DGX GPU·메모리·부하, 처리량(실행 종류별 s/step), 평가 1회 비용, 각 작업 시간, 연결 끊김 여부

### 독립 리뷰
지적 사항과 처리 결과

### 자율 결정 목록
DECISIONS.md 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 멈춘 구간, 학습 대기열 타임라인

### 확인된 제한사항

### 게이트 결과
(맥과 DGX의 unittest 수치, 허용 경로 밖 변경 없음 확인)

### 정직한 판단
WM-2의 "효과 예측 실패"는 무엇 때문이었나? 짝 갈래로 학습하면 신의 개입 효과를 단순 기준선보다 잘 예측하는가? 위험도 헤드는 분열을 예측하게 하는가? 얻은 것이 있다면 목적함수 덕인가, 더 학습한 덕인가? 논문의 중심 주장은 무엇이 되어야 하는가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #512 comment URL, the DGX paths of every run, and the time spent per Part.
