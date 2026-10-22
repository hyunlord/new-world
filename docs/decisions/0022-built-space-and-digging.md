# ADR 0022 — Built space and digging: walls, doors, floors and dug hill tiles

Status: draft (M3 Part 0); finalized at the end of M3.

## Context
ROADMAP M3: "언덕을 파서 만든 방이 창고가 되고, 사람들이 그리로 나른다." `TileGrid` has carried `wall_material` and
`floor_material` since V7 Material Phase 2, unused by the colony. M3 lets the player build walls, doors and floors
from items (ADR 0019) and dig into hills for stone, and these tiles must change passability (ADR 0021) and enclose
rooms (ADR 0023).

## Decision
- New blueprint kinds, each a 1×1 site with its own `MaterialNeeds`: `LogWall` (log 2, work 40, wall of the log's
  material), `StoneWall` (stone 3, work 60, droppable), `Door` (log 1, work 30: a wall tile with `TileGrid.door`,
  passable at cost 12, bounding rooms like a wall), `PackedFloor` (no material, work 15: `floor_material =
  packed_earth`), `Bedroll` (hide 1 or log 1, work 10: one bed, ADR 0023).
- Drag commands `PlaceLine { kind, x0, y0, x1, y1 }` (Bresenham) and `PlaceRect { kind, x0, y0, x1, y1, hollow }`
  place one site per valid tile. A wall site is impassable from placement; it is refused on a tree, a resource node, a
  building, a site, a zone or a person. Items on the tile do not refuse it: they stay and a hauler carries them off
  (a haul-away job) before the wall can be built.
- Space sites are built from any 8-neighbour (a wall site cannot be stood on) by the job layer, which advances the
  site's progress itself; `ConstructionSystem` finishes a space site with the same function when a V7 builder works one
  in place.
- `Deconstruct` designations on walls, doors and floors: a job returns half the materials (rounded down) as items on
  the tile and clears it.
- `Dig` designations on undug hill tiles 8-adjacent to a passable tile and not on the map edge: `JobKind::Dig`
  (`WorkType::Mine`, 150 work ticks scaled by the new `Mining` skill); the tile keeps terrain `Hill`, gets
  `TileGrid.dug`, becomes passable with `floor_material = rock`, and yields 3 stone of the hill's rock material.
  Undug hill around a dug pocket bounds it like a wall.
- Every change of a wall, door, floor or dug flag updates the nav cost map incrementally and marks the rooms dirty.

## Alternatives considered
- Walls built by the V7 co-located builder flow: a builder would stand inside the wall it finishes.
- Refusing a wall site placed on items (the goal's first sentence): the goal also asks for a haul-away job; keeping the
  site and hauling the items off is what the player expects from a drag over a busy tile.
- Separate wall and door entities: the tile grid already has the columns, and the renderer reads tiles.

## Consequences
- The colony's material registry stays empty (M2 D4), so walls do not block warmth or danger yet; setting
  `wall_material` is the one per-tile write the influence code reads. Room temperature is not in M3.
- Ledger: wall, door and bedroll materials are consumed at completion; deconstruction creates half of them again;
  digging creates stone.

## Date
2026-10-22

## Source
docs/v8/m3/M3_GOAL.md §3; docs/v8/m3/M3_DECISIONS.md
