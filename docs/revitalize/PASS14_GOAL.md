# GOAL: WorldSim — Revitalize Pass 14: when and how hard must the god push? A dose–timing map of interventions on `game_v19` (no rule changes)

Repository `hyunlord/new-world`. Continue the Pass 13 line: `codex/revitalize-2026-10` = `2ba871c09819c240904510bbf26b0fb2a59a79b0` (Pass 13's code, gate green), `codex/revitalize-2026-10-wip` = `a12d7a355cd89b8f5aca83b2dae8a52128b889fe` (docs and probe outputs after it). Draft PR #510.
This run is executed by **Claude Code on the Mac** in the simulation clone, unattended; **all recording and scoring run on the DGX Spark over ssh**. At the same time an art run (Codex, Art Pass 8) works on the Mac in `/Users/rexxa/work/new-world-art-prod` and measures frame times there, and a learned-model run (WM-1.3) trains on the DGX's GPU: keep the Mac light (edits, small unit tests, git) and keep your DGX jobs gentle (section 1).

**This pass changes no game rule and no Rust code.** `game_v19` stays exactly as Pass 13 froze it. It is an experiment: which acts of the god, at which moments and in which amounts, change the shape of a world (when villages split, how many there are, how many people live) by more than noise does. Its product is an intervention map that dataset v2 will use.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # a12d7a35
cp ~/Downloads/claude_goal_worldsim_pass14.md docs/revitalize/PASS14_GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS14_GOAL.md>
```
The clone's hooks are neutralized as before (`.claude/settings.json` = `{}`, `--skip-worktree`); never edit, stage or commit it and never clear the flag. Leave the local `.gitignore` edit and `.ignore` uncommitted. Stage explicit paths only; never stage outputs of earlier passes.

**This document is `docs/revitalize/PASS14_GOAL.md`.** Commit it unchanged in Part 0. Re-read the relevant section at the start of every Part and after every compaction or pause. The owner drives the run with `/goal`: a small model reads only the transcript, so the section 12 report, the K table, commit hashes and `git ls-remote` heads must appear in your final message.

## 0.2 Overrides of CLAUDE.md
As Pass 13's 0.2 (no `install_hooks.sh`, no `lead/main`/`main`, no harness pipeline, built-in `general-purpose` subagents only, a blocking hook other than `/goal` → record, push, report). First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 14`.
2. **Do not end your turn between Parts or while DGX jobs run.** One task per Part; poll long jobs every 10–20 minutes with short calls.
3. Review loops: two rounds maximum. Push after every Part.
4. On failure: work around, note it, continue. A negative result (no arm beats noise) is a valid result; a missing number is not.
5. **Time rule:** past **10 hours** of working time (pauses excluded and recorded), finish the current step, push, go to Part 6.
6. **Drop order:** the wood / weather arms of Stage A (6.2 group W) → Stage B's second half of seeds (keep 14000–14031; PREREG states that K2 is then judged on those 32). Never drop Part 0, the factual and noise runs, the food arms of Stage A, Stage B's pre-registration, or Part 5–6.
7. **DGX jobs are detached:** launch every long job with `setsid nohup <cmd> > <log> 2>&1 &`, record its PID and an `.exit` file. **Connection loss:** if the DGX stops answering, keep polling every 10 minutes for up to 2 hours while doing Mac-side work; after reconnecting, check the job; a dead recorder job is relaunched with the same command plus `--resume` (it verifies finished episodes, `export.rs` ≈ L526–542).

---

# ⚠️ 1. MACHINES
- **Mac** (`/Users/rexxa/work/new-world-pass4`): edit, commit, push, Python unit tests on tiny synthetic inputs. **No cargo builds, no probe or recorder runs, no Godot** (the art run measures frame times on this Mac). Never touch `/Users/rexxa/work/new-world-art-prod`, `/Users/rexxa/work/new-world-wm1`, `~/github/new-world-wt/lead` or any other checkout.
- **DGX** (`ssh hyunlord@100.70.109.50`; a throwaway `ssh -F` config or the full address; never edit `~/.ssh/config`):
  - your code clone `/home/hyunlord/work/new-world-p14` (fresh clone of the repository, detached at a pushed WIP commit before every job; `P14_CODE_COMMIT` recorded);
  - your build clone `/home/hyunlord/work/p14-build` detached at `2ba871c0` (the frozen `game_v19`): `cargo build --release -p sim-test` there once; `worldsim_wm` and `worldsim_probe` from it are this pass's frozen binaries (record their sha256);
  - your data `/home/hyunlord/work/worldsim-wm-data/p14/` (everything else under `worldsim-wm-data/` is read-only to you; `w13/` belongs to the WM run);
  - Python: `/home/hyunlord/work/wm1-venv/bin/python` for numpy (use it, never install into it).
  - **Forbidden:** `/home/hyunlord/work/new-world-wm1`, `/home/hyunlord/work/wm1-runs/**`, `~/github/worldsim-training`, ComfyUI (8188), dev servers 3100/3200, any process you did not start.
  - **Be gentle with the WM run:** before every job `nproc`, `uptime`, `free -h`, `nvidia-smi`, `pgrep -fl "m1_tf|m1_gru|worldsim_"`. Run every recording and probe under `nice -n 10` with `--workers` = min(4, nproc − 12); never more than one recorder or probe job at a time; never use the GPU. If the WM run's training is active and the load average exceeds nproc, pause launching until it falls.
- Every recording sets `WORLDSIM_GIT_SHA=2ba871c09819c240904510bbf26b0fb2a59a79b0`.

---

# 2. GIT
- Commits to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose gate (4.2) is green. Never force-push. `git remote -v && git branch --show-current` before every push; `git ls-remote` after.
- Commit messages `revitalize(p14-N): …`.
- **May change:** `tools/revitalize/**`, `docs/revitalize/**`. **Must not change:** `rust/**` (this pass has no Rust change: `git diff --name-only 2ba871c0..HEAD -- rust/` must stay empty), `tools/worldmodel/**`, `docs/worldmodel/**`, and everything outside the simulation line's paths (`scripts/**`, `scenes/**`, `assets/**`, `localization/**`, `docs/art/**`, `tools/art_pipeline/**`, …).
- Merge notes as Pass 13's section 2 (the art branch's getters; nothing to act on).

