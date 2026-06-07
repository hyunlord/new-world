//! V7 viz-B `show-agent-needs-bars` — per-agent need surface (FFI) + the
//! head need-bar overlay (`need_bar_renderer.gd`).
//!
//! viz-A made resources visibly deplete; viz-B closes the causal loop by
//! showing WHO is starving/dehydrating/exhausted. The agent snapshot gains
//! `hunger` / `thirst` / `sleep` (the live need values, `[0, 100]`), and a
//! standalone overlay draws a yellow→red bar above each at-risk agent
//! (`danger = max(need)/100 ≥ 0.5`). Agents render via a single
//! MultiMeshInstance2D, so the overlay re-derives positions from the snapshot
//! each frame (the `seek_viz_renderer.gd` pattern).
//!
//! Assertion map (locked plan, plan_attempt 3):
//!   A1 need arrays mirror live components; length + order alignment ....... Type A
//!   A2 source fields not swapped (sleep from `.fatigue`); non-vacuity floors Type A/C
//!   A3 danger = max(need)/100 ≥ 0 ; at-risk path reachable ................ Type A/C
//!   A4 existing snapshot fields + locked `agent_rows_split` 4-tuple ....... Type D
//!   A5 renderer source + DANGER_THRESHOLD==0.5 + bridge key/field mapping . Type D
//!   A6 main.tscn additive wiring (+1 load_steps, +1 ext_resource) ......... Type D
//!
//! Assertion 7 (the cross-phase `load_steps` retarget) is NOT in this file's
//! 6-assertion budget — it is verified by the two EXPLICITLY-authorised locked
//! tests (`harness_p14_zeta_zoom_adaptive::a17`,
//! `harness_p14_epsilon_activity_trails::a12`, both now `load_steps == 12`) and
//! `harness_p14_delta` A11 (`load_steps >= 8`, unaffected).
//!
//! ─── Type C floor measurement (REQUIRED by plan; re-measure on change) ───
//! Measured at seed 42, 20 agents, 2000 ticks, on 2026-06-05 against the
//! working tree atop commit 6dfa7f7c. This file's `make_stage1_engine` uses
//! EXPLICIT moderate sub-saturation growth rates (NOT the production
//! BOOTSTRAP_* constants): production rates (Thirst 0.08, Hunger 0.05) drive
//! BOTH hunger and thirst to the `SATURATION == 100` clamp well before tick
//! 2000, which would (a) make hunger == thirst, collapsing the A2(c)
//! mutually-distinct floor to 0, and (b) trip the `>= SATURATION` starvation
//! damage path and despawn agents mid-run. The fixture instead places the
//! three needs in non-overlapping sub-100 bands by tick 2000
//! (sleep≈[16,18] < hunger≈[44,51] < thirst≈[80,84]) so every need is
//! distinct AND the worst need is in the at-risk band (≥ 50 ⟺ danger ≥ 0.5).
//! If the need-growth model OR these fixture rates change, RE-MEASURE the
//! eprintln'd counts and reset the floors below to max(1, floor(k×observed)).
//!   A2(b) `fatigue != growth_rate` subset .. observed 20 → floor max(1, ⌊0.5·20⌋) = 10
//!   A2(c) mutually-distinct-need agents .... observed 20 → floor max(1, ⌊0.5·20⌋) = 10
//!   A3(c) peak at-risk (max ≥ 50) count .... observed 20 → floor max(1, ⌊0.3·20⌋) =  6
//! An observed count of 0 is a REPORTABLE FINDING (vacuous guard), not a pass.

use std::collections::HashMap;
use std::path::PathBuf;

use sim_bridge::ffi::{agent_rows_split, collect_agent_snapshot, AgentSnapshotRow};
use sim_core::components::{Agent, Hunger, Position, SeekTarget, Sleep, Thirst};
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;

const W: u32 = 64;
const H: u32 = 64;
/// Locked tick horizon for the FFI assertions (plan: ticks 2000).
const TICKS: u32 = 2000;
/// Need value at full saturation — mirrors `Hunger::SATURATION` and the
/// renderer's `SATURATION` constant; the danger ratio is `value / 100`.
const SATURATION: f32 = 100.0;
/// Danger ratio at/above which the renderer draws a bar — mirrors
/// `need_bar_renderer.gd::DANGER_THRESHOLD`. The at-risk need band is
/// `max(h,t,s) >= 50.0` ⟺ `danger >= 0.5`.
const DANGER_THRESHOLD: f32 = 0.5;
/// Number of agents the fixture spawns (plan: agent_count 20).
const AGENTS: u32 = 20;

