# ADR 0070 — Prove external Tripo assets behind an opt-in data overlay

## Context

Four already generated, licensed Tripo sources need a game-budget pipeline before spending more credits. Their raw geometry and textures are too large for ordinary game assets. Existing rendering resolves content through bindings, shared manifests and position VAT; completed houses use procedural geometry.

## Decision

Process the existing external sources offline with Blender: normalize ground pivot/scale, generate three triangle-budget LODs, bake albedo/tangent normals, rig woman/goat onto the existing shared skeletons and bake the existing VAT format. Keep raw sources and run evidence outside Git. No API generation or new dependencies are required.

At startup `WORLDSIM_3D_BINDINGS` optionally names a recursive JSON overlay. Unset retains baseline data; dictionaries merge, leaves and arrays replace. The provided overlay explicitly retains baseline manifest patterns and adds Tripo static and VAT fragments. Content mappings remain data: complete log-house template, bloomery station, one neolithic adult-female variant and goat identities.

An available complete-template model replaces procedural walls inside its footprint and its room roof. Building/cancelled sites and missing models preserve procedural rendering. Rebuild identity includes template sites so completion triggers the replacement without requiring a changed wall material. Simulation, navigation and room ownership are unchanged.

## Alternatives considered

- Replace baseline bindings unconditionally: prevents a clean existing-asset comparison.
- Add hardcoded content branches in render layers: violates the data-only identity contract.
- Regenerate the bloomery or buy more source models: spends credits before this proof is accepted.
- New skeleton, runtime normal-VAT format or interior/cutaway mesh authoring: expands this four-asset proof beyond the existing pipeline.

## Consequences

The complete imported house is opaque and lacks procedural roof cutaway. Bloomery idle/working share one mesh and preserve the unusual source tube. Normal maps survive static/rigged GLB export, but the existing crowd shader derives flat normals from position VAT and samples albedo only. Heat weights and shared procedural clips need visual scrutiny after heavy decimation; static schema checks are not animation acceptance.

Focused data tests cover overlay sibling preservation and complete-template footprint/status/missing-model fallback. Same-spot z1/z2/z4 summer/dusk captures, Godot strict/runtime checks and the 500-agent z1 FPS threshold require separate admitted machine-gate evidence. This decision does not declare those checks successful or authorize more credits.

## Date

2026-11-18.

## Source

External owner brief `/Users/rexxa/work/art-img-data/BRIEF_TRIPO1.md`; `assets/3d/bindings3d_tripo.json`; `scripts/world3d/data3d.gd`; `scripts/world3d/structures3d.gd`; `tools/v8/world3d_data_test.gd`; `tools/3d/blender/README.md`.
