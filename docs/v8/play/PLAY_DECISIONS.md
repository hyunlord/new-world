# V8-PLAY decisions

Goal: `docs/v8/play/PLAY_GOAL.md`. Data outside the repository: `/Users/rexxa/work/play-data/`
(`shots/`, `logs/`, `gate/`). Base `892edca1` (`v8/main` = `v8/wip`).

## D1 — The M1 shot set is a `--m1-set` mode of `tools/v8/capture.gd`

- Context: 1.1 lists nine M1 shots at fixed ticks, with the U-script orders at tick 360 and a GUI-driven order
  bar shot; `capture.gd` may change, game code may not.
- Choice: one flag, `--m1-set`, in `capture.gd`. It freezes the live loop (as the existing modes do), advances
  one tick at a time with `advance_ticks`, resolves the U-script in GDScript the way
  `rust/crates/sim-test/src/probe/colony.rs` does (`nearest_trees`, `stockpile_block`, `hut_spot`) and sends the
  orders through `SimReads.mutate_raw` (the path the order tool uses). P7/P8 use `OrderTool.click_world` (the
  `--panels` path); P9 uses synthetic `InputEventKey` F3 and `InputEventMouseMotion` (the `m1_gui_test.gd`
  method). It writes `m1_set.json` with every shot's tick and the order results.
- Reason: the existing modes stay unchanged; one bounded run reproduces the whole set.
- Evidence: `/Users/rexxa/work/play-data/shots/m1/m1_set.json`: chop 15 accepted, stockpile 16 tiles
  (113,69)–(116,72), hut top-left (104,70) accepted; hearth (107,70); seed 20261010. A second run of the
  final script (`shots/m1_recheck/`) gave the same `m1_set.json` and pixel-identical day shots; the three
  night shots differ only inside a ~250-px box around the hearth (the fire glow animates on frame time).

## D2 — The goal's ticks do not match the game clock; "work" shots use the measured work tick

- Context: 1.1 assumes tick 360 = 06:00 day 0 and work in progress at tick 900. The colony clock starts at
  08:00 (`hud_topbar.gd` shows `time_of_day`): tick 360 = 1일차 14:00, tick 900 = 23:00. On the default seed the
  hut is finished at tick 766, so at tick 900 nobody carries anything (first run: `nobody carries at tick 900`).
- Choice: P1 at 360 and P2 at 361 as written. A first pass without frames finds the **work tick**: before the hut
  is finished, the first tick with the most people carrying (481, 16:01, 3 people). The world is then reset to
  the same seed (`reset_world(20261010)`) and shot; P3, P6, P7, P8 and P9 are taken at the work tick. P4 =
  hut tick + 60 (826) as written. P5 = the first tick after P4 whose clock reads 23:00 (900). The literal tick
  900 frame is kept as `m1_03b_t900_2x.png`.
- Reason: the shots must show what 1.1 describes (work, a carrying person); relabelling tick 900 as "work in
  progress" would be false. Every shot's real tick and clock is in its caption and in SCREEN_REPORT.
- Evidence: `m1_set.json` `scan` (work_tick 481, hut 766) equals the shot pass (`hut_completed_tick` 766), so
  the reset reproduced the world.

## D3 — P6 cannot show the whole 192×192 map

- Context: 1.1 asks for the whole map at 1×. 1× is the smallest zoom (`camera_controller.gd` `ZOOM_STEPS`);
  1920×1080 at 1× shows 120×67.5 tiles.
- Choice: P6 at 1× centred on the camp, as written otherwise; SCREEN_REPORT says the whole map does not fit.
  No non-game zoom in the capture tool.
- Reason: a 0.5× frame would show a view the game cannot show.

## D4 — P7 is the inspector scrolled to its job and carry lines

- Context: the inspector opens at its top; "지금 …" and "들고 있음: 통나무 5" are below the fold.
- Choice: `m1_07_inspector.png` = scrolled so the job line is near the top (shows both lines);
  `m1_07a_inspector_top.png` = the panel as it opens. Both in play-data; the scrolled one is committed.
- Evidence: the clicked person carried 5 logs (`m1_set.json` `inspected`); the job line read
  "오두막에 통나무 가져가는 중", not 1.1's "통나무 나르는 중" (a different job).

## D5 — V1/V2 in the existing `v7-final-art` worktree

- Context: 1.1 asks for V7 shots "in the base worktree at `v7-final-art` (adapt paths as Part 0 of M1 did)".
  `/Users/rexxa/work/new-world-v8-base` is at `b1ca032f` (= `v7-final-art`), imported, with its debug bridge
  built and the V8-0 capture script copied in (untracked) by M1 Part 0.
- Choice: an untracked copy `tools/v8/capture_v7_play.gd` there with the near zoom set to 8 (V7's
  `ZOOM_MAX`, "nearest zoom"); far at 1× as before; tick 1,440.
- Evidence: `/Users/rexxa/work/play-data/logs/capture_v7.log`; `shots/v7/`.

## D6 — The contact sheet

- Choice: `tools/v8/contact_sheet.py` takes a manifest (`docs/v8/play/contact_sheet_m1.json`: file and Korean
  caption), scales each shot to 960×540 with nearest neighbour, writes name + caption in a 64-px strip in
  Galmuri11 24 px, two per row, and saves a 256-colour palette PNG. 12 shots: P1–P9, the literal-tick-900
  frame, V1, V2 (`m1_07a` left out).
- Reason: nearest keeps pixel art sharp at an exact 2:1 step; the palette PNG keeps the sheet at 0.5 MB
  (limit 4 MB); the manifest makes the sheet reproducible.
