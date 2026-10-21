# GOAL: WorldSim — V8-PLAY: screenshots of the current game for the owner, and a one-command way to play any version

Repository `hyunlord/new-world`. Base: **`v8/main` = `v8/wip` = `892edca1d31b26e535a3e9b62b37d62b99bb1130`** (M1, gate green at `6ea19569`).

This run is executed by **Claude Code** on the Mac in `/Users/rexxa/work/new-world-pass4`. It is short (target 2 hours, hard stop 4). No DGX. Windowed Godot runs are **authorized and required**. Close every window you open.

## Why this pass exists

The owner has not yet played the V8 game. Two things stand in the way:
1. there is no picture of what the game looks like right now, after M1 replaced the whole view;
2. running it means remembering `cargo build -p sim-bridge`, two Godot import passes and the Godot path. That friction is why the V7 roadmap's "owner playtest" step never happened.

This pass removes both. It changes **no simulation code and no game code**. Only `tools/`, `docs/images/`, `README.md` and the docs checklist change.

---

# 0. Rules (short; AGENTS.md and the skills still apply)
- Commit to `v8/wip`; `v8/main` moves by fast-forward only after 4.3's gate run. Messages `v8-play(pN): …`.
- Never `lead/main`, `main`, `codex/*`; never force-push; stage explicit paths; the hook file and local ignore lines stay as they are.
- **Do not edit** `rust/**`, `scripts/**`, `scenes/**`, `assets/**`, `localization/**`, `shaders/**`. If the picture or the launcher needs a game change, write it to the report as a request and stop there.
- Never ask the owner anything; decisions go in `docs/v8/play/PLAY_DECISIONS.md`.
- Work only in the pass4 checkout. Data outside the repo goes to `/Users/rexxa/work/play-data/`.

---

# 1. PART 1 — Pictures of the current game

## 1.1 Capture set
Use `tools/v8/capture.gd` (windowed, `--rendering-driver opengl3 --resolution 1920x1080`). If the M1 scene needs a new option (for example a colony order script so the shots show work in progress), add it to `capture.gd` as a flag; `capture.gd` is tooling and may change.

Seed: the live game's default. Take these shots on **M1 (`v8/main`)**:

