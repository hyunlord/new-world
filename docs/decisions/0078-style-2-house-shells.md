# 0078. Style 2 template house shells

## Context

At village zoom, the walls remained visible as individual tile pieces. The
building catalogue and saved site rows already define the actual template
instance, its footprint, and its construction state.

## Decision

Style 2 draws one shell per intact roofed template site. It caches the rotated
piece plan from the existing placement preview and keeps independent wall and
fence orders as open pieces. Authored round roofs produce conical thatch on a
low earth ring. Angular rectangles use long-axis thatch gables and stone
footings. The roof mesher can draw hips and valleys over an irregular exterior
tile mask. Only a render fixture supplies L and U masks.

Reuse the room and cutaway controls. Selecting a house focuses the cutaway
on that house, including at z4 and closer. At close zoom with no focus, the
existing full cutaway remains available. The Inside view button and I key
activate the view at wider zoom. Construction combines completed pieces
and measured physical work from the existing snapshots.

Keep early travel wear grassy and preserve the grass instance budget. Warmer
daylight falls back to the shared twilight and night colours.
Raise the existing Style 2 night silhouette value so villagers remain
readable against pale thatch.

## Alternatives considered

Roofing all enclosed wall groups would also cover independently placed walls
and would split one template across its rooms. Expanding the 3D bridge
would duplicate footprint, placement and work data already available to Godot.

## Consequences

The same simulation, snapshots, save files and construction commands drive
both render styles. A renderer-only fixture exercises the irregular roof
mesher while the authored templates keep their existing rectangular footprints.

## Date

2026-11-20

## Source

`BRIEF_S2HOUSE.md` and [ADR 0077](0077-style-2-environment.md).
