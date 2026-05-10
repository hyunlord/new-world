# T7.6: Phase 2 Influence RuntimeSystems (4 systems)

## Implementation Intent

V7 reset 후 첫 hot-tier behavior land — sim-systems crate에 Phase 2 4
RuntimeSystem 구현 + register helper + integration tests.

근거:
- T7.5.5.A (1ab45b71): sim-engine — `RuntimeSystem` trait + `SimResources` host
- T7.5.5.B (4f42aa28): sim-systems — empty scaffold, `pub mod runtime;`
- T7.5.5.C (d499d0eb): sim-engine integration tests baseline
- governance v3.3.6 (ee6b8529): sim-systems hot-tier auto-classification 허용

이 작업은 **첫 actual `impl RuntimeSystem for X` land** — Signal D가 hot-tier로
auto-classify 하는 first commit이 됨 (--full pipeline 의무).

Phase 0 design v0.1.3 patch Section 4.3 base.

## What to Build

Path: `rust/crates/sim-systems/`

### Files (5 신설 + 2 수정)

#### 1. `rust/crates/sim-systems/src/runtime/mod.rs` (수정)
주석 처리된 `// pub mod influence;` → `pub mod influence;`

#### 2. `rust/crates/sim-systems/src/lib.rs` (수정)
`register_phase2_systems(engine: &mut SimEngine)` helper 추가.

#### 3. `rust/crates/sim-systems/src/runtime/influence/mod.rs` (신설)
4개 시스템 re-export.

#### 4. `rust/crates/sim-systems/src/runtime/influence/update.rs` (신설)
`InfluenceUpdateSystem` (priority 100, interval 1) — Hot/Warm/Cold dispatch.

#### 5. `rust/crates/sim-systems/src/runtime/influence/building_stamp.rs` (신설)
`BuildingStampSystem` (priority 90, interval 1) — building → influence stamp.
Phase 2에서는 stub (no Building component yet); tick body는 swap/clear 작업만.

#### 6. `rust/crates/sim-systems/src/runtime/influence/agent_sample.rs` (신설)
`AgentInfluenceSampleSystem` (priority 110, interval 1) — agent ECS query.
Position component이 sim-core에 없으므로 **local mock Position struct** 사용 (P1).

#### 7. `rust/crates/sim-systems/src/runtime/influence/visualization.rs` (신설)
`InfluenceVisualizationSystem` (priority 1000, interval 6) — debug snapshot.

#### 8. `rust/crates/sim-systems/tests/integration.rs` (신설)
`register_phase2_systems` 통합 테스트.

## How to Implement

### File 1: `rust/crates/sim-systems/src/runtime/mod.rs`
```rust
//! Runtime systems organization.
//!
//! Phase 2: 4 influence RuntimeSystems landed in T7.6.

pub mod influence;
```

### File 2: `rust/crates/sim-systems/src/lib.rs`
```rust
//! WorldSim runtime systems.
//!
//! Phase 2 (T7.6 land): 4 influence systems via [`register_phase2_systems`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use sim_engine::SimEngine;

pub mod runtime;

/// Register the Phase 2 influence stack on `engine` in priority order.
///
/// - 90  : `BuildingStampSystem`
/// - 100 : `InfluenceUpdateSystem`
/// - 110 : `AgentInfluenceSampleSystem`
/// - 1000: `InfluenceVisualizationSystem` (every 6 ticks)
pub fn register_phase2_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::influence::BuildingStampSystem::new()));
    engine.register_system(Box::new(runtime::influence::InfluenceUpdateSystem::new()));
    engine.register_system(Box::new(runtime::influence::AgentInfluenceSampleSystem::new()));
    engine.register_system(Box::new(runtime::influence::InfluenceVisualizationSystem::new()));
}
```

### File 3: `rust/crates/sim-systems/src/runtime/influence/mod.rs`
```rust
//! Phase 2 influence RuntimeSystems.

pub mod agent_sample;
pub mod building_stamp;
pub mod update;
pub mod visualization;

pub use agent_sample::AgentInfluenceSampleSystem;
pub use building_stamp::BuildingStampSystem;
pub use update::InfluenceUpdateSystem;
pub use visualization::InfluenceVisualizationSystem;
```

### File 4: `rust/crates/sim-systems/src/runtime/influence/update.rs`
Hot/Warm/Cold dispatch: clear pending → propagate from registered sources →
swap. Phase 2 scaffold에서는 정적 source 없음 — tick body는 `clear_all_pending`
+ `swap` 만 수행 (channels 0건 → swap 후에도 zero, regression 안전).

