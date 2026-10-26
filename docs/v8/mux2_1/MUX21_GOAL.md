# GOAL: WorldSim — M-UX2.1 (Codex): find and fix the ¼× frame-rate regression — 200 people at ¼× now run at 35 FPS where M-ART2.1 ran 1,000 people at ¼× at 137.5 FPS — without losing M-UX2's ¼× marker visibility; publish; report.

Repository `hyunlord/new-world`, local clone `/Users/rexxa/work/new-world-pass4`. Base: **`v8/main` = `9855bd082ad88a4c49296410f206f148d98c4df9`** (M-UX2 published). Executed by **Codex (GPT Astra)**, unattended, windowed Godot runs allowed. Target 4 hours, hard stop 6.

## Why
`docs/v8/mux2/MUX2_OWNER_MEASUREMENTS.json` (same method as M-UX2's FPS: `mart2_fps.gd`, release bridge, 30 s warm-up + ≥ 60 s, 1920×1080, vsync off): N = 200 → ½× 119.3, 1× 225.8, **¼× 35.1** (valid; 33.5 on the invalid repeat). The roadmap row for M-ART2.1 records 1,000 people at ¼× **137.5 FPS** and ½× 149.1 (`docs/ROADMAP.md`, `docs/perf/`). ¼× is the zoom the owner uses to watch the whole village, and it is now the slowest zoom with 5× fewer people. Suspects (unverified): M-UX2's ¼× markers (14-px disc + light ring + household ring + dark outline, "drawn above everything except UI" — possibly one node/draw call per marker per frame), the ¼× canopy bake or chunk rebakes (controlled rebake max 7.239 ms > 4 ms, natural rebake NOT_MEASURED), the minimap, or a per-frame cost that scales with visible chunks.

# 0. RULES
- Own worktree `/Users/rexxa/work/new-world-q4-dev`, branch `v8/mux2-1` from BASE (`git -C /Users/rexxa/work/new-world-pass4 worktree add -b v8/mux2-1 /Users/rexxa/work/new-world-q4-dev 9855bd082ad88a4c49296410f206f148d98c4df9`; if the path exists and is not a registered worktree, use `…-2`). Never edit/build/run in `/Users/rexxa/work/new-world-pass4`, `/Users/rexxa/work/new-world-play2-dev`, `/Users/rexxa/work/new-world-mux2-dev` or another session's worktree; never kill or signal other processes. Commit this brief first as `docs/v8/mux2_1/MUX21_GOAL.md` (read it from `~/Downloads/codex_goal_worldsim_mux2_1.md`).
- **Renderer/UI only:** no change under `rust/crates/sim-core`, `sim-systems`, `sim-engine` (another session may be working there). Baseline identity and all colony fixtures stay byte-identical.
- Commits `v8-mux21(pN): …`, explicit paths. Records: `docs/HISTORY.md`, `CHANGELOG.md`, `docs/ISSUES.md`; a GitHub issue for the regression (registry row + issue), closed with the public commit link when fixed. DECISIONS in `docs/v8/mux2_1/MUX21_DECISIONS.md`. Never ask the owner.
- **Publish:** after a gate with status 0, `git push origin v8/mux2-1:v8/wip` then `git push origin v8/mux2-1:v8/main`. If refused because `v8/main` advanced (another pass published), rebase onto `origin/v8/main` only if the rebase has no conflicts, re-run the full gate, then push; on a conflict or a red gate, stop and report. Never force.

# 1. Find it (measure before changing anything)
1. Same method as `MUX2_OWNER_MEASUREMENTS.json` (record load before/after; a run with 1-min load > 8 is invalid and repeated). Measure ¼× at N = 10, 40, 200, 1,000 on three builds in one interleaved session: M-ART2.1 `3a51636d`, M-UX `0207b497`, BASE. If the harness differs between builds, use BASE's harness on all three and say so. This tells whether the regression came in M-UX or M-UX2.
2. Bisect the guilty range (M-UX2 commits p1–p16 or M-UX's) with the ¼× N = 200 measurement until one commit is found.
3. Godot profiler (or `Performance` monitors + timers) at ¼× N = 200 on BASE: frame time split into script process, draw calls/objects drawn, marker drawing, canopy/chunk bakes, minimap, alerts/UI facts. Record the top 5 costs.

# 2. Fix
Fix the causes found: e.g. draw all markers in one `MultiMeshInstance2D` (or one `_draw` with a cached texture per ring style), cache static ring textures, bake only dirty chunks, throttle the minimap. Keep the M-UX2 look: ¼× marker disc 14 px, light ring value ≥ 0.85, household outer ring, 1-px dark outline, selected/drafted rings.

**Bars (release, same method):**
- ¼× N = 200 ≥ 120 FPS and N = 1,000 ≥ 100 FPS (reference: M-ART2.1 1,000 at ¼× = 137.5 with simpler markers; 100 leaves ~35 % more frame time for the richer markers);
- ¼× N = 10 and 40 ≥ 144 FPS;
- ½× N = 100 ≥ 150 (now 164.9), ½× N = 200 ≥ 105 (now 119.3), 1× N = 200 ≥ 200 (now 225.8) — no zoom gets more than ~10 % slower;
- M-UX2's GPU visibility test still passes (¼× ≥ 140 px of contrast per person, day and night) and the selection/draft ring checks still pass;
- natural ¼× rebake measured (no longer NOT_MEASURED); report its max ms; aim ≤ 4 ms per frame, report if not met.

# 3. Gate, publish, report
Full gate (all current steps + the FPS table as a recorded windowed check) → publish (§0) → remove the worktree if `git worktree remove` succeeds without `--force`, else leave it and list why. `docs/v8/mux2_1/MUX21_REPORT.md` (Korean) and the final message:
```
## 구현 완료 보고 — M-UX2.1
### 원인 (빌드 3개 비교, 이분 탐색으로 찾은 커밋, 프로파일 상위 5개)
### 고친 것
### FPS 전후 표 (인구 10/40/200/1,000 × ¼×·½×·1×, 막대 판정)
### 가시성 재확인
### 닫은 GitHub 이슈 / 새로 연 이슈
### 변경된 파일 목록 (커밋별)
### 확인된 제한사항
### Gate 결과
### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)
git ls-remote 결과
```

# 4. 인게임 확인사항 (오너용)
1. ¼×로 줄여도 화면이 부드럽게 움직인다(트랙패드로 끌 때 버벅임 없음).
2. ¼×에서 사람은 여전히 밝은 테두리 동그라미로 잘 보이고, 선택한 사람과 소집한 사람은 테두리로 구분된다.
3. ½×·1×도 전보다 느려지지 않는다.