// ── Committed Type C floors (see header measurement block) ──────────────────
const FLOOR_A2B_FATIGUE_NE_GROWTH: usize = 10;
const FLOOR_A2C_DISTINCT_NEEDS: usize = 10;
const FLOOR_A3C_AT_RISK_PEAK: usize = 6;

/// Advance the engine `n` ticks. `SimEngine` exposes only single-tick
/// `tick()`, so this is the canonical sim-test pattern.
fn run_ticks(engine: &mut SimEngine, n: u32) {
    for _ in 0..n {
        engine.tick();
    }
}

/// Build a stage-1 engine with `AGENTS` agents on a deterministic lattice,
/// each carrying (Agent, Position, MovementRng, Hunger, Thirst, Sleep,
/// AgentState). Need growth uses EXPLICIT moderate sub-saturation rates (see
/// the header block) so the three needs stay distinct AND climb into the
/// at-risk band by tick 2000 — production rates would saturate hunger==thirst
/// and risk starvation despawn. No startup buildings / resource sources, so
/// needs climb monotonically (no consumption) and there is no
/// settlement-migration freeze.
fn make_stage1_engine(seed: u64, agent_count: u32) -> SimEngine {
    use sim_core::components::AgentState;
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);
    // Rates chosen so by tick 2000 (× rate): sleep ≈ +16, hunger ≈ +44,
    // thirst ≈ +80 — non-overlapping bands, all < 100.
    const HUNGER_RATE: f32 = 0.022;
    const THIRST_RATE: f64 = 0.040;
    const SLEEP_RATE: f64 = 0.008;
    for i in 0..agent_count {
        let x = 16 + (i % 4);
        let y = 16 + (i / 4);
        let entity = engine.spawn_agent(x, y);
        // Small per-agent staggered starts (deterministic, not RNG) so agents
        // diverge cross-row; the within-agent distinctness comes from the
        // distinct bands above.
        let h0 = (i % 8) as f32;
        let t0 = (i % 5) as f64;
        let sl0 = (i % 3) as f64;
        engine
            .world
            .insert(
                entity,
                (
                    MovementRng::new(seed.wrapping_add(i as u64)),
                    Hunger::new(h0, HUNGER_RATE),
                    Thirst::new(t0, THIRST_RATE),
                    Sleep::new(sl0, SLEEP_RATE),
                    AgentState::Idle,
                ),
            )
            .expect("freshly spawned agent must still exist");
    }
    engine
}

fn engine() -> SimEngine {
    make_stage1_engine(42, AGENTS)
}

fn danger(r: &AgentSnapshotRow) -> f32 {
    r.hunger.max(r.thirst).max(r.sleep) / SATURATION
}

/// Live per-entity mirror of the need/position values, keyed by
/// `Entity::to_bits().get()` (the value the row's `entity_bits` and the
/// split's `ids` array carry). Need values use the SAME `unwrap_or(0.0)` +
/// cast rules as `collect_agent_snapshot` so the comparison is bit-exact.
struct LiveAgent {
    x: u32,
    y: u32,
    hunger: f32,
    thirst: f32,
    sleep: f32,
    agent_id: u64,
    has_seek: bool,
}