| # | file | what | how |
|---|---|---|---|
| P1 | `m1_01_start_2x.png` | the camp at tick 360 (06:00 day 0), 2× zoom, nothing ordered | capture at tick 360 |
| P2 | `m1_02_orders_2x.png` | right after the U-script orders are placed: chop marks on 15 trees, a 4×4 stockpile, a hut ghost "통나무 0/12" | apply the U-script of `docs/v8/m1/` through the bridge commands at tick 360, capture at 361 |
| P3 | `m1_03_work_2x.png` | work in progress: people carrying logs (bundles over heads), a felled area, logs in the stockpile | capture at tick 900 |
| P4 | `m1_04_hut_done_2x.png` | the finished hut with "오두막이 완성됨" visible in the chronicle (toggle `C`) | capture at the tick the hut completes + 60 (read it from the overlay's progress) |
| P5 | `m1_05_night_2x.png` | night, people asleep in the hut and shelters | the first tick after P4 at 23:00 |
| P6 | `m1_06_far_1x.png` | the whole 192×192 map at 1×, with the camp centred | tick 900 |
| P7 | `m1_07_inspector.png` | a person clicked while carrying: inspector with "지금: 통나무 나르는 중" and "들고 있음: 통나무 5" | `--panels` style click on an agent whose carry count > 0 at tick 900 |
| P8 | `m1_08_why.png` | the causal "왜?" panel on an empty tile near the hearth | `--panels` style |
| P9 | `m1_09_orderbar.png` | the bottom order bar with F3 (오두막) active and a red ghost over water or a tree | drive the order tool with synthetic input as `tools/v8/m1_gui_test.gd` does |

And for comparison, in the base worktree at **`v7-final-art`** (adapt paths as Part 0 of M1 did):

| # | file | what |
|---|---|---|
| V1 | `v7_01_near.png` | the old game's nearest zoom at tick 1,440 |
| V2 | `v7_02_far.png` | the old game's far view at tick 1,440 |

## 1.2 Contact sheet and commit
- `tools/v8/contact_sheet.py` (PIL, unit-tested for output size): each shot scaled to 960×540 with its file name and a one-line Korean caption under it, two per row → `docs/images/m1_contact_sheet.png` (≤ 4 MB).
- Commit **only** the contact sheet and P1, P3, P4, P6, P7 to `docs/images/` (each ≤ 1.5 MB; PNG, no scaling of the originals, crop if needed and say so). The rest stay in `/Users/rexxa/work/play-data/shots/`, paths listed in the report.
- `check_docs.sh` allows ≤ 12 images in `docs/images/`; stay within it (there is already `v8_target_view_b.png`).

## 1.3 What the pictures must honestly show
Write `docs/v8/play/SCREEN_REPORT.md` (Korean), one paragraph per shot:
- what is on screen;
- what the owner should look for;
- anything that looks wrong, unfinished or unlike `docs/images/v8_target_view_b.png` (the chosen mock-up). Do not soften it: this report is the owner's first look.

End with a table "목업 대비" listing, for each element of the mock-up (ground palette, person size and shadow, tunic colours, stockpile look, blueprint ghost, hut, trees, hill rock, side panel, bottom bar), whether M1 has it: 있음 / 임시 / 없음.

---

# 2. PART 2 — One-command play

## 2.1 `tools/play.sh` (committed; macOS bash 3.2 compatible)
```
usage: tools/play.sh [version] [--seed N] [--speed 1-4] [--no-build] [--headless-check]
  version: main (default, = v8/main), wip, m1, v8-0, v7, or any commit/tag/branch
```
Behaviour:
1. **Resolve Godot:** `GODOT_BIN` if set; else the path in `tools/v8/godot_bin.sh`; else `mdfind "kMDItemCFBundleIdentifier == 'org.godotengine.godot'"`; else `/Applications/Godot.app/Contents/MacOS/Godot`. If none exists, print one line saying where to put Godot 4.6 and exit 2.
2. **Resolve the version** into a worktree under `/Users/rexxa/work/new-world-play/<name>/` (created with `git worktree add --detach` from the pass4 clone, fetching first; reused if it exists and fast-forwarded when it tracks a branch). `v7` → tag `v7-final-art`; `v8-0` → `24e16677`; `m1` → `892edca1`; `main`/`wip` → the remote branch heads. The pass4 checkout itself is never used for playing.
3. **Build** `cargo build -p sim-bridge --release` in that worktree unless `--no-build`, and only when the Rust sources or `Cargo.lock` are newer than the built library (compare mtimes; print "빌드 생략" when skipped). Check that the `.gdextension` points at the built profile; if it points at `debug`, build debug instead and say so.
4. **Import** headless (`--import --editor --quit`) when `.godot/` is missing or older than the newest file under `assets/`, `scripts/`, `scenes/`, `shaders/`; twice on a fresh worktree.
5. **Launch** the game windowed at 1920×1080: `"$GODOT_BIN" --path . --resolution 1920x1080`. Pass `--seed` and `--speed` only if the game already reads them from the command line (check `scripts/`; if it does not, print "이 버전은 --seed를 지원하지 않습니다" and ignore them — do not add game code).
6. **Log** everything to `/Users/rexxa/work/play-data/logs/<version>-<timestamp>.log` and print the path. Print a one-line Korean summary before launch: version, commit, build (done/skipped), import (done/skipped), seed, speed.
7. `--headless-check` runs steps 1–4 and a 300-frame headless smoke instead of launching; exit 0 only when the smoke has 0 `SCRIPT ERROR` lines.

Exit codes: 0 ok, 1 build/import failed, 2 Godot not found, 3 unknown version.

## 2.2 The desktop launcher
- `tools/play/WorldSim.command` (committed): a double-clickable macOS script that `cd`s to the pass4 clone and runs `tools/play.sh main`, keeping the Terminal window open on error (`read -n 1` after a non-zero exit). Mark it executable in git (`git update-index --chmod=+x`).
- `tools/play/WorldSim-v7.command`: the same with `v7`, so the owner can compare.
- In the report, give the owner the two lines to put them on the Desktop:
  ```bash
  ln -sf /Users/rexxa/work/new-world-pass4/tools/play/WorldSim.command ~/Desktop/WorldSim.command
  ln -sf /Users/rexxa/work/new-world-pass4/tools/play/WorldSim-v7.command ~/Desktop/WorldSim-v7.command
  ```
  and note that macOS may ask once to allow Terminal to run it (right-click → 열기).

## 2.3 Tests
- `tools/v8/test_play.sh`: `play.sh` with an unknown version exits 3; with `GODOT_BIN=/nonexistent` exits 2; `--headless-check m1` exits 0; `--headless-check v7` exits 0 (or document why it cannot, without editing the tag).
- Run `tools/play.sh main` and `tools/play.sh v7` for real, once each, windowed; confirm the window opened (screenshot into play-data) and close it.

## 2.4 README
Replace the "빌드와 실행" section with:
- "가장 쉬운 방법: 바탕화면의 `WorldSim.command`를 더블클릭" plus the two symlink lines;
- "터미널: `tools/play.sh` (`main` / `m1` / `v7` …)" with the usage block;
- the manual three commands kept under "수동으로".
Add a "지금 화면" line linking `docs/images/m1_contact_sheet.png` and `docs/v8/play/SCREEN_REPORT.md`.

---

# 3. PART 3 — Keep it current
- Add to `.claude/skills/worldsim-docs-upkeep` end-of-pass checklist: "7. `tools/play.sh --headless-check main` passes, and `docs/images/m1_contact_sheet.png` is replaced by `<pass>_contact_sheet.png` when the view changed (keep ≤ 12 images; delete the previous pass's sheet)". Update `check_docs.sh` to require that a `*_contact_sheet.png` exists and that README links it.
- Add the launcher and the capture recipe to `worldsim-godot` (where the shots come from, how to add a shot).
- One ADR is not needed; record the launcher design in `PLAY_DECISIONS.md`.

---

# 4. PART 4 — Gate and report
## 4.1 Review
One fresh `general-purpose` subagent gets only README.md, `tools/play.sh`, the two `.command` files and the contact sheet, and answers: "Starting from a Mac with Godot in Downloads and this clone, what exactly do I do to play the current game, and what will I see?" Fix README until the answer is right (two rounds).

## 4.2 Gate
`tools/v8/gate.sh` must stay green (the pass touches docs, tools and images only; steps 14 and the new play check are the ones that matter). Run it on the final commit with nothing edited during the run.

## 4.3 Publish
Fast-forward `v8/main`; `git ls-remote origin v8/wip v8/main`.

---

# 5. 구현 완료 보고 (한국어, `docs/v8/play/PLAY_REPORT.md`와 최종 메시지)

```
## 구현 완료 보고 — V8-PLAY

### 스크린샷
(P1–P9, V1–V2 표: 파일 경로, 한 줄 설명. 커밋한 것과 play-data에만 둔 것 구분)

### 화면 정직 평가
(SCREEN_REPORT.md 요약: 잘 보이는 것 3가지, 어색한 것 3가지, 목업 대비 표)

### 실행 방법
(바탕화면 더블클릭 두 줄, tools/play.sh 사용법, 지원 버전 목록, 로그 위치)
(실제로 main과 v7를 창으로 띄워 본 결과: 빌드 시간, 가져오기 시간, 뜨는 데 걸린 시간)

### 변경된 파일 목록
(커밋별)

### 제한사항
(게임 코드를 못 고쳐서 못 한 것, --seed 지원 여부, 기타)

### Gate 결과
(단계별, 로그 경로)

git ls-remote 결과
```
