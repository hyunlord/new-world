# Section 16-α0 — Resource Substrate (non-depleting SOURCE tiles)

> Governance: Stage 60 (`68d68e85`) → Stage 61.
> Pipeline mode: `--full` (SimBridge FFI addition + sim-systems logic change).
> Section 16 = full gathering loop (α0 substrate → α targeting → β movement). **This ticket is α0 ONLY.**

---

## Section 1: Implementation Intent

**Problem (verified against `lead/main` @ `68d68e85`, not assumed).** Agents visibly "freeze." Step-0 code inspection found three independent causes:

1. `movement.rs` freezes Brownian motion for both `Seeking` and `Consuming` (`suppresses_movement()` + an explicit `matches!(s, Consuming)` guard). Move-toward-target is **α/β scope — NOT this ticket.**
2. `bootstrap_spawn_agents` (sim-bridge) populates agents but **never populates `food_tiles` / `water_tiles` / `sleep_tiles`.** Production resource maps are empty, so a hungry agent enters `Seeking{Food}` and finds no matching resource on its tile → stays frozen. (Confirmed: grep found zero production callers of `set_*_tile`.)
3. `world_renderer.gd`'s resource sprites are a **decorative** `RESOURCE_SEED` scatter (Phase 13-β/14-β), unrelated to the backend tile maps. The screen does not show where real resources are.

**α0 scope = solve (2) and (3) only.** Populate a resource **substrate** of non-depleting SOURCE tiles, expose it over FFI, and render it from the backend (Method 1: backend is the source of truth, the screen reflects it). α (`SeekTarget`) and β (directional movement) solve (1) later, in separate tickets.

**Source model rationale.** A source tile never depletes → agents always have a reachable goal → once α/β land movement, the freezing symptom is permanently solved. Non-depletion *is* perpetual regeneration, so **no separate regeneration system is needed** — the simplest possible substrate.

**Mechanism — sentinel `u8::MAX` (255) = "infinite source."** The tile maps are already `HashMap<(u32,u32), u8>` whose struct doc states "Saturating at `u8::MAX` is intentional." Reusing 255 as the infinite-source sentinel adds **zero new data structures** and stays ripple-free across FFI/save. The `Consuming` decrement skips sentinel tiles (they persist forever); non-sentinel (finite) tiles keep their **existing** decrement-and-remove behavior, preserving the locked Assertions 16/17 and leaving room for a future finite-economy Section.

**Tradeoff considered & rejected:** a dedicated `is_source: bool` / parallel `HashSet` is more explicit but adds a second structure + a save-format change for a single-use distinction. Sentinel is the surgical choice.

---

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly six files. No new GDScript files. No new locale keys. `movement.rs` and all `sim-core` components are NOT touched. The four stale "zero-Rust" guards (p12_alpha A17, p12_beta A24, p13_epsilon A16, p14_beta A22) and the p14_gamma A23 FFI allowlist are already handled by the Stage-61 prep commit — DO NOT touch any harness guard file; do NOT add `HARNESS_LANE` skips.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `rust/crates/sim-engine/src/lib.rs` | Add ONE item: `pub const RESOURCE_SOURCE_INFINITE: u8 = u8::MAX;`, doc-commented next to the `food_tiles` field. **No other change** — `set_food_tile/water/sleep` already exist and are reused as-is. |
| 2 | `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` | In the three `Consuming` arms (`TargetKind::Food/Water/Sleep`, ~lines 1055-1091), wrap the tile decrement in `if *counter != RESOURCE_SOURCE_INFINITE { … }`. Import `sim_engine::RESOURCE_SOURCE_INFINITE`. Need-decrement and `*state = Idle` stay **unconditional**. |
| 3 | `rust/crates/sim-bridge/src/ffi/world_node.rs` | (a) Source-tile spawn in/after `bootstrap_spawn_agents`. (b) New FFI surface: `ResourceSnapshotRow`, `collect_resource_snapshot`, `resource_rows_split`, `resource_rows_to_dict`, `#[func] get_resource_snapshot` — following the Bridge Identity Contract. |
| 4 | `rust/crates/sim-bridge/src/ffi/mod.rs` | Add ONE `pub use` line — `pub use world_node::{collect_resource_snapshot, resource_rows_split, ResourceSnapshotRow};` — mirroring the existing `collect_construction_snapshot` / `agent_rows_split` re-exports so sim-test can import them. **Required** by the harness, NOT optional churn (the prior Evaluator flagged it only because it was unauthorized — it is now authorized and necessary). |
| 5 | `scripts/ui/world_renderer.gd` | New `_render_resource_sources()` that draws backend-truth Food/Water/Sleep markers. Decorative `RESOURCE_SEED` layer preserved unchanged. |
| 6 | `rust/crates/sim-test/tests/harness_s16_alpha0_resource_substrate.rs` | New harness, ≥10 assertions. |

