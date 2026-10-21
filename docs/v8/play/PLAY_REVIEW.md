# V8-PLAY review (goal 4.1)

A fresh `general-purpose` subagent per round got only `README.md`, `tools/play.sh`, the two `.command` files
and `docs/images/m1_contact_sheet.png` (copied into an isolated folder), and answered: "Starting from a Mac
with Godot in Downloads and this clone, what exactly do I do to play the current game, and what will I see?"
Full reports: `/Users/rexxa/work/play-data/review/review{1,2}_report.md`.

## Round 1 (README at `e34d63a4`) — 17 findings

| # | sev | finding | disposition |
|---|---|---|---|
| 1 | high | F1–F5 are system keys on Mac keyboards; every M1 order sits behind them; no Home key on a MacBook | fixed: README "맥 키보드 주의" (hold fn, or the standard-function-keys setting, or click the bar buttons; H for Home) |
| 2 | med | prerequisites incomplete (how to get Rust, command-line tools, network, disk); a missing `cargo` showed only "빌드 실패" | fixed: README prerequisites (rustup, `xcode-select --install`, network, disk 1.4 / 4.3 GB measured); `play.sh` now says "Rust(cargo)가 없습니다 — https://rustup.rs …" (tested with cargo hidden) |
| 3 | med | first-start time probably understated; silence during the build looks like a hang | fixed: README gives the measured times (main ≈ 30 s, v7 ≈ 1 min on this Mac; minutes when crates are downloaded first) and says silence is normal; the build line now says so too |
| 4 | med | nothing tells the player that nothing happens until orders | fixed: README "처음 할 일" (F1 drag, F2 drag, F3 click; ≈ 34 s to a hut at 1×) |
| 5 | med | right-click → Open no longer bypasses Gatekeeper on recent macOS | fixed: both routes given (right-click → 열기; System Settings → 개인정보 보호 및 보안 → "그래도 열기") |
| 6 | med | the Downloads-folder permission prompt is not mentioned; denying it gives "Godot not found" | fixed: README says to allow it, or move Godot.app to `/Applications` |
| 7 | low | Godot 4.6 is never checked | fixed: `play.sh` warns when `--version` is not 4.6 (not fatal) |
| 8 | low | failures before the log exists show no log path, yet the `.command` says to check it | fixed: `.command` text "위의 메시지(있으면 로그 경로)"; README says when the log exists |
| 9 | low | exit-code list incomplete (worktree problems, game crash → 1; a set `GODOT_BIN` has no fallback) | fixed in README |
| 10 | low | offline first run of `main` reports "unknown version" | fixed: extra hint line about the network / `git fetch origin`; README exit code 3 mentions it |
| 11 | low | the launcher depends on the clone (scripts, `.git` of the worktrees) | fixed: README says not to move or delete the clone |
| 12 | low | 1920×1080 window larger than a MacBook screen; closing Terminal kills the game | fixed: README notes both (title-bar double-click fits the window; `canvas_items` stretch) |
| 13 | low | "debug로 빌드" presented as fixed | fixed: README says the profile comes from `macos.debug` (debug in every version today) |
| 14 | low | `--seed`/`--speed` statement is a snapshot; invalid values exit 4; usage omits `-- godot args` | fixed: README states the detection rule, exit 4, and `-- <Godot 인자>`; the usage block stays as the goal wrote it |
| 15 | low | import staleness covers only some folders | no change: those are the folders 2.1 names (plus `project.godot`, the `.gdextension`); `localization/compiled` changes come with script changes in practice — recorded here |
| 16 | low | the manual section runs the clone itself and uses `$GODOT_BIN` before defining it | fixed: `source tools/v8/godot_bin.sh` first, "runs this clone as checked out", `--resolution 1920x1080` |
| 17 | low | wording: "아래 1–4단계", "배" for speed and zoom, no cleanup advice | fixed: the step reference is gone, zoom written as "확대", log/worktree cleanup advice added |

## Round 2 (README after the round 1 fixes) — 13 findings, the last round

The reviewer's step-by-step answer was right on every code claim it checked (Godot lookup, `main` =
`origin/v8/main`, worktrees, build/import conditions, `--seed`/`--speed`, exit codes, `--`, log path, symlinks).

| # | sev | finding | disposition |
|---|---|---|---|
| 1 | med | a key press after a failure does not close the Terminal window; a window is also left after a normal quit | fixed: launcher text "아무 키나 누르면 끝납니다 (그다음 터미널 창은 ⌘W로 닫으세요)"; README says ⌘W |
| 2 | med | worktree recovery needs `git worktree prune` run against the clone | fixed: `rm -rf …/new-world-play/<버전>` then `git -C /Users/rexxa/work/new-world-pass4 worktree prune` |
| 3 | med | the clone's branch decides which `play.sh` runs; the launcher is not updated by `git fetch` | fixed: README names the branch (`v8/wip` or `v8/main`) and `git -C … pull --ff-only` |
| 4 | med (cond.) | the Downloads lookup works only if `godot_bin.sh` holds a literal path | checked: it holds `/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot` (literal); README now writes the absolute path and mentions the Spotlight fallback |
| 5 | low | the quarantined item is Godot.app, not the `.command` | fixed: "open Godot.app once from Finder" line |
| 6 | low | night comes soon after the start; the hut timing assumes quick orders | fixed: the game starts at 08:00 and night falls ≈ 65 s later at 1×; README says to pause with P while ordering (orders apply while paused, as `m1_gui_test.gd` does) |
| 7 | low | `--no-build` unexplained; usage omits `-- godot args` | fixed: `--no-build` bullet; `--` already had a bullet; the usage block stays as the goal wrote it |
| 8 | low | an update can re-import as well as rebuild | fixed wording |
| 9 | low | small screens are the normal MacBook case | fixed wording; opening view (08:00, 2×, camp centred) stated |
| 10 | low | F4/F5 meaning unclear | fixed: F4 removes stockpile tiles in the dragged area, F5 removes chop marks and blueprints (`cmd_remove_zone_area`, `cmd_cancel_area`) |
| 11 | low | a Korean input source may stop letter keys | added as a conditional hint (not verified on this Mac) |
| 12 | low | "M1 스크린샷 12장" is 10 M1 + 2 V7; the sheet goes stale when `v8/main` moves | fixed the count and stated `v8/main` = M1 now; replacing the sheet is end-of-pass item 7 |
| 13 | low | opening zoom and HUD fields not described | opening view added; HUD fields left to SCREEN_REPORT P1 |
