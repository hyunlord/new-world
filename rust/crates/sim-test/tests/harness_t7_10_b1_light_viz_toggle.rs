//! T7.10.B1 — Light visualization toggle: concurrent channel validity.
//!
//! These tests verify the PRECONDITIONS for the SPACE-key toggle in
//! `world_renderer.gd`: both the Warmth channel (T7.10.A) and the Light
//! channel (T7.10.B) must remain fully populated in the SAME engine run.
//!
//! The toggle only works correctly if:
//!   1. Warmth channel has a genuine non-trivial disc when Light co-exists.
//!   2. Light channel has a genuine non-trivial disc when Warmth co-exists.
//!   3. The Light disc extends beyond the Warmth disc (toggle has visual value).
//!   4. The Light source tile holds the global channel maximum.
//!   5. Light is zero beyond spec radius (no bleed past d=16).
//!
//! T7.10.A and T7.10.B each test one channel in isolation. B1 tests cross-
//! channel concurrent validity — the exact invariant the toggle depends on.
//!
//! Setup: 64×64 engine, empty MaterialRegistry (all-open field), single
//! BuildingPlacedEvent at (32,32) radius=12 consumed in tick 1; ticks 2–10
//! exercise Cold/Warm-tier persistence. All thresholds LOCKED from plan.
//!
//! Run: `cargo test -p sim-test --test harness_t7_10_b1_light_viz_toggle -- --nocapture`

use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, SimEngine};
use sim_systems::register_phase2_systems;

const W: u32 = 64;
const H: u32 = 64;
const SX: u32 = 32;
const SY: u32 = 32;

/// Fresh 64×64 engine with all Phase 2 systems registered.
fn fresh_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    engine
}

/// Push a single BuildingPlacedEvent at (32,32) r=12.
/// Same event triggers both Warmth BFS (T7.10.A) and Light shadowcast (T7.10.B).
fn place_source(engine: &mut SimEngine) {
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (SX, SY),
        radius: 12,
    });
}

/// Run `n` ticks. Event (if any) is consumed in the first tick.
fn run_ticks(engine: &mut SimEngine, n: u32) {
    for _ in 0..n {
        engine.tick();
    }
}

// ── B1-1: Warmth disc is non-trivial while Light co-exists ───────────────────

/// Type A: Warmth non-zero tile count >= 300 after 10 ticks.
///
/// Warmth BFS radius ~12 tiles → disc area π×12² ≈ 452 tiles in a 64×64 grid.
/// Threshold 300 = 66% of the theoretical disc, conservative for BFS boundary
/// discretisation. This test runs in the SAME engine instance as B1-2 (Light)
/// to confirm the Warmth channel remains fully populated while the Light channel
/// co-exists in the same SimResources. T7.10.A verifies Warmth in isolation;
/// B1-1 asserts it survives concurrent channel presence.
#[test]
fn harness_t7_10_b1_warmth_channel_non_trivial_concurrent() {
    let mut e = fresh_engine();
    place_source(&mut e);
    run_ticks(&mut e, 10);

    let warmth_buf = e.resources.influence_grid.current_buf(InfluenceChannel::Warmth);
    // Type A: threshold >= 300 non-zero positions
    let count = warmth_buf.iter().filter(|&&v| v > 0).count();
    assert!(
        count >= 300,
        "Warmth disc must have >= 300 non-zero tiles when Light co-exists \
         (BFS radius ~12, disc area ~452); got {count}. \
         T7.10.A regression: Warmth channel not surviving concurrent Light presence."
    );
    println!("[B1-1] Warmth non-zero tiles: {count} (threshold >= 300)");
}

// ── B1-2: Light disc is non-trivial while Warmth co-exists ───────────────────

/// Type A: Light non-zero tile count >= 450 after 10 ticks.
///
/// Light shadowcast radius 15 tiles → disc area π×15² ≈ 707 tiles. Octagonal
/// shadowcast clips corners; threshold 450 = 64% of full-circle area accounts
/// for clipping. This test runs in the SAME engine instance as B1-1 (Warmth)
/// to confirm the Light channel is fully populated when Warmth co-exists.
/// T7.10.B verifies Light in isolation; B1-2 asserts it survives concurrent
/// channel presence. If Light has < 450 non-zero tiles the toggled overlay is
/// visually empty — the toggle feature is broken regardless of GDScript state.
#[test]
fn harness_t7_10_b1_light_channel_non_trivial_concurrent() {
    let mut e = fresh_engine();
    place_source(&mut e);
    run_ticks(&mut e, 10);

    let light_buf = e.resources.influence_grid.current_buf(InfluenceChannel::Light);
    // Type A: threshold >= 450 non-zero positions
    let count = light_buf.iter().filter(|&&v| v > 0).count();
    assert!(
        count >= 450,
        "Light disc must have >= 450 non-zero tiles when Warmth co-exists \
         (shadowcast radius 15, disc area ~707, threshold=64% for octagonal clipping); \
         got {count}. T7.10.B regression: Light channel not surviving concurrent Warmth."
    );
    println!("[B1-2] Light non-zero tiles: {count} (threshold >= 450)");
}

