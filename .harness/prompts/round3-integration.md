# round3-integration: Round 3 Sprite Asset Integration Verification

## Summary
Round 3 assets (fire_pit 14 variants + lean_to 14 variants) were committed in 1fddb83.
This feature adds permanent harness assertions to guarantee asset integrity and validates
via Visual Verify that fire_pit/lean_to sprites actually render in-game (not emoji fallback).

## Changes
- File: rust/crates/sim-test/src/main.rs (assertions only, no sim logic)
- Added harness_sprite_assets_round3 module with 3 assertions:
  - A16: fire_pit directory has 14 PNG variants (1..=14 continuous)
  - A17: lean_to directory has 14 PNG variants (1..=14 continuous)
  - A18: Round 2 v2 selective files present (birch/packed_earth/wood_plank, 9 files)
- No GDScript changes (emoji fallback retained as defensive mechanism)
- No Rust simulation logic changes

## Assets (already committed in 1fddb83)
- assets/sprites/furniture/fire_pit/{1..14}.png (32×32 RGBA)
- assets/sprites/furniture/lean_to/{1..14}.png (64×32 RGBA)
- assets/sprites/walls/birch/{1..3}.png (Round 2 v2 selective)
- assets/sprites/floors/packed_earth/{1..3}.png (Round 2 v2 selective)
- assets/sprites/floors/wood_plank/{1..3}.png (Round 2 v2 selective)

## Verification
- A16 PASS: fire_pit has 14 variants (1..=14 continuous)
- A17 PASS: lean_to has 14 variants (1..=14 continuous)
- A18 PASS: 9 Round 2 v2 selective files present
- cargo test --workspace: all crates pass (stage1_simulation_tick_under_budget flaky under load, passes in isolation)
- cargo clippy: clean
- Visual Verify: fire_pit/lean_to sprites render in shelter interior (not emoji 🔥/🛏)
- No new localization keys
