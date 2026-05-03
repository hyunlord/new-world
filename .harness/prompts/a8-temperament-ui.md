# A-8 Part 2: SimBridge TCI Temperament Data

## Summary
Add TCI temperament axes (NS/HA/RD/P) and temperament label key to `entity_detail()` 
in SimBridge so the UI (entity_detail_panel_v4.gd) can display temperament data.

## Changes
- `rust/crates/sim-bridge/src/runtime_queries.rs`: Added Temperament import and 
  TCI data (tci_ns, tci_ha, tci_rd, tci_p, temperament_label_key) to entity_detail() output.

## Verification
- Gate: `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- UI reads `tci_ns`, `tci_ha`, `tci_rd`, `tci_p` from entity detail dict
- `temperament_label_key` returns one of: TEMPERAMENT_SANGUINE/CHOLERIC/MELANCHOLIC/PHLEGMATIC
- All locale keys already exist in en/ and ko/

## Test Criteria
- entity_detail() Dictionary contains tci_ns/tci_ha/tci_rd/tci_p as f64 values in [0.0, 1.0]
- entity_detail() Dictionary contains temperament_label_key as a valid locale key string
- Values match the entity's Temperament.expressed axes
