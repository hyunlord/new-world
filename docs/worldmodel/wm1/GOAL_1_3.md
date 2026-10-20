# GOAL: WorldSim — WM-1.3: is the tie a data problem? Five times the worlds, training to convergence, confidence intervals — and a first look at worlds that split

Repository `hyunlord/new-world`. Base: `codex/worldmodel-2026-10-wip` = `codex/worldmodel-2026-10` = `a43bdab2fc6fb8f8063f55f778e0a7f10183c6ba` (WM-1.2 final; draft PR #512).
This run is executed by **Claude Code on the Mac** in the WM clone, unattended; all recording, training and evaluation run **on the DGX Spark over ssh**. Revitalize Pass 12 (Claude Code) runs on the same Mac at the same time in `/Users/rexxa/work/new-world-pass4` and measures timings there, and an art run (Codex) may run Godot on the Mac: keep the Mac light (edits, small unit tests, git only).

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-wm1
git fetch origin && git checkout codex/worldmodel-2026-10-wip
git merge --ff-only origin/codex/worldmodel-2026-10-wip   # a43bdab2
cp ~/Downloads/claude_goal_worldsim_wm1_3.md docs/worldmodel/wm1/GOAL_1_3.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/worldmodel/wm1/GOAL_1_3.md>
```
Hooks stay neutralized in this clone (`.claude/settings.json` = `{}`, `--skip-worktree`); never edit, stage or commit it and never clear the flag.

**This document is `docs/worldmodel/wm1/GOAL_1_3.md`.** Commit it unchanged in Part 0. Where it disagrees with `GOAL.md`, `GOAL_1_1.md` or `GOAL_1_2.md`, this one wins for this run. Re-read the relevant section at the start of every Part and after every compaction or pause.

The owner drives the run with `/goal`: a small model reads only the transcript; the section 13 report, the Y table, commit hashes and `git ls-remote` heads must appear in your final message.

## 0.2 Overrides of CLAUDE.md
As WM-1.2's 0.2: never run `install_hooks.sh`; never check out, commit to or push `lead/main` or `main`; no harness pipeline — 4.4 is the gate; a blocking hook other than `/goal` → record in DECISIONS.md, push, report; subagents are built-in `general-purpose`. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decisions and reasons go to `DECISIONS.md` under `## WM-1.3` (ids `G0.1`, …); times to `RUNLOG.md` under `## WM-1.3`.
2. **Do not end your turn between Parts, and do not end it while DGX jobs run.** One task per Part. While training runs, poll every 10–20 minutes with short calls and do the other Parts' work in between. After a pause: re-read this file's section, the task list, `git log --oneline -5`, the DGX job logs.
3. Review loops: two rounds maximum. Push after every Part.
4. On failure: work around, note it, continue. An honest negative result is valid; a missing number is not.
5. **Time rule:** the training queue is long by design (≈ 7 h per pair of runs, four pairs). Past **36 hours** of wall time from the start (pauses excluded and recorded), start no new training, let running jobs end through their own `--max-seconds` (9.1), evaluate what exists, push, go to Part 7.
6. **Drop order:** the v17 probe (9.4) → `m1tf_w13v16_n200_s0` → `m1tf_w13v17_n400_s1` → `m1tf_w13v16_n80_s1` → the legacy-test evaluation (10.3). (The queue order of 9.1 puts the droppable runs last.) Never drop Parts 0–3, the three `m1tf_w13v16_n400` seeds, `m1tf_w13v16_n80_s0`, `m1tf_w13v17_n400_s0`, the calendar family at N = 80 and 400, the bootstrap, or Part 7.
7. **Long DGX jobs run detached** (`nohup … &`, `.exit` files), polled with short calls; `pgrep -fl` before relaunching. Test-role evaluation (`final`) only after the pre-registration commit (section 8) is pushed.
8. **Connection loss:** if the DGX stops answering, keep polling every 10 minutes for up to 2 hours while doing Mac-side work; jobs already launched keep running. Never call a run lost until you have checked its directory after reconnecting.

---

# ⚠️ 1. MACHINES
- **Mac** (`/Users/rexxa/work/new-world-wm1`): edit, commit, push, unit tests on tiny synthetic arrays. No recording, no training, no full evaluations, no cargo builds. Never touch `/Users/rexxa/work/new-world-pass4`, `/Users/rexxa/work/new-world-art-prod`, `~/github/new-world-wt/lead` or any other checkout. `/Users/rexxa/work/worldsim-wm-data/**` is read-only and not needed by this run.
- **DGX** (`ssh hyunlord@100.70.109.50`; throwaway `ssh -F` config or the full address; never edit `~/.ssh/config`): code clone `/home/hyunlord/work/new-world-wm1` detached at a pushed WIP commit before every job (`WM1_CODE_COMMIT` set); runs `/home/hyunlord/work/wm1-runs/<run>/`; venv `/home/hyunlord/work/wm1-venv`; data `/home/hyunlord/work/worldsim-wm-data/` (`v0/`, `v1/`, `p9/`, `p10/` read-only; this run adds `w13/`). Build clone for this run `/home/hyunlord/work/wm1-runs/_build/src_wm13`. Forbidden: `~/github/worldsim-training`, ComfyUI (8188), dev servers 3100/3200, other checkouts, processes you did not start.
- **GPU and memory:** `nvidia-smi`, `free -h`, `df -h /home/hyunlord/work` before every job. At most **two** M1b trainings at once (≈ 21 GB each at k = 168, measured in WM-1.2); never more than 64 GB of unified memory in total; if another user's process holds more than 32 GB of GPU memory, run one at a time and record it. Recording uses CPU workers: `nproc` first, `--workers` = min(8, nproc − 2), never while two trainings run.
- Packages as WM-1 (numpy, torch; no scipy, pandas, sklearn).

---

# 2. GIT
- Branches: commits to `codex/worldmodel-2026-10-wip`; `codex/worldmodel-2026-10` moves only by fast-forward to a WIP commit whose gate (4.4) is green on both machines. Never force-push. `git remote -v && git branch --show-current` before every push; `git ls-remote` after.
- Commit messages `worldmodel(wm1.3-N): …`. Stage explicit paths only; check `git diff --cached --stat` before every commit.
- **One merge, Part 0:** merge `origin/codex/revitalize-2026-10` at `31559232e0a24af12d29f490d440d1c5ee5f3661` (Pass 11's final head) with a merge commit (`git merge --no-ff 31559232`). The planner checked that it merges without conflicts (`git merge-tree` exit 0; this branch changed only `wm1/` paths since `14372ea2`). It brings recorder schema 5 (`wm_data.py` `SCHEMAS` 1–5), the `game_v17` / `game_v17_nofission` scenes in the recorder and probe, and Pass 11's tools. Never rebase. Do not merge anything else (Pass 12 is in progress on that line).
- **May change after the merge:** `tools/worldmodel/wm1/**` and `docs/worldmodel/wm1/**` only. The merged files are read-only to you; a bug in them is worked around inside `wm1/` and recorded.
- PR #512: stays draft; after the green fast-forward add a comment with the Y table (`gh api repos/hyunlord/new-world/issues/512/comments -f body=…`).

---

# 3. CONTEXT

## 3.1 Where the research stands (planner's review of WM-1.2)
- **WM-1.2 (test, 16 worlds of `game_v16`):** at 168 h every-hour starts M1b (3 seeds) 0.221 vs the validation-selected analog forecaster `an5` 0.206 (Z1 NONE); bias-removed 0.223 vs `cal_abs` 0.211 (Z2 NONE); at 24 h 0.160 vs 0.167 (Z1b WEAK). World diversity hurts the calendar family a lot (`an5` +50 % from v15 to v16) and the model hardly at all (+5 %): the gap fell from 0.071 (v15) to 0.014. The probe shows the model identifies the world (starting agents R² 0.998, bushes 0.89).
- **The planner's caveats:** on **validation** the sign was reversed (model 0.229–0.231 vs `an5` 0.238): with 16 test worlds a 0.01–0.02 difference has no stable sign. Two of three seeds stopped at the 3,000-step cap while still improving (s0 best at step 2,850 of 3,000). The calendar family and the model were both fitted on 80 worlds. So WM-1.2 is a tie under-powered on both sides, not a loss.
- **Pass 11 (`game_v17`, frozen):** villages split. On its pilot, fission raised food η² by 0.24 and wood by 0.20; settlement counts differ by world (1–3 at day 118). Fission timing is still pinned to display day 29 (Pass 12 is changing that in a new `game_v18`); `game_v17` itself is frozen and is the first multi-settlement world family.

## 3.2 The questions of this run
1. **Scale.** With 5× the training worlds (400) and training until validation stops improving, does M1b beat the calendar family on `game_v16` — and does the calendar family (whose analog forecaster also improves with more worlds) improve as much? Answered with world-level confidence intervals on 80 test worlds.
2. **Steps vs data.** How much of any change comes from training longer on the same 80 worlds, and how much from more worlds?
3. **Splitting worlds.** On `game_v17` (400 worlds), does the picture change when worlds differ by a structural, path-dependent event?
4. **Dataset v2 sizing.** How many worlds does the benchmark need, and what does one training run cost?

---

# 4. RULES

## 4.1 Comparability
Every number uses the WM-1.2 harness (`wm1/evaluate.py`, `wm1/climatology.py`, `wm1/calcore.py`) and its definitions: split std over rows ≥ 24 of the scored split, every-hour starts primary and 06:00 starts reported, raw and bias-removed scores, `cal_best` / `cal_best_fair` / `cal_best_anom` selected on validation, `apfit_abs`, persistence, the observable ridge. WM-1.2's numbers on `p10_*` and v0's numbers must not change: the golden tests stay green, and a new test recomputes one WM-1.2 calendar test number on `p10_v16` — `test.an5.T2_any["168"].test.mean` = 0.2064778167503853 in `runs/calendar_p10_v16_1a7dfdb6/calendar.json` (every-hour starts; the key `T2` holds 06:00 starts), relative 1e-9 — after every `data.py` or `climatology.py` edit. It needs the p10 data, so it runs in the DGX gate and skips on the Mac.

## 4.2 No leakage
- Roles (fixed here, recorded in the manifest): **`game_v16`** train 30000–30399 (nested subsets: N = 80 → 30000–30079, N = 200 → 30000–30199, N = 400 → all), val 30400–30479, test 30480–30559; **`game_v17`** train 32000–32399, val 32400–32479, test 32480–32559. Every training size uses the same val and test worlds.
- Inputs: observable fields only (`wm_data`'s observable filter). `game_v16` arm: exactly WM-1.2's 65 input names, in order (asserted; a missing or extra column is an error). `game_v17` arm: the state list `ALL_STATE_OBS` plus `g.settlements_alive` and `g.fissions_total` (schema-5 globals 37–38; `fissions_total` is deliberately the raw cumulative level, not a `d.` hourly difference) placed after `g.injured_members` and **before the calendar columns** — they are state features: normalized, predicted and rolled out like every other state column (appending them after the calendar would feed the rollout the true future, because the rollout takes calendar columns from the true rows). Input list = 37 state + 30 calendar = 67. The extra columns are chosen per dataset (a new `Dataset.extra_state` field), never per schema, so a schema-5 `game_v16` episode still gives exactly 65; `dataset_variant` and the feature-table check in `train()` (`m1_gru.py` ≈ L305, L324) use the dataset's own list. `parent_slot` and `founded_tick` (settlement columns 9–10) are not inputs. Never inputs: every hidden column, `latent.npy`, `meta.json`'s `world_variety` block (including region centres). A unit test asserts both input lists and the absence of each hidden field.
- Fits on train, selection and early stopping on val, test once per pre-registered configuration (section 8).

## 4.3 Reproducibility
As WM-1.2: every run writes `config.json` (code commit, data manifest sha, seed, hyper-parameters, device, torch version, dataset), `train_log.jsonl`, `val_metrics.json`, `metrics.json`, and the stop reason. GPU reproducibility is not measured (limitation).

## 4.4 The gate
Mac: `python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'` (recorder-dependent tests may skip on the Mac). DGX: the same discovery in the venv **with zero skips**, with `WM_BIN` and `PROBE_BIN` pointing at the build clone `_build/src_wm13` (built at the merge commit) and `WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data`. Expected after the merge: 117 (WM-1.2's 114 plus 3 merged from Pass 11), then this run's on top. `git diff --name-only <merge commit>..HEAD` lists only `wm1/` paths.

---

# 5. PART 0 — Merge and setup
1. Record start time, both machines' `uname -a`, the DGX's `nproc`, `nvidia-smi`, `free -h`, `swapon --show`, `df -h`. Commit `GOAL_1_3.md` unchanged; add `## WM-1.3` headings to DECISIONS.md and RUNLOG.md.
2. Merge `31559232` (section 2). Run the Mac unit tests; fix nothing outside `wm1/`. Push.
3. On the DGX: fetch, detach the code clone at the pushed merge, build `worldsim_wm` and `worldsim_probe` (`cargo build --release -p sim-test`) in a fresh build clone `_build/src_wm13` at the merge commit (record the time), run the gate (4.4). Push. Every recording of this run sets `WORLDSIM_GIT_SHA=<merge commit sha>` (the recorder writes `meta.git_sha` from that variable, `export.rs` ≈ L477, else "unknown").

# 6. PART 1 — Recording on the DGX
1. **Recorder equivalence:** record `game_v16` seeds 20000–20003 (120 days, `--every 60`, no `--agents`) with the new binary into a scratch directory and compare with `p10/pilot/game_v16/` (recorded by `58119582`, schema 4): every array column present in both, matched by name, must be identical (exact for integers, bit-identical for floats); list the columns only the new recording has. A difference stops the `game_v16` arm (record it; continue with `game_v17`).
2. **Record** `game_v16` seeds 30000–30559 and `game_v17` seeds 32000–32559 (120 days, `--every 60`, no `--agents`, `--workers` per section 1) into `/home/hyunlord/work/worldsim-wm-data/w13/<scene>/`. Check the space first (expected ≈ 0.7 MB per episode, ≈ 0.8 GB in total). Verify every episode with `wm_data.verify_episode`; every `meta.json` carries `git_sha` = the merge commit. Seeds 30000–30111 were used by Pass 10 as scratch seeds for range exploration (another binary, other ranges; never in a dataset): record that in DECISIONS.md — it is no leakage.
3. **Manifest** `docs/worldmodel/wm1/w13_manifest.json` (one entry per episode: scene, seed, role, `split` = scene, `schedule_id` = `none`, path, files with sha256, rows) and registry entries in `wm1/common.py`: `w13_v16_n80`, `w13_v16_n200`, `w13_v16_n400`, `w13_v17_n400` (val and test shared within a scene, counterfactual group None, tasks as the pilot datasets). Nested training sets need a mechanism: `Dataset` gains `train_seeds: tuple[int, int] | None` (half-open), applied in `common.load_groups` and in `probe_world.labels` to the train group; manifest roles stay train / val / test. A test asserts 80 / 200 / 400 train episodes with seeds 30000–30079 / 30000–30199 / 30000–30399 and identical val and test across the three. Also keep `p10_v16`'s test as the **legacy test** (16 worlds, report only). Commit, push.
4. Describe the new data briefly in `WM1_3.md` (Part 6): per scene, η² and R²_abs of the 8 V on the test split (Pass 11's `tools/revitalize/diversity_p11.py` `scene_block`: fit on the train role, score on test; read-only use), the share of `game_v17` worlds with ≥ 1 fission and the first-fission day distribution, so the reader sees what the worlds are like.

# 7. PART 2 — The harness for scale
1. **Schema 5 inputs** (`wm1/data.py`; v0 and p10 numbers unchanged, 4.1): the two input lists of 4.2; schema-5 episodes of `game_v16` give the same 65 columns as schema 4 (tested on a 1.1 scratch episode against its p10 twin: identical feature tables).
2. **Curriculum decoupled from the step cap:** a new hyper-parameter `ramp_steps` in both `m1_gru.DEFAULTS` and `m1_tf.DEFAULTS` with default **0** (an int: the parsers build flags with `type=type(default)`, so a None default would break `--ramp-steps`), meaning today's `max_steps // 2`; `half = cfg["ramp_steps"] or cfg["max_steps"] // 2`, used for the k ramp (`m1_gru.py` ≈ L349) and the patience start (≈ L374). Test: with 0 the k sequence and patience start are unchanged. Long runs use r3 with `--ramp-steps 1500` (identical to r3 for the first 3,000 steps) and train on at k = 168.
   **WM-1.2's stopping rule, online:** inside `train()`, also track the run as WM-1.2 would have stopped it (cap 3,000 steps, patience 5 evaluations from step 1,500, evaluations every 150 steps) and save that checkpoint as `best_wm12rule.pt` with its step in `val_metrics.json`; `final --ckpt best_wm12rule.pt --out <run>_short` scores it. This gives a steps-only comparison on the same run (Y3).
3. **Per-world scores, then the world-level bootstrap.** `evaluate.t2` today pools every start and stores only `per_var` / `mean` (`evaluate.py` ≈ L211–231), and `final` refuses to run twice (`m1_gru.py` ≈ L433). **Before the pre-registration commit**, `t2` gains `per_world`: per episode seed, the per-variable nMAE raw and bias-removed with the split std; `final` and `climatology.py final` write it into `metrics.json` / `calendar.json`; a test asserts that the mean of the per-world scores equals the pooled `mean` to 1e-12 (every episode has 2,881 rows and the same starts — assert equal start counts). Then `wm1/bootstrap.py`: for a predictor and task, the per-world score = that world's mean over the 8 V (split std fixed from the full test split). Paired resampling of test worlds with replacement, B = 2,000, numpy `default_rng(0)`; 95 % percentile interval of (a) each predictor's score, (b) the difference of two predictors (model = mean of its seeds' per-world scores). Raw and bias-removed (the bias-removed per-world score de-means each world's errors as WM-1.2 does). Tests on synthetic per-world scores (a constant shift gives a degenerate interval at the shift; identical predictors give [0, 0]).
4. **Cross-dataset evaluation:** `m1_gru.py final --run <dir> --eval-dataset <name> --out <dir>` evaluates a trained run on another dataset's test split with the run's own normalizer, `round_counts` and input list (refused unless `config["input_names"]` equals the target dataset's list; tested), split std from the target's test split, writes `run_dataset` and `eval_dataset`, and applies the run-once guard to `--out`. WM-1.2's `best.pt` files exist only on the DGX: check them first. Used for WM-1.2's runs on the new `game_v16` test (Y3) and new runs on the legacy test.
5. **Calendar family at every size:** for each of `w13_v16_n80`, `w13_v16_n200`, `w13_v16_n400`, `w13_v17_n400`: `climatology.py val --dataset <name>` (fit on that train, score the shared val), then — after section 8's commit — `climatology.py final --dataset <name> --selection <its val output>`. Early stopping and model selection of M1b stay on validation T2 24 h at 06:00 starts (`m1_gru.py` ≈ L260); "convergence" in the tables means that metric. Commit, push.

# 8. PART 3 — Pre-registration
Commit `docs/worldmodel/wm1/METRICS_1_3.md` **before any test-role number of a new model exists** (baseline test numbers may come after it too): the roles, the members and selection rule, the bootstrap, the runs to be evaluated on test (section 9.1), and the rules. "CI" is the paired bootstrap 95 % interval of (model − baseline) on the test worlds; "every seed" means every trained seed of that configuration.

| # | rule |
|---|---|
| Y1 | **`game_v16`, N = 400, raw.** M1b (`m1tf_w13v16_n400_s0..2`) vs `cal_best` = the member validation-selected (N = 400) for key `T2any_168` at T2 168 h every-hour starts: **KEEP** if the seed mean ≤ 0.8 × `cal_best`, every seed < `cal_best` and CI upper < 0; **WEAK** if CI upper < 0 but not KEEP; **TIE** if the CI contains 0; **NONE** if CI lower > 0. Y1b: the same at 24 h (`T2any_24`). WEAK here needs an interval below 0, stricter than WM-1.2's point-estimate WEAK: say so beside any comparison with Z1. The CI resamples worlds only; also report each seed's own paired CI. |
| Y2 | **`game_v16`, N = 400, bias-removed:** the Y1 rule on the bias-removed T2 168 h against `cal_best_anom` (key `T2anybr_168`, N = 400). |
| Y3 | **data vs steps** (`game_v16` test, raw, 168 h every hour; reported with CIs, no verdict): Δ_steps = M1b(long) − M1b(`best_wm12rule.pt`) on the same N = 80 runs (7.2); Δ_data = M1b(N = 400, long) − M1b(N = 80, long); Δ_cal = `cal_best`(N = 400) − `cal_best`(N = 80). "Scale helps the model more than the calendar" if the CI of Δ_data − Δ_cal lies below 0. WM-1.2's three runs on the new test are reported as `Δ_steps_xworld`, labelled confounded (other training worlds, 16 val worlds). |
| Y4 | **`game_v17`, N = 400:** M1b (`m1tf_w13v17_n400_s0`, `_s1`) vs `cal_best` (`T2any_168`, N = 400) raw and vs `cal_best_anom` (`T2anybr_168`) bias-removed, the Y1 verdict rule. |
| Y5 | **learning curve** (reported): test score vs N ∈ {80, 200, 400} for M1b and for `cal_best`, `an5`, `apfit_abs`; per variable at 168 h. |
| Y6 | **world identification on `game_v17`** (reported): WM-1.2's probe on `m1tf_w13v17_n400_s0` at row 166 for every hidden parameter (`PARAMS` in `probe_world.py` gains `wild_regions`, `region_bushes`, `region_deadwood`), plus two labels from observable columns read after training: fission by day 120 (yes/no; accuracy against the majority class) and `settlements_alive` at row 2,832; the first-fission day only if its test variance is above 0 (it is pinned to display day 29 in `game_v17`, so the calendar family can learn it — say so). Beside the raw-row reference (37 columns for v17). |

# 9. PART 4 — Models (DGX)
1. **Runs** (r3 recipe, action-blind, plus `--max-steps 9000 --ramp-steps 1500 --eval-every 150 --patience 12` (the same 1,800-step patience window as 300 × 6, and WM-1.2's evaluation grid for the online rule of 7.2) and `--max-seconds` = min(30000, the 36-hour deadline − now − 3,600) at launch, so every run ends inside `train()` and writes `val_metrics.json`, `round_counts` and `stop` (a killed process never does, and `final` would silently fall back to `round_counts` False); everything else as WM-1.2's command: `--batch 32 --clamp-nonneg 1 --k-max 168 --roll-min-context 167 --roll-starts 1 --tf-l1 1.0 --threads 8 --v-weight 5.0 --window 336 --variant sev-blind_action-blind`):
   - `m1tf_w13v16_n400_s0`, `_s1`, `_s2` (`--dataset w13_v16_n400`)
   - `m1tf_w13v16_n80_s0`, `_s1` (`--dataset w13_v16_n80`)
   - `m1tf_w13v17_n400_s0`, `_s1` (`--dataset w13_v17_n400`)
   - `m1tf_w13v16_n200_s0` (`--dataset w13_v16_n200`)
   Order: n400_s0 + n400_s1 → n400_s2 + v17_s0 → n80_s0 + n80_s1 → v17_s1 + n200_s0. Budget with WM-1.2's measured cost (two jobs at once: ≈ 2,400 s for the 1,500-step ramp, then ≈ 3.1 s/step at k = 168): ≈ 7 h per run at 9,000 steps unless patience stops it. Before the first pair, a **throughput check** at k = 168: `--ramp-steps 1 --max-steps 100 --eval-every 1000` into scratch directories, one job alone, then two at once; if the per-job s/step with two at once is ≥ 1.8 × the single-job s/step, run one at a time and say so (the drop order then applies).
2. Record each run's stop reason (`patience`, `max_steps`, `time`), best step and wall time. A run that ends on `max_steps` or `time` with its best step in the last 10 % is "not converged" in every table. Evaluate with `m1_gru.py final --run <dir>` only after section 8's commit is pushed; WM-1.2's three `p10_v16` runs with `--eval-dataset w13_v16_n400` (Y3).
3. Copy each run's small files (never checkpoints) into `docs/worldmodel/wm1/runs/<run>/`.
4. **Probe** (drop first): Y6 with WM-1.2's `probe_world.py`, labels read only after training.

# 10. PART 5 — Baselines and intervals
1. Calendar family, persistence and the observable ridge at N = 80, 200, 400 (`game_v16`) and 400 (`game_v17`): validation selection, then test.
2. Bootstrap intervals for every Y comparison, and per-variable skill (1 − nMAE / baseline) with intervals for food, alive, bushes, working adults (`game_v16`, N = 400; `game_v17` also for `wood_stock`).
3. Legacy test (16 worlds of `p10_v16`, drop fifth): the N = 400 models and `cal_best`(N = 400) evaluated there, beside WM-1.2's numbers. Commit, push.

# 11. PART 6 — Analysis
`docs/worldmodel/wm1/WM1_3.md` (English): the Y table with intervals; raw and bias-removed tables per dataset, size and horizon; the learning curve; convergence of every run; the `game_v17` data description (6.4) and Y4/Y6; and a section **"Dataset v2: how many worlds, which worlds"**: (a) is WM-1.2's tie a data or steps limit, or a real tie? (b) the number of worlds and the training budget a WM-2 run needs (from the learning curve and the measured s/step), (c) whether `game_v17`'s splits make the forecasting problem harder for the calendar family than for the model, (d) what Pass 12's `game_v18` must deliver for dataset v2 to be worth recording (point to UBS, the new between-world measure Pass 12 introduces). One line at the top of `WM1_2.md`: "Followed by WM1_3.md (scale and game_v17)."

# 12. PART 7 — Review, gate, report
1. Independent review (fresh `general-purpose` subagent, two rounds maximum): the merge (only expected files), 4.1 (v0 and p10 numbers unchanged), 4.2 (input lists, roles, nested training sets, no hidden field), pre-registration order (METRICS_1_3.md commit precedes every new test number: commit time vs `metrics.json` times on the DGX), the bootstrap code against a hand calculation on one comparison, `ramp_steps` default reproduces r3, Y verdicts follow the rules. Findings and fixes in `REVIEW.md` under `## WM-1.3`.
2. Gate (4.4) on both machines; fast-forward `codex/worldmodel-2026-10` if green; PR #512 comment.
3. `docs/worldmodel/wm1/REPORT_WM1_3.md` in Korean (section 13 format) and the same text in the final message.

---

# 13. 구현 완료 보고 (REPORT_WM1_3.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
병합, 기록기 동등성 확인, 1,120개 세계 기록·검증·매니페스트, 스키마 5 입력, 커리큘럼과 스텝 상한 분리(ramp_steps), 세계 단위 부트스트랩, 교차 데이터셋 평가, 사전 등록, 학습 대기열과 처리량, 탐침

### 기능 설명
연구 관점에서 무엇이 확인되었고 데이터셋 v2에 무엇이 정해졌는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
Y1~Y6 표(신뢰구간 포함), 데이터셋·크기·지평별 원점수와 편향 제거 점수(M1b 각 시드·평균, cal_best, cal_best_fair, cal_best_anom, apfit_abs, persistence, ridge), 학습 곡선, 실행별 종료 이유·최적 스텝·시간(수렴 여부), 변수별 skill, v17 데이터 설명, 탐침 R², 옛 시험 집합 비교

### 환경
DGX GPU·메모리·스왑·디스크 상황, 처리량 측정(동시 1개 대 2개), 각 작업 시간, 연결 끊김 여부

### 독립 리뷰
지적 사항과 처리 결과

### 자율 결정 목록
DECISIONS.md의 WM-1.3 항목 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 멈춘 구간, 학습 대기열 타임라인

### 확인된 제한사항

### 게이트 결과
(맥과 DGX의 unittest 수치, 허용 경로 밖 변경 없음 확인)

### 정직한 판단
WM-1.2의 동점은 데이터 부족이었나, 학습 부족이었나, 진짜 동점이었나? 세계를 다섯 배로 늘리면 학습 모델이 달력 계열을 신뢰구간 밖으로 이기는가? 분열하는 세계(v17)에서는 어떤가? 데이터셋 v2는 몇 개의 세계, 어떤 판으로 가야 하는가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #512 comment URL, the DGX run directory of every new run, and the time spent per Part.
