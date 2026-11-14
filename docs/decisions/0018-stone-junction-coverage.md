# C3A Pass3: require solid connected stone junctions

## Context

BRIEF5 overturns the Pass2 visual acceptance: the5x4 inner corners and L-room
junctions exposed background. Nonempty masks and correct straight width did
not establish join completeness. Copying the pinned log alpha is insufficient;
its geometry also has partial side quadrants.

## Decision

Keep the unchanged native course field and fill complete connected side
quadrants on south-bearing corner/T/X masks6,7,12,13,14,15. A first attempt
restored only the22px face and was rejected because actual L/T/X holes remained.
The final coverage continues through row63; masks14/15 are solid junction
blocks. Unconnected-side floor remains transparent. Straight dimensions and
all ten unaffected masks remain unchanged. No texture is stretched or blurred.

Require failing-first coverage tests for all9 corner/junction masks, a T+X plan
with all four Ts, and26 actual room join contexts at nearest4x, beside pinned
log references. Retain intermediate failures externally and supersede the old
art pins, without changing game configuration or unrelated art.

## Alternatives considered

- Fill only rows26..37: rejected by visible L/T/X squares after the first repair.
- Fill every mask's entire tile: rejected because it destroys unconnected floor
  and straight26px shafts.
- Repaint or resize stone: rejected because the fault is missing geometry.
- Reuse log alpha as a truth mask: rejected because it retains partial quadrants.

## Consequences

Connected corner/junction coverage is deliberately fuller than the straight
wall shaft. Metadata distinguishes the two. The old Pass2 approval is historical;
new4x views and full-coverage tests govern acceptance. Runtime occupancy, mixed
materials, construction states and site-ghost integration remain separate.

## Date

2026-11-14

## Source

`art/batch_c3a/walls_pass2/native.py`, `test_native.py`, `review_boards.py`;
external C3A/pass3/DIAGNOSIS.md and BRIEF5.md.