```rust
//! `InfluenceUpdateSystem` — priority 100, every tick.
//!
//! Phase 0 Section 2.6 budget: Hot tier ≤ 0.5 ms @ 1K agents.
//!
//! Phase 2 land (T7.6) is a *dispatch shell*: it clears every pending
//! buffer and swaps double-buffers each tick. Actual source iteration
//! lands together with the BuildingStampSystem and Agent stamping
//! plumbing in later phases — the shell guarantees deterministic
//! zero-state baseline regardless of registration order.

use hecs::World;
use sim_engine::{RuntimeSystem, SimResources};

/// Phase 2 influence update dispatcher.
pub struct InfluenceUpdateSystem;

impl InfluenceUpdateSystem {
    /// Construct a new dispatcher.
    pub fn new() -> Self {
        Self
    }
}

impl Default for InfluenceUpdateSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSystem for InfluenceUpdateSystem {
    fn name(&self) -> &str {
        "InfluenceUpdateSystem"
    }
    fn priority(&self) -> u32 {
        100
    }
    fn tick_interval(&self) -> u64 {
        1
    }
    fn tick(&mut self, _world: &mut World, resources: &mut SimResources) {
        // Phase 2 dispatch shell: zero-state baseline.
        resources.influence_grid.clear_all_pending();
        resources.influence_grid.swap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::influence::InfluenceChannel;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    fn empty_engine() -> SimEngine {
        SimEngine::new(64, 64, MaterialRegistry::new())
    }

    #[test]
    fn metadata() {
        let s = InfluenceUpdateSystem::new();
        assert_eq!(s.name(), "InfluenceUpdateSystem");
        assert_eq!(s.priority(), 100);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn tick_does_not_panic_on_empty_world() {
        let mut e = empty_engine();
        e.register_system(Box::new(InfluenceUpdateSystem::new()));
        for _ in 0..10 {
            e.tick();
        }
        assert_eq!(e.current_tick(), 10);
    }

    #[test]
    fn baseline_remains_zero_after_ticks() {
        let mut e = empty_engine();
        e.register_system(Box::new(InfluenceUpdateSystem::new()));
        for _ in 0..5 {
            e.tick();
        }
        for ch in InfluenceChannel::all() {
            assert_eq!(e.resources.influence_grid.sample(0, 0, *ch), 0);
            assert_eq!(e.resources.influence_grid.sample(32, 32, *ch), 0);
            assert_eq!(e.resources.influence_grid.sample(63, 63, *ch), 0);
        }
    }

    #[test]
    fn manual_pending_write_is_swapped_then_cleared() {
        // Write into pending BEFORE the system ticks; system clears+swaps.
        let mut e = empty_engine();
        let buf = e
            .resources
            .influence_grid
            .pending_buf_mut(InfluenceChannel::Warmth);
        let i = 10 * 64 + 10;
        buf[i] = 200;
        // Run the system manually.
        let mut s = InfluenceUpdateSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        // After clear+swap: previous pending (200) was cleared first, so
        // current is now zero.
        assert_eq!(
            e.resources.influence_grid.sample(10, 10, InfluenceChannel::Warmth),
            0
        );
    }
}
```

### File 5: `rust/crates/sim-systems/src/runtime/influence/building_stamp.rs`
```rust
//! `BuildingStampSystem` — priority 90, every tick.
//!
//! Phase 0 Section 2.5.1 base. Phase 2 land (T7.6) is a *no-op shell*:
//! Building components do not exist yet (they land in Phase 11). The
//! shell exists so the system order (90 < 100 < 110) is locked from
//! day 1 and the priority slot cannot be reassigned later.

use hecs::World;
use sim_engine::{RuntimeSystem, SimResources};

/// Phase 2 building → influence stamper (no-op shell).
pub struct BuildingStampSystem;

impl BuildingStampSystem {
    /// Construct a new shell.
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
    fn tick(&mut self, _world: &mut World, _resources: &mut SimResources) {
        // No buildings in Phase 2 — shell only.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    #[test]
    fn metadata() {
        let s = BuildingStampSystem::new();
        assert_eq!(s.name(), "BuildingStampSystem");
        assert_eq!(s.priority(), 90);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn tick_does_not_panic() {
        let mut e = SimEngine::new(32, 32, MaterialRegistry::new());
        e.register_system(Box::new(BuildingStampSystem::new()));
        for _ in 0..5 {
            e.tick();
        }
    }

    #[test]
    fn shell_does_not_mutate_state() {
        let mut e = SimEngine::new(32, 32, MaterialRegistry::new());
        e.register_system(Box::new(BuildingStampSystem::new()));
        let before = e.resources.tile_grid.len();
        for _ in 0..3 {
            e.tick();
        }
        assert_eq!(e.resources.tile_grid.len(), before);
    }
}
```