fn live_map(e: &SimEngine) -> HashMap<u64, LiveAgent> {
    let mut map = HashMap::new();
    for (ent, (agent, pos, mh, mt, ms, mseek)) in e
        .world
        .query::<(
            &Agent,
            &Position,
            Option<&Hunger>,
            Option<&Thirst>,
            Option<&Sleep>,
            Option<&SeekTarget>,
        )>()
        .iter()
    {
        map.insert(
            ent.to_bits().get(),
            LiveAgent {
                x: pos.x,
                y: pos.y,
                hunger: mh.map(|h| h.value).unwrap_or(0.0),
                thirst: mt.map(|t| t.value as f32).unwrap_or(0.0),
                sleep: ms.map(|s| s.fatigue as f32).unwrap_or(0.0),
                agent_id: agent.id,
                has_seek: mseek.is_some(),
            },
        );
    }
    map
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

// ─── A1: need arrays mirror live components — value, length, order ──────────
#[test]
fn harness_needs_a1_arrays_mirror_components_length_and_order() {
    // Type A — the snapshot is a pure read of live components and the FFI need
    // arrays ARE the row f32 fields verbatim, so this is an exact marshalling
    // invariant (not a tuning value). The renderer paints the bar at
    // (xs[i], ys[i]); a row-order scramble would draw the wrong agent's needs,
    // so order alignment (c) is load-bearing.
    let mut e = engine();
    run_ticks(&mut e, TICKS);
    let rows = collect_agent_snapshot(&e.world);
    let live = live_map(&e);

    // (b) Length — rows.len() == live (Agent,Position) count.
    assert_eq!(
        rows.len(),
        live.len(),
        "A1(b): rows.len() must equal live (Agent,Position) count"
    );
    assert!(!rows.is_empty(), "A1(b): seed-42 fixture must produce agents");

    let mut value_devs = 0usize;
    let mut pos_misaligns = 0usize;
    for (i, r) in rows.iter().enumerate() {
        let la = live
            .get(&r.entity_bits)
            .unwrap_or_else(|| panic!("A1: row {i} entity_bits has no live entity"));
        // (a) Value mirror — exact (same component, same casts).
        if (r.hunger - la.hunger).abs() > f32::EPSILON {
            value_devs += 1;
        }
        if (r.thirst - la.thirst).abs() > f32::EPSILON {
            value_devs += 1;
        }
        if (r.sleep - la.sleep).abs() > f32::EPSILON {
            value_devs += 1;
        }
        // (c) Order alignment — the need value at index i belongs to the same
        // agent whose position sits at index i.
        if r.x != la.x || r.y != la.y {
            pos_misaligns += 1;
        }
    }
    assert_eq!(value_devs, 0, "A1(a): need value deviations must be 0");
    assert_eq!(pos_misaligns, 0, "A1(c): row-order/position misalignments must be 0");
    println!(
        "[viz-B A1] {} rows mirror live needs; 0 value devs, 0 order misaligns ✓",
        rows.len()
    );
}

// ─── A2: source fields not swapped; non-vacuity floors ──────────────────────
#[test]
fn harness_needs_a2_source_fields_not_swapped_with_floors() {
    let mut e = engine();
    run_ticks(&mut e, TICKS);
    let rows = collect_agent_snapshot(&e.world);

    // Re-read the raw live components to drive the source-correctness check
    // (sleep MUST come from `.fatigue`, never `.growth_rate`; hunger/thirst
    // not transposed).
    let mut comp: HashMap<u64, (f32, f32, f32, f32)> = HashMap::new(); // h, t, fatigue, growth_rate
    for (ent, (_a, h, t, s)) in e
        .world
        .query::<(&Agent, &Hunger, &Thirst, &Sleep)>()
        .iter()
    {
        comp.insert(
            ent.to_bits().get(),
            (h.value, t.value as f32, s.fatigue as f32, s.growth_rate as f32),
        );
    }

    let mut source_violations = 0usize;
    let mut fatigue_ne_growth = 0usize;
    let mut distinct_needs = 0usize;
    for r in &rows {
        let (h, t, fatigue, growth) = comp[&r.entity_bits];
        // (a) Source correctness — no transposition.
        if (r.hunger - h).abs() > f32::EPSILON {
            source_violations += 1; // hunger not from Hunger.value
        }
        if (r.thirst - t).abs() > f32::EPSILON {
            source_violations += 1; // thirst not from Thirst.value
        }
        if (r.sleep - fatigue).abs() > f32::EPSILON {
            source_violations += 1; // sleep not from Sleep.fatigue
        }
        // The `!= growth_rate` clause only bites where they differ.
        if (fatigue - growth).abs() > f32::EPSILON {
            fatigue_ne_growth += 1;
            if (r.sleep - growth).abs() <= f32::EPSILON {
                source_violations += 1; // sleep wrongly sourced from growth_rate
            }
        }
        // (c) Cross-need divergence — all three mutually distinct.
        if (r.hunger - r.thirst).abs() > f32::EPSILON
            && (r.thirst - r.sleep).abs() > f32::EPSILON
            && (r.hunger - r.sleep).abs() > f32::EPSILON
        {
            distinct_needs += 1;
        }
    }
    eprintln!(
        "[viz-B A2] observed: fatigue!=growth={fatigue_ne_growth}, distinct-needs={distinct_needs} \
         (floors {FLOOR_A2B_FATIGUE_NE_GROWTH} / {FLOOR_A2C_DISTINCT_NEEDS})"
    );
    // (a) Type A — exact mapping invariant.
    assert_eq!(source_violations, 0, "A2(a): field-source violations must be 0");
    // (b)/(c) Type C — non-vacuity floors (a 0 observation is a finding, not a pass).
    assert!(
        fatigue_ne_growth >= FLOOR_A2B_FATIGUE_NE_GROWTH,
        "A2(b): fatigue!=growth_rate subset {fatigue_ne_growth} must be >= floor {FLOOR_A2B_FATIGUE_NE_GROWTH} \
         (else the `!= growth_rate` swap guard is vacuous — re-measure)"
    );
    assert!(
        distinct_needs >= FLOOR_A2C_DISTINCT_NEEDS,
        "A2(c): mutually-distinct-need agents {distinct_needs} must be >= floor {FLOOR_A2C_DISTINCT_NEEDS} \
         (else a hunger<->thirst transposition is unobservable — re-measure)"
    );
    println!("[viz-B A2] sleep from .fatigue, no transposition; floors met ✓");
}

// ─── A3: danger ratio non-negative; at-risk path reachable ──────────────────
#[test]
fn harness_needs_a3_danger_ratio_and_at_risk_reachable() {
    // (a) Non-negativity — Type A invariant. Run once and check every row.
    let mut e = engine();
    let mut max_danger = 0.0f32;
    let mut peak_at_risk = 0usize;
    // (c) Reachability — sample at 100-tick checkpoints, record the PEAK count
    // of at-risk rows (max(h,t,s) >= 50.0 ⟺ danger >= 0.5).
    let checkpoints = TICKS / 100;
    for _ in 0..checkpoints {
        run_ticks(&mut e, 100);
        let rows = collect_agent_snapshot(&e.world);
        let mut at_risk = 0usize;
        for r in &rows {
            let d = danger(r);
            assert!(d >= 0.0, "A3(a): danger must be >= 0 (needs are never negative); got {d}");
            if d > max_danger {
                max_danger = d;
            }
            if r.hunger.max(r.thirst).max(r.sleep) >= 50.0 {
                at_risk += 1;
            }
        }
        if at_risk > peak_at_risk {
            peak_at_risk = at_risk;
        }
    }
    // (b) Upper bound — reported softly (no hard <= 1.0; the [0,100] clamp lives
    // in another, churning system).
    eprintln!(
        "[viz-B A3] observed max danger = {max_danger:.4} (soft, no upper-bound assert); \
         peak at-risk = {peak_at_risk} (floor {FLOOR_A3C_AT_RISK_PEAK})"
    );
    assert!(
        peak_at_risk >= FLOOR_A3C_AT_RISK_PEAK,
        "A3(c): peak at-risk count {peak_at_risk} must be >= floor {FLOOR_A3C_AT_RISK_PEAK} \
         (else the renderer's bar-draw branch is unreachable by real data — re-measure)"
    );
    println!("[viz-B A3] danger >= 0 for all rows; at-risk path reachable ✓");
}

// ─── A4: existing snapshot fields + locked 4-tuple split preserved ──────────
#[test]
fn harness_needs_a4_existing_fields_and_split_preserved() {
    // Type D regression guard — viz-B is strictly additive. The
    // `agent_rows_split` 4-tuple is locked by `harness_p4_gamma_rendering`,
    // and the Section 16-ε head-dot/goal-line fields must be untouched.
    let mut e = engine();
    run_ticks(&mut e, TICKS);
    let rows = collect_agent_snapshot(&e.world);
    let live = live_map(&e);
    assert!(!rows.is_empty(), "A4: fixture must produce rows");

    for r in &rows {
        let la = live
            .get(&r.entity_bits)
            .expect("A4: every row entity_bits maps to a live entity");
        assert_eq!(r.x, la.x, "A4: x preserved");
        assert_eq!(r.y, la.y, "A4: y preserved");
        assert!(r.state_tag <= 3, "A4: state_tag must be in {{0,1,2,3}}; got {}", r.state_tag);
        assert!(r.seek_kind <= 3, "A4: seek_kind must be in {{0,1,2,3}}; got {}", r.seek_kind);
        assert_eq!(r.agent_id, la.agent_id, "A4: agent_id mirrors Agent.id");
        if !la.has_seek {
            assert_eq!(r.target_x, -1, "A4: absent SeekTarget → target_x sentinel -1");
            assert_eq!(r.target_y, -1, "A4: absent SeekTarget → target_y sentinel -1");
        }
    }

    // Locked 4-tuple split: arity 4 (destructuring is a compile-time arity
    // check), all lengths == rows.len(), and the exact value mappings.
    let (ids, xs, ys, states) = agent_rows_split(&rows);
    assert_eq!(ids.len(), rows.len(), "A4: ids length == row count");
    assert_eq!(xs.len(), rows.len(), "A4: xs length == row count");
    assert_eq!(ys.len(), rows.len(), "A4: ys length == row count");
    assert_eq!(states.len(), rows.len(), "A4: states length == row count");
    for (i, r) in rows.iter().enumerate() {
        assert_eq!(ids[i], r.entity_bits as i64, "A4: ids[i] == entity_bits as i64");
        assert_eq!(xs[i], r.x as i32, "A4: xs[i] == x as i32");
        assert_eq!(ys[i], r.y as i32, "A4: ys[i] == y as i32");
        assert_eq!(states[i], r.state_tag, "A4: states[i] == state_tag");
    }
    println!("[viz-B A4] existing fields + locked agent_rows_split 4-tuple preserved ✓");
}

// ─── A5: renderer source + gate value + exact bridge key/field mapping ──────
#[test]
fn harness_needs_a5_renderer_and_bridge_contract() {
    // Type D — structural integration invariant across renderer + bridge.
    // (a) Renderer source tokens.
    let r = read_file(&["scripts", "ui", "need_bar_renderer.gd"]);
    assert!(r.contains("extends Node2D"), "A5(a): renderer extends Node2D");
    assert!(r.contains("get_agent_snapshot"), "A5(a): renderer reads get_agent_snapshot");
    for key in ["hungers", "thirsts", "sleeps"] {
        assert!(r.contains(key), "A5(a): renderer must read the `{key}` snapshot array");
    }
    assert!(r.contains("xs") && r.contains("ys"), "A5(a): renderer reads positions xs/ys");
    assert!(r.contains("DANGER_THRESHOLD"), "A5(a): renderer declares DANGER_THRESHOLD");
    assert!(r.contains("Z_NEED_BAR") && r.contains("7"), "A5(a): renderer declares Z_NEED_BAR = 7");
    assert!(r.contains("draw_rect"), "A5(a): renderer draws via draw_rect (track + fill)");
    assert!(r.contains(".lerp("), "A5(a): renderer lerps colour yellow→red");
    let has_max_ratio =
        (r.contains("max") && r.contains("/ SATURATION")) || r.contains("/ 100");
    assert!(has_max_ratio, "A5(a): renderer computes a max-of-three danger ratio");

    // (b) Gate-value tie — DANGER_THRESHOLD literal must equal 0.5 (the value
    // A3's at-risk band `max >= 50` is measured against). Parse the literal.
    let gate = parse_gd_const_f32(&r, "DANGER_THRESHOLD")
        .expect("A5(b): need_bar_renderer.gd must declare a numeric DANGER_THRESHOLD");
    assert!(
        (gate - DANGER_THRESHOLD).abs() < f32::EPSILON,
        "A5(b): DANGER_THRESHOLD must equal {DANGER_THRESHOLD}; got {gate}"
    );

    // (c) Bridge key+field mapping — the FFI source must insert under the EXACT
    // key strings and populate each from the matching row field.
    let b = read_file(&["rust", "crates", "sim-bridge", "src", "ffi", "world_node.rs"]);
    for (key, field) in [("hungers", "hunger"), ("thirsts", "thirst"), ("sleeps", "sleep")] {
        assert!(
            b.contains(&format!("dict.set(\"{key}\"")),
            "A5(c): bridge must insert under the exact key string \"{key}\""
        );
        // The array variable carrying the key is filled from `row.<field>`.
        assert!(
            b.contains(&format!("{key}[i] = row.{field}")),
            "A5(c): `{key}` array must be populated from `row.{field}` (no swap)"
        );
    }
    println!("[viz-B A5] renderer source + DANGER_THRESHOLD==0.5 + bridge key/field mapping ✓");
}

/// Parse `const NAME := <float>` or `const NAME: float = <float>` from a
/// GDScript source, returning the literal as f32.
fn parse_gd_const_f32(src: &str, name: &str) -> Option<f32> {
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("const ") || !t.contains(name) {
            continue;
        }
        let rhs = t.rsplit_once('=').map(|(_, r)| r.trim())?;
        // Strip a trailing comment if present.
        let rhs = rhs.split('#').next().unwrap_or(rhs).trim();
        if let Ok(v) = rhs.parse::<f32>() {
            return Some(v);
        }
    }
    None
}

