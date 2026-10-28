# ADR 0051 — Place markers at stable tile targets

## Context

ADR0048 preserves exact placement for each input, but interpolated sprite positions change on every render frame. A crowded moving group invalidates the whole placement cache even between simulation ticks. The next diagnostic still exposed substantial placement cost; its end load exceeded8, so it is not an accepted FPS result.

## Decision

AgentLayer supplies separate placement anchors from each person's current simulation tile target, plus the existing bed offset for sleeping people. LodMarkers projects those anchors for collision placement and cache identity. Current interpolated positions still update leader anchors; current canvas transforms still update picking coordinates. Other callers may omit placement anchors and retain their previous behavior.

Marker positions can now stay fixed between tile-target changes instead of following every interpolated pixel. Ring styles, batching, stable-ID priority and the pure placement algorithm remain unchanged. Integer lattice inputs use direct scalar interval bounds; fractional inputs retain conservative float32 boundary checks.

## Alternatives considered

- Recompute placement on every interpolated pixel: expensive without adding useful whole-village information.
- Throttle all marker presentation: would delay glyph, color and selection feedback.
- Move geometry into the simulation bridge: unnecessary scope and runtime-source change for a renderer cache problem.

## Consequences

Windowed regression tests verify that interpolation alone reuses placement while leaders remain current, and that a changed tile target invalidates placement. Pure placement still matches the frozen original oracle. Real GPU visibility and FPS remain mandatory; test success alone is not performance acceptance.

## Date

2026-10-28.

## Source

- `scripts/world/colony/agent_layer.gd`, `scripts/world/colony/lod_markers.gd`.
- `scripts/world/colony/marker_layout.gd`, `tools/v8/mux21_layout_test.gd`.
- `docs/v8/mux2_1/MUX21_DECISIONS.md`.
