# Art Pass 13 decisions

- Follow GOAL over generic session-start instructions: no lead/main, main, reset, hooks or harness pipeline. Every git mutation disables hooks using core.hooksPath=/dev/null. Existing local ignore edits remain unstaged.
- The supplied six-Part execution contract is already approved and decision-complete. Planning skill approval/handoff defaults do not override explicit unattended execution; TASKS.md tracks the contract directly without out-of-scope plan artifacts.
- Start clock is conservatively 2026-10-19T01:46:00Z. Never infer acceptance from technical gate or older-pass results.
- No owner feedback file existed at launch.
- Calibration precedes any Godot session, including the full gate's smoke/input checks. No builds will run during measured holds.
- W-multi source recipe is19603/90840 (day64 10:00); source evaluation revision8f681805 is distinguished from current merged runtime. Population/villages remain runtime requirements.
- The full gate runs p13 real-input tests windowed via the same lock as p12. The static SimReads scanner recognizes the copied p13 capture counter helper only under its existing safe_counter_dispatch predicate; protected getters remain checked.
- Global admission waiting is the union of explicit wait intervals, not the sum of overlapping per-world retry intervals. Productive side work does not count as waiting; every deliberate admission sleep is ledgered with --record-wait.
- Before the first performance attempt, audit found and fixed retry spacing from start rather than completion, refused filename slots consuming the six-attempt budget, and incomplete global/window-lock wait accounting. Calibration is unchanged. Eight focused regression tests cover these boundaries.
- The ablation includes every top-level Main node plus UI children. WorldSim-off and the all-off floor explicitly stop simulation; Camera2D is an already-frozen control; SimReads has synchronous getters, so process-disable does not eliminate cache/getter work. These three controls are reported but never ranked as removable presentation cost. Complete inventory and release provenance are validated.
- These measurement-only corrections require a revised tooling commit before release measurements. Retain all initial c1b1358e captures as preliminary evidence; move both detached worktrees to the corrected tooling commit and collect final references there, avoiding mixed-commit baseline provenance. No production rendering or simulation behavior changed in the correction.

## Acceptance validator audit, 02:42Z

- Canonical calibration acceptance now binds the resolved path, SHA256 of the same bytes read, and recomputed R from the 15 original probes. New sidecars carry that hash. The nine first-attempt refused records remain unchanged; missing hashes cannot be retrospectively accepted. Calibration itself is unchanged.
- Identity reports default to `reference_integrity_only`. Explicit `--candidate declump-kernel` requires off A/A-prime and on B with available method, valid strict-boolean readback, and the correct change ID. Running/pan acceptance checks every captured frame stamp; exact dump bytes remain mandatory.
- Fresh actual corrected-reference CLI checks pass (pixel and running). Deliberately submitting the actual off reference as candidate B fails both CLIs with exit 1. Evidence: external identity/validated-reference-control, validated-running-reference.json, negative-off-off-pixels and negative-off-off-running.json. No candidate acceptance is claimed.
- Full Python discovery: 467 tests, 28.498 seconds, OK, exit 0; external acceptance-validator-full-python.log. Focused 52-test rerun also passed after updating an ablation fixture to provide canonical calibration provenance. The initial failed fixture run is retained. Code diff whitespace check passed.
- These are retrospective comparator/admission-validation changes only. Runtime baseline, collectors and both detached worktrees remain 6d493fa8; no build or recapture required.

## Part0 measured diagnosis

- All9 required sessions succeeded on their second, properly spaced attempts; no continued baseline admission waiting needed. Keep the nine rejected first attempts and the W500 population-loss exclusions.
- W200 AgentRenderer intervention21.8505ms; all outside presentation nodes below1ms. WorldSim9.7855ms is a simulation-disabled control and cannot be candidate(c). Do not change simulation or expand the prescribed candidate scope.
- Optional Wmulti node ablation omitted in the explicit first-drop position to prioritize the kernel decision, write-skip trial and mandatory visual/readability work. Both required running section/getter diagnoses are complete.
- Running section captures retain stale speed_evidence prose; actual setter/numeric conditions/tick trajectories establish speed2. Preserve the original evidence and flag the inconsistency; do not falsely recategorize them as frozen.

## Kernel accepted, Part1

Original §5.1 met: all6exactRGBA, all2280running/panframesexactwithoff/onstamps; admittedW500ratio0.7962943016 andW2000.9642023799,3/3fastereach. Independentrawauditagrees. DefaultonwithP12_DECLUMP_KERNEL=0 referenceescape. Stageonlythisone-lineproductchangeforacceptedkernelcommit; preparedwrite-skipworkremainsunstagedandunaccepted. Re-captureaccepted-buildreferencesbeforejudgingnextcandidate.

## 2026-10-19T03:47:44.550253+00:00 Windowless execution override

User prohibited Godot windows, then requested CLI-Anything. The audited installed headless CLI is permitted; GUI runners stay stopped. Kernel acceptance remains valid historical measurement. Write-skip stays default OFF and is NOT_VERIFIED (user-interrupted, not contended); no automatic windowed retry. End Part1 and proceed with non-windowed Parts2–3 implementation and honest evidence gaps. Do not label headless timers, geometry, or raster approximations as display-performance/screenshot acceptance.
