# Phase 12-γ — Settlement Centroid Furniture Placeholder

Feature: p12-gamma-settlement-furniture
Lane: --quick (small Rust FFI extension paralleling β.2 A3 + GDScript
renderer extension + new harness file)
Parent: Section 13+ (`6843176f`) + `.harness/plans/phase12.md` (local)
+ Phase 12-α (`7f6a6d76`) + β.1 (`7c203764`) + β.2 A3 (`a9402d46`).

## Section 1: Implementation Intent

Phase 12-γ closes the Phase 12 sprint with the **last substrate-honest
visible delta available** on the current sim-core. Step 0 grep
(this dispatch, live files) disclosed a three-way substrate gap for
the verbatim "furniture + Settlement integration" scope:

- **No `FurnitureType` enum** in sim-core (mirrors the β.2 BuildingType
  finding).
- **No `Furniture` ECS component** anywhere.
- **No `Settlement.position` field** — `Settlement` only carries
  `settlement_id`, `member_agents: HashSet<AgentId>`,
  `member_buildings: HashSet<BuildingId>`, `population_stats`,
  `community_history`. Settlement boundaries are derived from member
  agent positions via Phase 10-β SettlementSystem's proximity scan.

The two scope options that survived this audit are G1 (defer γ
entirely) and **G3 (render one placeholder furniture sprite per
Settlement, positioned at the substrate-derived centroid of its
member agents)**. G3 is selected because it:

- Surfaces an existing real entity (`Settlement`) with a
  substrate-derived position (centroid of `member_agents`).
- Mirrors the β.2 A3 pattern exactly: parallel-PackedArray FFI,
  Dictionary<entity_bits, Sprite2D> dynamic management,
  single-placeholder sprite, no fabricated taxonomy.
- Produces a visible "settlements have a hearth at their centre" cue
  — directly answers the user mandate "최소한 게임같이" without
  inventing substrate.

Multi-furniture-type taxonomy + walls + per-building furniture
mapping all require substrate additions (`FurnitureType`,
`Furniture` component, building-furniture association, `Wall`
component) deferred to a future phase.

Preserved invariants (verified by tests):
- Phase 4-γ `SPRITE_SCALE = 0.25` (agent tile-fit)
- Phase 11-α + D1 STATE_TINTS 4-color palette
- Phase 12-α Camera2D zoom (2,2) + camera_controller.gd
- Phase 12-β.1 TileMapLayer terrain + bootstrap building + overlay
  z=10 alpha 0.65
- Phase 12-β.2 A3 ConstructionSite Sprite2D layer z=5

## Section 2: What to Build

**Modified files**:
- `rust/crates/sim-bridge/src/ffi/world_node.rs` — add
  `SettlementSnapshotRow { entity_bits, settlement_id, centroid_x,
  centroid_y, member_count }`, pure-Rust
  `collect_settlement_snapshot(world)` (queries `&Settlement`,
  derives centroid by averaging `Position` of every member agent
  found via `query::<(&Agent, &Position)>`), `settlement_rows_to_dict`
  marshaller emitting 5 parallel `PackedArray`s, and a
  `#[func] get_settlement_snapshot(&self) -> VarDictionary` on
  `WorldSimNode`. Pattern mirrors `collect_construction_snapshot`
  exactly.
- `rust/crates/sim-bridge/src/ffi/mod.rs` — re-export new symbols.
- `scripts/ui/world_renderer.gd` — add a per-settlement Sprite2D
  layer at z=4 (between TileMapLayer z=0 and ConstructionSite z=5,
  below the static bootstrap building z=5 and influence overlay
  z=10). Single placeholder sprite
  `res://assets/sprites/furniture/hearth/1.png` per Settlement,
  positioned at the FFI-supplied centroid. `_furniture_sprites:
  Dictionary` keyed by `entity_bits` with add/update/reap loop
  matching the β.2 ConstructionSite pattern.

**New files**:
- `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs`
  — 12 static-inspection assertions.

**Not changed**:
- Any other Rust crate (sim-core, sim-engine, sim-systems): no
  Settlement struct change, no new system, no new component
- `scripts/ui/agent_renderer.gd` (D1 STATE_TINTS preserved)
- `scripts/ui/camera_controller.gd` (Phase 12-α preserved)
- `shaders/palette_swap.gdshader`
- `scenes/main.tscn`
- `assets/tilesets/world_terrain.tres`
- `harness_p11_alpha_agent_renderer.rs` (22 PASS preserved)
- `harness_p12_alpha_camera_zoom.rs` (18 PASS preserved)
- `harness_p12_beta_terrain.rs` (25 PASS preserved)
- `harness_p12_beta2_construction_sites.rs` (14 PASS preserved)
- Any locale file
- Any other sprite asset

