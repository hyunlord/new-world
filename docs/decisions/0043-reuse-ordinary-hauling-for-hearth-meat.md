# ADR 0043 — Reuse ordinary hauling for hunted meat near a live hearth

## Context

A hunter currently leaves meat at the kill when no stockpile cell accepts it. The ordinary hourly larder already consumes ground meat near existing hearth anchors. Immediate food credit would bypass travel, reservations, perishability and cancellation.

## Decision

Preserve stockpile priority. If no accepting reachable unreserved stockpile room exists, choose legal grazer-meat ground within the existing hearth radius, excluding the source, ordered by source Chebyshev distance then y/x. Require an actual live colony hearth building as well as anchor coverage.

Use the existing Haul job and unchanged pickup, movement, drop and later feed. At both source and destination stages, validate the narrow hearth alternative using the actual source or Carry spec, room/reach and reservation ownership. Loss of that destination uses existing cancellation/drop conservation. Ordinary stockpile validation and generic haul assignment remain unchanged.

## Alternatives considered

- Credit food at the kill: rejected because it bypasses the normal physical path.
- Add a special job, zone or saved origin flag: rejected because ordinary Haul already preserves the needed state.
- Permit stale formation anchors with no hearth building: rejected because AnimalSystem cannot feed without a live building hearth.

## Consequences

Only hunter assignment generates new fallback destinations. With no saved origin flag, execution recognizes current meat/location validity rather than proving historical hunter origin. Any remaining hearth building satisfies the global existence guard; feed still uses the existing first sorted settlement. No per-settlement ownership is implied.

Meat delivery, spoilage and downstream food/economy may change intentionally. Separate H recapture and frozen BPTH evaluation retain differences. No safe ground or no live hearth keeps the original no-haul result.

## Date

2026-10-25.

## Source

`docs/v8/mux/MUX_GOAL.md` B4; `docs/v8/mux/B4_PREREGISTRATION.md` section7;
`docs/v8/mux/MUX_DECISIONS.md` D42.
