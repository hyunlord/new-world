# Art Pass 6 final code / contract audit

Reviewed commit range: `54ad3458..44011f92347dc4176cc115195b1cc55a16ff93ae`. Read-only review of production source; this report is the only authored file. Concurrent uncommitted measurement/presentation work is outside this commit audit.

Verdict: **COMMENT — no critical/high simulation-scope defect found; one bounded correctness defect and visual-proof limitations remain.** This is not an A1–A25 or visual acceptance verdict.

## Finding

- **Medium: identical casts after the 32-record window fills can stop producing transient effects while paused.** `scripts/rendering/art/god_hand_effects.gd:91–96` identifies a record solely by tick/kind/request/applied and finds suffix/prefix overlap. After 32 identical gift-food clicks at one frozen tick (each applying60), the next getter snapshot is byte-for-byte equal to the previous32 records. The overlap becomes32 and the loop processes no new record, so the successful next click produces neither a renewed label flash nor a burst. The command and toast still work. The getter intentionally exposes only32 records, so recovery requires an additional presentation event signal from successful panel clicks or a stable sequence/cursor; do not modify simulation semantics to repair it. `test_god_hand.gd` does not exercise repeated identical commands at one tick across log truncation.

## Pixel-review follow-up: what code establishes, and what it does not

- Doorway animation is **not driven by frozen simulation ticks**: `building_renderer.gd:59–68` advances `_elapsed` using frame delta and calls the shelter helper each frame. `capture_framings.gd:81` disables WorldSim only. `shelter_cold.gd:80–98` writes the puff alpha to metadata and applies that alpha/vertical rise to a child sprite created after the Frost child. There is no static frozen-clock or obvious reversed child-order defect here. The puff is only3×2 native pixels, peaks at alpha0.55, and rises4 native pixels (`cold_effects.gd:17–19`). A positive metadata alpha does not establish perceptible doorway breath through world occlusion, tint or capture scaling. Retain the independent pixel rejection when it looks static/indistinguishable.
- Gift/spoil flash code does change a world label, not only the toast: `god_hand_effects.gd:154–166` selects `stockpile_label` and assigns `self_modulate`; `building_renderer.gd:240–254` makes that the food label with a separate wood-label child. Consequently **gift_wood/rot_wood also flash the food text; the wood child does not inherit self_modulate**. This is a concrete presentation limitation, although the brief asks generically for the stockpile label. The four particles are only2 screen pixels (`god_hand_effects.gd:186–190`). Metadata/state tests cannot overturn a failed visual reading of these effects.
- Windfall writes a real pile offset in `god_hand_effects.gd:114–132`, lasting0.6 seconds. All registered deadwood spots remain in the resource snapshot at amount0 (`world_node.rs:2063–2068`), so requiring a previous wood entry does not inherently omit depleted registered piles. A shot at+1 second should correctly show no falling offset. The prescribed+0.2 second frame, its actual timing and on-screen pile pixels are needed for visual proof.
- **Gate limitation:** `scripts/test/art/test_god_hand.gd:87–98` checks state arrays, metadata, offset and modulation, but never a framebuffer or visible rain streak count. Its green result establishes mechanics only. It must not be substituted for the independent visual verdict or A24 rendered evidence.

## Contract checks

- The only changed Rust file in the art-only range is `rust/crates/sim-bridge/src/ffi/world_node.rs`, with102 additions and0 deletions: additive getter, pure collector/columns and unit test. No simulation algorithm or pre-existing Rust test was changed in this range.
- Collector uses immutable resources, sorts active rain by settlement ID, excludes expired entries, preserves variant-specific requested amount and last32 oldest-first ordering. Seven panel presets match the brief; buttons disable without a settlement; mutation is confined to `apply_intervention`.
- The Part0 merge resolution is recorded under `docs/art/TEST_LEDGER.md:378–380`; the audited art-only range has no further Rust source-pin edits. Existing Godot assertion changes reflect explicit plan expansion, seated-crowd behavior and the documented typed-zero HUD count; no numeric acceptance threshold was lowered in the inspected source diff.
- Morning memory receives capture advance chunks (8 ticks), clears on world reset and uses separate chill alpha/puff/shiver keys. Frost selection uses cold Consuming/Sleep occupants on shelter tiles, and ash uses booked winter severity plus extinguished fire. These source checks do not establish visual acceptance.
- Carry runtime installs only reviewed food layers; rejected wood composites remain disabled. Wood fallback prefers existing same-body carry art. `carry/IMPLEMENTATION.md` explicitly preserves unresolved source/region-map and visual limitations instead of certifying them as passed.
- Changed tracked paths in the audited range are inside the brief allowlist. No earlier-pass evidence path changed.

## Verification performed in this review

- Inspected the commit diff, focused production implementations and associated tests.
- Re-ran headless `scripts/test/art/test_god_hand.gd`: exit0, `GOD_HAND failures=0` (Godot4.6 runtime / extension API4.5).
- `git diff --check` reports only two blank-at-EOF warnings in retained cargo logs; no code whitespace errors. No screenshots or full Rust suite were rerun by this reviewer; the parent owns final gate evidence.