## Section 3: How to Implement

### Rust FFI extension

Add after the `ConstructionSnapshotRow` block in `world_node.rs`.
Centroid derivation: iterate `query::<&Settlement>`, then for each
settlement iterate `query::<(&Agent, &Position)>` and average the
positions of members whose `Agent.id` is in
`settlement.member_agents`. Skip settlements with zero members
(would produce `NaN` centroid).

```rust
/// Single row of the settlement snapshot returned by
/// [`collect_settlement_snapshot`].
///
/// V7 Phase 12-γ — surfaces `Settlement` entities to the GDScript
/// renderer with a substrate-derived centroid (mean position of
/// member agents). The substrate has no `Settlement.position`
/// field; this is a derived UI affordance that does NOT add or
/// modify any sim-core data. `member_count` is included so the
/// renderer can hide single-agent "settlements" if desired (the
/// formation threshold is 3 per `SETTLEMENT_FORMATION_AGENT_THRESHOLD`,
/// but races/dissolutions can momentarily drop below).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettlementSnapshotRow {
    /// `hecs::Entity::to_bits().get()` of the Settlement entity.
    pub entity_bits: u64,
    /// `Settlement::settlement_id`.
    pub settlement_id: u32,
    /// Tile-x centroid of member agents (floor of mean — i32 for
    /// FFI uniformity with construction snapshot).
    pub centroid_x: i32,
    /// Tile-y centroid of member agents.
    pub centroid_y: i32,
    /// Number of member agents at snapshot time.
    pub member_count: u32,
}

/// Pure-Rust collector. Mirrors `collect_construction_snapshot` shape
/// but joins each Settlement to its member agents' Positions.
pub fn collect_settlement_snapshot(world: &hecs::World) -> Vec<SettlementSnapshotRow> {
    use sim_core::components::{Agent, Position, Settlement};

    // First pass: collect every (Agent.id, Position) pair from the
    // world. Settlements reference members by AgentId, not by
    // hecs::Entity, so we need this lookup.
    let mut agent_positions: std::collections::HashMap<u64, (u32, u32)> = Default::default();
    for (_, (agent, pos)) in world.query::<(&Agent, &Position)>().iter() {
        agent_positions.insert(agent.id, (pos.x, pos.y));
    }

    let mut rows = Vec::new();
    for (entity, settlement) in world.query::<&Settlement>().iter() {
        let mut sum_x: u64 = 0;
        let mut sum_y: u64 = 0;
        let mut count: u32 = 0;
        for member_id in settlement.member_agents.iter() {
            if let Some(&(x, y)) = agent_positions.get(member_id) {
                sum_x += x as u64;
                sum_y += y as u64;
                count += 1;
            }
        }
        if count == 0 {
            continue;
        }
        let centroid_x = (sum_x / count as u64) as i32;
        let centroid_y = (sum_y / count as u64) as i32;
        rows.push(SettlementSnapshotRow {
            entity_bits: entity.to_bits().get(),
            settlement_id: settlement.settlement_id,
            centroid_x,
            centroid_y,
            member_count: count,
        });
    }
    rows
}

/// Marshal a [`SettlementSnapshotRow`] slice into the FFI dictionary
/// shape consumed by `WorldRenderer`. Five parallel `PackedArray`s:
/// - `ids`: `PackedInt64Array` — `entity_bits` per row.
/// - `settlement_ids`: `PackedInt32Array` — `settlement_id` per row.
/// - `centroid_xs`: `PackedInt32Array` — tile-x centroid.
/// - `centroid_ys`: `PackedInt32Array` — tile-y centroid.
/// - `member_counts`: `PackedInt32Array` — member agent count.
fn settlement_rows_to_dict(rows: &[SettlementSnapshotRow]) -> VarDictionary {
    let n = rows.len();
    let mut ids = PackedInt64Array::new();
    let mut settlement_ids = PackedInt32Array::new();
    let mut centroid_xs = PackedInt32Array::new();
    let mut centroid_ys = PackedInt32Array::new();
    let mut member_counts = PackedInt32Array::new();
    ids.resize(n);
    settlement_ids.resize(n);
    centroid_xs.resize(n);
    centroid_ys.resize(n);
    member_counts.resize(n);
    for (i, row) in rows.iter().enumerate() {
        ids.set(i, row.entity_bits as i64);
        settlement_ids.set(i, row.settlement_id as i32);
        centroid_xs.set(i, row.centroid_x);
        centroid_ys.set(i, row.centroid_y);
        member_counts.set(i, row.member_count as i32);
    }
    let mut dict = VarDictionary::new();
    dict.set("ids", ids);
    dict.set("settlement_ids", settlement_ids);
    dict.set("centroid_xs", centroid_xs);
    dict.set("centroid_ys", centroid_ys);
    dict.set("member_counts", member_counts);
    dict
}
```

