# GOAL: WorldSim — WM-1: the first learned world models, trained on dataset v0 on the DGX Spark

Repository `hyunlord/new-world`. Base: `codex/revitalize-2026-10` at `ce7acb25d816e6e6fdcc2c8666b90119c42b9121` (Pass 7: `game_v13`, the `worldsim_wm` recorder, dataset v0, WM-Bench v0).
This run is executed by **Claude Code on the Mac**, unattended. All training and evaluation run **on the DGX Spark over ssh**; the Mac only edits code, commits and pushes. The simulation's Pass 8 (Claude Code) and possibly an art run (Codex) work on the same Mac at the same time, in other clones; section 1 says how to share.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
git clone --branch codex/revitalize-2026-10 https://github.com/hyunlord/new-world.git /Users/rexxa/work/new-world-wm1
cd /Users/rexxa/work/new-world-wm1
echo '{}' > .claude/settings.json && git update-index --skip-worktree .claude/settings.json
git checkout -b codex/worldmodel-2026-10-wip ce7acb25d816e6e6fdcc2c8666b90119c42b9121
mkdir -p docs/worldmodel/wm1 && cp ~/Downloads/claude_goal_worldsim_wm1.md docs/worldmodel/wm1/GOAL.md
tmux new -s wm1
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/worldmodel/wm1/GOAL.md>
```
The repository's Claude Code hooks are neutralized **in this clone only** (`.claude/settings.json` = `{}`, hidden with `--skip-worktree`). Never edit, stage or commit it and never clear that flag.

**This document is `docs/worldmodel/wm1/GOAL.md`.** Commit it unchanged in Part 0. Re-read the relevant section at the start of every Part, after every compaction or pause, and whenever you are unsure of a number.

The owner drives the run with `/goal`: after each turn a small model reads only the transcript and decides whether the condition is met. The evidence it asks for (the section 16 report, the W table, commit hashes, `git ls-remote` heads) must appear in your final message itself.

## 0.2 This document overrides CLAUDE.md where they conflict
CLAUDE.md stays authoritative for conventions where they apply (doc comments, tests per module, no secrets). These do **not** apply to this run:
1. "Session Startup: run `install_hooks.sh`" — never run it.
2. "Working branch: lead/main", its pre-flight and its push — never check out, commit to or push `lead/main` or `main`.
3. The harness pipeline (`harness_pipeline.sh`, score decomposition, `HARNESS_SKIP`, ENV-BYPASS, `.harness/audit|baseline|reviews/`) — not used; section 4.4 is the gate.
4. If any hook other than the owner's `/goal` blocks a command or a stop, do not get around it: record it in `docs/worldmodel/wm1/DECISIONS.md`, push what you have, end with the report.
5. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json` — expect no git hooks, a line starting with `S`, `{}`.
6. Never use the repository's `.claude/agents/harness-*` agents; when this document asks for a subagent, use the built-in `general-purpose` type.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decide, implement, record decisions and reasons in `docs/worldmodel/wm1/DECISIONS.md`.
2. **Do not end your turn between Parts.** Keep a task list (one task per Part). After a pause: re-read this file's section for the current Part, the task list, `git log --oneline -5`, the DGX job logs, then continue.
3. **Review loops: two rounds maximum.**
4. **Push after every Part** to `codex/worldmodel-2026-10-wip`.
5. **On failure: work around, note it, continue.** A missing model with an honest report beats a stalled run.
6. **Time rule:** record start time and each Part's start and end. Past **12 hours** of working time, finish the current step, push, go to Part 7 (Part 2 must have finished at least one evaluated M1 configuration before that).
7. **Three strikes** per failing target (a training recipe that does not reach a target counts one strike per recipe change), then keep the best configuration and record the evidence and a hypothesis.
8. **Drop order when time is short:** M3 (Part 5) → the M1 transformer (Part 3) → the second and third training seeds of every configuration except M1-GRU's main one → M2 (Part 4) → the action-blind ablation. Never drop Parts 0, 1, 2 (M1-GRU with T1–T4 and the severity-blind ablation), 6 or 7.
9. **Keep bulky output out of context:** training logs, metrics and predictions go to files on the DGX; read summaries.
10. **Long jobs run detached on the DGX** (a Bash call is cut off after 10 minutes): `ssh dgx "cd $RUN && nohup bash -c '<command>; echo \$? > <name>.exit' > <name>.log 2>&1 &"`, poll with short calls (`tail`, `cat <name>.exit`), check `pgrep -fl` before starting a job again.

