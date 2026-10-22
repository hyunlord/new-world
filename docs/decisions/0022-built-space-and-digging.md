# ADR 0022 — Built space and digging: walls, doors, floors, bedrolls and dug hill tiles

Status: accepted (M3, 2026-10-22).

## Context
ROADMAP M3: "언덕을 파서 만든 방이 창고가 되고, 사람들이 그리로 나른다." `TileGrid` has carried `wall_material` and
`floor_material` since V7 Material Phase 2, unused by the colony. M3 lets the player build walls, doors, floors and
bedrolls from items (ADR 0019) and dig into hills for stone; these tiles change passability (ADR 0021) and enclose
rooms (ADR 0023).

## Decision
- Blueprint kinds, each a 1×1 site with its own `MaterialNeeds`: `LogWall` (log 2, work 40), `StoneWall` (stone 3,
  work 60), `Door` (log 1, work 30: a wall tile with `TileGrid.door`, passable at cost 12, bounding rooms), `PackedFloor`
  (work 15), `Bedroll` (log 1, work 10: one bed; hides have no source yet). Gated on `ColonyRule::space`
  (`SpaceRule`; the colony world's rule is `ColonyRule::M3`).
- Placement: `PlaceBlueprint`, `PlaceLine` (Bresenham), `PlaceRect { hollow }`; every tile passes or fails
  `commands::check_space` on its own (`OutOfMap`, `Water`, `Hill`, `Built`, `Blocked`, `Person` for walls). Items on the
  tile do not refuse a site: a `Clear` job (work type Build) carries them off first, and the site is ready only when its
  tile is clear.
- Building: the job layer works a space site from beside it (a wall site is impassable) or on it, advancing the
  site's progress one step per tick; `jobs::space::complete_site` writes the tile (the delivered material becomes the
  wall's), consumes the materials and despawns the site. A V7 builder finishing a door or floor site in place reaches
  the same function through `ConstructionSystem` (no building entry).
- `Deconstruct` marks: bedroll, then wall or door, then packed floor; half the plan's materials (rounded down) come
  back in the piece's material.
- `Dig` marks on undug hill off the map edge; `JobKind::Dig` (`WorkType::Mine`, `SkillKind::Mining`, 150 ticks at
  level 5) is offered on the frontier only (the region filter needs a passable neighbour); a dug tile stays `Hill`, gets
  `TileGrid.dug`, a granite floor and 3 granite stone; a boulder on it goes with it.
- Every change calls `nav::refresh` and marks the rooms dirty.

## Alternatives considered
- Walls built by the V7 co-located builder flow: the builder would stand in the wall it finishes.
- Refusing a site on items: the goal also asks for a haul-away; a drag over a busy tile should work.
- Separate wall and door entities: the tile grid already has the columns, and renderers read tiles.

## Consequences
- The colony's material registry stays empty (M2 D4), so walls do not block warmth or danger; room temperature is
  not in M3.
- Ledger: wall, door and bedroll materials are consumed at completion, deconstruction creates half again, digging
  creates stone (per-kind ledger exact in every test, soak and evaluation run).
- A box of walls placed across a corridor can cut a colony off (its sites block at once); the player sees it, the
  evaluation's resolver avoids it (M3 DECISIONS D14).

## Date
2026-10-22

## Source
docs/v8/m3/M3_GOAL.md §3.2; docs/v8/m3/M3_DECISIONS.md D11, D14; rust/crates/sim-systems/src/commands.rs;
rust/crates/sim-systems/src/runtime/jobs/space.rs; rust/crates/sim-test/tests/colony_space.rs