Inside the `#[godot_api] impl WorldSimNode` block (after
`get_construction_snapshot`):

```rust
    /// V7 Phase 12-γ FFI — settlement snapshot with substrate-derived
    /// centroid for furniture placement. Returns a `VarDictionary`
    /// with five `PackedArray` keys.
    ///
    /// The `#[func]` body consists solely of forwarding to
    /// [`collect_settlement_snapshot`] (Bridge Identity Contract).
    #[func]
    fn get_settlement_snapshot(&self) -> VarDictionary {
        let rows = collect_settlement_snapshot(&self.engine.world);
        settlement_rows_to_dict(&rows)
    }
```

Update `rust/crates/sim-bridge/src/ffi/mod.rs` to add
`collect_settlement_snapshot, SettlementSnapshotRow` to the re-export.

### GDScript renderer extension (scripts/ui/world_renderer.gd)

Add to file-level constants:

```gdscript
# V7 Phase 12-γ — Settlement furniture placeholder.
const FURNITURE_SPRITE_PATH := "res://assets/sprites/furniture/hearth/1.png"
const Z_FURNITURE := 4
```

Add member var:

```gdscript
# entity_bits (int) → Sprite2D for the settlement's furniture placeholder.
var _furniture_sprites: Dictionary = {}
```

Add helper method (mirrors `_update_construction_sites` shape):

```gdscript
func _update_settlement_furniture() -> void:
    var snap: Dictionary = world_sim.get_settlement_snapshot()
    var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
    var xs: PackedInt32Array = snap.get("centroid_xs", PackedInt32Array())
    var ys: PackedInt32Array = snap.get("centroid_ys", PackedInt32Array())
    var n: int = ids.size()
    var seen: Dictionary = {}
    var tex: Texture2D = load(FURNITURE_SPRITE_PATH) as Texture2D
    for i in n:
        var entity_id: int = ids[i]
        seen[entity_id] = true
        var px: float = float(SPRITE_ORIGIN_X + xs[i] * TILE_SIZE + TILE_SIZE / 2)
        var py: float = float(SPRITE_ORIGIN_Y + ys[i] * TILE_SIZE + TILE_SIZE / 2)
        var sprite: Sprite2D = _furniture_sprites.get(entity_id, null) as Sprite2D
        if sprite == null:
            sprite = Sprite2D.new()
            sprite.texture = tex
            sprite.z_index = Z_FURNITURE
            add_child(sprite)
            _furniture_sprites[entity_id] = sprite
        sprite.position = Vector2(px, py)
    for entity_id in _furniture_sprites.keys():
        if not seen.has(entity_id):
            var s: Sprite2D = _furniture_sprites[entity_id]
            if s != null:
                s.queue_free()
            _furniture_sprites.erase(entity_id)
```

Hook into `_process(_delta)` AFTER `_update_construction_sites()`:

```gdscript
    _update_construction_sites()
    # V7 Phase 12-γ — ingest settlement snapshot.
    _update_settlement_furniture()
```

### Rust harness

12 assertions (A1–A12) following the β.2 precedent:

1. `a1_settlement_snapshot_collector_exists` — sim-bridge source
   contains `pub fn collect_settlement_snapshot(world: &hecs::World)`.
2. `a2_settlement_snapshot_row_struct_exists` — sim-bridge source
   declares `pub struct SettlementSnapshotRow` with the 5 fields
   `entity_bits`, `settlement_id`, `centroid_x`, `centroid_y`,
   `member_count`.
3. `a3_collector_iterates_settlement_and_agent_positions` — sim-bridge
   stripped source contains both `query::<&Settlement>` AND
   `query::<(&Agent, &Position)>`.
4. `a4_get_settlement_snapshot_func` — sim-bridge stripped source
   contains `#[func]` immediately preceding
   `fn get_settlement_snapshot(&self) -> VarDictionary` with body
   forwarding to `collect_settlement_snapshot(&self.engine.world)`.
5. `a5_dict_has_five_parallel_arrays` — sim-bridge stripped source
   sets `dict.set("ids", …)`, `…("settlement_ids", …)`,
   `…("centroid_xs", …)`, `…("centroid_ys", …)`,
   `…("member_counts", …)`.
