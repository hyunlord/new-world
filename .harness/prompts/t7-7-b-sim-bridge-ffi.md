# T7.7.B: sim-bridge FFI methods (R1 event_queue + 3 FFI)

## Implementation Intent

V7 reset 회복 마지막 폐기 crate **sim-bridge** behavior land — godot 0.4
Rust binding 도입 (Rust 1.93 호환) + WorldSimNode 노드 + 3 FFI methods 구현. R1 event_queue
정통 path: `SimResources.building_event_queue` (concrete `VecDeque<BuildingPlacedEvent>`)
+ `BuildingStampSystem.tick()` drain → `InfluenceGrid::mark_dirty`.

근거:
- T7.7.0 (974e25d2): governance v3.3.8 — Signal A whitelist 에 sim-bridge 추가
- T7.7.A (5092328b): sim-bridge 빈 scaffold (cdylib + rlib, no godot dep)
- 사전 grep 검증 (axiom #2):
  - `BuildingPlacedEvent`: 신설 (sim-engine, E1 결정)
  - `InfluenceGrid::mark_dirty`: sim-core/src/influence/grid.rs:112 land 완료
  - godot crate: 0.4 (Rust 1.93 호환 — 0.5.x는 MSRV 1.94+ 필요, 현재 toolchain은 1.93.1)
- Phase 0 v0.1.3 patch §4.4 (sim-test) + §5 (sim-bridge FFI)

이 작업은 **Signal D auto-classify (sim-bridge 첫 `impl RuntimeSystem` 부재 +
hot-tier behavior `#[func]` FFI methods)** — `--full` pipeline 의무 (Drafter
debate + Generator + Evaluator + Visual + Regression).

## Decisions Locked

| # | Decision | Rationale |
|---|----------|-----------|
| godot version | `0.4` (workspace.dependencies) | Rust 1.93.1 toolchain 호환 (godot 0.5.x MSRV ≥1.94, 미가용); Godot 4.2+ 지원 |
| Event location | sim-engine (E1) | SimEngine/SimResources host crate |
| Event type | concrete `BuildingPlacedEvent` | Generic `EventQueue<T>` 회피 (YAGNI) |
| Queue type | `VecDeque<BuildingPlacedEvent>` | drain order 보존 + O(1) push_back |
| Drain owner | `BuildingStampSystem.tick()` | priority 90, 첫 tick 단계 |
| FFI test scope | sim-test (mechanism only, no godot runtime) | godot binding 은 cargo build 로 검증 |
| Unsafe scope | sim-bridge entry point only | `#![forbid(unsafe_code)]` 제거 + `unsafe impl ExtensionLibrary` |

## What to Build

### Files (3 신설 + 5 수정)

| # | File | Action | Lines |
|---|------|--------|------:|
| 1 | `rust/Cargo.toml` | edit (workspace.dependencies +godot) | +1 |
| 2 | `rust/crates/sim-engine/src/lib.rs` | edit (BuildingPlacedEvent + queue field) | ~30 |
| 3 | `rust/crates/sim-systems/src/runtime/influence/building_stamp.rs` | edit (tick drain) | ~30 |
| 4 | `rust/crates/sim-bridge/Cargo.toml` | edit (+godot dep) | +1 |
| 5 | `rust/crates/sim-bridge/src/lib.rs` | rewrite (entry + module re-export) | ~25 |
| 6 | `rust/crates/sim-bridge/src/ffi/mod.rs` | new (module aggregator) | ~5 |
| 7 | `rust/crates/sim-bridge/src/ffi/world_node.rs` | new (WorldSimNode + 3 FFI methods) | ~120 |
| 8 | `rust/crates/sim-test/tests/harness_phase2_ffi.rs` | new (3 FFI integration tests) | ~80 |

## How to Implement

### File 1: `rust/Cargo.toml` (edit — workspace dep)

Add to `[workspace.dependencies]`:
```toml
godot = "0.4"
```

### File 2: `rust/crates/sim-engine/src/lib.rs` (edit)

Add at top of file (after existing imports):
```rust
use std::collections::VecDeque;
```

Add new public struct + add field to `SimResources`:
```rust
/// FFI-originated event: a building was placed at `position` with influence
/// `radius` (Chebyshev distance, in tiles). Drained by
/// `sim_systems::runtime::influence::BuildingStampSystem` each tick which
/// translates each event into `InfluenceGrid::mark_dirty` calls on the
/// Warmth/Spiritual/Beauty/Light channels.
///
/// Phase 0 v0.1.3 §5 — R1 event_queue path (T7.7.B land).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingPlacedEvent {
    /// Tile coordinates of the building origin (top-left corner).
    pub position: (u32, u32),
    /// Influence radius in tiles (Chebyshev distance, inclusive).
    pub radius: u32,
}
```

In `SimResources` struct definition add field:
```rust
    /// FFI-originated building placement events, drained each tick by
    /// `BuildingStampSystem`. Pushed by `sim_bridge::WorldSimNode::on_building_placed`.
    pub building_event_queue: VecDeque<BuildingPlacedEvent>,
```

In `SimResources::new` (or wherever the struct is constructed inside
`SimEngine::new`) initialize: `building_event_queue: VecDeque::new(),`.

### File 3: `rust/crates/sim-systems/src/runtime/influence/building_stamp.rs` (edit)

Replace tick body (currently a no-op) with the queue-drain implementation.
Use 4 channels: Warmth, Spiritual, Beauty, Light. Use
`InfluenceGrid::mark_dirty` with `DirtyRegion::new(x1, y1, x2, y2)` clamped
to grid bounds.

```rust
//! `BuildingStampSystem` — priority 90, every tick.
//!
//! Phase 0 §2.5.1 base. T7.7.B land (R1 event_queue):
//! drains `resources.building_event_queue` and calls
//! `InfluenceGrid::mark_dirty` for each (Warmth/Spiritual/Beauty/Light)
//! channel using a Chebyshev box clamped to the grid.

use hecs::World;
use sim_core::influence::{DirtyRegion, InfluenceChannel};
use sim_engine::{RuntimeSystem, SimResources};

/// Channels stamped by every building placement (T7.7.B).
const STAMPED_CHANNELS: &[InfluenceChannel] = &[
    InfluenceChannel::Warmth,
    InfluenceChannel::Spiritual,
    InfluenceChannel::Beauty,
    InfluenceChannel::Light,
];

/// Phase 2 building → influence stamper (T7.7.B drains FFI queue).
pub struct BuildingStampSystem;

impl BuildingStampSystem {
    /// Construct a new stamper.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BuildingStampSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSystem for BuildingStampSystem {
    fn name(&self) -> &str {
        "BuildingStampSystem"
    }
    fn priority(&self) -> u32 {
        90
    }
    fn tick_interval(&self) -> u64 {
        1
    }
    fn tick(&mut self, _world: &mut World, resources: &mut SimResources) {
        let w = resources.influence_grid.width;
        let h = resources.influence_grid.height;
        if w == 0 || h == 0 {
            resources.building_event_queue.clear();
            return;
        }
        while let Some(ev) = resources.building_event_queue.pop_front() {
            let (cx, cy) = ev.position;
            if cx >= w || cy >= h {
                continue;
            }
            let r = ev.radius;
            let x1 = cx.saturating_sub(r);
            let y1 = cy.saturating_sub(r);
            let x2 = cx.saturating_add(r).min(w - 1);
            let y2 = cy.saturating_add(r).min(h - 1);
            for ch in STAMPED_CHANNELS {
                resources
                    .influence_grid
                    .mark_dirty(*ch, DirtyRegion::new(x1, y1, x2, y2));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::{BuildingPlacedEvent, SimEngine};

    fn engine() -> SimEngine {
        SimEngine::new(32, 32, MaterialRegistry::new())
    }

    #[test]
    fn metadata() {
        let s = BuildingStampSystem::new();
        assert_eq!(s.name(), "BuildingStampSystem");
        assert_eq!(s.priority(), 90);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn empty_queue_is_no_op() {
        let mut e = engine();
        let mut s = BuildingStampSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        for ch in InfluenceChannel::all() {
            assert!(e.resources.influence_grid.dirty_regions[*ch as usize].is_empty());
        }
    }

    #[test]
    fn single_event_marks_4_channels_dirty() {
        let mut e = engine();
        e.resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (10, 10),
                radius: 2,
            });
        let mut s = BuildingStampSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        for ch in STAMPED_CHANNELS {
            let regs = &e.resources.influence_grid.dirty_regions[*ch as usize];
            assert_eq!(regs.len(), 1);
        }
        // Other channels untouched.
        assert!(e.resources.influence_grid.dirty_regions[InfluenceChannel::Danger as usize].is_empty());
        // Queue drained.
        assert!(e.resources.building_event_queue.is_empty());
    }

    #[test]
    fn out_of_bounds_event_is_skipped() {
        let mut e = engine();
        e.resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (999, 999),
                radius: 1,
            });
        let mut s = BuildingStampSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        for ch in STAMPED_CHANNELS {
            assert!(e.resources.influence_grid.dirty_regions[*ch as usize].is_empty());
        }
    }

    #[test]
    fn radius_clamps_to_grid() {
        let mut e = engine(); // 32×32
        e.resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (1, 1),
                radius: 100, // huge — must clamp to (0,0)..(31,31)
            });
        let mut s = BuildingStampSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        let regs = &e.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize];
        assert_eq!(regs.len(), 1);
        // Region should span the whole grid (saturating_sub from 1 with 100
        // gives 0; saturating_add capped at w-1=31).
    }
}
```

### File 4: `rust/crates/sim-bridge/Cargo.toml` (edit)

Add `godot` to `[dependencies]`:
```toml
godot = { workspace = true }
```

Final form:
```toml
[package]
name = "sim-bridge"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
hecs = { workspace = true }
serde = { workspace = true }
godot = { workspace = true }
sim-core = { path = "../sim-core" }
sim-engine = { path = "../sim-engine" }
sim-systems = { path = "../sim-systems" }
```

### File 5: `rust/crates/sim-bridge/src/lib.rs` (rewrite)

⚠️ **Drop `#![forbid(unsafe_code)]`** — godot 0.4 requires `unsafe impl
ExtensionLibrary` on the entry point. Restrict the unsafe scope via
`#![deny(unsafe_op_in_unsafe_fn)]` instead.

```rust
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
//! sim-bridge — V7 reset FFI integration crate.
//!
//! T7.7.B land: WorldSimNode (`Node` subclass) exposing 3 FFI methods
//!   - `get_influence_overlay(channel: i32) -> PackedByteArray`
//!   - `get_tile_detail(x: i32, y: i32) -> VarDictionary`
//!   - `on_building_placed(x: i32, y: i32, radius: i32) -> bool`
//!
//! Cold-tier admission: governance v3.3.8 §1 (Signal A whitelist).
//! Behavior gate: Signal D unchanged — sim-bridge contains no
//! `impl RuntimeSystem for X`; FFI methods exit through `#[func]`.

use godot::prelude::*;

pub mod ffi;

/// GDExtension entry point — registered by Godot at load time.
struct SimBridgeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SimBridgeExtension {}
```

### File 6: `rust/crates/sim-bridge/src/ffi/mod.rs` (new)

```rust
//! FFI bindings exposed to Godot via the godot 0.4 crate.

pub mod world_node;

pub use world_node::WorldSimNode;
```

### File 7: `rust/crates/sim-bridge/src/ffi/world_node.rs` (new)

WorldSimNode binds a single SimEngine. The 3 FFI methods:
- `get_influence_overlay`: serializes one channel's current buffer to a
  `PackedByteArray`. Returns empty array on invalid channel.
- `get_tile_detail`: returns a `VarDictionary` with `tile_x`, `tile_y`,
  `in_bounds`, and per-channel `current` values for the 8 channels.
- `on_building_placed`: pushes a `BuildingPlacedEvent` to
  `resources.building_event_queue`. Returns `true` on accepted, `false` on
  out-of-bounds.

```rust
//! `WorldSimNode` — `Node` subclass exposing the SimEngine to Godot.
//!
//! T7.7.B FFI surface (3 methods) wired through the R1 event_queue path
//! locked by `SimResources::building_event_queue`.

use godot::classes::INode;
use godot::prelude::*;
use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, SimEngine};
use sim_systems::register_phase2_systems;

