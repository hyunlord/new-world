# ADR 0068 — Generated material textures share a padded atlas

## Context
The 3D settlement has palette-tinted meshes and terrain but few surface textures.
Part T of the 2026-11-16 image brief requires generated tileable albedos without
changing simulation facts, crowd rendering, batching or material draw counts.

## Decision
Keep individual 1024-square albedos under `assets/3d/textures/`, and sample an
8-by-8 atlas through existing materials. Each 1024 cell contains a 1008-square
image with an eight-pixel wrapped gutter. Share a parallel normal atlas, neutral
for smooth materials. Preserve existing colour grading as tint. Material mapping
is presentation data in `assets/3d/textures3d.json`; the crowd/VAT path is unchanged.
Imports use VRAM compression and mipmaps. Generated prompts, rejection records,
seam checks and raw captures remain in the external Part T data directory.

## Alternatives considered
- One new material per texture: rejected because it can split existing batches.
- Procedural substitute images: rejected because the owner requested imagegen art.
- Removing palette tint: rejected because it changes the established visual palette.

## Consequences
The atlas limits sampler count and shares texture residency. Gutter width limits
high-mip isolation; use shader-local gradients/LOD handling when sampling cells.
The 8192-square atlas needs real-device VRAM and 500-agent frame-rate verification;
headless tests alone cannot prove the 60 FPS requirement. GPU capture and timing
must wait for the canonical shared-machine gate lock and timing admission.

## Date
2026-11-16

## Source
Owner brief `/Users/rexxa/work/art-img-data/BRIEF_IMG.md`, Part T.
