# ADR 0070 — Bounded village growth from selected content

## Context

M-GROW makes settlement growth visible through worn routes, contiguous zones and
occupied houses. Existing steward commands, building templates and upgrade
chains already perform physical construction. Replacing their provenance or
adding cosmetic simulation writes would break cancellation/replay contracts.
The published format23 body contains nested bincode structs whose field order
must remain readable.

## Decision

- Author path stages/costs/intervals in `paths.ron`, layout rules in `zones.ron`
  keyed by `buildings.ron` zone ids, and residence stages/props in `yards.ron`.
  Resolve selected content through the catalogue. Categories describe behavior;
  additional content ids do not add code branches.
- Accumulate human footfall during movement; decay active tiles at day
  boundaries and refresh their navigation costs. Grass/trodden use three-tick
  steps, dirt/paved use two initially. Paved roads remain physically placed.
  Baseline worlds retain their existing movement and system registration.
- Run deterministic zone layout before steward planning, including command
  replay. Obey reachability, terrain, obstacle, spacing, path adjacency and
  population caps. Each kind may bootstrap its first zone before paths exist.
  Housing starts at 9×9 with 4×4 slots: four two-bed houses and internal spacing
  match the eight-person cap. This repairs a geometry/capacity mismatch, not an
  outcome-derived balance adjustment.
- Use the same usable interior for storage requests, extension and free-slot
  admission, preserving the future wall/door shell. Existing stock cells count
  as Storehouse service capacity; Bedroom furniture still needs free cells.
- When actual storage demand has no legal empty placement, reuse the layout
  search to find a site blocked only by preparable trees. The ordinary Stockpile
  action emits existing Chop designations; FoodStorage uses the same helper only
  with a positive ordinary capacity gap and no empty-cell expansion commands.
  Workers must remove the trees before normal daily layout admission. Preserve
  geometry, caps, path/obstacle guards, Nodes, vetoes and steward command policy.
  Never create a layout or clear a tree as a planning side effect.
- Keep `BuildCause::Steward(rule)` and store a separate site-zone link. The
  persisted zone cause records the trigger, population and prior capacity;
  the chronicle and building Why view use this evidence.
- Automatic zones are immutable steward planning facts (D-MGROW-2). Existing
  `RemoveZoneArea` / `remove_housing_area` removes player-drawn housing zones
  and real stockpile zones, leaving automatic layout envelopes unchanged.
  When the action touches an automatic zone, show a short ko/en Locale notice
  explaining that it is the village's own plan. Removing manual housing cleans
  its `site_zones` links; retain strict save validation and historical causes.
  Z only shows/hides automatic layouts. Construction cancellation and steward
  scope/off remain the available controls; direct automatic-zone editing is
  deferred as "player override of automatic zones — M-NATION/UX".
- Count occupied days only for intact completed dwellings with assigned living
  residents. Monotone stages consult local stored goods and resident skill.
  Era material waves use existing `upgrade_from`/`upgrade_to` chains at 750‰;
  upgrades require all materials and finished reachable replacement beds.
  At most one dwelling construction/upgrade per settlement is admitted.
- Yard props are read-only, data-authored uses of existing GLBs. Preload their
  selected models, place only on free adjacent tiles and skip paths/doors or
  occupied space. The zone overlay defaults off.
- Format24 appends a growth tail outside historical nested serialization.
  Validate path dimensions/codes, layout ids/geometry, cause and site links,
  dwelling types and residence stages on load. Admit known historical content
  identities and missing systems explicitly. Do not accept arbitrary catalogue
  or scheduler changes as a migration.

## Alternatives considered

- Replace a steward construction cause with a zone cause: loses the originating
  rule and changes cancellation/history validation. Keep both facts instead.
- Scan every map tile every tick: unnecessary for day-boundary wear and decay.
  Maintain active footfall indices and rebuild them from saved counts on load.
- Fractional step accumulation: broader movement change than required. Integer
  stage intervals are the approved first design; balance is deferred to M-BAL.
- Serialize new fields inside historical nested structs: bincode has no field
  names or schema-aware missing-field recovery. Append a versioned body tail.
- Give decorations construction or production effects: adds mechanics outside
  the pass. They represent existing facts and never claim simulation tiles.
- Let the existing erase command silently trim automatic layouts: this leaves
  orphan causes/site memberships or changes the meaning of historical plans.
  Do not weaken save validation or erase those causes to permit the operation;
  a future player override needs explicit planning/history semantics.

## Consequences

Colony trajectories change by design; byte-identical baseline means worlds
without colony rules. Publication still requires exact save/load and replay,
the ordinary-steward 120-day growth acceptance, selected-pack extension tests,
real 3D captures, the frozen HEAD's full gate and its separate crash detector.
The first four-seed feature acceptance passed while candidate seed1 still died
by day120 against a surviving BASE. Seed5 also failed the separate development
48-day collapse floor. Missing tree preparation prevented initial or expanded
storage. Subsequent development comparisons and full replay acceptance confirm
recovery: BASE120-day survivors [14,16,16,16], candidate [14,17,16,16]; the
separate development48-day collapse floor also passes. Preserve the original
failures and source-specific evidence in `docs/v8/mgrow/MGROW_REPORT.md`.
This is development evidence, not the exact frozen HEAD's pre-gate, full gate
or official crash detector; those publication requirements remain pending.
Automatic-layout player overrides remain M-NATION/UX debt. Initial values and
limitations remain in `docs/v8/BALANCE_DEBT.md`.

## Date

2026-11-17

## Source

Owner #686; `worldsim-operations/dispatch/goal_worldsim_mgrow.md` §§1–5;
`mgrow-data/D-MGROW-1.md` and `mgrow-data/D-MGROW-2.md` (2026-11-17); implementation in
`runtime/steward/zone_growth.rs`, `runtime/paths.rs`, `runtime/house_growth.rs`
and `save/growth.rs` under `rust/crates/sim-systems/src/`.