// ─── A6: scene wiring is additive (delta-based, offset-independent) ─────────
#[test]
fn harness_needs_a6_scene_wiring_additive() {
    // Type D additive-preservation regression. The +1 DELTA is the invariant
    // (offset-independent), never an absolute identity. The baseline tracks the
    // shared scene: viz-B's own land put it at load_steps/ext = 12; viz-D then
    // added death_viz_renderer.gd, so this guard now describes the 12 → 13
    // transition (baseline 12, +1 → 13). The +1-additive invariant is preserved.
    const LOAD_STEPS_BEFORE: i64 = 12;
    const EXT_COUNT_BEFORE: usize = 12;

    let scene = read_file(&["scenes", "main.tscn"]);

    let load_steps_after = tscn_load_steps(&scene).expect("A6: main.tscn must declare load_steps=N");
    let ext_count_after = scene
        .lines()
        .filter(|l| l.trim_start().starts_with("[ext_resource type="))
        .count();

    // (b) Additive delta — exactly one added on each.
    assert_eq!(
        load_steps_after,
        LOAD_STEPS_BEFORE + 1,
        "A6(b): load_steps must be baseline+1 ({}); got {load_steps_after}",
        LOAD_STEPS_BEFORE + 1
    );
    assert_eq!(
        ext_count_after,
        EXT_COUNT_BEFORE + 1,
        "A6(b): ext_resource count must be baseline+1 ({}); got {ext_count_after}",
        EXT_COUNT_BEFORE + 1
    );

    // (c) Additive preservation — new wiring present, prior wiring intact.
    assert!(
        scene.contains("need_bar_renderer.gd"),
        "A6(c): new ext_resource must reference need_bar_renderer.gd"
    );
    let has_node = scene.lines().any(|l| {
        let t = l.trim();
        t.starts_with("[node ") && t.contains("name=\"NeedBarRenderer\"") && t.contains("type=\"Node2D\"")
    });
    assert!(has_node, "A6(c): NeedBarRenderer Node2D node must exist");
    // Every previously-registered renderer/overlay node + ext_resource preserved.
    for token in [
        "name=\"WorldRenderer\"",
        "name=\"AgentRenderer\"",
        "name=\"ActivityTrailRenderer\"",
        "name=\"SeekVizRenderer\"",
        "name=\"SettlementOverviewRenderer\"",
        "name=\"ZoomLodController\"",
        "seek_viz_renderer.gd",
        "settlement_overview_renderer.gd",
        "activity_trail_renderer.gd",
    ] {
        assert!(scene.contains(token), "A6(c): pre-existing wiring `{token}` must be preserved (removal==0)");
    }
    println!(
        "[viz-B A6] load_steps {LOAD_STEPS_BEFORE}→{load_steps_after}, ext {EXT_COUNT_BEFORE}→{ext_count_after}; \
         NeedBarRenderer wired, prior nodes preserved ✓"
    );
}

/// Extract `load_steps=N` from a `[gd_scene ... load_steps=N ...]` header,
/// tolerating whitespace around `=`.
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
