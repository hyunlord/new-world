//! T7.9.B harness — render mechanism + fixed-tick accumulator (Phase 0 design #9).
//!
//! Builds on T7.9.A's scaffold: process() now drives engine.tick() through a
//! Gaffer 30 TPS accumulator, and WorldRenderer actually uploads the Warmth
//! influence overlay to a Sprite2D texture every frame.
//!
//! Assertion 1: world_node.rs declares `FIXED_DT` = 1.0/30.0 and
//!              `MAX_ITERS_PER_FRAME` = 5 module-level consts.
//! Assertion 2: WorldSimNode struct contains `accumulator: f64` field; init
//!              initialises it to 0.0; process() uses the Gaffer pattern.
//! Assertion 3: world_renderer.gd has the bootstrap call
//!              `on_building_placed(BOOTSTRAP_X, BOOTSTRAP_Y, BOOTSTRAP_RADIUS)`,
//!              creates an Image with FORMAT_L8, instantiates a Sprite2D, and
//!              pulls `get_influence_overlay(CHANNEL_WARMTH)` in _process.
//! Assertion 4: T7.7.B Bridge Identity Contract preserved — 3 `#[func]` methods
//!              still present, `enqueue_building_placed` body still consists of
//!              the bounds-check + queue push (no scope leak).
//!
//! All assertions are Type S (source identity) using include_str! — they fail
//! at compile time if a file is missing and at runtime if required tokens
//! are absent.
//!
//! Run: `cargo test -p sim-test --test harness_t7_9_b_render_mechanism -- --nocapture`

// ── B1: world_node_consts_present ────────────────────────────────────────────

/// Type S: world_node.rs declares the Gaffer accumulator pacing consts.
///
/// `FIXED_DT = 1.0 / 30.0` locks the simulation to 30 TPS per Phase 0 design #9.
/// `MAX_ITERS_PER_FRAME = 5` is the spiral-of-death cap. Changing either value
/// is a deliberate pacing decision that must move with this assertion.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_9_b_world_node_pacing_consts_present() {
    let src = include_str!("../../sim-bridge/src/ffi/world_node.rs");

    assert!(
        src.contains("const FIXED_DT: f64 = 1.0 / 30.0"),
        "world_node.rs must declare `const FIXED_DT: f64 = 1.0 / 30.0` \
         (Phase 0 design #9: 30 TPS fixed simulation step)"
    );
    assert!(
        src.contains("const MAX_ITERS_PER_FRAME: u32 = 5"),
        "world_node.rs must declare `const MAX_ITERS_PER_FRAME: u32 = 5` \
         (spiral-of-death cap on catch-up ticks per Godot frame)"
    );

    println!("B1 PASS: FIXED_DT = 1/30, MAX_ITERS_PER_FRAME = 5");
}

// ── B2: world_node_accumulator_field_and_loop ────────────────────────────────

/// Type S: WorldSimNode owns an `accumulator: f64` field, init zeroes it, and
/// process() advances the simulation through the Gaffer loop.
///
/// The four required tokens together describe the loop's structure without
/// over-fitting to whitespace: field declaration, init initialiser, += delta,
/// and the bounded while condition.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_9_b_world_node_accumulator_loop_present() {
    let src = include_str!("../../sim-bridge/src/ffi/world_node.rs");

    assert!(
        src.contains("accumulator: f64"),
        "WorldSimNode must declare `accumulator: f64` field"
    );
    assert!(
        src.contains("accumulator: 0.0"),
        "INode::init must initialise `accumulator: 0.0`"
    );
    assert!(
        src.contains("self.accumulator += delta"),
        "process() must accumulate `delta` into `self.accumulator`"
    );
    assert!(
        src.contains("self.accumulator >= FIXED_DT")
            && src.contains("iters < MAX_ITERS_PER_FRAME"),
        "process() must run the bounded Gaffer loop \
         `while self.accumulator >= FIXED_DT && iters < MAX_ITERS_PER_FRAME`"
    );

    println!("B2 PASS: accumulator field, init, += delta, bounded while loop");
}

// ── B3: world_renderer_render_path_present ───────────────────────────────────

