# GOAL: WorldSim — M-UX + REVIEW (Codex): make the game playable for its owner on a Mac (mouse/trackpad camera, on-screen speed and zoom, minimap, alerts, a learning helper, goals, and a "마을 스스로" steward that runs the basics by default), then review the whole code base and simulation for structure and extensibility, refactor what violates ADR 0033, delete what is dead, and write every piece of history and every issue down in the repository and on GitHub.

Repository `hyunlord/new-world`, local clone `/Users/rexxa/work/new-world-pass4`. Base: **`v8/main` = `67a6232ab41a4bf5051b11303bcac97543b111f0`** (V8-PLAY2). Executed by **Codex (GPT Astra)**, unattended. Windowed Godot runs allowed. Three stages, each ending with a green gate and a fast-forward of `v8/main`, so the owner gets each stage as soon as it is done. Targets: Stage A 10 h, Stage B 14 h, Stage C 4 h; hard stop 32 h total (then finish the current step, gate, publish what is green, report).

## Why
**The owner's first real playtest (2026-10-25):** "키보드로밖에 화면을 못 움직이니까 답답하다. 축소나 속도 바꾸는 건 어떻게 하는지 모르겠다." and "뭘 해야 하는지 모르겠다. 베기 같은 모든 걸 내가 지시해야 하나?" The camera moves only by keyboard, wheel and middle-button drag (no middle button on a Mac trackpad); speed (1–4, Space) and zoom (wheel, `-`/`=`) have no on-screen controls; and nothing tells a player what the colony needs — RimWorld solves this with an alert list, a learning helper and storyteller pressure, and our game has none of them. Without orders the people only forage and rest. The owner chose (planner's recommendation): a steward mode **on by default** that issues the basic orders itself, which the player can switch off.

**The owner's standing directives (2026-10-24/25):** every code and design structure must be extensible (ADR 0033); delete code and structures that are no longer needed; keep the code structure clean; record the history of the work and every issue on GitHub.

