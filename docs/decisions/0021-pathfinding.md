# ADR 0021 — Pathfinding: a cost map, regions and A* with a memo

Status: draft (M3 Part 0); finalized at the end of M3.

## Context
Until M3 an agent stepped by the sign of the difference toward its target and ignored passability except where
worldgen avoided water and mountain (`AgentMovementSystem`, `fission.rs::walkable`). M3 adds walls, doors and dug
hill tiles (ADR 0022); the moment a wall exists, a walk that ignores it is wrong. The pathfinder must be
deterministic (C1), cheap at N = 1,000 (C3, budget in M3 DECISIONS D2), integer (C4) and invisible to the baseline
world (ADR 0014).

## Decision
- **Cost map** (`sim_core::nav`): one byte per tile, 0 = impassable. Impassable: river, mountain, undug hill, a wall
  without a door, a wall's construction site. Cost 10 for any other tile, 12 for a door; a diagonal step costs 14.
  It is derived from `TileGrid` (terrain, walls, `door`, `dug`) and the colony's 1×1 space sites, and kept current
  **incrementally**: the code that changes a tile (wall built or removed, door, dig finished, site placed or
  cancelled) sets that cell and bumps `nav_version`. A full rebuild from the same state gives the same map (tested).
- **Regions**: connected components of the passable tiles under the same move rule as the search, recomputed lazily
  once per `nav_version`. `reachable(from, to)` is one comparison (for an impassable `to`, any 8-neighbour of it in the
  same region). Job assignment offers only reachable targets; a search for an unreachable goal never runs.
- **A\***: 8 neighbours, integer costs, heuristic `10·max(dx, dy) + 4·min(dx, dy)` (octile; admissible and
  consistent for these costs), ties broken by (f, h, y, x); a diagonal step is allowed only when both orthogonal
  neighbours are passable (no corner cutting). A goal on an impassable tile is reached from any passable 8-neighbour.
- **Memo**: paths keyed by `(from, to)` for the current `nav_version`, at most 4,096 entries evicted in insertion
  order, cleared when the version changes. A memo hit returns exactly what a fresh search would, so the memo never
  changes behaviour.
- **Movement** (colony only): a seeking agent follows its stored path one step per move interval. When `nav_version`
  changes the remaining steps are re-validated and the path is kept if still passable; it is re-planned when its goal
  changes or a step is blocked. Every other colony step (idle wander, play, tether, haul home) is refused when it
  would enter an impassable tile or cut a corner. A job whose goal has become unreachable is cancelled
  (`CancelReason::Unreachable`) and the agent avoids that target for one day. The baseline keeps the sign step
  (identity).

## Alternatives considered
- Chebyshev heuristic `10·max(dx, dy)` (the goal's wording): also admissible, but it ties on a wide band of nodes on open
  ground; octile is tighter for the same integer costs and finds the same optimal cost.
- Re-planning every path on every `nav_version` change: a finished wall would trigger up to N searches in one tick.
- Flow fields for every destination: memory per destination; kept as the droppable cache for the most-used
  destinations (goal 6.4).
- Hierarchical pathfinding: not needed on a 192² map at the measured costs.

## Consequences
- The river, mountains and undug hills now split the map; agents live on the reachable side of their camp, and
  targets on the far side are never assigned.
- Paths and the version are saved; the cost map, regions and memo are derived and rebuilt on load.

## Date
2026-10-22

## Source
docs/v8/m3/M3_GOAL.md §3; docs/v8/m3/M3_DECISIONS.md