/// Type S: world_renderer.gd implements the render mechanism — bootstrap
/// building, FORMAT_L8 image, Sprite2D, and per-frame overlay pull.
///
/// Tokens checked:
///   - `CHANNEL_WARMTH := 0`             (channel constant)
///   - `on_building_placed(BOOTSTRAP_X`   (bootstrap call in _ready)
///   - `Image.FORMAT_L8`                  (single-channel 8-bit format)
///   - `Sprite2D.new()`                   (sprite instantiation)
///   - `get_influence_overlay(CHANNEL_WARMTH)` (per-frame data pull)
///   - `texture.update(image)`           (reuse GPU texture, don't realloc)
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_9_b_world_renderer_render_path_present() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");

    assert!(
        src.contains("CHANNEL_WARMTH := 0"),
        "world_renderer.gd must declare `CHANNEL_WARMTH := 0` \
         (Warmth = InfluenceChannel index 0, the first overlay render target)"
    );
    assert!(
        src.contains("on_building_placed(BOOTSTRAP_X"),
        "world_renderer.gd _ready must call \
         `world_sim.on_building_placed(BOOTSTRAP_X, BOOTSTRAP_Y, BOOTSTRAP_RADIUS)` \
         to give BuildingStampSystem something to emit"
    );
    assert!(
        src.contains("Image.FORMAT_L8"),
        "world_renderer.gd must use `Image.FORMAT_L8` (single-channel 8-bit \
         matches InfluenceGrid u8 cell layout)"
    );
    assert!(
        src.contains("Sprite2D.new()"),
        "world_renderer.gd must instantiate `Sprite2D.new()` as the texture host"
    );
    // T7.10.B1: _process now pulls a mutable `current_channel` so SPACE can
    // toggle between Warmth (T7.10.A) and Light (T7.10.B). Initial state
    // `current_channel: int = CHANNEL_WARMTH` preserves T7.9.B/T7.10.A baseline
    // (Warmth disc on first frame), and the CHANNEL_LIGHT constant must exist.
    assert!(
        src.contains("get_influence_overlay(current_channel)"),
        "world_renderer.gd _process must call \
         `world_sim.get_influence_overlay(current_channel)` (T7.10.B1 toggle)"
    );
    assert!(
        src.contains("current_channel: int = CHANNEL_WARMTH"),
        "world_renderer.gd must initialise `current_channel: int = CHANNEL_WARMTH` \
         so the first visible frame matches the T7.10.A Warmth baseline"
    );
    assert!(
        src.contains("CHANNEL_LIGHT := 1"),
        "world_renderer.gd must declare `CHANNEL_LIGHT := 1` for the toggle target \
         (Light = InfluenceChannel index 1)"
    );
    assert!(
        src.contains("texture.update(image)"),
        "world_renderer.gd must call `texture.update(image)` per frame \
         (re-uploads pixels without re-creating the GPU resource)"
    );

    println!("B3 PASS: bootstrap, L8 image, Sprite2D, overlay pull, texture.update");
}

// ── B4: bridge_identity_contract_preserved ───────────────────────────────────

/// Type S: T7.7.B Bridge Identity Contract still holds after T7.9.B edits.
///
/// The 3 `#[func]` methods and the `enqueue_building_placed` `pub fn` must be
/// present and untouched in their public surface. This is the
/// non-regression guard for the 21 FFI assertions landed in T7.7.B.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_9_b_bridge_identity_contract_preserved() {
    let src = include_str!("../../sim-bridge/src/ffi/world_node.rs");

    // Count attribute lines only — doc comments also reference `#[func]` in
    // prose and would inflate a naive `matches("#[func]")` count.
    let func_hits = src
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("#[func]") && !line.contains("//")
        })
        .count();
    assert_eq!(
        func_hits, 3,
        "world_node.rs must expose exactly 3 #[func] attribute lines \
         (get_influence_overlay, get_tile_detail, on_building_placed). \
         Found {func_hits}"
    );

    assert!(
        src.contains("fn get_influence_overlay(&self, channel: i32) -> PackedByteArray"),
        "get_influence_overlay signature must be byte-identical"
    );
    assert!(
        src.contains("fn get_tile_detail(&self, x: i32, y: i32) -> VarDictionary"),
        "get_tile_detail signature must be byte-identical"
    );
    assert!(
        src.contains("fn on_building_placed(&mut self, x: i32, y: i32, radius: i32) -> bool"),
        "on_building_placed signature must be byte-identical"
    );

    // Bridge Identity Contract: on_building_placed body is solely the forwarding call.
    assert!(
        src.contains("enqueue_building_placed(&mut self.engine.resources, x, y, radius)"),
        "on_building_placed body must forward to \
         `enqueue_building_placed(&mut self.engine.resources, x, y, radius)` \
         (Bridge Identity Contract per T7.7.B)"
    );

    println!("B4 PASS: 3 #[func] methods + enqueue_building_placed forwarding intact");
}
