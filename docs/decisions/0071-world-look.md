# ADR 0071 — Grade and detail the village in the existing 3D view

## Context

The owner asked for subdued natural daylight, blue-grey dusk and night, and a
colder look during hardship. Large flat surfaces and a visible tile grid made
the landscape look like clay at closer camera angles.

## Decision

Use one Environment. Blend three colour-correction curves and light profiles
over the existing clock's dawn/dusk windows. Read the UI food runway, recent
human deaths and season through the existing bridge and smooth hardship over
simulation time. Select warm point lights by camera distance from the emissive
windows and fire sockets of occupied roofed rooms.

Reuse the padded material atlas. Blend the surface lookup in world space and
retain the simulation's distinct path stages. Use chunked grass MultiMeshes
and leaf-card shells, with surface masks, camera-based density and a fixed
instance budget. Share tuning for the grade and world detail in
`assets/3d/mood.json`.

## Alternatives considered

- A post-process screen quad would add a screen read and would need separate
  handling of HUD compositing.
- One grass node per tuft would increase scene and draw overhead.

## Consequences

The grade consumes simulation snapshots and time without modifying the
simulation. Window and hearth visibility follows current occupancy. Grass
density, atmospheric settings, and light visibility have native GPU acceptance
criteria.

## Date

2026-11-18

## Source

Owner briefs `BRIEF_MOOD1.md` and `BRIEF_DETAIL1.md`, and the 2026-11-18
instruction to implement both on the same branch before one combined GPU run.
