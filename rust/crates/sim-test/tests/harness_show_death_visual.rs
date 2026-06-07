//! V7 viz-D `show-death-visual` — recent-deaths buffer (sim-engine) + FFI
//! collector/to_dict (sim-bridge) + the fading death-marker overlay
//! (`death_viz_renderer.gd`).
//!
//! viz-A made resources visibly deplete; viz-B showed WHO is starving; viz-D
//! closes the chain by showing the death itself — a short-lived,
//! reason-coloured marker at the death tile. Death is an instantaneous event,
//! so the renderer reads a bounded `recent_deaths` buffer pushed by the shared
//! `despawn_agent` helper (both StarvationSystem and CombatSystem route through
//! it) and pruned past `RECENT_DEATH_RETAIN_TICKS`.
//!
//! Assertion map (locked plan, plan_attempt 2):
//!   A1  death pushes an exact-fidelity buffer entry (production path) ........ Type A
//!   A2  FFI collector / split mirror the buffer bit-exactly ................. Type A
//!   A3  buffer pruned past the retain window; fresh entries survive ......... Type A
//!   A4  DeathReason::as_u8 mapping is locked (0/1/2) + row reason_u8 ........ Type A
//!   A5  death_viz_renderer.gd structural wiring (presence/relationship lock)  Type D
//!   A6  main.tscn additive wiring (+1 load_steps, +1 ext_resource) .......... Type D
//!   A7  get_recent_deaths #[func] presence + thin-forwarder shape ........... Type A
//!   A8  regression — death path still chronicles AgentDied (additive) ....... Type D
//!   A9  Combat death end-to-end yields reason_u8 == 2 (shared-path proof) ... Type A
//!   A10 to_dict current_tick scalar equals the engine current tick .......... Type A
//!   A11 mass simultaneous death bound (N == 10 same tick) .................. Type D
//!
//! Run:
//!   cargo test -p sim-test --test harness_show_death_visual -- --nocapture

use std::path::PathBuf;

use hecs::Entity;
use sim_bridge::ffi::{collect_recent_deaths, recent_death_rows_split, RecentDeathRow};
use sim_core::causal::event::{CausalEvent, DeathReason};
use sim_core::components::{
    Agent, AgentId, AgentState, BodyHealth, Hunger, Memory, Thirst, DEFAULT_MAX_HP,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{RecentDeath, RuntimeSystem, SimEngine, RECENT_DEATH_RETAIN_TICKS};
use sim_systems::runtime::combat::CombatSystem;
use sim_systems::runtime::survival::{despawn_agent, StarvationSystem};

const W: u32 = 64;
const H: u32 = 64;

/// Bare engine, NO runtime systems — death is driven directly through the
/// shared `despawn_agent` helper (or `CombatSystem`) so the buffer write is
/// isolated from need-decay / movement / formation interference.
fn iso_engine() -> SimEngine {
    SimEngine::new(W, H, MaterialRegistry::new())
}

/// Spawn an agent and immediately route it through the SHARED production death
/// helper at `(x, y)` with `reason` / `tick`. Returns the agent id. Mutates
/// `recent_deaths` ONLY through `despawn_agent` (never a direct buffer push),
/// closing the circular-`+1` hole flagged in the plan.
fn production_death(e: &mut SimEngine, x: u32, y: u32, reason: DeathReason, tick: u64) -> AgentId {
    let ent = e.spawn_agent(x, y);
    let id = e.world.get::<&Agent>(ent).expect("freshly spawned agent").id;
    despawn_agent(&mut e.world, &mut e.resources, ent, id, (x, y), reason, tick);
    id
}

/// Spawn a controlled agent with a fixed starting hp and zero need-growth so
/// the TEST (not a decay system) owns the need values. Mirrors the
/// `harness_starvation_death.rs` fixture so A1 can drive a death through the
/// production `StarvationSystem` rather than calling `despawn_agent` directly.
fn spawn_controlled(e: &mut SimEngine, x: u32, y: u32, hp: f64) -> (Entity, AgentId) {
    let ent = e.spawn_agent(x, y);
    let id = e.world.get::<&Agent>(ent).expect("agent").id;
    e.world
        .insert(
            ent,
            (
                AgentState::Idle,
                BodyHealth { hp, max_hp: DEFAULT_MAX_HP },
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Memory::new(),
            ),
        )
        .expect("seed controlled agent");
    (ent, id)
}

fn alive(e: &SimEngine, ent: Entity) -> bool {
    e.world.get::<&Agent>(ent).is_ok()
}

/// Drive the production `StarvationSystem` directly, re-pinning Hunger to
/// `SATURATION` every tick (zero need-growth means the test owns the value).
/// Returns `Some(tick)` — the `resources.current_tick` value at which the agent
/// died (the SAME clock `despawn_agent` stamps into the buffer entry) — or
/// `None` if it survived `max` ticks. Death is reached ONLY through the real
/// system funnel, never a direct push.
fn drive_starvation_to_death(e: &mut SimEngine, ent: Entity, max: u64) -> Option<u64> {
    let mut sys = StarvationSystem::new();
    for t in 0..max {
        e.resources.current_tick = t;
        if let Ok(mut h) = e.world.get::<&mut Hunger>(ent) {
            h.value = Hunger::SATURATION;
        }
        sys.tick(&mut e.world, &mut e.resources);
        if !alive(e, ent) {
            return Some(t);
        }
    }
    None
}

fn project_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .expect("project root = sim-test/../../..")
}

