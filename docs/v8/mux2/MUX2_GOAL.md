# GOAL: WorldSim — M-UX2 (Codex): the owner's second playtest — people visible at every zoom, no giant empty "왜?" panel on a click, no periodic stalls, a steward that defends the village instead of watching it die, firewood that reaches the stockpile, readable Korean text — then publish, record, and hand the owner a build to play.

Repository `hyunlord/new-world`, local clone `/Users/rexxa/work/new-world-pass4`. Base: **`v8/main` = `0207b49760a31a71968b441298fb52b073791cb4`** (M-UX + REVIEW). Executed by **Codex (GPT Astra)**, unattended. Windowed Godot runs allowed. Target 10 hours, hard stop 14.

## Why
**Owner playtest, 2026-10-26 03:06 (screenshot at ½×, steward on, day 1 evening):** "좀 더 볼만하게 축소를 해 봤는데, 바로 인물들이 아무것도 안 보여." "트랙패드는 된다." "왼쪽에 검정 박스는 뭔데 저렇게 크게 떠 있냐." The planner's reading of the screenshot:
- At ½× people are drawn as 10-px top-colour discs (LOD), nearly invisible on the dark ground. At ½× a route I figure would be ~28 px tall — perfectly visible. The LOD threshold was set too early (planner's mistake in M-ART2).
- The black box is the "왜? — 인과 기록" panel: a plain click on empty ground opens it; since M-UX the end of a left-drag pan counts as that click. It fills a quarter of the screen to say "이 타일의 인과 기록이 없습니다".
- Also visible: the top button row is crowded; alerts at day 1 ("먹을 것 3일 미만", "할 일 없는 어른 2").

**Owner decisions, same message ("너 추천대로 가자"):** (1) the steward gets **defence, on by default**: when wolves come it drafts armed adults and undrafts them after; the player can switch it off, and a player's own draft always wins; flee targets are fixed. (2) The V7 baseline separation waits until after M6 prep.

**From the M-UX report (`docs/v8/mux/STAGE_A_REPORT.md`, `MUX_REPORT.md`):** A8 held-out — day48 survival 22/32, cold-snap firewood 21/32, beds by day 10 30/32. Every death was combat (wolves), drafted = 0 in every beast event (the planner's "steward never drafts" rule), fleeing people died outdoors 2–36 tiles from a reachable roofed room. One world had 90 firewood on the ground and 0 in stockpiles/settlement. ISS030: repeated 73–99 ms stalls in normal debug play, up to ~650 ms. ISS274/275: Korean words broken mid-word and alerts truncated.

# 0. RULES
- **Isolation:** your own worktree `/Users/rexxa/work/new-world-mux2-dev` on branch `v8/mux2` from `origin/v8/main` (`git -C /Users/rexxa/work/new-world-pass4 worktree add -b v8/mux2 /Users/rexxa/work/new-world-mux2-dev origin/v8/main`; if the path exists and is not a registered worktree of this clone, use `…-2`). Never edit, build or run in `/Users/rexxa/work/new-world-pass4` or `/Users/rexxa/work/new-world-play2-dev`; never kill or signal other processes. Read this brief from `~/Downloads/codex_goal_worldsim_mux2.md` and commit it as `docs/v8/mux2/MUX2_GOAL.md` first.
- **Git:** commits `v8-mux2(pN): …`; publish only after a gate with status 0: `git push origin v8/mux2:v8/wip` then `git push origin v8/mux2:v8/main` (fast-forwards; if refused, stop and report; never force). Never push to `lead/main`, `main`, `codex/*`, `v8/art-*`; explicit paths; no art merges.
- AGENTS.md, ADR 0016, ADR 0033 binding; the M-UX safety net applies: behaviour-preserving commits keep baseline identity and the steward-off colony fixtures byte-identical; behaviour-changing commits are separate, predicted, measured, recorded.
- **Records as you go:** `docs/HISTORY.md`, `CHANGELOG.md`, `docs/ISSUES.md` (update rows you fix, add rows you find), and on GitHub close the issues **this project's registry created (#513–#619)** that you fix, each with a comment linking the commit and the registry row; never edit issues you did not create or that the registry did not create.
- DECISIONS in `docs/v8/mux2/MUX2_DECISIONS.md`. Never ask the owner; questions go in the report.
- Drop order: §6 HUD compaction → §5 Korean wrapping beyond the alerts → firewood (§4) diagnostics depth. **Never drop:** §1 LOD, §2 why panel, §3 steward defence + flee, §3 evaluation, §5 alert wrapping, ISS030 stall diagnosis with a fix or a measured mitigation, the gate, the report.

# 1. People visible at every zoom (renderer)
- **½×: draw the route I figures** (scaled 0.5 with the same anchor/shadow rules), not markers. **Markers only at ¼×**, and make them readable: filled disc 14 px with a 2-px light ink ring (value ≥ 0.85) plus the household-colour outer ring, drawn above everything except UI; a 1-px dark outline beyond it for dark ground.
- Selected and drafted people keep their outlines at every zoom; drafted markers are red-ringed at ¼×.
- Params in `render_params.json` (`lod.marker_zoom_max`, sizes, ring values); F12 reloads.
- **Visibility test (headless GPU or windowed):** for seed 0 day 1 and day 20 scenes at ½× and ¼×, every on-screen person covers ≥ 60 px (½×) / ≥ 140 px (¼×) whose luminance differs from the local ground by ≥ 0.20; fail otherwise. Also night.
- FPS stays within M-UX's budgets (1,000 people at ½× ≥ 60; use MultiMesh or atlas batching if sprites at ½× cost too much).

# 2. The "왜?" panel
- **No automatic opening on map clicks.** Open it only by: the "왜?" button in the inspector (person/room/item), a "왜?" tool button in the top row (then click a tile), or ⌥-click on a tile. A drag-end is never a click.
- **Compact:** width ≤ 420 px, height fits content (max 50 % of the screen, scroll beyond), anchored next to what it explains; closes on Esc, on clicking outside, and on a close button.
- **Empty result → no panel:** a small toast "이 칸에는 기록된 원인이 없어요" for 2 s.
- UI tests: drag-pan never opens it; ⌥-click opens it; empty → toast; Esc closes.

# 3. Steward defence and flight (behaviour-changing, ADR 0035 amendment)
- **Defence rule** in `steward.ron` (scope "방어", **on by default** with the steward; its own toggle in the steward menu and settings): when a wolf pack is spawned and heading in (the existing approach signal), draft the `min(4, armed adults)` best fighters (weapon first: spear > axe; then health; then distance to the camp), arm the unarmed among them from stock (the existing auto-arm), attack the nearest wolf; undraft them when no wolf is within 24 tiles for 60 ticks or the pack retreats; never draft children, the sick below 50 hp, or anyone the player has drafted or undrafted manually in the last day (the player's choice wins). Chronicle "마을이 스스로: 늑대를 막으려고 n명을 불렀어요" and a toast.
- **Flee fix:** an undrafted person who flees picks the nearest **reachable roofed room that it reaches before the nearest wolf** (compare path lengths with the wolf's speed), else the hearth if the defenders are there, else away; re-evaluates every 30 ticks. Wolves still never attack inside roofed rooms (M5 rule).
- **Firewood to the stockpile:** diagnose the M-UX world with 90 ground firewood and 0 stocked (reproduce from the saved evidence or by seed); fix the cause (likely haul priority/zone acceptance/reservation); add a test that firewood split or dropped outside reaches a stockpile within one day when a zone has room.
- **Evaluation (held-out seeds 1200–1231, 48 days, steward on, no player orders):** day-1 stockpile 32/32; walled house with beds for all by day 10 ≥ 30/32; 0 cold deaths in the first winter ≥ 28/32; firewood never zero during a cold snap ≥ 24/32; **village alive at day 48 ≥ 30/32**; wolf deaths per world ≤ 1 in ≥ 28/32; ledger exact 32/32. Also report the same seeds with defence off (expect A8's numbers). Calibration on seeds 0–7 only (≤ 4 rounds, logged); policy frozen before held-out.

# 4. Stalls (ISS030)
- Profile normal debug play (the launcher's build) for 5 minutes at 1× and ½× on seed 0 with the steward on: record every frame > 33 ms with a Godot profiler capture or `Performance` monitors and a script-time breakdown (minimap rebuild, chunk bakes, alert evaluation at 2 Hz, steward/UI fact reads, GC).
- Fix the causes found (spread work across frames, cache, move bakes to a worker thread if safe for Godot's Image API, or lower the rebuild cadence). **Bar:** no frame > 50 ms after the first 10 s of play, and ≤ 3 frames > 33 ms per minute, at 1× and ½×; report before/after tables.

# 5. Korean text (ISS274/275)
Alerts, lesson cards, tooltips, the help overlay and panels: no mid-word breaks (use word-boundary autowrap for Korean, `TextServer` break flags, or explicit spacing rules), no truncated alert lines (widen or wrap with the count kept visible); a screenshot test at UI scale 1.0 and 1.25 with a list of every text node checked.

# 6. HUD compaction (droppable)
The top button row (마을 스스로 · 마을 목표 · 설정 · ? · 작업 우선순위 · 기술 지도 · 이웃과 방문객) becomes icon buttons with tooltips and keys; the steward toggle stays a labelled pill. Nothing overlaps the alerts panel at 1.0/1.25 scale.

# 7. Gate, publish, report
- Gate (all current steps + the visibility test + the why-panel and steward-defence tests + the stall bar as a windowed check recorded in the report) → publish → remove your worktree.
- `docs/v8/mux2/MUX2_REPORT.md` (Korean) and the final message:
```
## 구현 완료 보고 — M-UX2
### 오너 플레이 지적과 처리 (½× 사람, 왜? 패널, 트랙패드)
### 사람 보이기 (LOD 규칙, 가시성 시험 수치, FPS)
### 왜? 패널 (여는 법, 크기, 빈 결과)
### 마을 방어와 도망 (규칙, 플레이어 우선, held-out 표 — 방어 켬/끔 비교)
### 장작 운반 (원인, 고친 것, 시험)
### 끊김 (원인별 측정, 고친 것, 전후 표)
### 한국어 줄바꿈과 위 막대
### 닫은 GitHub 이슈 / 새로 연 이슈
### 변경된 파일 목록 (커밋별)
### 확인된 제한사항
### 오너 결정 필요
### Gate 결과
### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)
git ls-remote 결과
```

# 8. 인게임 확인사항 (오너용)
1. ½×로 줄여도 사람 그림이 보인다. ¼×에서는 밝은 테두리의 동그라미로 확실히 보인다.
2. 땅을 클릭하거나 끌어도 검은 "왜?" 상자가 뜨지 않는다. 사람 정보창의 "왜?" 버튼이나 ⌥-클릭으로만 열리고, 작고, Esc로 닫힌다.
3. 늑대가 오면 "마을이 스스로: 늑대를 막으려고 4명을 불렀어요"가 뜨고 창 든 사람들이 막는다. 나머지는 가까운 방으로 피한다. 내가 직접 소집하면 내 선택이 우선이다.
4. 화면 이동·줌 중에 뚝뚝 끊기는 느낌이 없다.
5. 경고와 안내 문장이 단어 중간에서 잘리지 않는다.
