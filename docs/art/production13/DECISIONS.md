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
