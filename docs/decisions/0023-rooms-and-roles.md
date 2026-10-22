# ADR 0023 — Rooms and roles

Status: draft (M3 Part 0); finalized at the end of M3.

## Context
V7's signature RimWorld-style space: walls and furniture define a room's role, and the role changes behaviour. M3
must recognize rooms the colony builds (ADR 0022), give them a role automatically, let the player override it, and
make the role matter for sleep and storage.

## Decision
- **Detection** (`sim_core::rooms`, `RoomSystem` priority 152, every 60 ticks and at once when a wall, door or dig
  changed): flood fill over passable tiles, bounded by walls, doors and undug hill (also water and mountain). A region
  is a room when it is enclosed — it does not touch the map edge and has at most 120 tiles (a region of more than 400
  tiles is open ground) — and holds at least 2 tiles. Doors belong to no room; a room lists the doors on its border.
- **Stable ids**: a new room takes the id of the previous room it overlaps by at least 50 % of its tiles (largest
  overlap, then lowest id), else a fresh id; the key tile is the room's lowest `(y, x)` tile.
- **Roofed** = enclosed with at most 120 tiles (the enclosure is the roof; M3 has no roof item). A dug pocket is roofed
  by the hill (`dug = true` when any of its tiles is dug).
- **Roles** (recomputed with detection; `SetRoomRole { room, role }` overrides and is recorded): Workshop if it holds a
  workshop; Hall if it holds the hearth; Storehouse if ≥ 50 % of its tiles are stockpile cells; Bedroom if it holds a
  bed and no workshop or stockpile cell; else Unassigned.
- **Effects**: bed assignment prefers beds in a roofed bedroom and never assigns a bed its sleeper cannot reach;
  sleepers in a roofed room take no night cold; sleep recovers 20 % faster in a roofed bedroom (sleep decay ×800 ‰);
  hauling prefers stockpile cells inside a roofed storehouse (a bonus worth 16 tiles of distance) and, while a
  storehouse exists, `Maintain` bills count only the stock under a roof.

## Alternatives considered
- A roof item and a temperature model per room: not in M3 (§12); the enclosure stands in for the roof.
- Roles only by player choice: the goal wants the colony to recognize rooms itself; the override covers the rest.

## Consequences
- Room ids survive small edits (one wall moved), so overrides and the chronicle's `RoomFormed` stay attached.
- The cost of detection is a flood fill of the colony's built area when it changes, not per tick.

## Date
2026-10-22

## Source
docs/v8/m3/M3_GOAL.md §3; docs/v8/m3/M3_DECISIONS.md
