# Phase 12-β.2 (A3) — ConstructionSite Rendering + Progress Alpha

Feature: p12-beta2-a3-construction-sites
Lane: --quick (small Rust FFI extension + GDScript renderer extension +
new harness file)
Parent: Section 13+ (`6843176f`) + `.harness/plans/phase12.md` (local) +
Phase 12-α (`7f6a6d76`) + Phase 12-β.1 (`7c203764`).

## Section 1: Implementation Intent

Phase 12-β.2 A3 is the **narrow honest scope** chosen after Step 0 grep
disclosed that the dispatch's "multi-building + walls" verbatim scope
cannot be implemented faithfully on the current substrate:

- `rust/crates/sim-core/src/components/construction.rs` defines
  `ConstructionSite { blueprint, progress: u32, position }` and
  `BuildingBlueprint { id: BlueprintId, footprint_w/h, required_progress }`.
  **There is no `BuildingType` enum**. Buildings differ only by
  `BlueprintId: u64`.
- **There is no `Wall` ECS component or system** anywhere in sim-core
  or sim-systems. Walls have never been a simulation concept; only
  wall *sprites* exist on disk.
- Completed buildings leave no persistent ECS component — `ConstructionSite`
  is the only Phase-6 substrate that exists per-entity.

The dispatch's `BuildingSnapshotRow { building_type, construction_state }`
would require either inventing fake building taxonomy (axiom #2
violation) or adding `BuildingType` + `Wall` substrate to sim-core
(scope creep into a full Rust-crate phase).

**A3 chooses the honest narrow path**: render `ConstructionSite`
entities (which DO exist) with a single placeholder sprite, alpha
proportional to `progress / required_progress`. The "minimally
game-like" delta is: agents now visibly *build things* — sites appear
faintly when work begins and become solid as construction progresses.
Multi-type taxonomy + walls deferred to a future phase that adds the
substrate first.

Preserved invariants (verified by tests):
- Phase 4-γ `SPRITE_SCALE = 0.25` (agent tile-fit)
- Phase 11-α + D1 STATE_TINTS 4-color palette
- Phase 12-α Camera2D zoom (2,2) default + camera_controller.gd
- Phase 12-β.1 TileMapLayer terrain + bootstrap building Sprite2D +
  influence overlay z=10 alpha 0.65 (the β.1 hardcoded bootstrap
  building remains as the visible-anchor for the influence stamp; it
  is NOT removed because ConstructionSite renders are a separate
  dynamic layer driven by a different snapshot)

## Section 2: What to Build

**Modified files**:
- `rust/crates/sim-bridge/src/ffi/world_node.rs` — add
  `ConstructionSnapshotRow` struct, `collect_construction_snapshot`
  pure-Rust collector, `construction_rows_to_dict` FFI dict
  marshaller, and a new `#[func] get_construction_snapshot` method on
  `WorldSimNode`. Pattern mirrors the existing `AgentSnapshotRow` /
  `collect_agent_snapshot` / `agent_rows_to_dict` /
  `get_agent_snapshot` quartet at lines 234-241, 1027-1135.
- `rust/crates/sim-bridge/src/ffi/mod.rs` — re-export the new
  collector + row type alongside the existing
  `collect_agent_snapshot, agent_rows_split, AgentSnapshotRow`
  re-export.
- `scripts/ui/world_renderer.gd` — extend `_process(_delta)` to poll
  `WorldSim.get_construction_snapshot()` each frame, maintaining a
  `_construction_sprites: Dictionary` keyed by `entity_bits` with
  Sprite2D children created/updated/freed as the snapshot changes.
  Alpha computed from `progress / max(required_progress, 1)`,
  remapped to `[0.3, 1.0]` so freshly-started sites are still
  visible.

**New files**:
- `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs`
  — 14 static-inspection assertions (Phase 11-α / 12-α / 12-β
  precedent).

**Not changed**:
- Any other Rust crate (sim-core, sim-engine, sim-systems): no
  ConstructionSite component change, no new system, no new event,
  no new variant
