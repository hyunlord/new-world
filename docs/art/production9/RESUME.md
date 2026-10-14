# Live execution checkpoint

2026-10-15T07:36+09:00; still Part 0. Re-read GOAL.md sections 0–4 before resuming. Full goal is all Parts 0–5, not merely reading the attachment. Ten-hour conservative deadline 17:19+09:00.

## Authoritative running handle

- Full merged baseline gate: exec session **81297**, started with `CARGO_BUILD_JOBS=6 GATE_OUTPUT_DIR=docs/art/production9/gates/p0 bash tools/art_pipeline/gate.sh p0`. Last poll confirmed live, progressing through Rust integration tests; `cargo_test` has not yet written its terminal results.tsv row. Poll the same handle; do not launch duplicate gate. Outputs in gates/p0/.
- Base build session4444, import91250, reimport29842: all completed exit0. Original import had transient missing-font startup errors; reimport no errors. Base headless30-frame smoke exit0.

## Git

- Both remote branches initially 56059e0f. Backup art-p9-local-backup points there.
- fe185a23 committed exact owner brief + initial ledgers.
- 49bc0b7c87159340b8024d6b9327bc394dc6daa0 merged da8e45c7858b7973330ebecef8ff35ae5ee936ee without conflicts.
- WIP pushed and ls-remote verified at49bc0b7c. Production art branch stays56059e0f; no main/lead push.
- Preexisting .gitignore graft entry and .ignore untouched/un-staged. New tools/ledger edits currently uncommitted pending validation. Explicit staging only.

## Prepared artifacts and next actions

1. Finish/preserve p0 gate. Three strikes policy on real failing targets; no threshold weakening.
2. Baseline worktree `/Users/rexxa/work/art-p9-base` detached at49bc0b7c, own debug sim-bridge built/imported. Three same-file measurement/capture scripts copied untracked (perf_frames_p9.gd, capture_identity_p9.gd, capture_framings_p9.gd). If scripts change, recopy before launch. Never alter baseline tracked code.
3. After gate green, launch one W0 session via with_quiet_lock_p9.sh; verify actual raw output before the remaining matrix. run_perf_matrix_p9.sh prepares full matrix, but its first W0 is the pilot and subsequent10-minute cooldown permits inspection. No timing session started yet. Existing external worldsim_probe and high load mean quiet acquisition may wait up to60minutes then contended. Every session60min max, ten-minute gaps mandatory. Wrapper exports P9_CONTENDED and records holds.
4. `P9_DEADLINE_EPOCH` should be set to date epoch for17:19+09 before matrix launch. Matrix calls wrapper then window lock; no --fixed-fps. Script usage: `bash tools/art_pipeline/run_perf_matrix_p9.sh baseline /Users/rexxa/work/art-p9-base /Users/rexxa/work/art-production9-evidence/perf-baseline`. Run as background exec session and poll; inspect logs in cooldown. Keep failed attempts. Full baseline JSON/MD generated at production9/perf/baseline.*.
5. Before/after A30 still required separately using original scripts/test/capture_framings.gd CAPTURE_SOCIETY_PERF_ONLY=1, under quiet/window protocol. Not run.
6. Part1 game code not started. Read READ_AUDIT.md first: terrain static including worldgen swap; reset clears terrain immediately (signal deferred); HUD mutation and GodHandEffects._state.clear alias hazards; dynamic string dispatch and mixed real/mock tests. Packed sharing proven in 5-line probe and logged.
7. SimReads measurement API agreed: peek_uncounted(raw_getter_string), frame_call_counts() raw getter->count, frame_call_usec(). Dynamic fallback when absent. Future reader script must honor fake worlds passthrough and all wrapper invalidation requirements.
8. Identity tools prepared, not captured. P1/P2/P4/P5/P6 env P9_IDENTITY_ID/P9_IDENTITY_OUT and --fixed-fps60 (separate CLI args). P3 copied FV path opted-in CAPTURE_IDENTITY_P9=1 + CAPTURE_PLAN_JSON [["RESET",20261013],[33,"09:49","FV","all"]]. Preserve exact 120-frame schedules and compare sidecars. New scripts parsed successfully. pixel_identity_p9.py requires a fresh output directory to preserve rejected comparisons.
9. Independent final review has NOT happened. Fresh reviewer must not see GOAL sections3–7, per section9. All Parts1–5 remain.