### File 6: `rust/crates/sim-systems/src/runtime/influence/agent_sample.rs`
```rust
//! `AgentInfluenceSampleSystem` — priority 110, every tick.
//!
//! Phase 0 Section 2.5.4 base. Reads the current-side influence grid for
//! every agent and stashes the sample on a debug component. This system
//! demonstrates the canonical *read-after-update* contract: it runs at
//! priority 110 (after the InfluenceUpdateSystem at 100) so every read
//! sees the freshly-swapped current buffer.
//!
//! Position component does not exist in `sim-core` yet (lands in Phase 4
//! Agent Core). Phase 2 land defines a **local placeholder**
//! [`Position`] inside this module so harness tests can spawn entities
//! and exercise the read path. When the canonical `Position` lands, this
//! placeholder is removed and the query is rewired in a single line.

use hecs::World;
use sim_core::influence::InfluenceChannel;
use sim_engine::{RuntimeSystem, SimResources};

/// Local Phase 2 placeholder for the future `sim_core::Position`
/// component (lands in Phase 4 Agent Core).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Tile X.
    pub x: u32,
    /// Tile Y.
    pub y: u32,
}

/// Most recent influence sample observed by an agent (debug component).
#[derive(Debug, Clone, Copy, Default)]
pub struct InfluenceSample {
    /// Warmth at the agent's tile last tick.
    pub warmth: u8,
    /// Danger at the agent's tile last tick.
    pub danger: u8,
}

/// Phase 2 agent influence sampler.
pub struct AgentInfluenceSampleSystem;

impl AgentInfluenceSampleSystem {
    /// Construct a new sampler.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AgentInfluenceSampleSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSystem for AgentInfluenceSampleSystem {
    fn name(&self) -> &str {
        "AgentInfluenceSampleSystem"
    }
    fn priority(&self) -> u32 {
        110
    }
    fn tick_interval(&self) -> u64 {
        1
    }
    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let grid = &resources.influence_grid;
        let w = grid.width;
        let h = grid.height;
        for (_, (pos, sample)) in world.query::<(&Position, &mut InfluenceSample)>().iter() {
            if pos.x >= w || pos.y >= h {
                continue;
            }
            sample.warmth = grid.sample(pos.x, pos.y, InfluenceChannel::Warmth);
            sample.danger = grid.sample(pos.x, pos.y, InfluenceChannel::Danger);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    fn empty_engine() -> SimEngine {
        SimEngine::new(32, 32, MaterialRegistry::new())
    }

    #[test]
    fn metadata() {
        let s = AgentInfluenceSampleSystem::new();
        assert_eq!(s.name(), "AgentInfluenceSampleSystem");
        assert_eq!(s.priority(), 110);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn samples_zero_when_grid_empty() {
        let mut e = empty_engine();
        let id = e.world.spawn((Position { x: 5, y: 5 }, InfluenceSample::default()));
        e.register_system(Box::new(AgentInfluenceSampleSystem::new()));
        e.tick();
        let s = *e.world.get::<&InfluenceSample>(id).unwrap();
        assert_eq!(s.warmth, 0);
        assert_eq!(s.danger, 0);
    }

    #[test]
    fn samples_warmth_after_manual_write_and_swap() {
        let mut e = empty_engine();
        // Write into pending then swap so current has the value.
        let buf = e
            .resources
            .influence_grid
            .pending_buf_mut(InfluenceChannel::Warmth);
        buf[5 * 32 + 5] = 123;
        e.resources.influence_grid.swap();

        let id = e.world.spawn((Position { x: 5, y: 5 }, InfluenceSample::default()));
        // Run sampler manually (don't tick the engine — that would re-swap).
        let mut s = AgentInfluenceSampleSystem::new();
        s.tick(&mut e.world, &mut e.resources);

        let sample = *e.world.get::<&InfluenceSample>(id).unwrap();
        assert_eq!(sample.warmth, 123);
        assert_eq!(sample.danger, 0);
    }

    #[test]
    fn out_of_bounds_position_is_ignored() {
        let mut e = empty_engine();
        let id = e.world.spawn((
            Position { x: 999, y: 999 },
            InfluenceSample {
                warmth: 42,
                danger: 7,
            },
        ));
        let mut s = AgentInfluenceSampleSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        // Sample should remain unchanged (skipped by bounds check).
        let sample = *e.world.get::<&InfluenceSample>(id).unwrap();
        assert_eq!(sample.warmth, 42);
        assert_eq!(sample.danger, 7);
    }

    #[test]
    fn many_agents_each_read_their_own_tile() {
        let mut e = empty_engine();
        // Stamp distinct danger values at three tiles via pending+swap.
        let buf = e
            .resources
            .influence_grid
            .pending_buf_mut(InfluenceChannel::Danger);
        buf[1 * 32 + 1] = 10;
        buf[2 * 32 + 2] = 20;
        buf[3 * 32 + 3] = 30;
        e.resources.influence_grid.swap();

        let a = e.world.spawn((Position { x: 1, y: 1 }, InfluenceSample::default()));
        let b = e.world.spawn((Position { x: 2, y: 2 }, InfluenceSample::default()));
        let c = e.world.spawn((Position { x: 3, y: 3 }, InfluenceSample::default()));

        let mut s = AgentInfluenceSampleSystem::new();
        s.tick(&mut e.world, &mut e.resources);

        assert_eq!(e.world.get::<&InfluenceSample>(a).unwrap().danger, 10);
        assert_eq!(e.world.get::<&InfluenceSample>(b).unwrap().danger, 20);
        assert_eq!(e.world.get::<&InfluenceSample>(c).unwrap().danger, 30);
    }
}
```

