# ADR 0033 — The extensibility contract: kinds are data, behaviour is code

Status: accepted (M-ART2 Part 0, 2026-10-24; standing for every later pass).

## Context
- **Owner directive, 2026-10-24 (standing):** every code and design structure must be extensible. Stone, mud brick,
  fired brick, concrete — walls of every era — and by extension every content kind must go through the same code.
- M5.5 moved its neighbour, goods, story, wildlife and mood tables into RON data and listed what is still structure in
  code as debt for M-EXT (`docs/v8/m5_5/M55_DECISIONS.md` D28, D29).
- The art line proved the rule for one family: ART-POLISH2 added mud brick and fired brick walls by data and generated
  textures only, with no Python change (`v8/art-polish2` `c8ed8608`: `docs/v8/artpolish2/DATA_ONLY_PROOF.json`,
  `FIRED_BRICK_DATA_ONLY_PROOF.json`, `art/materials/ADDING_A_STYLE.md`).
- Until now the game renderer named its content in code: `scripts/world/colony/*.gd` drew sheet rects by name
  (`hearth`, `hut_wall`, `tunic` palettes) and the sim still has closed enums for items, blueprints, animals and tops.
  A new item or wall style meant edits in Rust, GDScript and the sprite generator together.

## Decision
1. **Kinds are data, behaviour is code.** Content kinds — items, materials, construction styles, recipes, knowledge,
   apparel, looks, animals, pressures, eras, UI labels — live in versioned data files (RON in the simulation, JSON in
   art and assets). Code branches on tags, categories and capabilities, never on a content name.
2. **Registry + resolver + fallback for every content family.** A family has one registry (loaded once, versioned,
   validated strictly: unknown fields and versions rejected, ids unique), one pure resolver from a sim identity to
   what to draw or do, and a category default, so an unknown entry renders or behaves by its category instead of
   failing.
3. **A "data only" test per family in the gate.** The test adds an entry by data alone (a fixture file, no code
   change) and shows it resolved and drawn or simulated. This pass adds the wall-style test on the game side
   (`resolver_parity` plus the headless data-only wall test); M-EXT adds items, recipes, pressures and animals.
4. **UI is generated from data**: build menu, designations, tech map lanes, item names and icons come from the
   registries, not from per-kind code.
5. **Every art family has a template, a schema, checks and an `ADDING_*.md`** (how to add one by data; the art line's
   `ADDING_A_STYLE.md` is the model). Imported art keeps a manifest with roles, hashes and metadata
   (`assets/v8art/manifest.json`).
6. **Systems meet through components and events.** Adding a system changes the others only by registration (the
   system list, a queue or an event kind), never by an edit inside another system.
7. **Saves are versioned with migrations.** A saved type change bumps `FORMAT_VERSION` and carries a migration or a
   documented refusal; data-file schemas carry `schema_version`.
8. **Every pass's code review checks for new hard-coded content names and new content enums.** New ones are refused;
   existing ones are listed as debt (M5.5 D29 is the first list; M-ART2 adds its own in its DECISIONS) and paid in
   M-EXT. For the renderer the check is a gate test: no id from the asset manifests appears as a literal in
   `scripts/world/**`.

## Alternatives considered
- **Keep closed enums and add a variant per kind:** cheapest per change, but every kind touches every consumer
  (exhaustive matches, FFI codes, GDScript maps, locale, sprite generator) — the lesson logged five times in
  `worldsim-architecture` ("adding an enum variant touched many files").
- **Scripting (a mod language) instead of data:** more power than the content needs; it moves behaviour into data and
  breaks determinism checks (C1–C5) that rely on code we compile and test.
- **Data without a fallback (fail on an unknown kind):** simpler loaders, but one missing look or texture would stop
  the game; a category default keeps it running and visible as "unstyled".

## Consequences
- The renderer resolves every drawable through a manifest and a resolver (ADR 0034); the gate greps `scripts/world/**`
  for manifest ids and runs the data-only wall test.
- Existing closed enums (`ItemKind`, `BlueprintKind`, `AnimalKind`, `Top`, `RoomRole`, `Modifier` …) stay until M-EXT;
  bridges translate their codes to data ids through binding files (`assets/v8art/bindings.json`) instead of code.
- Reviews add one checklist item; passes record new debt in their DECISIONS.
- **M-ART2, the first pass under this contract (2026-10-24):** the renderer (`scripts/world/**`) names no
  content: `tools/v8/test_mart2_extensibility.py` (gate step 33) checks every string literal there against the manifest
  ids, logical looks, material keys, style ids and the bindings' sim kind names, and every `16` in a tile-size position;
  both checks fail on planted lines; a wall style and material added by data only
  (`tools/v8/fixtures/data_only_wall.json`) resolves and renders (gate step 34). Debt recorded for M-EXT
  (`docs/v8/m_art2/MART2_DECISIONS.md` D16): the HUD's 32-px icons still come from the old atlas
  (`art_sprite_library.gd`), `economy_text.gd` maps item codes to locale keys by name, and `built_at` classifies stone
  by name; the bridge's closed enums are translated through `bindings.json`.

## Date
2026-10-24

## Source
docs/v8/m_art2/MART2_GOAL.md §3.1 and "Why"; docs/v8/m5_5/M55_DECISIONS.md D28, D29; branch `v8/art-polish2`
(`c8ed8608`): art/materials/ADDING_A_STYLE.md, docs/v8/artpolish2/DATA_ONLY_PROOF.json,
docs/v8/artpolish2/FIRED_BRICK_DATA_ONLY_PROOF.json
