# ADR 0010 — The god hand is removed; a storyteller comes in M5

## Context
The seven god-hand interventions (Pass 7) were the player's main tool in the god game.

## Decision
V8-0 removes the god-hand panel, its scene and effects, the bridge methods `apply_intervention` / `get_intervention_state`, `sim-engine/src/intervention.rs`, `sim-systems/src/runtime/intervention.rs` and the intervention-only state. Natural runs never intervened, so the simulation is unchanged (identity verified). Pressure returns in M5 as storyteller events chosen by the system (cold snaps, droughts, animals), built from scratch.

## Alternatives considered
Keep the god hand as a debug tool (rejected: the owner removed it from the player's tools, and the storyteller has different rules).

## Consequences
The removed code is available at `v7-final-art` / `v7-final-sim`.

## Date
2026-10-20

## Source
docs/VISION.md §1, §5