---

# ⚠️ 1. MACHINES

## 1.1 The Mac (orchestration only)
- Clone `/Users/rexxa/work/new-world-wm1` (this one). **No heavy compute on the Mac**: Pass 8 measures timings there. Unit tests on tiny synthetic arrays are fine.
- Never read, write or build in `/Users/rexxa/work/new-world-pass4` (Pass 8), `/Users/rexxa/work/new-world-art-prod` (art), `~/github/new-world-wt/lead` or any other checkout; never kill a process you did not start.
- Dataset v0 also exists on the Mac (`/Users/rexxa/work/worldsim-wm-data/v0`); this run reads the DGX copy only.

## 1.2 The DGX Spark (all training and evaluation)
- `ssh hyunlord@100.70.109.50` (define `Host dgx` in a throwaway `ssh -F` config or use the full address; never edit `~/.ssh/config`). GB10 Grace-Blackwell, 128 GB unified memory, aarch64.
- Paths (create them):
  - code: `/home/hyunlord/work/new-world-wm1` — a clone of `hyunlord/new-world`, checked out **detached at a pushed WIP commit** before every job (`git fetch origin codex/worldmodel-2026-10-wip && git checkout --detach <sha>`); never edit files there.
  - runs: `/home/hyunlord/work/wm1-runs/<run_id>/` (config, log, metrics JSON, predictions, checkpoint). Checkpoints never enter git.
  - venv: `/home/hyunlord/work/wm1-venv`.
  - data (read-only): `/home/hyunlord/work/worldsim-wm-data/v0` (copied and hash-verified by Pass 7).
- **Forbidden on the DGX:** `~/github/worldsim-training` (never read, write or import from it), ComfyUI (port 8188) and its processes, dev servers on 3100/3200, `/home/hyunlord/work/new-world-revitalize` and every other checkout. Never stop a process you did not start.
- **GPU sharing:** before each training job run `nvidia-smi`; if another process holds the GPU, keep your job's memory under 32 GB (smaller batch) and record it. Never use more than 64 GB of the unified memory in total.

## 1.3 The Python environment on the DGX (decide without asking, record the outcome)
1. Try the system interpreter: `python3 -c "import torch, numpy; print(torch.__version__, torch.cuda.is_available(), torch.cuda.get_device_name(0))"`.
2. Else create the venv and install PyTorch for aarch64 + CUDA from the official index: `python3 -m venv /home/hyunlord/work/wm1-venv && /home/hyunlord/work/wm1-venv/bin/pip install numpy torch --index-url https://download.pytorch.org/whl/cu130` (if that index has no matching wheel, try `cu129`, then `cu128`). Check that a 4096×4096 matmul runs on `cuda`.
3. Else, if `docker` and an NVIDIA PyTorch container are already present on the DGX, use that (mount code, data and runs read/write as above; pull nothing larger than 25 GB).
4. Else train on the Grace CPU (`torch` CPU wheel, `torch.set_num_threads(16)`); the models of this run are small enough. Record the choice in DECISIONS.md and in every run's config.
- Allowed packages: numpy, torch, and for Part 5 only `transformers`, `peft`, `accelerate`, `safetensors`. No scipy, pandas, sklearn, lightning, wandb (stdout and JSON logs only).

---

# 2. GIT

