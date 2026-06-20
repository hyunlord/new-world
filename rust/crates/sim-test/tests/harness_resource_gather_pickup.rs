//! Direction-2 slice 2-2 — Gather pickup (ground food → Inventory) harness.
//!
//! feature: inventory-2-2-pickup
//! plan_attempt: 2
//! code_attempt: 1
//! seed: 42
//! agent_count: 20
//! lane: --full (sim-systems decision hot-path change)
//!
//! A satisfied agent walks to nearby ground food and picks it up into its
//! `Inventory` (CARRY, not eat). Distinct FSM target `TargetKind::GatherFood`,
//! new lowest-but-one cascade arm `CascadeArm::Gather` (priority 6, below
//! Hunger/Thirst/Fatigue/Construction/Social/Combat, above Settlement).
//!
//! Assertion map (1:1 with the locked plan §Assertions):
//!   A1  : inventories EMPTY at tick 0 (sum==0, per-agent max==0).
//!   A2  : >=5 agents' Food rose from 0 to >=1 after the span (non-vacuity).
//!   A3  : per-agent total() <= INVENTORY_CAPACITY (overflow guard).
//!   A4  : inventories hold ONLY Food (Water+Wood+Stone sum == 0).
//!   A5  : finite tile depletes by exactly amount picked up (single-agent
//!         conservation).
//!   A6  : positive infinite-source (255) pickup WITHOUT sentinel corruption —
//!         controlled fixture, regen disabled, sentinel sampled mid-span + end.
//!   A7  : capacity-boundary conservation — room < tile amount; gain == room,
//!         tile decrement == room (overflow-return path); R==0 takes 0.
//!   A8  : concurrent pickup from one finite tile conserves food.
//!   A9  : gathering agent physically MOVES toward its target (no teleport).
//!         (+ radius-confounder regression: a closer out-of-radius tile must
//!          not mask an eligible in-radius tile.)
//!   A10 : gather does NOT decrease Hunger.
//!   A11 : Hunger arm preempts Gather (a hungry agent eats, never gathers).
//!   A12 : no targetless Seeking across the full span (freeze guard).
//!   A13 : no agent stuck in gather states beyond a bounded streak (<=200).
//!   A14 : Consuming{GatherFood} is a single-tick unconditional exit; >=3 cycles.
//!   A15 : member persistence under gathering (no membership collapse) + away>=1.
//!   A16 : lockstep determinism.
//!   A17 : suppresses_movement FSM correctness (Seeking{GatherFood}->true,
//!         Consuming{GatherFood}->false).
//!
//! Run:
//!   cargo test -p sim-test --test harness_resource_gather_pickup -- --nocapture

use std::collections::{BTreeMap, HashMap, HashSet};

use sim_bridge::ffi::world_node::{bootstrap_spawn_agents, enqueue_building_placed};
use sim_core::components::{
    Agent, AgentId, AgentState, BodyHealth, Hunger, Inventory, Memory, Position, ResourceKind,
    SeekTarget, Sleep, Social, TargetKind, Thirst, INVENTORY_CAPACITY,
    SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::decision::GATHER_SCAN_RADIUS;
use sim_systems::runtime::influence::BuildingStampSystem;

// ── Stage-1 bulk engine ─────────────────────────────────────────────────────

const STAGE1_W: u32 = 64;
const STAGE1_H: u32 = 64;
const YEAR_TICKS: u64 = 4380;

/// Infinite food sources seeded near the agent cluster so every bulk agent has
/// an eligible Gather tile within `GATHER_SCAN_RADIUS`. Sentinel-valued (255) so
/// they never deplete — this is the COMMON gather path (Assertion 6).
const BULK_FOOD: [(u32, u32); 5] = [(16, 16), (18, 18), (19, 20), (12, 18), (22, 18)];

/// Build the Stage-1 engine. 20 agents clustered at (16+i%4, 16+i/4) with an
/// EMPTY `Inventory::default()` and FLAT (rate 0) needs so the breach cascade is
/// always None → every Idle agent reaches the Gather arm. Infinite food tiles
/// are seeded in range. No buildings → no settlement interference.
fn make_stage1_engine(seed: u64, agent_count: u32) -> SimEngine {
    let mut engine = SimEngine::new(STAGE1_W, STAGE1_H, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);
    for i in 0..agent_count {
        let x = 16 + (i % 4);
        let y = 16 + (i / 4);
        let entity = engine.spawn_agent(x, y);
        engine
            .world
            .insert(
                entity,
                (
                    MovementRng::new(seed.wrapping_add(i as u64)),
                    AgentState::Idle,
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.0),
                    Memory::new(),
                    BodyHealth::new(),
                    Inventory::default(),
                ),
            )
            .expect("freshly spawned agent must still exist");
    }
    for &(x, y) in BULK_FOOD.iter() {
        engine.resources.set_food_tile(x, y, RESOURCE_SOURCE_INFINITE);
    }
    engine
}

// ── snapshot helpers ────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct AgentSnap {
    state: AgentState,
    has_seek: bool,
}

