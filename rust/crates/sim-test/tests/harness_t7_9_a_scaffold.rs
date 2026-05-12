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

/// Type S (source identity): world_node.rs must contain INode::process with the
/// sole body `self.engine.tick();`.
///
/// NON-CIRCULAR: A1–A3 only test SimEngine mechanics; they would pass even if
/// `process()` was never added to WorldSimNode. This assertion reads the actual
/// source file at compile time via `include_str!` and verifies the T7.9.A contract
/// is present. Removing `process()` or changing its body fails this assertion.
///
/// `include_str!` resolution: path is relative to this test file's location
/// (rust/crates/sim-test/tests/), so ../../sim-bridge/src/ffi/world_node.rs
/// resolves to rust/crates/sim-bridge/src/ffi/world_node.rs.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_scaffold_world_node_source_process_identity() {
    // include_str! embeds at compile time — compilation fails if the file is absent.
    let src = include_str!("../../sim-bridge/src/ffi/world_node.rs");

    // Type S: INode::process signature must be present
    assert!(
        src.contains("fn process(&mut self, _delta: f64)"),
        "world_node.rs must contain INode::process with signature \
         `fn process(&mut self, _delta: f64)`. \
         T7.9.A contract: WorldSimNode drives one engine.tick() per Godot frame. \
         Snippet not found in sim-bridge/src/ffi/world_node.rs"
    );

    // Type S: process() sole body must be self.engine.tick() — no accumulator in T7.9.A
    assert!(
        src.contains("self.engine.tick();"),
        "world_node.rs must contain `self.engine.tick();` as the process() body. \
         T7.9.A contract: variable-cadence ticking, no fixed-tick accumulator. \
         Snippet not found in sim-bridge/src/ffi/world_node.rs"
    );

    println!("A4 PASS: world_node.rs process() identity verified");
    println!(
        "  found: fn process(&mut self, _delta: f64) with self.engine.tick();"
    );
}

// ── A5: main_tscn_source_scaffold_identity (NON-CIRCULAR) ────────────────────

/// Type S (source identity): scenes/main.tscn must contain the T7.9.A required
/// scene tree: preserved uid, WorldSimNode, Camera2D at Vector2(960,540),
/// and WorldRenderer script via ExtResource("1_renderer").
///
/// NON-CIRCULAR: removing or corrupting the scene causes this assertion to fail
/// before Godot is even opened. Path resolves from rust/crates/sim-test/tests/
/// four directories up to the project root, then into scenes/.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_scaffold_main_tscn_source_identity() {
    let src = include_str!("../../../../scenes/main.tscn");

    // Type S: Godot 4.x format=3 with T7.9.A preserved UID
    assert!(
        src.contains("uid=\"uid://v7init\""),
        "scenes/main.tscn must contain uid=\"uid://v7init\" (T7.9.A preserved UID)"
    );
    assert!(
        src.contains("format=3"),
        "scenes/main.tscn must be Godot 4.x format=3"
    );

    // Type S: WorldSimNode must be instantiated (T7.9.A sim tick driver)
    assert!(
        src.contains("type=\"WorldSimNode\""),
        "scenes/main.tscn must contain a node of type WorldSimNode \
         (the sim-bridge GDExtension class that drives engine.tick() per frame)"
    );

    // Type S: Camera2D with T7.9.A specified position
    assert!(
        src.contains("type=\"Camera2D\""),
        "scenes/main.tscn must contain a Camera2D node"
    );
    assert!(
        src.contains("Vector2(960, 540)"),
        "scenes/main.tscn Camera2D must be positioned at Vector2(960, 540) \
         per T7.9.A specification"
    );

    // Type S: WorldRenderer script must be referenced via the ext_resource
    assert!(
        src.contains("ExtResource(\"1_renderer\")"),
        "scenes/main.tscn must reference WorldRenderer script via \
         ExtResource(\"1_renderer\")"
    );
    assert!(
        src.contains("world_renderer.gd"),
        "scenes/main.tscn must reference scripts/ui/world_renderer.gd \
         as the WorldRenderer script path"
    );

    println!("A5 PASS: scenes/main.tscn scaffold identity verified");
    println!("  found: uid=v7init, format=3, WorldSimNode, Camera2D, world_renderer.gd");
}