## Tooling QA

- perf collector + summarizer reviewed once, corrected restoration/error propagation, tick-carrying-frame requirement, monotonic stamps/control/ordering checks, baseline ablation requirements, after-W500 FFI evidence requirement.
- Unit suite test_summarize_perf_p9.py discoverable via unittest (28 synthetic boundary cases); test_pixel_identity_p9.py11 cases. Leader ran discovery to gates/tooling-unit.log. Synthetic checks are not performance evidence.
- Quiet wrapper isolated fixture checks documented in perf/quiet_wrapper_checks.md; actual lock not yet acquired.
- Last known graft savings leader115126 tokens (two calls), agents152394 tokens (three calls), total267520; no dollar estimate provided.

Latest QA correction: leader `python3 -m unittest discover -s tools/art_pipeline -p 'test_*p9.py'` initially discovered only collector1 test (28 cases); pixel tests were pytest-style and skipped. identity_tools agent has an active bounded follow-up converting those11 cases to unittest before gate's Python phase. Inspect its result and rerun discovery; do not claim identity unit coverage from initial tooling-unit.log.
Identity conversion finished. Leader re-ran discovery of test_*p9.py:8 discovered tests passed, covering identity11 cases + collector28 boundary checks; evidence gates/tooling-unit-final.log. All child agents now done. Gate81297 remains live; no timing sessions yet.

2026-10-15T07:39+09 follow-up: session **68542** is a live prerequisite watcher waiting on gate PID7051. It verifies expected65 gate rows (test script count +7), all exit0, copies the unchanged collector into base, then launches ONLY baseline W0 attempt1 through quiet/window wrapper. Outputs `/Users/rexxa/work/art-production9-evidence/perf-baseline/baseline-W0-attempt1.{json,log}`. If gate fails/incomplete, watcher exits1 without launch. Poll68542; do not launch duplicate W0 or the full matrix. Need inspect first W0 raw result then run remaining worlds with the required ten-minute cooldown. Gate handle81297 still must be collected terminally.

2026-10-15T08:03+09 update: first gate81297 terminal exit1,65 rows,63 pass and cargo124/zoom124. Watcher68542 terminal exit1 without launching W0. Diagnosed test profile mismatch using production8 exact precedent; debug/p0-timeout-diagnosis.md. Second full gate **session14473, PID12851**, `gates/p0-attempt2`, uses test-only OPT_LEVEL1, DEBUG_ASSERTIONS=true, OVERFLOW_CHECKS=true, jobs6/threads6 and unchanged1800 bound; dev/editor unchanged. First gate zoom line17 failure reproduced with --fixed-fps5; only test's two waits changed to Tween.finished, all assertions/camera unchanged. Fixed5 and normal runs PASS, logs debug/zoom-*.log, ledger row added. No timing session yet. Poll second gate; do not duplicate it. First p0 evidence preserved. Test fixture edit and current new tools/docs remain uncommitted.
Second prerequisite watcher **session82926** waits on gate PID12851, checks all65 rows exit0, then launches only baselineW0 attempt1 via quiet/window wrapper. Expected output/log paths unchanged. Do not start another W0 or matrix until watcher terminal and raw W0 inspected. This watcher has not launched a timing session yet.
2026-10-15T08:17+09: Part0 retry session14473 terminal exit0,65/65,2138 Rust tests,cargo762s. Provenance updated. Watcher82926 is still live and authorized to begin baselineW0 quiet preflight; poll same handle and perf/holds.md. Do not duplicate W0. Matrix runner now safely resumes validated pilot evidence (tests17/17 incl9resume tests; tooling-unit-resume.log), then next worlds. New run_identity_p9.sh prepares six separate fixed-fps captures; stub verified only, actualcaptures pending. Two new agent outputs are complete. Full Part0 baseline/ablation/A30, then Parts1–5 still required.