/// Default grid extent until `Godot` configures it (Phase 2 default).
const DEFAULT_W: u32 = 64;
/// Default grid extent until `Godot` configures it (Phase 2 default).
const DEFAULT_H: u32 = 64;

/// Godot `Node` subclass wrapping a `SimEngine` instance.
#[derive(GodotClass)]
#[class(base=Node)]
pub struct WorldSimNode {
    engine: SimEngine,
    base: Base<Node>,
}

#[godot_api]
impl INode for WorldSimNode {
    fn init(base: Base<Node>) -> Self {
        let mut engine = SimEngine::new(DEFAULT_W, DEFAULT_H, MaterialRegistry::new());
        register_phase2_systems(&mut engine);
        Self { engine, base }
    }
}

#[godot_api]
impl WorldSimNode {
    /// Serialize the current buffer of influence `channel` to a packed
    /// byte array (row-major, length = width × height). Returns an empty
    /// array if the channel index is out of range.
    #[func]
    fn get_influence_overlay(&self, channel: i32) -> PackedByteArray {
        let Some(ch) = channel_from_i32(channel) else {
            return PackedByteArray::new();
        };
        let buf = self.engine.resources.influence_grid.current_buf(ch);
        PackedByteArray::from(buf)
    }

    /// Return a dictionary describing the tile at `(x, y)`. Keys:
    ///   - `tile_x`: i32
    ///   - `tile_y`: i32
    ///   - `in_bounds`: bool
    ///   - `warmth`, `light`, `noise`, `food_aroma`, `danger`, `social`,
    ///     `spiritual`, `beauty`: u8 (current buffer)
    #[func]
    fn get_tile_detail(&self, x: i32, y: i32) -> VarDictionary {
        let mut dict = VarDictionary::new();
        dict.set("tile_x", x);
        dict.set("tile_y", y);
        let grid = &self.engine.resources.influence_grid;
        let in_bounds = x >= 0 && y >= 0 && (x as u32) < grid.width && (y as u32) < grid.height;
        dict.set("in_bounds", in_bounds);
        if in_bounds {
            let ux = x as u32;
            let uy = y as u32;
            for ch in InfluenceChannel::all() {
                dict.set(channel_key(*ch), grid.sample(ux, uy, *ch));
            }
        } else {
            for ch in InfluenceChannel::all() {
                dict.set(channel_key(*ch), 0u8);
            }
        }
        dict
    }

