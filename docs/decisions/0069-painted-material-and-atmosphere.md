# ADR 0069 — Preserve palette and simulation facts while enriching material detail

## Context
Round-one textures were too uniform at game-camera distance. T2 requires richer
materials, painted atmosphere and trace assets without changing simulation facts.

## Decision
Reuse the padded atlas and existing mesh batches. Normalize sampled luminance by
each tile mean, then apply bounded detail to the existing tint. Share four painted
sky/cloud states and hills through the existing clock/season presentation. Mirror
longitude to close panorama seams without changing vertical gradients. Add river
normal/flow samplers to existing water, with bounded two-phase advection.
Expose decals and unavailable surface variants in a data catalog for consumers.

## Alternatives considered
- Per-asset material instances: unnecessary batch splits.
- Inventing paths, dung or pond classifications: would fabricate simulation facts.
- Repeated image generation to repair panorama edges: mirrored sampling closes the
  seam while preserving accepted artwork and the three-attempt generation budget.

## Consequences
The same clouds and hills appear mirrored in opposing hemispheres. Imported
textures remain VRAM-compressed with mipmaps. Existing state supplies sky blends;
lighting, ambient and fog retain their existing policy. The orthographic game
camera rarely exposes the horizon, so separate sky-facing QA is required.
New floor/path/era choices without an authoritative selector remain catalog-only.
Windowed GPU evidence and 500-agent FPS must use the canonical shared-machine lock.

## Date
2026-11-17

## Source
Owner brief `/Users/rexxa/work/art-img-data/BRIEF_IMG2.md`, Part T2;
`assets/3d/textures3d.json`, `assets/3d/atmosphere_t2.json`.
