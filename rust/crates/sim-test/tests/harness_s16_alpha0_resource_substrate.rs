//! V7 Section 16-α0 — Resource Substrate (non-depleting SOURCE tiles) harness.
//!
//! Verifies the α0 substrate:
//!   - `bootstrap_spawn_agents` (sim-bridge) populates deterministic
//!     non-depleting source tiles (value `RESOURCE_SOURCE_INFINITE` = 255)
//!     for food / water / sleep, alongside the existing 64-agent lattice.
//!   - `AgentDecisionSystem`'s `Consuming` arms skip the tile decrement for
//!     sentinel (255) tiles while keeping need-decrement + `Idle` transition
//!     UNCONDITIONAL; finite tiles keep decrement-and-remove behavior.
//!   - The `collect_resource_snapshot` / `resource_rows_split` FFI marshalling
//!     path emits deterministic, sorted integers.
//!   - Static contract guards: Bridge Identity Contract + decorative layer.
//!
//! Run:
//!   cargo test -p sim-test --test harness_s16_alpha0_resource_substrate -- --nocapture

use std::fs;
use std::path::PathBuf;

use sim_bridge::ffi::world_node::bootstrap_spawn_agents;
use sim_bridge::ffi::{collect_resource_snapshot, resource_rows_split, ResourceSnapshotRow};
use sim_core::components::{Agent, AgentState, Hunger, TargetKind};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine, SimResources, RESOURCE_SOURCE_INFINITE};
use sim_systems::runtime::decision::{AgentDecisionSystem, HUNGER_CONSUME_AMOUNT};

// ── constants mirrored from the production bootstrap (read-only) ────────────

/// Map extent — mirrors `world_node::DEFAULT_W` / `DEFAULT_H` (private).
const DEFAULT_W: u32 = 64;
const DEFAULT_H: u32 = 64;

/// Agent-lattice tiles: `BOOTSTRAP_AGENT_OFFSET(4) + i·STRIDE(8)` for i∈0..8.
const LATTICE: [u32; 8] = [4, 12, 20, 28, 36, 44, 52, 60];

/// Expected agent count: `BOOTSTRAP_AGENT_AXIS²` = 8² = 64.
const EXPECTED_AGENTS: usize = 64;

// ── helpers ─────────────────────────────────────────────────────────────────

fn project_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .expect("project root above sim-test crate")
}

