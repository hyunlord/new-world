# A-9 Phase 3: Special Zones Runtime Application

## Section 1: Implementation Intent

### Problem

`RuleSpecialZone` is defined in `WorldRuleset` with `kind: String` and `count: (u32, u32)`, but nothing reads it at runtime. When `eternal_winter.ron` or a "dungeon_economy" ruleset defines `special_zones: [ZoneSpawner(kind: "oasis", count: (2, 4))]`, the zones never appear on the map.

### What this solves

After this feature, `apply_world_rules()` reads `special_zones` and spawns zone patches on the `WorldMap`. Each zone type modifies terrain and resources in a cluster of tiles. This is the mechanism for "미궁 경제" (dungeon nodes), "해양 세계" (island clusters), or "영원한 겨울" (hot spring oases).

### Design

Special zones are small tile clusters (5-15 tiles) placed randomly on valid terrain during world initialization. Each zone type maps to a set of tile modifications: change terrain type, add/boost resources, modify temperature/moisture. This is a compile-time operation (runs once in `apply_world_rules()`), not a per-tick system.

## Section 2: What Was Built

### RuleSpecialZone extensions (sim-data/src/defs/world_rules.rs)
- Added `radius: u32` (default 3), `terrain_override: Option<String>`, `resource_boost: Option<ZoneResourceBoost>`, `temperature_mod: Option<f32>`, `moisture_mod: Option<f32>`
- Added `ZoneResourceBoost` struct with `resource: String`, `amount: f64`, `max_amount: f64`, `regen_rate: f64`

### spawn_special_zones (sim-engine/src/engine.rs)
- Deterministic StdRng seeded from `map.seed + 7777`
- Circular cluster mask, max 200 placement attempts per zone
- Uses `TerrainType::from_str` / `ResourceType::from_str` (EnumString)
- Called from `apply_world_rules()` if `special_zones` is non-empty

### eternal_winter.ron
- Moved to `world_rules/scenarios/` (non-recursive loader won't scan it)
- Added hot_spring zone (2-4 oases, radius 3, Grassland override, Food boost, +temperature)

### Tests
- `sim-engine`: 3 unit tests (adds_resource_to_tiles, overrides_terrain, empty_list_is_noop)
- `sim-data`: 2 RON parse tests (all_fields, defaults)
- `sim-test`: harness_special_zones_spawn_on_map (negative test with base_rules.ron)

## Section 3: Verification

All new tests pass. `cargo test -p sim-data -p sim-engine` = 116/0. `harness_special_zones_spawn_on_map` = pass.
