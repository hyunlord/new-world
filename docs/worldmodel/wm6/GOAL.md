# GOAL: WorldSim — WM-6: is the headroom learnable? Nonlinear direct effect models and a learning curve over five times the counterfactual data, then the paper's conclusion

Repository `hyunlord/new-world`. Base: `codex/worldmodel-2026-10-wip` = `0f5ee066338acfe5eaa06366b788e2f64f2fa4ea`, `codex/worldmodel-2026-10` = `7405a470f3244f4598b5b6cd3f319e3aee1615bd` (WM-5 final; draft PR #512).
This run is executed by **Claude Code on the Mac** in the WM clone, unattended. All recording, training and evaluation run **on the DGX Spark over ssh**. Keep the Mac light (edits, git, tiny unit tests, web lookups for citations). Never take `/Users/rexxa/work/.mac_quiet.lock`; an art run may be using the Mac.

**Where WM-5 left the paper** (`docs/worldmodel/wm5/WM5.md`, `docs/worldmodel/paper/paper.md`): the benchmark has room. On v2 at 7 days the effect floor (another noise replicate's true effect) is 0.187 against `zero` 0.612, but nothing evaluated uses that room:

| method | v2 7 d |
|---|---|
| CRN, trained to its authors' full schedule | 1.649 |
| CT, trained to its authors' full schedule | 1.243 |
| `zero` | 0.612 |
| `an5_arm` (≈ the arm's mean effect) | 0.485 |
| `ridge_obs`, a linear map of the observable pre-act state | 0.480 |
| `ridge_priv`, the same with the hidden state | 0.434 |
| floor | 0.187 |

**The question a reviewer asks next:** is the gap between 0.434 and 0.187 **learnable**, or is the per-world effect essentially chaotic given the state? The two answers make different papers.
- **Learnable** — a nonlinear model of the pre-act state approaches the floor as data grows: the information is there, and sequence world models and causal estimators fail to extract it.
- **Not learnable** — even privileged nonlinear models stay flat: per-world intervention effects in this society are close to unpredictable, and only the mean effect is learnable.

**WM-6 answers it with a learning curve.**
- **Data:** record 1,600 new v2-style counterfactual worlds, so there are 2,000 training worlds instead of 400.
- **Models:** train nonlinear direct effect models (an MLP on WM-5's pre-act features, a recurrent encoder of the last week of observable history, and their privileged variants) at N = 400, 800, 1,600 and 2,000 worlds.
- **Paper:** update it to state the answer.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-wm1
git fetch origin && git checkout codex/worldmodel-2026-10-wip
git merge --ff-only origin/codex/worldmodel-2026-10-wip   # 0f5ee066
mkdir -p docs/worldmodel/wm6
cp ~/Downloads/claude_goal_worldsim_wm6.md docs/worldmodel/wm6/GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/worldmodel/wm6/GOAL.md>
```
Hooks stay neutralized in this clone (`.claude/settings.json` = `{}`, `--skip-worktree`). Never edit, stage or commit it, and never clear the flag.

**This document is `docs/worldmodel/wm6/GOAL.md`.**
- Commit it unchanged in Part 0.
- Where it disagrees with earlier WM goals, this one wins.
- Re-read the relevant section at every Part start and after every compaction or pause.
- Your final message must contain the section 12 report, the rule table, commit hashes and `git ls-remote` heads.

## 0.2 Overrides of CLAUDE.md
As WM-5's 0.2:
- no `install_hooks.sh`;
- never `lead/main`/`main`;
- no harness pipeline (4.5 is the gate);
- built-in `general-purpose` subagents only.

First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decisions go in `docs/worldmodel/wm6/DECISIONS.md` (`L0.1`, …); times go in `RUNLOG.md`.
2. **Do not end your turn between Parts or while DGX jobs run.** Poll every 10–20 minutes; write the paper while jobs run.
3. **Review loops:** two rounds maximum. Push after every Part.
4. **On failure:** work around, note it, continue. A negative result is valid; a missing number is not.
5. **Time rule:** past **30 hours** of wall time (pauses excluded and recorded), start no new recording or training; evaluate what exists; finish the paper with `[MISSING: reason]` where needed; go to Part 7.
6. **Drop order:**
   1. the GBM (7.4) → 2. the third seed of every neural model → 3. N = 800 → 4. the obs2 transfer evaluation (9.3).

   **Never drop:** Parts 0–3; the recording of at least 1,200 new worlds (N = 1,600); `mlp_obs`, `mlp_priv` and `seq_obs` at N = 400 and the largest N reached, with 2 seeds each; their test evaluation; the paper update; Part 7.
7. **DGX jobs are detached**, with WM-3's watchdog; one training job at a time. Recording uses 8 workers when no training runs and 4 while one does, `nice -n 10`.
8. **Mac sleep:** record the pause; DGX jobs continue.

---

# ⚠️ 1. MACHINES
As WM-5's section 1:
- runs go to `/home/hyunlord/work/wm1-runs/wm6/<run>/`; code worktrees to `wm6/_code/<commit>`;
- venv `/home/hyunlord/work/wm1-venv`, **never `pip install` into it**. An extra package goes into a separate venv created with `--system-site-packages`, installed with `pip install --no-deps`, and recorded.
- The recorder is `/home/hyunlord/work/wm1-runs/_build/src_wm2` (**do not rebuild**); recordings set `WORLDSIM_GIT_SHA=f5373ab2b7679c4a0d3a5c86b256971c86f93d41`.
- New data goes to `/home/hyunlord/work/worldsim-wm-data/v2x/`. `v2/`, `obs/`, `obs2/` and every earlier run are read-only.

**Measured throughput:** recording ≈ 2 s per episode at 8 workers. WM-2's v2 families took one call per seed range: fork 14,280, R = 2, factual included (`tools/worldmodel/wm2/launch/record_v2.sh`).

---

# 2. GIT
- **Branches:** commit to `codex/worldmodel-2026-10-wip`. `codex/worldmodel-2026-10` moves only by fast-forward to a gated WIP commit. Never force-push. Run `git ls-remote` after every push.
- **Commits:** messages `worldmodel(wm6-N): …`. Stage explicit paths only.
- **May change:** `tools/worldmodel/wm6/**`, `docs/worldmodel/wm6/**`, `docs/worldmodel/paper/**` (including `paper_src.md`, `paper_numbers.json`, figures and references), and `tools/worldmodel/wm5/figures.py` **only** to add WM-6 sources to its number map (record the change). Plus one line at the top of `docs/worldmodel/wm5/WM5.md`.
- **Read-only:** everything else. Import and wrap.
- **PR #512:** stays draft. After the green fast-forward, add a comment with the rule table and the paper link.

---

# 3. CONTEXT (verify each)
- **v2** (`tools/worldmodel/wm2/v2.py`):
  - fork tick 14,280; replicates {0, 1};
  - arms `sp240L`, `sp120L`, `gf240E`, `rw120E` (`ARM_SPEC`); W3 cells use `sp240L`, `gf240E`, `rw120E`;
  - roles: train 40000–40399, val 40400–40479, test 40480–40559 (`ROLES`);
  - hidden fields: `fission_pressure`, `fission_threshold`, `world_variety`, `latent` (`HIDDEN_FIELDS`).
- **The `sp*L` arms** need the world's `T_low` from the timeline (`schedules_p14.py`). WM-2 6.2: 71 of 80 test worlds have one.
- **WM-5's direct regressions** (`tools/worldmodel/wm5/effects5.py`):
  - `obs_features` (L341): 37 state features at c, their means over c−23…c and c−167…c, 30 calendar features, and the earlier-cast terms (0 on v2);
  - `priv_features` (L353) adds the hidden fields;
  - per-cell ridge with λ chosen on v2 val (`fit_cells` L392, `LAMBDAS` L71);
  - targets: the true effect divided by the cell's scale.
- **Scales, cells and the W1 rule:** as WM-4/5 (12 cells per lag, floored scales, world-level paired bootstrap B = 2,000).
- **The v2 test floor** (WM-5 F1): 0.187 (7 d), 0.267 (30 d).

---

# 4. RULES
## 4.1 Comparability
- **Test set:** v2's 80 test worlds and WM-5's scoring code, unchanged; the floor is reported beside every number.
- **Training cases:** use **replicate 0 only** at every N, so that N is the only thing that changes along the curve. The new worlds are recorded with R = 1.
- **WM-5 reference:** the WM-5 ridge on both replicates at N = 400 is re-run as a reference and must reproduce WM-5's numbers. Committed WM-2 … WM-5 numbers do not change.

## 4.2 Seeds
- **New v2 train worlds:** **50000–51599** (1,600).
- **Nested training sets:**
  - N = 400: WM-2's 40000–40399;
  - N = 800: + 50000–50399;
  - N = 1,600: + 50000–51199;
  - N = 2,000: + 50000–51599.
- **Validation:** v2 val (40400–40479), unchanged.
- **Unused check:** `grep -rhoE '"seed": ?5[01][0-9]{3}\b' docs/worldmodel | sort -u` must be empty, and on the DGX `find /home/hyunlord/work/worldsim-wm-data -maxdepth 4 -name '*_s5[01][0-9][0-9][0-9]*' | wc -l` must be 0. Otherwise shift by +2,000 and record.

## 4.3 No leakage
- **Observable models** read WM-2's observable input lists only (unit test). Privileged models are labelled diagnostic everywhere and never enter a method ranking against the observable ones.
- **Test** worlds are touched only in evaluation.
- **Selection:** model selection and early stopping use v2 val only.

## 4.4 Reproducibility
As WM-5: config, commit, manifest sha, seed, hyper-parameters, device, stop reason.

## 4.5 The gate
- **Mac:** `python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'` (recorder-dependent skips allowed).
- **DGX:** the same with zero skips.
- **Paths:** `git diff --name-only 0f5ee066..HEAD` lists only section 2's paths.
- **Paper numbers:** `figures.py check` passes.

---

# 5. PART 0 — Setup
- **Record:** start time and the machines' state; sha256 of the recorder and of the WM-2 v2 manifest.
- **Seed check:** 4.2.
- **Files:** commit `GOAL.md`; create `DECISIONS.md` and `RUNLOG.md`.

Push.

---

# 6. PART 1 — Five times the counterfactual data (DGX)
1. **Timeline:** `worldsim_probe --scene game_v19 --seeds 50000..51600 --days 120 --clock-samples` with WM-2's clock list and flags (`record_v2.sh`), then `timeline_p14.py` and `schedules_p14.py --arms sp240L,sp120L,gf240E,rw120E`, into `v2x/`.
2. **Families:** `worldsim_wm --scene game_v19 --seeds 50000..51600 --days 120 --every 60 --schedule-dir v2x/schedules --include-factual --fork-tick 14280 --replicates 1` into `v2x/flat/`.
   - That is ≈ 5 episodes per world (fewer where `T_low` is missing): ≈ 8,000 episodes, ≈ 4.5 h at 8 workers.
   - Verify every episode (`wm_data.verify_episode`) and every family's prefix (members equal to the factual up to row 238, per WM-2 6.2's rule).
3. **Manifest:** `docs/worldmodel/wm6/v2x_manifest.json` (WM-4/5's fields; wm6's own loader, since `families_v2.build_manifest` asserts R = {0, 1}).
4. **Dataset note:** `DATASET_V2X.md`:
   - counts;
   - arms present per world;
   - the effect distribution per arm and lag on the new worlds;
   - a two-sample comparison (KS statistic per cell) against WM-2's 400 train worlds, so that the new worlds are shown to be the same population.

Commit, push. **Start 7's models on N = 400 while recording runs.**

---

# 7. PART 2 — Models (`tools/worldmodel/wm6/`)
7.1 **Shared:**
- cases and features as WM-5's `effects5` (`obs_features`, `priv_features`);
- targets: the 24 normalized cell targets (4 outcomes × 2 lags, per arm);
- one model per method, with the arm given as a one-hot input;
- loss: mean absolute error over the outcome × lag targets of the case's arm (the metric's own form);
- censoring and exclusions as WM-2.

7.2 **`ridge_obs` / `ridge_priv`:** WM-5's code at each N, replicate 0 only. At N = 400 also with both replicates (the WM-5 reproduction, 4.1).

7.3 **`mlp_obs` / `mlp_priv`:**
- **Architecture:** inputs normalized by train mean/std; 3 hidden layers × 256, GELU, dropout 0.1.
- **Training:** AdamW, lr 1e-3, weight decay 1e-4, batch 256, at most 400 epochs; early stopping on v2 val cell error (patience 30 epochs). Seeds 0, 1, 2.
- **One small grid at N = 400 only**, chosen on val and then frozen for every N: width {256, 512} × dropout {0.1, 0.3}.

7.4 **`gbm_obs` / `gbm_priv`** (droppable):
- **Model:** LightGBM, one regressor per (arm, outcome, lag) cell, L1 objective. 500 rounds with early stopping on val, `num_leaves` 31, learning rate 0.05, `min_data_in_leaf` 20.
- **Install** into a separate venv (section 1). If it cannot be installed or imported, record it and drop.

7.5 **`seq_obs` / `seq_priv`:**
- **Encoder:** a 2-layer GRU (hidden 128) over the last 168 hourly rows before c of WM-2's 74 action-aware inputs (the hidden fields appended for `seq_priv`), normalized by train stds.
- **Head:** the last hidden state concatenated with `obs_features` → the MLP head of 7.3 (frozen grid choice).
- **Training:** the same training rule; seeds 0, 1, 2.

7.6 **Tests** (`test_wm6.py`):
- nested set construction;
- replicate-0-only cases;
- feature leakage (observable models never read a hidden field);
- the MLP and the GRU recover a planted nonlinear effect on synthetic families;
- the WM-5 reproduction at N = 400 (both replicates) equals WM-5's committed ridge numbers.

Commit, push.

---

# 8. PART 3 — Pre-registration
`docs/worldmodel/wm6/METRICS.md` is committed before any test-role number of a WM-6 model. **Closure** of a model = (`zero` − model) / (`zero` − floor) on the 12 cells of a lag, with a CI.

| # | rule |
|---|---|
| C1 | **nonlinearity on observables (primary), v2 7 d, largest N:** the better on val of `mlp_obs` and `seq_obs` vs `ridge_obs` at the same N: W1 rule |
| C2 | **the same at 30 d** |
| C3 | **nonlinearity on the full state, 7 d and 30 d, largest N:** `mlp_priv` vs `ridge_priv`: W1 rule (diagnostic) |
| C4 | **data, 7 d:** for `mlp_obs`, `seq_obs`, `mlp_priv`: the largest N vs N = 400, W1 rule; the slope of error against log2 N with a bootstrap CI (reported) |
| C5 | **closure (reported):** every model's closure at every N and lag |
| C6 | **observable vs hidden (reported):** `mlp_priv` − `mlp_obs` and `seq_priv` − `seq_obs` at the largest N |

**Pre-registered reading** (written before results, applied after):
- **"Learnable from observables":** the C1 model's 7-day closure ≥ **0.5** at the largest N, or its C4 slope's CI upper bound < 0 and closure ≥ 0.35.
- **"Learnable only with hidden state":** `mlp_priv` closure ≥ 0.5 while observable closure < 0.35.
- **"Not learnable at this scale":** every closure < 0.35 and no C4 slope's CI excludes 0.
- **Otherwise:** "partially learnable", with the numbers.

---

# 9. PART 4 — Training and evaluation (DGX)
1. **Training:** the models of section 7 at N = 400 (while recording runs), then 800, 1,600 and 2,000 as data become available. Every neural run has `--max-seconds` 3,600; record its stop reason.
2. **Test evaluation:** each model is evaluated once on v2 test only after Part 3 is pushed, at 7 d and 30 d, per cell and averaged, beside `zero`, `an5_arm`, `wm3_full` and the floor.
3. **obs2 transfer** (droppable): `mlp_obs` and `seq_obs` at the largest N on obs2 test (WM-5's cases and features, the earlier-cast terms included), beside WM-5's E1 row. Report only.
4. **Run files:** copy small files into `docs/worldmodel/wm6/runs/<run>/`.

# 10. PART 5 — Judge
`tools/worldmodel/wm6/judge6.py`: every rule, the learning curves (error and closure against N per model and lag), and the pre-registered reading → `docs/worldmodel/wm6/runs/judge6/judge.md` and `judge.json`. Commit, push.

---

# 11. PART 6 — Analysis and the paper
1. **`docs/worldmodel/wm6/WM6.md`:** the data; the curves; the reading; what it means for the benchmark. One line at the top of `docs/worldmodel/wm5/WM5.md`: "Followed by docs/worldmodel/wm6/WM6.md (learning curves of nonlinear direct effect models)."
2. **The paper** (`paper_src.md` → `paper.md` through `figures.py`):
   - add a section on learnability: the learning-curve figure and the closure table;
   - update the abstract, introduction and conclusion **only as the pre-registered reading allows**;
   - list the new data in the datasheet;
   - every new number goes through `paper_numbers.json`;
   - any new reference is verified on the web and logged in `REFERENCES_CHECK.md`.
   - Keep `[AUTHORS]`, `[VENUE]` and the release placeholders.
3. **Submission readiness note:** `docs/worldmodel/paper/READINESS.md` (Korean is fine), covering:
   - what a datasets-and-benchmarks reviewer will still ask (one simulator, one scene, three interventions);
   - which of those the paper answers;
   - the remaining work, as a list with an effort estimate each.

---

# 12. PART 7 — Review, gate, report
1. **Independent review** (fresh `general-purpose` subagent, two rounds maximum). Scope:
   - the new data's population check (KS);
   - nested sets and replicate-0-only use;
   - leakage;
   - the WM-5 reproduction;
   - training stops;
   - pre-registration order (METRICS commit time vs every test output time on the DGX);
   - the reading applied exactly as written;
   - every new paper number and claim.

   Findings go in `docs/worldmodel/wm6/REVIEW.md`.
2. **Gate and publish:** run the gate (4.5) on both machines; fast-forward `codex/worldmodel-2026-10`; add the PR #512 comment.
3. **Report:** `docs/worldmodel/wm6/REPORT.md` in Korean (the format below), and the same text in the final message.

```
## 구현 완료 보고

### 구현 의도
### 구현 내용
(Part별 파일과 핵심 구조)
### 구현 방법
(새 반사실 세계 녹화와 동질성 검사, 중첩 학습 집합, 비선형 직접 효과 모델(MLP·GRU·GBM, 관측·특권), 사전 등록된 해석 규칙, 논문 갱신)
### 기능 설명
여유는 학습 가능한가? 관측 상태로? 숨은 상태가 필요한가? 데이터를 늘리면 바닥에 다가가는가? 논문의 결론은 무엇으로 바뀌었는가?
### 변경된 파일 목록
(커밋별)
### 측정 결과
C1~C6 표(신뢰구간), 모델·N·지평별 오차와 닫힘 비율(학습 곡선), 기울기와 구간, 사전 등록 해석의 판정, 셀별 표, obs2 전이, 새 데이터의 동질성(KS), 실행별 종료 이유·시간
### 환경
### 독립 리뷰
### 자율 결정 목록
### 실행 기록
### 확인된 제한사항
### 게이트 결과
### 정직한 판단
이 결과로 논문은 어떤 이야기를 하는가? 제출하려면 무엇이 남았는가(READINESS.md 요약)?
```

The final message also includes:
- every commit hash with its Part;
- both branch heads with `git ls-remote` confirmation;
- the PR #512 comment URL;
- the DGX paths of the new data and every run;
- the time per Part.