/// Snapshot every agent's (state, has-SeekTarget) for the freeze/cycle metrics.
fn snapshot_agents(e: &SimEngine) -> BTreeMap<AgentId, AgentSnap> {
    let mut map = BTreeMap::new();
    for (_, (agent, state, seek)) in e
        .world
        .query::<(&Agent, &AgentState, Option<&SeekTarget>)>()
        .iter()
    {
        map.insert(
            agent.id,
            AgentSnap {
                state: *state,
                has_seek: seek.is_some(),
            },
        );
    }
    map
}

/// Metrics gathered by a single tick-by-tick bulk run.
struct Bulk {
    targetless_seeking: u64,
    max_gather_streak: u64,
    consuming_over_one: u64,
    complete_cycles: u64,
    engine: SimEngine,
}

/// Run `ticks` ticks of the bulk engine tracking every freeze/cycle metric per
/// tick (per-tick sampling — A12 forbids subsampling).
fn bulk_run(seed: u64, agents: u32, ticks: u64) -> Bulk {
    let mut e = make_stage1_engine(seed, agents);

    let mut prev = snapshot_agents(&e);
    let mut gather_streak: HashMap<AgentId, u64> = HashMap::new();
    let mut consuming_streak: HashMap<AgentId, u64> = HashMap::new();

    let mut targetless_seeking = 0u64;
    let mut max_gather_streak = 0u64;
    let mut consuming_over_one = 0u64;
    let mut complete_cycles = 0u64;

    for _ in 1..=ticks {
        e.tick();
        let snap = snapshot_agents(&e);

        for (id, s) in &snap {
            // A12 — a Seeking state must always carry a SeekTarget.
            if matches!(s.state, AgentState::Seeking { .. }) && !s.has_seek {
                targetless_seeking += 1;
            }

            // A13 — gather-state frozen streak.
            let in_gather = matches!(
                s.state,
                AgentState::Seeking { target: TargetKind::GatherFood }
                    | AgentState::Consuming { target: TargetKind::GatherFood }
            );
            let g = gather_streak.entry(*id).or_insert(0);
            if in_gather {
                *g += 1;
                max_gather_streak = max_gather_streak.max(*g);
            } else {
                *g = 0;
            }

            // A14 — Consuming{GatherFood} occupancy must be exactly 1 tick.
            let in_consuming =
                matches!(s.state, AgentState::Consuming { target: TargetKind::GatherFood });
            let c = consuming_streak.entry(*id).or_insert(0);
            if in_consuming {
                *c += 1;
                if *c > 1 {
                    consuming_over_one += 1;
                }
            } else {
                *c = 0;
            }

            // A14 — complete Idle→…→Consuming{GatherFood}→Idle round-trip.
            if let Some(p) = prev.get(id) {
                let was_consuming_gather =
                    matches!(p.state, AgentState::Consuming { target: TargetKind::GatherFood });
                let now_idle = matches!(s.state, AgentState::Idle);
                if was_consuming_gather && now_idle {
                    complete_cycles += 1;
                }
            }
        }

        prev = snap;
    }

    Bulk {
        targetless_seeking,
        max_gather_streak,
        consuming_over_one,
        complete_cycles,
        engine: e,
    }
}

// ── fixture helpers ─────────────────────────────────────────────────────────

const FIX_W: u32 = 32;
const FIX_H: u32 = 32;

fn fixture_engine() -> SimEngine {
    let mut e = SimEngine::new(FIX_W, FIX_H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    e
}

/// Spawn one fully-satisfied agent (all needs value 0, rate 0 → never breaches)
/// with `preload_food` Food already carried. Returns `(entity, AgentId)`.
fn spawn_gatherer(e: &mut SimEngine, x: u32, y: u32, preload_food: u32) -> (hecs::Entity, AgentId) {
    let ent = e.spawn_agent(x, y);
    let id = e.world.get::<&Agent>(ent).map(|a| a.id).expect("agent id");
    let mut inv = Inventory::default();
    if preload_food > 0 {
        inv.add(ResourceKind::Food, preload_food);
    }
    e.world
        .insert(
            ent,
            (
                MovementRng::new(7),
                AgentState::Idle,
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 0.0),
                Memory::new(),
                BodyHealth::new(),
                inv,
            ),
        )
        .expect("fixture agent must exist");
    (ent, id)
}

fn agent_food(e: &SimEngine, ent: hecs::Entity) -> u32 {
    e.world
        .get::<&Inventory>(ent)
        .map(|i| i.get(ResourceKind::Food))
        .unwrap_or(0)
}

fn agent_state(e: &SimEngine, ent: hecs::Entity) -> AgentState {
    *e.world.get::<&AgentState>(ent).expect("state")
}

fn agent_pos(e: &SimEngine, ent: hecs::Entity) -> (u32, u32) {
    let p = e.world.get::<&Position>(ent).expect("pos");
    (p.x, p.y)
}

fn chebyshev(a: (u32, u32), b: (u32, u32)) -> u32 {
    let dx = (a.0 as i64 - b.0 as i64).unsigned_abs();
    let dy = (a.1 as i64 - b.1 as i64).unsigned_abs();
    dx.max(dy) as u32
}

