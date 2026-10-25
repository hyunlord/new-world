# ADR 0042 — Author trader scarcity policy outside saved rules

## Context

Generation needs current adult/recent practitioner counts, while the historical
practice mask intentionally serves different purchase/eureka semantics. Adding
fields to serialized NeighbourRule would change the saved binary layout.

## Decision

Use a strict optional top-level goods policy with authored practitioner threshold,
weights, cutoff and rare era. None retains legacy behavior; the bundled enabled
policy is maximum2, scarce30/staple10/rare1, cutoff750, rare era Neolithic.
Compare against the authored era rather than adding a content-name branch.
Count current living adult practitioners at generation and classify rare first.
Preserve keyed draw slots, authored order, quantities, quality and barter.

Resolve the data through a pure passed-catalog validator, after catalog boot has
completed. Preserve historical all-Knowledge masks and purchase counters.
Independently validate the measurement policy and hash neighbours data because
it is not part of the catalog revision. Keep the registered scorer schema/formula.

## Alternatives considered

- Put new weights in saved NeighbourRule: rejected to avoid a binary-layout change
  and save-origin-dependent mixed policy values.
- Reuse cached practitioners or historical practice masks: rejected because their
  timing and population semantics differ from current adult/recent eligibility.
- Hardcode the rare era in production: rejected under ADR0033 data ownership.

## Consequences

Stored offers and rule bytes survive; future offers in an old save intentionally
follow the installed policy. Cross-build continuation identity is not promised.
New eligibility and rare weights may alter economy and knowledge. Separate T
fixtures and frozen BPT measurements retain every original comparison.

## Date

2026-10-25.

## Source

`docs/v8/mux/MUX_GOAL.md` B4; `docs/v8/mux/B4_PREREGISTRATION.md` section6;
`docs/v8/mux/MUX_DECISIONS.md` D41.
