# GOAL: WorldSim — WM-2: dataset v2 on `game_v19`, the noise floor, and the question that matters — can a learned world model predict what the god's hand will do?

Repository `hyunlord/new-world`. Base: `codex/worldmodel-2026-10-wip` = `codex/worldmodel-2026-10` = `ce40bf3ceccadd72bec31a14ef8adca6a533dc65` (WM-1.3 final; draft PR #512).
This run is executed by **Claude Code on the Mac** in the WM clone, unattended; all recording, training and evaluation run **on the DGX Spark over ssh**. An art run (Codex) may use the Mac at the same time and measure frame times there: keep the Mac light (edits, small unit tests, git).

This is the first run of the WM-2 stage. It records **dataset v2** — `game_v19` worlds with the god's interventions of the Pass 14 map, replicate branches, and a noise-floor set — and asks three pre-registered questions: (1) how close to the irreducible floor do the forecasters already get on factual worlds; (2) **can a learned, action-aware model predict the effect of an intervention better than the strongest simple baselines**; (3) does a model with a long memory anticipate a village's split, which in `game_v19` depends on weeks of accumulated hunger, better than a model that sees the last 7 days?

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-wm1
git fetch origin && git checkout codex/worldmodel-2026-10-wip
git merge --ff-only origin/codex/worldmodel-2026-10-wip   # ce40bf3c
mkdir -p docs/worldmodel/wm2
cp ~/Downloads/claude_goal_worldsim_wm2.md docs/worldmodel/wm2/GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/worldmodel/wm2/GOAL.md>
```
Hooks stay neutralized in this clone (`.claude/settings.json` = `{}`, `--skip-worktree`); never edit, stage or commit it and never clear the flag.

**This document is `docs/worldmodel/wm2/GOAL.md`.** Commit it unchanged in Part 0. Where it disagrees with the WM-1 goal documents, this one wins. Re-read the relevant section at the start of every Part and after every compaction or pause. The owner drives the run with `/goal`: a small model reads only the transcript; the section 13 report, the W table, commit hashes and `git ls-remote` heads must appear in your final message.

## 0.2 Overrides of CLAUDE.md
As WM-1.3's 0.2 (no `install_hooks.sh`; never `lead/main`/`main`; no harness pipeline — 4.5 is the gate; built-in `general-purpose` subagents only; a blocking hook other than `/goal` → record, push, report). First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decisions in `docs/worldmodel/wm2/DECISIONS.md` (ids `H0.1`, …); times in `docs/worldmodel/wm2/RUNLOG.md`.
2. **Do not end your turn between Parts or while DGX jobs run.** Poll every 10–20 minutes with short calls; do other Parts' work meanwhile.
3. Review loops: two rounds maximum. Push after every Part.
4. On failure: work around, note it, continue. A negative result is valid; a missing number is not.
5. **Time rule:** the training queue is long by design (≈ 36 h of training after ≈ 6 h of recording). Past **50 hours** of wall time from the start (pauses excluded and recorded), start no new training; let running jobs end through their own `--max-seconds` (9.1); evaluate what exists; push; go to Part 7.
6. **Drop order:** the v16 control model (9.1 run 6) → the second long-memory seed → the third action-aware seed. Never drop Parts 0–3, the noise-floor set, the first two action-aware seeds, the first long-memory seed, the effect evaluation (Part 5) or Part 7.
7. **DGX jobs are detached** (`setsid nohup … > log 2>&1 &`, PID and `.exit` recorded); a 30-minute silent-log watchdog as WM-1.3's `slot.sh`; a dead recorder job is relaunched with `--resume`. **Connection loss:** poll every 10 minutes for up to 2 hours while doing Mac-side work; never call a run lost before checking it after reconnecting.

---

# ⚠️ 1. MACHINES
- **Mac** (`/Users/rexxa/work/new-world-wm1`): edit, commit, push, unit tests on tiny synthetic inputs. No recording, training, evaluation or cargo builds. Never touch `/Users/rexxa/work/new-world-pass4`, `/Users/rexxa/work/new-world-art-prod`, `~/github/new-world-wt/lead` or other checkouts.
- **DGX** (`ssh hyunlord@100.70.109.50`; throwaway `ssh -F` config or the full address; never edit `~/.ssh/config`): code clone `/home/hyunlord/work/new-world-wm1` detached at a pushed WIP commit before every job (`WM2_CODE_COMMIT` recorded); runs `/home/hyunlord/work/wm1-runs/wm2/<run>/`; venv `/home/hyunlord/work/wm1-venv`; build clone `/home/hyunlord/work/wm1-runs/_build/src_wm2` (at the merge commit); data `/home/hyunlord/work/worldsim-wm-data/v2/` (new) — every other directory under `worldsim-wm-data/` is read-only (`w13/` WM-1.3; `p14/` Pass 14, used for the identity check). Forbidden: `~/github/worldsim-training`, ComfyUI (8188), dev servers 3100/3200, other checkouts, processes you did not start.
- **Throughput (WM-1.3, measured):** one M1b training alone 1.50 s/step, two at once 3.27 s/step each — **train one model at a time**. GPU memory ≈ 21 GB per job. `nvidia-smi`, `free -h`, `df -h`, `uptime` before every job. Recordings and probes: `--workers 8` when no training runs, 4 while one does, under `nice -n 10`.
- Every recording sets `WORLDSIM_GIT_SHA=<merge commit>`.

---

# 2. GIT
- Commits to `codex/worldmodel-2026-10-wip`; `codex/worldmodel-2026-10` moves only by fast-forward to a WIP commit whose gate (4.5) is green on both machines. Never force-push. `git remote -v && git branch --show-current` before every push; `git ls-remote` after.
- Commit messages `worldmodel(wm2-N): …`. Stage explicit paths only.
- **One merge, Part 0:** `git merge --no-ff f5373ab2b7679c4a0d3a5c86b256971c86f93d41` (the simulation's green head after Pass 14: `game_v19`, recorder schema 6, the Pass 14 tools; its Rust equals Pass 13's frozen `2ba871c0`). The planner checked `git merge-tree` exits 0. Never rebase; merge nothing else.
- **May change after the merge:** `tools/worldmodel/wm1/**`, `docs/worldmodel/wm1/**` (WM-1 files you extend), and the new `tools/worldmodel/wm2/**`, `docs/worldmodel/wm2/**`. Everything merged from the simulation line — including `tools/revitalize/timeline_p14.py`, `schedules_p14.py`, `tools/worldmodel/wm_data.py`, `bench_v1.py` — is **read-only**: call it, never edit it; a bug in it is worked around inside `wm2/` and recorded.
- PR #512 stays draft; after the green fast-forward add a comment with the W table.

---

# 3. CONTEXT

## 3.1 WM-1.3 (planner's review)
- Training length, not data, explained WM-1.2's tie: the same N = 80 run scores 0.214 under WM-1.2's stopping rule and 0.197 at 9,000 steps. With 400 worlds the model beats the validation-selected analog forecaster `an5` on raw T2 168 h by −0.013 [−0.019, −0.008] (24 h: −0.025), short of KEEP; bias-removed it ties (+0.002). The analog baseline improves with the library (80 → 400 worlds: −0.020), the model barely (−0.002, confounded by a fixed 9,000-step budget: 5 of 6 v16 runs had not converged).
- On splitting worlds (`game_v17`) the edge vanishes (raw TIE, bias-removed NONE). The model sees **168 rows = 7 days** of context (`context` 168, `warmup` 168; its training window of 336 rows is context + horizon).
- **Not yet measured:** the irreducible error — what no forecaster can beat because behaviour noise after the start is unknowable. Both the model and the analogs sit near 0.20 bias-removed; if the floor is near that, factual forecasting on these worlds is saturated and cannot separate methods.
- WM-1.3's report recommends recording `game_v18` gated on UBS. **That is superseded** (it did not know Passes 13–14): `game_v18`'s divergence is a daily die, UBS is retired (it counts threshold effects as history), and the dataset is recorded on `game_v19` (3.2).

## 3.2 The simulation now (Passes 13–14)
- **`game_v19`:** a village's fission pressure P grows on eligible days (spring–autumn, ≥ 28 members) with the food shortfall below a line of members × 2.4 × 4 days, decays by 10 a day otherwise, and the village splits when P reaches a hidden threshold θ drawn once per settlement (exponential, mean 1,200). Noise alone moves the first post-fork split by 2–3.5 days; path share of `alive` ≈ 0.02 (`docs/revitalize/probe/p13-5_pilot/paths.md` after the merge).
- **The intervention map** (`docs/revitalize/INTERVENTION_MAP.md`, Pass 14, confirmed on fresh worlds): spoiling 240 food on the first morning the store is below the line (`T_low`) brings the split forward by 26 days in 0.79 of pairs (effect-to-noise ratio 9.4) and raises population at day 118 by +7 to +12; spoiling 120 at `T_low` has ENR ≈ 4.6 (screen only); gifting 240 on the eve of the first winter (`T_eve`, tick 29,400) delays the split (0.58–0.59, ENR 3.9–4.9); rotting 120 wood at `T_eve` adds 1.6 deaths in 35 days. Small acts, `T_high`, spring wood, cold snaps and short rain are placebo-sized. `T_low` comes from each world's factual probe (`timeline_p14.py`).
- Hidden in the recorder (never inputs): `fission_pressure`, `fission_threshold` (schema 6), `world_variety`, `latent.npy`. Observable and new: settlement `depart_tick`.

## 3.3 The questions
- **Q1 (floor):** how far are today's forecasters from the irreducible error on `game_v19` factual worlds, at 24 h and 168 h?
- **Q2 (effects, the primary claim):** given a world's history up to a cast and the cast itself, can an action-aware learned model predict the intervention's effect — on food, population, deaths and the day of the next split — better than (a) predicting no effect, (b) the average effect of that arm in the training worlds, and (c) an analog forecaster that averages the same arm's effect in the 5 most similar training worlds?
- **Q3 (memory):** does a recurrent model that reads the whole history anticipate the first split better than the 7-day-context transformer and simple day predictors?

---

# 4. RULES

## 4.1 Comparability
WM-1.3's harness and definitions everywhere they apply (raw and bias-removed T2, every-hour starts, split std over rows ≥ 24 of the scored split, validation-selected `cal_best` / `cal_best_anom`, `apfit_abs`, persistence, ridge, world-level paired bootstrap B = 2,000, 95 %, `default_rng(0)`). WM-1.3's and WM-1.2's committed numbers must not change: their golden tests stay green.

## 4.2 Roles and seeds
- `game_v19` worlds: **train 40000–40399, val 40400–40479, test 40480–40559**. `game_v16` control worlds: train 42000–42399, val 42400–42479, test 42480–42559. Never used before; not in tests (tests use seeds < 1000). Seeds 30000–33999 (WM-1.3), 12000–12031, 14000–14063 (Pass 14), 24000–26063 (pilots) are not used.

## 4.3 No leakage
- Inputs: observable fields only. `game_v19` input list = WM-1.3's `game_v17` input list exactly (67 = the 35 `game_v16` state columns + `g.settlements_alive`, `g.fissions_total` as state + 30 calendar columns; `runs/m1tf_w13v17_n400_s0/config.json` `input_names`), plus the 7 action columns of `data.ACTION_NAMES` for the action-aware variant (74). `depart_tick` and the settlement slot columns are not inputs. Actions sit at the cast row (`data.actions_of`; the recorder's row t is the state *before* tick t's interventions, so row(c) is shared by an arm and its factual and the action enters the transition c → c + 60). Every cast tick must be a multiple of 60 (`actions_of` silently drops others): assert it in the manifest build. A unit test asserts the input lists and the absence of every hidden field (`fission_pressure`, `fission_threshold`, `world_variety`, `latent`).
- Fits on train, selection and early stopping on val, test once per pre-registered configuration (section 8).

## 4.4 Reproducibility
As WM-1.3 (`config.json` with code commit, manifest sha, seed, hyper-parameters, device, torch version, dataset; `train_log.jsonl`; `val_metrics.json`; `metrics.json`; stop reason). GPU reproducibility not measured.

## 4.5 The gate
Mac: `python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'` (recorder-dependent tests may skip). DGX: the same discovery in the venv **with zero skips**, `WM_BIN`/`PROBE_BIN` pointing at `_build/src_wm2`, `WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data`. `git diff --name-only <merge commit>..HEAD` lists only the paths of section 2.

---

# 5. PART 0 — Merge, build, identity
1. Record start time, `uname -a` of both machines, the DGX's `nproc`, `nvidia-smi`, `free -h`, `swapon --show`, `df -h`. Commit `GOAL.md`; create `DECISIONS.md`, `RUNLOG.md`.
2. Merge `f5373ab2` (section 2). Mac unit tests. Push.
3. DGX: fetch, detach the code clone at the pushed merge, build `worldsim_wm` and `worldsim_probe` in `_build/src_wm2` (record time and sha256), run the gate. **Identity:** with the new `worldsim_wm`, re-run Stage A's group-F command (`docs/revitalize/probe/p14-1_stageA/commands.md`) for `--seeds 12000..12002` only (half-open: two worlds; `--schedule-dir /home/hyunlord/work/worldsim-wm-data/p14/schedules_A/F --include-factual --fork-tick 14280 --replicates 2 --days 120 --every 60`) into `v2/_identity/`, and compare every `.npy` file's sha256 and `interventions.json` with the same episodes in `worldsim-wm-data/p14/stageA/` (only `git_sha` in `meta.json` may differ). A difference stops the pass. Push.

---

# 6. PART 1 — Dataset v2 (DGX)

## 6.1 Factual timelines
`worldsim_probe --scene game_v19 --seeds 40000..40560 --days 120` with Pass 14's timeline flags exactly as in `tools/revitalize/p14_dgx/p3_timelineB.sh` (`--clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 60 --check-determinism`; `--workers` per section 1) into `v2/v19/timeline/`, then `timeline_p14.py --summary v2/v19/timeline/summary.json --out v2/v19/timeline_out` → `T_low`, `T_high`, `T_eve` per world (`--seeds` is half-open: 40000..40560 is 560 worlds). Record the share of worlds with a `T_low` and the distribution of its display day.

## 6.2 Families (`game_v19`, all 560 worlds)
`schedules_p14.py --timeline v2/v19/timeline_out/timeline.json --arms sp240L,sp120L,gf240E,rw120E --out v2/v19/schedules` (one single-row file per world and arm; its `manifest.json` is ignored by the recorder); record with `--fork-tick 14280 --replicates 2 --schedule-dir … --include-factual --days 120 --every 60` into `v2/v19/flat/`. Per world: the factual (`none`) and up to four arms (worlds without `T_low` lack the two `sp…L` arms; counted), each with replicates 0 and 1. Expected ≤ 560 × 5 × 2 = 5,600 episodes (≈ 0.85 MB each; check space first). Then build **one directory per world** of symlinks, `v2/v19/fam/game_v19_s<seed>/`, because `wm_data.load_family` expects one seed per directory (WM-1.2's family smoke did the same with copies). **Per-replicate prefix check** (every family; in `wm2/`, not `families.check_prefix`, which requires equal settlement ids): rows ≤ row(c) of arm member r equal those of `none` member r (the arm's cast tick c from `interventions.json`), and rows ≤ row(14280) of every member equal `none_r0`'s — `global.npy` compared in full over those rows; `settlements.npy` ([T, S, 14], S = every settlement the episode ever had) compared over the slots both episodes share, after checking that the shorter `settlement_ids.npy` is a prefix of the longer, and the extra slots of the longer must be all-NaN in those rows. A failure stops the pass. Replicate 1's history between the fork and c differs from replicate 0's by design; that is why section 7.4 predicts per replicate.

## 6.3 The `game_v16` control (factual only)
`--scene game_v16 --seeds 42000..42560 --days 120 --every 60` into `v2/v16/`. No interventions (v16 has no fission; it is the "no split" reference for Q1 and W6).

## 6.4 The noise-floor set (test worlds only)
For each `game_v19` test world (40480–40559) and each start tick s ∈ {20,040 (display day 15, 06:00), 56,040 (day 40, 06:00), 99,240 (day 70, 06:00)}: a factual-only family forked at s with **R = 4** (`--fork-tick s --replicates 4 --schedule-dir <empty dir> --include-factual --days 120 --every 60`) into `v2/v19/floor_s<s>/flat/`, then per-world symlink directories `v2/v19/floor_s<s>/fam/game_v19_s<seed>/` as in 6.2. Members are `none_r0 … none_r3`; replicate 0 equals the unforked factual (check 2 worlds bit for bit; it is recorded anyway so every floor family is self-contained). 80 × 3 × 4 = 960 episodes. Same for the `game_v16` test worlds at the same s (another 960, `v2/v16/floor_s<s>/`).

**Order and load.** Record 6.1 → 6.2 → 6.3 first at `--workers 8` (WM-1.3 measured ≈ 2.3 s per `game_v17` episode at 8 workers: ≈ 4 h for 6.2–6.3); training (Part 4) needs them. Record 6.4 at `--workers 4` while the first training run runs; RUNLOG records its end, which must precede any W2 evaluation.

## 6.5 Manifest and verification
Every episode verified with `wm_data.verify_episode`; `docs/worldmodel/wm2/v2_manifest.json` (scene, seed, role, `split` and `schedule_id` — the keys `data.load_split` reads, `data.py:244-256` — family id, arm, replicate, fork tick, path, files with sha256, rows, cast ticks (each a multiple of 60, asserted) and applied amounts from `interventions.json`). `docs/worldmodel/wm2/DATASET_V2.md`: what was recorded, counts, sizes, timings, the share of worlds per arm, the per-arm true effect distribution on train worlds (so the reader knows what is there to predict). Commit, push.

---

# 7. PART 2 — The harness for effects and memory (`tools/worldmodel/wm2/`)

Definitions used below: a **row** is 60 ticks (one hour); a day is 24 rows; display day of tick t = (t + 480) // 1,440 + 1 (`timeline_p14.py`); the **last row** of an episode is row 2,880 (day 120). The **split event** is the first row after a reference row at which `g.fissions_total` exceeds its value at the reference row **by ≥ 0.5** — for truth (an integer) and prediction (a continuous rollout state) alike (the daughter's founding, which follows the departure by up to 2 days; `depart_tick` is reported beside it as a cross-check only, never scored). The **restricted split day** = min(split event day, last-row day) — censored worlds count at the censoring day (a restricted-mean rule: truth and prediction are restricted the same way), and every table reports the share censored.

1. **Registry** (in `tools/worldmodel/wm1/common.py` `DATASETS`, which `m1_tf` / `m1_gru --dataset` read; manifest keys per 6.5): `v2_v19` — train: every member of every train family (all arms, both replicates); val: replicate-0 members (factual and arms) of the val families — the group `m1_tf` / `m1_gru` validate on (`spec.val`); test: every member of the test families (evaluation reads them through `effects.py`, not `spec`); `v2_v19_fact` — replicate 0 of `none` only, per role (used by every factual baseline fit, W1, W2); `v2_v16` (factual only). Input lists of 4.3. Action-aware variant `sev-blind_action-aware` (`data.MAIN`). `tools/worldmodel/wm2/__init__.py` exists so the gate discovers `wm2/` tests.
2. **Rollout extension** (`wm2/rollout_ext.py`): `evaluate.Rollout` carries only the 8 V variables, fire, cold hours and deaths (`evaluate.py:41-45`). For the two M1 predictors expose, from the stepped state they already compute, `g.settlements_alive` and `g.fissions_total` as well (new optional fields; existing outputs and WM-1.2 / WM-1.3 goldens unchanged, tested). Calendar, `an5`, persistence and ridge do not predict these columns and are not used where they are needed (items 4, 6).
3. **Noise floor** (`floor.py`): for each floor family, start s and horizon h ∈ {24, 168} hours and each of the 8 V variables j (`bench_v0.VARS`): the replicate values y_r at row(s) + h; the floor error of replicate r = |y_r − mean of the other replicates| × √((R − 1)/R) (record the uncorrected value too), divided by the test split std of j. The **floor score** = the mean over worlds, starts, variables, replicates. State in `METRICS.md` and `WM2.md`: this estimates the error of the conditional-*mean* predictor, exactly in expectation only for Gaussian noise (an approximation for integer columns such as `alive`); the MAE-optimal predictor is the conditional median, so the true MAE floor is **at or below** this estimate, and headroom (W2) is therefore a **lower bound**. Every predictor is also scored at exactly these starts and horizons on replicate 0 (the factual) — the "floor-matched score". Tests on synthetic replicates (identical → 0; independent Gaussian noise → σ√(2/π)/std within 5 %).
4. **Effect evaluation** (`effects.py`, extending WM-1.2's `families.py`; its `check_prefix` assumes rolling from the fork row and is not used here — the per-replicate check of 6.2 replaces it). For a test family, arm with cast tick c, and **each replicate r separately**: the predictor starts at row(c) of replicate r's factual (`none_r`), whose history up to row(c) is identical to arm r's (6.2), and rolls two branches to the last row: the factual (no action) and the arm (the arm's action at row(c)). **True effect** of replicate r = arm_r − none_r; **predicted effect** = predicted arm − predicted factual from the same start. Outcomes at **c + 7 days and c + 30 days** (rows row(c) + 168, + 720): `food_stock`, `alive`, cumulative deaths (the sum of the five cumulative `global.npy` columns `deaths_starvation`, `deaths_dehydration`, `deaths_combat`, `deaths_old_age`, `deaths_exposure` — the same set as `Layout.deaths` and `families.DEATH_COLUMNS`), `g.settlements_alive`; a family whose c + 30 days is past the last row is excluded from the c + 30 outcomes (counted per arm). **Split shift** = restricted split day (reference row(c)) in the arm minus in the factual, truth and prediction alike. Metrics per arm, outcome and horizon: **effect MAE** = mean over (world, replicate) of |predicted − true|, scaled by the std over train (world, replicate) pairs of that arm's true effect for that outcome and horizon (if that std < 1e-9, the std over all arms of that outcome and horizon; if still < 1e-9, the outcome is dropped for that arm and recorded); **sign accuracy** over pairs with a true effect ≠ 0; for the split shift, MAE in days and the right-direction share over pairs with a true shift ≠ 0. **Between-replicate effect spread**: post-cast noise is part of every per-replicate target; report |e_0 − e_1| / √2 (scaled likewise; the R = 2 form of item 3's estimator) next to every effect MAE. It is **not** a floor: the two replicates start the cast from different states (they diverge for 1–17 days between the fork and c), so it overstates the noise a predictor that sees each replicate's history must pay; it only bounds that floor from above. World-level paired bootstrap (resampling worlds; both replicates of a world move together). Tests on synthetic families (a predictor equal to the truth scores 0; the zero predictor scores mean |e| / std; censoring on both sides gives shift 0).
5. **Effect baselines** — they predict the effect directly, for every outcome including the split shift; none needs a factual forecast:
   - `zero`: no effect (split shift 0).
   - `arm_mean`: the mean over train (world, replicate) pairs of that arm's true effect, as a function of rows since the cast; for the split shift, the train mean shift.
   - `an5_arm`: for the start row(c) of replicate r, the 5 nearest train (world, replicate) members of the **same arm**, at most one per world (the nearer replicate), by `an5`'s own distance (its 24-row window of the 8 V variables ending at the start row, with its normalization; `climatology.py`), each compared at its own cast row; the prediction is the mean of those 5 members' true effect curves aligned at their casts (and of their split shifts).
6. **Models:**
   - **M1b-act**: WM-1.3's M1b recipe with the action-aware variant: `m1_tf train --variant sev-blind_action-aware --window 336 --context 168 --warmup 168 --ramp-steps 1500 --eval-every 150 --patience 12 --batch 32 --clamp-nonneg 1 --k-max 168 --roll-min-context 167 --roll-starts 1 --tf-l1 1.0 --threads 8 --v-weight 5.0 --max-steps 12000 --max-seconds <9.1>`, `--dataset v2_v19`.
   - **M1-long**: `m1_gru` with a new hyper-parameter `burn_in` ∈ {`window`, `full`} (default `window` = today's behaviour, tested bit-identical on a fixed seed). With `full`, the hidden state at a start is the teacher-forced GRU state over **every** row from 0 to the start: in training, rows 0 … start − W run as **one batched no-grad pass** through `proj` and the per-layer `nn.GRU`s (`M1GRU.run(x, h)`, `m1_gru.py:86-104`; cuDNN; never a Python loop over rows), with the batch's different burn-in lengths handled by `pack_padded_sequence` (zero left-padding is not neutral: a GRU's state moves on zero input), then the last W rows run with gradient from those per-layer states as today (truncated BPTT); `data.sample_windows` does not return window starts (`data.py:356-377`) — add a variant in `wm2/` (or an optional return in `data.py`, default unchanged and tested) that does; in evaluation, the teacher-forced hidden states of every row are computed **once per episode** in one pass and indexed by start (the current `GRUPredictor._rollout` loops over warm-up rows in Python, `m1_gru.py:175-185` — do not use that loop for `full`). Same action inputs and heads; the rest as M1b where it applies; `--max-steps` per 9.1. Record s/step and an evaluation pass's wall time.
   - **Validation for early stopping**: the `v2_v19` val group (replicate-0 members, factual and arms, ≤ 400 episodes). If the cost check (9.1) shows one validation pass costs more than 10 % of the training time between two evaluations, use a fixed subsample of validation starts (`default_rng(0)`, the same for every run) that brings it under 10 %, and record the fraction.
   - Both models roll autoregressively with clamping to the last row (≤ 2,620 rows); record drift (the factual rollout's T2 error at 7, 30 and 45 days after the start) as a diagnostic.
7. **Split-timing task (Q3):** `game_v19` test worlds with `g.fissions_total` = 0 at the start row, start at **tick 34,440 (display day 25, 06:00)**, replicate 0 of `none`. Target: the restricted split day (reference = the start row; censored at day 120). Predictors: M1-long and M1b-act (rollout to the last row, the split event of the preamble), and three baselines that need no rollout, each using train worlds' replicate-0 `none` members with `g.fissions_total` = 0 at the same start: `median_day` (their median restricted split day), `an5_day` (the mean restricted split day of the 5 nearest by `an5`'s distance at the start row), `ridge_day` (ridge from `an5`'s 24-row window features at the start to the restricted split day, λ from `bench_v0.LAMBDAS` selected on val). Metric: MAE in days and the share within ± 7 days. The test worlds' hidden θ (`fission_threshold`) is read **after** evaluation to report how much of each predictor's error θ explains (a ridge from θ and the 24-row window summary to the error; labels only).
8. Commit, push.

---

# 8. PART 3 — Pre-registration
Commit `docs/worldmodel/wm2/METRICS.md` **before any test-role number of a new model exists** (baseline numbers may come after it): roles, members and selection rules, the floor and effect definitions of Part 2, the runs to be evaluated (9.1), and the rules. "CI" = world-level paired bootstrap 95 % interval of (model − baseline); "every seed" = every trained seed of the configuration.

| # | rule |
|---|---|
| W1 | **factual, raw** (`v2_v19_fact` test — replicate 0 of `none` — T2 168 h every-hour): M1b-act vs `cal_best`: KEEP if the seed mean ≤ 0.8 × `cal_best`, every seed below and CI upper < 0; WEAK if CI upper < 0; TIE if the CI contains 0; NONE if CI lower > 0. W1b at 24 h. |
| W2 | **floor** (reported, no verdict): on `game_v19`, floor-matched scores of M1b-act, M1-long, `cal_best`, `an5`, persistence and the floor at 24 h and 168 h; on `game_v16`, the same for the v16 M1b, `cal_best`, `an5`, persistence; **headroom** = `cal_best` − floor. If headroom at 168 h < 0.02, write "factual forecasting is within 0.02 of the conditional-mean floor on game_v19" in the analysis (not "saturated": headroom is a lower bound, 7.3). |
| W3 | **effects — the primary claim** (`v2_v19` test families, both replicates; the scaled effect MAE of 7.4 averaged over the four outcomes at c + 30 days and the arms `sp240L`, `gf240E`, `rw120E`, every (outcome, arm) cell weighted equally): M1b-act vs **one** baseline — the member of {`zero`, `arm_mean`, `an5_arm`} with the lowest value of this same average on the **validation families** (chosen once, not per cell): the W1 rule (KEEP / WEAK / TIE / NONE). Same for M1-long, reported as W3-long. The between-replicate effect spread of the same cells is reported beside it. |
| W4 | **split shift** (`sp240L` and `gf240E`, restricted days, both replicates, the two arms' MAEs averaged): M1b-act vs the one member of {`zero`, `arm_mean`, `an5_arm`} best on validation by the same average: the W1 rule; right-direction shares and censored shares reported. |
| W5 | **memory** (split-timing task, 7.7; restricted split-day MAE): **W5** = M1-long vs the one member of {`median_day`, `an5_day`, `ridge_day`} best on validation: the W1 rule. **W5b** = M1-long vs M1b-act: the W1 rule (the memory comparison proper). Both reported; the analysis states which one holds. |
| W6 | **control** (`v2_v16` test, reported): M1b (action-blind, factual) vs `cal_best`, raw T2 168 h — WM-1.3's Y1 replicated on fresh worlds. |

In every rule, "every seed below" uses the seeds actually trained (section 0.3.6 may drop some); a configuration with one seed can reach WEAK at most, never KEEP.

---

# 9. PART 4 — Training (DGX, one at a time)
1. **Runs** (order): `wm2_m1bact_v19_s0`, `wm2_m1long_v19_s0`, `wm2_m1bact_v19_s1`, `wm2_m1long_v19_s1`, `wm2_m1bact_v19_s2`, `wm2_m1b_v16_s0` (action-blind, factual only). Before the first, a 200-step cost check of each model kind (s/step at k = 168 including one validation pass, GPU memory). Each run gets `--max-seconds` = min(21,600, deadline − now − 3,600) so it ends inside `train()`. Budget: WM-1.3 measured ≈ 1.58 s/step all-in (14,217 s for 9,000 steps), so a 12,000-step M1b-act run takes ≈ 5.3 h; `--max-steps 12000`. M1-long: `--max-steps` = min(12,000, ⌊0.95 × 21,600 / measured s/step⌋), recorded. The six runs total ≈ 32–36 h; the floor-set recording (6.4) runs beside run 1 at 4 workers.
2. Record stop reason, best step, wall time; "not converged" if best step is in the last 10 % of steps. Evaluate (`final` + the effect and split-timing evaluations) only after section 8's commit is pushed.
3. Copy each run's small files (never checkpoints) into `docs/worldmodel/wm2/runs/<run>/`.

# 10. PART 5 — Baselines, floor, effects
1. Calendar family, persistence, ridge on `v2_v19` and `v2_v16` (fit on train factual replicate 0, select on val), the floor (7.3), the effect baselines (7.5) selected on validation families, the split-timing baselines (7.7).
2. All W comparisons with intervals; per-outcome and per-arm effect tables; the drift diagnostic. Commit, push.

# 11. PART 6 — Analysis
`docs/worldmodel/wm2/WM2.md` (English): the W table; the floor and headroom table; the effect tables (per arm and outcome, with the true effect distribution beside them); the split-timing result and how much θ explains of the error; and a section **"What a paper can claim"** — the strongest honest claim the evidence supports (factual forecasting, effect prediction, memory), with the comparisons and intervals that back it, the claims it does **not** support, and the next experiment each open claim needs. One line at the top of `docs/worldmodel/wm1/WM1_3.md`: "Followed by docs/worldmodel/wm2/WM2.md (dataset v2 on game_v19); WM-1.3's game_v18 recommendation is superseded there."

# 12. PART 7 — Review, gate, report
1. Independent review (fresh `general-purpose` subagent, two rounds maximum): the merge; identity (5.3); leakage (input lists; hidden fields; θ read only after evaluation); roles; pre-registration order (METRICS.md commit time vs every test `metrics.json` time on the DGX); the floor estimator (one family by hand); the effect alignment (cast row, shared prefix, censoring) on one family by hand; baseline selection on validation only; W verdicts follow the rules. Findings in `docs/worldmodel/wm2/REVIEW.md`.
2. Gate (4.5) on both machines; fast-forward `codex/worldmodel-2026-10` if green; PR #512 comment.
3. `docs/worldmodel/wm2/REPORT.md` in Korean (section 13 format) and the same text in the final message.

---

# 13. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
병합과 동일성 확인, 데이터셋 v2 기록(사실 세계 시간표, 개입 가족, v16 대조, 잡음 바닥 세트), 매니페스트, 바닥 추정, 효과 평가(정렬·검열·기준선), 긴 기억 모델(전체 이력 예열), 분열 시점 과제, 사전 등록, 학습 대기열

### 기능 설명
연구 관점에서 무엇이 확인되었는가? 논문이 주장할 수 있는 것과 없는 것은?

### 변경된 파일 목록
(커밋별)

### 측정 결과
W1~W6 표(신뢰구간), 바닥·여유(headroom) 표, 갈래·결과별 효과 표(참 효과 분포와 함께), 분열 이동 MAE와 방향 비율, 분열 시점 과제와 θ의 설명력, 롤아웃 표류, 실행별 종료 이유·최적 스텝·시간, 데이터셋 v2 규모와 기록 시간

### 환경
DGX GPU·메모리·스왑·디스크, 처리량(모델 종류별 s/step), 각 작업 시간, 연결 끊김 여부

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
사실 세계 예측은 이미 바닥에 닿았는가? 학습 모델은 신의 개입이 무엇을 할지 단순 기준선보다 잘 예측하는가? 긴 기억이 분열을 미리 보게 하는가? 논문의 중심 주장은 무엇이 되어야 하는가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #512 comment URL, the DGX paths of dataset v2 and every run, and the time spent per Part.
