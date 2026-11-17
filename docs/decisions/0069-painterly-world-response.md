# ADR 0069 — Shared painterly response for the world

## Context

The owner chose first-party models with the warm painterly look of the generated UI art.
L1 owns rendering, terrain, vegetation and water; L2 owns model detail. Simulation state,
model ids and shared skeletons must remain compatible with concurrent work.

## Decision

Share `shaders/world3d/painterly.gdshaderinc` for wrapped diffuse light, restrained lit rims
and broad pigment variation across terrain, flora, plants, walls, roofs, VAT and props.
Keep existing x-ray, emission, alpha, overlays and cutaways. Smooth terrain normals on a
shared lattice without changing heights, stand positions or snapshot colour arrays.
Reuse the first texture pass's atlas for triplanar ground and imported surfaces; cache
adapted consumer meshes while retaining original materials for seasonal recolouring.

Weld leaf normals on the existing overlapping canopy lobes. Decorate only existing wild
plant tiles with deterministic grass/flower MultiMeshes, rooted shader wind and detail
fade. Use sky ambient, AgX, a monotone 16³ colour LUT, soft shadows and subtle depth fog.
Water combines depth tint, slow anisotropic noise, a broken shore band and sky reflection.

## Alternatives considered

- Third-party packs are excluded by the owner's production decision.
- SDFGI/SSIL and tilt-shift are omitted; the required crowd FPS floors take priority.
- Global material replacement would break Flora/Plants source-colour contracts; adapt consumers.
- The first texture branch's full merge was aborted on code conflicts. Only independent
  atlas/helper files were reused; no conflict resolution or full merge is claimed.

## Consequences

No dependencies, new simulation reads, Rust edits, save changes or skeleton changes.
Target concepts are art direction, never runtime evidence. Raw prompts, comparisons and
measurement receipts live in `/Users/rexxa/work/art-img-data/look1/`. Actual pixel and FPS
claims require gated captures. The planner owns main integration and the full publish gate.

## Date

2026-11-17.

## Source

`/Users/rexxa/work/art-img-data/BRIEF_LOOK.md`, `/Users/rexxa/work/art-img-data/BRIEF_IMG.md`;
`/Users/rexxa/work/art-img-data/look1/TARGET.png`, `manifest.json`, `PLAN.md`, `REPORT.md`.
https://docs.godotengine.org/en/4.6/classes/class_environment.html and
https://docs.godotengine.org/en/4.6/tutorials/shaders/shader_reference/spatial_shader.html.
