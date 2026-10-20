# GOAL: WorldSim — WM-1.1: is the learned world model learning dynamics, or memorizing the calendar?

Repository `hyunlord/new-world`. Base: `codex/worldmodel-2026-10-wip` at `b2cf872ccd9dde82244b11cb3fbea651691c95b6` (WM-1 final: harness, M1 GRU, M1b transformer, M2, M3, REPORT_WM1.md; draft PR #512).
This run is executed by **Claude Code on the Mac**, unattended, in the WM-1 clone. All training and evaluation run **on the DGX Spark over ssh**. The simulation's Pass 9 (Claude Code) works on the same Mac at the same time in another clone and uses no DGX; section 1 says how to share.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-wm1
git fetch origin
git checkout codex/worldmodel-2026-10-wip
git merge --ff-only origin/codex/worldmodel-2026-10-wip
cp ~/Downloads/claude_goal_worldsim_wm1_1.md docs/worldmodel/wm1/GOAL_1_1.md
tmux new -s wm11
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/worldmodel/wm1/GOAL_1_1.md>
```
The clone's hooks stay neutralized as in WM-1 (`.claude/settings.json` = `{}`, `--skip-worktree`). Never edit, stage or commit it and never clear that flag.

**This document is `docs/worldmodel/wm1/GOAL_1_1.md`.** Commit it unchanged in Part 0. `GOAL.md` (WM-1) stays as it is; where the two disagree, this one wins for this run. Re-read the relevant section at the start of every Part, after every compaction or pause, and whenever you are unsure of a number.

The owner drives the run with `/goal`: after each turn a small model reads only the transcript and decides whether the condition is met. The evidence it asks for (the section 14 report, the X table, commit hashes, `git ls-remote` heads) must appear in your final message itself.

## 0.2 This document overrides CLAUDE.md where they conflict
Identical to WM-1 `GOAL.md` 0.2: never run `install_hooks.sh`; never check out, commit to or push `lead/main` or `main`; the harness pipeline (`harness_pipeline.sh`, `.harness/**`) is not used — section 4.4 is the gate; if any hook other than the owner's `/goal` blocks you, record it in `docs/worldmodel/wm1/DECISIONS.md`, push, end with the report; never use `.claude/agents/harness-*` agents — subagents are the built-in `general-purpose` type. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json` — expect no git hooks, a line starting with `S`, `{}`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decide, implement, record decisions and reasons in `DECISIONS.md` under a new heading `## WM-1.1` (ids `E0.1`, `E1.1`, …).
2. **Do not end your turn between Parts.** Keep a task list (one task per Part). After a pause: re-read this file's section for the current Part, the task list, `git log --oneline -5`, the DGX job logs, then continue.
3. **Review loops: two rounds maximum.**
4. **Push after every Part** to `codex/worldmodel-2026-10-wip`.
5. **On failure: work around, note it, continue.** An honest negative result is a valid outcome of this run; a missing number is not.
6. **Time rule:** record the start time and each Part's start and end in `RUNLOG.md` under `## WM-1.1`. Past **8 hours** of working time, finish the current step, push, go to Part 6.
7. **Three strikes** per failing step, then keep the best state and record the evidence and a hypothesis.
8. **Drop order when time is short:** M3 without the day field (9.3 item 2; item 1 is never dropped) → M1b severity-on (Part 3.2) → the dataset v1 diagnostics (Part 5). Never drop Parts 0, 1, 2, 3.1 (M1b seeds 1 and 2), 6.
9. **Keep bulky output out of context:** logs, metrics and predictions stay in files on the DGX; read summaries.
10. **Long jobs run detached on the DGX** (a Bash call is cut off after 10 minutes): `ssh dgx "cd $RUN && nohup bash -c '<command>; echo \$? > <name>.exit' > <name>.log 2>&1 &"`, poll with short calls (`tail`, `cat <name>.exit`), check `pgrep -fl` before starting a job again.
11. **Launch the long GPU jobs first.** Part 3.1's two trainings take about an hour on the GPU; start them as soon as Part 0 is pushed (they need no new code), then write Parts 1–2 while they run. Training writes validation numbers only; **`final` (test numbers) runs only after `METRICS_1_1.md` is pushed** (4.3).
12. **Part N is section N + 5** (Part 0 = section 5 … Part 6 = section 12); section 10 holds the decision rules.

---

# ⚠️ 1. MACHINES (as WM-1 `GOAL.md` section 1, with these points)

## 1.1 The Mac (orchestration only)
- Clone `/Users/rexxa/work/new-world-wm1` (this one). **No heavy compute on the Mac**: Pass 9 measures timings there. Unit tests on tiny synthetic arrays are fine.
- Never read, write or build in `/Users/rexxa/work/new-world-pass4` (Pass 9), `/Users/rexxa/work/new-world-art-prod`, `~/github/new-world-wt/lead` or any other checkout; never kill a process you did not start.

## 1.2 The DGX Spark (all training and evaluation)
- `ssh hyunlord@100.70.109.50` (a throwaway `ssh -F` config or the full address; never edit `~/.ssh/config`).
- Paths as in WM-1: code `/home/hyunlord/work/new-world-wm1` (detached at a pushed WIP commit before every job; never edit files there; export `WM1_CODE_COMMIT=<sha>` for every job), runs `/home/hyunlord/work/wm1-runs/<run_id>/`, venv `/home/hyunlord/work/wm1-venv` (torch 2.14.1+cu130, as WM-1 recorded), data read-only `/home/hyunlord/work/worldsim-wm-data/v0` and, for Part 5 only, `/home/hyunlord/work/worldsim-wm-data/v1`.
- New for Part 5 only: a second, separate clone `/home/hyunlord/work/wm1-runs/_build/v1src` checked out **detached at `4c35e295`** (`git clone https://github.com/hyunlord/new-world.git /home/hyunlord/work/wm1-runs/_build/v1src && git -C /home/hyunlord/work/wm1-runs/_build/v1src checkout --detach 4c35e295`). It provides Pass 8's `tools/worldmodel/wm_data.py` (recorder schema 2), `bench_v1.py`, `docs/worldmodel/dataset_v1_manifest.json` and `bench_v1.json`. Read-only; never edit or build anything there.
- **Forbidden on the DGX:** `~/github/worldsim-training`, ComfyUI (port 8188) and its processes, dev servers on 3100/3200, `/home/hyunlord/work/new-world-revitalize` and every other checkout. Never stop a process you did not start.
- **GPU sharing:** `nvidia-smi` before each GPU job. At most **two** M1b trainings at once (each measured at ≈ 19 GB in WM-1); never more than 64 GB of the unified memory in total; if another process holds more than 8 GB of GPU memory, run one M1b at a time. Record what you saw.
- Allowed packages: as WM-1 (numpy, torch; `transformers`, `peft`, `accelerate`, `safetensors` for M3 only). No scipy, pandas, sklearn.

---

# 2. GIT
- Branches: `codex/worldmodel-2026-10-wip` gets every commit; `codex/worldmodel-2026-10` moves only by fast-forward to a WIP commit whose gate (4.4) is green. Both are at `b2cf872c` now. Never force-push. Before every push `git remote -v && git branch --show-current`; after it `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `worldmodel(wm1.1-N): <what changed>` (N = Part number). Stage explicit paths only, never `git add -A`.
- **May change:** `tools/worldmodel/wm1/**` and `docs/worldmodel/wm1/**`, nothing else. `tools/worldmodel/*.py` (Pass 7's loader and bench) are imported read-only. Do not merge or cherry-pick anything from `codex/revitalize-2026-10*`; v1 code is used only through the DGX `v1src` clone (1.2).
- PR: keep draft PR #512 open; after the green fast-forward, add a comment summarising WM-1.1 (REST API: `gh api repos/hyunlord/new-world/issues/512/comments -f body=...`; GraphQL is unavailable). Do not mark it ready.

---

# 3. CONTEXT

## 3.1 What WM-1 found (verified by the planner against `RESULTS.md` and the run directories)
- Harness W1 PASS (bench reproduced to 3.5e-10). Inputs to M1/M1b: state fields, calendar = hour one-hot, season one-hot, day of season / 7 (**no** absolute day, year or tick), actions, severity only in sev-on variants (`NOTES.md`).
- T2 168 h (06:00 bench starts), mean nMAE over the 8 variables V: M1b transformer (one seed, `m1tf_main_r3_s0`) A/B/OOD_long **0.122 / 0.124 / 0.120**; M1 GRU main (3 seeds) 0.275 / 0.272 / 0.277; best non-oracle baseline `ridge_nosev` 0.223 / 0.227 / 0.241; persistence 0.554 / 0.549 / 0.536. M1b T1 0.068 vs ridge 0.100; T2 24 h 0.076 vs 0.138.
- T4 (C_cf test, 4 variants): no model learned intervention effects; every model sits at the zero-effect baseline (normalized MAE ≈ 1.0). Interventions sit on 0.28 % of B_act rows and 0.09 % of all training rows (384 of 414,864).
- M3 (Qwen2.5-0.5B LoRA, 39 % of one epoch): 24 h nMAE **0.056** on A test vs daily naive 0.139. Its JSON line carries the **absolute day number** (`"day": D`), which M1/M1b never see.
- T3b (winter outlook): M1b 0.203 vs the observable ridge 0.130 (severity oracle 0.134).

## 3.2 Why this run exists (the planner's review of WM-1)
Every baseline WM-1 compared against is a **one-step dynamics model rolled forward** (persistence, ridge) or a copy of yesterday. None of them knows **what this world usually looks like at this time of year**. In dataset v0 every episode starts from the same calendar instant, every split except OOD_v12 uses the same scene `game_v13`, and the 96 train seeds may differ only through a few random streams (idle movement, lifespans, worldgen details, winter severity draws). If the trajectories of different seeds are close to each other, a model that has merely memorized the mean trajectory — "on day 37 at 06:00 the store holds about X" — scores well at 168 h without understanding any dynamics. M3's 0.056 at 24 h with the absolute day in its prompt is exactly what such memorization would look like, and the transformer's 7-day (168-row) context plus its season/day-of-season input could identify the calendar position just as well.

So the 168 h number cannot yet be called "dynamics learning". Climate science and forecasting answer this with **climatology** and **anomaly persistence** baselines and with **analog forecasting** (Lorenz 1969): a forecast is only skilful if it beats them. This run adds those baselines, measures how different the seeds actually are, and re-judges M1b, M3 and the ridges against them. The answer decides the research path (planner's research plan v1): if the calendar explains most of the 168 h skill, the paper's datasets must be built for inter-seed diversity (Pass 9–10: keyed randomness, forks, resampled latents) and the "learned dynamics" claim waits for them.

## 3.3 Terms used below
- **Row** r: the bench's hourly row. Every v0 episode starts at tick 0, so row r is the same calendar instant in every episode (day, hour, season): tick = 60 r, one-based display day D = (60 r + 480) // 1440 + 1.
- **Year position** p(r) = (r + 8) mod 672 (672 rows = 28 days × 24 h; p = 0 is 00:00 of one-based day 1, 29, 57, …).
- **Train rows**: A_train episodes are 120 days; **T_tr = 2,881** rows (the manifest's `rows`; the last row, 2,880, is day 121 08:00). OOD_long has 5,761 rows, C_cf 841. Assert T_tr from the data and `ep.bench.tick == 60 * arange(rows)` for every episode.
- **Calendar family** = the baselines of Part 1 (CalAbs, CalYear, their anomaly-persistence versions, analogs).

---

# 4. NON-NEGOTIABLE RULES

## 4.1 Comparability
Every new number uses the WM-1 harness (`wm1/evaluate.py`) and so `bench_v0`'s definitions: the same splits, roles, variables, nMAE denominators (`bench_v0.split_std`), start rows (`bench_starts`, `any_hour_starts`), T3 evenings, T3b cases. Calendar-family baselines are implemented as `evaluate.Predictor` subclasses and scored by the same `t1`, `t2`, `t3`, `t3b` functions. Part 5 (v1) uses `bench_v1`'s definitions in the same way.

## 4.2 No leakage
- Calendar means, anomaly-persistence coefficients and analog libraries are fit on **A_train rows only** (natural dynamics; B_train is excluded because its interventions shift the mean). Selection among calendar-family members uses **validation only** (mean of A_val and B_val, as WM-1). Test roles, OOD_v12 and OOD_long are touched once per pre-registered configuration.
- A calendar-family prediction for start row s reads the test episode's rows ≤ s only (plus train statistics). A unit test changes the test episode's rows > s and asserts identical predictions. Actions are ignored by these baselines (a unit test asserts that); their T4 effect is zero by construction and equals `zero_effect` — report it as such, do not run T4 for them.
- `winter_severity`, `wood_target`, `latent.npy`, `lifespan_days` are never inputs of any calendar-family baseline.

## 4.3 Pre-registration
Before any test-role number of a new baseline or a new model run exists, commit `docs/worldmodel/wm1/METRICS_1_1.md` with: the calendar-family members and their exact definitions (Part 1), the validation selection rule, the decision rules X1–X6 (section 10), the list of configurations to be evaluated on test (`m1tf_main_r3_s1`, `m1tf_main_r3_s2`, `m1tf_sevon_aa_r3_s0`, `m3_noday_s0`, the calendar-family selection per task), and the parent code commit that will compute them. Any later change to `climatology.py`, `calcore.py` or `diversity.py` before the test numbers exist requires a dated amendment in `METRICS_1_1.md`. Never change a rule after test numbers exist; if a rule turns out ill-posed, report both the original verdict and a clearly labelled post-hoc one.

## 4.4 The gate (before the green branch moves)
On the Mac: `python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'` (torch tests may skip on the Mac with a message). On the DGX, inside the venv, the same discovery with torch present (no skips allowed except the three Pass 7 recorder tests if the recorder is not built; WM-1 recorded 0 skips on the DGX — match that). `git diff --name-only b2cf872c..HEAD` lists only allowed paths. WM-1's existing tests still pass unchanged.

---

# 5. PART 0 — Setup
1. Record start time, `uname -a` on both machines, the DGX's `nvidia-smi`, free disk and memory. Commit `GOAL_1_1.md` unchanged. Add `## WM-1.1` headings to `DECISIONS.md` and `RUNLOG.md`.
2. On the DGX: `git -C /home/hyunlord/work/new-world-wm1 fetch origin codex/worldmodel-2026-10-wip && git -C /home/hyunlord/work/new-world-wm1 checkout --detach <pushed sha>`. Check the venv: `python -c "import torch; print(torch.__version__, torch.cuda.is_available())"`.
3. Check that WM-1's run directories exist on the DGX (`m1tf_main_r3_s0/best.pt`, `m3_s0/lora/`, `w1_repro_832eaa2c/baselines.json`) and (after creating the `v1src` clone of 1.2) that each of the 5,012 file paths of its `docs/worldmodel/dataset_v1_manifest.json` (432 episodes) exists under `/home/hyunlord/work/worldsim-wm-data` (extra files such as schedules are recorded separately, not counted against it; if v1 is missing, Part 5 is skipped with a note — do not copy data from the Mac).
4. Push. **Then immediately launch Part 3.1** (no new code needed).

---

# 6. PART 1 — Calendar-family baselines (`tools/worldmodel/wm1/climatology.py`)
Do not name the module `calendar.py` (it would shadow Python's standard `calendar` module).

All statistics come from the A_train episodes' V arrays (`ep.bench.v`, rows ≥ 24 — the same rows `split_std` scores; rows 0–23 are excluded from every mean below).

## 6.1 Members
1. **CalAbs** (absolute calendar mean): `m_abs[r] = mean over A_train episodes of v[r]`, for every r in [0, T_tr). Rows 0–23 serve only as the anomaly reference at T1's first start (s = 23, `evaluate.t1_starts`); every score and diagnostic uses rows ≥ 24. For a row r ≥ T_tr (OOD_long only), wrap back by whole years to the last year that has data: `r' = r − 672 · ceil((r − (T_tr − 1)) / 672)`; assert `p(r') == p(r)` and `r' < T_tr`. Prediction for target row t: `m_abs[t]`.
2. **CalYear** (year-cyclic mean): `m_year[q] = mean over A_train episodes and over all rows r ≥ 24 with p(r) = q of v[r]`, q in 0..671 (every q is covered). Prediction: `m_year[p(t)]`. This one uses only what M1/M1b's calendar input can express (season, day of season, hour) — the fair comparison for models without absolute time.
3. **AP-full** (anomaly persistence, α = 1): `pred(s + k) = m(s + k) + (v[s] − m(s))`, for m = m_abs and m = m_year (two members: `ap1_abs`, `ap1_year`).
4. **AP-fit** (damped anomaly persistence): `pred(s + k) = m(s + k) + α[k, j] · (v[s, j] − m(s, j))` per variable j and lead k = 1..168, α fit by least squares on A_train: over every episode and every start s with 24 ≤ s and s + k < T_tr, `α[k, j] = Σ a_s · a_{s+k} / Σ a_s²` where a = v − m (0 when the denominator is 0). Two members `apfit_abs`, `apfit_year`. Record the fitted α at k = 1, 24, 168 per variable in `METRICS_1_1.md`.
5. **Analog** (Lorenz 1969 analog forecast): for a start s and horizon H, shift `Δ = 672 · max(0, ceil((s + H − (T_tr − 1)) / 672))` (one shift for the whole window and every target, so the window is never torn); compare the test episode's window `v[s−23 .. s]` with rows `s−23−Δ .. s−Δ` of every A_train episode and predict from their rows `s+k−Δ`; distance = Σ over the 24 rows and 8 variables of `((v_test − v_train) / std_train)²` with `std_train = bench_v0.split_std(A_train)`. Members: `an1` (the nearest episode's `v[s + k − Δ]`), `an5` (mean of the 5 nearest), `an5_off` (`an5` plus the offset `v_test[s] − mean of the 5 neighbours' v[s − Δ]`). The analog library is A_train only; an A_train episode is never its own analog (only matters for in-sample checks).
For CalAbs, CalYear and the analogs the rollout is direct (no feedback). `fire_lit` predictions are the averaged values (a fraction in [0, 1]); the harness clips them for T3 and thresholds them at 0.5 for T3b, as it does for every predictor.

## 6.2 Selection (validation only)
For each task and horizon (T1; T2 24 h and 168 h bench starts; T2-any 24 h and 168 h; T3 by Brier; T3b by MAE) evaluate all nine members (CalAbs, CalYear, ap1_abs, ap1_year, apfit_abs, apfit_year, an1, an5, an5_off) on A_val and B_val; the member with the lowest mean of the two is **`cal_best`** for that task and horizon; the best among the members that never read absolute time, {CalYear, ap1_year, apfit_year}, is **`cal_best_fair`**. Analogs read absolute time (same rows) and count as non-fair. (T3 compares by Brier on A_val only and T3b by MAE on A_val and B_val, since the harness's T3 is defined on A.) Record both selections and all validation numbers in `METRICS_1_1.md` **before** evaluating on test.

## 6.3 Tests (`tools/worldmodel/wm1/test_climatology.py`, numpy only, synthetic episodes)
1. Wrap rule: for T_tr = 2881 and r in {2881, 3552, 3553, 5760}, `p(r') == p(r)`, `r' < 2881`, `r' ≥ 2881 − 672`. Analog shift: for s = 2860, H = 168, the shifted rows stay inside [0, 2881).
2. `p(0) = 8`; day 2 06:00 is row 22 with p = 30; day 30 06:00 is row 694 with p = 30 (tick convention of 3.3).
3. α fit recovers a known AR coefficient (0.8 ± 0.02) on synthetic AR(1) anomalies around a sinusoidal calendar.
4. No future reads: perturbing test rows > s leaves every member's predictions unchanged; changing actions, `ep.bench.severity`, `wood_target` and `lifespan_days` leaves them unchanged.
5. Analogs: on synthetic data where test episode = train episode 7 + 0.001 noise, `an1` picks episode 7.

---

# 7. PART 2 — How different are the seeds? (`tools/worldmodel/wm1/diversity.py`, numpy only)
For each v0 split group (A_train, A_val, A_test, B_test, OOD_v12, OOD_long), per variable j of V, over rows ≥ 24 (OOD_long: all its rows ≥ 24, CalAbs wrapped):
1. **R²_abs[j]** = 1 − Σ (v − m_abs)² / Σ (v − v̄)², v̄ the split's pooled mean of the variable (A_train: in-sample, labelled as such). Likewise **R²_year[j]** with m_year.
2. **Seed-variance fraction η²[j]** = Σ_r Σ_e (v_e[r] − m_split[r])² / Σ_r Σ_e (v_e[r] − v̄)², m_split the split's own per-row mean over its episodes (OOD_long rows ≥ T_tr included: its own episodes define its mean). η² = 0 means every seed traces the same trajectory.
3. **Divergence day**: for every pair of episodes in the split (A_test: 120 pairs; cap at 500 random pairs with numpy seed 0 for A_train), the first row ≥ 1 where any V differs by more than 1e-9, converted to display day; report min / median / max and the number of pairs that never diverge (identical V throughout). Also per variable.
4. **Between-seed spread at fixed instants**: at 06:00 of days 30, 60, 90 and 118, the std over the split's episodes of each variable divided by `split_std`.
5. **Severity draws**: the number of distinct winter-severity sequences among the split's episodes (from `ep.bench.severity` at **18:00** of days 22, 50, 78, 106, and for OOD_long every winter up to day 218 — at 06:00 of a winter's first day the field still reads −1, `docs/worldmodel/README.md`; assert season == 3 and severity ≥ 0; a label used here for description only, never as an input) and η² recomputed within groups of equal sequence (pooled), to show how much of the seed variance the severity draws explain.
Write `docs/worldmodel/wm1/DIVERSITY.md` (tables per split and variable, a one-paragraph reading) and `runs/diversity_<sha8>/diversity.json` (copy the JSON into `docs/worldmodel/wm1/runs/diversity_<sha8>/`). Tests (`test_diversity.py`): η² = 0 for identical episodes, ≈ (E − 1)/E within 0.02 for independent noise around a constant (E = 50 episodes, T = 1,000 rows); divergence day on a synthetic pair that differs from row 100 on.

---

# 8. PART 3 — M1b seeds and severity-on

## 8.1 M1b main configuration, seeds 1 and 2 (never dropped)
Exactly WM-1's recorded command with only the seed and output changed (from `METRICS.md`):
```
python tools/worldmodel/wm1/m1_tf.py train --out /home/hyunlord/work/wm1-runs/m1tf_main_r3_s<SEED> --variant sev-blind_action-aware --seed <SEED> --batch 32 --clamp-nonneg 1 --eval-every 150 --k-max 168 --max-steps 3000 --roll-min-context 167 --roll-starts 1 --tf-l1 1.0 --threads 8 --v-weight 5.0 --window 336
```
for SEED = 1 and 2, in parallel (1.2 GPU rule), from the detached code clone with `WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data` and `WM1_CODE_COMMIT` set. Then, **after `METRICS_1_1.md` is pushed**, `python tools/worldmodel/wm1/m1_gru.py final --run /home/hyunlord/work/wm1-runs/m1tf_main_r3_s<SEED>` (all tasks, T4 included; `kind_of` routes m1_tf runs). Copy each run's small files (config, metrics, val metrics, logs; never checkpoints) into `docs/worldmodel/wm1/runs/<run>/` as WM-1 did.

## 8.2 M1b severity-on, one seed (drop second)
`m1tf_sevon_aa_r3_s0`: the same command with `--variant sev-on_action-aware --seed 0`. Then `final`. Its purpose: does the severity label help the transformer on T3b and on winter targets (W5's question for M1b)? It is a reference, not a claim of hidden-state inference.

## 8.3 GPU nondeterminism
Do not rerun s0. Record that M1b reproducibility on the GPU was not measured in WM-1 and is not measured here (a limitation).

---

# 9. PART 4 — Re-judging the WM-1 models

## 9.1 Calendar-family test numbers
Evaluate the selected `cal_best` and `cal_best_fair` (and, for the tables, every member) on A_test, B_test, OOD_v12, OOD_long through the harness: T1, T2 24/168 (bench starts), T2-any 24/168 (with `breakdown=True`: by target season), T3 (A test), T3b (all four). Script: `tools/worldmodel/wm1/climatology.py final --out /home/hyunlord/work/wm1-runs/calendar_<sha8>` on the DGX; copy its JSON into `docs/worldmodel/wm1/runs/calendar_<sha8>/`.

## 9.2 Skill against the calendar
For each model (M1 GRU main 3 seeds, M1b main 3 seeds, M1b sev-on, ridge_nosev, ridge_action_nosev) and each test split: **skill = 1 − nMAE_model / nMAE_cal_best** (and against `cal_best_fair`) for T2 24 h and 168 h (both start sets) and T1, mean over V and per variable. The mean skill uses the mean nMAEs (not a mean of per-variable skills); a per-variable skill is "—" when the baseline's nMAE is 0 or NaN (e.g. `cold_members`, `adults_working` at 06:00). Use the models' existing `metrics.json` files (no re-evaluation of WM-1 runs); if a per-variable number you need is missing from a WM-1 `metrics.json`, compute it with the harness from the run's checkpoint in a `final`-style evaluation that writes to a new file `metrics_1_1.json` beside it (never overwrite WM-1's `metrics.json`).

## 9.3 M3 and the day field
1. Compare M3's 24 h number (A test, bench starts, day granularity: the 06:00 row of day D+1 from day D's line) with CalAbs, `cal_best` and `cal_best_fair` on exactly the same rows and the same denominator, per variable.
2. **M3 without the day field** (`m3_noday_s0`, drop first): add a `--no-day` option to `m3_text.py` that replaces `"day": D` in every line by `"dos": int(round(ep.calendar[r, -1] * 7))` (the stored day of season; the season field stays), and a `--max-steps` option. Train on the DGX **CPU exactly as WM-1** (`m3_text.py` hard-codes `"cpu"`) with WM-1's recipe and `--max-steps 276` (M3's `steps_done`), so M3 − M3-noday is not confounded by training amount; store `no_day` in `config.json` so `final` builds the same lines; evaluate the same way. Tests: the no-day line contains no key `"day"` and parses back.

---

# 10. DECISION RULES (pre-registered in `METRICS_1_1.md`; reported as the X table)

| # | rule |
|---|---|
| X1 | **Diversity.** Report R²_abs, R²_year and η² per variable for every v0 split (no threshold; the reading is descriptive). Flag every variable with R²_abs ≥ 0.9 on A_test as "calendar-determined". |
| X2 | **M1b dynamics claim (main).** M1b main, 3 seeds. Cells = {A_test, B_test, OOD_long} × {T2 168 h bench starts, T2-any 168 h}. KEEP: in every cell the 3-seed mean ≤ 0.8 × `cal_best` of that task and every seed < `cal_best`. WEAK: not KEEP, but in every cell the mean < `cal_best`. NONE: otherwise. `cal_best` is the val-selected member only (never the best member on test; no refit on train + val). Report the same verdict against `cal_best_fair` beside it. |
| X3 | **M1b short horizon.** Same rule at T1 and T2 24 h (both start sets), reported as KEEP / WEAK / NONE. |
| X4 | **M3.** M3's 24 h A_test nMAE ≥ 20 % below `cal_best` (T2 24 h bench starts, val-selected) → "skill beyond the calendar"; otherwise "consistent with calendar memorization". Same for `m3_noday_s0` if run, and report M3 − M3-noday. |
| X5 | **Ridges and M1 GRU.** The same KEEP / WEAK / NONE as X2 (both start sets) for `ridge_nosev` (one model: its value is both the mean and every seed) and the M1 GRU main (3 seeds) — the baselines WM-1 judged M1b against may themselves lose to the calendar. |
| X6 | **Severity (M1b).** sev-on vs sev-blind M1b on T3b (all splits), T2-any 168 h winter targets, T4 dark nights; reported with the 3-seed spread of the sev-blind model. No threshold. |

These rules do not change WM-1's W verdicts; they are added beside them.

---

# 11. PART 5 — Dataset v1 diagnostics (drop third)
Purpose: did Pass 8's `game_v14` (night burn ± step, day fall ± spread, windfall over snow) make seeds more different than `game_v13`? Pass 9/10 build on the answer.
1. A separate entry point `tools/worldmodel/wm1/diversity_v1.py` that, **as its first statements**, inserts `<v1src>/tools/worldmodel` at `sys.path[0]` (path given by `--v1-tools`), imports `bench_v1` and `wm_data` from there and asserts `wm_data.__file__` is under that directory. It must not import the `wm1` package (whose `__init__` puts the branch's schema-1 `wm_data` on the path); share code with `diversity.py` / `climatology.py` by moving their pure array functions into `tools/worldmodel/wm1/calcore.py`, a module with no imports outside numpy, and loading it by file path (`importlib.util.spec_from_file_location`) in `diversity_v1.py`.
2. Load v1 with `bench_v1.load_split(Path(root), manifest, split)` (v1's `Ep` has `hidden_severity`, not `severity`; use it only for item 5's description) (manifest = `<v1src>/docs/worldmodel/dataset_v1_manifest.json`, root = `/home/hyunlord/work/worldsim-wm-data`: the manifest's paths already begin with `v1/`; 432 episodes, 5,012 files; v1 seeds are numbered from 10000, so do not compare seeds across datasets by number; split groups by `entry["role"]` as `bench_v1.run` does). Compute Part 2's diagnostics for A_train, A_val, A_test, B_test, OOD_v13, OOD_long (OOD_v13 is game_v13 episodes recorded by Pass 8; compare it with v0's A_test in the text).
3. Calendar-family baselines on v1 for T1 and T2 24/168 h, at both v1 start sets (`bench_v1.rollout_starts(n_rows, h, every_hour)` with True and False), selected on v1 val as in 6.2, scored with `bench_v1.split_std` and `bench_v1.nmae`; compare with `bench_v1.json`'s ridge numbers (read from `<v1src>/docs/worldmodel/bench_v1.json`; for example the 168 h every-hour A_test ridge_action 0.267).
4. Write the v1 sections into `DIVERSITY.md` (side by side with v0) and `runs/diversity_v1_<sha8>/`. Tests: `diversity_v1.py --self-test` runs the calcore functions on synthetic arrays (no data needed), and a unittest that imports `calcore` by file path and checks one η² value. **Never import `diversity_v1` from a unittest process** (a `wm_data` already cached in `sys.modules` would defeat the path insertion). v1 test-role numbers are listed in `METRICS_1_1.md` before they are computed, like v0's; otherwise label them exploratory.

---

# 12. PART 6 — Analysis, review, final, report
1. `docs/worldmodel/wm1/WM1_1.md` (English): what was measured, the X table, the skill tables, DIVERSITY's main numbers for v0 (and v1), M3 vs CalAbs, and a section **"What this means for the research plan"**: (a) can the v0 benchmark distinguish dynamics learning from calendar memorization? (b) what inter-seed diversity (η², divergence day) would a dataset v2 need for the X2 rule to be informative? (c) which of WM-1's statements must be qualified. Add one line at the top of `ANALYSIS.md` and `RESULTS.md`: "Qualified by WM1_1.md (calendar baselines)." Do not otherwise rewrite WM-1's documents. Regenerate nothing of WM-1's tables except by adding rows.
2. **Independent review** (fresh `general-purpose` subagent, at most two rounds): give it this file, `METRICS_1_1.md`, `climatology.py`, `diversity.py`, `calcore.py`, `diversity_v1.py` and the result JSONs. It checks: leakage (4.2: train-only fits, val-only selection, rows ≤ s, no severity input), pre-registration order (METRICS_1_1.md commit precedes every test-number commit — compare commit times and contents), wrap rule, η² formula, that skill numbers match the JSON files, and that the X verdicts follow the rules. Record findings and fixes in `REVIEW.md` under `## WM-1.1`.
3. Gate (4.4) on the final WIP head on both machines; fast-forward `codex/worldmodel-2026-10` if green; comment on PR #512 (section 2).
4. `docs/worldmodel/wm1/REPORT_WM1_1.md` in the section 14 format, Korean, and the same text in the final message.

---

# 13. 결과 확인사항 (주인이 직접 확인할 것)
- `WM1_1.md`의 X 표에서 M1b의 168시간 판정(KEEP / WEAK / NONE)과 그 근거 숫자(`cal_best` 대비 몇 % 낮은지, 세 시드 각각)를 확인한다.
- `DIVERSITY.md`에서 A_test의 변수별 R²_abs와 η², 그리고 시드 쌍이 처음 갈라지는 날(중앙값)을 확인한다. η²가 0에 가깝고 R²_abs가 0.9 이상인 변수가 많으면, v0에서는 "역학 학습"과 "달력 암기"를 구분할 수 없다는 뜻이다.
- M3의 24시간 0.056이 CalAbs 숫자와 얼마나 가까운지, 그리고 day 필드를 뺀 M3가 얼마나 나빠졌는지 확인한다.
- v1 진단이 돌았다면, v0와 v1의 η²·갈라지는 날을 나란히 보고 game_v14가 시드 다양성을 실제로 늘렸는지 확인한다.
- `METRICS_1_1.md` 커밋이 모든 시험 숫자 커밋보다 앞서는지(사전 등록 순서) 커밋 로그로 확인한다.

---

# 14. 구현 완료 보고 (REPORT_WM1_1.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
달력 기준선(CalAbs·CalYear·이상치 지속·유사 사례), 검증 기반 선택, 다양성 지표(R²·η²·갈라지는 날), 사전 등록, M1b 추가 시드와 sev-on, M3 day 제거 실험, v1 진단 방식

### 기능 설명
연구 관점에서 무엇이 확정되었는가? 연구 계획(데이터셋 v2, Pass 9–10)에 주는 의미

### 변경된 파일 목록
(커밋별)

### 측정 결과
X1~X6 표, 기준선별 시험 수치(분할·과제·지평), 모델별 달력 대비 skill(평균과 변수별), M1b 세 시드, M3 vs CalAbs, v0/v1 다양성 비교

### 환경
DGX GPU 공유 상황, 각 학습의 시간과 메모리

### 독립 리뷰
지적 사항과 처리 결과

### 자율 결정 목록
DECISIONS.md의 WM-1.1 항목 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 멈춘 구간, 컨텍스트 압축 뒤 목표 파일을 다시 읽은 시점, DGX 작업 목록

### 확인된 제한사항

### 게이트 결과
(맥과 DGX의 unittest 수치, 허용 경로 밖 변경 없음 확인)

### 정직한 판단
M1b의 168시간 성능은 역학 학습인가, 달력 암기인가? M3는? v0 벤치마크는 무엇을 측정할 수 있고 무엇은 못 하는가? 데이터셋 v2에 필요한 다양성은 얼마인가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #512 comment URL, the DGX run directory of every new run, and the time spent per Part.
