# rust/crates/sim-bridge/AGENTS.md

## Purpose

- Godot <-> Rust translation layer and the only crate that touches `godot::` types.

## Current Boundary

- This crate exposes the live runtime and helper API surface to Godot.
- It translates between Godot types and Rust types; it does not own gameplay rules.
- Document and extend only the APIs that actually exist in code.

## Must Follow

- Keep Godot type conversion at this boundary.
- Treat public `#[func]` changes as interface changes for GDScript callers.
- Use the current runtime surface when documenting or extending behavior: `runtime_init`, `runtime_tick_frame`, `runtime_get_entity_detail`, `runtime_get_entity_tab`, `runtime_get_entity_list`, `runtime_apply_commands_v2`, save/load, registry, and debug helpers.
- If you add a new UI-visible getter or event shape, update the corresponding GDScript callers in the same task.
- Keep bridge comments aligned with actual `runtime_*` and helper APIs.

## Do Not

- Do not put simulation/business logic here.
- Do not document or add imaginary getters just because old CLAUDE files mention them.
- Do not let Godot types leak into lower crates.
- Do not cache Godot objects in ways that create lifetime hazards.

## Verification

- `cd rust && cargo test -p sim-bridge`
- `cd rust && cargo build --release -p sim-bridge`
