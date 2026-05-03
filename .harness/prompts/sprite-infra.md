# Feature 1: sprite-infra — Ritual Layer + Variant Loader Infrastructure

## Section 1: Implementation Intent

### What this solves

WorldSim's emergent building system currently lacks:
1. **No ritual/spiritual layer**: RoomRole only has Shelter/Hearth/Storage/Crafting. No cultural space emergence.
2. **No outdoor landmarks**: No cairn/gathering_marker structures for cultural objects.
3. **No sprite variant support**: building_renderer loads only `{id}.png` — no variant diversity.
4. **No furniture sprite loader**: Furniture renders as emoji only.

### Design decisions

- **totem = Furniture** with `role_contribution: "ritual"` — participates in room role voting
- **hearth = Furniture** — upgraded fire_pit with stronger warmth/social/light emissions
- **cairn = Structure** (Manual role "landmark") — outdoor influence-only landmark
- **gathering_marker = Structure** (Manual role "gathering") — outdoor social hub
- **RoomRole::Ritual** — new enum variant, assigned when totem is majority furniture
- **No new ChannelId** — reuses existing `Spiritual` channel
- **Comfort +0.02 for Ritual rooms** — placeholder until EffectStat::Meaning exists
- **Furniture voting** added alongside existing building voting in `assign_room_roles_from_buildings`

## Section 2: What was built

### Rust changes
- `sim-core/src/room.rs`: Added `RoomRole::Ritual` variant
- `sim-bridge/src/tile_info.rs`: Added Ritual match case in `room_role_locale_key()`
- `sim-systems/src/runtime/influence.rs`:
  - Furniture grid scan voting block (totem→ritual, fire_pit/hearth→hearth, storage_pit→storage, workbench/drying_rack→crafting)
  - `majority_role()`: Added `"ritual" => RoomRole::Ritual`
  - `apply_room_effects()`: Added `RoomRole::Ritual => Comfort +0.02`

### RON data
- NEW: `sim-data/data/furniture/ritual.ron` (totem + hearth FurnitureDefs)
- NEW: `sim-data/data/structures/landmarks.ron` (cairn + gathering_marker StructureDefs)
- MODIFIED: `sim-data/data/structures/shelters.ron` (hearth added to optional_components)

### Localization
- 4 new keys: BUILDING_TYPE_CAIRN, BUILDING_TYPE_GATHERING_MARKER, FURN_TOTEM, FURN_HEARTH
- Added to en/ko JSON (structures.json, furniture.json) + en/ko fluent messages.ftl

### GDScript
- `building_renderer.gd`:
  - `_load_building_texture()` extended with variant folder support (entity_id seed)
  - `_load_furniture_texture()` new function with variant support
  - `_get_variant_count()`, `_pick_variant_for_entity()`, `_pick_variant_for_tile()`, `_deterministic_seed_for_tile()` helpers
  - Furniture rendering: sprite attempt before emoji fallback
  - New emoji entries: totem (🗿), hearth (🔥)

### Tests
- `sim-data/tests/ron_registry_test.rs`: `registry_loads_new_structures_and_furniture`
- `sim-systems/src/runtime/influence.rs` tests: `ritual_room_role_assigned_when_totem_placed`, `hearth_furniture_votes_hearth_role`

## Section 3: Verification expectations

- cargo test --workspace: 1049+ passed, 0 failed
- cargo clippy: clean
- Harness sim-test: 216+ passed
- `harness_shelter_built_after_one_year`: pass (no regression)
- Existing influence tests: 25/25 pass
- New tests: 3/3 pass (registry + ritual room + hearth vote)
- Visual: No visible change (no PNG sprites yet — Feature 2)

## Section 4: Scope boundaries

OUT OF SCOPE:
- Actual PNG sprite files (Feature 2)
- Wall/floor material rendering (Feature 3)
- Ritual system logic (pray_at_totem action)
- EffectStat::Meaning
- Shelter quality modifiers
- Building renderer refactoring