- `scripts/ui/agent_renderer.gd` (D1 STATE_TINTS palette preserved)
- `scripts/ui/camera_controller.gd` (Phase 12-α preserved)
- `shaders/palette_swap.gdshader`
- `scenes/main.tscn` (no scene structure change — WorldRenderer is
  already a child and creates its own subnodes)
- `assets/tilesets/world_terrain.tres` (β.1 preserved)
- `harness_p11_alpha_agent_renderer.rs` (A1-A22 remain green)
- `harness_p12_alpha_camera_zoom.rs` (A1-A18 remain green — A18 lock
  already permits world_renderer.gd modification per β.1)
- `harness_p12_beta_terrain.rs` (A1-A25 remain green — β.1 terrain
  + bootstrap building unchanged)
- Any locale file (no new keys)
- Any sprite asset

## Section 3: How to Implement

### Rust FFI extension

Insert after the `AgentSnapshotRow` / `collect_agent_snapshot` block
(around line 1135). Mirror the precedent exactly:

```rust
/// Single row of the construction snapshot returned by
/// [`collect_construction_snapshot`].
///
/// V7 Phase 12-β.2 (A3) — surfaces `ConstructionSite` entities to the
/// GDScript renderer so the user can see agent construction activity.
/// Only the `progress` ratio is exposed; the `BlueprintId` and
/// `footprint` fields of the underlying `BuildingBlueprint` are
/// intentionally NOT in the row because the substrate has no
/// `BuildingType` taxonomy — multi-type rendering is a separate
/// (deferred) phase that first adds that substrate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstructionSnapshotRow {
    /// `hecs::Entity::to_bits().get()` of the construction-site entity.
    pub entity_bits: u64,
    /// Tile-x coordinate of the site's footprint top-left.
    pub x: u32,
    /// Tile-y coordinate of the site's footprint top-left.
    pub y: u32,
    /// Current construction progress in `ConstructionSystem` ticks.
    pub progress: u32,
    /// Total ticks required for completion (from `BuildingBlueprint`).
    pub required_progress: u32,
}

/// Pure-Rust collector for [`ConstructionSnapshotRow`] — mirrors
/// [`collect_agent_snapshot`] but queries `(&ConstructionSite, &Position)`
/// instead of `(&Agent, &Position, Option<&AgentState>)`.
///
/// Position is taken from the entity's `Position` component (the
/// canonical sim-core source) rather than `ConstructionSite::position`
/// so the rendered tile matches whatever the simulation considers
/// authoritative for that entity. The two should agree in practice.
pub fn collect_construction_snapshot(world: &hecs::World) -> Vec<ConstructionSnapshotRow> {
    let mut rows = Vec::new();
    for (entity, (site, pos)) in world
        .query::<(&ConstructionSite, &Position)>()
        .iter()
    {
        rows.push(ConstructionSnapshotRow {
            entity_bits: entity.to_bits().get(),
            x: pos.x,
            y: pos.y,
            progress: site.progress,
            required_progress: site.blueprint.required_progress,
        });
    }
    rows
}

/// Marshal a [`ConstructionSnapshotRow`] slice into the FFI dictionary
/// shape consumed by `WorldRenderer._process()`. Five parallel
/// `PackedArray`s, lengths always equal to `rows.len()`.
///
/// Keys:
/// - `ids`:  `PackedInt64Array` — `entity_bits` per row (signed cast
///   matches the agent snapshot precedent).
/// - `xs`:  `PackedInt32Array` — tile-x per row, as `i32`.
/// - `ys`:  `PackedInt32Array` — tile-y per row, as `i32`.
/// - `progresses`: `PackedInt32Array` — current progress per row, as `i32`.
/// - `required_progresses`: `PackedInt32Array` — required progress per
///   row, as `i32` (writer guarantees `> 0` via `max(.., 1)` upstream
///   but the renderer must still defend against div-by-zero).
fn construction_rows_to_dict(rows: &[ConstructionSnapshotRow]) -> VarDictionary {
    let n = rows.len();
    let mut ids = PackedInt64Array::new();
    let mut xs = PackedInt32Array::new();
    let mut ys = PackedInt32Array::new();
    let mut progresses = PackedInt32Array::new();
    let mut required_progresses = PackedInt32Array::new();
    ids.resize(n);
    xs.resize(n);
    ys.resize(n);
    progresses.resize(n);
    required_progresses.resize(n);
    for (i, row) in rows.iter().enumerate() {
        ids.set(i, row.entity_bits as i64);
        xs.set(i, row.x as i32);
        ys.set(i, row.y as i32);
        progresses.set(i, row.progress as i32);
        required_progresses.set(i, row.required_progress as i32);
    }
    let mut dict = VarDictionary::new();
    dict.set("ids", ids);
    dict.set("xs", xs);
    dict.set("ys", ys);
    dict.set("progresses", progresses);
    dict.set("required_progresses", required_progresses);
    dict
}
```

