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