- Branches: `codex/worldmodel-2026-10-wip` gets every commit; `codex/worldmodel-2026-10` moves only by fast-forward to a WIP commit whose gate (4.4) is green. Both start at `ce7acb25`. Never force-push. Before every push `git remote -v && git branch --show-current`; after it `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `worldmodel(wm1-N): <what changed>`. Stage explicit paths only, never `git add -A`.
- **May change:** `tools/worldmodel/wm1/**` and `docs/worldmodel/wm1/**`, nothing else. `tools/worldmodel/*.py` (Pass 7's loader and bench) are imported read-only; if one has a bug that blocks you, work around it inside `wm1/` and record it — Pass 8 owns those files.
- At the end, open a **draft PR** from `codex/worldmodel-2026-10` into `codex/revitalize-2026-10` titled `WM-1: first learned world models on dataset v0` (skip and say so if `gh` is not authenticated).

---

# 3. CONTEXT

## 3.1 What exists (Pass 7, verified by the planner)
- `game_v13`: one village of about 40 agents; four seasons of 7 days; each winter draws a severity (mild / normal / harsh, weights 52 / 28 / 20 %) that sets the hourly burn (2 / 3 / 4) and the daily deadwood fall (8 / 6 / 4); seven god interventions (gift/spoil food, gift/rot wood, rain, cold snap, windfall).
- Dataset v0 (`docs/worldmodel/DATASET_v0.md`, `docs/worldmodel/dataset_v0_manifest.json`, recorder schema `docs/worldmodel/README.md`): 400 episodes, 1,368 MiB, one row per simulated hour. Splits A_obs (128 × 120 days, no actions; `--agents` for 0–31, 96–99, 112–119), B_act (64 × 120 days, 8 random interventions each), C_cf (32 seeds × factual + 4 variants × 35 days), OOD_v12 (32, the Pass 6 rules), OOD_long (16 × 240 days). Roles: A train 0–95 / val 96–111 / test 112–127; B train 1000–1047 / val 1048–1055 / test 1056–1063; C_cf val 2000–2007 / test 2008–2031.
- Loader `tools/worldmodel/wm_data.py` (`load_episode`, `Episode.column`, meta fields with `hidden` flags); bench `tools/worldmodel/bench_v0.py` (`load_split`, `features`, `Ridge`, `t1_scores`, `rollout_starts`, `t2_scores`, `winter_evenings`, `t3_scores`, `c_cf_targets`, `c_cf_effects`, `t4_scores`); report `docs/worldmodel/BENCH_v0.md`.

## 3.2 Numbers to beat (BENCH_v0.md; mean nMAE over the 8 variables V unless stated)
| Task | Split | Best baseline |
|---|---|---|
| T1 next hour | A test / B test / OOD_v12 / OOD_long | 0.100 / 0.103 / 0.106 / 0.100 (ridge) |
| T2 24 h | A test / B test / OOD_v12 / OOD_long | 0.139 / 0.137 / 0.137 / 0.135 (daily naive = persistence) |
| T2 168 h | A test / B test / OOD_v12 / OOD_long | 0.541 (action ridge) / 0.549 (daily naive) / 0.419 (action ridge) / 0.528 (action ridge) |
| T3 dark night | A test | Brier 0.0015, AUC 1.000 (logistic) |
| T4 counterfactual | C_cf test | per variant and target in BENCH_v0.md (zero effect / val mean effect) |
V = food_stock, wood_stock, fire_lit, cold_members, alive, deadwood_total, food_on_bushes_total, adults_working.

## 3.3 Known weaknesses of v0 (the planner's review of Pass 7 — design around them, report against them)
1. **Severity is a visible label that almost fixes the winter.** On held-out seeds 32–95 (256 seed-winters): mild → 0 dark nights in 152 of 152, normal → 1 dark night on the seventh in 62 of 65, harsh → 2 or 3. The severity-only R² of the dark-night count is 0.94. `winter_severity` is a column of `global.npy` and an input of the ridge. A model fed severity can memorise the outcome; **every M1 result is reported with and without the severity input** (the severity-blind model must infer the winter from the observed burn and fall).
2. **T3 is arithmetic** (the 18:00 stock and the night's rate decide 02:00). Report it, but it does not rank models; add the longer-lead T3b of 7.4.
3. **T4's "mean MAE over four targets" mixes units** (cold member-hours dominates). Report T4 per target, plus a normalized score (7.5); never the cross-unit mean.
4. **T2 starts only at 06:00**, where `cold_members` and `adults_working` are always 0. Report the bench protocol and, in addition, starts at every hour (7.3).
5. **No intervention changes deaths** in C_cf; drop the deaths target from T4 rankings (report it).
6. Windfall often applies 0 in winter (snow cap); the intervention log records `applied`.

## 3.4 What WM-1 is for
Three questions, answered with numbers: (a) can a learned model beat the simple baselines on rollouts and counterfactuals, not only on one step? (b) does it need the severity label, or can it infer the winter? (c) what does v0 lack for a harder benchmark (input to Pass 8's dataset v1)? Then WM-2 (later) puts a model in the game: the god's foresight ("what happens if I cast rain tonight?") and aggregate surrogates for far settlements.

---

# 4. NON-NEGOTIABLE RULES

## 4.1 Comparability
Every number compared with a baseline uses the bench's own definitions: the same splits and roles, the same variables, the same nMAE denominators (`bench_v0`'s std over the scored rows), the same rollout start rows, the same T3 evenings and the same C_cf targets. Import them from `bench_v0.py`; never re-implement a metric when the bench has it. Part 1 proves this by reproducing the baselines through your harness.

## 4.2 No leakage
Fit on train roles only; choose hyper-parameters and early-stop on val roles only (A val + B val; C_cf val for T4 calibration only); touch test roles, OOD_v12 and OOD_long only for final evaluation, once per configuration (the evaluated configurations are listed in METRICS before their test numbers exist). Normalization statistics come from train rows. In v0 (recorder schema 1) only `lifespan_days` is marked hidden, but three more things reveal the severity exactly: `global.npy`'s `winter_severity`, `settlements.npy`'s `wood_target` (in winter `winter_base_days` 4 × the severity's rate × 13 = 104, 156 or 208) and every field of `latent.npy` (the change of `wood_winter_acc` gives the day's fall). **Severity-blind** variants never see any of them; **severity-on** variants may add `winter_severity` and `wood_target`. `latent.npy` and `lifespan_days` are never inputs. A unit test asserts the input field list of each variant.

## 4.3 Reproducibility
Every run writes `config.json` (code commit, data manifest sha, seed, all hyper-parameters, device, torch version), `train_log.jsonl`, `metrics.json`. Seeds fixed (`torch.manual_seed`, numpy). Re-running the main M1 configuration with the same seed must reproduce its test metrics to 1e-6, or the nondeterminism must be measured (two runs) and recorded.

## 4.4 The gate (before the green branch moves)
On the Mac: `python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'` (Pass 7's tests plus yours; `tools/worldmodel/wm1/__init__.py` must exist so discovery enters `wm1/` — check the test count includes yours; your torch tests skip with a message when torch is missing). On the DGX: the same discovery inside the training environment, with torch present (no skips allowed there). No file outside 2's allowed paths changed (`git diff --name-only ce7acb25..HEAD` lists only those).

---

# 5. PART 0 — Setup and verification
1. Record start time, `uname -a` on both machines, the DGX's `nvidia-smi`, free disk and memory. Commit `GOAL.md`.
2. Environment per 1.3; record the outcome.
3. Clone on the DGX (1.2), verify the dataset: `sha256sum -c` over the manifest entries of one episode per split, plus the total file count (4,660) and size (1,368.3 MiB). A mismatch stops the run until explained.
4. Read `docs/worldmodel/README.md`, `DATASET_v0.md`, `BENCH_v0.md` and `bench_v0.py`; write `docs/worldmodel/wm1/NOTES.md` (one page: the fields you will use, the hidden ones, the task definitions in your own words). Commit, push.

# 6. PART 1 — Data pipeline and the evaluation harness

## 6.1 `tools/worldmodel/wm1/data.py`
- Settlement-level row features (one vector per hour per episode): the columns of `global.npy` allowed by 4.2 for the variant; `settlements.npy` reduced as the bench reduces it (stocks summed, fire lit = max over settlements, formation omitted, `wood_target` only in severity-on variants, NaN → 0 with a presence flag); calendar features (hour one-hot, season one-hot incl. −1, day of season); the action vector (per kind, the requested amount applied at this row, as `bench_v0` encodes it); optionally `winter_severity` one-hot (the severity-on variant). Cumulative tallies enter as their hourly differences.
- Targets: the next row's change of every continuous feature (normalized by train std of the change) and the next `fire_lit` as a binary.
- Windows: random contiguous windows of `L` rows (default 168) from train episodes; validation and test use whole episodes.
- Unit tests on synthetic episodes (shape, normalization from train only, hidden fields excluded, action alignment: an intervention of tick t is a feature of row t and first shows in row t+1).

## 6.2 `tools/worldmodel/wm1/evaluate.py` — one harness for every predictor
A predictor implements `step(history, actions) -> next row` (and optionally a probability for `fire_lit`). The harness runs T1, T2 (bench starts and every-hour starts), T3, T3b, T4 and T5 (section 7) for any predictor, writing `metrics.json` and a Markdown table.
**Fidelity check (W1):** wrap `bench_v0`'s persistence, daily-naive, ridge and action-ridge as predictors and run them through the harness's T1 and T2; run the rule and logistic baselines through its T3 code and the zero-effect and val-mean-effect baselines through its T4 code. Every number must reproduce `BENCH_v0.md` / `bench_v0.json` to 1e-3 on every split. Commit the reproduction table. Until W1 holds, no model number is reported.

# 7. TASKS (as the harness computes them)
## 7.1 T1 next hour — bench definition.
## 7.2 T2 rollout — bench definition (starts 06:00 of days 3, 10, 17 …, horizons 24 h and 168 h; hour and season from the calendar; actions as recorded in B; severity held at the start row's value in severity-on variants).
## 7.3 T2-any — additionally, starts at every hour of the same days (24 per start day), reported separately as mean nMAE over V at 24 h and 168 h, per split.
## 7.4 T3 and T3b
- T3: bench definition (18:00 → fire out at 02:00); a model's probability is its `fire_lit` head after rolling 8 steps from 18:00 (or the fraction of 16 sampled rollouts if the model samples).
- T3b winter outlook (new): at 06:00 of each winter's third day (one-based display days 24, 52, 80, 108), predict the number of dark nights among the five remaining winter nights (evenings of days 24–28, and so on), from observables only. Metrics: MAE and exact accuracy. Baselines (implement in the harness): the train mean; ridge on (wood stock at 06:00, deadwood total, the wood stock drop over each of the two previous nights, deadwood total change over the two previous days, members); and the severity oracle (the train mean per severity — an upper reference, not a competitor). A model's prediction: roll out to the end of the winter and count dark 02:00 rows.
## 7.5 T4 counterfactual
On C_cf test seeds: start from the factual episode's row at the intervention tick (the prefix is identical; row t is recorded before tick t's intervention), roll out twice to day 29 07:00 (tick 40,260; `c_cf_targets` reads that row) — with the intervention and without — and compute the Δ targets with `bench_v0.c_cf_targets` on rows that are the factual rows before the intervention tick and the predicted rows from it on (rain and cold-snap targets start at day 22 18:00, before their interventions). Report per variant and target: MAE, sign accuracy, and **normalized MAE = MAE / mean |true effect|** of that variant and target on the test seeds (undefined, "—", when that mean is 0, as for deaths in v0). Rank on dark nights, cold member-hours and food store (deaths reported, not ranked). Compare with zero effect and the val-mean effect.
## 7.6 T5 agent next hour (new; A_obs agent episodes)
Train 0–31, val 96–99, test 112–119 (and B test 1056–1063 for actions). Per living agent and row: next state tag (accuracy), next target kind (accuracy), next position (mean Chebyshev error in tiles), next hunger, thirst and fatigue (nMAE). Baselines: persistence; the same agent 24 hours earlier.

# 8. PART 2 — M1: settlement-level recurrent world model (`wm1/m1_gru.py`)
- Architecture: input projection → 2-layer GRU (hidden 256) → heads: Gaussian mean (and log-variance) for the continuous changes, a Bernoulli logit for `fire_lit`. Prediction uses the mean (deterministic); the variance is reported (calibration: share of true changes within ±2σ on val).
- Training: AdamW (lr 3e-4, weight decay 1e-4), batch 64 windows of 168 rows, teacher forcing plus a rollout loss (unroll k steps on its own predictions, k grows 1 → 24 over the first half of training), early stopping on val mean nMAE of T2-24 h (bench starts) with patience 5 evaluations; at most 2 hours per run.
- Variants: **sev-on** (severity one-hot as input) and **sev-blind** (no severity); **action-aware** (A + B train) and **action-blind** (A train only). Main configuration: sev-blind, action-aware. Three seeds of the main configuration, one of each other.
- Evaluate every variant on T1, T2 (bench and any-hour), T3, T3b, T4 through the harness. Commit configs, metrics and the tables; push.

# 9. PART 3 — M1b: causal transformer (`wm1/m1_tf.py`, optional per drop order)
4 layers, d_model 256, 4 heads, context 168 rows, learned hour/season embeddings, the same heads and training recipe. Main configuration only, one seed.

# 10. PART 4 — M2: agent-level model (`wm1/m2_agents.py`)
- Per agent per row: its own observable features (no `lifespan_days`), its settlement row, the global row and calendar, plus the mean and max of its 8 nearest living agents' features. A small set transformer over all living agents of the settlement (2 layers, d 128) or an MLP over the above (choose by val).
- Heads: state tag, target kind (cross-entropy), position change (dx, dy as classes −3…+3, clipped), needs (regression).
- Evaluate T5; also aggregate: sum predicted `adults_working` over agents and compare with the settlement truth (nMAE).

# 11. PART 5 — M3: text world model (optional, first to drop)
- Serialize each day's 06:00 row (V, season, day, the day's interventions) as a compact JSON line; the task is to predict the next day's 06:00 JSON.
- Base model: a Qwen checkpoint of at most 2B parameters **already present in `~/.cache/huggingface/hub` on the DGX** (list it; never read `~/github/worldsim-training`). If none is present, download one instruct checkpoint of at most 1B parameters from Hugging Face if the network allows; otherwise drop Part 5 and say so.
- LoRA (rank 16) on A train days, 1 epoch, at most 2 hours. Evaluate on A test days: JSON parse rate, nMAE of V at 24 h against daily naive (T2 bench starts at day granularity).

# 12. PART 6 — Analysis
`docs/worldmodel/wm1/ANALYSIS.md`: where models beat baselines and where they do not (by task, split, variable, season, severity); sev-on vs sev-blind (does the blind model infer the winter? T3b and the winter rows of T2); OOD_v12 and OOD_long degradation; T4 per variant with the error cases; calibration; and **"what v1 needs"** — concrete changes to the simulation and the dataset that would make the benchmark harder and fairer (this feeds Pass 8).

# 13. TARGETS (reported; honest failure is allowed, a missing number is not)
| # | target |
|---|---|
| W1 | harness fidelity: bench baselines reproduced to 1e-3 on T1–T4, every split |
| W2 | M1 main (sev-blind, action-aware) beats the best baseline on T2 168 h mean nMAE on A test, B test and OOD_long by ≥ 10 % relative (0.9 × the `bench_v0.json` values: ≤ 0.4867 / 0.4938 / 0.4756), and does not lose on T1 by more than 5 % |
| W3 | T4: on dark nights and cold member-hours, M1 main's normalized MAE below the best baseline for ≥ 3 of 4 variants; sign accuracy ≥ 0.75 on those targets |
| W4 | T3b: M1 main's MAE below the ridge baseline; report the gap to the severity oracle |
| W5 | severity ablation: sev-on vs sev-blind on T2 winter rows, T3b and T4, with the three-seed spread of the main configuration |
| W6 | M2: T5 state-tag accuracy ≥ persistence + 5 points and position error below persistence on A test |
| W7 | reproducibility: same-seed rerun of M1 main reproduces test metrics to 1e-6, or measured nondeterminism recorded |
| W8 | leakage audit: an independent reviewer confirms 4.2 from the code and configs |

# 14. PART 7 — Independent review, final, report
1. **Independent review:** a built-in `general-purpose` subagent with fresh context gets `git diff ce7acb25..HEAD`, sections 4, 6, 7 and 13, and the W table; it checks leakage (roles, normalization, hidden fields, early stopping), metric fidelity against `bench_v0.py`, the T4 rollout protocol (identical prefix, both branches from the same state), and claims in ANALYSIS.md against the metrics files, with file:line evidence. Fix confirmed defects (two rounds maximum); record every finding in DECISIONS.md.
2. Gate (4.4) on the final WIP head; fast-forward `codex/worldmodel-2026-10` if green; draft PR (section 2).
3. `docs/worldmodel/wm1/REPORT_WM1.md` in Korean in the section 16 format, and `docs/worldmodel/wm1/METRICS.md` (every run: id, config, val and test numbers).

# 15. 인게임/결과 확인사항 (주인이 직접 확인할 것)
1. `docs/worldmodel/wm1/REPORT_WM1.md`의 W 표에서 학습 모델이 168시간 롤아웃과 반사실(T4)에서 기준선을 이겼는지 본다. 한 시간 예측(T1)만 이긴 것은 의미가 작다.
2. 혹독함을 입력에서 뺀 모델(sev-blind)이 겨울을 추론하는지 T3b와 겨울 롤아웃 오차로 확인한다. sev-on과 차이가 크면 v0의 겨울은 여전히 라벨로 정해진다는 뜻이다.
3. 반사실 예시 하나(같은 시드, 비 vs 비 없음)의 예측 궤적과 실제 궤적 그림이 ANALYSIS.md에 있다.
4. DGX의 체크포인트 경로와 재학습 명령이 METRICS.md에 있어 그대로 다시 돌릴 수 있다.
5. 게임 코드(Rust, GDScript)는 바뀌지 않았다.

# 16. 구현 완료 보고 (REPORT_WM1.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
데이터 파이프라인(입력·숨은 필드·정규화), 평가 하네스와 재현 검증(W1), M1/M1b/M2/M3 구조와 학습법, 반사실 롤아웃 방식, 누설 방지

### 기능 설명
연구 관점에서 무엇을 할 수 있게 되었는가? WM-2(게임 안 예견, 대리 모델)로 가는 길에서 어디까지 왔는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
W1~W8 표: 기준선 / 모델(세 시드면 평균±범위), 과제·분할·변수별 주요 수치, sev-on vs sev-blind, 행동 인지 vs 무시

### 환경
DGX 파이썬·torch·CUDA 구성, GPU 공유 상황, 학습 시간과 메모리

### 독립 리뷰
지적 사항과 처리 결과

### 자율 결정 목록
DECISIONS.md 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 멈춘 구간, 컨텍스트 압축 뒤 목표 파일을 다시 읽은 시점, DGX 작업 목록

### 확인된 제한사항

### 게이트 결과
(맥과 DGX의 unittest 수치, 허용 경로 밖 변경 없음 확인)

### 정직한 판단
학습 모델이 이 세계를 기준선보다 잘 아는가? 그 차이는 어디서 오는가? v0 데이터의 한계와 v1에 필요한 것은?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the draft PR URL (or why none), the DGX run directory of every reported model, and the time spent per Part.