Inside the `#[godot_api] impl WorldSimNode` block (after
`get_agent_snapshot`), add:

```rust
    /// V7 Phase 12-β.2 (A3) FFI — construction-site snapshot for the
    /// GDScript renderer. Returns a `VarDictionary` with five
    /// `PackedArray` keys (`ids`, `xs`, `ys`, `progresses`,
    /// `required_progresses`) of equal length. Empty arrays when no
    /// `ConstructionSite` entities exist.
    ///
    /// The `#[func]` body consists solely of forwarding to
    /// [`collect_construction_snapshot`] (Bridge Identity Contract).
    #[func]
    fn get_construction_snapshot(&self) -> VarDictionary {
        let rows = collect_construction_snapshot(&self.engine.world);
        construction_rows_to_dict(&rows)
    }
```

In `rust/crates/sim-bridge/src/ffi/mod.rs`, extend the existing
`pub use world_node::{...}` line to include
`collect_construction_snapshot, ConstructionSnapshotRow`.

Add the `ConstructionSite` use to the top of `world_node.rs` if not
already imported.

### GDScript renderer extension (scripts/ui/world_renderer.gd)

After the existing `_ready()` body (which already creates the
TileMapLayer, the bootstrap cairn Sprite2D, and the influence overlay),
add to the file-level constants:

```gdscript
# V7 Phase 12-β.2 (A3) — construction-site rendering constants.
const CONSTRUCTION_SPRITE_PATH := "res://assets/sprites/buildings/cairn/1.png"
const Z_CONSTRUCTION := 5
const CONSTRUCTION_ALPHA_MIN := 0.3
const CONSTRUCTION_ALPHA_MAX := 1.0
```

Add a member var:

```gdscript
# entity_bits (int) → Sprite2D for that construction site.
var _construction_sprites: Dictionary = {}
```

Add a helper method:

```gdscript
func _update_construction_sites() -> void:
    var snap: Dictionary = world_sim.get_construction_snapshot()
    var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
    var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
    var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
    var progresses: PackedInt32Array = snap.get("progresses", PackedInt32Array())
    var required: PackedInt32Array = snap.get("required_progresses", PackedInt32Array())
    var n: int = ids.size()
    var seen: Dictionary = {}
    var tex: Texture2D = load(CONSTRUCTION_SPRITE_PATH) as Texture2D
    for i in n:
        var entity_id: int = ids[i]
        seen[entity_id] = true
        var px: float = float(SPRITE_ORIGIN_X + xs[i] * TILE_SIZE + TILE_SIZE / 2)
        var py: float = float(SPRITE_ORIGIN_Y + ys[i] * TILE_SIZE + TILE_SIZE / 2)
        var req: int = max(int(required[i]), 1)
        var ratio: float = clampf(float(progresses[i]) / float(req), 0.0, 1.0)
        var alpha: float = CONSTRUCTION_ALPHA_MIN + (CONSTRUCTION_ALPHA_MAX - CONSTRUCTION_ALPHA_MIN) * ratio
        var sprite: Sprite2D = _construction_sprites.get(entity_id, null) as Sprite2D
        if sprite == null:
            sprite = Sprite2D.new()
            sprite.texture = tex
            sprite.z_index = Z_CONSTRUCTION
            add_child(sprite)
            _construction_sprites[entity_id] = sprite
        sprite.position = Vector2(px, py)
        sprite.modulate = Color(1.0, 1.0, 1.0, alpha)
    # Reap entries no longer present in the snapshot (despawned sites).
    for entity_id in _construction_sprites.keys():
        if not seen.has(entity_id):
            var s: Sprite2D = _construction_sprites[entity_id]
            if s != null:
                s.queue_free()
            _construction_sprites.erase(entity_id)