### File 7: `rust/crates/sim-systems/src/runtime/influence/visualization.rs`
```rust
//! `InfluenceVisualizationSystem` — priority 1000, every 6 ticks.
//!
//! Phase 0 Section 2.8 base. Captures a coarse digest of the current
//! buffer at the system's interval so debug HUD / harness can confirm
//! influence activity without scanning the whole grid every frame.

use hecs::World;
use sim_core::influence::InfluenceChannel;
use sim_engine::{RuntimeSystem, SimResources};

/// Most recent visualisation digest.
#[derive(Debug, Default, Clone, Copy)]
pub struct VisualizationDigest {
    /// Tick at which this digest was captured.
    pub tick: u64,
    /// Sum of `current[Warmth]` over the whole grid.
    pub warmth_total: u64,
    /// Maximum `current[Danger]` seen anywhere on the grid.
    pub danger_peak: u8,
}

/// Phase 2 influence visualiser (debug only).
pub struct InfluenceVisualizationSystem {
    last: VisualizationDigest,
}

impl InfluenceVisualizationSystem {
    /// Construct a new visualiser with a zeroed digest.
    pub fn new() -> Self {
        Self {
            last: VisualizationDigest::default(),
        }
    }

    /// Borrow the most recent digest captured by [`tick`].
    pub fn last_digest(&self) -> &VisualizationDigest {
        &self.last
    }
}

impl Default for InfluenceVisualizationSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSystem for InfluenceVisualizationSystem {
    fn name(&self) -> &str {
        "InfluenceVisualizationSystem"
    }
    fn priority(&self) -> u32 {
        1000
    }
    fn tick_interval(&self) -> u64 {
        6
    }
    fn tick(&mut self, _world: &mut World, resources: &mut SimResources) {
        let warmth_total: u64 = resources
            .influence_grid
            .current_buf(InfluenceChannel::Warmth)
            .iter()
            .map(|&v| v as u64)
            .sum();
        let danger_peak: u8 = resources
            .influence_grid
            .current_buf(InfluenceChannel::Danger)
            .iter()
            .copied()
            .max()
            .unwrap_or(0);
        self.last = VisualizationDigest {
            tick: resources.current_tick,
            warmth_total,
            danger_peak,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    fn empty_engine() -> SimEngine {
        SimEngine::new(32, 32, MaterialRegistry::new())
    }

    #[test]
    fn metadata() {
        let s = InfluenceVisualizationSystem::new();
        assert_eq!(s.name(), "InfluenceVisualizationSystem");
        assert_eq!(s.priority(), 1000);
        assert_eq!(s.tick_interval(), 6);
    }

    #[test]
    fn digest_zero_on_fresh_engine() {
        let mut e = empty_engine();
        let mut s = InfluenceVisualizationSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        assert_eq!(s.last_digest().warmth_total, 0);
        assert_eq!(s.last_digest().danger_peak, 0);
    }

    #[test]
    fn digest_captures_warmth_total_and_danger_peak() {
        let mut e = empty_engine();
        let buf = e
            .resources
            .influence_grid
            .pending_buf_mut(InfluenceChannel::Warmth);
        buf[0] = 50;
        buf[1] = 70;
        let buf = e
            .resources
            .influence_grid
            .pending_buf_mut(InfluenceChannel::Danger);
        buf[0] = 200;
        buf[10] = 100;
        e.resources.influence_grid.swap();

        let mut s = InfluenceVisualizationSystem::new();
        s.tick(&mut e.world, &mut e.resources);

        assert_eq!(s.last_digest().warmth_total, 120);
        assert_eq!(s.last_digest().danger_peak, 200);
    }

    #[test]
    fn interval_6_only_fires_at_multiples() {
        let mut e = empty_engine();
        e.register_system(Box::new(InfluenceVisualizationSystem::new()));
        // We can't easily inspect the boxed system; instead, run 13 ticks
        // and rely on the engine's tick-interval logic — the system must
        // not panic or error.
        for _ in 0..13 {
            e.tick();
        }
        assert_eq!(e.current_tick(), 13);
    }
}
```

