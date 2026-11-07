# 0061 — Physical building templates with persistent construction provenance

## Context

M-SPINE's second publication connects authored buildings to existing piece jobs.
The first publication establishes eras, visible Inert content and catalogue debt
(ADR 0060). Buildings must retain physical costs, identity and command causes
through partial construction, upgrades and save/load. This decision records the
integrated implementation; publication verification remains in progress.

## Decision

`buildings.ron` describes ordered plans over existing blueprints, not new
placeable kinds. The initial catalogue contains six Connected templates and
sixteen Inert templates. Connected expansion validates references, physical
footprints, station work tiles and actual blueprint costs before mutation.
Non-floor pieces and their work tiles cannot overlap. A floor may underlie a
piece or work tile only when authored earlier, permitting sequential physical
construction without an exception to placement rules.

`BuildTemplate` records an accepted command and persistent `BuildingSite`, then
activates ordinary piece sites in deterministic site-id and authored-piece order.
The selected station is persisted; omission selects the first authored station.

The brief's request to reuse a site allocator meets a legacy constraint: physical
`ColonySite` has no persistent SiteId allocator. Its ECS component stores blueprint
and footprint; job lookup keys the centre tile, and completion despawns the entity.
The aggregate therefore adds checked monotonic `next_site_id` and `next_piece_id`
for retained provenance across physical completion, cancellation and save/load.
These ids neither replace physical tile/ECS identity nor borrow workshop, bill or
room allocators. Housing zones reuse the existing `ColonyState.next_zone_id` while
keeping their tile index separate from stockpiles.

Existing physical completion callbacks advance piece phases. Aggregate completion
requires every piece's physical evidence; neither preview nor elapsed time
completes a building. Storage uses the existing stockpile pipeline when the
validated function requests `all`. Damage or cancellation must not retain a
completed roof through presentation metadata alone.

BuildingComplete60 records the completed template's immutable catalogue ordinal.
Its Building cause retains the site id, accepted command record and origin:
Command, Zone or Steward. Later upgrades may replace the site's current template
and source record; historical sentences and Why links retain their own witnesses.
Catalogue revision binding protects ordinal meaning across saves.

Save format19 appends building state as a sidecar. BuildingState and derived Room
associations are skipped in historical nested resource serialization; frozen
predecessor bodies preserve positional bytes. Load restores the sidecar, validates
physical and command/chronicle semantics, then derives room associations before
returning the engine. Invalid data is rejected rather than repaired. The saved
BuildingTemplateSystem slot is inserted only for historical colony formats;
current-format missing or duplicate slots remain errors.

Commands submitted between ticks use the engine's admission tick, while resources
retain the last executed system tick. Build and upgrade generations retain the
explicit command tick. Save validation receives the engine boundary and load
validation receives the saved body boundary: pending accepted sources may reach
that boundary, but completed chronicle facts remain bounded by the executed tick.
Neither a rejected source nor a mismatched generation tick is accepted.

The same catalogue's `zone_kinds` rows author housing as Connected and reserve
storage/workshop as Inert with explicit debt. Connected template zone references
must resolve to Connected definitions; only housing has a connected driver in
this pass. Reserved rows add no serialized physical zone variant or automatic job.

Housing zones have a separate tile map from stockpiles. Steward housing selection
filters Connected, known, expandable, currently affordable dwelling templates,
then orders highest tier first and template id ascending on ties. Availability
uses unreserved ground materials minus actual outstanding commitments, counting
matching in-transit deliveries once; carried cargo and future production are not
free supply. With housing zones present, the whole candidate footprint must fit
inside one zone. A person's home is derived from their assigned bed and an intact
physical template association, not their current position.

Upgrades retain the same footprint: the authored connected path is 4×4 log house
to 4×4 stone house. The 3×3 hut has no upgrade edge to the 4×4 log house. Workshop
and drying templates use 4×4 footprints to fit the existing 2×1 station and its
south work tile. Costs come from executable blueprints; the brief's illustrative
smaller geometry and costs cannot override existing placement or material rules.

Placeholder art reads shape, material tint, era outline and glyph metadata.
Existing material looks supply tint transforms. Item placeholders use eight shape
selectors; buildings use Round/Angular roofs and ten authored glyphs. Actual-world
roofs read persisted sites and physical intactness; active roofs reuse site wash.
Inert templates appear only in previews, grey with half-alpha clipped hatching,
and remain represented in generated content debt with their authored reason.

## Alternatives considered

- New placeable kinds or template-id behavior branches: rejected because existing
  blueprints, jobs and data-only extension must remain authoritative.
- Queueing overlapping physical pieces together or granting completion from a
  plan: rejected because occupied tiles can block jobs permanently and a plan is
  not completed construction.
- Reusing unrelated workshop, bill or room ids as building provenance: rejected
  because those namespaces have separate ownership and lifetimes; no legacy
  persistent construction SiteId allocator exists to reuse.
- Growing a hut into a larger house during upgrade: rejected because resizing
  lacks an accepted physical migration and placement contract.
- Appending ordinary fields inside historical bincode resources: rejected because
  nested positional decoding could consume subsequent historical bytes.
- Resolving history from the current upgraded site: rejected because it relabels
  earlier buildings and causes. Immutable event witnesses retain each generation.
- Choosing the highest known tier without material commitments: rejected because
  it can overbook physical inputs or credit unfinished production.

## Consequences

Templates can extend by data without simulation branches, while construction and
conservation stay in existing jobs. The site sidecar, saved system slot and
immutable provenance add explicit migration and corruption checks. UI reads must
distinguish pending, intact completed and Inert content.

The second publication's runtime, performance and visual verification is in
progress. Required evidence includes ordinary command construction and upgrade,
partial/completed save continuation, identity and negative provenance cases,
data-only extension, native captures, crash seeds and the full publication gates.
No held-out evaluation, calibration or number tuning is introduced.

## Date

2026-11-07.

## Source

Owner brief `codex_goal_worldsim_spine.md`, issues #690/#692 and ADR 0060;
`rust/crates/sim-core/data/colony/buildings.ron`,
`rust/crates/sim-systems/src/runtime/building_templates/`,
`rust/crates/sim-systems/src/save/legacy_buildings_v18.rs`, and
`rust/crates/sim-test/tests/spine_chain.rs`.
