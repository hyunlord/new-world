# 0079. Style 2 house shapes and afternoon lighting

## Context

The first Style 2 shell showed several square timber buildings as conical huts
because their catalogue rows carried a legacy Round art hint. The round roof,
cutaway floors and tile-lattice roof valleys also needed clearer materials.

## Decision

Derive the shell from the rotated site plan. Use the round silhouette for a
compact shelter with a Round hint and a usable interior. Use a gable on other
rectangles and intersecting planar hips over irregular footprints. Use the same
shape during work and after completion. Retain the wall materials from the
actual built pieces.

Build an equivalent shell for a qualifying constructed room outside the
template system. Use its mapped tiles and wall rim. Select it through the
existing room panel, with the same fade and zoom rules as a template shell.

Fit the Tripo pit-house berm and roof as separate parts in one uniform
transform so the berm survives cutaway. Use projected Tripo thatch and wood
maps with the modular gables and hips. Fit the authored roof and wall details
to each shell. Repeat the shelter post bay along the ridge. Build the straw
courses and vent holes with the geometry.

Draw occupied interiors through the normal people, bed, station and item
renderers. Mark the completed site's interior as sheltered packed earth in the
terrain presentation. Place dropped items at the top of a built floor slab
where one covers the terrain.

Tune Style 2 contact occlusion, the soft 35-degree daylight key, aerial
perspective and warm midtones in `assets/3d/mood.json`. Keep the existing glow.
The lighting trials added one setting at a time to the retained profile.
`WORLDSIM_S2HOUSE2_LIGHTING` reproduces those comparisons using a comma-separated
list of effects or `none` for the pre-trial lighting.

## Alternatives considered

A single fitted kit roof could cover a rectangle, but could not represent
both a part-built roof and a vent at the right position. A tile-distance
height field left visible steps at re-entrant corners.
Screen-space indirect lighting and added bloom cost frames in the live
500-person scene and brought little visible improvement.

## Consequences

The default renderer uses its existing path. The simulation, bridge fields,
template rows and save format are shared by both styles. L and U footprints
have renderer-only fixtures.

## Date

2026-11-20

## Source

`BRIEF_S2HOUSE2.md` and [ADR 0078](0078-style-2-house-shells.md).