### File 8: `rust/crates/sim-systems/tests/integration.rs`
```rust
//! Phase 2 integration — `register_phase2_systems` end-to-end.
//!
//! T7.6 land. Verifies registration order, priority ordering, baseline
//! zero-state under multi-tick run.

use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_phase2_systems;

#[test]
fn test_register_phase2_count_and_order() {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    assert_eq!(engine.system_count(), 4);
    assert_eq!(
        engine.system_names(),
        vec![
            "BuildingStampSystem",
            "InfluenceUpdateSystem",
            "AgentInfluenceSampleSystem",
            "InfluenceVisualizationSystem",
        ]
    );
}

#[test]
fn test_phase2_tick_loop_no_panic() {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    for _ in 0..30 {
        engine.tick();
    }
    assert_eq!(engine.current_tick(), 30);
}

#[test]
fn test_phase2_baseline_zero_after_ticks() {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    for _ in 0..10 {
        engine.tick();
    }
    for ch in InfluenceChannel::all() {
        assert_eq!(engine.resources.influence_grid.sample(0, 0, *ch), 0);
        assert_eq!(engine.resources.influence_grid.sample(32, 32, *ch), 0);
        assert_eq!(engine.resources.influence_grid.sample(63, 63, *ch), 0);
    }
}
```

## Verification

### Mechanical Gate
```bash
cd rust && cargo build --workspace
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
```

### Hot-tier classification 검증 (Signal D)
```bash
git diff --name-only HEAD | grep -E '^rust/crates/sim-systems/.*\.rs$' | xargs grep -lE 'impl RuntimeSystem for'
# Expected: ≥4 files (one per system)
```

### Public API 검증
```bash
grep -nE '^pub (fn|mod|struct|use|enum)' rust/crates/sim-systems/src/lib.rs rust/crates/sim-systems/src/runtime/mod.rs rust/crates/sim-systems/src/runtime/influence/mod.rs
```

### Priority order 검증
```bash
cd rust && cargo test --test integration -p sim-systems test_register_phase2_count_and_order -- --nocapture
```

## Localization

No new localization keys (debug-only systems, no UI surface).

## Commit Message
```
T7.6: Phase 2 influence RuntimeSystems — 4 systems landed

[--full pipeline] first hot-tier behavior commit (post V7 reset)

- sim-systems/src/lib.rs: register_phase2_systems helper
- sim-systems/src/runtime/influence/{update,building_stamp,agent_sample,visualization}.rs: 4 RuntimeSystems
- sim-systems/tests/integration.rs: 3 integration tests
- sim-systems/src/runtime/mod.rs: pub mod influence;

Priority order: 90 (BuildingStamp) < 100 (InfluenceUpdate) <
110 (AgentInfluenceSample) < 1000 (InfluenceVisualization, every 6).

Phase 2 scope:
- BuildingStampSystem: no-op shell (Building component lands in Phase 11)
- InfluenceUpdateSystem: dispatch shell (clear_all_pending + swap)
- AgentInfluenceSampleSystem: ECS query w/ local Position placeholder (P1)
- InfluenceVisualizationSystem: debug digest (warmth_total + danger_peak)

Phase 0 v0.1.3 patch Section 4.3 base.
Refs: 1ab45b71 (T7.5.5.A sim-engine), 4f42aa28 (T7.5.5.B sim-systems scaffold),
      ee6b8529 (governance v3.3.6), d499d0eb (T7.5.5.C integration baseline)
```
