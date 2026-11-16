# ADR 0068 — M3D-SWITCH: the 3D view is the only world view

## Date
2026-11-15

## Source
Owner decision 2026-11-15 (planner goal `GOAL_3D_SWITCH.md`): switch fully to 3D now. Supersedes the part of ADR 0066
that kept the 2D renderer as the default and the 3D view behind `--renderer=3d`. Keeps ADR 0016 (the view never changes
the sim) and ADR 0033 (kinds are data).

## Context
ADR 0066 added a 3D view beside the 2D renderer. It still hit-tested people, animals, visitors and rooms through the
hidden 2D layers, and every pixel or frame-time gate check (renderer agreement, the M-ART2 render tests, the MUX2
visibility, stall and why receipts) measured the 2D renderer. Keeping two renderers doubles every visual change and
leaves the gate checking a view the player no longer sees.

## Decision
- `World3D` is always made by `main_bootstrap.gd`; `renderer_switch.gd`, the `renderer_3d` setting and every 2D world
  layer (ground, trees, buildings, plants, fields, livestock, outcrops, agents, animals, visitors, overlay, rooms, LOD
  markers, daylight) and their shaders are deleted. The 2D *UI* stays.
- `ColonyView` stays as a hidden container for `OrderTool` (orders, no drawing) and `WorldMarker`, so tools and panels
  keep their node paths.
- Data lookups that lived in 2D layers move to renderer-neutral nodes: `Main/RoomIndex` (rooms, R key) and
  `Main/PeopleView` (mood flag, M key, chronicle highlights, person look, household keys).
- `ViewProjection` is 3D only: `pick`/`pick_world` return final ids (person bits, animal id, caravan or wave) from
  World3D's body hit-tests; `person_rect` and `people_lod` serve camera framing and lesson highlights.
- On-map text and bars (site progress, hover labels, room labels, people marks) are drawn in screen space by
  `scripts/world3d/labels3d.gd` with the 2D wording and keys.
- Crowds: people are vertex-animated MultiMeshes binned by model and 16-tile chunk (per-chunk culling); each bin draws
  the full set B character near the camera and a ≤ 400-column LOD bake beyond `people.lod_distance` (visibility ranges).
- Gate steps that measured the 2D renderer now measure the 3D view at the same thresholds: 23 renderer agreement
  (People `state_of`), 34 `w3d_render` + `w3d_labels` (pinned terrain chunk hashes, dig rebuild, data-only wall, people
  model/look, tree seasons, label texts), 36 (World3D ore), 38 (camp framing and validators), 39 visibility
  (per-person removal from the 3D crowd), 41 stall (World3D per-layer timings), 46 why (through ViewProjection),
  48 (plants3d). Checks of deleted 2D-only code (badge packing, wall-bake caches, ground tone caps) are dropped.

## Alternatives considered
- Keep both renderers with 2D as a fallback: every visual change and gate check twice, and the gate would keep
  measuring a view the player does not use. Rejected by the owner decision.
- Move to Unity now: evaluated in parallel (UNITY-SPIKE) against the criteria in `GODOT_BLOCKERS.md`; none of them
  had fired when this switch landed, so the 3D view stays in Godot.
- Reparent OrderTool and WorldMarker under `Main`: cleaner tree, but every tool and panel path changes; the hidden
  `ColonyView` container keeps them stable at no cost.

## Consequences
- One renderer to change and to gate. The 3D view needs a GPU for pixel checks; headless runs cover data, meshes and
  label texts.
- Godot friction met on the way is tracked in `3d-data/GODOT_BLOCKERS.md` (no Metal GPU timings, no per-instance
  MultiMesh culling, GDScript per-frame cost).
- 2D-only art under `assets/v8art` is kept for now (the UI icons, portraits and site ghosts still use parts of it);
  pruning it needs a manifest regeneration and is left for a later pass.
