# ADR 0075 — Keep the Style 2 anchor opt-in

## Context

The local anchor trial compares a subdued, simplified realistic look with the
existing game on the same Slice 1 save. The assets must be reproducible with
existing inputs.

## Decision

Activate the anchor through Godot user argument `--style=s2` on the normal
World3D path. Reuse the Tripo woman's 17-bone rig and the existing VAT format,
and keep the authored hide colours through the crowd renderer. A one-metre
gait cycle follows rendered travel distance. The far crowd uses reduced
geometry and a single VAT sample. Two existing CC0 tree models share the
muted palette and switch to wind-animated crossed foliage cards at distance.

Use the current modular walls and roof with softened log and thatch textures.
The existing roof cutaway remains the room-view mechanism. The grade uses
MOOD1 R4 zoom endpoints and the existing m2 lighting, with adjusted grass
cards and low-contrast terrain layers.

## Alternatives considered

The existing Tripo log house replaces an entire building with one opaque
mesh. It has no separately addressable roof or interior. The modular house
provides both for this trial.

## Consequences

This is a local render trial. All villager roles share the anchor mesh while
style 2 is enabled. The individual wardrobe and model rollout requires
additional modelling. Captures, performance samples and the step 39 receipt
live outside Git in the anchor output directory.

## Date

2026-11-19.

## Source

Owner brief `/Users/rexxa/work/art-img-data/BRIEF_ANCHOR2.md`,
`docs/v8/slice1/README.md`, and MOOD1 R4 `3f0775448`.