```

Hook the helper into `_process(_delta)` AFTER the existing influence
overlay update:

```gdscript
func _process(_delta: float) -> void:
    if world_sim == null:
        return
    var data: PackedByteArray = world_sim.get_influence_overlay(current_channel)
    if data.size() != GRID_W * GRID_H:
        return
    image = Image.create_from_data(GRID_W, GRID_H, false, Image.FORMAT_L8, data)
    texture.update(image)
    # V7 Phase 12-β.2 (A3) — ingest construction-site snapshot.
    _update_construction_sites()
```

No change to `_unhandled_input`, click handling, or the existing
β.1 TileMapLayer / overlay setup.

### Rust harness (rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs)

Pattern: Phase 11-α / 12-α / 12-β precedent. Project root helper +
strip GDScript `#` comments before grep.

14 assertions (A1–A14):

1. `a1_construction_snapshot_collector_exists` — sim-bridge source
   contains `pub fn collect_construction_snapshot(world: &hecs::World)`.
2. `a2_construction_snapshot_row_struct_exists` — sim-bridge source
   contains `pub struct ConstructionSnapshotRow` with all 5 fields
   (`entity_bits`, `x`, `y`, `progress`, `required_progress`).
3. `a3_collector_queries_construction_site_and_position` — sim-bridge
   source contains
   `query::<(&ConstructionSite, &Position)>` (or the stripped-comment
   equivalent).
4. `a4_get_construction_snapshot_func` — sim-bridge source contains
   `#[func]` immediately preceding
   `fn get_construction_snapshot(&self) -> VarDictionary` and the body
   calls `collect_construction_snapshot(&self.engine.world)`.
5. `a5_dict_has_five_parallel_arrays` — sim-bridge source contains
   `dict.set("ids", …)`, `…("xs", …)`, `…("ys", …)`,
   `…("progresses", …)`, `…("required_progresses", …)`.
6. `a6_collector_smoke_test` — instantiate a `SimEngine`, spawn one
   entity with `(ConstructionSite, Position)`, call
   `collect_construction_snapshot`, assert one row returned with
   matching fields.
7. `a7_empty_world_returns_empty_snapshot` — empty world → empty
   `Vec`.
8. `a8_world_renderer_loads_construction_sprite_path` — stripped
   `world_renderer.gd` source contains
   `CONSTRUCTION_SPRITE_PATH` const referencing
   `assets/sprites/buildings/cairn/1.png`.
9. `a9_world_renderer_alpha_constants_present` — stripped source
   contains `CONSTRUCTION_ALPHA_MIN = 0.3` and
   `CONSTRUCTION_ALPHA_MAX = 1.0` and `Z_CONSTRUCTION = 5`.
10. `a10_world_renderer_polls_get_construction_snapshot` — stripped
    source contains `world_sim.get_construction_snapshot()`.
11. `a11_world_renderer_reaps_despawned_sites` — stripped source
    contains a `queue_free()` call inside the
    `_update_construction_sites` reaper loop AND the loop iterates
    over `_construction_sprites.keys()` checking against a `seen`
    dictionary.
12. `a12_phase4_gamma_sprite_scale_invariant_preserved` — reads
    `scripts/ui/agent_renderer.gd`, strips comments, confirms
    `SPRITE_SCALE := 0.25` (or `SPRITE_SCALE: float = 0.25`) remains.
13. `a13_d1_state_tints_palette_preserved` — reads agent_renderer.gd,
    strips comments, confirms all four D1 STATE_TINTS literal Colors
    remain (`Color(0.55, 0.70, 0.95, 1.0)`,
    `Color(1.0, 0.85, 0.15, 1.0)`, `Color(1.0, 0.40, 0.75, 1.0)`,
    `Color(0.30, 0.95, 0.35, 1.0)`).
