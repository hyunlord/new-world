# Art Pass 10 execution ledger

Started: 2026-10-15T15:03:00Z (2026-10-16 00:03 KST). Twelve-hour cutoff: 2026-10-16T03:03:00Z.

| Part | Deliverable and verification | Status | Start UTC | End UTC |
|---|---|---|---|---|
| 0 | Preserve brief, merge P17, full gate, live getter census, rederived framings, base build, baseline matrix and A/Aprime | IN_PROGRESS | 2026-10-15T15:03:00Z | |
| 1 | Section timers; measured pixel-identical optimizations; six framing identity plus 600-frame running equality | NOT_STARTED | | |
| 2 | Activity getter/poses/icons/labels, exhaustive mapping tests, six live captures and sidecars, blind review | NOT_STARTED | | |
| 3 | Temperament/family inspector, household rings, fake-world tests and parent capture | NOT_STARTED | | |
| 4 | Final performance matrix, H1-H8 and before/after section costs | NOT_STARTED | | |
| 5 | Independent review, full gate, publish both heads and PR511, Korean report, base cleanup | NOT_STARTED | | |

## Execution events
- 2026-10-15T15:03Z: Saved supplied brief unchanged before repository edits. Original local HEAD and both remote heads verified as 21d9ddd089a77bfc3daa9b9765888eb76a890a85.
- Preserved unrelated preexisting .gitignore modification and untracked .ignore.
- Brief commit fd598fad; merge 0caabf5fd1c62b9d3c99845daab1805e79799db7, no conflicts.
- 2026-10-15T15:04Z: Started full p0 gate, CARGO_BUILD_JOBS=6, after confirming quiet lock absent. Exec session 6597. No renderer changes before baseline.

## Optimization execution plan
1. Instrument only after baseline A/Aprime and timing matrix are complete.
2. Rank section costs; accept one bounded change at a time after alternating 150-frame measurements and six-framing pixel check.
3. Preserve all frame-dependent pose timers, interpolation, visibility and sorting. Cache key includes tick, mutation and reset generations.
4. Spatial candidates retain original iteration order. Protect current behavior with baseline running dumps before optimization.
5. Commit and push each accepted change; record failed attempts and revert only our failed optimization where needed.
- 2026-10-15T15:07Z: Pushed WIP merge; git ls-remote confirms WIP 0caabf5fd1c62b9d3c99845daab1805e79799db7 and publication branch still 21d9ddd089a77bfc3daa9b9765888eb76a890a85. Base worktree created detached at merge.
- Original brief SHA256 equals saved GOAL SHA256: 114e66afd7b8aae6c2c6b8ea50867fc6fce0ebfcbb2e7c4e807e2429d8a4ad40.
- Part 0 preparation: copied twelve P9 measurement tools to P10; original framing ticks deliberately awaiting live census. Separate census and deterministic running-identity tools prepared without changing rendered behavior.
- First full-gate cargo leg failed (exit 101, 275s) on new evidence scripts bypassing protected SimReads access; preserved original log. Fixed scripts through existing ReadRoute; exact P10 diagnostic-helper path added under unchanged call allowlist and logged in TEST_LEDGER.
- Targeted reader-boundary rerun: 4/4 PASS. First gate's clippy, strict (42 scripts), bridge, smoke and Python legs pass; remaining Godot tests still running.
- Base debug sim-bridge built independently in 38.42s; base Godot import running. New comparison Python tests 10/10 PASS; shell syntax checks PASS. Running/capture tools remain runtime-unverified until live captures.
- 2026-10-15T15:14Z: Live day30 10:00 census completed, tick41880, population29, settlements1, seven activity-state holders across TendFire/HearthCircle/Play/SitByFire, zero unknown kinds, five family/temperament samples. Raw JSON archived under baseline/. Started six-framing census (exec53982), bounded1800s.
- Base independent debug build and full Godot import completed exit0; 6241 assets imported, no ERROR/SCRIPT ERROR. Running-dump script check-only exits0 without errors.
- 2026-10-15T15:20Z: Initial full gate terminal exit1 solely from now-fixed source boundary; all remaining legs passed, including W0/W500 600-frame consumer test (299s) and read equivalence (177s). Started full p0-attempt2 rerun at b0d3ab6c, preserving p0 failure logs.
- Original P1 census: population29, settlement1, unlit at40560. Original P2: population29, settlement1, lit at31320. Remaining framing census still live; do not restart.

