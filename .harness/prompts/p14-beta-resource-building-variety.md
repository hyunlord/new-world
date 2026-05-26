# Phase 14-β — Resource + Building Variety (5 resources + 4 new buildings)

Feature: p14-beta-resource-building-variety
Lane: --quick (GDScript-only — one file edit + new harness)
Parent: `.harness/plans/phase14.md` (P14Plan-3 + P14Plan-4 +
P14Plan-9) + Phase 14-α (`24bbbbde`) chain successor.

## Section 1: Implementation Intent

Phase 14-β is the **second substage** of the Phase 14 RimWorld-like
Visual Overhaul Sprint. Reference games: RimWorld + Banished +
Songs of Syx.

Current state after Phase 14-α:
- Resources: 20 identical `storage_pit/1.png` sprites scattered
  via Phase 13-β RESOURCE_SEED=88675123 (no type variety —
  the world reads as "20 piles of the same thing")
- Buildings: 3 distinct sprites — bootstrap = campfire,
  ConstructionSite = cairn, Settlement centroid = hearth — but
  the user-visible map shows only those 3 categories at fixed
  positions.

The mandated visual goal "RimWorld + Banished + Songs of Syx 처럼"
requires (i) recognizable resource type variety and (ii) a
clustered "village fixture" density that reads as a settled
location. β delivers BOTH in a single substage, GDScript-only:

1. **5 resource types** — Wood, Stone, Berry, Water, Food. Each
   type maps to an existing sprite from the 207-asset inventory
   (P14Plan-3 selections). RESOURCE_COUNT stays at 20, distributed
   as 4 per type via deterministic `i % 5` cycle using the
   preserved RESOURCE_SEED. Water uses a blue-modulate tint on a
   floor tile because no native water asset exists in the
   inventory.

2. **4 new village fixtures** (in addition to 3 existing
   building categories) — Workshop, Drying area, Shelter,
   Storage — placed at fixed positions (28,28) / (36,28) /
   (28,36) / (36,36) so they cluster around the existing
   3-campfire bootstrap row at y=32. The fixture positions form
   a diamond around the centre campfire, making the centre
   region read as "a settled village" instead of "three isolated
   campfires".

**Honest disclosure**: V7 backend has no ResourceNode
component and no BuildingType / Profession ECS substrate
(verified Step 0 grep against `rust/crates/sim-core/src/components/`).
Agents do NOT interact with the new sprites — they are
decorative-only, deterministic-placement scenery. Real
ResourceNode + BuildingType systems are deferred to Section 16+.
The "Water" type uses a blue-modulate floor tile because no
native water sprite exists in the 207-asset inventory (verified
Step 0 grep: `find assets/sprites -name "*water*"` returned 0
matches).

Substrate verified (Step 0 grep, 2026-05-26):
- `scripts/ui/world_renderer.gd:90`: `RESOURCE_SPRITE_PATH :=
  "res://assets/sprites/furniture/storage_pit/1.png"` — current
  single-type sprite
- `scripts/ui/world_renderer.gd:92-93`: `RESOURCE_COUNT := 20`,
  `RESOURCE_SEED := 88675123`
- `scripts/ui/world_renderer.gd:192-208`: resource scatter loop —
  current `for _i in RESOURCE_COUNT` body
- 5 resource sprites confirmed present:
  - `assets/sprites/furniture/storage_pit/1.png` (Berry — reuses
    Phase 13-β path for backward compatibility)
  - `assets/sprites/furniture/workbench/1.png` (Wood)
  - `assets/sprites/walls/limestone/1.png` (Stone)
  - `assets/sprites/floors/stone_slab/1.png` (Water, modulate-tinted)
  - `assets/sprites/furniture/hearth/2.png` (Food)
- 4 new building sprites confirmed present:
  - `assets/sprites/furniture/workbench/2.png` (Workshop —
    variant 2 to differentiate from Wood resource)
  - `assets/sprites/furniture/drying_rack/1.png` (Drying area)
  - `assets/sprites/furniture/lean_to/1.png` (Shelter)
  - `assets/sprites/furniture/storage_pit/2.png` (Storage —
    variant 2 to differentiate from Berry resource)
