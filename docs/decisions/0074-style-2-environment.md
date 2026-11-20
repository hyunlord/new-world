# 0074. Style 2 ground, understory and walk skinning

## Context

The S2KIT village kept the beige ANCHOR2 ground and visible flat wild-plant
cards. Walking characters showed folds above their boots in the swing phase.

## Decision

Keep the environment changes behind `--style=s2`. Tint the existing local
terrain textures with broad, muted meadow colours. Reuse the renderer's path,
hearth, door and workshop positions to wear the ground, and share the mask
with the DETAIL1 grass batches. Add a denser, capped grass profile and use
Slice 1 ferns for wild plant and bush silhouettes. Reuse the ground-cover
chunks for tall bank reed clumps and small stones on water-adjacent tiles. Bring the daylight sun
slightly lower and use a warmer ambient fill, fading the change before dusk.

Centre the knee and ankle weight bands on the rest joints of the 17-bone rig
and bake the four sources at all three detail levels. Increase the opacity of
Style 2’s existing distant night silhouettes so the reclined figures remain
readable against roofs.

## Alternatives considered

A second vegetation pass would add draw calls and duplicate the camera budget
already maintained by the DETAIL1 grass batches.

## Consequences

The render profile changes the presentation of the same save and snapshots.
The same terrain and grass shaders still serve the default style.

## Date

2026-11-20

## Source

`BRIEF_S2ENV.md` and [ADR 0073](0073-style-2-model-kit.md).
