# C3A: reuse pinned art and keep failed topology disabled

## Context

BRIEF3 requests bronze gaps and iron art on an isolated art branch. Some gaps
in BRONZE_ART_INVENTORY are stale: A3 already contains IronOre and CompositeBow.
Existing numeric failures must remain visible. The named W2 wall exporter and
wall_pieces capability are absent from the inspected art-tree/W2/current-main
snapshots. The separate visual-v3 importer at 87c67b1 does splice a log-only
wall_pieces capability and compose 20 local files; that is not a drystone art
pass or a tool present on this branch. This pass does not modify the runtime.

## Decision

Pin and reuse accepted A2/A3/B3 Git blobs. Generate only absent or failed items,
retaining the cumulative three-attempt ceiling. Export topology through a thin
wrapper around the existing material compositor; leave the proposed capability
disabled when the resulting broad blurred tops fail visual review. Workshop
rooms remain independently ordered layers, with source anchors and origins.
Road transitions reuse the terrain edge compositor and preserve both passing
and failing seam diagnostics. Deliver additive import/binding fragments only.

## Alternatives considered

- Redraw all iron assets: rejected because accepted assets and provenance exist.
- Flatten rooms into one sprite: rejected because it loses wall/layer capability.
- Relax style thresholds or silently accept old numeric failures: rejected.
- Add runtime wall/road selectors: outside this art-only task.

## Consequences

The importer receives explicit source pins and honest gaps. Some required
assets remain held after the allowed attempts. Current art cannot establish
that the game state selector or wall capability works; integration must verify
that separately. No new repository dependency is added.

## Date

2026-11-14

## Source

`art/batch_c3a/STYLE.md`, `reuse/inventory.json`, `walls/manifest.json`,
`roads/tiles/manifest.json`; external C3A/INVENTORY.md and WALL_RESEARCH.md.