**Data shapes (locked):**
- Sentinel: `pub const RESOURCE_SOURCE_INFINITE: u8 = u8::MAX;` (sim-engine).
- `pub struct ResourceSnapshotRow { pub x: u32, pub y: u32, pub kind: u8 }` — `kind` encoding `0=Food, 1=Water, 2=Sleep` (matches `TargetKind` discriminant order).
- FFI dict keys: `xs`, `ys`, `kinds` — all `PackedInt32Array` of equal length (mirror `construction_rows_to_dict`).
- `pub fn resource_rows_split(rows: &[ResourceSnapshotRow]) -> (Vec<i32>, Vec<i32>, Vec<i32>)` — the pure-Rust marshalling helper that `resource_rows_to_dict` builds its `PackedInt32Array`s from. This is the **single** marshalling path, so the harness can verify the exact emitted integers without a Godot runtime (non-circular test target).
- Source placement: ≥4 tiles per kind (≥12 total), value `RESOURCE_SOURCE_INFINITE`, deterministic (fixed `const` arrays — **no RNG, no HashMap iteration**), all within `0..DEFAULT_W` × `0..DEFAULT_H` (64×64), positioned within ~4 tiles of the agent lattice (agents occupy `{4,12,20,28,36,44,52,60}²`).

**Explicit scope boundary — these are NOT in this ticket (do not invent them):** `SeekTarget` component, directional/path movement, any change to `movement.rs`, a regeneration system, finite/scarcity/competition economy, Wood/Stone backend tiles, starvation/death, any new `sim-core` component, any new locale key, any new `.gd`/`.tscn` file.

---

## Section 3: How to Implement

### T1 — sentinel const (sim-engine/lib.rs)
Add next to the `food_tiles` field doc (~line 130-150):
```rust
/// Sentinel quantity marking a tile as a non-depleting **source**
/// (V7 Section 16-α0). A tile whose counter equals this value is never
/// decremented or removed by the `Consuming` cascade — it persists for
/// the lifetime of the run, guaranteeing agents always have a reachable
/// goal. Finite tiles (any other non-zero value) keep their existing
/// decrement-and-remove behavior.
pub const RESOURCE_SOURCE_INFINITE: u8 = u8::MAX;
```

### T2 — `Consuming` source guard (agent_decision.rs)
Import the const (top of file): `use sim_engine::RESOURCE_SOURCE_INFINITE;` (or fully-qualify). For **each** of the three need arms, change ONLY the tile-mutation block:
```rust
TargetKind::Food => {
    if let Some(counter) = resources.food_tiles.get_mut(&key) {
        if *counter != RESOURCE_SOURCE_INFINITE {          // ← source guard
            *counter = counter.saturating_sub(1);
            if *counter == 0 {
                resources.food_tiles.remove(&key);
            }
        }
    }
    if let Some(h) = hunger_opt {
        h.value = (h.value - HUNGER_CONSUME_AMOUNT).max(0.0);   // UNCONDITIONAL
    }
    *state = AgentState::Idle;                                   // UNCONDITIONAL
}
```
Apply the identical `if *counter != RESOURCE_SOURCE_INFINITE` guard to `Water` (`water_tiles` / `thirst_opt`) and `Sleep` (`sleep_tiles` / `sleep_opt`). **Do not** alter the need-decrement or the `Idle` transition — those remain unconditional (Assertions 16/17). Leave the `ConstructionSite` / `Agent` arms untouched.

> **Regression caution for the Generator:** grep existing tests for tile values of `255` / `u8::MAX` before finalizing. If any current test seeds a tile at exactly 255 and asserts it decrements to 254, that test now needs its value lowered — but the substrate behavior for **finite** tiles must stay byte-for-byte identical. `cargo test --workspace` is the gate.

### T3 — bootstrap spawn + FFI (world_node.rs)

