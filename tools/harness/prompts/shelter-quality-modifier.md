# shelter-quality-modifier — Shelter Boost + Lean-to Warmth

## Section 1: Implementation Intent

Improve shelter quality mechanics:
- **Option A**: When a shelter building has optional components (hearth, storage_pit)
  inside its tile footprint, boost `influence_when_complete` emission intensity by
  `1.0 + 0.3 × optional_count`. With one: ×1.3; with two: ×1.6.
- **Option B**: Add warmth emission (radius=3.0, intensity=0.3) to the lean_to furniture
  tile so it provides a standalone warmth effect.

## Section 2: What to Build

### sim-core/src/config.rs
- `SHELTER_OPTIONAL_BOOST: f64 = 0.3`
- `SHELTER_OPTIONAL_FURNITURE_IDS: &[&str] = &["hearth", "storage_pit"]`

### sim-data/data/furniture/basic.ron
- lean_to: add `InfluenceEmission(channel: "warmth", radius: 3.0, intensity: 0.3)`

### sim-systems/src/runtime/influence.rs
- `count_shelter_optional_components(tile_grid, x, y, w, h) -> usize`
- `append_shelter_boosted_emissions(emitters, x, y, emissions, tags, multiplier)`
- In `collect_building_emitters`: shelter registry path applies multiplier when optional_count > 0

### sim-test/src/main.rs
- `harness_shelter_emission_no_boost_without_optional`: base shelter emits warmth > 0.01
- `harness_shelter_emission_boosted_with_hearth`: shelter+storage_pit ratio > 1.15× base
- `harness_lean_to_emits_warmth_standalone`: lean_to tile warmth > 0.01

## Section 3: Key Implementation Notes

- Channel "shelter" → ChannelId::Warmth; "safety" has no ChannelId mapping (excluded)
- `tick_update` only stamps dirty emitters (set dirty by `replace_emitters` every 2 ticks).
  Test A2 uses `run_ticks(2)` to stay pre-normalization — after enough ticks both engines
  normalise to the same peak value regardless of intensity, hiding the boost.
- storage_pit chosen as optional component for A2 (influence_emissions: []) to avoid
  competing warmth emissions interfering with normalization comparison.

## Section 4: Files Changed

- `rust/crates/sim-core/src/config.rs` — 2 constants added
- `rust/crates/sim-data/data/furniture/basic.ron` — lean_to emission added
- `rust/crates/sim-systems/src/runtime/influence.rs` — boost helpers + shelter path
- `rust/crates/sim-test/src/main.rs` — 3 harness tests appended

## Section 5: Localization Checklist

No new localization keys.

## Section 6: Verification

```bash
cargo test -p sim-test harness_shelter -- --nocapture
# A1: warmth=0.8800 > 0.01 ✓
# A2: ratio=1.300 > 1.15 ✓
cargo test -p sim-test harness_lean_to -- --nocapture
# A3: warmth=0.7430 > 0.01 ✓
cargo test --workspace && cargo clippy --workspace -- -D warnings
```