6. `a6_collector_smoke_test_with_settlement_and_members` — instantiate
   a `SimEngine`, spawn 3 agents at distinct positions, spawn a
   Settlement entity whose `member_agents` references the 3 agents,
   call `collect_settlement_snapshot`, assert 1 row returned with
   correct centroid (integer mean of the 3 positions) and
   `member_count == 3`.
7. `a7_zero_member_settlement_is_skipped` — Settlement with empty
   `member_agents` set produces no row.
8. `a8_world_renderer_furniture_sprite_path_const` — stripped
   `world_renderer.gd` source contains `FURNITURE_SPRITE_PATH` const
   referencing `assets/sprites/furniture/hearth/1.png`.
9. `a9_world_renderer_z_furniture_constant` — stripped source
   contains `Z_FURNITURE = 4` (or `Z_FURNITURE := 4`).
10. `a10_world_renderer_polls_get_settlement_snapshot` — stripped
    source contains `world_sim.get_settlement_snapshot()` AND a
    reaper loop with `queue_free()` over `_furniture_sprites.keys()`.
11. `a11_phase4_gamma_and_d1_invariants_preserved` — read
    agent_renderer.gd, strip comments, confirm SPRITE_SCALE = 0.25
    AND all four D1 STATE_TINTS literal Colors remain.
12. `a12_phase12_alpha_beta1_beta2_invariants_preserved` — read
    main.tscn (Camera2D zoom + camera_controller.gd attached) and
    world_renderer.gd (β.1 `TERRAIN_TILESET_PATH`,
    `BUILDING_SPRITE_PATH`, `OVERLAY_ALPHA = 0.65`; β.2
    `CONSTRUCTION_SPRITE_PATH`, `Z_CONSTRUCTION = 5`).

## Section 4: Locale

No new keys. γ is renderer + FFI only.

## Section 5: Verification

```bash
cd rust && cargo test -p sim-test --test harness_p12_gamma_settlement_furniture -- --nocapture
cd rust && cargo test -p sim-test --test harness_p11_alpha_agent_renderer -- --nocapture
cd rust && cargo test -p sim-test --test harness_p12_alpha_camera_zoom -- --nocapture
cd rust && cargo test -p sim-test --test harness_p12_beta_terrain -- --nocapture
cd rust && cargo test -p sim-test --test harness_p12_beta2_construction_sites -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --headless --check-only --script scripts/ui/world_renderer.gd
```

Expected:
- harness_p12_gamma_settlement_furniture: 12 PASS
- All prior phase harnesses: green baselines preserved
- workspace: PASS, clippy clean, GDScript parse clean

## Section 6: Lane

`--quick` — small Rust FFI extension (mirrors β.2 A3 pattern
exactly), GDScript renderer extension, new harness file. No shader,
no scene structure, no sim-core / sim-systems / sim-engine touch.

Pipeline stages: Visual Verify + Evaluator.

## Section 7: 인게임 확인사항 (VLM + Human Visual Verification)

**Expected visual evidence**:
- When Phase 10-β SettlementSystem forms a settlement (requires ≥3
  proximate agents per `SETTLEMENT_FORMATION_AGENT_THRESHOLD`), a
  32×32 hearth sprite appears at the centroid of its member agents.
- Generic visual checklist (Warmth disc, influence stamps, agents,
  β.1 terrain, β.2 ConstructionSites) remains intact.

**VLM signal for APPROVE / VISUAL_PASS**:
- Generic checklist tokens pass; no regression.

**VLM signal for WARNING** (acceptable):
- No settlement visible if none formed during the capture window
  (Phase 10-β formation conditions may not have been met). Same
  precedent as β.2 ConstructionSite — environment-dependent, not a
  code defect.

**VLM signal for FAIL** (block):
- Generic visual regression.
- Furniture sprite stuck at the world origin (centroid math broken).
- Sprite leaks across frames (reaper loop broken).
- Sim-core or sim-systems modification accidentally introduced.

**Honest disclosure**:
- γ delivers Settlement visibility via a single placeholder. It does
  NOT deliver per-type furniture placement, per-building furniture
  association, or any wall rendering — those require substrate
  additions (`FurnitureType` enum, `Furniture` ECS component,
  building→furniture relation, `Wall` component).
- Centroid is a GDScript-side UI affordance derived from real
  `member_agents` positions. It is not stored in sim-core; the
  substrate remains untouched.
- After γ lands, Phase 12 chain (α + β.1 + β.2 A3 + γ) is the
  full sprint scope and user-perceptual verification (windowed
  Godot run) is the appropriate closure step per phase12.md §6.