fn read_file(rel: &[&str]) -> String {
    let mut path = project_root();
    for seg in rel {
        path.push(seg);
    }
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// Parse `const NAME := <float>` or `const NAME: T = <float>` from GDScript,
/// returning the literal as f64. Strips a trailing comment.
fn parse_gd_const_f64(src: &str, name: &str) -> Option<f64> {
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("const ") || !t.contains(name) {
            continue;
        }
        let rhs = t.rsplit_once('=').map(|(_, r)| r.trim())?;
        let rhs = rhs.split('#').next().unwrap_or(rhs).trim();
        if let Ok(v) = rhs.parse::<f64>() {
            return Some(v);
        }
    }
    None
}

/// Extract `load_steps=N` from a `[gd_scene ... load_steps=N ...]` header.
fn tscn_load_steps(src: &str) -> Option<i64> {
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("[gd_scene") {
            continue;
        }
        let idx = t.find("load_steps")? + "load_steps".len();
        let rest = t[idx..].trim_start().strip_prefix('=')?.trim_start();
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        return digits.parse::<i64>().ok();
    }
    None
}

// ════════════════════════════════════════════════════════════════════════════
// A1: death pushes an exact-fidelity buffer entry (production path only).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a1_production_death_pushes_exact_entry() {
    // Type A — drive the death through the PRODUCTION `StarvationSystem.tick()`
    // (pin Hunger at SATURATION + low BodyHealth, run until it kills the agent),
    // NOT a direct `despawn_agent` / buffer push. This proves the real system
    // path reaches the shared death funnel and writes an exact-fidelity entry.
    // We measure the +1 growth AND the exact field equality (x/y/reason/tick).
    let mut e = iso_engine();
    assert!(e.resources.recent_deaths.is_empty(), "A1: buffer must start empty");

    // (edge case) Death on tile (0,0) — x/y of 0 must be a genuine entry, not a
    // "no data" sentinel. Low hp + saturated Hunger → StarvationSystem kills it.
    let (ent0, _id0) = spawn_controlled(&mut e, 0, 0, 0.1);
    let len_before_1 = e.resources.recent_deaths.len();
    let death_tick_0 = drive_starvation_to_death(&mut e, ent0, 50)
        .expect("A1: StarvationSystem must kill the (0,0) agent within 50 ticks");
    assert_eq!(
        e.resources.recent_deaths.len(),
        len_before_1 + 1,
        "A1: buffer length must increase by exactly 1 per production death"
    );
    let d0 = e.resources.recent_deaths.last().expect("A1: first entry present");
    assert_eq!(d0.x, 0, "A1: x == position.0 (0,0 death is genuine)");
    assert_eq!(d0.y, 0, "A1: y == position.1 (0,0 death is genuine)");
    assert_eq!(
        d0.reason,
        DeathReason::Starvation,
        "A1: reason == Starvation (Hunger saturated, Thirst zero)"
    );
    assert_eq!(u64::from(d0.tick), death_tick_0, "A1: tick == the production death tick (same clock)");

    // A second starvation death at a nonzero tile — +1 again, all fields exact.
    let (ent1, _id1) = spawn_controlled(&mut e, 12, 34, 0.1);
    let len_before_2 = e.resources.recent_deaths.len();
    let death_tick_1 = drive_starvation_to_death(&mut e, ent1, 50)
        .expect("A1: StarvationSystem must kill the (12,34) agent within 50 ticks");
    assert_eq!(
        e.resources.recent_deaths.len(),
        len_before_2 + 1,
        "A1: second production death also grows the buffer by exactly 1"
    );
    let d1 = e.resources.recent_deaths.last().expect("A1: second entry present");
    assert_eq!((d1.x, d1.y), (12, 34), "A1: x/y exactly equal the death position");
    assert_eq!(d1.reason, DeathReason::Starvation, "A1: reason exactly equal (Starvation)");
    assert_eq!(u64::from(d1.tick), death_tick_1, "A1: tick exactly equal the production death tick");
    println!("[viz-D A1] StarvationSystem.tick() deaths → +1 each; x/y/reason/tick all exact ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// A2: FFI collector / split mirror the buffer bit-exactly.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a2_collector_split_mirror_buffer() {
    // Type A — collector/split are pure reads with documented casts
    // (reason→u8→i32, u32→i32, u64→i64) preserving buffer order. Exact mirror.
    let mut e = iso_engine();
    production_death(&mut e, 0, 0, DeathReason::Starvation, 1); // (0,0) edge in the array
    production_death(&mut e, 5, 9, DeathReason::Dehydration, 2);
    production_death(&mut e, 63, 63, DeathReason::Combat, 3);

    let rows: Vec<RecentDeathRow> = collect_recent_deaths(&e.resources);
    // rows.len() == buffer len.
    assert_eq!(
        rows.len(),
        e.resources.recent_deaths.len(),
        "A2: collected rows.len() must equal recent_deaths.len()"
    );
    // Every row mirrors the matching buffer entry (order preserved).
    let mut mismatches = 0usize;
    for (i, r) in rows.iter().enumerate() {
        let d = &e.resources.recent_deaths[i];
        if r.x != d.x as i32 || r.y != d.y as i32 {
            mismatches += 1;
        }
        if r.reason_u8 != d.reason.as_u8() as i32 {
            mismatches += 1;
        }
        if r.tick != d.tick as i64 {
            mismatches += 1;
        }
    }
    assert_eq!(mismatches, 0, "A2: row/buffer field mismatches must be 0 (order preserved)");

    // (0,0) edge — the first row's xs/ys must be a genuine 0, not a sentinel.
    let (xs, ys, reasons, ticks) = recent_death_rows_split(&rows);
    assert_eq!(xs.len(), rows.len(), "A2: xs length == rows.len()");
    assert_eq!(ys.len(), rows.len(), "A2: ys length == rows.len()");
    assert_eq!(reasons.len(), rows.len(), "A2: reasons length == rows.len()");
    assert_eq!(ticks.len(), rows.len(), "A2: ticks length == rows.len()");
    assert_eq!(xs[0], 0, "A2: (0,0) death → xs[0] == 0 (genuine, not sentinel)");
    assert_eq!(ys[0], 0, "A2: (0,0) death → ys[0] == 0 (genuine, not sentinel)");
    for (i, r) in rows.iter().enumerate() {
        assert_eq!(xs[i], r.x, "A2: xs[i] == row.x");
        assert_eq!(ys[i], r.y, "A2: ys[i] == row.y");
        assert_eq!(reasons[i], r.reason_u8, "A2: reasons[i] == row.reason_u8");
        assert_eq!(ticks[i], r.tick, "A2: ticks[i] == row.tick");
    }
    println!("[viz-D A2] collector + split mirror the buffer bit-exactly (order preserved) ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// A3: buffer is pruned past the retain window; fresh entries survive.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a3_prune_past_retain_window() {
    // Type A — the prune predicate `tick.saturating_sub(d.tick) <
    // RECENT_DEATH_RETAIN_TICKS` is a deterministic, order-independent boundary
    // defined by a named const. The boundary is expressed using the SYMBOL,
    // never the literal 120 — a retune of the const must follow automatically.
    const T_OLD: u64 = 0;
    const T_FRESH: u64 = 50;
    let mut e = iso_engine();
    let _old = production_death(&mut e, 3, 4, DeathReason::Starvation, T_OLD);
    let _fresh = production_death(&mut e, 7, 8, DeathReason::Combat, T_FRESH);
    assert_eq!(e.resources.recent_deaths.len(), 2, "A3 setup: two seeded deaths");

    // Advance to current_tick == RETAIN. The last tick() pruned at RETAIN-1,
    // whose old-entry age is RETAIN-1 < RETAIN → still present. Bare engine: the
    // only per-tick effect is the prune itself.
    while e.current_tick() < RECENT_DEATH_RETAIN_TICKS {
        e.tick();
    }
    let old_present_before = e
        .resources
        .recent_deaths
        .iter()
        .any(|d| d.x == 3 && d.y == 4 && u64::from(d.tick) == T_OLD);
    let fresh_present_before = e
        .resources
        .recent_deaths
        .iter()
        .any(|d| d.x == 7 && d.y == 8 && u64::from(d.tick) == T_FRESH);
    assert!(
        old_present_before,
        "A3: aged entry must remain while current_tick - T_old < RECENT_DEATH_RETAIN_TICKS"
    );
    assert!(fresh_present_before, "A3: fresh entry present before the boundary");

    // One more tick → prune runs at current_tick == RETAIN: age(old) ==
    // RETAIN >= RETAIN → pruned; age(fresh) == RETAIN - T_FRESH < RETAIN → kept.
    e.tick();
    let aged = e
        .resources
        .recent_deaths
        .iter()
        .filter(|d| u64::from(d.tick) == T_OLD && d.x == 3 && d.y == 4)
        .count();
    let fresh = e
        .resources
        .recent_deaths
        .iter()
        .filter(|d| u64::from(d.tick) == T_FRESH && d.x == 7 && d.y == 8)
        .count();
    assert_eq!(
        aged, 0,
        "A3: aged entry removed once current_tick - T_old >= RECENT_DEATH_RETAIN_TICKS"
    );
    assert_eq!(fresh, 1, "A3: fresh entry survives (current_tick - T_fresh < RETAIN)");
    println!(
        "[viz-D A3] prune at RECENT_DEATH_RETAIN_TICKS={RECENT_DEATH_RETAIN_TICKS}: old gone, fresh kept ✓"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// A4: DeathReason::as_u8 mapping is locked; rows carry the matching reason_u8.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a4_reason_u8_mapping_locked() {
    // Type A — discriminant mapping is a hard contract the GDScript colour
    // switch depends on (0=brown, 1=blue, 2=red). Any other value mis-colours.
    assert_eq!(DeathReason::Starvation.as_u8(), 0, "A4: Starvation == 0");
    assert_eq!(DeathReason::Dehydration.as_u8(), 1, "A4: Dehydration == 1");
    assert_eq!(DeathReason::Combat.as_u8(), 2, "A4: Combat == 2");

    // A collector row built from each reason as the source carries the matching
    // reason_u8 (end-to-end at the row layer; combat end-to-end is A9).
    for (reason, expect) in [
        (DeathReason::Starvation, 0i32),
        (DeathReason::Dehydration, 1),
        (DeathReason::Combat, 2),
    ] {
        let mut e = iso_engine();
        production_death(&mut e, 1, 1, reason, 0);
        let rows = collect_recent_deaths(&e.resources);
        assert_eq!(rows.len(), 1, "A4: one death → one row");
        assert_eq!(rows[0].reason_u8, expect, "A4: row reason_u8 matches {reason:?}");
    }
    println!("[viz-D A4] as_u8 mapping locked (0/1/2) + row reason_u8 mirrors reason ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// A5: death_viz_renderer.gd structural wiring (presence / relationship lock).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a5_renderer_structural_wiring() {
    // Type D — structural integration regression guard across the renderer
    // surface. This is a PRESENCE / relationship lock, NOT a colour-correctness
    // proof (colour correctness is windowed-Godot only — see NOT in Scope).
    let r = read_file(&["scripts", "ui", "death_viz_renderer.gd"]);
    assert!(r.contains("extends Node2D"), "A5: renderer extends Node2D");
    assert!(r.contains("get_recent_deaths"), "A5: renderer calls the FFI getter get_recent_deaths");
    assert!(r.contains("FADE_TICKS"), "A5: renderer declares FADE_TICKS");
    assert!(r.contains("Z_DEATH"), "A5: renderer declares Z_DEATH");
    assert!(r.contains("func _draw"), "A5: marker drawn in _draw");
    assert!(r.contains("queue_redraw"), "A5: renderer requests a per-frame redraw");
    // Fade ratio: FADE_TICKS must appear in a division/ratio with an `age`
    // term — tolerant of algebraically-identical variants (`1.0 - age/FADE_TICKS`,
    // `(FADE_TICKS - age)/FADE_TICKS`, `1.0 - float(age)/float(FADE_TICKS)`), so a
    // single literal cannot false-negative a correct rearrangement.
    let fade_ratio_ok = r.lines().any(|l| {
        let has_div = l.contains("/ FADE_TICKS") || l.contains("/FADE_TICKS");
        let has_age = l.contains("age");
        has_div && has_age
    });
    assert!(
        fade_ratio_ok,
        "A5: a fade ratio dividing by FADE_TICKS with an `age` term must be present"
    );
    // Type-guards (mirrors need_bar): defensive FFI reads.
    assert!(r.contains("is Dictionary"), "A5: type-guards `is Dictionary`");
    assert!(r.contains("is PackedInt32Array"), "A5: type-guards `is PackedInt32Array`");
    assert!(r.contains("is PackedInt64Array"), "A5: type-guards `is PackedInt64Array`");

    // The three reason→colour literals are asserted PRESENT (wiring lock); this
    // does NOT prove they are the correct hues.
    for col in [
        "Color(0.45, 0.30, 0.15)", // Starvation brown
        "Color(0.20, 0.45, 0.95)", // Dehydration blue
        "Color(0.95, 0.15, 0.10)", // Combat red
    ] {
        assert!(r.contains(col), "A5: reason colour literal `{col}` must be present");
    }

    // Numeric relationships — BOTH sides read from source (not hardcoded), so the
    // layering/visibility contract survives a future const change on either side.
    let fade = parse_gd_const_f64(&r, "FADE_TICKS").expect("A5: FADE_TICKS must be numeric");
    let z_death = parse_gd_const_f64(&r, "Z_DEATH").expect("A5: Z_DEATH must be numeric");
    let renderer_retain =
        parse_gd_const_f64(&r, "RETAIN_TICKS").expect("A5: renderer RETAIN_TICKS must be numeric");
    let need_bar = read_file(&["scripts", "ui", "need_bar_renderer.gd"]);
    let z_need_bar = parse_gd_const_f64(&need_bar, "Z_NEED_BAR")
        .expect("A5: NeedBarRenderer Z_NEED_BAR must be read from its source");
    assert!(
        fade < renderer_retain,
        "A5: FADE_TICKS ({fade}) must be < renderer RETAIN_TICKS ({renderer_retain}) \
         so the marker fades before its data leaves the buffer window"
    );
    assert!(
        z_death > z_need_bar,
        "A5: Z_DEATH ({z_death}) must be > NeedBarRenderer Z_NEED_BAR ({z_need_bar}) \
         (read from source) so death markers are not occluded by need bars"
    );
    println!(
        "[viz-D A5] renderer wiring + FADE<RETAIN(read) + Z_DEATH>Z_NEED_BAR(read) + colours ✓"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// A6: main.tscn additive wiring (+1 load_steps, +1 ext_resource).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a6_scene_wiring_additive() {
    // Type D additive-preservation regression. Baseline MEASURED from the
    // current tree at implementation time (viz-C shipped between viz-B and viz-D
    // without changing the count): load_steps = 12, ext_resource count = 12.
    // viz-D adds exactly one ext_resource (death_viz_renderer.gd) → +1 on both.
    // We assert the +1 DELTA (offset-independent), never an absolute identity.
    const LOAD_STEPS_BEFORE: i64 = 12;
    const EXT_COUNT_BEFORE: usize = 12;

    let scene = read_file(&["scenes", "main.tscn"]);
    let load_steps_after = tscn_load_steps(&scene).expect("A6: main.tscn must declare load_steps=N");
    let ext_count_after = scene
        .lines()
        .filter(|l| l.trim_start().starts_with("[ext_resource type="))
        .count();

    assert_eq!(
        load_steps_after,
        LOAD_STEPS_BEFORE + 1,
        "A6: load_steps must be baseline+1 ({}); got {load_steps_after}",
        LOAD_STEPS_BEFORE + 1
    );
    assert_eq!(
        ext_count_after,
        EXT_COUNT_BEFORE + 1,
        "A6: ext_resource count must be baseline+1 ({}); got {ext_count_after}",
        EXT_COUNT_BEFORE + 1
    );
    // Exact ext_resource PATH string (not a bare filename substring) — a typo'd
    // or loose path on a present node must FAIL. Capture the ext_resource id so we
    // can confirm the DeathVizRenderer node actually binds THIS script.
    let ext_line = scene
        .lines()
        .find(|l| {
            let t = l.trim_start();
            t.starts_with("[ext_resource type=")
                && t.contains("path=\"res://scripts/ui/death_viz_renderer.gd\"")
        })
        .expect("A6: an ext_resource with exact path res://scripts/ui/death_viz_renderer.gd must exist");
    let ext_id = ext_line
        .split("id=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .expect("A6: death_viz ext_resource must declare an id");

    // The DeathVizRenderer node must exist AND reference exactly that ext_resource.
    let mut in_death_node = false;
    let mut node_binds_script = false;
    for l in scene.lines() {
        let t = l.trim();
        if t.starts_with("[node ") {
            in_death_node = t.contains("name=\"DeathVizRenderer\"") && t.contains("type=\"Node2D\"");
        } else if in_death_node
            && t.starts_with("script =")
            && t.contains(&format!("ExtResource(\"{ext_id}\")"))
        {
            node_binds_script = true;
        }
    }
    assert!(
        node_binds_script,
        "A6: DeathVizRenderer Node2D must exist AND bind the death_viz_renderer.gd ext_resource \
         (id={ext_id}) — a present node with a wrong/missing script path fails"
    );

    // Every previously-registered renderer/overlay preserved (removal == 0).
    for token in [
        "name=\"WorldRenderer\"",
        "name=\"AgentRenderer\"",
        "name=\"SeekVizRenderer\"",
        "name=\"NeedBarRenderer\"",
        "need_bar_renderer.gd",
        "seek_viz_renderer.gd",
    ] {
        assert!(scene.contains(token), "A6: pre-existing wiring `{token}` must be preserved");
    }
    println!(
        "[viz-D A6] load_steps {LOAD_STEPS_BEFORE}→{load_steps_after}, ext {EXT_COUNT_BEFORE}→{ext_count_after}; \
         DeathVizRenderer wired, prior nodes preserved ✓"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// A7: get_recent_deaths #[func] presence + thin-forwarder shape.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a7_ffi_func_presence_and_forwarding() {
    // Type A — PRESENCE + forwarding-shape + key-emission check (no negative
    // "nothing else computes" claim; a substring scan cannot prove a negative).
    let b = read_file(&["rust", "crates", "sim-bridge", "src", "ffi", "world_node.rs"]);
    assert!(
        b.contains("fn get_recent_deaths"),
        "A7: `#[func] fn get_recent_deaths` must exist"
    );
    assert!(
        b.contains("collect_recent_deaths"),
        "A7: body must reference the collector `collect_recent_deaths`"
    );
    assert!(
        b.contains("recent_death_rows_to_dict"),
        "A7: body must reference the to_dict helper `recent_death_rows_to_dict`"
    );
    assert!(
        b.contains("current_tick"),
        "A7: body must pass `current_tick` through to the to_dict producer"
    );
    // to_dict emits all five keys.
    for key in ["xs", "ys", "reasons", "ticks", "current_tick"] {
        assert!(
            b.contains(&format!("dict.set(\"{key}\"")),
            "A7: to_dict must emit the key \"{key}\""
        );
    }
    println!("[viz-D A7] get_recent_deaths #[func] forwards to collector + to_dict, 5 keys emitted ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// A8: regression — death path still chronicles AgentDied (additive, not replaced).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a8_agent_died_chronicle_additive() {
    // Type D — regression guard for the locked scope note "keep the existing
    // causal_log AgentDied push untouched". The buffer is ADDITIVE: BOTH the
    // causal-log entry AND the recent_deaths entry must exist after a death.
    let mut e = iso_engine();
    let (px, py) = (21u32, 13u32);
    let id = production_death(&mut e, px, py, DeathReason::Starvation, 42);

    // Chronicle path intact — AgentDied at the death tile (query immediately:
    // the per-tile causal log is an 8-slot ring).
    let width = e.resources.tile_grid.width;
    let tile_idx = py * width + px;
    let log = e.resources.causal_log.get(tile_idx).expect("A8: tile must have a causal ring");
    let chronicled = log.as_slice().iter().any(|ev| {
        matches!(
            ev,
            CausalEvent::AgentDied { agent, position, reason, tick, .. }
            if *agent == id
                && *position == (px, py)
                && *reason == DeathReason::Starvation
                && *tick == 42
        )
    });
    assert!(chronicled, "A8: AgentDied must still be chronicled at the death tile (path intact)");

    // Buffer path also populated (additive).
    let buffered = e
        .resources
        .recent_deaths
        .iter()
        .any(|d| d.x == px as i32 && d.y == py as i32 && d.reason == DeathReason::Starvation && d.tick == 42);
    assert!(buffered, "A8: recent_deaths must ALSO hold the death (additive, not a replacement)");
    println!("[viz-D A8] AgentDied chronicle + recent_deaths buffer both present (additive) ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// A9: Combat death end-to-end yields reason_u8 == 2 (shared-path placement proof).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a9_combat_death_reason_u8_two() {
    // Type A — closes the strongest gaming vector: every other death-path
    // assertion drives starvation/dehydration. Driving a death through the
    // PRODUCTION CombatSystem proves the buffer push lives in the SHARED
    // `despawn_agent` (not the starvation branch only).
    let mut e = iso_engine();
    let attacker = e.spawn_agent(5, 5);
    let defender = e.spawn_agent(5, 5);
    let attacker_id = e.world.get::<&Agent>(attacker).unwrap().id;
    let defender_id = e.world.get::<&Agent>(defender).unwrap().id;
    e.world
        .insert(attacker, (AgentState::Idle, BodyHealth::new(), Memory::new()))
        .unwrap();
    e.world
        .insert(
            defender,
            (AgentState::Idle, BodyHealth { hp: 5.0, max_hp: DEFAULT_MAX_HP }, Memory::new()),
        )
        .unwrap();

    // ENGINEERED single-resolution single-death determinism:
    //   (a) lethal-in-one — defender hp 5.0 ≪ attacker's per-resolution damage;
    //   (b) zero evasion — `CombatSystem` has NO dodge/evasion/RNG roll (pure
    //       deterministic damage), so seed cannot produce a miss;
    //   (c) attacker survives — attacker at full BodyHealth so any retaliation in
    //       the same window cannot kill it → EXACTLY one death (+1, not +2).
    let len_before = e.resources.recent_deaths.len();
    e.resources.combat_pairs.insert((attacker_id, defender_id));
    e.resources.current_tick = 1;
    let mut cs = CombatSystem::new();
    cs.tick(&mut e.world, &mut e.resources);

    // Precondition — exactly the defender died; the attacker survives (single victim).
    assert!(
        e.world.get::<&Agent>(defender).is_err(),
        "A9: scenario must produce a combat death (defender must despawn)"
    );
    assert!(
        e.world.get::<&Agent>(attacker).is_ok(),
        "A9: attacker must SURVIVE (single-victim setup → exactly one death, +1 not +2)"
    );
    assert_eq!(
        e.resources.recent_deaths.len(),
        len_before + 1,
        "A9: a Combat death adds exactly 1 buffer entry (shared despawn_agent path)"
    );
    let entry = e
        .resources
        .recent_deaths
        .last()
        .expect("A9: the new entry must be present");
    assert_eq!(entry.reason, DeathReason::Combat, "A9: buffer entry reason == Combat");
    assert_eq!((entry.x, entry.y), (5, 5), "A9: entry x/y == the victim's death tile");
    assert_eq!(entry.tick, 1, "A9: entry tick == the combat death tick");

    let rows = collect_recent_deaths(&e.resources);
    let combat_row = rows
        .iter()
        .find(|r| r.reason_u8 == 2)
        .expect("A9: a collected row with reason_u8 == 2 must exist");
    assert_eq!(combat_row.reason_u8, 2, "A9: Combat row → reason_u8 == 2 (red marker)");
    println!(
        "[viz-D A9] CombatSystem death → +1 (attacker survives), Combat, x/y/tick exact, reason_u8==2 ✓"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// A10: to_dict current_tick scalar equals the engine current tick.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a10_current_tick_scalar_is_live() {
    // Type A — the renderer computes `age = current_tick - entry.tick` for the
    // fade. The #[func] emits `self.engine.resources.current_tick` (NOT the
    // `engine.current_tick()` completed-count: deaths are stamped from
    // `resources.current_tick` too, so the FFI scalar and the death tick share
    // the SAME clock — using the completed-count would skew every age by 1).
    // The VarDictionary value itself needs the Godot runtime (windowed), so the
    // cargo-accessible surface is: (a) the live source value the #[func] reads
    // is the advanced, nonzero tick N (distinguishable from a stale 0), and
    // (b) the #[func] forwards that exact field into the to_dict producer.
    //
    // `tick()` sets `resources.current_tick = self.current_tick` then increments
    // `self.current_tick`, so after K calls `resources.current_tick == K - 1`.
    const TICK_CALLS: u64 = 38;
    let mut e = iso_engine();
    for _ in 0..TICK_CALLS {
        e.tick();
    }
    let n = e.resources.current_tick; // the exact value the #[func] emits
    assert_eq!(
        n,
        TICK_CALLS - 1,
        "A10(a): `resources.current_tick` (what the #[func] reads) must track the live clock"
    );
    assert!(n > 0, "A10(a): the scalar must be nonzero (a stale-zero would freeze every marker)");

    // (b) Forwarding shape — the #[func] passes the engine's current_tick into
    // the to_dict producer (guards against a stale/zero scalar).
    let b = read_file(&["rust", "crates", "sim-bridge", "src", "ffi", "world_node.rs"]);
    assert!(
        b.contains("self.engine.resources.current_tick"),
        "A10(b): get_recent_deaths must forward `self.engine.resources.current_tick`"
    );
    println!("[viz-D A10] current_tick source == N and is forwarded into to_dict ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// A11: mass simultaneous death bound (N == 10 same tick).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a11_mass_same_tick_deaths() {
    // Type D — A3 prunes by age but NOTHING bounds buffer length WITHIN the
    // retain window. The prompt specifies NO cap, so we assert length == N and
    // FLAG that the buffer is unbounded within the retain window (a famine/raid
    // mass-death could push dozens of entries the renderer iterates each frame).
    const N: u32 = 10;
    const SAME_TICK: u64 = 5;
    let mut e = iso_engine();
    let mut expected: Vec<(u32, u32)> = Vec::new();
    for i in 0..N {
        let x = i; // distinct tiles 0..N (includes the (0,0) tile at i==0)
        let y = 20;
        production_death(&mut e, x, y, DeathReason::Combat, SAME_TICK);
        expected.push((x, y));
    }

    assert_eq!(
        e.resources.recent_deaths.len() as u32,
        N,
        "A11: N same-tick deaths → buffer length == N (no cap specified in the prompt)"
    );
    let rows = collect_recent_deaths(&e.resources);
    assert_eq!(rows.len() as u32, N, "A11: rows.len() == N");
    // Order preserved — buffer is a Vec, push order == collector order.
    for (i, (ex, ey)) in expected.iter().enumerate() {
        assert_eq!(rows[i].x, *ex as i32, "A11: row[{i}].x preserves push order");
        assert_eq!(rows[i].y, *ey as i32, "A11: row[{i}].y preserves push order");
    }
    eprintln!(
        "[viz-D A11] FINDING: recent_deaths is UNBOUNDED within the {RECENT_DEATH_RETAIN_TICKS}-tick \
         retain window (no cap in prompt). {N} same-tick deaths all retained; a mass-death tick \
         (famine/raid) could push dozens the renderer iterates every frame."
    );
    println!("[viz-D A11] {N} same-tick deaths all present, order preserved (unbounded — flagged) ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// A12: empty-buffer collector + single genuine (0,0,Starvation,0) fixture.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a12_empty_then_single_fixture_collector() {
    // Type A — the collector/split must NOT fabricate or drop rows. (a) On a
    // fresh engine with zero deaths, every output is empty. (b) A single
    // injected RecentDeath whose every field is 0 (the worst "looks-like-no-data"
    // case) must surface as EXACTLY one genuine row — proving 0 is a real value,
    // not a dropped/sentinel entry.
    let mut e = iso_engine();

    // (a) empty buffer → empty collector + all four split arrays length 0.
    assert!(e.resources.recent_deaths.is_empty(), "A12: fresh buffer must be empty");
    let empty_rows = collect_recent_deaths(&e.resources);
    assert_eq!(empty_rows.len(), 0, "A12(a): zero deaths → collector returns 0 rows");
    let (xs0, ys0, reasons0, ticks0) = recent_death_rows_split(&empty_rows);
    assert_eq!(xs0.len(), 0, "A12(a): empty xs");
    assert_eq!(ys0.len(), 0, "A12(a): empty ys");
    assert_eq!(reasons0.len(), 0, "A12(a): empty reasons");
    assert_eq!(ticks0.len(), 0, "A12(a): empty ticks");

    // (b) inject ONE all-zero entry (x=0, y=0, Starvation→0, tick=0) directly
    // into the otherwise-empty buffer. This isolates the collector's read path
    // (A1 already proves the production WRITE path).
    e.resources
        .recent_deaths
        .push(RecentDeath { x: 0, y: 0, reason: DeathReason::Starvation, tick: 0 });
    let rows = collect_recent_deaths(&e.resources);
    assert_eq!(rows.len(), 1, "A12(b): single fixture → exactly 1 row (not dropped)");
    let (xs, ys, reasons, ticks) = recent_death_rows_split(&rows);
    assert_eq!(xs.len(), 1, "A12(b): xs length 1");
    assert_eq!((xs[0], ys[0]), (0, 0), "A12(b): (0,0) is a GENUINE row, not a sentinel");
    assert_eq!(reasons[0], 0, "A12(b): Starvation → reason_u8 == 0 (genuine zero)");
    assert_eq!(ticks[0], 0, "A12(b): tick 0 → ticks[0] == 0 (genuine, not dropped)");
    // The dict's `current_tick` SCALAR presence is a Godot-VarDictionary surface
    // (needs the engine runtime); its source emission is owned by A7 (to_dict
    // emits the "current_tick" key) and its live value by A10.

    // (c) future-tick entry (tick > current_tick): the renderer computes
    // `age = current_tick - entry.tick` and MUST clamp via saturating semantics —
    // a future-tick entry yields age 0 (non-negative), never an underflow panic.
    // The collector must still surface the entry (not drop it). We exercise the
    // saturating_sub here (the value the renderer mirrors) and confirm the
    // renderer source carries the `age < 0` draw-skip guard.
    let mut e2 = iso_engine();
    let known_now: u64 = 5;
    e2.resources.current_tick = known_now;
    let future_tick = known_now + 50;
    e2.resources
        .recent_deaths
        .push(RecentDeath { x: 3, y: 4, reason: DeathReason::Combat, tick: future_tick as u32 });
    let frows = collect_recent_deaths(&e2.resources);
    assert_eq!(frows.len(), 1, "A12(c): future-tick entry must still be collected (not dropped)");
    assert_eq!(frows[0].tick, future_tick as i64, "A12(c): the future tick is preserved verbatim");
    let age = known_now.saturating_sub(future_tick);
    assert_eq!(age, 0, "A12(c): saturating_sub(current,future) == 0 (no underflow, clamped)");
    let r = read_file(&["scripts", "ui", "death_viz_renderer.gd"]);
    assert!(
        r.contains("age < 0"),
        "A12(c): renderer must guard `age < 0` so a future-tick marker is skipped, never drawn negative"
    );
    println!(
        "[viz-D A12] empty→0; all-zero fixture→1 genuine row; future-tick age saturates to 0 (no panic) ✓"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// A13: a real DEHYDRATION death pushes a reason_u8==1 entry (blue path, end-to-end).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a13_dehydration_death_reason_u8_one() {
    // Type A — closes the Challenger's biggest gaming vector: NO other production
    // death exercises Dehydration (A1=Starvation, A9=Combat). Drive a real death
    // with BOTH Hunger AND Thirst saturated; StarvationSystem's precedence
    // (`thirst_saturated → Dehydration`) MUST surface reason_u8 == 1 (blue). The
    // expected reason is hard-coded as Dehydration BECAUSE the test saturated
    // thirst — it does NOT read the reason from the path it verifies (anti-circular).
    let mut e = iso_engine();
    let (ent, _id) = spawn_controlled(&mut e, 7, 19, 0.1);
    let len_before = e.resources.recent_deaths.len();

    let mut sys = StarvationSystem::new();
    let mut death_tick: Option<u64> = None;
    for t in 0..50u64 {
        e.resources.current_tick = t;
        if let Ok(mut h) = e.world.get::<&mut Hunger>(ent) {
            h.value = Hunger::SATURATION;
        }
        if let Ok(mut th) = e.world.get::<&mut Thirst>(ent) {
            th.value = Thirst::SATURATION;
        }
        sys.tick(&mut e.world, &mut e.resources);
        if !alive(&e, ent) {
            death_tick = Some(t);
            break;
        }
    }
    let death_tick = death_tick.expect("A13: both-saturated agent must die within 50 ticks");

    assert_eq!(
        e.resources.recent_deaths.len(),
        len_before + 1,
        "A13: a Dehydration death adds exactly 1 buffer entry (shared funnel)"
    );
    let d = e.resources.recent_deaths.last().expect("A13: entry present");
    assert_eq!(
        d.reason,
        DeathReason::Dehydration,
        "A13: both-saturated death → reason == Dehydration (thirst precedence)"
    );
    assert_eq!((d.x, d.y), (7, 19), "A13: x/y == the victim's death tile");
    assert_eq!(u64::from(d.tick), death_tick, "A13: tick == captured death tick");
    let rows = collect_recent_deaths(&e.resources);
    let row = rows.last().expect("A13: collected row present");
    assert_eq!(row.reason_u8, 1, "A13: Dehydration → reason_u8 == 1 (blue marker, end-to-end)");
    println!("[viz-D A13] real Dehydration death → +1, reason_u8 == 1 (blue) end-to-end ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// A14: same-tick multiple deaths → buffer grows by exactly 2, order preserved.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a14_same_tick_two_deaths_plus_two_ordered() {
    // Type A — the only assertion exercising TWO simultaneous deaths; pins the
    // "buffer is a LIST, not a per-tile/per-tick slot" invariant (+2 exactly,
    // both present, order preserved), catching a slot-overwrite/same-tick-dedup bug.
    // PRODUCTION path: two controlled agents with identical low hp + saturated
    // Hunger die in the SAME StarvationSystem tick (same damage rate ⇒ same tick).
    let mut e = iso_engine();
    let (ent_a, _ida) = spawn_controlled(&mut e, 11, 11, 0.05);
    let (ent_b, _idb) = spawn_controlled(&mut e, 22, 22, 0.05);
    let len_before = e.resources.recent_deaths.len();

    let mut sys = StarvationSystem::new();
    let mut shared_tick: Option<u64> = None;
    for t in 0..50u64 {
        e.resources.current_tick = t;
        for ent in [ent_a, ent_b] {
            if let Ok(mut h) = e.world.get::<&mut Hunger>(ent) {
                h.value = Hunger::SATURATION;
            }
        }
        sys.tick(&mut e.world, &mut e.resources);
        if !alive(&e, ent_a) && !alive(&e, ent_b) {
            shared_tick = Some(t);
            break;
        }
        // Neither may die strictly before the other (identical hp ⇒ same tick).
        assert!(
            alive(&e, ent_a) == alive(&e, ent_b),
            "A14: identical-hp agents must die on the SAME tick (no split-tick death)"
        );
    }
    let shared_tick = shared_tick.expect("A14: both agents must die within 50 ticks");

    // +2 across the single shared tick — list semantics, not a slot overwrite.
    assert_eq!(
        e.resources.recent_deaths.len(),
        len_before + 2,
        "A14: two same-tick deaths → buffer grows by EXACTLY 2 (list, not a per-tick slot)"
    );
    let n = e.resources.recent_deaths.len();
    let e1 = &e.resources.recent_deaths[n - 2];
    let e2 = &e.resources.recent_deaths[n - 1];
    // Both deaths stamped with the shared tick; both distinct positions present.
    assert_eq!(u64::from(e1.tick), shared_tick, "A14: first entry stamped with the shared tick");
    assert_eq!(u64::from(e2.tick), shared_tick, "A14: second entry stamped with the shared tick");
    let positions = [(e1.x, e1.y), (e2.x, e2.y)];
    assert!(positions.contains(&(11, 11)), "A14: agent A's death tile present");
    assert!(positions.contains(&(22, 22)), "A14: agent B's death tile present");
    assert_ne!((e1.x, e1.y), (e2.x, e2.y), "A14: the two entries are DISTINCT (both kept, no dedup)");

    // Order: the collector preserves buffer (push/despawn) order verbatim.
    let rows = collect_recent_deaths(&e.resources);
    assert_eq!((rows[n - 2].x, rows[n - 2].y), (e1.x, e1.y), "A14: collector preserves entry order");
    assert_eq!((rows[n - 1].x, rows[n - 1].y), (e2.x, e2.y), "A14: collector preserves entry order");
    println!("[viz-D A14] two same-tick deaths → +2 exactly, both distinct, order preserved ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// A15: cross-language retain-window sync — GDScript RETAIN_TICKS == Rust const.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_death_a15_cross_language_retain_const_sync() {
    // Type D — a manually-mirrored constant has no compile-time link across the
    // FFI. If the renderer's retain window drifts from the Rust buffer's, the
    // fade/age normalization assumes a different window than the buffer keeps,
    // producing wrong tail-fade alphas. BOTH sides are read from their sources:
    // the Rust value is the authoritative `RECENT_DEATH_RETAIN_TICKS` const
    // itself (the compiled value, stronger than parsing its text); the GDScript
    // value is parsed from death_viz_renderer.gd.
    let renderer = read_file(&["scripts", "ui", "death_viz_renderer.gd"]);
    let gd_retain = parse_gd_const_f64(&renderer, "RETAIN_TICKS")
        .expect("A15: death_viz_renderer.gd must declare a numeric RETAIN_TICKS");
    assert_eq!(
        gd_retain, RECENT_DEATH_RETAIN_TICKS as f64,
        "A15: GDScript RETAIN_TICKS ({gd_retain}) must EQUAL Rust RECENT_DEATH_RETAIN_TICKS \
         ({RECENT_DEATH_RETAIN_TICKS}) — cross-language retain-window sync"
    );
    println!(
        "[viz-D A15] GDScript RETAIN_TICKS == Rust RECENT_DEATH_RETAIN_TICKS ({RECENT_DEATH_RETAIN_TICKS}) ✓"
    );
}