---

# 3. CONTEXT (planner's review of Pass 13, with evidence)
- **`game_v19`:** a village's pressure P grows at 06:00 on days every fission gate is open (spring to autumn, never winter; eligible from 28 members; age, cooldowns, `max_settlements`) by h = 300 × g × s / 10⁶, where g rises from 0 at 24 members to 1 at 44 and s is the food shortfall below a line of members × adult need × 4 days (about 300 food for 32 members); crowding has weight 0; P decays by 10 a day when h = 0; the village splits when P reaches its hidden θ (exponential, mean 1,200, a world draw). (`rules.rs` `V19_HAZARD`, `V19_PRESSURE`, `game_v19_with`.)
- **Noise is small now:** in the 120-day macro forks (seeds 12000–12031, R = 2, fork at 28,800) noise alone changes the post-fork first-fission day by > 3 days in 0.094 of seeds (mean |Δ| 3.55 days) and alive at day 118 by 2.4 people (v18: 0.688, 34.5 days, 18.4).
- **The god's acts point the right way but are small:** `spoil_food` 60 split earlier in 0.51 of pairs (later 0.13; mean shift −0.7 days), `gift_food` 60 later in 0.53 (earlier 0.14; +2.7 days); J11 failed (`docs/revitalize/probe/p13-6_macro/macro.md`).
- **The planner's reading — mostly an experiment-design failure, not a mechanism failure:** every cast was at tick 29,400 (display day 21, 18:00), the eve of winter. In winter P is frozen, and the pre-winter store is full from winter preparation, so the scarcity term s is 0 at the time of the cast; a single dose then has to survive a winter of homeostasis (a gifted village gathers less). Nothing was cast when the scarcity term was active.
- **Path share is ~0.02** (`p13-5_pilot/paths.md`): in this simulation the future is set by the world's parameters, its deterministic dynamics and the god — not by behavioural noise. Pass 13's J7b band [0.10, 0.50] was an unfounded aspiration; this pass retires it as a gate (4.3).

---

# 4. RULES

## 4.1 What is fixed
`game_v19` and its frozen binaries (section 1); no Rust change; seeds below; every arm's schedule written before the run that uses it and committed.