**Known state of the code** (planner survey at `67a6232a`): Rust ≈ 70 k lines outside tests (sim-core 23 k, sim-systems 36 k, sim-bridge 10 k, sim-engine 1.6 k) and sim-test 88 k; GDScript 51 files, 16 k lines. Hot spots: `sim-systems/src/runtime/decision/system.rs` 3,494 lines (V7 decision cascade; 80 % of the tick in M2's profile), `runtime/settlement/settlement_system.rs` 1,571, `sim-bridge/src/ffi/snapshot.rs` 1,481. Content enums that ADR 0033 lists as debt (M5.5 D29, M-ART2 D16): `ItemKind`, `BlueprintKind`, `Designation`, `RecipeKind`, `SkillKind`, `WorkType`, `Top`, `AnimalKind`, `PressureKind`, `RoomRole`, `ChronicleKind`, `BuildingKind`, `ResourceKind`, `TerrainType`, `JobKind` — some are content, some are mechanics; the review decides which. Other recorded debts: trees have no species (every log is `oak`; the renderer hashes a species per tile), the top-bar icons still use the old atlas (`art_sprite_library.gd`, `assets/art/atlas/*`), `economy_text.gd` item name keys, order-bar blueprint names in GDScript, the extensibility grep's blind spots. Owner decisions waiting for implementation (M5.5 → M-EXT): population cap also at acceptance; trader goods include items of techniques with ≤ 2 practitioners plus rare next-era artifacts; meat goes beside the hearth when no stockpile cell accepts it; F4/F9 bars rewritten with their arithmetic. Renderer notes after M-ART2.1: water reads milky (should be bluer and darker than land, keeping the contrast rule), river staircase needs a larger shore radius (≈ 32 px), the winter snow overlay reads as camouflage (larger softer noise, alpha ≤ 0.45, drifts at tree bases and walls).

---

# 0. RULES
- **Isolation.** Work only in your own worktree `/Users/rexxa/work/new-world-mux-dev` on branch `v8/mux` created from `origin/v8/main` (`git -C /Users/rexxa/work/new-world-pass4 worktree add -b v8/mux /Users/rexxa/work/new-world-mux-dev origin/v8/main`; if the path exists and is not a registered worktree of this clone, use `…-mux-dev-2`). Never edit, build or run anything in the `/Users/rexxa/work/new-world-pass4` working tree or in `/Users/rexxa/work/new-world-play2-dev` (the owner's launcher). Never kill or signal other processes. Read this brief from `~/Downloads/codex_goal_worldsim_mux_review.md` and commit it as `docs/v8/mux/MUX_GOAL.md` (your first commit).
- **Git.** Commits `v8-mux(A|B|C pN): …`; publish only after a stage's gate is status 0: `git push origin v8/mux:v8/wip` then `git push origin v8/mux:v8/main` — both fast-forwards; if refused, stop and report; never force-push; never push to `lead/main`, `main`, `codex/*`, `v8/art-*`; stage explicit paths only; no merge of art branches.
- **AGENTS.md, ADR 0016 (simulation contract C1–C5) and ADR 0033 (extensibility contract) are binding.** Read the skills in `.claude/skills/` first (`worldsim-architecture`, `-godot`, `-testing`, `-pitfalls`, `-docs-upkeep`).
- **Behaviour safety net.** A change is either *behaviour-preserving* — baseline identity byte-identical **and** the colony fixtures (soak, story, neighbours, knowledge F10, the M5.5 evaluation script on seeds 0–7 × 48 days with the steward **off**) produce byte-identical summaries before/after — or *behaviour-changing*, in which case it is its own commit, its expected effect is written before the change, the fixtures are re-captured with the diff explained, and a DECISIONS entry names it. Never mix the two in one commit.
- **Never ask the owner** → `docs/v8/mux/MUX_DECISIONS.md` (D-numbered). Questions for the owner go in the stage report.
- **Records (owner directive):** every stage appends to `docs/HISTORY.md`, `CHANGELOG.md`, and the issue registry `docs/ISSUES.md` (§C2) as it goes — not only at the end.
- Drop order if a stage runs long: A9 settings panel → A6 goals panel → B5 tree species → B4 M5.5 decisions (defer to the next pass, recorded). **Never drop:** A1–A5 and A7 (camera, on-screen controls, minimap, help, alerts, learning helper core), A8 steward, the Stage B review document and the behaviour-preserving content-enum refactor of at least items, blueprints/construction styles and recipes, Stage C issue registry, history, the reports.

# 1. STAGE A — Playable on a Mac (M-UX)

## A1 Camera
- Trackpad: two-finger scroll pans (`InputEventPanGesture`), pinch zooms (`InputEventMagnifyGesture`) toward the cursor; a mouse wheel zooms toward the cursor; **left-drag on the map pans when no order tool is active** (a click without movement still selects); right-drag pans always (right-click alone still cancels a tool); keyboard WASD/arrows stay; optional edge-pan (off by default, in settings).
- Zoom ladder unchanged (¼×, ½×, 1×, 2×) but **smooth** (lerp 120 ms) and anchored at the cursor; bounds unchanged.
- Window: at launch, size to fit the screen's usable area (keep 16:9, cap 1920×1080) instead of a fixed 1920×1080 that overflows MacBook screens; ⌃⌘F toggles fullscreen.

## A2 On-screen controls (bottom-right, always visible)
Speed buttons ⏸ ▶ ▶▶ ▶▶▶ ▶▶▶▶ (highlight the active one; Space and 1–4 unchanged), zoom ＋/− with the level (replaces the plain "확대 1×" text), a "캠프로" button (H/Home), all with tooltips naming the key.

## A3 Minimap
Bottom-right above the controls, 200×200: terrain classes in the ¼× map colours, rooms/walls, people as dots in their household colour, animals (wolves red), the camera rectangle; click or drag to move the camera; collapsible; rebuilt on chunk changes at low rate (≤ 2 Hz), budget ≤ 0.5 ms/frame.

## A4 Help and tooltips
A "?" button (top-right) and the `/` key open a help overlay with the full key/mouse table (generated from one data file `assets/ui/controls.json`, which README and the overlay both use — single source); every order-bar button, panel button and HUD item gets a tooltip with its key.

## A5 Alerts (right edge, RimWorld-style)
- Data-driven: `assets/ui/alerts.json` defines each alert (id, severity, condition id, text keys, "how to fix" key, jump target kind); conditions are computed from bridge reads by a GDScript `alert_evaluator.gd` (no simulation change), at most 2 Hz.
- First set (≥ 15): food for < 3 days; firewood short for the coming winter (needs × days); people sleeping outside; no stockpile zone; storehouse full; idle adults; unreachable orders; designations waiting for a practitioner (knowledge); technique at risk (1 practitioner); wolves approaching; migrants waiting; trader here; someone sick; someone in a breakdown; someone injured (< 50 hp); meat spoiling outside a stockpile; no bed for a newborn's household.
- Each alert: icon + count; hover → explanation and "how to fix" (with the key/button); click → camera jumps to (and selects) the cause; repeated clicks cycle through instances; dismiss per alert until it recurs.

## A6 Goals panel (droppable)
`assets/ui/goals.json`: first-year goals (stockpile, a walled house with beds for everyone, firewood for winter, survive the first winter with everyone alive, teach one technique), era goals (the era progress bar, the next marker techniques). A small panel (G key and a HUD button) with ticks.

## A7 Learning helper
`assets/ui/lessons.json`: ordered lessons with a trigger condition, a completion condition and a highlight target (UI element or map area): camera (move/zoom/minimap) → speed → select a person (inspector) → stockpile (F2) → chop (F1) → build walls/door/bedrolls (F3) → priorities (P) → workshops and bills → rooms (R) → tech map and teaching (T, K) → alerts → drafting (D) when wolves first come → neighbours (N) when migrants first come. A small card (bottom-left) with "다음/건너뛰기/끄기"; progress stored in `user://settings.cfg`, not in saves; the helper and the steward cooperate (lessons that the steward already did are explained as "마을이 스스로 했어요 — 직접 하려면…").

## A8 Steward — "마을 스스로" (simulation side, ADR 0035)
- A colony system `StewardSystem` (runs every 360 ticks, after jobs assignment) that, when enabled, **issues ordinary player commands** through the command path with `source: Steward` (recorded in the command log, so replay stays byte-identical; the probe and every evaluation script run with the steward **off** unless they say otherwise). Toggled by a recorded command `SetSteward { on, scope }`; the game's new-game default is **on**; saves keep the setting.
- Rules are data (`sim-core/data/colony/steward.ron`), each with a condition, an action and a cooldown, evaluated in a fixed order:
  1. no stockpile zone → a zone of `6 + adults/2` cells square nearest the hearth on valid ground;
  2. logs + firewood below `need_days × daily_need` → chop marks on the nearest N trees (N by deficit; never in the camp radius);
  3. people sleeping outside and no bed for them → place a log-walled room template (size by beds needed, door facing the hearth) with bedrolls, on the nearest valid ground (the existing place-preview validation); one building at a time;
  4. maintain bills: firewood for winter, `Maintain 2` stone axes, `Maintain 4` flint blades when a knapping spot exists; build a knapping spot / wood bench if none and the knowledge exists;
  5. food below 3 days and `tracking` known → a hunt zone over the nearest deer forest;
  6. a technique at risk → a teach order from its practitioner to the best candidate (the M5 teach suggestion);
  7. migrants: **never** decides — the player always answers (alert); wolves: **never** drafts — alert only.
- The player always wins: steward actions never override or cancel player orders; player-cancelled steward orders are not reissued for 2 days; every steward action posts a chronicle line "마을이 스스로: …" and a small toast; the HUD shows "마을 스스로 켜짐" with a toggle (and a scope menu: 전부 / 집만 / 먹을 것만 / 끔).
- **Evaluation (held-out seeds 1000–1031, no player orders, steward on, 48 days):** stockpile by day 1, a walled house with beds for all by day 10, 0 cold deaths in the first winter in ≥ 28/32, firewood never zero during a cold snap in ≥ 24/32, colony alive at day 48 in 32/32; with the steward off the same seeds reproduce the M5.5 behaviour (forage and rest). Calibration on seeds 0–7 only (≤ 6 rounds, logged).

## A9 Settings panel (droppable)
Edge-pan, invert pan, UI scale (1.0/1.25/1.5), helper on/off, steward default; `user://settings.cfg`.

## A10 Renderer notes
Water bluer and darker than land (keep the D19 contrast rule, flip its sign), shore radius 32 px, snow overlay softer (feature ≥ 512 px, alpha ≤ 0.45, drift masks at tree bases and walls) — params plus the code they need; re-capture the compare sheet.

## A11 Stage A verification and publish
- `mux_ui_test.gd` (windowed, ≥ 40 checks: gestures and drags pan/zoom, buttons, minimap click, help overlay, tooltips, each alert's jump, lessons advance, steward toggle and its chronicle line) and Rust tests for the steward (rules fire once per condition, cooldowns, never overrides player orders, replay byte-identical with steward commands in the log, save/load keeps the setting).
- Gate (all current steps + `steward` fixture: 2 seeds × 24 days steward on, ledger exact, 0 panics) → publish → `docs/v8/mux/STAGE_A_REPORT.md` (Korean) with screenshots of the new HUD, an alert being clicked, a lesson card, the steward building the first house; `README` play section updated from `controls.json`.

# 2. STAGE B — Review and refactor (M-EXT)

## B1 The review (`docs/v8/review/CODE_REVIEW_2026-10.md`, Korean with English identifiers)
Read the whole tree and write: the architecture map (crates, modules, systems with priority/interval, bridge collectors, GDScript layers and panels) with line counts; the data flow per pass (M1–M5.5, M-ART2); every **content enum** with its uses (count of `match` sites, files) and a verdict content/mechanics; hard-coded content names in Rust and GDScript (grep with the manifest/data id lists); V7 legacy that only the frozen baseline needs (paths, line counts, what the identity gate and the world-model research need from it) — **do not delete baseline code**; propose its isolation as an owner decision; dead code (unused functions/modules/files: `cargo +nightly udeps` if available, `cargo clippy -W dead_code` on the workspace, GDScript files and functions referenced nowhere, data files loaded nowhere, tools and docs referring to deleted things); duplication; determinism risks (f32 in simulation, HashMap iteration order, time-dependent code); performance hot spots (a `worldsim_perf` per-system profile at N = 1,000 and the share of `decision/system.rs`); test-suite health (slow tests, tests of removed V7 features still running, overlapping tests); docs drift (README, ARCHITECTURE, skills vs code). End with a prioritised refactor list, each item: problem, evidence, fix, risk, behaviour-preserving or not.

## B2 Content enums → data (behaviour-preserving)
For each enum the review marks as **content**, at least `ItemKind`, `BlueprintKind` (with the construction styles of the art material system), `RecipeKind`/workshops, and as far as time allows `Top`, `AnimalKind`, `PressureKind`, `Designation`: replace the closed enum by a registry loaded from versioned RON (`sim-core/data/colony/*.ron`) with stable numeric codes equal to today's (saves keep loading; add a migration test from M5.5 and M-ART2 saves), tags/capabilities instead of name matches, and a category fallback. The bridge and GDScript read names/codes from the registry (no name lists in GDScript; `bindings.json` keys map to registry ids). **Every refactor commit passes the safety net** (identity + fixtures byte-identical).

## B3 "Data only" tests per family (ADR 0033 §3)
A test-only data pack (`sim-test/data/ext_pack/`) adds one item, one recipe, one blueprint/construction style (mud brick), one pressure kind, one animal, one knowledge node and one alert/lesson — and a test world uses them **without any code change**: gate step `data_only_ext`. Extend the extensibility grep to GDScript UI scripts and shaders, single-quoted and concatenated strings.

## B4 M5.5 owner decisions (behaviour-changing, each its own commit)
Population cap also at acceptance; trader goods weighting (techniques with ≤ 2 practitioners, rare next-era artifacts); meat beside the hearth when no stockpile cell accepts it; F4 and F9 bars rewritten with their arithmetic and re-measured on held-out seeds 1100–1131 (48 days) — report the before/after.

## B5 Tree species (behaviour-changing, droppable)
Trees get a species from worldgen (weighted by terrain/biome, data `sim-core/data/colony/flora.ron`, names = wood materials `oak`/`pine`/`birch`); felling yields logs of that material; the snapshot gets a species column; the renderer uses it instead of the coordinate hash (remove the hash path); walls built of pine logs tint as pine through the material system. Fixtures re-captured, effect explained.

## B6 Refactors from the review
Do the review's top items that are behaviour-preserving and fit the stage budget — expected candidates: split `decision/system.rs` by arm into modules without changing logic (byte-identical); split `snapshot.rs` collectors by domain; remove tests of deleted features; consolidate duplicated helpers. Each with the safety net.

## B7 Stage B publish
Gate (all steps + `data_only_ext`) → publish → `docs/v8/mux/STAGE_B_REPORT.md` (Korean): what the review found (summary table), what was refactored (before/after line counts and enum counts), what was left and why, the behaviour-changing commits with their measured effects.

# 3. STAGE C — Cleanup and records

## C1 Delete what is dead
Everything the review proved unused (with the proof per item: no reference in code, scenes, tools, gate, docs): code, scenes, assets (old atlas once B2 moved the top-bar icons to the manifest), tools, stale docs (move superseded docs to `docs/archive/` with an index line, do not lose history). Not the baseline/V7 paths the identity gate or research needs. Gate after deletion.

## C2 Issue registry and GitHub
- `docs/ISSUES.md`: one row per issue since V8-0, backfilled from every pass report, DECISIONS file, review and the planner/owner notes listed in this brief (known limitations, failed bars and their causes, debts, process incidents) — id `ISS-NNN`, title, found in (pass, file, commit), category (bug/design/debt/process/art/perf), status (open/fixed/won't fix/deferred), resolution (commit or pass), link. Include the process incidents: the contended benchmark ledger (M3/M4), two sessions in one checkout (M-ART2 D19), the wrong play command (2026-10-24), the goal-file start-check stop (PLAY2), stopping a gate to change an instrument (M5 D19), the planner's mis-sized evaluation bars (F7/F8 across M2–M5.5).
- `docs/LESSONS.md`: the lessons those incidents taught, each linked to the skill/AGENTS rule that now prevents it.
- **GitHub Issues:** if `gh auth status` succeeds for `hyunlord/new-world`, create one GitHub issue per **open** registry row (labels `debt`, `bug`, `design`, `art`, `perf`, `process` — create them if missing), body linking the registry row and evidence; write the issue number back into `docs/ISSUES.md`. Do not create issues for fixed rows; do not close or edit existing issues you did not create. If `gh` is unavailable, record that and leave the registry as the record.
- `docs/HISTORY.md`: complete and consistent from V8-0 to this pass (every pass, its gate commit, its report, its owner decisions); `docs/ROADMAP.md` updated (M-UX and M-EXT done or partial; next: M6 prep — recommend).

## C3 Final publish and report
Gate → publish → remove your worktree → `docs/v8/mux/MUX_REPORT.md` (Korean) and the final message:
```
## 구현 완료 보고 — M-UX + 검토
### 구현 의도
### Stage A — 맥 조작, 화면 버튼, 미니맵, 도움말, 경고, 목표, 학습 도우미, 마을 스스로 (평가 표 포함)
### Stage B — 전체 검토 요약, 데이터로 옮긴 열거형, 데이터만으로 추가 시험, M5.5 결정 반영, 나무 종, 리팩터링 (전후 수치)
### Stage C — 지운 것(근거), 이슈 목록(건수·상태별), GitHub 이슈 번호, 교훈
### 안전망 (각 리팩터링 커밋의 identity·fixture 결과)
### 성능 (틱 비용, UI 프레임 비용)
### 변경된 파일 목록 (단계·커밋별)
### 확인된 제한사항
### 오너 결정 필요 (기준선 V7 코드 분리 여부 등)
### Gate 결과 (단계별)
### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)
git ls-remote 결과
```

# 4. 인게임 확인사항 (오너용)
1. 트랙패드 두 손가락으로 화면이 움직이고, 핀치로 확대·축소된다. 도구를 안 고른 상태에서 땅을 끌어도 화면이 움직인다.
2. 오른쪽 아래에 ⏸ ▶ ▶▶ 속도 버튼, ＋/− 줌 버튼, 미니맵이 있고, 미니맵을 누르면 그곳으로 간다. 버튼에 마우스를 올리면 단축키가 보인다. 오른쪽 위 "?"를 누르면 조작 전체가 나온다.
3. 오른쪽에 "겨울 장작 부족", "밖에서 자는 사람 3명" 같은 경고가 뜨고, 누르면 그 원인으로 화면이 가고, 어떻게 고치는지 알려 준다.
4. 왼쪽 아래 학습 카드가 하나씩 다음 할 일을 알려 준다(끌 수 있다).
5. 아무 명령도 안 해도 "마을 스스로"가 비축 구역을 만들고, 나무를 베고, 벽 있는 집과 침상을 짓고, 겨울을 넘긴다. 연대기에 "마을이 스스로: …"가 남는다. 위 막대에서 끄면 다시 모든 걸 직접 지시한다. 내가 내린 명령이 늘 우선이다.
6. G를 누르면 첫해 목표와 시대 목표가 체크 표시와 함께 보인다.
7. 물이 땅보다 짙은 파랑이고, 겨울 눈이 위장 무늬가 아니라 부드럽게 쌓인다.
8. 게임 속도·FPS는 그대로다.