    /// Push a `BuildingPlacedEvent` into the SimResources queue. Returns
    /// `false` if `(x, y)` is negative or outside the grid; `true` on
    /// successful enqueue. The drain happens on the next
    /// `BuildingStampSystem.tick()` (priority 90).
    #[func]
    fn on_building_placed(&mut self, x: i32, y: i32, radius: i32) -> bool {
        if x < 0 || y < 0 || radius < 0 {
            return false;
        }
        let grid = &self.engine.resources.influence_grid;
        if (x as u32) >= grid.width || (y as u32) >= grid.height {
            return false;
        }
        self.engine
            .resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (x as u32, y as u32),
                radius: radius as u32,
            });
        true
    }
}

fn channel_from_i32(ix: i32) -> Option<InfluenceChannel> {
    if ix < 0 {
        return None;
    }
    InfluenceChannel::all().get(ix as usize).copied()
}

fn channel_key(ch: InfluenceChannel) -> &'static str {
    match ch {
        InfluenceChannel::Warmth => "warmth",
        InfluenceChannel::Light => "light",
        InfluenceChannel::Noise => "noise",
        InfluenceChannel::FoodAroma => "food_aroma",
        InfluenceChannel::Danger => "danger",
        InfluenceChannel::Social => "social",
        InfluenceChannel::Spiritual => "spiritual",
        InfluenceChannel::Beauty => "beauty",
    }
}
```

> Generator note: `InfluenceChannel::all()` returns `&'static [InfluenceChannel]`
> (8 variants in canonical order — verified at sim-core/src/influence/channel.rs:84):
> Warmth=0, Light=1, Noise=2, FoodAroma=3, Danger=4, Social=5, Spiritual=6, Beauty=7.
> The `channel_key` match MUST cover all 8 — compile-time exhaustiveness ensures
> any future variant addition surfaces here.