fn read_file(rel: &[&str]) -> String {
    let mut path = project_root();
    for seg in rel {
        path.push(seg);
    }
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// Build a fresh engine via the production bootstrap source-population path.
fn bootstrapped_engine() -> SimEngine {
    let mut engine = SimEngine::new(DEFAULT_W, DEFAULT_H, MaterialRegistry::new());
    bootstrap_spawn_agents(&mut engine);
    engine
}

/// Minimum Chebyshev distance from `(x, y)` to any agent-lattice tile.
fn min_chebyshev_to_lattice(x: u32, y: u32) -> u32 {
    let mut best = u32::MAX;
    for &lx in LATTICE.iter() {
        for &ly in LATTICE.iter() {
            let dx = x.abs_diff(lx);
            let dy = y.abs_diff(ly);
            best = best.min(dx.max(dy));
        }
    }
    best
}

/// Drive a single genuine `Seeking{Food} → Consuming{Food}` cycle on `entity`
/// in `engine`, re-raising Hunger above the trigger and resetting AgentState
/// first (per the locked drive discipline). Returns `true` iff a real
/// `Consuming{Food}` execution fired (Seeking→Consuming then Consuming→Idle).
fn drive_one_food_consume(engine: &mut SimEngine, entity: hecs::Entity, sys: &mut AgentDecisionSystem) -> bool {
    // Re-raise Hunger + reset AgentState so this is a genuine transition,
    // NOT a re-read of the same tile value.
    engine
        .world
        .insert_one(entity, Hunger::new(80.0, 0.0))
        .expect("entity alive");
    engine
        .world
        .insert_one(entity, AgentState::Seeking { target: TargetKind::Food })
        .expect("entity alive");

    // Tick 1: Seeking → Consuming (genuine transition; tile must have value > 0).
    sys.tick(&mut engine.world, &mut engine.resources);
    let after_seek = *engine.world.get::<&AgentState>(entity).expect("state present");
    if after_seek != (AgentState::Consuming { target: TargetKind::Food }) {
        return false;
    }

    // Tick 2: Consuming executes (need-decrement + tile guard), → Idle.
    sys.tick(&mut engine.world, &mut engine.resources);
    let after_consume = *engine.world.get::<&AgentState>(entity).expect("state present");
    after_consume == AgentState::Idle
}

// ─── Assertion 1: bootstrap populates food source tiles ────────────────────
#[test]
fn harness_resource_bootstrap_populates_food_tiles() {
    // Type: D — bug #2 regression guard: >= 4 source tiles, <= 16 dup guard.
    let engine = bootstrapped_engine();
    let n = engine.resources.food_tiles.len();
    assert!(
        (4..=16).contains(&n),
        "A1: food_tiles must hold 4..=16 source tiles after bootstrap; got {n}"
    );
    println!("[S16-α0 A1] food_tiles.len() = {n} (∈ 4..=16) ✓");
}

// ─── Assertion 2: bootstrap populates water source tiles ───────────────────
#[test]
fn harness_resource_bootstrap_populates_water_tiles() {
    // Type: D — bug #2 regression guard for the water substrate.
    let engine = bootstrapped_engine();
    let n = engine.resources.water_tiles.len();
    assert!(
        (4..=16).contains(&n),
        "A2: water_tiles must hold 4..=16 source tiles after bootstrap; got {n}"
    );
    println!("[S16-α0 A2] water_tiles.len() = {n} (∈ 4..=16) ✓");
}

// ─── Assertion 3: bootstrap populates sleep source tiles ───────────────────
#[test]
fn harness_resource_bootstrap_populates_sleep_tiles() {
    // Type: D — bug #2 regression guard for the sleep substrate.
    let engine = bootstrapped_engine();
    let n = engine.resources.sleep_tiles.len();
    assert!(
        (4..=16).contains(&n),
        "A3: sleep_tiles must hold 4..=16 source tiles after bootstrap; got {n}"
    );
    println!("[S16-α0 A3] sleep_tiles.len() = {n} (∈ 4..=16) ✓");
}

// ─── Assertion 4: substrate is deterministic ───────────────────────────────
#[test]
fn harness_resource_substrate_deterministic() {
    // Type: A — invariant: fixed const arrays, zero RNG → byte-identical maps.
    let a = bootstrapped_engine();
    let b = bootstrapped_engine();
    assert_eq!(
        a.resources.food_tiles, b.resources.food_tiles,
        "A4.1: food_tiles must be identical across two bootstraps"
    );
    assert_eq!(
        a.resources.water_tiles, b.resources.water_tiles,
        "A4.2: water_tiles must be identical across two bootstraps"
    );
    assert_eq!(
        a.resources.sleep_tiles, b.resources.sleep_tiles,
        "A4.3: sleep_tiles must be identical across two bootstraps"
    );
    println!("[S16-α0 A4] two bootstraps produce identical food/water/sleep maps ✓");
}

// ─── Assertion 5: source tile never depletes (hand-seeded) ─────────────────
#[test]
fn harness_resource_source_never_depletes_handseeded() {
    // Type: A — sentinel invariant: >= 5 genuine consumes leave 255 intact.
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    let entity = engine.spawn_agent(5, 5);
    engine
        .world
        .insert(entity, (AgentState::Idle, Hunger::new(80.0, 0.0)))
        .expect("seed agent");
    engine
        .resources
        .set_food_tile(5, 5, RESOURCE_SOURCE_INFINITE);

    let mut sys = AgentDecisionSystem::new();
    let mut consume_count = 0u32;
    for _ in 0..6 {
        assert!(
            drive_one_food_consume(&mut engine, entity, &mut sys),
            "A5.1: each cycle must be a genuine Seeking→Consuming→Idle execution"
        );
        consume_count += 1;
    }
    // Type: A — at least 5 genuine consume executions fired.
    assert!(
        consume_count >= 5,
        "A5.2: expected >= 5 genuine Consuming executions; got {consume_count}"
    );
    // Type: A — tile still present at the sentinel value (never decremented).
    assert_eq!(
        engine.resources.food_tiles.get(&(5, 5)),
        Some(&RESOURCE_SOURCE_INFINITE),
        "A5.3: source tile must remain == 255 after {consume_count} consumes"
    );
    println!("[S16-α0 A5] {consume_count} genuine consumes; tile still == 255 ✓");
}

// ─── Assertion 6: need still satisfied at a source tile ────────────────────
#[test]
fn harness_resource_source_need_decrement_unconditional() {
    // Type: A — need-decrement + Idle transition fire even on a 255 source.
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    let entity = engine.spawn_agent(7, 7);
    engine
        .world
        .insert(
            entity,
            (
                AgentState::Consuming { target: TargetKind::Food },
                Hunger::new(80.0, 0.0),
            ),
        )
        .expect("seed agent");
    engine
        .resources
        .set_food_tile(7, 7, RESOURCE_SOURCE_INFINITE);

    let before = engine.world.get::<&Hunger>(entity).expect("hunger").value;
    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);
    let after = engine.world.get::<&Hunger>(entity).expect("hunger").value;
    let state = *engine.world.get::<&AgentState>(entity).expect("state");

    // Type: A — Hunger drops by exactly HUNGER_CONSUME_AMOUNT (within epsilon).
    assert!(
        (before - after - HUNGER_CONSUME_AMOUNT).abs() < 1e-3,
        "A6.1: Hunger must drop by HUNGER_CONSUME_AMOUNT ({HUNGER_CONSUME_AMOUNT}); \
         before={before} after={after}"
    );
    // Type: A — agent unconditionally transitions to Idle.
    assert_eq!(state, AgentState::Idle, "A6.2: agent must transition to Idle");
    // Type: A — source tile still 255 (decrement skipped).
    assert_eq!(
        engine.resources.food_tiles.get(&(7, 7)),
        Some(&RESOURCE_SOURCE_INFINITE),
        "A6.3: source tile unchanged at 255"
    );
    println!("[S16-α0 A6] need −{HUNGER_CONSUME_AMOUNT}, Idle, tile == 255 ✓");
}