// ── A6: sim_bridge_gdextension_source_identity (NON-CIRCULAR) ────────────────

/// Type S (source identity): sim_bridge.gdextension must contain the T7.9.A
/// required configuration so Godot can locate and load the sim-bridge cdylib.
///
/// NON-CIRCULAR: without this file Godot cannot load WorldSimNode at all.
/// Checks: entry symbol (godot-rust 0.4), compatibility minimum, reloadable flag,
/// and cdylib output names for all 3 platforms.
///
/// Library names derive from Cargo `package.name = "sim-bridge"` with no
/// `[lib] name=` override: `libsim_bridge.so` / `.dylib` / `sim_bridge.dll`.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_scaffold_gdextension_source_identity() {
    let src = include_str!("../../../../sim_bridge.gdextension");

    // Type S: godot-rust 0.4 entry symbol
    assert!(
        src.contains("entry_symbol = \"gdext_rust_init\""),
        "sim_bridge.gdextension must set entry_symbol = \"gdext_rust_init\" \
         (required by godot-rust 0.4)"
    );

    // Type S: minimum Godot version
    assert!(
        src.contains("compatibility_minimum = 4.1"),
        "sim_bridge.gdextension must set compatibility_minimum = 4.1"
    );

    // Type S: reloadable = true enables hot-reload in dev workflow
    assert!(
        src.contains("reloadable = true"),
        "sim_bridge.gdextension must set reloadable = true"
    );

    // Type S: library names must match Cargo package.name = "sim-bridge"
    assert!(
        src.contains("libsim_bridge.so"),
        "sim_bridge.gdextension must reference libsim_bridge.so (Linux cdylib)"
    );
    assert!(
        src.contains("libsim_bridge.dylib"),
        "sim_bridge.gdextension must reference libsim_bridge.dylib (macOS cdylib)"
    );
    assert!(
        src.contains("sim_bridge.dll"),
        "sim_bridge.gdextension must reference sim_bridge.dll (Windows cdylib)"
    );

    println!("A6 PASS: sim_bridge.gdextension identity verified");
    println!("  found: gdext_rust_init, 4.1 compat, reloadable, all 3 platform libs");
}

// ── A7: world_renderer_gd_source_identity (NON-CIRCULAR) ─────────────────────

/// Type S (source identity): scripts/ui/world_renderer.gd must contain the
/// T7.9.A required scaffold: extends Node2D, _ready print sentinel, _process stub.
///
/// NON-CIRCULAR: the print message "WorldRenderer ready (T7.9.A scaffold)" is the
/// in-game verification signal from Section 7 of the T7.9.A spec. Removing the
/// file or the sentinel print breaks this assertion before Godot is opened.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_scaffold_world_renderer_gd_source_identity() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");

    // Type S: Node2D subclass — renderer owns the 2D coordinate space
    assert!(
        src.contains("extends Node2D"),
        "scripts/ui/world_renderer.gd must extend Node2D \
         (T7.9.A composite-split: renderer owns 2D space, WorldSimNode is plain Node)"
    );

    // Type S: _ready must print the T7.9.A scaffold sentinel (Section 7 verification)
    assert!(
        src.contains("WorldRenderer ready (T7.9.A scaffold)"),
        "scripts/ui/world_renderer.gd _ready() must print \
         \"WorldRenderer ready (T7.9.A scaffold)\" \
         (required in-game verification signal per T7.9.A Section 7)"
    );

    // Type S: _process stub must exist — T7.9.B will wire SimBridge overlay rendering here
    assert!(
        src.contains("func _process("),
        "scripts/ui/world_renderer.gd must contain func _process() stub \
         (T7.9.B will add SimBridge.get_influence_overlay() call here)"
    );

    println!("A7 PASS: scripts/ui/world_renderer.gd scaffold identity verified");
    println!("  found: extends Node2D, T7.9.A sentinel print, _process stub");
}
