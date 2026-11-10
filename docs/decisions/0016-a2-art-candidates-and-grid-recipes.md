# A2: reviewed sprite candidates and exact-grid room recipes

## Context

The 2026-11-10 art batch requests structures, resource nodes and crop growth
stages through the iron age on `v8/art-batch-a2`. C2/C3 will import data later.
This art checkout predates the game branch's illustrated-art importer; the
import schema is therefore a read-only reference, not a file to modify here.

Generated room pictures can look convincing while their floors converge or
their declared multi-tile footprint is much larger than the visible object.
The existing numeric projection check does not establish either property.
The generation preview can also display a gold background that is absent
from the saved PNG's alpha composite.

## Decision

Keep generated originals, exact prompts, reference hashes and rejected
attempts in the external A2 evidence directory. Use the existing STRUCT
normalizer, frozen style calibration and projection checker without relaxing
their thresholds. Adopt only numeric-ready sprites with an independent
visual scorecard covering material, line, light, palette, identity, saved
RGBA edges and footprint scale beside existing assets.

Render whole rooms from existing material textures and the existing wall
compositor. Preserve per-layer anchors and input hashes; room preview PNGs
are review artifacts, not a proposal to flatten runtime depth ordering.
Walls retain exact 64 px grid cells and 60 px vertical geometry.

Ground textures use opaque full-frame generation and the existing strip resize.
Their northwest-cell anchor is `(32,43)` and every exported pixel must be alpha
255. The legacy alpha>=128 coverage test alone cannot establish ground opacity;
both repeat axes and the frozen ground style check also remain required.

Store accepted sprites and sidecars under `art/style05/struct/a2/`, append
unique metadata to `docs/v8/artstruct/assets.json`, and keep contact sheets
under `art/batch_a2/boards/`. Draft import rows without applying them. Unknown
simulation IDs remain unresolved. Reuse already covered assets rather than
regenerate them merely to increase the count.

## Alternatives considered

- Accept a numeric pass alone: rejected because it cannot prove species,
  material identity, orthogonal floor geometry or world-scale occupancy.
- Export generated whole-room images directly: rejected because a footprint
  label does not repair perspective or preserve wall/actor occlusion.
- Remove the apparent preview halo with image processing: rejected because
  actual saved RGBA composites were already clean; cleanup would change art
  without evidence that a defect existed.
- Import candidates in this pass: excluded by the explicit art-only brief.

## Consequences

Generation, adoption and derived room composition have separate counts.
Nonselected attempts remain auditable. A candidate may pass numeric checks
and still be withheld for visual or geometric reasons. The later game import
must resolve simulation codes and validate placement, seams and occlusion in
the actual renderer; this art pass makes no runtime claim.

## Date

2026-11-10.

## Source

- `/Users/rexxa/work/art-batch/BRIEF.md`
- `art/style05/PROJECTION.md`
- `art/batch_a2/STYLE_CONTRACT.md`
- `art/batch_a2/process.py`
- `art/compositor/walls.py`
- `3d4e8f127` (first 37 independently reviewed candidates).

Number 0016 is the next free ADR number on this isolated art branch. This
record does not supersede the game branch's separate ADR sequence.