**Spawn** — add deterministic `const` position arrays (top of module, near `BOOTSTRAP_*`) and a helper called at the end of `bootstrap_spawn_agents` (or inline before its closing brace):
```rust
/// V7 Section 16-α0 — deterministic non-depleting resource sources.
/// Fixed lattice (NOT RNG): each tile is reachable within a few steps of
/// the BOOTSTRAP agent lattice and lies inside the 64×64 map.
const SOURCE_FOOD:  [(u32, u32); 4] = [(8, 8),  (56, 8),  (8, 56),  (56, 56)];
const SOURCE_WATER: [(u32, u32); 4] = [(32, 4), (4, 32),  (60, 32), (32, 60)];
const SOURCE_SLEEP: [(u32, u32); 4] = [(20, 20),(44, 20), (20, 44), (44, 44)];
```
```rust
// inside / after bootstrap_spawn_agents(engine):
for &(x, y) in SOURCE_FOOD.iter() {
    engine.resources.set_food_tile(x, y, RESOURCE_SOURCE_INFINITE);
}
for &(x, y) in SOURCE_WATER.iter() {
    engine.resources.set_water_tile(x, y, RESOURCE_SOURCE_INFINITE);
}
for &(x, y) in SOURCE_SLEEP.iter() {
    engine.resources.set_sleep_tile(x, y, RESOURCE_SOURCE_INFINITE);
}
```
Import `sim_engine::RESOURCE_SOURCE_INFINITE` (the `use sim_engine::{…}` line already exists). The Drafter/Generator may pick different concrete coordinates **provided** the constraints hold (≥4/kind, distinct, in-bounds, near lattice, deterministic const).

**FFI — Bridge Identity Contract (the Evaluator verifies this).** The `#[func]` body must be *solely* a forward to a pure-Rust collector + converter, exactly like `get_construction_snapshot`:
```rust
/// Single row of the resource-substrate snapshot. `kind`: 0=Food, 1=Water,
/// 2=Sleep (matches `TargetKind` discriminant order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceSnapshotRow { pub x: u32, pub y: u32, pub kind: u8 }

/// Pure-Rust collector over the three sparse tile maps on `SimResources`.
/// **Sorted by (kind, x, y)** — `HashMap` iteration order is unspecified,
/// so the sort is what makes the snapshot deterministic for the renderer
/// and the determinism harness assertion.
pub fn collect_resource_snapshot(resources: &SimResources) -> Vec<ResourceSnapshotRow> {
    let mut rows: Vec<ResourceSnapshotRow> = Vec::with_capacity(
        resources.food_tiles.len() + resources.water_tiles.len() + resources.sleep_tiles.len(),
    );
    for &(x, y) in resources.food_tiles.keys()  { rows.push(ResourceSnapshotRow { x, y, kind: 0 }); }
    for &(x, y) in resources.water_tiles.keys() { rows.push(ResourceSnapshotRow { x, y, kind: 1 }); }
    for &(x, y) in resources.sleep_tiles.keys() { rows.push(ResourceSnapshotRow { x, y, kind: 2 }); }
    rows.sort_by_key(|r| (r.kind, r.x, r.y));
    rows
}

/// Pure-Rust marshalling split — the SINGLE source of the (xs, ys, kinds)
/// arrays. Extracted so the harness can verify the exact integers the FFI
/// emits WITHOUT a Godot runtime (VarDictionary/PackedInt32Array need Godot).
/// `resource_rows_to_dict` MUST build its PackedInt32Arrays from this — no
/// duplicate marshalling logic.
pub fn resource_rows_split(rows: &[ResourceSnapshotRow]) -> (Vec<i32>, Vec<i32>, Vec<i32>) {
    let mut xs = Vec::with_capacity(rows.len());
    let mut ys = Vec::with_capacity(rows.len());
    let mut kinds = Vec::with_capacity(rows.len());
    for r in rows {
        xs.push(r.x as i32);
        ys.push(r.y as i32);
        kinds.push(r.kind as i32);
    }
    (xs, ys, kinds)
}

fn resource_rows_to_dict(rows: &[ResourceSnapshotRow]) -> VarDictionary {
    let (xv, yv, kv) = resource_rows_split(rows); // SAME path the harness tests
    let n = rows.len();
    let mut xs = PackedInt32Array::new();
    let mut ys = PackedInt32Array::new();
    let mut kinds = PackedInt32Array::new();
    xs.resize(n); ys.resize(n); kinds.resize(n);
    for i in 0..n {
        xs[i] = xv[i]; ys[i] = yv[i]; kinds[i] = kv[i];
    }
    let mut dict = VarDictionary::new();
    dict.set("xs", xs); dict.set("ys", ys); dict.set("kinds", kinds);
    dict
}
```
```rust
// #[func] method on WorldSimNode — body is ONLY the two-line delegation:
/// V7 Section 16-α0 FFI — resource-substrate snapshot for the renderer.
/// `VarDictionary` with three equal-length `PackedInt32Array` keys
/// (`xs`, `ys`, `kinds`), sorted by (kind, x, y). Reads `engine.resources`.
#[func]
fn get_resource_snapshot(&self) -> VarDictionary {
    let rows = collect_resource_snapshot(&self.engine.resources);
    resource_rows_to_dict(&rows)
}
```

