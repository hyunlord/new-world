# GOAL: WorldSim — M6a repair round R2 (Claude Code, new session): finish the M6a food chain so it can be published — fix the neighbours village that starves to death, settle spoilage by diagnosis, remove the performance regression, fix three broken gate instruments — then evaluate once on fresh seeds, run the full gate, publish, report.

Repository `hyunlord/new-world`. Executed by **Claude Code** in its own worktree (§0). Windowed Godot runs allowed. Target 10 hours, hard stop 14. Long builds, gates and probes run with `run_in_background`; wait on the notification, never poll with sleeps.

## Context (read these first, in this order)
The pass so far was done by Codex on branch `v8/m6a`. The latest state is pushed as **`origin/v8/m6a-review` = `ae340c11a35f546884180aeca28821d1412fa729`**: gate source `9b6644579`, plus a docs follow-up. It is unpublished, and `v8/main` = `v8/wip` = `5b0c88812374b87e7854cc9203f144ad0ccd9009` (BASE).
1. `docs/v8/m6a/M6A_GOAL.md` — the original goal (scope, bars §3, rules).
2. `docs/v8/m6a/M6A_REPORT.md` — results of the first run and of repair round R1, including the 48-step gate table and the fresh held-out table.
3. `docs/v8/m6a/M6A_R1_DIAGNOSIS.md` and `docs/v8/m6a/M6A_DECISIONS.md` — death classification, fixes P16/P17/P20/P24, decisions so far.
4. `docs/v8/DEPTH_ROADMAP.md` §3 and §7 (principles, process rules), `AGENTS.md`, `CLAUDE.md`, and the skills it lists (load `worldsim-testing`, `worldsim-pitfalls`, `systematic-debugging`, `verification-before-completion` before starting).
5. Raw evidence from earlier runs lives outside the repo under `/Users/rexxa/work/m6a-data/` (read only).

**Where R1 ended:**
- The standing soak has 0 hunger deaths and 0 KnowledgeLost.
- Fresh held-out 1328–1359 (72 days, steward on) passes 9 of 10 bars; 72-day survival is 30/32 against BASE 31/32. Keep all of this.
- The gate finished 42/48. These failed:

| Step | Failure | Kind |
|---|---|---|
| 25 bench | N200 script mean ×1.257 / p99 ×1.364; N200 p99 ×1.128; N1000 wolves ×1.101 | real regression |
| 30 neighbours | seed 1 made 0 trades. Both fixture worlds end with **0 survivors** (hunger 16/17, KnowledgeLost 54/52); P12's earlier "PASS" also had 0 survivors | real defect plus a fixture hole |
| 39 visibility | one person at 16/60 px, whose body bounds are y = −28 (clipped by the top of the screen) | instrument defect |
| 40 why input | 2 of 42 checks failed (empty-tile toast and its 2 s expiry); the cause was never recorded | unknown |
| 41 stall | `[Errno 9] Bad file descriptor` before capture started, so no frames were measured | harness bug |
| 45 evaluation | spoilage ≤ 30 % of harvest held in only 3/32 worlds (26 needed); calibration showed 0/8 at 30.9–51.3 % | diagnose (R2-2) |

## 0. RULES
- **Start check:** `git -C /Users/rexxa/work/new-world-pass4 fetch origin`, then confirm all of these. If any differs, stop and report.
  - `origin/v8/m6a-review` = `ae340c11a…`.
  - `origin/v8/main` = `5b0c8881…`.
  - No other process uses `/Users/rexxa/work/new-world-m6a-dev`. Check with `lsof -a -d cwd -c codex -c claude -c godot -c cargo` and list it.