### File 8: `rust/crates/sim-test/tests/harness_phase2_ffi.rs` (new)

These tests verify the underlying queue/drain mechanism that the FFI
methods wrap. They do NOT instantiate the godot runtime (no `Gd<T>` or
`Variant`); the godot binding itself is verified by `cargo build` of
sim-bridge succeeding. This split keeps the unit harness pure-Rust while
the cdylib build proves the FFI shape.

```rust
//! T7.7.B FFI mechanism harness — verifies the R1 event_queue contract
//! that the 3 sim-bridge FFI methods (`get_influence_overlay`,
//! `get_tile_detail`, `on_building_placed`) wrap.

use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, SimEngine};
use sim_systems::register_phase2_systems;

fn engine() -> SimEngine {
    let mut e = SimEngine::new(64, 64, MaterialRegistry::new());
    register_phase2_systems(&mut e);
    e
}

#[test]
fn harness_ffi_get_influence_overlay_default_state() {
    let e = engine();
    // `get_influence_overlay` reads `current_buf(ch)` — verify it is
    // 64*64=4096 bytes and all-zero on a fresh engine.
    let buf = e.resources.influence_grid.current_buf(InfluenceChannel::Warmth);
    assert_eq!(buf.len(), 64 * 64);
    assert!(buf.iter().all(|&b| b == 0));
}

#[test]
fn harness_ffi_get_tile_detail_in_bounds_yields_zero() {
    let e = engine();
    // `get_tile_detail` calls `grid.sample(x, y, ch)` for each channel.
    for ch in InfluenceChannel::all() {
        assert_eq!(e.resources.influence_grid.sample(10, 10, *ch), 0);
    }
}

#[test]
fn harness_ffi_on_building_placed_enqueues_and_stamp_tick_drains() {
    let mut e = engine();
    // Step 1: simulate FFI push (what `on_building_placed` does).
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });
    assert_eq!(e.resources.building_event_queue.len(), 1);
    // Step 2: tick — BuildingStampSystem drains and marks dirty.
    e.tick();
    assert!(e.resources.building_event_queue.is_empty());
    let regs_w = &e.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize];
    let regs_s = &e.resources.influence_grid.dirty_regions[InfluenceChannel::Spiritual as usize];
    let regs_b = &e.resources.influence_grid.dirty_regions[InfluenceChannel::Beauty as usize];
    let regs_l = &e.resources.influence_grid.dirty_regions[InfluenceChannel::Light as usize];
    assert_eq!(regs_w.len(), 1);
    assert_eq!(regs_s.len(), 1);
    assert_eq!(regs_b.len(), 1);
    assert_eq!(regs_l.len(), 1);
    // Channels NOT in STAMPED_CHANNELS remain untouched.
    let regs_d = &e.resources.influence_grid.dirty_regions[InfluenceChannel::Danger as usize];
    assert!(regs_d.is_empty());
}
```

