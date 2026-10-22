# ADR 0021 — Pathfinding: a cost map, regions and A* with a memo

Status: accepted (M3, 2026-10-22).

## Context
Until M3 an agent stepped by the sign of the difference toward its target and ignored passability except where
worldgen avoided water and mountain (`AgentMovementSystem`, `fission.rs::walkable`). M3 adds walls, doors and dug
hill tiles (ADR 0022); the moment a wall exists, a walk that ignores it is wrong. The pathfinder must be
deterministic (C1), cheap at N = 1,000 (C3, budget in M3 DECISIONS D2), integer (C4) and invisible to the baseline
world (ADR 0014).

## Decision
- **Cost map** (`sim_core::nav::CostMap` in `SimResources.nav`): one byte per tile, 0 = impassable. Impassable:
  river, mountain, undug hill, a wall without a door, a wall's construction site (from placement). Cost 10 for any other
  tile, 12 for a door; a diagonal step costs 14/10 of its tile. `runtime::nav::tile_cost` is the one definition; the
  code that changes a tile calls `nav::refresh`, which bumps `nav.version` when the cost moved. A full rebuild from the
  same state equals the incremental map (tested).
- **Regions**: connected components under the search's own move rule, recomputed lazily once per version. A request
  whose goal lies in another region returns `None` without searching; the job assignment offers only targets reachable
  from the agent's region (`NavState::reaches`) and keeps its empty-search memos per region.
- **A\***: 8 neighbours, integer costs, octile heuristic `10·max + 4·min` (admissible and consistent for these
  costs; the goal's `10·max` Chebyshev would be too, but ties on a wide band), ties `(f, h, y, x)`, no corner cutting.
  A goal on an impassable tile is reached from any passable 8-neighbour (jobs only: a decision consumes standing on its
  tile). Someone standing on an impassable tile may step off onto any passable neighbour.
- **Memo**: paths by `(from, to)` for the current version, ≤ 4,096, insertion-order eviction, dropped on a version
  change; a hit returns exactly what a search would, so the memo changes no behaviour and is not saved.
- **Movement** (colony only): seeks and haul-home walks follow a stored path (`nav.paths`, saved); on a version change
  the remaining steps are re-checked and kept if walkable, otherwise re-planned. Idle wander, play, the tether pull and
  the evening hall are refused into impassable tiles and corners. A job whose goal turns unreachable is cancelled
  `Unreachable` (one-day cooldown for that person and target); a decision target out of reach or not standable moves to
  the nearest reachable tile of its kind, or the walk ends. The baseline keeps the sign step (identity PASS every Part).

## Alternatives considered
- Chebyshev heuristic `10·max(dx, dy)`: same optimal cost, more ties expanded.
- Re-planning every walker on every version change: a finished wall would trigger up to N searches in one tick.
- Flow fields for every destination: memory per destination. The droppable cache (goal 6.4) was not needed: the
  budget held with regions and the memo (M3 DECISIONS D2, `nav_bench`).
- Passable wall sites (build first, block later): no deadlock from a box across a corridor, but a builder could end
  inside a finished wall; kept the goal's rule and made the script resolver avoid cutting the camp off (D14).

## Consequences
- The river, mountains and undug hills split the map; colony worldgen moves founders off impassable ground (D9) and
  builds the nav map at tick 0.
- The decision system's own selectors (untouched, ADR 0013) can still pick a far-side bush; movement retargets or ends
  that walk, so a forager near the river can loop (a recorded limitation, D10).
- Paths and the version are saved (format 3); the cost map, regions and memo are rebuilt on load.

## Date
2026-10-22

## Source
docs/v8/m3/M3_GOAL.md §3.1, §4; docs/v8/m3/M3_DECISIONS.md D2, D7–D10, D14; rust/crates/sim-core/src/nav.rs;
rust/crates/sim-systems/src/runtime/nav.rs; rust/crates/sim-systems/src/runtime/agent/movement.rs