// ─── Assertion 7: finite tiles unchanged (16/17 regression) ────────────────
#[test]
fn harness_resource_finite_tile_decrements_and_removes() {
    // Type: D — finite (< 255) tiles keep decrement-and-remove behavior.
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    let entity = engine.spawn_agent(9, 9);
    engine
        .world
        .insert(entity, (AgentState::Idle, Hunger::new(200.0, 0.0)))
        .expect("seed agent");
    engine.resources.set_food_tile(9, 9, 3);

    let mut sys = AgentDecisionSystem::new();
    let expected_after_each: [Option<u8>; 3] = [Some(2), Some(1), None];
    for (i, expected) in expected_after_each.iter().enumerate() {
        engine
            .world
            .insert_one(entity, Hunger::new(200.0, 0.0))
            .expect("entity alive");
        engine
            .world
            .insert_one(entity, AgentState::Consuming { target: TargetKind::Food })
            .expect("entity alive");
        sys.tick(&mut engine.world, &mut engine.resources);
        let got = engine.resources.food_tiles.get(&(9, 9)).copied();
        assert_eq!(
            got, *expected,
            "A7.{}: finite tile after consume #{} must be {:?}; got {:?}",
            i + 1,
            i + 1,
            expected,
            got
        );
    }
    println!("[S16-α0 A7] finite tile 3→2→1→removed ✓");
}

