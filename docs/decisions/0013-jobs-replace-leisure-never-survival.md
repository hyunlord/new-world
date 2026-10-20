# ADR 0013 — Jobs replace leisure and idle time, never survival; the decision seam

## Context
Orders need hands: someone must fell the marked trees, carry the logs and build the hut. The V7 decision system (`rust/crates/sim-systems/src/runtime/decision/system.rs`, one ~2,900-line tick function) already decides hunger, thirst, sleep, foraging, firewood, social life, fire and flight. It must not grow into a job system.

## Decision
All job logic lives in `rust/crates/sim-systems/src/runtime/jobs/`:
- `JobAssignSystem` (124) gives a `Job` to an adult (elders included; never a child or a porter) who is idle or at leisure. The order is Build (deliver logs, then build) > Chop > Haul, nearest first, with reservations.
- `JobExecuteSystem` (127) walks each job through its stages (`ToSource`, `Working`, `ToDest`). It hands a builder to the V7 construction flow and checks the item ledger and the reservations.

The decision system calls the job layer at one narrow seam (`jobs::seam`; +26 −1 lines in `decision/system.rs`):
- a job holder runs the Idle cascade as a temporary Idle, with leisure skipped for it;
- after the loop, a choice of nothing, the meal wait or the worker's own colony site keeps the job;
- any other choice (food, water, sleep, foraging, firewood, social, fire, flight) releases it and stands;
- the post-pass keeps the job holder's target.

One more line keeps the V7 memory "bias-only Construction" arm out of the colony (`M1_DECISIONS.md` D20).

## Alternatives considered
- Jobs as new arms of the decision cascade: hundreds of lines in the tick function, and job state spread over the decision's locals.
- Jobs that pre-empt needs (RimWorld's "forced" work): people would starve working; survival must win in M1.
- A separate scheduler that overrides `AgentState` after the decision: two systems fighting over the same state every tick.

## Consequences
- People work only when nothing more urgent calls. Jobs are often released and re-taken: 90–150 cancels per seed in 10 days in the evaluation, with no reservation left behind.
- A new job type is a new `JobKind` with its keys, its assign rule and its execute stages, all inside `jobs/` (`worldsim-architecture`, "add a job type").
- The decision seam is the only coupling and is recorded with its line range in DECISIONS and the architecture skill.

## Date
2026-10-20

## Source
docs/v8/m1/M1_GOAL.md 4.3, 7; docs/v8/m1/M1_DECISIONS.md D9, D10, D13, D20; rust/crates/sim-systems/src/runtime/jobs/seam.rs