## 4.2 The gate
`tools/revitalize/gate_pass14.sh`: both Python unittest discoveries (`tools/revitalize`, `tools/worldmodel`) on the Mac **with `WM_BIN=/nonexistent PROBE_BIN=/nonexistent`** (several `tools/worldmodel` tests otherwise run the recorder whenever a local binary exists — no recorder runs on the Mac in this pass; record the skip counts), and the check that `git diff --name-only 2ba871c0..HEAD -- rust/` is empty (so Pass 13's Rust gate on `2ba871c0` still covers the code). Record the discovery counts.

## 4.3 Evaluation amendments (recorded, applied to Pass 13's outputs without rerunning)
- **J6 judged relative:** `game_v19_nofission` vs `game_v16` on the same seeds, per Pass 6 Y9 clause: pass if the v19n statistic is no worse than v16's by more than 0.03 (the absolute 0.25 bound is reported only — v16 itself is at 0.241). Re-score Pass 13's held-out and tuning J6 with `evaluate_pass14.py` and state the result beside Pass 13's verdict (never overwriting it).
- **J7b:** path share is reported, not gated.

## 4.4 Seeds
- Stage A: seeds 12000–12031 (the fork seeds Passes 9–13 used).
- Stage B (confirmation): **fresh seeds 14000–14063**, never used before Part 4, not in tests (tests use seeds < 1000).
- Seeds 30000–33999 belong to the WM run; 24000–24111 and 26000–26063 are earlier pilots.

---

# 5. PART 0 — Setup and the factual worlds
1. Record start time, `uname -a` of both machines, the DGX's `nproc`, `free -h`, `nvidia-smi`, `pgrep -fl "m1_tf|m1_gru"` (the WM run's state). Commit `PASS14_GOAL.md`; add `## Pass 14` to DECISIONS.md and METRICS.md.
2. DGX: clone, build the frozen binaries at `2ba871c0` with the repository's pinned Rust toolchain (record time and sha256; round 3 ran at `9f0d38ca`, whose Rust equals `2ba871c0`). **Identity:** run `worldsim_probe --scene game_v19 --seeds 0..32 --days 120` with round 3's flags (`METRICS.md` ≈ L2494: `--clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120 --workers 4 --check-determinism`; here `--workers` per section 1) and compare with `docs/revitalize/probe/p13-3_round3_v19_120day/summary.json` using `tools/revitalize/compare_summaries.py` (timing excluded): identical on every shared key; a difference means a portability problem — stop the pass and report it. **Replicate 0:** `worldsim_wm --scene game_v19 --seeds 12000..12002 --days 120 --every 60 --out p14/unforked` and compare the array sha256 values in each `meta.json` with Stage A's `…_none_r0` episodes later (only `noise_fork` metadata may differ).
3. Run `worldsim_probe --scene game_v19 --seeds 12000..12032 --days 120` with round 3's flags except `--sample-every 60`, for the timeline. **Factual timeline per seed** (`tools/revitalize/timeline_p14.py`): the eligible daily checks with members and food are in each run's `pass12.fission_checks` = `[tick, sid, members, capacity, food, g, q, s, h, draw, split, site_free]` (`probe/pass12.rs` ≈ L109–132; the schema-13 `pressure_checks` hold `[tick, sid, P, θ, h, g, q, s]` only); the **line** = max(1, floor(members × 2.4 × 4)) (adult need 0.05 × 1440 / 30 = 2.4, `food_economy.rs` ≈ L103; s in thousandths — check that the formula reproduces s on every check of 2 seeds); fissions and departures from the schema-13 block and `pass11` fission log. Probe day fields are zero-based. For the home settlement (lowest id), list its eligible days with s > 0 and with s = 0 and their food / line ratio, and its first departure. Define per seed:
   - **T_low** = 06:00 of the first eligible home day with s > 0 at least 3 days before the factual first departure; if none (the village departs before any shortfall), the first such day after that departure and at least 3 days before its next one; a seed that never departs uses its first eligible day with s > 0; none at all → no T_low (counted);
   - **T_high** = 06:00 of the eligible home day with s = 0 and the **largest food / line ratio** (ties: earliest) at least 3 days before the factual first departure, used only if that ratio ≥ 1.25 (the planner's check of round 3: the ratio peaks at 1.67, median 1.44 — a "2 × the line" rule would leave no seed); else no T_high;
   - **T_eve** = tick 29,400 (display day 21, 18:00; Pass 13's cast, the eve of the first winter), for comparison.
   A cast at a check tick is applied before that tick's systems run (`export.rs` ≈ L354–355), so it changes that same morning's s; say so in the map. Commit the timeline table (`docs/revitalize/probe/p14-0_timeline/`) with the share of seeds having each time. Push.

---

# 6. PART 1 — Stage A: the screen (DGX)

## 6.1 Recording
`worldsim_wm --scene game_v19 --seeds 12000..12032 --days 120 --every 60 --schedule-dir <p14/schedules_A> --include-factual --fork-tick 14280 --replicates 2 --workers <section 1> --out /home/hyunlord/work/worldsim-wm-data/p14/stageA` (fork at display day 11, 06:00 — before every cast; the first eligible check is day 12 under `min_age_days` 10). The recorder does **not** refuse a row before the fork tick (it would be applied to both replicates before they diverge) and silently never applies a row after the run's end, so the schedule writer and the scorer assert fork_tick ≤ tick < 120 × 1440 for every row, rows in non-decreasing tick order, and a matching `interventions.json` record for every row. Schedule files `s<seed>_<arm>.json`, each a list of rows `{"tick", "kind", "settlement": "home", "amount"}`; a seed lacking the arm's time has no file for that arm (counted).

## 6.2 Arms
**Group F (food; never dropped):**

| arm | kind | amount | when |
|---|---|---|---|
| `sp60L`, `sp120L`, `sp240L` | spoil_food | 60 / 120 / 240 | T_low |
| `sp240H` | spoil_food | 240 | T_high |
| `sp240E` | spoil_food | 240 | T_eve |
| `sp3x80L` | spoil_food | 80 at T_low, T_low + 2 d, T_low + 4 d | split dose |
| `gf60L`, `gf120L`, `gf240L` | gift_food | 60 / 120 / 240 | T_low |
| `gf240H` | gift_food | 240 | T_high |
| `gf240E` | gift_food | 240 | T_eve |
| `gf3x80L` | gift_food | 80 × 3 as above | split dose |

(If `spoil_food` applies less than requested because the store is smaller, the recorder's `applied` says so — report applied amounts.)

**Group W (wood and weather; drop first):** `rw120E` (rot_wood 120 at T_eve), `rw120L` (rot_wood 120 at T_low), `rain13W` (rain 13 h at Pass 9's C_cf rain tick 33,720 = day 24, 18:00), `cs2W` (cold_snap 2 nights at 35,160 = day 25, 18:00; it ignores the settlement). Rain and cold snap have no "right direction" for the split day: K5 alone judges them. (16 food + 4 W = 20 arms; at most 21 × 32 × 2 = 1,344 Stage A episodes, fewer where a seed lacks T_low or T_high.)

## 6.3 Scoring (`tools/revitalize/dose_map_p14.py`, with tests)
For every arm, per (seed, replicate) against the factual of the same replicate, and for **noise** the factual r0 vs r1:
- **Departure** = any group leaving any settlement, read from schema 6's `depart_tick` (set only for founded daughters: cancelled departures and departures whose daughter is not founded by the run's end are not seen — Pass 13 had 0 of 63 cancellations; cross-check the factual r0 against the probe's fission log).
- **Δday** = signed shift, in days, of the first departure after the cast tick. **Noise for an arm** = factual r0 vs r1 on that arm's seeds, measured after that arm's cast tick.
- **right-direction share** = share of pairs with Δday < 0 (spoil, rot) or > 0 (gift) over all pairs; a split in one run only counts in the direction it implies (the arm splits and the factual does not = earlier); ties and wrong signs count against.
- **mean |Δday|** and share |Δday| > 3 over pairs where both runs split after the cast (as `macro_sensitivity.py --j11`); the **effect-to-noise ratio** ENR = mean |Δday| (arm) / max(0.5 day, mean |Δday| (noise)); ENR = NA if the noise has fewer than 8 such pairs.
- settlement count at days 60/90/118 (share of pairs differing), alive at day 118 (mean signed and absolute Δ), deaths by cause to day 120 and within 35 days of the cast (mean signed Δ).
- Intervals: bootstrap over seeds (1,000×, `default_rng(0)`), 90 %.
Outputs `docs/revitalize/probe/p14-1_stageA/` (`dose_map.md`, `dose_map.json`, commands, schedule manifest with every file's sha256). Commit, push.

---

# 7. PART 2 — Pre-registration of Stage B
Before any Stage B recording, commit `docs/revitalize/probe/p14-2_prereg/PREREG.md` with:
- the **best spoil arm** and **best gift arm** of Stage A by right-direction share among the arms that are not `*E` and have ≥ 16 seeds (ties: larger ENR, then smaller dose), and the `sp240E` / `gf240E` arms as the eve-of-winter controls — four arms;
- the K2 thresholds below, unchanged;
- the seeds (14000–14063), R = 2, fork tick 14,280, and the timing rule of 5.3 (Part 0 item 3) applied to Stage B's own factual timeline.
Push before Stage B starts.

---

# 8. PART 3 — Stage B: confirmation on fresh worlds (DGX)
Probe the factual timeline of seeds 14000–14063 (Part 0 item 3), write the four arms' schedules, record as 6.1 into `p14/stageB`, score with `dose_map_p14.py`. Outputs `docs/revitalize/probe/p14-3_stageB/`. Commit, push.

---

# 9. PART 4 — The intervention map and the amendments
1. `docs/revitalize/INTERVENTION_MAP.md` (English): per kind, the dose–response at T_low (three doses), the timing contrast (T_low vs T_high vs T_eve at dose 240), single vs split dose, the effect on deaths of the wood and weather arms, Stage B's confirmed effect sizes with intervals, and a recommended **intervention schedule for dataset v2**: which kinds, doses and timing rule, how many arms per world, and the expected effect-to-noise ratio of each. State plainly which acts of the god do not change a world's shape beyond noise.
2. The 4.3 re-scoring (`evaluate_pass14.py` on Pass 13's outputs; no rerun): J6 relative per clause, J7b reported. Commit, push.

---

# 10. TARGETS (K)

| # | target |
|---|---|
| K1 | **identity**: the frozen binaries on the DGX reproduce Pass 13's frozen round-3 `game_v19` summary for seeds 0–31 (5.2); replicate 0 equals the unforked factual (2 seeds); `git diff 2ba871c0..HEAD -- rust/` empty |
| K2 | **an act that changes the world** (Stage B, pre-registered): the best spoil arm and the best gift arm each have right-direction share ≥ 0.75 **and** ENR ≥ 2.0 **and** the share of pairs whose day-118 settlement count differs ≥ the noise share + 0.15. Report each clause; K2 passes if both arms pass |
| K3 | **timing matters** (Stage A, reported with its interval): for both kinds, the right-direction share at T_low exceeds the share at T_eve for dose 240, compared on the seeds that have both times (paired) |
| K4 | **dose matters** (Stage A, reported): right-direction share and mean |Δday| non-decreasing over doses 60 → 120 → 240 at T_low for both kinds (state any inversion) |
| K5 | **deaths** (Stage A, reported): the wood and weather arms' death effects within 35 days and to day 120 against noise |
| K6 | the intervention map delivered (9.1), with a concrete dataset-v2 schedule |
| K7 | the evaluation amendments applied to Pass 13's outputs and stated beside its verdicts (4.3) |

---

# 11. PART 5–6 — Review, gate, report
1. **Independent review** (fresh `general-purpose` subagent, two rounds maximum): the timeline rule (Part 0 item 3) on 3 seeds by hand; schedule files match the arm table; casts happen after the fork tick; Δday uses departures after the cast; the noise baseline uses the same rule; the pre-registration commit precedes every Stage B recording (commit time vs the DGX files' mtimes); K verdicts follow the rules; no Rust change.
2. Gate (4.2); fast-forward `codex/revitalize-2026-10` if green.
3. `docs/revitalize/REPORT_PASS14.md` in Korean (section 12 format) and the same text in the final message. Update PR #510's description with a short Pass 14 section.

## 11.1 Out of scope
Any rule or Rust change (including the daughter-failure question and new mechanics, which go to a later `game_v20`); dataset v2 itself; learned models; UI and art.

---

# 12. 구현 완료 보고 (REPORT_PASS14.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
동결 바이너리와 동일성 확인, 사실 세계의 시간표(T_low·T_high·T_eve), 개입 갈래 설계, 1단계 선별, 사전 등록, 2단계 확인(새 세계), 효과 대 잡음 비율과 구간, 평가 기준 수정(J6 상대 비교, J7b 보고만)

### 기능 설명
연구 관점에서 무엇이 확인되었는가? 신의 손은 언제, 얼마나 밀어야 세계의 모양을 바꾸는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
K1~K7 표, 1단계 전체 갈래 표(방향 비율·평균 |Δ일|·효과 대 잡음 비율·마을 수 변화·인구·사망, 구간), 용량–반응, 시점 대비, 나눠 주기 대 한 번에, 2단계 확인 결과, 잡음 기준값, Pass 13 판정 옆에 적은 J6·J7b 재평가

### 환경
DGX 부하와 WM 실행과의 공존(작업 수, nice, 시간), 연결 끊김 여부

### 독립 리뷰
지적 사항과 처리 결과

### 자율 결정 목록
DECISIONS.md 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 멈춘 구간, DGX 작업 타임라인

### 확인된 제한사항

### 게이트 결과
(Python 단위 테스트 두 묶음 수치, rust/ 변경 없음 확인)

### 정직한 판단
신의 작은 개입이 세계의 모양을 바꾸는가, 아니면 큰 개입만 그런가? 데이터셋 v2의 개입 일정은 무엇이어야 하고, 그 일정에서 학습 모델이 배울 효과는 잡음보다 충분히 큰가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the DGX paths of the recordings, the paths of `INTERVENTION_MAP.md`, `dose_map.md` (A and B) and `PREREG.md`, and the time spent per Part.
