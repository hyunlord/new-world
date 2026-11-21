# ADR 0081 — Style 2 is the default play style (INTEG1)

## Context

The Style 2 art track (ADRs 0074–0080: grade and detail, anchor, model kit, environment, house shells, house
shapes and light, motion capture) lived on the local branch `v8/s2kit` while `v8/main` gained MERGE1's HUD and mood
blend, ACTION1's visible actions and HOUSE1's houses and households. Roadmap v4 N1 is "S2 style integrated into the
main branch". The S2 reviews also left presentation problems: a z1 view that read as top-down, every roof opening in
close views, white hide piles, gable thatch that read as planks and a faceted cooking pot.

## Decision

- `v8/s2kit` is merged with a real merge commit. Simulation, bridge, UI and HUD behaviour follow `v8/main`; the 3D
  renderers and shaders follow Style 2 for the look and `v8/main` for correctness, hunk by hunk.
- Style 2 is the default. `Data3D.style2()` is the one switch; the user argument `--style=legacy` selects the previous
  look for one release as a regression fallback (`--style=s2` is still accepted). The legacy path keeps `v8/main`'s
  depth-gap x-ray, outline, far fill, MOOD1 grade, sky and terrain fragment; Style 2 keeps its stencil x-ray
  (`vat_xray_s2.gdshader`, `people.style2_outline` / `style2_xray`) and its `SLICE_PBR` terrain branch, which also
  draws `v8/main`'s footprints in snow and mud.
- One mood system: `sky3d.gd` keeps `v8/main`'s profile blend (night, dusk, day, hardship; zoom fade to the map
  light). The Style 2 grade is the default `grade` and `sky` data in that schema: the former `style2.day` and
  `house_lighting` values are folded into the day profile (sun colour, ambient, fog colour and density, contrast,
  saturation, LUT midtones), the warm midday becomes an optional `noon` profile, and the sky gains optional clouds,
  height fog, SSAO, shadow blur and aerial perspective. `--style=legacy` reads `legacy.grade` and `legacy.sky`, the
  unchanged `v8/main` values.
- Camera: the pitch above the horizon is data per zoom stage (`camera.pitch_by_zoom`: z1 48°, z2 40°, z3 34°, z4 28°,
  z5 22°; REVIEW-FIX-1), interpolated in log zoom so zoom tweens tilt smoothly. The Camera2D clamp uses the tilted view's ground
  reach (half width, ahead, behind) so the visible ground stays on the map.
- Roofs (Style 2): closed at every zoom. A roof opens with the existing cutaway (fade and lift) only for the
  selected house (whose walls also drop), the house hovered for `s2_house.hover_open_seconds` (0.3 s) and, while the
  inside view (HUD button, key V) is on, houses within `s2_house.inside_radius` tiles of the view focus. The room
  overlay (R) still opens every roof. Hidden villagers keep their silhouettes; selecting one opens the house they
  stand in.
- Near-tier mocap (ADR 0080) now chooses clips from ACTION1's visible-action rows and item cues: visual key, then
  fallback key, mapped through `params3d` `mocap.clip_for_visual`, then idle (walk while moving). ADR 0080's job-code,
  talker-tile and workshop heuristics are replaced; the elder log seat and wood-versus-generic carry stay.
- Houses in the inspector read only `get_house_snapshot` / `get_household_snapshot` (now SimReads tick getters): a house
  section in the room panel and a home line in the agent inspector that selects the house.
- The ADR numbers of the art track were renumbered after `v8/main`'s 0071–0073.

## Alternatives considered

Rebasing or squashing the art branch would hide its review history. Running the Style 2 lighting as a second layer
over MOOD1 (as `v8/s2kit` did) keeps two mood systems whose interaction nobody owns. Keeping the legacy look as the
default would leave the chosen style unseen by players. Opening every house within the focus on hover or with a
per-house wall cut would need a list of cut rectangles in the wall shaders; Style 2 walls are low, so the roof alone
is opened there.

## Consequences

Players see Style 2 by default; step 39 and its recorded hashes now measure Style 2. The legacy style stays
selectable until the next release removes it with its data (`mood.json` `legacy`, `people.outline`/`xray`, the
terrain `#else` branch). The Style 2 grade is an approximation of the former overlay: its day override now follows
the profile blend from dusk instead of its own 0.6–1.0 daylight ramp, and the per-hour sun elevation scale is replaced
by the noon elevation.

## Date

2026-11-21

## Source

`art-img-data/BRIEF_INTEG1.md` (planner, N1 close-out); conflict log `art-img-data/integ1/CONFLICTS.md`.
