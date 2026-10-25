# ADR 0038 — Immutable content identity and catalog-bound saves

## Context

ADR 0033 requires content extension through data while preserving executable mechanics. Closed item, blueprint and recipe enums combine identity, presentation and behavior. Historical saves encode those identities as enum discriminants, while several Chronicle fields encode one-byte numeric causes. Simply widening those fields or rebuilding derived-looking indexes would lose the old binary contract or saved history.

## Decision

Use private typed u32 identities resolved by one immutable catalog. Versioned RON owns explicit stable codes, canonical and serde names, categories, capabilities, roles and cross-references. Existing codes and legacy Debug names remain stable. Ordered definitions and BTreeMap indexes make iteration deterministic. Mechanical enums remain code; content names do not select behavior.

The host acquires an optional descriptor and freezes its source bytes before typed rules or world creation. Pure catalog construction validates definitions without installing process state. One sticky bootstrap installs either the validated catalog or its failure. Ordinary getters never read environment variables; typed deserialization requires prior installation. A failed explicit selection never falls back to bundled data. Legacy Rust constructors may initialize bundled content only when no selection has occurred. The Godot factory validates presentation before adding the main scene's children.

Bind saves to the selected source revision. Format 8 has an outer revision and opaque body, allowing rejection before typed decoding. The revision uses length-framed logical names and exact base/extension bytes; it is an identity checksum, not an authentication mechanism. Presentation sources participate in selected-pack identity. Preserve explicit format 6 and 7 readers with frozen resource and Chronicle representations. Convert their fields directly, including saved indexes, rather than synthesizing history. Historical saves are accepted only with the bundled catalog.

Store novel completed blueprint/style identity in sparse layer-specific state, separate from the historical physical projection. Base worlds keep that state empty and retain their old material behavior. Persist actual non-default workshop cutters. Validate loaded identity against registered definitions, physical slots, layers, bounds and capabilities before navigation reconstruction. Digging clears replaced floor identity.

The first implementation covers items, blueprints/construction styles and recipes plus their consumers, knowledge dependencies and host bootstrap. Pressure and animal migration and the complete seven-family pack proof remain required Stage B work; this ADR does not claim them complete. Unpublished format 8 work is one stage draft; a subsequently published layout change requires a new version and reader.

## Alternatives considered

- Extend enum match arms: rejected because every content addition still requires executable changes.
- Generate codes from input order: rejected because reordering data would change saved identities.
- Mutable global registries or lazy environment reads: rejected because process order could change world behavior and failed selections could silently resolve differently.
- Decode before checking revision: rejected because typed IDs may resolve against the wrong catalog.
- Normalize fixture output or rebuild historical logs: rejected because either would hide behavior or history changes.
- Replace all physical construction representation at once: rejected because preserving base projections with sparse novel identity keeps the compatibility boundary explicit.

## Consequences

One process uses one catalog. A pack change requires a fresh process and matching save revision. Unsupported mechanics remain explicit validation errors or recorded debt, not new content-name branches. Existing art parameter reload remains separate from selected extension identity.

Each coherent refactor commit must retain original baseline identity, all nine literal OFF summaries and frozen Stage A ON summaries/command logs. Real M5.5, M-ART2 and Stage A save continuations complement codec boundary tests. Successful parsing, unit tests or these three migrated families alone do not satisfy the mandatory seven-family runtime and UI gate.

## Date

2026-10-25

## Source

- `docs/v8/mux/MUX_GOAL.md`, B2–B3 and behavior safety net.
- `docs/v8/mux/MUX_DECISIONS.md`, D36.
- `rust/crates/sim-core/src/colony/content/`.
- `rust/crates/sim-engine/src/legacy_v7.rs`.
- `rust/crates/sim-systems/src/save/`.
- `rust/crates/sim-test/tests/content_registry_migration.rs`.