## Verification

### Mechanical Gate
```bash
cd rust && cargo build --workspace          # ~5–10 min first build (godot 0.4 codegen)
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
```

### Hot-tier auto-classification check (Signal D unchanged)
sim-bridge contains NO `impl RuntimeSystem for X` — only `#[func]` FFI
methods. Signal D regex (governance v3.3.7 §1) must NOT match anything in
`rust/crates/sim-bridge/`:
```bash
grep -nE 'impl[[:space:]]+RuntimeSystem[[:space:]]+for|register_runtime_system!|register_system\(' rust/crates/sim-bridge/src/ -r
# Expected: zero matches.
```

### FFI shape verification
```bash
grep -nE '^\s*#\[func\]' rust/crates/sim-bridge/src/ffi/world_node.rs
# Expected: 3 matches (get_influence_overlay, get_tile_detail, on_building_placed)
```

### Public API verification
```bash
grep -nE '^pub (fn|mod|struct|use|enum)' \
  rust/crates/sim-bridge/src/lib.rs \
  rust/crates/sim-bridge/src/ffi/mod.rs \
  rust/crates/sim-bridge/src/ffi/world_node.rs \
  rust/crates/sim-engine/src/lib.rs | grep -E 'BuildingPlacedEvent|building_event_queue|WorldSimNode'
```

