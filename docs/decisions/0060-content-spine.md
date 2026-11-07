# 0060 — Building templates over pieces, visible grey content and data-driven eras

## Context

The owner-approved M-SPINE brief (2026-11-07, milestone 13) requires a data-first
four-era development ladder. The existing technology map has a two-era enum,
32-bit practitioner masks and absent-marker weights that cap visible progress.
The construction system already owns piece jobs, materials and room recognition.

## Decision

Era thresholds, ordering, lane names and lane aliases are authored in `eras.ron`.
Progress is a separate weighted coverage within each era. Era transitions remain
reversible when knowledge holders disappear. Historical node indices remain a
prefix; wider masks and additive packed-id bits admit new nodes without aliasing.

Every node carries era, lane, tier, Connected/Inert status and debt. Connected
references resolve against the selected catalogue. Inert gates never constrain
runtime jobs, and inert discoveries cannot activate unsupported behavior. Practice
conditions count completed jobs, using the existing completion seam.

The bridge exposes authored eras and lanes; the technology map renders empty
cells and grey pending nodes as well as connected content. The generated debt
section is checked against the resolved catalogue; manual historical debt stays.

Building templates will describe existing pieces and their ordering, rather than
introducing new placeable kinds. Construction remains command driven, with site
provenance linking command, housing zone or steward request to its chronicle line.
Template integration and placeholder rendering belong to the second publication.

## Alternatives considered

- Hard-coded four-era UI and per-content behavior branches: rejected because
  external packs must extend the map by data alone (ADR 0033).
- Hiding unwired authored content: rejected because debt must be visible to the player.
- Retuning marker weights or survival outcomes: excluded by the approved content pass.

## Consequences

The save format must migrate historical aggregate, skills and packed knowledge
layouts. Catalogue selection remains immutable and revision-bound. Runtime tests,
old-save migration and ko/en rendered captures are separate publication evidence.

## Date

2026-11-07.

## Source

Owner brief `codex_goal_worldsim_spine.md`; content bible issue #690; roadmap #692.