// ═══════════════════════════════════════════════════════════════════════════
// A1 — inventories EMPTY at tick 0 (load-bearing baseline).
// Type A: exact == 0 (sum and per-agent max), measured BEFORE any run_ticks.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a1_inventories_empty_at_tick_zero() {
    let e = make_stage1_engine(42, 20);
    let mut sum = 0u32;
    let mut per_agent_max = 0u32;
    let mut count = 0u32;
    for (_, inv) in e.world.query::<&Inventory>().iter() {
        sum += inv.total();
        per_agent_max = per_agent_max.max(inv.total());
        count += 1;
    }
    assert_eq!(count, 20, "A1: all 20 agents must carry an Inventory at spawn");
    assert_eq!(sum, 0, "A1: sum of all inventory totals at tick 0 must be 0");
    assert_eq!(per_agent_max, 0, "A1: per-agent max inventory total at tick 0 must be 0");
    println!("[gather A1] tick-0 inventories empty (sum=0, max=0, agents=20) ✓");
}

// ═══════════════════════════════════════════════════════════════════════════
// A2 — Gather pickup strictly increases Food from the empty baseline.
// Type C: >= 5 agents have Food that rose from 0 to >= 1.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a2_pickup_increases_food_nonvacuous() {
    let b = bulk_run(42, 20, YEAR_TICKS);
    let mut risen = 0u32;
    let mut total_live = 0u32;
    for (_, inv) in b.engine.world.query::<&Inventory>().iter() {
        total_live += 1;
        if inv.get(ResourceKind::Food) >= 1 {
            risen += 1;
        }
    }
    // Observed seed-42 count (recorded for the Drafter's 1.5x-margin gate).
    println!(
        "[gather A2] {risen} of {total_live} live agents carry Food >= 1 after {YEAR_TICKS} ticks \
         (floor 5; observed must be >= 8 for 1.5x margin)"
    );
    assert!(
        risen >= 5,
        "A2: at least 5 agents must have Food rise from 0 to >= 1; got {risen}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A3 — per-agent inventory never exceeds capacity.
// Type A: total() <= INVENTORY_CAPACITY for every live agent.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a3_never_exceeds_capacity() {
    let b = bulk_run(42, 20, YEAR_TICKS);
    let mut max_total = 0u32;
    for (_, inv) in b.engine.world.query::<&Inventory>().iter() {
        max_total = max_total.max(inv.total());
        assert!(
            inv.total() <= INVENTORY_CAPACITY,
            "A3: inventory total {} must be <= INVENTORY_CAPACITY {}",
            inv.total(),
            INVENTORY_CAPACITY
        );
    }
    println!("[gather A3] max inventory total {max_total} <= cap {INVENTORY_CAPACITY} ✓");
}

// ═══════════════════════════════════════════════════════════════════════════
// A4 — inventories hold ONLY Food this slice.
// Type A: Water + Wood + Stone sum across all agents == 0.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a4_food_only() {
    let b = bulk_run(42, 20, YEAR_TICKS);
    let mut non_food = 0u32;
    for (_, inv) in b.engine.world.query::<&Inventory>().iter() {
        non_food += inv.get(ResourceKind::Water)
            + inv.get(ResourceKind::Wood)
            + inv.get(ResourceKind::Stone);
    }
    assert_eq!(non_food, 0, "A4: only Food may be carried this slice; non-Food sum {non_food}");
    println!("[gather A4] non-Food carried sum == 0 ✓");
}

// ═══════════════════════════════════════════════════════════════════════════
// A5 — finite tile depletes by exactly the amount picked up (single-agent).
// Type A: pickup completed; tile decrement == Food gain == min(C, room).
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a5_finite_conservation_single_agent() {
    let c: u8 = 5; // finite count, < room (CAP=10) → agent takes all 5, tile removed.
    let mut e = fixture_engine();
    let (ent, _) = spawn_gatherer(&mut e, 12, 12, 0);
    e.resources.set_food_tile(12, 12, c);
    let tile_before = e.resources.food_tiles.get(&(12, 12)).copied().unwrap_or(0) as u32;
    let food_before = agent_food(&e, ent);

    let mut completed = false;
    for _ in 0..300 {
        e.tick();
        if agent_food(&e, ent) > food_before
            && !matches!(
                agent_state(&e, ent),
                AgentState::Consuming { target: TargetKind::GatherFood }
            )
        {
            completed = true;
            break;
        }
    }
    assert!(completed, "A5: the single-agent pickup must complete within 300 ticks");

    let gain = agent_food(&e, ent) - food_before;
    let tile_after = e.resources.food_tiles.get(&(12, 12)).copied().unwrap_or(0) as u32;
    let room = INVENTORY_CAPACITY - food_before;
    let expected = tile_before.min(room);
    assert_eq!(gain, expected, "A5: Food gain must equal min(C, room) = {expected}");
    assert_eq!(
        tile_before - tile_after,
        expected,
        "A5: tile decrement must equal the Food gain (conservation)"
    );
    assert!(
        !e.resources.food_tiles.contains_key(&(12, 12)),
        "A5: a tile drained to 0 must be removed from food_tiles (iff count hit 0)"
    );
    println!("[gather A5] finite conservation: gain={gain}, tile {tile_before}->{tile_after} ✓");
}

