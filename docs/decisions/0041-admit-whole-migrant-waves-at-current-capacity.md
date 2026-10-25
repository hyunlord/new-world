# ADR 0041 — Admit whole migrant waves against current capacity

## Context

Generation checks cannot protect admission when births, deaths or another wave
change the population while visitors travel. An early answer is not a capacity
reservation. Partial admission would lose the existing wave and identity contract.

## Decision

Check all live Agents, including children, plus the entire wave against the
configured cap at explicit acceptance and immediately before joining. Reject zero
cap and over-cap populations. Preserve normal whole-wave joining and assigned IDs.
A previously accepted wave that no longer fits becomes Waiting without an answer,
retaining its ordinary arrival deadline and receiving no automatic attitude penalty.

Use a transient admission error mapped to the existing public Blocked result.
Failed explicit answers leave domain state untouched; existing command receipt and
rejected-command accounting still apply. Preserve the generation-only capped tally.
Evaluation counts command/arrival attempts from immutable observations, not a new
per-tick or saved counter. Births are outside this migrant admission restriction.

## Alternatives considered

- Reserve slots at early acceptance: rejected because live capacity must be checked
  at joining and no reservation lifecycle was requested.
- Admit a fitting subset: rejected because wave membership and preassigned IDs are
  atomic under the registered behavior.
- Add a serialized rejection variant or broaden capped: rejected because existing
  Blocked plus separate evaluation observations preserve save and historical meanings.

## Consequences

Formerly overshooting migrations can be refused, changing downstream population
and economy. Ten deterministic boundary cases, recaptured fixtures and separately
frozen BP observations must document the effect; no original golden is overwritten.

## Date

2026-10-25.

## Source

`docs/v8/mux/MUX_GOAL.md` B4; `docs/v8/mux/B4_PREREGISTRATION.md` section5;
`docs/v8/mux/MUX_DECISIONS.md` D40.
