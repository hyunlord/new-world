# ADR 0040 — Measure policy changes through read-only observations

## Context

M5.5's historical F4/F9 bars combined availability, exposure and changing craft
populations. B4 requires population, trader and meat-hauling decisions to be
separate behavior changes, with arithmetic registered before held-out results.
The existing end-of-run summaries cannot reconstruct a craft's pre-award skill
or a caravan's generation inputs.

## Decision

Use an opt-in probe observer, controlled public-boundary fixtures, and an
independent Python scorer. The observer takes immutable engine references and
retains pre-tick craft inputs plus the complete persistent craft tally map,
including deceased agents. It is not a RuntimeSystem and adds no save fields.
Ordinary output remains unchanged when observation is disabled.

Register raw field types and all outcome bars before implementation. Runtime
metadata identifies the arm, installed catalog and immutable arithmetic
manifest. Rust records an externally supplied SHA256; the root runner and
scorer verify the actual file bytes. This avoids a new dependency or a circular
hash embedded in the measured binary.

Controlled cold fixtures measure the real apparel Cold filter, not natural
survival. Controlled craft fixtures first pay inputs through normal admission,
then compare real finish-boundary arithmetic at one fixed tick, not job speed.
Natural tick-boundary observations are not relabelled as intra-system captures.
Unresolved hauling and population attribution remain explicit.

Freeze B0/BP/BPT/BPTH binaries, scripts and scorer before viewing the 32 held-out
seeds. Adjacent differences are conditional sequential effects, not a factorial
experiment. Preserve original failed bars and every invalid measurement.

## Alternatives considered

- Production callbacks or a registered observer system: rejected because they
  expand simulation or serialized state for an evaluation-only requirement.
- Reconstruct craft inputs from final survivors: rejected because award, decay
  and same-tick death can change or erase the required inputs.
- Treat zero cold denominators or zero crafts as success: rejected because they
  do not measure the stated mechanism.
- Full M5.5 control matrix: preregistered NOT_RUN; this experiment makes B4-only
  claims and does not replace the historical complete evaluation.

## Consequences

Raw evidence is larger and strict reconciliation can invalidate a run. Such a
failure requires a preserved, documented correctness repair before freezing,
not dropped rows or post-hoc acceptance changes. Each behavior-changing policy
still needs its own decision and commit; this ADR authorizes only measurement.

## Date

2026-10-25.

## Source

`docs/v8/mux/MUX_GOAL.md` B4; `docs/v8/mux/MUX_DECISIONS.md` D39;
`docs/v8/mux/B4_PREREGISTRATION.md`; `docs/v8/mux/B4_OBSERVER_SCHEMA.md`.