// ═══════════════════════════════════════════════════════════════════════════
// A6 — positive infinite-source pickup WITHOUT sentinel corruption.
// Type A: controlled fixture — regen DISABLED (no source-max registered), one
// 255 food tile under one satisfied agent; pickup completes, Food gain >= 1, and
// the tile stays exactly 255 at every sampled tick + span end.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a6_infinite_source_no_sentinel_corruption() {
    let tile = (10u32, 10u32);
    let mut e = fixture_engine();
    let (ent, _) = spawn_gatherer(&mut e, tile.0, tile.1, 0);
    e.resources.set_food_tile(tile.0, tile.1, RESOURCE_SOURCE_INFINITE);
    // Regen DISABLED: ResourceRegenSystem only acts on tiles registered in
    // `food_source_max`. None is registered here, so it is a pure no-op AND it
    // skips the 255 sentinel anyway — nothing can heal a mid-span corruption.
    assert!(
        e.resources.food_source_max.is_empty(),
        "A6 precondition: no regen ceiling registered → regen cannot heal corruption"
    );

    let food_before = agent_food(&e, ent);
    let span = 300u32; // bounded: pickup completes early, sampling continues to end.
    let mut completed = false;
    let mut sentinel_intact_every_tick = true;
    for _ in 1..=span {
        e.tick();
        // (c) sample the 255 key EVERY tick (covers all mid-span checkpoints + end).
        if e.resources.food_tiles.get(&tile).copied() != Some(RESOURCE_SOURCE_INFINITE) {
            sentinel_intact_every_tick = false;
        }
        // (a) the infinite-take branch actually executed (pickup completed).
        if !completed
            && agent_food(&e, ent) > food_before
            && !matches!(
                agent_state(&e, ent),
                AgentState::Consuming { target: TargetKind::GatherFood }
            )
        {
            completed = true;
        }
    }
    assert!(
        completed,
        "A6(a): a completed pickup against the 255 tile must occur within the span"
    );
    assert!(
        agent_food(&e, ent) >= 1,
        "A6(b): agent Food gain must be >= 1 on the infinite branch; got {}",
        agent_food(&e, ent)
    );
    assert!(
        sentinel_intact_every_tick,
        "A6(c): the 255 tile must remain exactly 255 at every sampled tick (no corruption)"
    );
    assert_eq!(
        e.resources.food_tiles.get(&tile).copied(),
        Some(RESOURCE_SOURCE_INFINITE),
        "A6(c): the infinite tile must remain present and == 255 at span end"
    );
    println!(
        "[gather A6] infinite pickup completed, Food={}, 255 tile intact across span ✓",
        agent_food(&e, ent)
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A7 — capacity-boundary conservation (room < tile amount; overflow-return path).
// Type A: gain == R, tile decrement == R, tile retains C-R; R==0 → 0, no re-seek.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a7_capacity_boundary_conservation() {
    // R == 2 (room remaining): preload CAP-2, finite tile count C=5 > R.
    let r: u32 = 2;
    let c: u8 = 5;
    {
        let mut e = fixture_engine();
        let (ent, _) = spawn_gatherer(&mut e, 10, 10, INVENTORY_CAPACITY - r);
        e.resources.set_food_tile(10, 10, c);
        let tile_before = e.resources.food_tiles.get(&(10, 10)).copied().unwrap_or(0);
        let food_before = agent_food(&e, ent);

        let mut completed = false;
        for _ in 0..300 {
            e.tick();
            if agent_food(&e, ent) > food_before
                && !matches!(
                    agent_state(&e, ent),
                    AgentState::Consuming { target: TargetKind::GatherFood }
                )
            {
                completed = true;
                break;
            }
        }
        assert!(completed, "A7: the R=2 pickup must complete within 300 ticks");

        let gain = agent_food(&e, ent) - food_before;
        let tile_after = e.resources.food_tiles.get(&(10, 10)).copied().unwrap_or(0);
        assert_eq!(gain, r, "A7: Food gain must equal remaining room R={r} (NOT C={c})");
        assert_eq!(
            tile_before - tile_after,
            r as u8,
            "A7: tile decrement must equal R={r} (NOT the attempted C={c})"
        );
        assert_eq!(
            tile_after,
            c - r as u8,
            "A7: the tile must retain C-R={} (no invisible food destruction)",
            c - r as u8
        );
        assert_eq!(
            agent_food(&e, ent),
            INVENTORY_CAPACITY,
            "A7: inventory must be filled to exactly capacity, never beyond"
        );
        println!("[gather A7] R=2: gain={gain}, tile {tile_before}->{tile_after} (retains 3) ✓");
    }

    // R == 0 (full inventory): pickup takes 0, tile untouched, no re-seek loop.
    {
        let mut e = fixture_engine();
        let (ent, _) = spawn_gatherer(&mut e, 10, 10, INVENTORY_CAPACITY);
        e.resources.set_food_tile(10, 10, c);
        let mut ever_gather_seek = false;
        for _ in 0..300 {
            e.tick();
            if matches!(
                agent_state(&e, ent),
                AgentState::Seeking { target: TargetKind::GatherFood }
                    | AgentState::Consuming { target: TargetKind::GatherFood }
            ) {
                ever_gather_seek = true;
            }
        }
        assert_eq!(
            agent_food(&e, ent),
            INVENTORY_CAPACITY,
            "A7(R=0): a full inventory takes 0 — total unchanged"
        );
        assert_eq!(
            e.resources.food_tiles.get(&(10, 10)).copied(),
            Some(c),
            "A7(R=0): an untouched tile keeps its full count (no phantom removal)"
        );
        assert!(
            !ever_gather_seek,
            "A7(R=0): a full agent must NOT enter the gather FSM (no re-seek loop)"
        );
        assert_eq!(
            agent_state(&e, ent),
            AgentState::Idle,
            "A7(R=0): a full agent stays Idle"
        );
        println!("[gather A7] R=0: took 0, tile intact, no re-seek, Idle ✓");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// A8 — concurrent pickup from one finite tile conserves food (race fixture).
// Type A: sum gain == C - tile_after; total removed <= C; no underflow.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a8_concurrent_conservation() {
    let c: u8 = 6; // C < combined demand (2 agents x 10 room).
    let mut e = fixture_engine();
    let (a, _) = spawn_gatherer(&mut e, 14, 14, 0);
    let (b, _) = spawn_gatherer(&mut e, 14, 14, 0);
    e.resources.set_food_tile(14, 14, c);
    let c_total = c as u32;

    for _ in 0..400 {
        e.tick();
        let settled = !matches!(
            agent_state(&e, a),
            AgentState::Seeking { target: TargetKind::GatherFood }
                | AgentState::Consuming { target: TargetKind::GatherFood }
        ) && !matches!(
            agent_state(&e, b),
            AgentState::Seeking { target: TargetKind::GatherFood }
                | AgentState::Consuming { target: TargetKind::GatherFood }
        );
        // Stop once the tile is exhausted AND both agents have left gather states.
        if settled && !e.resources.food_tiles.contains_key(&(14, 14)) {
            break;
        }
    }

    let gain_a = agent_food(&e, a);
    let gain_b = agent_food(&e, b);
    let tile_after = e.resources.food_tiles.get(&(14, 14)).copied().unwrap_or(0) as u32;
    let total_removed = gain_a + gain_b;
    assert_eq!(
        total_removed,
        c_total - tile_after,
        "A8: total Food gained must equal C - tile_after (conservation)"
    );
    assert!(
        total_removed <= c_total,
        "A8: total removed {total_removed} must not exceed C={c_total} (no double-spend)"
    );
    // tile_after is a u32 read of a u8 map (>=0 by type) — underflow would have
    // panicked on the saturating subtraction in production; assert it stayed in range.
    assert!(tile_after <= c_total, "A8: tile count must never go below 0 / above C");
    assert_eq!(
        agent_state(&e, a),
        AgentState::Idle,
        "A8: agent A exits to Idle cleanly"
    );
    assert_eq!(
        agent_state(&e, b),
        AgentState::Idle,
        "A8: agent B exits to Idle cleanly"
    );
    println!("[gather A8] concurrent: gain {gain_a}+{gain_b}={total_removed} == C-after {} ✓", c_total - tile_after);
}

// ═══════════════════════════════════════════════════════════════════════════
// A9 — gathering agent physically MOVES toward its target (teleport guard).
// Type A: distance strictly decreases over Seeking; pickup pos != start pos.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a9_agent_moves_to_target() {
    let start = (10u32, 10u32);
    let target = (10u32, 16u32); // chebyshev 6 (within radius 8), NOT adjacent.
    assert!(
        chebyshev(start, target) <= GATHER_SCAN_RADIUS && chebyshev(start, target) > 0,
        "A9 precondition: target within radius and not co-located"
    );
    let mut e = fixture_engine();
    let (ent, _) = spawn_gatherer(&mut e, start.0, start.1, 0);
    e.resources.set_food_tile(target.0, target.1, 50);

    let mut seeking_dists: Vec<u32> = Vec::new();
    let mut pickup_pos: Option<(u32, u32)> = None;
    let mut prev_food = 0u32;
    let mut prev_pos = start;
    let mut prev_state = AgentState::Idle;
    for _ in 0..300 {
        e.tick();
        let st = agent_state(&e, ent);
        let pos = agent_pos(&e, ent);
        let food = agent_food(&e, ent);
        if matches!(st, AgentState::Seeking { target: TargetKind::GatherFood }) {
            seeking_dists.push(chebyshev(pos, target));
        }
        // The pickup lands on the Consuming{GatherFood} -> (food increases) tick.
        if food > prev_food
            && matches!(prev_state, AgentState::Consuming { target: TargetKind::GatherFood })
        {
            pickup_pos = Some(prev_pos);
            break;
        }
        prev_food = food;
        prev_pos = pos;
        prev_state = st;
    }

    assert!(
        seeking_dists.len() >= 2,
        "A9: agent must spend >= 2 ticks Seeking (traveling); got {}",
        seeking_dists.len()
    );
    for w in seeking_dists.windows(2) {
        assert!(
            w[1] < w[0],
            "A9: distance to target must strictly decrease while Seeking; {:?}",
            seeking_dists
        );
    }
    let pp = pickup_pos.expect("A9: a pickup must complete");
    assert_eq!(pp, target, "A9: pickup happens adjacent (on the target tile)");
    assert_ne!(pp, start, "A9: pickup position must differ from the spawn position");
    println!("[gather A9] traveled {:?} -> picked up at {:?} ✓", seeking_dists, pp);
}

