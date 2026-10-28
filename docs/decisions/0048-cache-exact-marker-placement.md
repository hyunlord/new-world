# ADR 0048 — Cache exact marker placement

## Context

The quiet M-UX2.1 comparison on published M-CORE BASE found 34.948 FPS at quarter zoom with 200 founders. Marker processing consumed 24.570 ms per frame. Existing markers already use MultiMesh instances and cached ring textures; the regression was the per-frame collision placement added in M-UX2 p6, `1c8d2979e`.

## Decision

Keep the existing marker art, batches, picking, glyphs and rings. Cache placement by rounded visible anchors, stable identities, viewport, UI obstacles, extent, glyph footprint and gap. Refresh presentation and current world/canvas coordinates even on a cache hit. Preserve fractional offscreen positions.

Replace cell-by-cell search with overlapping maximal free rectangles. Subtract UI obstacles and successful reservations, discard regions smaller than a marker, and prune contained regions. Solve the feasible lattice interval in each region directly; choose the original minimum Chebyshev ring, then row, then column. Exhaustion retains the original anchor without reserving it. Delete the unused production occupancy helpers; retain the frozen original implementation in the regression oracle.

## Alternatives considered

- More draw batching: the existing renderer already batches markers; this does not remove layout work.
- Perimeter traversal and repeated-anchor memoization: exact but the first N1000 diagnostic remained far below the bar; distinct moving anchors still searched many occupied cells.
- A different global placement grid: would change existing placement and visibility. Preserve exact positions instead.
- Cache presentation buffers indiscriminately: risks stale glyphs, household colors and picking coordinates.

## Consequences

The frozen geometry oracle and windowed integration checks protect placement and cache-hit presentation. Performance, GPU visibility and selection/draft acceptance remain separate required runtime checks. No simulation source or records change.

## Date

2026-10-28.

## Source

- `docs/v8/mux2_1/MUX21_DECISIONS.md`, D12 and later evidence.
- `scripts/world/colony/marker_layout.gd`, `scripts/world/colony/lod_markers.gd`.
- `tools/v8/mux21_layout_test.gd`.
