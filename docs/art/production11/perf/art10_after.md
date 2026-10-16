# Art Pass 10 after-verdicts — NOT_MEASURED (dropped by GOAL priority)

Decision at 2026-10-16T14:00Z. This is the optional GOAL4.6 experiment, not the required P11 release after matrix. P0 W500 section timers have waited since13:14:53; load checks at13:39–13:59 remained4.70–16.03 with no task Godot running after13:35:52. Four final-world sessions may each need the prescribed60-minute quiet wait. Preserve Parts4/5 and target a freeze near15:25 rather than spend three more holds on historical measurements. GOAL0.3.7 drop order is followed: no additional Part3 candidates after the first accepted one, then this optional historical experiment. No historical runtime was launched and no cdf debug build was installed during this pass.

| Historical debug verdict | Before | After | Speed-up | Status |
|---|---|---|---|---|
| Art10 H1: W0 frozen Z4/Z1 | NOT_MEASURED (original final matrix had0 sessions) | NOT_MEASURED | N/A | NOT_VERIFIED |
| Art10 H3: W500 frozen Z4/Z1 | NOT_MEASURED | NOT_MEASURED | N/A | NOT_VERIFIED |
| Art10 H6: W500 AgentRenderer ablation | NOT_MEASURED | NOT_MEASURED | N/A | NOT_MEASURED |

Historical section-timer before total was47.2085ms, debug and contended; source remains `docs/art/production10/perf/agent_sections.md`. Its after is NOT_MEASURED. The historical alternating result29.04ms is not a substitute for a complete final matrix. P11 release measurements likewise cannot fill these debug cells. Art Pass10 remains TIMEBOXED_PARTIAL, and its rejected tick-cache verdict is unchanged.

External preparation is preserved at `/Users/rexxa/work/art-production11-evidence/p3-prep/ART10_AFTER_EXECUTION.md` and `art10-debug-sections/`: exact-cdf/debug/unchanged-P10-collector checks were designed and offline-tested, but the adapter was neither installed nor executed. No procedural preparation is counted as measured performance.