// ─── Assertion 8: FFI marshalling emits exact expected integers ────────────
#[test]
fn harness_resource_ffi_marshalling_literal_vectors() {
    // Type: A — non-circular: literal expected vectors from KNOWN inputs.
    let mut res = SimResources::new(DEFAULT_W, DEFAULT_H, MaterialRegistry::new());
    res.set_food_tile(8, 8, RESOURCE_SOURCE_INFINITE);
    res.set_water_tile(4, 32, RESOURCE_SOURCE_INFINITE);
    res.set_sleep_tile(20, 20, RESOURCE_SOURCE_INFINITE);

    let rows = collect_resource_snapshot(&res);
    let (xs, ys, kinds) = resource_rows_split(&rows);

    // Hand-authored expectations from the known seeds, sorted by (kind, x, y):
    //   food(8,8)→kind0, water(4,32)→kind1, sleep(20,20)→kind2.
    assert_eq!(xs, vec![8, 4, 20], "A8.1: xs must equal [8, 4, 20]");
    assert_eq!(ys, vec![8, 32, 20], "A8.2: ys must equal [8, 32, 20]");
    assert_eq!(kinds, vec![0, 1, 2], "A8.3: kinds must equal [0, 1, 2]");
    println!("[S16-α0 A8] resource_rows_split → ([8,4,20],[8,32,20],[0,1,2]) ✓");
}

// ─── Assertion 9: snapshot ordering deterministic and sorted ───────────────
#[test]
fn harness_resource_snapshot_sorted_and_deterministic() {
    // Type: A — strict (kind, x, y) ordering; call-1 == call-2.
    let mut res = SimResources::new(DEFAULT_W, DEFAULT_H, MaterialRegistry::new());
    // Scrambled insertion order, multiple per kind.
    res.set_food_tile(30, 5, RESOURCE_SOURCE_INFINITE);
    res.set_sleep_tile(2, 2, RESOURCE_SOURCE_INFINITE);
    res.set_food_tile(1, 9, RESOURCE_SOURCE_INFINITE);
    res.set_water_tile(50, 50, RESOURCE_SOURCE_INFINITE);
    res.set_food_tile(1, 1, RESOURCE_SOURCE_INFINITE);
    res.set_water_tile(3, 3, RESOURCE_SOURCE_INFINITE);

    let call1 = collect_resource_snapshot(&res);
    let call2 = collect_resource_snapshot(&res);
    // Type: A — two calls return identical Vecs.
    assert_eq!(call1, call2, "A9.1: snapshot must be deterministic across calls");
    // Type: A — strictly non-decreasing by (kind, x, y).
    for w in call1.windows(2) {
        let (a, b) = (w[0], w[1]);
        assert!(
            (a.kind, a.x, a.y) <= (b.kind, b.x, b.y),
            "A9.2: ordering violation: {a:?} then {b:?}"
        );
    }
    println!("[S16-α0 A9] snapshot sorted by (kind,x,y) + deterministic ✓");
}

// ─── Assertion 10: source tiles in-bounds and near the lattice ─────────────
#[test]
fn harness_resource_sources_in_bounds_and_near_lattice() {
    // Type: A (bounds) / E (reachability, soft, reported).
    let engine = bootstrapped_engine();
    let rows = collect_resource_snapshot(&engine.resources);
    assert!(!rows.is_empty(), "A10.0: snapshot must be non-empty after bootstrap");

    let mut max_dist = 0u32;
    for r in rows.iter() {
        // Type: A — hard bounds invariant.
        assert!(
            r.x < DEFAULT_W && r.y < DEFAULT_H,
            "A10.1: tile ({}, {}) out of 0..{}×0..{} bounds",
            r.x, r.y, DEFAULT_W, DEFAULT_H
        );
        let d = min_chebyshev_to_lattice(r.x, r.y);
        max_dist = max_dist.max(d);
    }
    eprintln!("[S16-α0 A10] max Chebyshev distance to lattice = {max_dist}");
    // Type: E — soft reachability heuristic (half the lattice spacing of 8).
    assert!(
        max_dist <= 8,
        "A10.2 (soft): every source must be within Chebyshev 8 of the lattice; max={max_dist}"
    );
    println!("[S16-α0 A10] all sources in-bounds; max lattice distance {max_dist} ≤ 8 ✓");
}

