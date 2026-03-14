# Temperament Label UI Design

## Goal
Show the agent temperament four-humor label in entity detail UI for the final Phase 1 gate.

## Approach
1. Extend the bridge entity detail payload with `temperament_label_key` and the expressed TCI axes.
2. Render the temperament label in the existing personality section without deriving temperament in GDScript.
3. Add English and Korean locale keys for the four temperament labels.

## Data Flow
- Rust bridge reads `Temperament` from ECS.
- `runtime_get_entity_detail()` writes `temperament_label_key`, `tci_ns`, `tci_ha`, `tci_rd`, and `tci_p`.
- GDScript reads `temperament_label_key` from `_detail` and resolves it with `Locale.ltr()`.

## Constraints
- No simulation logic in GDScript.
- No local temperament derivation in UI.
- Keep the existing HEXACO archetype label visible alongside the temperament label.

## Verification
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test -p sim-bridge`
- grep checks for bridge fields, UI wiring, and locale keys
