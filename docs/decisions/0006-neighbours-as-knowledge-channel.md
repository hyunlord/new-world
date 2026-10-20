# ADR 0006 — Neighbours are the channel of knowledge

## Context
Ten isolated founders lose complex technology (Henrich's critical population). Dwarf Fortress (seasonal migrant waves) and RimWorld (storyteller-scheduled trader visits with faction checks) were studied.

## Decision
Neighbour settlements exist from the start and run off-screen as aggregate simulations. v1.0: DF-style migrant waves (first two fixed at 2–6 people, then scaled by wealth, reputation and food surplus, with a population cap) that bring their own knowledge, and seasonal visiting traders (RimWorld-style candidate dates + relation check) whose goods give technology hints. v1.x: relation values (V7 village relation rules rewritten per faction), raids, player-sent trade expeditions, marriage exchange.

## Alternatives considered
No neighbours in v1.0 (rejected: isolated groups lose skills); full diplomacy in v1.0 (deferred).

## Consequences
The V7 inter-village relations code (porters, aid, disputes, estrangement) is kept but scaled down to relations inside one settlement until v1.x.

## Date
2026-10-20

## Source
docs/VISION.md §1, §10.2
