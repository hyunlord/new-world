# M1 decisions

Numbered decisions of the M1 run (goal `docs/v8/m1/M1_GOAL.md` 0.2): context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/m1-data/` ($DATA), outside the repository.

## D1 — The target image was regenerated from the committed reference script

- Context: 0.1 copies `~/Downloads/mock_B_near.png` to `docs/images/v8_target_view_b.png`. At the start of the run
  `docs/images/` was empty and no `mock_B_near.png` existed in `~/Downloads` (the copy had failed silently).
- Choice: run `python3 docs/v8/m1/mockup_gen_reference.py $DATA/mock assets/fonts/galmuri/Galmuri11.ttf` (the
  script the brief names as the one that drew the image; seeded `random.Random(7)`, deterministic) and copy its
  `mock_B_near.png` unchanged to `docs/images/v8_target_view_b.png`.
- Reason: the script is the image's source; Part 4 and the player's-eye reviewer need the image.
- Evidence: sha256 `b9cc1efccdf0b96310c3eba6a5dda050247c039c4d1e8582855e66649e4d8c4a`, 1920×1080; the script also
  wrote `mock_B_far.png` and `mock_A_near.png` to `$DATA/mock/` (not committed).

## D2 — `.gitignore`: `*.uid` removed with the three exception lines; evidence images ignored under docs/

- Context: 5.3 says `.uid` files are committed and lists the agreed rules (ignore `docs/**/*.png|jpg` except
  `docs/images/**`; remove the three `.uid` exception lines for files that no longer exist).
- Choice: remove `*.uid` together with its three `!…gd.uid` exceptions (committing `.uid` files needs the
  ignore gone), add `docs/**/*.png`, `docs/**/*.jpg`, `!docs/images/**`. Staged with the 5.3 patch procedure;
  the owner's local hunk (`/graft/`) stays unstaged.
- Reason: the owner's decision of 2026-10-20.
- Evidence: `git diff --cached .gitignore` showed only these lines; `git diff .gitignore` still showed the
  `/graft/` hunk (`$DATA/gi.patch`).

## D3 — What the UID check checks

- Context: 5.3 asks for a check that fails when a tracked `.gd` / `.gdshader` / `.tscn` lacks its `.uid` or a
  `.uid` has no source. Godot 4.6 writes `<file>.uid` for scripts and shaders only; a scene keeps its UID in its
  header (`[gd_scene … uid="uid://…"]`), and folders with `.gdignore` (`tools/`, `rust/`, `.agents/`) are never
  imported, so their scripts have no `.uid`.
- Choice: `python3 tools/v8/check_docs_helpers.py uid` (check 8 of `tools/v8/check_docs.sh`; gate step
  `uid_check` in Part 6): every tracked `.gd`/`.gdshader` outside a `.gdignore`'d folder has a tracked `.uid`,
  every tracked `.uid` has a tracked source, every tracked `.tscn` header has a `uid=`.
- Reason: checks what Godot actually produces; a rule for `.tscn.uid` files could never pass.
- Evidence: before staging, 41 sources lacked a tracked `.uid`; after staging the 42 local `.uid` files whose
  source is tracked, the check prints nothing.

## D4 — Orphan local `.uid` files moved out of the tree

- Context: removing `*.uid` from `.gitignore` showed 12 untracked `.uid` files whose sources V8-0 deleted
  (`shaders/*.gdshader.uid` ×8, `addons/harness/*` ×4).
- Choice: move them to `$DATA/orphan_uids/` (not deleted, not committed).
- Reason: they belong to no tracked source; committing them would fail the UID check, leaving them would keep
  `git status` noisy.
- Evidence: `$DATA/logs/orphan_uids.txt`.

## D5 — How the V8-0 screen check measures S1–S8

- Context: 5.1 asks for `tools/v8/verify_v8_0.gd` (extending `capture.gd`), run windowed on V8-0 and on the
  V7 game, and names eight checks.
- Choice: one script for both games (node paths are the same, so no adaptation was needed). It freezes the
  live loop and steps with `advance_ticks`, so both games are compared at exactly ticks 1,440 and 4,320.
  - S1/S2: a node-tree scan against the 18 deleted V7 UI files (`tools/v8/v7_ui_deleted.txt`, taken from
    the `V8_0_INVENTORY.md` verdict table; INDEX.md has no "7.3 UI" heading, the inventory's 7.3 keep set
    is the list).
  - S3: computes the top-bar values from the bridge with `hud_topbar.gd`'s formulas (the two games' HUD
    labels are not built the same way).
  - S4/S5/S6: synthetic `Input.parse_input_event` clicks and key presses.
  - S8: 120 s of the live loop at `set_sim_speed(3.0)`.
- Reason: identical inputs on both games; the bridge values are what the top bar shows.
- Evidence: `docs/v8/m1/V8_0_SCREEN_CHECK.md`; `$DATA/v8_0_check/`.

## D6 — S8 compares debug libraries and is reported as two contended pairs

- Context: both checkouts' `sim_bridge.gdextension` load `rust/target/debug/libsim_bridge.dylib`, the game's
  default. The Mac carried foreign load (1-minute load 4–21), and the first V7 run overlapped one of my
  cargo builds.
- Choice: run the pair twice, in both orders (V8 first, then V7 first), each with the debug library the
  game loads. Report all four numbers and the loads. Do not present them as perf evidence beyond the S8
  ratio.
- Reason: like-for-like library and procedure; the order swap guards against drift during the session.
- Evidence: V8-0 111.00 / 109.98 against V7 102.37 / 102.42 mean FPS (ratios 1.084 / 1.074).

## D7 — Docs round 1 for Q3 and Q7

- Context: 5.2. A fresh reader (README, docs/** without docs/v8/m1/, skills) answered both questions in
  substance and listed 15 + 11 gaps.
- Choice: fix the Q3 gaps now (ARCHITECTURE §6 rewritten as 10 steps, architecture skill B and its
  procedures, testing E, godot bridge rules). Fix the Q7 wording now (real-time ticks per second, speed
  keys, pause, the dropped surplus, display-clock day and season, ages in days). Re-ask both in round 2
  after Part 3 changes the time scale.
- Reason: Part 3 rewrites the time scale anyway; one round now, one after, keeps within two rounds.
- Evidence: `$DATA/logs/docs_q3_q7_round1.md`, `$DATA/logs/docs_fix_round1.md`.