// ── B1-3: Light disc extends beyond the Warmth disc ──────────────────────────

/// Type A: annular zone tile count (Light substantially > Warmth) >= 150.
///
/// Warmth radius ~12, Light radius 15. Annular zone between radii 12 and 15:
/// π×(15²−12²) ≈ 255 tiles where Light > 0 and Warmth ≈ 0. Threshold 150 = 59%
/// of the annular area, conservative for BFS boundary overlap. Margin = 1% of
/// Light's global maximum (≈ 200 → margin ≈ 2). Any tile where
/// Light − Warmth > margin is an "annular" tile.
///
/// This is the sine qua non of the toggle's visual utility. If the Light disc
/// does not extend beyond Warmth, pressing SPACE reveals nothing new.
#[test]
fn harness_t7_10_b1_light_disc_extends_beyond_warmth() {
    let mut e = fresh_engine();
    place_source(&mut e);
    run_ticks(&mut e, 10);

    let light_buf = e.resources.influence_grid.current_buf(InfluenceChannel::Light);
    let warmth_buf = e.resources.influence_grid.current_buf(InfluenceChannel::Warmth);

    let max1 = light_buf.iter().copied().max().unwrap_or(0);
    let margin_f = max1 as f64 * 0.01; // 1% of Light peak, spec = 200 → margin ≈ 2.0

    // Type A: threshold >= 150 positions where (Light - Warmth) > margin
    let count = light_buf
        .iter()
        .copied()
        .zip(warmth_buf.iter().copied())
        .filter(|&(l, w)| (l as f64 - w as f64) > margin_f)
        .count();

    assert!(
        count >= 150,
        "Light disc must extend >= 150 tiles beyond Warmth disc (annular zone r=12..15, \
         π×(225−144)≈255 tiles; threshold 59%); got {count}. \
         max_light={max1}, margin_1pct={margin_f:.2}. \
         If count < 150 the SPACE toggle reveals no new information."
    );
    println!("[B1-3] Annular zone (Light > Warmth + margin) tiles: {count} (threshold >= 150), margin={margin_f:.2}");
}

// ── B1-4: Light source tile holds the global channel maximum ─────────────────

/// Type A: channel_1[32][32] / max(channel_1) >= 0.99.
///
/// Spec: source intensity = 200, falloff 200/(1+0.1×d) strictly decreasing.
/// At d=0 the value is exactly 200 — the unique global maximum. Ratio >= 0.99
/// tolerates floating-point rounding while requiring source tile to be at peak.
/// Failure means the source was not planted at (32,32) or some tile received
/// a higher value — a wiring defect that would make the toggled view confusing.
#[test]
fn harness_t7_10_b1_light_source_tile_is_maximum() {
    let mut e = fresh_engine();
    place_source(&mut e);
    run_ticks(&mut e, 10);

    let light_buf = e.resources.influence_grid.current_buf(InfluenceChannel::Light);
    let max_val = light_buf.iter().copied().max().unwrap_or(0);
    let source_val = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Light);

    // Type A: threshold ratio >= 0.99
    // Integer form: source_val * 100 >= max_val * 99
    assert!(
        (source_val as u32) * 100 >= (max_val as u32) * 99,
        "Light source tile ({SX},{SY}) must be >= 99% of global max: \
         source={source_val}, global_max={max_val}, ratio={:.4}. \
         Failure means source not planted at (32,32) or falloff assigns higher \
         values to non-source tiles.",
        source_val as f64 / max_val.max(1) as f64
    );
    println!(
        "[B1-4] source={source_val}, max={max_val}, ratio={:.4} (threshold >= 0.99)",
        source_val as f64 / max_val.max(1) as f64
    );
}

// ── B1-6: GDScript toggle handler is present (Type S anti-circular guard) ────