// ═══════════════════════════════════════════════════════════════════════════
// A9b (radius-confounder regression) — a CLOSER food tile OUTSIDE
// GATHER_SCAN_RADIUS must NOT mask an eligible tile INSIDE it.
//
// Agent at (10,10); decoy at (10,21) is the global Manhattan-nearest tile but
// Chebyshev 11 > radius 8 (out of range); eligible tile at (17,17) is Chebyshev
// 7 <= 8 (in range) but Manhattan-farther. The buggy shape
// `nearest_resource_tile(..).filter(in_radius)` selects the Manhattan-nearest
// decoy then rejects it → Gather never fires (None) and no SeekTarget is set.
// The fix filters to the radius set FIRST → selects (17,17), attaches it as the
// SeekTarget, and the agent walks to it and picks up — leaving the out-of-radius
// decoy untouched.
//
// Geometry is chosen so the invariant survives the single Brownian step an Idle
// agent may take before its first decision: the decoy stays out-of-radius AND
// Manhattan-nearest, and the eligible tile stays in-radius, for EVERY position
// in start±1 (the verified-below pins make this explicit).
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a9b_radius_confounder_selects_in_range_tile() {
    let start = (10u32, 10u32);
    let decoy = (10u32, 21u32); // Chebyshev 11 > 8 (out), Manhattan 11 (nearest).
    let eligible = (17u32, 17u32); // Chebyshev 7 <= 8 (in), Manhattan 14 (farther).

    let manhattan = |a: (u32, u32), b: (u32, u32)| {
        (a.0 as i64 - b.0 as i64).abs() + (a.1 as i64 - b.1 as i64).abs()
    };
    // Pin the confounder geometry across the full start±1 Brownian neighbourhood
    // so the test stays meaningful (and deterministic) if the radius constant is
    // ever retuned: decoy always OUT-of-radius and Manhattan-nearest; eligible
    // always IN-radius.
    for dx in -1i64..=1 {
        for dy in -1i64..=1 {
            let p = ((start.0 as i64 + dx) as u32, (start.1 as i64 + dy) as u32);
            assert!(
                chebyshev(p, decoy) > GATHER_SCAN_RADIUS,
                "precondition: decoy must be OUT of radius from {p:?}"
            );
            assert!(
                chebyshev(p, eligible) <= GATHER_SCAN_RADIUS,
                "precondition: eligible must be IN radius from {p:?}"
            );
            assert!(
                manhattan(p, decoy) < manhattan(p, eligible),
                "precondition: decoy must be the global Manhattan-nearest from {p:?}"
            );
        }
    }

    let mut e = fixture_engine();
    let (ent, _) = spawn_gatherer(&mut e, start.0, start.1, 0);
    e.resources.set_food_tile(decoy.0, decoy.1, 50);
    e.resources.set_food_tile(eligible.0, eligible.1, 50);

    let mut seek_target: Option<(u32, u32)> = None;
    let mut pickup_pos: Option<(u32, u32)> = None;
    let mut prev_food = 0u32;
    let mut prev_state = AgentState::Idle;
    let mut prev_pos = start;
    for _ in 0..400 {
        e.tick();
        if matches!(
            agent_state(&e, ent),
            AgentState::Seeking { target: TargetKind::GatherFood }
        ) {
            if let Ok(seek) = e.world.get::<&SeekTarget>(ent) {
                seek_target = Some(seek.tile);
            }
        }
        let food = agent_food(&e, ent);
        if food > prev_food
            && matches!(prev_state, AgentState::Consuming { target: TargetKind::GatherFood })
        {
            pickup_pos = Some(prev_pos);
            break;
        }
        prev_food = food;
        prev_state = agent_state(&e, ent);
        prev_pos = agent_pos(&e, ent);
    }

    assert_eq!(
        seek_target,
        Some(eligible),
        "radius-confounder: SeekTarget must be the in-radius tile {eligible:?}, not the decoy"
    );
    let pp = pickup_pos.expect("radius-confounder: a pickup must complete at the eligible tile");
    assert_eq!(
        pp, eligible,
        "radius-confounder: the pickup must occur at the in-radius eligible tile"
    );
    assert_eq!(
        e.resources.food_tiles.get(&decoy).copied(),
        Some(50),
        "radius-confounder: the out-of-radius decoy tile must remain untouched"
    );
    println!(
        "[gather A9b] SeekTarget={seek_target:?}, picked up at {pp:?}, decoy {decoy:?} intact ✓"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A10 — gather does NOT decrease Hunger.
// Type A: Hunger_after >= Hunger_before across the pickup tick.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a10_gather_does_not_reduce_hunger() {
    let mut e = fixture_engine();
    let ent = e.spawn_agent(10, 10);
    e.world
        .insert(
            ent,
            (
                MovementRng::new(7),
                AgentState::Idle,
                Hunger::new(10.0, 0.0), // below threshold, rate 0 → stays 10.0
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 0.0),
                Memory::new(),
                BodyHealth::new(),
                Inventory::default(),
            ),
        )
        .expect("agent");
    e.resources.set_food_tile(10, 10, 50);

    let hunger_before = e.world.get::<&Hunger>(ent).map(|h| h.value).unwrap_or(0.0);
    let food_before = agent_food(&e, ent);
    let mut hunger_after = hunger_before;
    let mut completed = false;
    for _ in 0..300 {
        e.tick();
        if agent_food(&e, ent) > food_before
            && !matches!(
                agent_state(&e, ent),
                AgentState::Consuming { target: TargetKind::GatherFood }
            )
        {
            hunger_after = e.world.get::<&Hunger>(ent).map(|h| h.value).unwrap_or(0.0);
            completed = true;
            break;
        }
    }
    assert!(completed, "A10: a gather pickup must complete within 300 ticks");
    assert!(
        hunger_after >= hunger_before,
        "A10: gather must NOT decrease Hunger; before {hunger_before}, after {hunger_after}"
    );
    println!("[gather A10] hunger {hunger_before} -> {hunger_after} (not reduced by gather) ✓");
}

