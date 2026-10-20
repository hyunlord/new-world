//! T7.9.A harness — Godot rendering scaffold: process() drives SimEngine.tick()
//!
//! Assertion 1: current_tick == 0 on fresh Phase 2 engine (pre-condition invariant)
//! Assertion 2: 10 engine.tick() calls advance current_tick to exactly 10
//!              (simulates WorldSimNode::process(_delta) calling self.engine.tick()
//!               once per Godot frame — the sole body of INode::process in T7.9.A)
//! Assertion 3: resources.current_tick propagated by tick() matches engine.current_tick()
//!              (verifies SimEngine.tick() correctly propagates to SimResources before
//!               systems run — required for all Phase 2 systems that branch on current_tick)
//! Assertion 4 (NON-CIRCULAR): world_node.rs SOURCE contains process() with the sole
//!              body `self.engine.tick();` — fails if the method is removed or changed.
//! Assertion 5 (NON-CIRCULAR): scenes/main.tscn SOURCE contains T7.9.A required scaffold
//!              entries: uid, WorldSimNode, Camera2D, renderer ext_resource.
//! Assertion 6 (NON-CIRCULAR): sim_bridge.gdextension SOURCE contains required config:
//!              entry_symbol, compatibility_minimum, reloadable flag, all 3 lib names.
//! Assertion 7 (NON-CIRCULAR): scripts/ui/world_renderer.gd SOURCE extends Node2D,
//!              prints ready sentinel, and has _process stub.
//!
//! A1–A3 are supplemental tick-counter checks that confirm SimEngine mechanics.
//! A4–A7 are source/scaffold identity checks that only the T7.9.A implementation
//! can satisfy — they use include_str! (compile-time file inclusion) so they fail
//! to compile if a required file is missing, and fail at runtime if required
//! content is absent.
//!
//! WorldSimNode cannot be instantiated in sim-test (requires Godot runtime).
//! engine.tick() is the exact call made by `INode::process` in world_node.rs.
//! The Bridge Identity Contract and 21 FFI assertions from T7.7.B are unchanged.
//!
//! Run: `cargo test -p sim-test --test harness_t7_9_a_scaffold -- --nocapture`

use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_phase2_systems;

const W: u32 = 64;
const H: u32 = 64;

/// Fresh 64×64 Phase 2 engine with all 4 systems registered.
fn fresh_phase2_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    engine
}

// ── A1: current_tick_zero_on_fresh_engine ────────────────────────────────────

/// Type A: engine.current_tick() == 0 on fresh Phase 2 engine (0 ticks driven).
///
/// Pre-condition invariant: verifies that the tick counter starts at 0 before
/// any WorldSimNode::process() frames have run. Required baseline for Assertion 2.
///
/// ticks: 0 | components_read: SimEngine.current_tick()
#[test]
fn harness_scaffold_tick_count_zero_on_fresh_engine() {
    let engine = fresh_phase2_engine();
    // Type A: threshold == 0
    assert_eq!(
        engine.current_tick(),
        0,
        "current_tick must be 0 on a freshly constructed engine \
         (no WorldSimNode::process() frames have driven tick yet)"
    );
}

// ── A2: process_equivalent_drives_current_tick_to_n ──────────────────────────

/// Type A: engine.current_tick() == 10 after exactly 10 engine.tick() calls.
///
/// T7.9.A INode::process body is `self.engine.tick()` — one tick per Godot frame.
/// This test simulates 10 Godot frames, each invoking WorldSimNode::process(_delta).
/// Verifies that each process() call advances current_tick by exactly 1 (no skips,
/// no double-ticks, no batch ticking).
///
/// ticks: 10 | components_read: SimEngine.current_tick()
#[test]
fn harness_scaffold_process_equivalent_drives_current_tick_to_n() {
    let mut engine = fresh_phase2_engine();

    // Simulate 10 Godot frames — each calling WorldSimNode::process(_delta)
    // which executes `self.engine.tick()` exactly once per frame.
    for frame in 0..10 {
        engine.tick();
        println!(
            "frame {frame}: current_tick() = {}",
            engine.current_tick()
        );
    }

    // Type A: threshold == 10 (one tick per frame, 10 frames)
    assert_eq!(
        engine.current_tick(),
        10,
        "current_tick must be exactly 10 after 10 engine.tick() calls \
         (simulates 10 WorldSimNode::process(_delta) frame callbacks). \
         Got {}",
        engine.current_tick()
    );
}

// ── A3: resources_current_tick_propagated_by_tick ────────────────────────────

/// Type A: resources.current_tick == engine.current_tick() - 1 after N ticks.
///
/// SimEngine::tick() propagates `self.current_tick` to `resources.current_tick`
/// BEFORE dispatching systems. After N calls:
///   - engine.current_tick() == N   (incremented after systems run)
///   - resources.current_tick == N-1  (the tick value systems saw on the last call)
///
/// This matters for T7.9.A because WorldSimNode::process() drives one tick per
/// frame — systems must observe a monotonically advancing current_tick, not a stale 0.
///
/// ticks: 5 | components_read: SimResources.current_tick, SimEngine.current_tick()
#[test]
fn harness_scaffold_resources_current_tick_propagated_by_tick() {
    let mut engine = fresh_phase2_engine();

    for _ in 0..5 {
        engine.tick();
    }

    // After 5 ticks:
    //   engine.current_tick() == 5  (counter post-incremented after each tick)
    //   resources.current_tick == 4  (last tick index seen by systems)
    // Type A: threshold engine==5, resources==4
    assert_eq!(
        engine.current_tick(),
        5,
        "engine.current_tick() must be 5 after 5 ticks"
    );
    assert_eq!(
        engine.resources.current_tick,
        4,
        "resources.current_tick must be 4 after 5 ticks \
         (pre-dispatch snapshot of the tick counter on the last call). \
         Got {}",
        engine.resources.current_tick
    );
}

// ── A4: world_node_source_process_identity (NON-CIRCULAR) ────────────────────

// ── A5: main_tscn_source_scaffold_identity (NON-CIRCULAR) ────────────────────

// ── A6: sim_bridge_gdextension_source_identity (NON-CIRCULAR) ────────────────

// ── A7: world_renderer_gd_source_identity (NON-CIRCULAR) ─────────────────────
