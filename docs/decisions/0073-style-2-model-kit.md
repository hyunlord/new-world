# ADR 0073 — Style 2 model kit

## Context

ANCHOR2 established the optional renderer mode. S2KIT extends it with the owner's
12 Tripo models and the existing day-120 comparison fixture.

## Decision

The renderer selects the kit only with `--style=s2`. People use the existing
snapshot ages and sexes, a shared ANCHOR2 17-bone rig and clip timing, and three
vertex-animation texture tiers. Static models keep their source UV maps and
use a smaller, palette-matched albedo. The renderer keeps the ANCHOR2 wall,
door and roof geometry where the kit cannot carry the existing topology or
interaction.

## Alternatives considered

The closed door model fused the door to its wall. Keeping the existing
animated opening preserves passage visibility and the cutaway. Irregular roof
footprints and connected wall junctions still need the existing geometry.

## Consequences

The local branch targets renderer comparison, with the simulation and bridge
shared by both arms. Source models and evidence are stored outside the checkout.

For regeneration, run `s2kit_prepare.py` with the art-data directory, then
`s2kit.py` with its `s2kit` subdirectory and the model keys `p1 p2 p3 p4 h1 h2
h3 x1 x2 x3 x4 x5` through the shared GPU lock. Extract `template.glb` in the
output directory from the pinned `anchor2_woman.py` source revision at
`assets/3d/models/tripo_woman_lod0.glb`.

## Date

2026-11-20

## Source

`BRIEF_S2KIT.md`, `tools/3d/blender/s2kit.py`, and [ADR 0072](0072-style-2-anchor.md).