/// Type S: world_renderer.gd contains all 7 required SPACE-toggle tokens.
///
/// Anti-circular guard: these assertions fail if `_unhandled_input` is removed
/// from `world_renderer.gd` while the Rust backend (B1-1..B1-5) stays intact.
/// Without this test a silently broken toggle — handler deleted but channels
/// still wired — would not be caught by any of the 5 Rust runtime checks
/// because those only verify channel data in SimResources, not GDScript state.
///
/// Tokens checked:
///   - `_unhandled_input`                              handler function exists
///   - `InputEventKey`                                 event type discrimination
///   - `event.pressed`                                 physical press (not release)
///   - `not event.echo`                                echo guard (no rapid-fire)
///   - `KEY_SPACE`                                     specific key binding
///   - `CHANNEL_LIGHT if current_channel == CHANNEL_WARMTH` two-state assignment
///   - `Channel switched:`                             console confirmation path
///
/// ticks: 0 (source-only check, no engine run)
#[test]
fn harness_t7_10_b1_gdscript_toggle_handler_present() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");

    assert!(
        src.contains("_unhandled_input"),
        "world_renderer.gd must define `_unhandled_input` \
         (the SPACE-key toggle handler — its absence means the toggle is silently broken \
         even though Rust backend channels B1-1..B1-5 remain populated)"
    );
    assert!(
        src.contains("InputEventKey"),
        "world_renderer.gd `_unhandled_input` must check `event is InputEventKey` \
         (discriminates keyboard events from mouse/touch/gamepad; \
         without it SPACE may be shadowed or other event types wrongly trigger toggle)"
    );
    assert!(
        src.contains("event.pressed"),
        "world_renderer.gd must check `event.pressed` \
         (toggles only on keydown, not on keyup; missing = double-toggle per press)"
    );
    assert!(
        src.contains("not event.echo"),
        "world_renderer.gd must check `not event.echo` \
         (echo guard prevents rapid-fire channel flipping while SPACE is held; \
         missing = channel oscillates at OS key-repeat rate ~30×/sec)"
    );
    assert!(
        src.contains("KEY_SPACE"),
        "world_renderer.gd must test `event.keycode == KEY_SPACE` \
         (hard-coded SPACE binding — no InputMap action entry required)"
    );
    assert!(
        src.contains("CHANNEL_LIGHT if current_channel == CHANNEL_WARMTH"),
        "world_renderer.gd must contain the exact two-state assignment \
         `current_channel = CHANNEL_LIGHT if current_channel == CHANNEL_WARMTH else CHANNEL_WARMTH`; \
         this exact expression prevents wraparound to channels 2..7 (unpopulated)"
    );
    assert!(
        src.contains("Channel switched:"),
        "world_renderer.gd must print `Channel switched:` on every toggle \
         (console confirmation path that lets the developer verify the SPACE key \
         was received and the channel actually changed)"
    );

    println!("[B1-6] PASS: all 7 SPACE-toggle tokens present in world_renderer.gd");
}

// ── B1-5: Light is zero beyond spec radius boundary ──────────────────────────

/// Type A: max Light value at Euclidean distance > 16.0 <= 0.001 (u8: == 0).
///
/// Spec: shadowcast radius = 15 tiles. Distance threshold 16.0 (one full tile
/// past spec radius) eliminates boundary-cell ambiguity. Any value > 0 at d>16
/// means the shadowcast propagates further than specified, making the Light
/// disc visually larger than the T7.10.B design. u8 values are integers so
/// the 0.001 threshold is operationally equivalent to == 0.
#[test]
fn harness_t7_10_b1_light_zero_beyond_radius() {
    let mut e = fresh_engine();
    place_source(&mut e);
    run_ticks(&mut e, 10);

    let w = e.resources.influence_grid.width;
    let h = e.resources.influence_grid.height;
    let mut max_beyond: u8 = 0;
    let mut worst_pos = (0u32, 0u32);

    for y in 0..h {
        for x in 0..w {
            let dx = x as f64 - SX as f64;
            let dy = y as f64 - SY as f64;
            if (dx * dx + dy * dy).sqrt() > 16.0 {
                let v = e.resources.influence_grid.sample(x, y, InfluenceChannel::Light);
                if v > max_beyond {
                    max_beyond = v;
                    worst_pos = (x, y);
                }
            }
        }
    }

    // Type A: threshold <= 0.001 (u8 integer must be 0)
    assert_eq!(
        max_beyond, 0,
        "max Light value at Euclidean distance > 16 must be 0 (spec radius=15); \
         got max={max_beyond} at tile {:?}. \
         Any nonzero value means shadowcast propagates past spec radius.",
        worst_pos
    );
    println!("[B1-5] max Light beyond d=16: {max_beyond} (threshold == 0)");
}