### T4 — backend-driven markers (world_renderer.gd)
Mirror the existing `_update_construction_sites()` / `_update_settlement_furniture()` idiom (tile→pixel: `px = float(SPRITE_ORIGIN_X + x * TILE_SIZE) + float(TILE_SIZE) / 2.0`). Sources are fixed, so draw **once** (idempotent guard), no reconcile/reap loop:
```gdscript
# V7 Section 16-α0 — backend-truth resource source markers (Method 1).
# Distinct from the decorative RESOURCE_SEED layer (z=3); drawn once since
# source positions are fixed for the run.
const Z_RESOURCE_SOURCE := 4
const SOURCE_KIND_COLORS: Array = [
	Color(0.30, 0.85, 0.30, 1.0),  # 0 Food  — green
	Color(0.30, 0.65, 1.00, 1.0),  # 1 Water — blue
	Color(0.90, 0.75, 0.30, 1.0),  # 2 Sleep — amber
]
var _resource_sources_drawn: bool = false

func _render_resource_sources() -> void:
	if world_sim == null or _resource_sources_drawn:
		return
	var snap: Dictionary = world_sim.get_resource_snapshot()
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var kinds: PackedInt32Array = snap.get("kinds", PackedInt32Array())
	var n: int = min(xs.size(), min(ys.size(), kinds.size()))
	for i in n:
		var k: int = kinds[i]
		if k < 0 or k >= SOURCE_KIND_COLORS.size():
			continue
		var px: float = float(SPRITE_ORIGIN_X + xs[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var py: float = float(SPRITE_ORIGIN_Y + ys[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var marker := Sprite2D.new()
		marker.texture = <reuse an existing loaded sprite texture, e.g. RESOURCE_SPRITE_PATH>
		marker.modulate = SOURCE_KIND_COLORS[k]
		marker.z_index = Z_RESOURCE_SOURCE
		marker.position = Vector2(px, py)
		add_child(marker)
	if n > 0:
		_resource_sources_drawn = true
```
Call `_render_resource_sources()` from `_ready()` **after** `world_sim` is assigned (and/or once from the first per-frame update, guarded by `_resource_sources_drawn`). Type-guard exactly like the other snapshot consumers. **No user-facing text → no locale keys.** Use whatever distinct marker representation reads clearly (a modulated `Sprite2D` reusing an existing texture is fine — do **not** add a new asset file). Keep the existing decorative scatter loop intact.

### T5 — harness (harness_s16_alpha0_resource_substrate.rs)
Follow the `harness_p13_beta_resource_placeholders.rs` mix of runtime (build a `SimEngine`, drive the FSM) + static file-inspection. ≥10 assertions:

1. After bootstrap, `food_tiles.len() >= 4`.
2. After bootstrap, `water_tiles.len() >= 4`.
3. After bootstrap, `sleep_tiles.len() >= 4`.
4. **Determinism:** two independently-bootstrapped engines (same params) have identical `food/water/sleep_tiles` key sets and values.
5. **Source infinite:** seed a tile at `RESOURCE_SOURCE_INFINITE`, drive an agent through `Seeking→Consuming{Food}`, assert the tile is still present at `RESOURCE_SOURCE_INFINITE` afterward (and after N repeats).
6. **Need still satisfied:** the same `Consuming{Food}` step drops `Hunger.value` by `HUNGER_CONSUME_AMOUNT` (need-decrement preserved, unconditional).
7. **Finite unchanged (Assertion 16/17 regression):** a tile seeded at a small finite value (e.g. 3) still decrements on `Consuming` and is removed at 0.
8. **FFI marshalling (NON-CIRCULAR — mandatory):** seed a fresh `SimResources` with a few KNOWN tiles (e.g. `set_food_tile(8,8,255)`, `set_water_tile(4,32,255)`, `set_sleep_tile(20,20,255)`), run `collect_resource_snapshot` then `resource_rows_split`, and assert the returned `(xs, ys, kinds)` equal **hand-written literal expected vectors derived from those known positions** — e.g. sorted by `(kind,x,y)`: `xs == vec![8,4,20]`, `ys == vec![8,32,20]`, `kinds == vec![0,1,2]`. The expected vectors MUST be authored from the known inputs, **NOT** re-derived from `collect_resource_snapshot`'s own output (no `rows.iter().map(|r| r.x)` self-comparison — that is the circular A11 the prior Evaluator rejected). This exercises the exact integers the `VarDictionary` path emits without needing a Godot runtime.
9. **Bounds + reach:** every source tile is in `0..64 × 0..64` and within a small Chebyshev distance of some agent-lattice tile.
10. **Decorative preserved (static):** `world_renderer.gd` still contains `RESOURCE_SEED` and `RESOURCE_COUNT` (Phase 13-β/14-β not regressed).
11. **Bridge Identity Contract (static):** `world_node.rs`'s `get_resource_snapshot` `#[func]` body is solely the `collect_resource_snapshot` + `resource_rows_to_dict` delegation; AND `resource_rows_to_dict` builds its `PackedInt32Array`s from `resource_rows_split` (static grep) — so the integers Assertion 8 verifies are the same ones the `VarDictionary` emits (closes the non-circular gap end-to-end).
12. **Existing systems intact:** bootstrap still spawns `BOOTSTRAP_AGENT_AXIS²` (64) agents with `AgentState::Idle`.

---

## Section 4: Dispatch Plan

Single-feature, single isolated Generator pass; logical decomposition (dispatch-friendly, ≥60%):

| Ticket | File / Concern | Routing | Depends on |
|--------|----------------|---------|-----------|
| T1 | sim-engine sentinel const | 🟢 DISPATCH | — |
| T2 | agent_decision source guard | 🟢 DISPATCH | T1 |
| T3a | world_node bootstrap source spawn | 🟢 DISPATCH | T1 |
| T3b | world_node FFI (`collect_resource_snapshot` / `resource_rows_split` / `resource_rows_to_dict` / `#[func] get_resource_snapshot`) + `ffi/mod.rs` re-export | 🟢 DISPATCH | — |
| T4 | world_renderer source markers | 🟢 DISPATCH | T3b |
| T5 | harness | 🟢 DISPATCH | T1–T4 |

Dispatch ratio 100%. No 🔴 DIRECT tickets (no shared interface coordination beyond the single sentinel const, which is a one-line addition consumed by T2/T3a).

---

## Section 5: Localization Checklist

**No new localization keys.** Resource source markers are visual sprites with no user-facing text.

---

## Section 6: Verification & Notion

**Gate:**
```bash
cd rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
```
**Harness:**
```bash
cd rust && cargo test -p sim-test --test harness_s16_alpha0_resource_substrate -- --nocapture
```
**Pipeline:**
```bash
bash tools/harness/harness_pipeline.sh s16-alpha0-resource-substrate \
  .harness/prompts/s16-alpha0-resource-substrate.md --full
```
**Visual expectation:** Food (green) / Water (blue) / Sleep (amber) source markers appear at backend-truth tile positions; the decorative Wood/Stone/Berry scatter is unchanged. (VLM is sub-resolution for single-sprite changes — overall scene composition + no-crash is the bar.)

**Honest disclosure (must appear in the final report):** α0 alone produces **no agent-behavior change** — agents still freeze, because move-toward-target is α/β. The only observable change is that Food/Water/Sleep **source markers** now render at backend-truth positions, and a source tile no longer depletes when consumed. Wood/Stone/Berry decorative sprites are untouched.

**Governance chain:** Stage 60 `68d68e85` → Stage 61 (prep: retire stale guards) → Stage 62 (this S16-α0 re-run). This re-run runs AFTER the prep commit, so the four stale "zero-Rust" guards are retired and p14_gamma A23 already allowlists `get_resource_snapshot` — the Generator must NOT touch any harness guard or add `HARNESS_LANE` skips. No ENV-BYPASS. If the Drafter regresses (stub/short or scope-explosion), retry 1–2×; if it still regresses, split into (T1 const + T2 decision guard) / (T3 FFI + mod.rs) / (T4 renderer) single-concern prompts.