- Evidence: 1944×3680, 493,963 bytes; `tools/v8/test_contact_sheet.py` (4 tests).

## D7 — Game bugs seen in the shots are requests, not fixes

- Context: rule 0 forbids `scripts/**`. The chronicle panel drops HutBuilt (kind 22 is not in `NOTABLE`,
  `scripts/ui/panels/chronicle_panel.gd` line 7) although the bridge has the row; the causal panel shows the
  development tag "(γ-2-β)".
- Choice: report both in SCREEN_REPORT and PLAY_REPORT "제한사항" as requests; change nothing in `scripts/`.
- Evidence: `m1_set.json` `chronicle_kinds_at_p4` has `"22": 1`; `m1_04_hut_done_2x.png` shows no such line.

## D8 — Launcher design: one worktree per version, the pass4 clone only as the git source

- Context: 2.1. Playing must never touch the working checkout; several versions must coexist.
- Choice: `tools/play.sh` resolves the version to a commit in this clone after `git fetch` (a failed fetch
  warns and uses the last fetched heads), then `git worktree add --detach` under
  `PLAY_ROOT=/Users/rexxa/work/new-world-play/<name>`; an existing worktree is moved to the new commit
  (`main`/`wip` follow `origin/v8/main`/`origin/v8/wip`; a move that is not a fast-forward is reported, not
  refused, since the worktree holds no work). Arbitrary refs prefer `origin/<ref>`, then any commit-ish; the
  name replaces `/`, `:` and spaces with `-`. `rust/Cargo.lock` is gitignored, so a new worktree gets this
  clone's lock (worldsim-pitfalls, probe row) before its first build.
- Reason: worktrees share objects (cheap), keep each version's `rust/target` and `.godot/` warm, and never
  disturb the pass4 tree.
- Evidence: `/Users/rexxa/work/new-world-play/{main,m1,v7}`; `git worktree list`.

## D9 — Build the profile the editor binary loads (debug), only when stale

- Context: 2.1 step 3 asks for a release build unless the `.gdextension` points at debug. The playable
  Godot is the editor binary, which always has the debug feature, so it loads `macos.debug` =
  `res://rust/target/debug/libsim_bridge.dylib` in every version (m1, v8-0, v7 alike).
- Choice: read `macos.debug` from `sim_bridge.gdextension`; build that profile (`debug` today) and say so;
  skip ("빌드 생략") when no `*.rs`, `Cargo.toml` or `Cargo.lock` under `rust/` (outside `target/`) is newer
  than that library. `--no-build` fails if the library is missing.
- Reason: a release build would never be loaded; the M1 FPS evaluation (U7, ≥ 60 at 4×) was taken with the
  same debug library, so play speed matches what was evaluated.
- Evidence: fresh builds: main 20 s, m1 21 s, v7 21 s (`logs/main-20261021-105305.log`, …).

## D10 — Import: a stamp file decides "older than"

- Context: 2.1 step 4: import when `.godot/` is missing or older than `assets/ scripts/ scenes/ shaders/`.
  A directory mtime does not track edits deep inside it.
- Choice: after a successful import `play.sh` touches `.godot/play_import.stamp`; it imports again when
  any file under those folders (or `project.godot`, `sim_bridge.gdextension`) is newer than the stamp, or the
  stamp is missing (once), or `.godot/` is missing (twice; only the second run is judged: exit 0 and no
  `SCRIPT ERROR`).
- Evidence: fresh imports: main 6 s, v7 37 s; a deliberate parse error in the m1 play worktree failed the
  import step; a runtime-only null call failed the 300-frame check (Godot exited 0 in both); both restored.

## D11 — `--seed`/`--speed` are detected, not added; extra exit code 4

- Context: 2.1 step 5. No version's game scripts read the command line (`OS.get_cmdline_user_args` appears
  only under `scripts/test/` at `v7-final-art`, and nowhere in m1/main).
- Choice: pass them (after `--`) only when the worktree's `scripts/` outside `scripts/test/` contain
  `"--seed"`/`"--speed"`; otherwise print "이 버전은 --seed를 지원하지 않습니다 (무시)". A malformed option
  (`--speed 7`) exits **4**, which 2.1 does not list; using 1–3 would mislabel it. Godot arguments after `--`
  pass through (`-- --print-fps`). A game that exits non-zero (crash or kill) makes `play.sh` exit 1 so the
  `.command` window stays open.

## D12 — Window evidence: Godot's Movie Maker instead of `screencapture`

- Context: 2.3 asks to confirm the window opened with a screenshot. From this session `screencapture -x`
  fails ("could not create image from display": no screen-recording permission in the background session).
- Choice: run `tools/play.sh main|v7 -- --write-movie <dir>/frame.png --quit-after 90`: the real windowed game
  writes its rendered frames; keep frame 89 (1920×1080) per version, delete the rest. Time to window is
  measured separately with `-- --print-fps` (the first "Project FPS" line follows the first second of frames),
  then the window is closed (SIGTERM).
- Evidence: `/Users/rexxa/work/play-data/shots/window/{main,v7}_window_frame89.png`; warm runs: 4.9 s (main)
  and 4.8 s (v7) from the command to the first FPS line, both 117–120 FPS (display-capped).

## D13 — Gate step 21 runs `tools/v8/test_play.sh`

- Context: 4.2 names "the new play check"; 3 asks that `tools/play.sh --headless-check main` pass at every
  pass end.
- Choice: gate step 21 `play_check` = `tools/v8/test_play.sh` (exit codes 3, 2, 4 and the headless check of
  main, m1 and v7). It uses the play worktrees (warm after the first run; about 100 s cold).
- Reason: one step proves both the launcher's contract and that the published versions still load.
