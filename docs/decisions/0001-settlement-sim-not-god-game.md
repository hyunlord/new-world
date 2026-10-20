# ADR 0001 — V8 is a settlement sim, not a god game

## Context
Through V7 WorldSim was a god game: the player watched villages and pressed intervention buttons (the 'god hand': gift/spoil food, gift/rot wood, rain, cold snap, windfall). The owner found that pressing buttons and watching becomes monotonous quickly and gives the player no goals.

## Decision
WorldSim becomes a Dwarf Fortress–style settlement sim: the player gives orders (dig, chop, build, zones, work priorities, craft orders) and people decide for themselves how to carry them out. Starting around 10 founders, growing to 30–200, with an architecture that can scale to thousands. Technology is owned by people and spreads between them; eras are observed from how widely marker technologies have spread.

## Alternatives considered
Keep the god game and add goals (rejected: the core loop stays passive); RimWorld-style colony sim with a storyteller as the main driver (partly adopted: the storyteller returns in M5 as pressure, not as the player's tool).

## Consequences
The god-hand panel and intervention API are removed (ADR 0010). New gameplay (orders, items, rooms, knowledge) is built from M1. The simulation core (Rust ECS, determinism, temperament, families, chronicle, causal 'why?') is kept and reconnected to the new loop.

## Date
2026-10-20

## Source
docs/VISION.md §0–§2, §5, §7 (docs/v8/V8_DIRECTION.md §1)
