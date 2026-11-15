# ADR 0067 — Content-indexed 3D asset delivery

## Context

M3D-ASSETS supplies the independent M3D-SWITCH renderer with source-generated GLBs.
The existing M3D-1 pack has generic item shapes and procedural walls; later C3
content includes iron production and cut-stone construction. The requested
`origin/v8/c3` ref was unavailable at inventory time. The frozen baseline and
local C3 supplement are recorded explicitly in `assets/3d/INVENTORY.json`.

## Decision

- Keep the existing M3D-1 model IDs intact. Deliver additional models through
  separate manifest fragments with content identities, state or stack metadata.
- Author static geometry with the existing Blender primitive/export helpers:
  metres, feet origin, Godot +Z forward, existing material names and flat shading.
- Generate display construction previews from template geometry. Foundation and
  half-built labels describe visual progress previews, not new simulation states.
- Keep the before-work inventory distinct from delivered model coverage. An
  unavailable authoritative ref remains a documented gap rather than a substitute
  ref being reported as authoritative.
- Keep captures and raw validation evidence outside the repository. Family sheets
  use the game camera and explicit 1×/2× scale beside M3D-1 references.
- Living models retain their actual source and rig provenance; static procedural
  substitutes must never be described as TRELLIS output.

## Alternatives considered

- Add content-specific branches to the renderer: rejected; integration should
  resolve identities through data, and this job does not own the renderer.
- Replace all baseline models: rejected; independent switch integration needs
  stable IDs and the original models for visual comparison.

## Consequences

The asset branch can be consumed in batches without changing simulation data.
Renderer integration, runtime selection of state/stack variants and acceptance
of the missing remote content ref are distinct from asset-file validation.

## Date

2026-11-15

## Source

Owner job brief `/Users/rexxa/work/3d-data/BRIEF_M3D_ASSETS.md`; ADR 0066;
`assets/3d/INVENTORY.md`.