// ─── Assertion 11: Bridge Identity Contract (static) ───────────────────────
#[test]
fn harness_resource_bridge_identity_contract_static() {
    // Type: D — static grep: #[func] delegates; dict built from split.
    let src = read_file(&["rust", "crates", "sim-bridge", "src", "ffi", "world_node.rs"]);

    // Extract the get_resource_snapshot fn body.
    let fn_pos = src
        .find("fn get_resource_snapshot")
        .expect("A11.1: get_resource_snapshot must exist");
    let brace_rel = src[fn_pos..]
        .find('{')
        .expect("A11.2: get_resource_snapshot must have a body");
    let open = fn_pos + brace_rel;
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut end = open;
    for (i, b) in bytes[open..].iter().enumerate() {
        match *b as char {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = open + i;
                    break;
                }
            }
            _ => {}
        }
    }
    let body = &src[open..=end];

    // (a) Body delegates to the collector + dict converter.
    assert!(
        body.contains("collect_resource_snapshot(&self.engine.resources)"),
        "A11.3: #[func] body must call collect_resource_snapshot(&self.engine.resources); body:\n{body}"
    );
    assert!(
        body.contains("resource_rows_to_dict("),
        "A11.4: #[func] body must call resource_rows_to_dict(...); body:\n{body}"
    );
    // (a) No inline business logic / tile iteration inside the #[func].
    assert!(
        !body.contains("food_tiles") && !body.contains(".keys()") && !body.contains("for "),
        "A11.5: #[func] body must NOT contain inline tile iteration / business logic; body:\n{body}"
    );

    // (b) resource_rows_to_dict builds its PackedInt32Arrays from the split.
    let dict_pos = src
        .find("fn resource_rows_to_dict")
        .expect("A11.6: resource_rows_to_dict must exist");
    assert!(
        src[dict_pos..].contains("resource_rows_split("),
        "A11.7: resource_rows_to_dict must build from resource_rows_split(...)"
    );
    println!("[S16-α0 A11] Bridge Identity Contract: #[func] delegates; dict ← split ✓");
}

// ─── Assertion 12: decorative RESOURCE_SEED layer preserved (static) ───────
#[test]
fn harness_resource_decorative_layer_preserved_static() {
    // Type: D — Phase 13-β/14-β decorative scatter must remain.
    let src = read_file(&["scripts", "ui", "world_renderer.gd"]);
    assert!(
        src.contains("RESOURCE_SEED"),
        "A12.1: world_renderer.gd must still contain RESOURCE_SEED"
    );
    assert!(
        src.contains("RESOURCE_COUNT"),
        "A12.2: world_renderer.gd must still contain RESOURCE_COUNT"
    );
    println!("[S16-α0 A12] decorative RESOURCE_SEED + RESOURCE_COUNT preserved ✓");
}

// ─── Assertion 13: existing agent spawn intact ─────────────────────────────
#[test]
fn harness_resource_agent_spawn_intact() {
    // Type: A — 64 agents, all AgentState::Idle after bootstrap.
    let engine = bootstrapped_engine();
    let mut total = 0usize;
    let mut idle = 0usize;
    for (_, (_, state)) in engine.world.query::<(&Agent, &AgentState)>().iter() {
        total += 1;
        if *state == AgentState::Idle {
            idle += 1;
        }
    }
    assert_eq!(
        total, EXPECTED_AGENTS,
        "A13.1: bootstrap must spawn exactly {EXPECTED_AGENTS} agents; got {total}"
    );
    assert_eq!(
        idle, EXPECTED_AGENTS,
        "A13.2: all {EXPECTED_AGENTS} agents must start Idle; got {idle}"
    );
    println!("[S16-α0 A13] {total} agents, all Idle ✓");
}

