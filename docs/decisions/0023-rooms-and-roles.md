# ADR 0023 — Rooms and roles

Status: accepted (M3, 2026-10-22).

## Context
V7's signature RimWorld-style space: walls and furniture define a room's role, and the role changes behaviour. M3
must recognize rooms the colony builds (ADR 0022), give them a role automatically, let the player override it, and
make the role matter for sleep and storage.

## Decision
- **Detection** (`sim_core::rooms::detect`; `RoomSystem` priority 152, at once when a wall, door or dig changed —
  `colony.rooms_dirty` — and roles re-read every 60 ticks): a 4-connected flood fill bounded by walls, doors, undug hill,
  water and mountain (a wall site does not bound). A room has 2–120 tiles, does not touch the map edge, and was made by
  the colony: a built wall or door on its border or a dug tile inside (a meadow ringed by hills is not a room). Doors
  belong to no room; a room lists its border doors.
- **Stable ids**: a room may keep the id of an old room holding ≥ 50 % of its tiles; such claims are granted largest
  overlap first (ties: lower old id, then the earlier new room), each id once, so a split leaves the id with the larger
  part (M3 DECISIONS D19); overrides follow the id. Every room is roofed (the enclosure is the roof); a room with a dug tile is `dug`.
- **Roles**: Workshop (a workshop in it) > Hall (the hearth) > Storehouse (≥ 50 % stockpile cells) > Bedroom (a bed,
  no stockpile cell) > Unassigned; `SetRoomRole { room, role }` overrides (`None` = automatic). `RoomFormed` is logged
  once per room.
- **Effects**: beds are offered roofed-bedroom bedrolls first, then huts, then other bedrolls, and never a bed the
  settlement cannot walk to; a sleeper in a roofed room takes no night cold; asleep in a roofed bedroom fatigue grows at
  800 ‰; hauls go to a roofed storehouse cell unless it is more than 16 tiles farther than the nearest cell, and with no
  loose stack haulers restock open-air stock into it; with a storehouse, `Maintain` bills count only the stock under its
  roof; a workshop in a roofed workshop room crafts at 900 ‰ ticks; idle steps in the evening window stay in the hall.

## Alternatives considered
- A roof item and a per-room temperature model: not in M3 (§12); the enclosure stands in for the roof.
- Roles only by player choice: the goal wants the colony to recognize rooms itself; the override covers the rest.
- Natural enclosures as rooms (the goal's literal rule): hill-ringed meadows became roofed "rooms" on most seeds.

## Consequences
- Room ids survive small edits (one wall moved), so overrides and the chronicle stay attached.
- Detection costs one flood fill of the map when built space changes, not per tick.

## Date
2026-10-22

## Source
docs/v8/m3/M3_GOAL.md §3.3; docs/v8/m3/M3_DECISIONS.md D12; rust/crates/sim-core/src/rooms.rs;
rust/crates/sim-systems/src/runtime/rooms.rs; rust/crates/sim-test/tests/colony_rooms.rs