- **Your worktree** (created by the owner before starting you, see the launch commands): `/Users/rexxa/work/new-world-m6a-r2` on branch `v8/m6a-r2` from `origin/v8/m6a-review`. Work only there.
  - Never edit, build or run in `/Users/rexxa/work/new-world-pass4`, `new-world-play2-dev`, `new-world-m6a-dev` (Codex's; leave it intact) or any other worktree.
  - Never kill or signal processes you did not start.
  - Never touch the local `.claude/settings.json`; never install hooks.
- **Start commit:** commit this brief first as `docs/v8/m6a/M6A_R2_GOAL.md` (from `~/Downloads/claude_goal_worldsim_m6a_r2.md`).
- **Git:**
  - Commits `v8-m6a(r2-N): …`, explicit paths only, never `git add -A`.
  - After each commit, `git push origin v8/m6a-r2:refs/heads/v8/m6a-review`. It must fast-forward from `ae340c11a`; never force.
  - Publish only after a full gate with status 0: `git push origin v8/m6a-r2:v8/wip`, then `git push origin v8/m6a-r2:v8/main`. Both must fast-forward from `5b0c8881`. If `v8/main` moved, rebase: docs conflicts keep both sides; code or data conflicts stop. Then re-run the full gate.
  - Never push `lead/main`, `main`, `codex/*`, `v8/art-*`.
- **A red gate means diagnose, fix, re-run the full gate from step 1.** Stop only for:
  - a code or data conflict,
  - an identity or determinism failure you cannot fix,
  - the condition at the end of R2-1,
  - the hard stop.
- **Baseline identity** stays byte-identical in every commit (`tools/v8/identity.sh`).
- **Measurement:**
  - Every timing repeat starts at 1-min load < 8; record 1/5/15.
  - Interleave BASE and candidate in one session.
  - Never run two measuring jobs at once.
  - The owner will not play the game while you measure.
- **Issue IDs:** new ledger rows use temporary IDs (M6A-R2-1, …) until publish, then the next free ISS numbers, with GitHub issues created then.
- **Records:** record each decision in `M6A_DECISIONS.md` citing "R2 brief 2026-10-30". Never ask the owner; questions go in the report.

## 1. The work (R2 decisions, fixed before any result)
**R2-1 Neighbours fixture (the most serious defect).**
- (a) Extend the fixture behind gate step 30 (find it in `tools/v8/gate.sh` and the scripts it calls):
  - survivors at day 48 ≥ founders − 2 in each world,
  - 0 KnowledgeLost,
  - the existing trade and migrant checks stay.
  - Run it on BASE `5b0c8881` too and record BASE survival. If BASE also dies there, the survival threshold becomes BASE's value, recorded.
- (b) Diagnose every death as R1 §2a did: stores by kind at death, reachable ripe plants, open jobs and holders, the last MindChoice and blocked candidates, the steward target, migrants joined, and food traded away.
- (c) Fix the root causes, each with its own commit and a regression test. Then re-check seed 1's trades.
- **Stop condition:** if the village still dies after fixes while BASE survives, stop and report the numbers instead of publishing.

**R2-2 Spoilage.**
- First diagnose on calibration seeds 0–7. For each spoiled unit, record:
  - whether the settlement's edible store was above its need horizon at that moment (Part 1 arithmetic in `M6A_DESIGN.md`),
  - whether a roofed store or pot with room existed.
- Owner rule, fixed now: if ≥ 80 % of spoiled units spoiled while stores were above need, the "spoiled ≤ 30 % of harvest" bar is mis-specified (it punishes surplus). It is then replaced by two bars:
  1. "no day on which a person is hungry (hunger ≥ seek threshold) while food of their settlement spoils" in ≥ 28/32 worlds;
  2. "stored food placed in a roofed store or pot when room exists" ≥ 90 % of stored units.
- Otherwise it is a defect: fix it and keep the 30 % bar.
- Record which branch applied, with the numbers.

**R2-3 Bench (step 25).**
- Profile the N200 script scene: top 10 functions before and after, with a sampling profiler on `worldsim_perf` release (samply, or macOS `sample`).
- Fix the regression.
- `--justified` is allowed only if the remaining cost is the new food chain's necessary work, the CHANGELOG states it with numbers, and every scene stays ≤ ×1.15 mean and p99.
- C3 ceilings must pass.

**R2-4 Instruments.** Fix the instrument, prove the fix on BASE, record it.
- **Step 39:**
  - Judge only people whose body bounds are fully on screen.
  - Add a test that a fully visible, low-contrast person still fails.
  - Re-run on BASE and on the candidate.
- **Step 41:**
  - Fix the file-descriptor hand-off in the gate → nested capture lane, and re-run.
  - Waiver: the stall bar may fail only if all other steps pass, each zoom has ≤ 2 frames > 50 ms, none > 60 ms, and the capture is complete. Record whether it was used.
- **Step 40:**
  - Instrument the check so it records which part failed (presence, visibility, text, timing).
  - Fix the product or the harness accordingly, with evidence.

**R2-5 Evaluation protocol.**
- Calibration seeds 0–7 have no round limit (log every round).
- Freeze the policy and data, then evaluate once on **fresh held-out seeds 1360–1391**, 72 days, steward on, no orders.
- Use the goal §3 bars plus R2-1 and R2-2, with a BASE column on every row.
- 1296–1327 and 1328–1359 are retired.
- No tuning after the freeze.

## 2. Gate, publish, report
- Run the full gate from step 1 on the final commit (it builds in your worktree's own `rust/target`; expect a long first build), then publish (§0).
- After publishing:
  - close #671, #672, #673 if fixed, with public commit links; #670 (plant art) stays open;
  - edit #649's document line to link `docs/v8/DEPTH_ROADMAP.md` at the published SHA;
  - update `docs/ROADMAP.md` (M6a done) and `docs/HISTORY.md`;
  - remove your worktree only if `git worktree remove` succeeds without `--force`.
- Append a section "R2" to `docs/v8/m6a/M6A_REPORT.md` (Korean). The final message, in Korean, uses this template:
```
## 구현 완료 보고 — M6a R2
### 이웃 세계 사망 (원인 표, 고친 것, BASE 대비 생존)
### 썩음 진단 (어느 가지를 적용했나, 수치)
### 성능 (프로파일 상위 10, 전후 장면별 비율)
### 계기 수리 (39·40·41, BASE에서의 증명)
### held-out 1360–1391 (후보 / BASE 표)
### Gate 결과 (48단계 표)
### 닫은 GitHub 이슈 / 새로 연 이슈
### 변경된 파일 목록 (커밋별)
### 확인된 제한사항
### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)
git ls-remote 결과
```

## 3. 인게임 확인사항 (오너용)
1. 여름에 덤불에 열매가 달리고 사람들이 따서 비축 구역으로 나른다. 겨울에는 저장한 것을 먹는다.
2. 화덕에서 굽고 끓인 음식을 먹고, 정보창에 마지막 식사가 보인다.
3. 이주민이 들어와도 마을이 굶지 않는다.
4. 3년을 지나도 굶어 죽는 사람이 거의 없고, 있으면 연대기에서 원인을 따라갈 수 있다.