- No water sprite exists in `assets/sprites/**` (verified empty
  grep result)

Preserved invariants (≥12 cross-phase):
- Phase 4-γ SPRITE_SCALE = 0.25
- Phase 8-δ RECALL_CUE_SCALE_BOOST = 1.25
- Phase 9-δ COMBAT_CUE_SCALE_BOOST = 1.3
- Phase 11-α + D1 STATE_TINTS 4-color palette
- Phase 12-α ZOOM_MIN(0.5,0.5) / ZOOM_MAX(4.0,4.0)
- Phase 12-β.1 TileMapLayer + Z_TERRAIN=0
- Phase 12-β.2 (A3) Z_CONSTRUCTION=5 + ConstructionSite cairn
  rendering (mechanism unchanged)
- Phase 12-γ Z_FURNITURE=4 + Settlement hearth rendering
  (mechanism unchanged)
- Phase 13-α ZOOM_DEFAULT = Vector2(3.0, 3.0) + BUILDING_SPRITE_PATH
  → campfire/1.png + BOOTSTRAP_X/Y/RADIUS
- Phase 13-β RESOURCE_SPRITE_PATH (legacy, kept as the Berry-type
  entry) + RESOURCE_COUNT = 20 + RESOURCE_SEED = 88675123 +
  `.seed = RESOURCE_SEED` binding + `randi_range(0, GRID_W - 1)`
  grid-bound pattern + Z_RESOURCE = 3
- Phase 13-γ STATE_SCALE_BOOST 4-entry [1.0, 1.15, 1.15, 1.15]
- Phase 13-δ HudTopbar 4-cell layout
- Phase 13-ε three-campfire bootstrap at (24,32) / (32,32) /
  (40,32) — BOOTSTRAP_X_LEFT = 24, BOOTSTRAP_X_RIGHT = 40
- Phase 14-α agent role HUE bucket count + head-icon mount
  point constants

## Section 2: What to Build

**Modified file**:
- `scripts/ui/world_renderer.gd` — add 5-type resource array,
  add 4-fixture village layer, extend `_ready()` to load + place
  the new sprites. The legacy `RESOURCE_SPRITE_PATH` constant is
  PRESERVED verbatim so Phase 13-β harness assertions
  (`harness_p13_beta_a1` + `harness_p13_beta_a6`) stay green.
  Resource scatter loop is updated to cycle `i % 5` through the
  type array; the seed binding `.seed = RESOURCE_SEED` and the
  `randi_range(0, GRID_W - 1)` call pattern are preserved
  verbatim so `harness_p13_beta_a5` stays green.

**New file**:
- `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs`
  — static file-inspection assertions following the Phase 13-β
  / 13-ε / 14-α precedent. ≥12 assertions.

**Not changed**:
- Rust crate code (zero `.rs` change in sim-core / sim-systems /
  sim-engine / sim-bridge / sim-data — β is GDScript-only)
- `scripts/ui/agent_renderer.gd` (Phase 14-α just landed; no
  re-touch)
- `scripts/ui/camera_controller.gd`
- `scripts/ui/panels/hud_topbar.gd`
- `shaders/palette_swap.gdshader`
- All existing harness files — must remain green (incl.
  `harness_p13_beta_resource_placeholders.rs` and
  `harness_p14_alpha_agent_sprite_overhaul.rs`)