// ═══════════════════════════════════════════════════════════════════════════
// A11 — Hunger arm preempts Gather (a hungry agent eats, never gathers).
// Type A: gather-state ticks while hungry == 0 AND final Hunger < initial.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a11_hunger_preempts_gather() {
    let mut e = fixture_engine();
    let ent = e.spawn_agent(10, 10);
    e.world
        .insert(
            ent,
            (
                MovementRng::new(7),
                AgentState::Idle,
                Hunger::new(60.0, 0.0), // ABOVE threshold (50)
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 0.0),
                Memory::new(),
                BodyHealth::new(),
                Inventory::default(),
            ),
        )
        .expect("agent");
    // Infinite food tile so eating works without depleting the source.
    e.resources.set_food_tile(10, 10, RESOURCE_SOURCE_INFINITE);

    let initial_hunger = 60.0f32;
    let mut gather_ticks_while_hungry = 0u64;
    for _ in 0..600 {
        e.tick();
        let hunger = e.world.get::<&Hunger>(ent).map(|h| h.value).unwrap_or(0.0);
        if hunger > 50.0
            && matches!(
                agent_state(&e, ent),
                AgentState::Seeking { target: TargetKind::GatherFood }
                    | AgentState::Consuming { target: TargetKind::GatherFood }
            )
        {
            gather_ticks_while_hungry += 1;
        }
        if hunger < 50.0 {
            break;
        }
    }
    let final_hunger = e.world.get::<&Hunger>(ent).map(|h| h.value).unwrap_or(0.0);
    assert_eq!(
        gather_ticks_while_hungry, 0,
        "A11: a hungry agent must NEVER be in a gather state; got {gather_ticks_while_hungry}"
    );
    assert!(
        final_hunger < initial_hunger,
        "A11: the agent must actually eat (final Hunger {final_hunger} < initial {initial_hunger})"
    );
    println!("[gather A11] 0 gather-ticks-while-hungry; hunger {initial_hunger} -> {final_hunger} ✓");
}