- 2026-10-15T15:29Z: W200 running baseline/control comparison PASS: 600 exact frame rows, 300 SimReads mutations, tick1440→1740. Harness validation only; final Part1 H5 remains pending. Started baseline W0 running dump (session56844).
- 2026-10-15T15:34Z: P5 merge-baseline capture exactly equals P9 final B in decoded RGBA pixels (1920x1080), evidence baseline/p5-consistency.json. W0 baseline running dump exit0; control capture session83648 active.
- 2026-10-15T15:35Z: Measurement provenance/resume fixes verified: 17 P10 Python tests PASS, Bash syntax PASS, Godot perf collector check-only exit0. Updated collector copied unchanged into detached merge base.
- 2026-10-15T15:36Z: Full running harness control PASS for both W0 and W200, each600 frames and300 SimReads mutations. WIP pushed/ls-remote verified d33daa50fd81f1692bcb288accf8767186ea3ae0; publication remains21d9ddd089a77bfc3daa9b9765888eb76a890a85. P6 visible-inspector fixture validation started session12318.
- 2026-10-15T15:39Z: Six-framing census complete, zero failures,1482.304s; original seeds/ticks retained. P6 panel visible in actual framebuffer; baseline A then Aprime runner started. Baseline matrix session28873 waiting for quiet conditions since15:38Z.
- 2026-10-15T15:49Z: Prepared fail-closed150-frame alternating runner with three control/candidate pairs and full source/build provenance. Python contract tests and Godot parse pass; real candidate-toggle integration and runtime measurement remain pending Part1. A P1/P2/P3 captures exit0 with artifacts/no errors; A P4 ongoing. Existing full gate and quiet-wait handles confirmed live, not restarted.
- 2026-10-15T15:50Z: p0-attempt2 cargo_test terminated124 at1800s during new simulation hazard tests, without an assertion failure in captured output. This is a failed timeout, not PASS; remaining gate legs continue on session59019. WIP455f83193d2e3abd46658f9525fcf1ddec808eb8 pushed/ls-remote confirmed.
- Third gate queued in session84772: waits for verified live gate PID53515 to exit, then for quiet-lock absence, then runs full gate with test opt-level2 into p0-attempt3. Do not start a duplicate. Baseline identity session23540 and matrix session28873 remain active.
- 2026-10-15T15:58Z: p0-attempt2 terminal exit1:73 checks,72PASS; cargo_test alone timeout124. Consumer/equivalence runtime tests181s/129s PASS. Third full gate session84772 has started compiling optimized test profile. Baseline A P4/P5 completed exit0, no log errors; P6 ongoing.
- 2026-10-15T15:59Z: Baseline A completed six launches exit0/no errors,1920x1080 each. A/P5 decoded pixels again exactly equal P9 final B. A-prime has started in same runner23540. Third gate84772 compiling test-opt2; matrix28873 still quiet-waiting.
- 2026-10-15T16:11Z: p0-attempt3 cargo_test PASS exit0 in826s with test opt-level2; clippy/strict/bridge also PASS. Full gate remains active until smoke/Python/Godot legs finish. A-prime P1/P2/P3 pixels exactly equal A; P4 ongoing.
- 2026-10-15T16:20Z: Third full gate terminal exit0,73/73PASS. Archived all three attempts and SHA256 indexes; earlier assertion-boundary failure and timeout remain failures. Baseline A/Aprime all six exact (0 changed pixels/0 max-channel), strict fixed-FPS schedules equal; Part1 candidate H5 still NOT_VERIFIED. Part0 remaining: performance matrix, session28873 waiting since15:38Z.
- 2026-10-15T16:24Z: Diagnostic-only section generator preparation verified: original nonblank statements preserved,25 exclusive spans,33 marker removals plus duplicate/extra-return/reinstrumentation rejection. Clock/loader/generated script Godot check-only exit0;30 P10 unittest-discovered tests PASS. No production renderer edits; runtime loader behavior, profiled identity and section timings remain pending.
- 2026-10-15T16:34Z: Prepared diagnostic W500 section-session harness and bounded runner; leader rerun37 P10 Python tests PASS. Added independent UI/layer/callback/framing and finite-metric checks after review. Godot runtime and generated-script installation remain NOT_VERIFIED; no production changes. Baseline matrix28873 still live,55-minute quiet wait.