- `BUILDING_SPRITE_PATH` / `CONSTRUCTION_SPRITE_PATH` /
  `FURNITURE_SPRITE_PATH` — Phase 12-β.2 / 12-γ / 13-α
  invariants, preserved verbatim. The new fixtures are a
  SEPARATE layer with their own constants; they do NOT replace
  any existing building rendering.

  **LITERAL VALUES (do not paraphrase, do not relocate)**:
  - `BUILDING_SPRITE_PATH := "res://assets/sprites/buildings/campfire/1.png"` (the `buildings/` directory, NOT `furniture/`)
  - `CONSTRUCTION_SPRITE_PATH := "res://assets/sprites/buildings/cairn/1.png"` (the `buildings/` directory, NOT `furniture/`)
  - `FURNITURE_SPRITE_PATH := "res://assets/sprites/furniture/hearth/1.png"`

  The Phase 13-α / 12-β.2 invariants reference the `buildings/`
  directory because that is where the actual sprite assets live
  on disk. No `furniture/campfire/` or `furniture/cairn/`
  directory exists in the 207-asset inventory. Harness
  assertions A16 + A17 MUST use exact-equality string match
  against the `buildings/...` literals above — `.contains()` is
  NOT acceptable.
- `RESOURCE_SPRITE_PATH` constant — preserved verbatim (used as
  RESOURCE_TYPE_PATHS[0] = Berry); the constant declaration
  itself stays for Phase 13-β harness compatibility.

## Section 3: How to Implement

### `scripts/ui/world_renderer.gd` — 5-type resource constants

After the existing Phase 13-β `RESOURCE_*` constants (~line 93),
insert:

```gdscript
# V7 Phase 14-β — 5 resource types (Wood / Stone / Berry / Water /
# Food). Each type reuses an existing sprite from the 207-asset
# inventory (P14Plan-3). RESOURCE_COUNT (20) is preserved;
# distribution becomes 4 per type via `i % 5` cycle using the
# preserved RESOURCE_SEED.
#
# Honest disclosure: V7 backend has no ResourceNode component
# (verified 2026-05-26 against rust/crates/sim-core/src/components/).
# Agents do NOT interact with these sprites. Real ResourceNode
# system is deferred to Section 16+.
#
# Index 0 = Berry deliberately reuses RESOURCE_SPRITE_PATH so the
# Phase 13-β `storage_pit/1.png` literal stays referenced and the
# `harness_p13_beta_a1` assertion remains green.
const RESOURCE_TYPE_PATHS: Array = [
	RESOURCE_SPRITE_PATH,                                         # 0: Berry (legacy alias)
	"res://assets/sprites/furniture/workbench/1.png",             # 1: Wood
	"res://assets/sprites/walls/limestone/1.png",                 # 2: Stone
	"res://assets/sprites/floors/stone_slab/1.png",               # 3: Water (modulate-tinted)
	"res://assets/sprites/furniture/hearth/2.png",                # 4: Food
]
# Water (type index 3) has no native sprite in the inventory;
# blue modulate on a floor tile is the closest visual approximation
# without DGX Spark generation.
const RESOURCE_WATER_TINT := Color(0.45, 0.65, 1.0, 1.0)
```

### `scripts/ui/world_renderer.gd` — 4-fixture village layer

After the new resource type constants, insert:

```gdscript
# V7 Phase 14-β — 4 village fixture sprites placed around the
# 3-campfire bootstrap row (Phase 13-ε) so the centre region
# reads as "a settled village" instead of "three isolated
# campfires". Positions form a diamond around (32, 32):
#   (28, 28) Workshop    (36, 28) Drying area
#   (28, 36) Shelter     (36, 36) Storage
#
# Honest disclosure: V7 backend has no BuildingType / Profession
# ECS substrate (verified 2026-05-26). These fixtures are
# decorative-only — agents do NOT interact with them.
# `_update_construction_sites` / `_update_settlement_furniture`
# remain the substrate-driven render paths. Real BuildingType
# system is deferred to Section 16+.
const VILLAGE_FIXTURE_PATHS: Array = [
	"res://assets/sprites/furniture/workbench/2.png",     # 0: Workshop
	"res://assets/sprites/furniture/drying_rack/1.png",   # 1: Drying area
	"res://assets/sprites/furniture/lean_to/1.png",       # 2: Shelter
	"res://assets/sprites/furniture/storage_pit/2.png",   # 3: Storage
]
const VILLAGE_FIXTURE_POSITIONS: Array = [
	Vector2i(28, 28),  # 0: Workshop  (top-left of diamond)
	Vector2i(36, 28),  # 1: Drying    (top-right)
	Vector2i(28, 36),  # 2: Shelter   (bottom-left)
	Vector2i(36, 36),  # 3: Storage   (bottom-right)
]
const Z_VILLAGE_FIXTURE := 5  # same plane as ConstructionSite layer
```