// ─── Assertion 14: bootstrap source tiles hold the sentinel (CRITICAL) ─────
#[test]
fn harness_resource_bootstrap_tiles_are_sentinel() {
    // Type: A — every bootstrap source tile value == 255 (production path).
    let engine = bootstrapped_engine();
    for ((x, y), v) in engine.resources.food_tiles.iter() {
        assert_eq!(
            *v, RESOURCE_SOURCE_INFINITE,
            "A14.1: food tile ({x},{y}) must be the sentinel 255; got {v}"
        );
    }
    for ((x, y), v) in engine.resources.water_tiles.iter() {
        assert_eq!(
            *v, RESOURCE_SOURCE_INFINITE,
            "A14.2: water tile ({x},{y}) must be the sentinel 255; got {v}"
        );
    }
    for ((x, y), v) in engine.resources.sleep_tiles.iter() {
        assert_eq!(
            *v, RESOURCE_SOURCE_INFINITE,
            "A14.3: sleep tile ({x},{y}) must be the sentinel 255; got {v}"
        );
    }
    println!("[S16-α0 A14] every bootstrap source tile == 255 ✓");
}

// ─── Assertion 15: bootstrap-produced tile does not deplete (behavioral) ───
#[test]
fn harness_resource_bootstrap_tile_non_depleting_behavior() {
    // Type: A — drive consumption on a REAL bootstrap food tile; stays 255.
    let mut engine = bootstrapped_engine();
    // Select a real bootstrap food tile deterministically (smallest coord).
    let mut food_coords: Vec<(u32, u32)> = engine.resources.food_tiles.keys().copied().collect();
    food_coords.sort();
    let (fx, fy) = *food_coords.first().expect("A15.0: bootstrap must produce >=1 food tile");

    // Place a hungry agent ON that tile (distinct from the lattice agents).
    let entity = engine.spawn_agent(fx, fy);
    engine
        .world
        .insert(entity, (AgentState::Idle, Hunger::new(80.0, 0.0)))
        .expect("seed agent");

    let mut sys = AgentDecisionSystem::new();
    let mut consume_count = 0u32;
    for _ in 0..6 {
        assert!(
            drive_one_food_consume(&mut engine, entity, &mut sys),
            "A15.1: each cycle must be a genuine Seeking→Consuming→Idle execution on the bootstrap tile"
        );
        consume_count += 1;
    }
    // Type: A — >= 5 genuine consumes fired on the production tile.
    assert!(
        consume_count >= 5,
        "A15.2: expected >= 5 genuine Consuming executions; got {consume_count}"
    );
    // Type: A — the real bootstrap tile is still present at 255.
    assert_eq!(
        engine.resources.food_tiles.get(&(fx, fy)),
        Some(&RESOURCE_SOURCE_INFINITE),
        "A15.3: bootstrap tile ({fx},{fy}) must remain == 255 after {consume_count} consumes"
    );
    println!("[S16-α0 A15] bootstrap tile ({fx},{fy}): {consume_count} consumes, still 255 ✓");
}

// ─── Assertion 16: empty-snapshot path yields empty marshalling output ─────
#[test]
fn harness_resource_empty_snapshot_path() {
    // Type: A — n==0 contract: empty Vec + three empty split Vecs.
    let res = SimResources::new(DEFAULT_W, DEFAULT_H, MaterialRegistry::new());
    let rows = collect_resource_snapshot(&res);
    assert!(rows.is_empty(), "A16.1: empty SimResources → empty snapshot Vec");
    let (xs, ys, kinds) = resource_rows_split(&rows);
    assert!(xs.is_empty(), "A16.2: xs must be empty");
    assert!(ys.is_empty(), "A16.3: ys must be empty");
    assert!(kinds.is_empty(), "A16.4: kinds must be empty");
    // Touch the row type so the import is exercised even on the empty path.
    let _phantom: Option<ResourceSnapshotRow> = None;
    println!("[S16-α0 A16] empty SimResources → empty snapshot + empty split ✓");
}
