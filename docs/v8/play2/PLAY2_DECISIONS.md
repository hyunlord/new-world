# V8-PLAY2 decisions and execution ledger

- D1: Attachment overrides the older goal's checkout/branch: isolated `/Users/rexxa/work/new-world-play2-dev`, branch `v8/play2`, created from fetched `origin/v8/main` = `f0b759fe29d1f8c16f4ae6a994807036933b50df`. Never edit/build/run in `new-world-pass4`; never signal another process. Existing main checkout branch remains untouched.
- D2: Follow the supplied executable plan without a new approval handoff. Allowed edits only: launcher, README, play2 docs and one line in each milestone document. No game code, Desktop writes or unrelated cleanup. Launcher-owned play worktrees are exercised as explicitly requested; temporary F12 parameter change must be restored.
- D3: Publish only via fast-forward `v8/play2: v8/wip` then `v8/play2: v8/main`, after a full clean-commit gate exits 0. Any remote divergence blocks publication. Gate observations and runtime observations remain separate.

## Execution plan

1. COMPLETED: inspect play root, launcher and Desktop shortcut; record baseline headless/windowed behavior and timings.
2. COMPLETED: reproduce dirty/offline failures; minimally repair launcher with regression verification.
3. COMPLETED: verify windowed checklist and F12; rewrite Korean README and PLAY.md from source-backed keys.
4. COMPLETED: run shell/docs/headless checks; commit only allowed paths with Lore trailers.
5. IN_PROGRESS: full gate on frozen clean commit; inspect every step; publish fast-forward only if green.
6. PENDING: Korean report, owner launcher delivery verification, safe cleanup, final remote verification.

- D4: Require registered same-clone worktrees, no tracked/untracked edits, and branch ancestry before checkout, even at unchanged HEAD. Do not discard bytes or local commits. Isolated fake-engine regression suite first reproduced 14 failures in 26 checks; fixes passed 33 checks, including update-only and offline log persistence.
- D5: Delegate incremental dependency tracking to Cargo on every normal launch, instead of an incomplete mtime filter. `.ron` files are compiled via `include_str!`; modification and deletion each reproduced a skipped build (41-check suite, 2 failures). This removes the source-extension heuristic; `--no-build` remains explicit opt-out. No Rust changes.
- D6: OS capture and System Events are unavailable in this session. Windowed Godot viewport screenshots plus synthetic input prove rendering and game handlers, not physical keyboard/fn/IME or Finder double-click. Preserve that distinction. First window audit passed other controls but failed to construct a wall before F12; retain failed evidence and fix the test setup rather than claim wall acceptance.
- D7: Keep `new-world-play2-dev` as a clean launcher checkout after completion. Removing it would break the newly documented durable command. Existing Desktop symlink cannot be changed under the contract; give the owner its exact replacement command. Do not execute its forbidden pass4 target. No direct edits or local branch movement in pass4.
- D8: Raw captures, logs, backup bytes and generated graft state live under `/Users/rexxa/work/play-data/`. No contact sheet replacement: game view is unchanged and docs/images is outside allowed paths. Pass decisions and lessons stay here rather than changing forbidden ADR/skill paths.
- D9: Code review found exit-zero ERROR masking also on the normal windowed path. Add a failing windowed regression, then scan only output appended by that launch (not earlier first-import warnings). Baseline 42 checks/1 failure; final suite must pass all 42.
- D10: Window audit retry succeeded: 79 checks/0 failures. UI-only setup now verifies chop count, stockpile acceptance and blueprint acceptance before waiting for a real wall. Do not relax the F12 visible-wall criterion to a toast-only check. No human visual/physical-keyboard acceptance inferred.
- D11: Full gate at b05d31d39 was interrupted during debug tests after reproducing a real launcher defect: under pipefail, `printf large-output | grep -q` returned SIGPIPE instead of detecting an early ERROR. The 44-check regression failed two assertions before the fix. Make grep consume the full import stream. Interrupted gate is not PASS; logs retained in gate-play2. Only verified descendants of this session's gate PID 16027 were terminated, no foreign processes. Restart all 34 steps at the fixed commit under gate-play2-final.
- D12: A report cannot contain its own Git SHA. Record tested implementation HEAD, exact gate results and remote output after the green implementation push; the subsequent report-only commit is identified separately and checked for unchanged launcher/game content and docs validity. Never present the report-only commit as the exact gate-tested SHA.