### `scripts/ui/world_renderer.gd` — resource scatter loop update

Change the existing `_ready()` block (currently lines 186-208).
Before:

```gdscript
	var resource_tex: Texture2D = load(RESOURCE_SPRITE_PATH) as Texture2D
	if resource_tex != null:
		var rng_res := RandomNumberGenerator.new()
		rng_res.seed = RESOURCE_SEED
		for _i in RESOURCE_COUNT:
			var rtx: int = rng_res.randi_range(0, GRID_W - 1)
			var rty: int = rng_res.randi_range(0, GRID_H - 1)
			var res_sprite := Sprite2D.new()
			res_sprite.texture = resource_tex
			res_sprite.position = Vector2(
				float(SPRITE_ORIGIN_X + rtx * TILE_SIZE) + float(TILE_SIZE) / 2.0,
				float(SPRITE_ORIGIN_Y + rty * TILE_SIZE) + float(TILE_SIZE) / 2.0,
			)
			res_sprite.z_index = Z_RESOURCE
			add_child(res_sprite)
	else:
		push_warning("WorldRenderer: failed to load resource sprite at %s" % RESOURCE_SPRITE_PATH)
```

After:

```gdscript
	# V7 Phase 14-β — pre-load all 5 resource type textures so the loop
	# doesn't hit the resource loader RESOURCE_COUNT times. Failure to
	# load any one type emits a warning and leaves the corresponding
	# entry null; the loop's null-guard then skips that index.
	var resource_textures: Array = []
	for path in RESOURCE_TYPE_PATHS:
		var tex: Texture2D = load(path) as Texture2D
		resource_textures.append(tex)
		if tex == null:
			push_warning("WorldRenderer: failed to load resource sprite at %s" % path)
	var rng_res := RandomNumberGenerator.new()
	rng_res.seed = RESOURCE_SEED
	for i in RESOURCE_COUNT:
		var rtx: int = rng_res.randi_range(0, GRID_W - 1)
		var rty: int = rng_res.randi_range(0, GRID_H - 1)
		var type_idx: int = i % RESOURCE_TYPE_PATHS.size()
		var rtex: Texture2D = resource_textures[type_idx] as Texture2D
		if rtex == null:
			continue
		var res_sprite := Sprite2D.new()
		res_sprite.texture = rtex
		res_sprite.position = Vector2(
			float(SPRITE_ORIGIN_X + rtx * TILE_SIZE) + float(TILE_SIZE) / 2.0,
			float(SPRITE_ORIGIN_Y + rty * TILE_SIZE) + float(TILE_SIZE) / 2.0,
		)
		res_sprite.z_index = Z_RESOURCE
		# V7 Phase 14-β — Water type uses a blue modulate tint because
		# no native water sprite exists in the inventory. All other
		# types render with default modulate (white).
		if type_idx == 3:
			res_sprite.modulate = RESOURCE_WATER_TINT
		add_child(res_sprite)
```

### `scripts/ui/world_renderer.gd` — village fixture placement

Immediately AFTER the resource scatter loop (still inside
`_ready()`), append:

```gdscript
	# V7 Phase 14-β — place 4 village fixture sprites at the
	# pre-determined diamond positions around the bootstrap row.
	# Each fixture has its own texture; they share Z_VILLAGE_FIXTURE
	# = Z_CONSTRUCTION = 5 so they sit in the same plane as
	# substrate-driven construction sites.
	for fi in VILLAGE_FIXTURE_PATHS.size():
		var fpath: String = VILLAGE_FIXTURE_PATHS[fi]
		var ftex: Texture2D = load(fpath) as Texture2D
		if ftex == null:
			push_warning("WorldRenderer: failed to load village fixture at %s" % fpath)
			continue
		var fpos: Vector2i = VILLAGE_FIXTURE_POSITIONS[fi]
		var fixture_sprite := Sprite2D.new()
		fixture_sprite.texture = ftex
		fixture_sprite.position = Vector2(
			float(SPRITE_ORIGIN_X + fpos.x * TILE_SIZE) + float(TILE_SIZE) / 2.0,
			float(SPRITE_ORIGIN_Y + fpos.y * TILE_SIZE) + float(TILE_SIZE) / 2.0,
		)
		fixture_sprite.z_index = Z_VILLAGE_FIXTURE
		add_child(fixture_sprite)
```

### Rust harness

`rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs`

Follow the Phase 13-β / 14-α test file structure (project_root,
strip_gd_comments, find_decl_rhss, unique_decl_rhs, no_ws).

≥15 assertions:

1. `a1_resource_type_paths_declared` — file-scope `const
   RESOURCE_TYPE_PATHS: Array = [...]` declared. Extract array
   literal, parse entries.
2. `a2_resource_type_paths_has_five_entries` — exactly 5 entries.
3. `a3_resource_type_paths_index_0_is_legacy` — entry 0 equals
   `RESOURCE_SPRITE_PATH` (preserves Phase 13-β legacy
   reference). Implementation: stripped source must contain the
   literal `RESOURCE_SPRITE_PATH` as the first element of the
   array literal (whitespace-tolerant).
4. `a4_resource_type_paths_wood_at_index_1` — entry 1 equals the
   workbench/1.png literal.
5. `a5_resource_type_paths_stone_at_index_2` — entry 2 equals
   the limestone/1.png literal.
6. `a6_resource_type_paths_water_at_index_3` — entry 3 equals
   the floors/stone_slab/1.png literal.
7. `a7_resource_type_paths_food_at_index_4` — entry 4 equals the
   hearth/2.png literal.
8. `a8_resource_water_tint_declared` — `RESOURCE_WATER_TINT :=
   Color(0.45, 0.65, 1.0, 1.0)`.
9. `a9_resource_loop_uses_modulo_cycle` — stripped source
   contains `i % RESOURCE_TYPE_PATHS.size()` (or `% 5` literal —
   accept either) inside the resource scatter block, AND the
   loop iterator is `for i in RESOURCE_COUNT` (not `for _i`).
10. `a10_village_fixture_paths_declared_with_four_entries` —
    `VILLAGE_FIXTURE_PATHS: Array = [...]` with exactly 4
    entries: workbench/2.png, drying_rack/1.png, lean_to/1.png,
    storage_pit/2.png.
11. `a11_village_fixture_positions_declared_with_four_entries` —
    `VILLAGE_FIXTURE_POSITIONS: Array = [...]` with exactly 4
    entries: `Vector2i(28, 28)`, `Vector2i(36, 28)`,
    `Vector2i(28, 36)`, `Vector2i(36, 36)`.
12. `a12_z_village_fixture_equals_five` — `Z_VILLAGE_FIXTURE :=
    5`.
13. `a13_village_fixture_loop_present` — stripped source contains
    a `for fi in VILLAGE_FIXTURE_PATHS.size()` loop (or
    equivalent token: accept `for fi in 4` literal too) and
    inside the loop body uses `VILLAGE_FIXTURE_POSITIONS[fi]`.
14. `a14_phase13_beta_resource_constants_preserved` —
    RESOURCE_SPRITE_PATH still equals
    `res://assets/sprites/furniture/storage_pit/1.png`,
    RESOURCE_COUNT = 20, RESOURCE_SEED = 88675123, Z_RESOURCE = 3
    (Phase 13-β invariants).
15. `a15_phase13_alpha_bootstrap_buildings_preserved` —
    BUILDING_SPRITE_PATH = campfire/1.png, BOOTSTRAP_X = 32,
    BOOTSTRAP_Y = 32, BOOTSTRAP_X_LEFT = 24, BOOTSTRAP_X_RIGHT =
    40.
