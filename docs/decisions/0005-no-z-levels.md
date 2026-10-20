# ADR 0005 — No z-levels: dig hills on the plane; deep layers as mine buildings

## Context
Dwarf Fortress uses z-levels; WorldSim's simulation and renderer are 2D.

## Decision
Stay 2D. Hill bedrock on the same plane is dug directly to make rooms; deep layers (L1–L4) are abstracted as mine buildings plus chunked strata data, gated by tool grade (summary of the owner's 'WorldSim_지하자원_4자합성' design).

## Alternatives considered
Full z-levels (rejected: cost in simulation, rendering and UI).

## Consequences
Room recognition (M3) works on one plane; mining depth is data, not geometry.

## Date
2026-10-20

## Source
docs/VISION.md §3
