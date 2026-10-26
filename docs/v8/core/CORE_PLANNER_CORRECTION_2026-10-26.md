# Planner correction — 2026-10-26 22:00 KST

The following owner-provided planner note overrides the original load rule
in CORE_GOAL §1.1 and §5; the original goal remains preserved verbatim in commit `001f236c3` and the working goal incorporates this correction.

> Planner correction for M-CORE (2026-10-26 22:00 KST): the "1-min load ≤ 4" condition in CORE_GOAL §1.1 was the planner's error — it is stricter than the repository's own rule and than every accepted ledger row (M5.5 quiet rows were taken at load 6.44, M-ART2 at ~7). Replace it with the ledger rule (docs/perf/PERF_LEDGER.md, M4 goal 3.6): a run is valid when every repeat starts at a 1-minute load below 8; record the 1/5/15-minute loads before each repeat; runs at ≥ 8 are contended, kept as INVALID, never a base. For every comparison (old decision vs mind, BASE vs M-CORE) interleave the two binaries in the same session so the ratio, not the absolute time, carries the bar. The same rule applies to §5's perf bar. Record this as a DECISIONS entry citing this note.
>
> No other planner job is running on this Mac now (M-UX2.1 is paused until you publish). Resume the goal from the quiet share and arm-cost measurement (§1.1) and continue with the full scope unchanged. If loads stay ≥ 8 for three windows 15 minutes apart, record the samples and continue with the parts that need no timing (inventory, mind, shadow, held-out evaluation), and do the timing at the end before the gate.