16. `a16_phase12_construction_constants_preserved` —
    CONSTRUCTION_SPRITE_PATH = cairn/1.png, Z_CONSTRUCTION = 5.
17. `a17_phase12_gamma_furniture_constants_preserved` —
    FURNITURE_SPRITE_PATH = hearth/1.png, Z_FURNITURE = 4.
18. `a18_phase14_alpha_role_bucket_count_preserved` —
    `agent_renderer.gd` ROLE_BUCKET_COUNT = 4 and ICON_OFFSET_PX
    = Vector2(0, -12) (Phase 14-α invariants).
19. `a19_new_resource_sprite_assets_exist` — each of the 4 new
    resource paths (workbench/1.png, limestone/1.png,
    stone_slab/1.png, hearth/2.png) exists on disk + PNG magic.
20. `a20_new_village_fixture_assets_exist` — each of the 4
    village fixture paths exists on disk + PNG magic.

## Section 4: Locale

No new keys.

## Section 5: Verification

```bash
cd rust && cargo test -p sim-test --test harness_p14_beta_resource_building_variety -- --nocapture
cd rust && cargo test -p sim-test --test harness_p14_alpha_agent_sprite_overhaul -- --nocapture
cd rust && cargo test -p sim-test --test harness_p13_beta_resource_placeholders -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/world_renderer.gd
```

Expected: harness_p14_beta ≥15 PASS, all prior phases green
(especially harness_p13_beta unchanged), workspace + clippy
clean, GDScript parse clean.

Pipeline Step 2.4 (`a437c547`) GDScript strict check runs
automatically.

## Section 6: Lane

`--quick` — one GDScript file edit (additive: new constants +
modified scatter loop body + new fixture loop) + one new Rust
test file. Zero Rust crate change. Zero shader change. Zero
asset change.

## Section 7: 인게임 확인사항

**Expected visual change in pipeline VLM capture**:
- 20 resource sprites still scattered at same RESOURCE_SEED
  positions, but now showing 5 distinct shapes (4 of each type
  via `i % 5` cycle).
- 4 new village fixtures clustered around the centre campfire
  in a diamond pattern: workshop top-left, drying top-right,
  shelter bottom-left, storage bottom-right.
- Phase 13-ε's three campfires at y=32 still visible
  (unchanged).
- Phase 14-α agent rendering (4 body buckets + head-icon mount
  metadata) unchanged.

**VLM signal for APPROVE / VISUAL_PASS**: generic checklist
tokens still pass; no regression. The 5-type resource grid + 4
diamond fixtures are above sub-resolution threshold (32×32
sprites at zoom 3.0× = 96×96 px screen).

**VLM signal for WARNING** (acceptable): VLM may not articulate
all 5 type names — fine perceptual judgement at the sprite
scale + style overlap (workbench vs drying_rack are similar
silhouettes). The Rust harness's strict path checks (A4-A7) are
the authoritative validation.

**VLM signal for FAIL**: generic visual regression — resources
disappear, all-same colour, fixtures off-position, scene crash.

**Honest disclosure**:
- No backend ResourceNode or BuildingType ECS substrate. Agents
  do NOT path to / harvest / interact with the new sprites. The
  visual variety is decorative pre-substrate scenery.
- Water uses a blue modulate tint on a floor tile because the
  207-asset inventory has no native water sprite (verified Step
  0 grep). The visual approximation is the minimum that reads
  as "water-like" without DGX Spark generation.
- Workshop (`workbench/2.png`) and Wood resource
  (`workbench/1.png`) use the same furniture family but
  different variants so the village fixture is distinguishable
  from the resource scatter. Same pattern for Storage
  (`storage_pit/2.png`) vs Berry (`storage_pit/1.png`).
- Phase 13-β `RESOURCE_SPRITE_PATH` constant is INTENTIONALLY
  preserved verbatim so `harness_p13_beta_a1` + `_a6` stay
  green. It is now the index-0 (Berry) entry of
  `RESOURCE_TYPE_PATHS`, semantically reassigned but textually
  unchanged.