14. `a14_phase12_alpha_and_beta1_invariants_preserved` — reads
    `scenes/main.tscn` (Camera2D `zoom = Vector2(2, 2)` +
    `camera_controller.gd` script attached) AND
    `scripts/ui/world_renderer.gd` (β.1 `TERRAIN_TILESET_PATH`,
    `BUILDING_SPRITE_PATH`, and `OVERLAY_ALPHA = 0.65` all still
    declared).

## Section 4: Locale

No new localization keys. β.2 A3 is renderer + FFI only.

## Section 5: Verification

```bash
# New β.2 A3 harness
cd rust && cargo test -p sim-test --test harness_p12_beta2_construction_sites -- --nocapture

# All prior invariants
cd rust && cargo test -p sim-test --test harness_p11_alpha_agent_renderer -- --nocapture
cd rust && cargo test -p sim-test --test harness_p12_alpha_camera_zoom -- --nocapture
cd rust && cargo test -p sim-test --test harness_p12_beta_terrain -- --nocapture

# Workspace regression
cd rust && cargo test --workspace

# Clippy
cd rust && cargo clippy --workspace --all-targets -- -D warnings

# GDScript parse check
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --headless --check-only --script scripts/ui/world_renderer.gd
```

Expected:
- harness_p12_beta2_construction_sites: 14 PASS
- harness_p11_alpha_agent_renderer: 22 PASS (baseline preserved)
- harness_p12_alpha_camera_zoom: 18 PASS (baseline preserved)
- harness_p12_beta_terrain: 25 PASS (baseline preserved)
- cargo test --workspace: PASS
- clippy: clean
- GDScript parse: no errors

## Section 6: Lane

`--quick` — small Rust FFI extension (≈100 lines mirroring an existing
pattern), GDScript renderer extension, new Rust harness file. No
shader change, no scene structure change, no sim-core / sim-systems /
sim-engine touch.

Pipeline stages: Visual Verify + Evaluator. Per-feature SceneTree
harness path NOT used for β.2 A3 because the visible delta is
substrate-driven (construction events happen on tick N depending on
agent decisions); a deterministic windowed run is hard to script. The
generic `harness_visual_verify.gd` captures whatever construction
state exists at its capture frame.

## Section 7: 인게임 확인사항 (VLM + Human Visual Verification)

**Expected pipeline visual evidence**:
- Generic visual checklist (Warmth disc + influence stamps + agents)
  remains intact — β.2 adds a new optional layer that only renders
  when ConstructionSites exist.
- If any agent has decided to start construction by the capture
  frame, a faint (alpha ≥ 0.3) cairn sprite appears at that tile,
  becoming more opaque as progress accumulates.

**VLM signal for APPROVE / VISUAL_PASS**:
- Generic checklist tokens pass.
- No regression on terrain / overlay / agents / bootstrap building.

**VLM signal for WARNING** (acceptable, do not block):
- No construction sites visible if none happened to spawn during the
  capture window. This is environment-dependent, not a code defect.
  Rule 7 +8 env-cost applies precedent-style.

**VLM signal for FAIL** (block and fix):
- Generic visual regression (Warmth disc broken, agents missing,
  scene crash).
- Construction sprite stuck at full opacity when progress is partial
  (alpha mapping broken).
- Sprites leak across frames (reaper loop broken).

**Honest disclosure**:
- A3 delivers ConstructionSite visibility — the substrate-faithful
  step toward "agents visibly build things." It does NOT deliver
  multi-type building taxonomy or wall rendering, both of which
  require substrate additions (`BuildingType` enum + `Wall`
  component) deferred to a future phase.
- The β.1 hardcoded bootstrap cairn at the influence-stamp source
  remains in place as the visual anchor for the influence overlay;
  this is a separate static layer, not a ConstructionSite.
- User-driven windowed Godot run remains the source of truth for
  perceptual quality: leave the sim running long enough for agents
  to begin building and observe progress alpha changes.