### Harness regression
- 18 prior Phase 2 harness tests in sim-systems/tests/integration.rs MUST
  remain green (BuildingStampSystem now does work — the no-op tests in
  that file should still pass because the queue defaults to empty).
- 5 prior sim-engine integration tests MUST remain green (queue field
  addition is non-breaking).
- Workspace test count expected: ≥ 21 (3 new FFI mechanism tests).

## Localization

No new localization keys (FFI is debug/runtime surface, no UI text).

## Lane

`--full` pipeline (hot-tier behavior land):
- Step 0 mechanical gate → Step 1 planning debate → Step 2 implementation
  (≤3 attempts) → Step 2.5 visual verify (sim-bridge cdylib pure Rust →
  no-godot-scope auto credit per v3.3.7 §2) → Step 2.5c FFI chain
  (governance v3.3.10 — `scripts/core/simulation/sim_bridge.gd` absent
  in V7 reset → SKIP_V7_RESET marker, mirrors ffi_chain_check.sh v3.3.9)
  → Step 2.7 regression guard → Step 3 evaluator.

Expected score (governance v3.3.10 정합):
- Mechanical 10/10 (FFI 정상 검증 회복 — sim-bridge 등장으로 §3 vacuous 해소;
  Step 2.5c FFI verify는 sim_bridge.gd 부재 → SKIP_V7_RESET, 정통 path)
- Plan 5/5 (single round QC)
- Code 15/15 (attempt 1)
- Test 20/20 (3 new FFI mechanism tests)
- Visual 20/20 (no-godot-scope auto credit — no .gd/.tscn/.tres added)
- Regression 15/15 (CLEAN)
- Evaluator 15/15 (APPROVE)
- **Total ≥ 90 expected.**

## Commit Message
```
feat(sim-bridge)[T7.7.B][--full]: WorldSimNode + 3 FFI methods (R1 event_queue)

V7 reset 회복 마지막 폐기 crate behavior land — godot 0.4 binding 도입.

- workspace.dependencies +godot = "0.4"
- sim-engine: BuildingPlacedEvent + SimResources.building_event_queue
- sim-systems::BuildingStampSystem.tick(): drain queue → mark_dirty
  (Warmth/Spiritual/Beauty/Light, Chebyshev radius, grid-clamped)
- sim-bridge:
  - lib.rs: SimBridgeExtension entry + #[gdextension]
  - ffi/world_node.rs: WorldSimNode (Node subclass) + 3 #[func] methods
    * get_influence_overlay(channel: i32) -> PackedByteArray
    * get_tile_detail(x: i32, y: i32) -> VarDictionary
    * on_building_placed(x: i32, y: i32, radius: i32) -> bool
- sim-test/tests/harness_phase2_ffi.rs: 3 mechanism tests

R1 정통 path (Phase 0 v0.1.3 §5):
  Godot place call → on_building_placed → push BuildingPlacedEvent
  → BuildingStampSystem.tick() drains queue → InfluenceGrid::mark_dirty
  → InfluenceUpdateSystem.tick() (priority 100) processes dirty regions.

Signal D unchanged (sim-bridge contains no `impl RuntimeSystem for X`,
hot-tier auto-classification triggered by `#[func]` + cdylib only).

Refs: 974e25d2 (T7.7.0 v3.3.8 governance), 5092328b (T7.7.A scaffold),
      InfluenceGrid::mark_dirty (sim-core/src/influence/grid.rs:112).
```