// ═══════════════════════════════════════════════════════════════════════════
// A12 — no targetless Seeking across the full span (freeze guard, primary).
// Type A: == 0 observations of Seeking without a SeekTarget.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a12_no_targetless_seeking() {
    let b = bulk_run(42, 20, YEAR_TICKS);
    assert_eq!(
        b.targetless_seeking, 0,
        "A12: a Seeking state must always carry a SeekTarget; {} targetless observations",
        b.targetless_seeking
    );
    println!("[gather A12] 0 targetless-Seeking observations over {YEAR_TICKS} ticks ✓");
}

// ═══════════════════════════════════════════════════════════════════════════
// A13 — no agent stuck in gather states beyond a bounded streak.
// Type D: max consecutive gather-state ticks <= 200.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a13_no_stuck_gather_streak() {
    let b = bulk_run(42, 20, YEAR_TICKS);
    println!(
        "[gather A13] observed max gather-state streak {} (bound 200)",
        b.max_gather_streak
    );
    assert!(
        b.max_gather_streak <= 200,
        "A13: max gather-state streak {} must be <= 200 (freeze regression)",
        b.max_gather_streak
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A14 — Consuming{GatherFood} is a single-tick unconditional exit; >= 3 cycles.
// Type C: every Consuming{GatherFood} occupancy == 1 tick; >= 3 cycles observed.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a14_consuming_single_tick_and_cycles() {
    let b = bulk_run(42, 20, YEAR_TICKS);
    println!(
        "[gather A14] complete gather cycles {} (floor 3, prefer >= 5); consuming-over-one {}",
        b.complete_cycles, b.consuming_over_one
    );
    assert_eq!(
        b.consuming_over_one, 0,
        "A14: Consuming{{GatherFood}} must occupy exactly 1 tick; {} over-one observations",
        b.consuming_over_one
    );
    assert!(
        b.complete_cycles >= 3,
        "A14: >= 3 complete gather cycles must be observed; got {}",
        b.complete_cycles
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A15 — member persistence under gathering + non-vacuity floor.
// Type A: violations == 0 AND away-member observation count >= 1.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a15_member_persistence_under_gathering() {
    // Production scene: 64 bootstrap agents + 3 startup buildings → settlements
    // form. Bootstrap seeds INFINITE food/water/sleep at the 4 map corners, so a
    // satisfied member gathers far from its settlement centre while staying a
    // member (belonging model). No finite seeding → tiles never vanish.
    let mut e = SimEngine::new(64, 64, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    for (x, y) in [(32i32, 32i32), (24, 32), (40, 32)] {
        let ok = enqueue_building_placed(&mut e.resources, x, y, 8);
        assert!(ok, "A15: startup building ({x},{y}) must enqueue");
    }
    let mut bss = BuildingStampSystem::new();
    bss.tick(&mut e.world, &mut e.resources);

    let mut ever_member: HashSet<AgentId> = HashSet::new();
    let mut away_observed: HashSet<AgentId> = HashSet::new();

    let sample_every = 20u64;
    for t in 1..=YEAR_TICKS {
        e.tick();
        if !t.is_multiple_of(sample_every) {
            continue;
        }
        // Live agent positions.
        let positions: HashMap<AgentId, (u32, u32)> = e
            .world
            .query::<(&Agent, &Position)>()
            .iter()
            .map(|(_, (a, p))| (a.id, (p.x, p.y)))
            .collect();
        for s in e.resources.settlements.values() {
            let centre = s.formation_tile;
            for mid in s.member_agents.iter().copied() {
                ever_member.insert(mid);
                if let Some(pos) = positions.get(&mid) {
                    if chebyshev(*pos, centre) > SETTLEMENT_PROXIMITY_RADIUS {
                        away_observed.insert(mid);
                    }
                }
            }
        }
    }

    // Final membership / liveness reconciliation.
    let live_ids: HashSet<AgentId> = e
        .world
        .query::<&Agent>()
        .iter()
        .map(|(_, a)| a.id)
        .collect();
    let in_any_roster: HashSet<AgentId> = e
        .resources
        .settlements
        .values()
        .flat_map(|s| s.member_agents.iter().copied())
        .collect();

    // Violation = an agent that WAS a member, is still ALIVE, yet belongs to NO
    // settlement roster (dropped for a reason other than death/migration).
    let violations: Vec<AgentId> = ever_member
        .iter()
        .copied()
        .filter(|id| live_ids.contains(id) && !in_any_roster.contains(id))
        .collect();

    println!(
        "[gather A15] ever_member={}, away_observed={}, violations={}",
        ever_member.len(),
        away_observed.len(),
        violations.len()
    );
    assert!(
        !away_observed.is_empty(),
        "A15: inconclusive — no member was ever observed gathering outside its proximity radius"
    );
    assert!(
        violations.is_empty(),
        "A15: {} alive member(s) dropped from all rosters while away — membership collapse regression",
        violations.len()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A16 — lockstep determinism.
// Type A: canonical digest byte-identical across two independent runs.
// ═══════════════════════════════════════════════════════════════════════════

/// Canonical determinism digest: (sorted agent positions, sorted per-agent Food
/// inventories, sorted `food_tiles` map). Aliased to keep clippy's
/// `type_complexity` lint quiet on the nested tuple.
type GatherDigest = (Vec<(u32, u32)>, Vec<(u64, u32)>, Vec<((u32, u32), u8)>);

#[test]
fn harness_gather_a16_lockstep_determinism() {
    fn digest(ticks: u64) -> GatherDigest {
        let mut e = make_stage1_engine(42, 20);
        for _ in 0..ticks {
            e.tick();
        }
        let mut positions: Vec<(u32, u32)> = e
            .world
            .query::<&Position>()
            .iter()
            .map(|(_, p)| (p.x, p.y))
            .collect();
        positions.sort_unstable();
        let mut foods: Vec<(u64, u32)> = e
            .world
            .query::<(&Agent, &Inventory)>()
            .iter()
            .map(|(_, (a, inv))| (a.id, inv.get(ResourceKind::Food)))
            .collect();
        foods.sort_unstable();
        let mut tiles: Vec<((u32, u32), u8)> =
            e.resources.food_tiles.iter().map(|(k, v)| (*k, *v)).collect();
        tiles.sort_unstable();
        (positions, foods, tiles)
    }
    let a = digest(2000);
    let b = digest(2000);
    assert_eq!(a.0, b.0, "A16: sorted agent positions must be byte-identical");
    assert_eq!(a.1, b.1, "A16: per-agent Food inventories must be byte-identical");
    assert_eq!(a.2, b.2, "A16: sorted food_tiles must be byte-identical");
    // Locked non-vacuity gate (Assertion 16, second clause): gathering must have
    // actually occurred in BOTH runs, otherwise determinism is proven only over
    // non-gather behaviour and says nothing about the new gather hot path.
    let food_a: u32 = a.1.iter().map(|(_, f)| *f).sum();
    let food_b: u32 = b.1.iter().map(|(_, f)| *f).sum();
    assert!(food_a > 0, "A16: run A must have gathered (total carried Food > 0)");
    assert!(food_b > 0, "A16: run B must have gathered (total carried Food > 0)");
    println!(
        "[gather A16] determinism: {} agents, {} food tiles identical ✓ (carried Food a={} b={})",
        a.1.len(),
        a.2.len(),
        food_a,
        food_b
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A17 — suppresses_movement FSM correctness.
// Type A: Seeking{GatherFood} -> true; Consuming{GatherFood} -> false.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_gather_a17_suppresses_movement_fsm() {
    assert!(
        AgentState::Seeking { target: TargetKind::GatherFood }.suppresses_movement(),
        "A17: Seeking{{GatherFood}} must suppress Brownian movement"
    );
    assert!(
        !AgentState::Consuming { target: TargetKind::GatherFood }.suppresses_movement(),
        "A17: Consuming{{GatherFood}} must NOT suppress movement (decision owns its exit)"
    );
    println!("[gather A17] suppresses_movement: Seeking=true, Consuming=false ✓");
}
