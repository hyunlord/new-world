# ADR 0053 — Finish food actions and budget durable winter reserves

Date: 2026-10-29. Status: accepted for bounded round-2 implementation; verification pending.
Authority: [M6A_GOAL](../v8/m6a/M6A_GOAL.md); follows [ADR 0052](0052-item-food-chain.md).
Source: calibration round 1 at `dba478e54`, `round1-food-diagnosis.md` and
`water-diagnostic-1.jsonl` under `/Users/rexxa/work/m6a-data/20261029`.

## Context

Round 1 preserves exact food accounting in all eight worlds but none survives
72 days. Durable food at first winter is far below the warm ten-adult need of
144 units. Day-42 seed 3 has 113 stored units, but 108 expire the next day.
The original one-day cooking budget also caps preservation. Meanwhile cooking
requires a lit hearth, but the inherited fire only burns at night, and daytime
planning suspends cooking bills. These are separate from seed 7 dehydration:
food targets alternate across a doorway as geometric nearest order reverses.
Water remains reachable, but the food action never reaches either destination.

## Decision

Keep the owner bars and baseline behavior unchanged. Treat this as a separate
behavior-changing candidate for logged round 2, never a reinterpretation of
round 1. No harvest yield, food lifetime, hunger threshold or work-rate changes.

Persist each active colony eating action's exact FoodOwner until completion,
explicit interruption or invalidation. Recompute the owner's physical location
and freshness; never teleport or reset ages. Choose cooked/novel/nearest food
at the next action boundary. Clear targets on finish/interruption/death. Store
the map in the unpublished format-13 food tail; legacy formats through 12
start empty. A recently depleted target may remain briefly until its next
update, so save validation must not reject normal concurrent consumption.

Use one horizon for reserve accounting and harvest eligibility. Winter is
[18,24); expiry at 24 qualifies, expiry at 23 does not. Set the existing
winter_keep_days to six, requiring remaining >= winter_in + six and remaining
> zero. Immediate usable food still includes earlier-expiring meals.

Keep immediate cooking's one-day budget separate from preservation. Let R be
the existing six-day planning demand (at least the frozen warm-adult need),
S actual stored whole-winter-eligible food and P nonduplicated eligible pending
preservation outputs plus completed outputs awaiting haul. Deficit is
max(0,R-S-P). Allocate it deterministically among eligible long-life recipes;
request their actual ingredients through existing harvest/supply jobs. Pending
output is a planning commitment only, never actual stored food or bar credit.
Account for both completion date and output expiry. Avoid repeated commitments
while drying or hauling is in progress. The cold-night envelope for ten adults
can reach 168 units versus 144 warm, but actual cold duration is not measured;
this repair does not silently retune R to that hypothetical envelope.

Allow planning and supply to request future cooking heat. Actual crafting still
requires a real lit hearth. Only colony fires may serve daytime unsatisfied,
active heat-requiring bills with prepared inputs or actual pending craft;
withdraw real fuel through the existing accounting. Paused/fulfilled bills do
not demand heat. No fuel means no flame or cooked output. Baseline night fire
and its deterministic results remain unchanged.

## Alternatives considered

- Re-rank food every tick: rejected because Euclidean proximity does not track
  progress through a wall/door detour.
- Remove the lit-hearth requirement or grant food: rejected; causes must remain
  physical, visible and conserved.
- Count every fresh stack at winter entry: rejected because early expiry hides
  the remaining winter deficit.
- Raise yields or relax survival bars: rejected before mechanism repairs and
  measured next-round evidence.

## Consequences

The unpublished format-13 food tail gains active owner state; published formats
through 12 keep their existing migration path. Food actions can finish a longer
navigation detour instead of chasing a newly nearer stack. Winter preparation
requests more durable work, and daytime cooking competes for real fuel and
labor. These are intended colony behavior changes, not a preserving refactor;
the baseline remains subject to byte identity and resource costs to fresh perf.

## Date

2026-10-29.

## Source

[M6A_GOAL](../v8/m6a/M6A_GOAL.md), [M6A_DECISIONS](../v8/m6a/M6A_DECISIONS.md),
calibration round 1 and the seed-7 observer named above. Focused regression logs
are retained under `/Users/rexxa/work/m6a-data/20261029`.

## Verification

Focused red/green regressions pass for target retention/invalidation, actual
meal and later drinking, save continuity, shared expiry boundaries, preservation
commitment accounting, and fuel-backed daytime heat. The natural seed-7 six-day
diagnostic now ends with ten living people and zero deaths, versus ten deaths
before retention. This is a mechanism check, not a 72-day survival result.
Independent review found a second-workshop selection mismatch; its failing
regression now passes by passing the chosen workshop ID from budget to bill
application. The full systems suite passes 470 tests and workspace all-target
Clippy passes. Fresh baseline identity,
round-2 cohort, final performance, held-out evaluation and full gate remain
required. This decision is not an acceptance or publication claim.
