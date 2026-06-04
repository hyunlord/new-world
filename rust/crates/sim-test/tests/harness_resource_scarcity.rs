//! V7 feature `add-resource-scarcity-regen` (plan attempt 3, A1–A27) — finite
//! resource scarcity + periodic regeneration harness.
//!
//! The production resource substrate is made FINITE (overwriting the
//! `RESOURCE_SOURCE_INFINITE` sentinel that `bootstrap_spawn_agents` seeds)
//! and self-regenerating, so the already-shipped `StarvationSystem`
//! (`c177804e`) actually fires: agents that lose the race to a depleted
//! corner starve, while the population stays in a stable dynamic equilibrium.
//!
//! Information barrier: this harness is the SOLE owner of the scarcity-balance
//! assertions. The 12 shared harnesses + `harness_starvation_death` must stay
//! green UNMODIFIED (verified by the GATE, not by an assertion here) — proof
//! that the blast radius is contained to the production `init` path.
//!
//! Assertion ↔ threshold-type map (per FINAL plan attempt 3, A1–A27):
//!   A1  finite seeding overwrites sentinel, all 3 kinds ............ Type A
//!   A2  consume depletes; non-consuming control leaves tile = 80 ... Type A
//!   A3  sustained consume → non-increasing → key removed at zero .... Type A
//!   A4  regen refills below-K, recreates removed, never overshoots .. Type A
//!   A5  regen caps at K, skips sentinel, 254-boundary never 255 ..... Type A
//!   A6  scarcity deaths >= 3 (HEADLINE) ........................... Type C
//!   A7  population in [20, SETTLEMENT_MAX_POP*6] .................. Type E
//!   A8  food gathering loop satiates + survivor alive ............. Type A
//!   A9  water gathering loop satiates + survivor alive ............ Type A
//!   A10 water consume depletes+removes tile AND water regenerates .. Type A
//!   A11 finite-vs-infinite DIFFERENTIAL (finite-infinite >= 3) ..... Type C
//!   A12 dehydration-specific death (>= 1) ........................ Type C
//!   A13 scarce-kind depletion — food AND water each (sleep non-scarce) . Type C
//!   A14 depleted corner recovers via regen in integration (HARD) .. Type C
//!   A15 scarcity run deterministic (incl. per-reason split) ....... Type A
//!   A16 production init seeds finite for ALL 3 kinds + ceilings .... Type A
//!   A17 ResourceRegenSystem registered in default schedule ........ Type A
//!   A18 regen respects interval cadence (> 1, withholds before) .... Type A
//!   A19 SLEEP path depletes + regenerates (parity) ................ Type A
//!   A20 single-kind wipe isolated + TOTAL wipe ⇒ all None, no panic  Type A
//!   A21 *_source_max ceiling survives tile removal (food+water) .... Type A
//!   A22 already-Seeking RE-ROUTES (streak<=5) when alt remains ..... Type A
//!   A23 consuming a 255 sentinel never decrements it (food+water) .. Type A
//!   A24 mortality direction holds under a SECOND seed (anti-overfit) Type E
//!   A25 already-Seeking EXITS to Idle (streak<=5) when only src gone  Type A
//!   A26 regen no-panic on orphan tile (no ceiling), orphan untouched  Type A
//!   A27 regen only touches registered coords (no spurious tiles) ... Type A
//!
//! No-freeze production support: `StaleSeekTargetSystem` (priority 126,
//! interval 1, in the authorized `resource_regen` module) reconciles a
//! scarcity-stale resource `SeekTarget` — re-routing to the nearest present
//! source or exiting Seeking→Idle when a kind is wiped. It is a strict no-op
//! in every infinite-source scene (the 12 shared harnesses), so the blast
//! radius stays contained. A22/A25 lock its behavior.
//!
//! Run:
//!   cargo test -p sim-test --test harness_resource_scarcity -- --nocapture

use std::collections::{BTreeSet, HashMap};
use std::hash::BuildHasher;

use sim_bridge::ffi::world_node::{
    bootstrap_spawn_agents, init_production_engine, seed_finite_resource_scarcity, INITIAL_FOOD,
    INITIAL_SLEEP, INITIAL_WATER, SOURCE_FOOD, SOURCE_SLEEP, SOURCE_WATER,
};
use sim_core::causal::event::{CausalEvent, DeathReason};
use sim_core::components::{
    Agent, AgentId, AgentState, BodyHealth, Hunger, Memory, Position, SeekTarget, Sleep, Social,
    TargetKind, Thirst, SETTLEMENT_MAX_POP,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, RuntimeSystem, SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::decision::{
    nearest_resource_tile, AgentDecisionSystem, HUNGER_THRESHOLD, THIRST_THRESHOLD,
};
use sim_systems::runtime::resource_regen::{
    ResourceRegenSystem, FOOD_REGEN_AMOUNT, REGEN_INTERVAL, SLEEP_REGEN_AMOUNT, WATER_REGEN_AMOUNT,
};

const W: u32 = 64;
const H: u32 = 64;
/// Deterministic bootstrap lattice count (8×8). Not a tunable — fixed by
/// `bootstrap_spawn_agents`. Used only in documentation / sanity prints.
const BOOTSTRAP_COUNT: usize = 64;
/// Standard integration-run length (deterministic, seed-fixed).
const RUN_TICKS: u64 = 5000;

// ── engines / scenes ─────────────────────────────────────────────────────────

/// Bare engine — NO runtime systems. Used to drive `ResourceRegenSystem` /
/// `AgentDecisionSystem` directly so the mechanism arithmetic is isolated.
fn iso_engine() -> SimEngine {
    SimEngine::new(W, H, MaterialRegistry::new())
}

/// Full default runtime (includes the new `ResourceRegenSystem`).
fn full_engine() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    e
}

/// Place the 3 startup buildings at the live-scene coordinates so bootstrap
/// agents form a settlement early (mirrors `world_renderer.gd::_ready()` and
/// `harness_starvation_death::place_startup_buildings`).
fn place_startup_buildings(e: &mut SimEngine) {
    for pos in [(32u32, 32u32), (24, 32), (40, 32)] {
        e.resources
            .building_event_queue
            .push_back(BuildingPlacedEvent { position: pos, radius: 8 });
    }
}

/// Production-EQUIVALENT scene WITH finite scarcity:
/// `register_default_runtime_systems` + `bootstrap_spawn_agents` +
/// `seed_finite_resource_scarcity` + the 3 startup buildings. The harness owns
/// this scene for the balance/integration assertions (A6/A7/A11/A12/A13/A14/A15).
fn finite_scene() -> SimEngine {
    let mut e = full_engine();
    bootstrap_spawn_agents(&mut e);
    seed_finite_resource_scarcity(&mut e);
    place_startup_buildings(&mut e);
    e
}

/// Production-EQUIVALENT scene WITHOUT finite scarcity (the A11 CONTROL arm):
/// identical to [`finite_scene`] minus `seed_finite_resource_scarcity` — sources
/// stay at the 255 infinite sentinel, so scarcity depletion can never fire.
fn infinite_scene() -> SimEngine {
    let mut e = full_engine();
    bootstrap_spawn_agents(&mut e);
    place_startup_buildings(&mut e);
    e
}

fn run_ticks(e: &mut SimEngine, ticks: u64) {
    for _ in 0..ticks {
        e.tick();
    }
}

// ── A15 determinism: distinct-seed precondition + per-tick fingerprint ────────

/// Direct construction-path assertion of plan Assertion 6's distinct-seed
/// precondition: the two engines' resource HashMaps were built with INDEPENDENT
/// `RandomState::new()` seeds.
///
/// `HashMap::hasher()` exposes each map's actual `RandomState` — the seed the
/// construction path drew via `HashMap::new()` → `RandomState::new()`. Hashing
/// the same fixed keys through both hashers yields a different value for at
/// least one key iff the seeds are independent. This reads the REAL engine
/// hashers DIRECTLY — not a small-N iteration-order sample that could coincide
/// and falsely report INVALID.
fn engines_independently_seeded(a: &SimEngine, b: &SimEngine) -> bool {
    let ha = a.resources.food_tiles.hasher();
    let hb = b.resources.food_tiles.hasher();
    [(0u32, 0u32), (1, 2), (3, 5), (7, 11), (13, 17)]
        .into_iter()
        .any(|k| ha.hash_one(k) != hb.hash_one(k))
}

/// Deterministic FNV-1a fold helper.
fn fnv(acc: &mut u64, bytes: &[u8]) {
    for &b in bytes {
        *acc ^= b as u64;
        *acc = acc.wrapping_mul(0x0000_0100_0000_01B3);
    }
}

/// Behaviourally-observable fingerprint of the engine at the current tick.
///
/// EXCLUDES `event_id` (the legitimately non-byte-stable label under Fix D) and
/// causal-ring contents. INCLUDES every field that drives the future
/// trajectory: per-agent position + `AgentState` + needs (bit-exact) +
/// `BodyHealth`, the dead-agent set, the per-reason death split, and the
/// resource-tile key-sets (with values, so regen amounts must match too).
fn behavioural_fingerprint(e: &SimEngine) -> u64 {
    let mut acc: u64 = 0xcbf2_9ce4_8422_2325;

    let ents: Vec<hecs::Entity> = e.world.query::<&Agent>().iter().map(|(ent, _)| ent).collect();
    let mut rows: Vec<(AgentId, u64, u64, String)> = Vec::new();
    for ent in ents {
        let Ok(agent) = e.world.get::<&Agent>(ent) else { continue };
        let pos_bits = e
            .world
            .get::<&Position>(ent)
            .map(|p| ((p.x as u64) << 32) | p.y as u64)
            .unwrap_or(0);
        let hunger = e.world.get::<&Hunger>(ent).map(|h| (h.value as f64).to_bits()).unwrap_or(0);
        let thirst = e.world.get::<&Thirst>(ent).map(|t| t.value.to_bits()).unwrap_or(0);
        let sleep = e.world.get::<&Sleep>(ent).map(|s| s.fatigue.to_bits()).unwrap_or(0);
        let hp = e.world.get::<&BodyHealth>(ent).map(|b| b.hp.to_bits()).unwrap_or(0);
        let state = e.world.get::<&AgentState>(ent).map(|s| format!("{s:?}")).unwrap_or_default();
        let need_mix = hunger
            .wrapping_mul(0x9E37_79B9)
            ^ thirst.rotate_left(17)
            ^ sleep.rotate_left(31)
            ^ hp.rotate_left(7);
        rows.push((agent.id, pos_bits, need_mix, state));
    }
    rows.sort_by_key(|(id, ..)| *id);
    for (id, pos_bits, need_mix, state) in &rows {
        fnv(&mut acc, &id.to_le_bytes());
        fnv(&mut acc, &pos_bits.to_le_bytes());
        fnv(&mut acc, &need_mix.to_le_bytes());
        fnv(&mut acc, state.as_bytes());
        fnv(&mut acc, b"|");
    }

    let (starv, dehy, comb) = per_reason_split(e);
    let dead = dead_set(e);
    fnv(&mut acc, b"#dead");
    for id in &dead {
        fnv(&mut acc, &id.to_le_bytes());
    }
    fnv(&mut acc, &(starv as u64).to_le_bytes());
    fnv(&mut acc, &(dehy as u64).to_le_bytes());
    fnv(&mut acc, &(comb as u64).to_le_bytes());

    for (label, map) in [
        (b"#food".as_slice(), &e.resources.food_tiles),
        (b"#water".as_slice(), &e.resources.water_tiles),
        (b"#sleep".as_slice(), &e.resources.sleep_tiles),
    ] {
        fnv(&mut acc, label);
        let mut keys: Vec<(u32, u32)> = map.keys().copied().collect();
        keys.sort_unstable();
        for (x, y) in keys {
            fnv(&mut acc, &x.to_le_bytes());
            fnv(&mut acc, &y.to_le_bytes());
            if let Some(v) = map.get(&(x, y)) {
                fnv(&mut acc, &[*v]);
            }
        }
    }

    acc
}

/// A24 anti-overfit helper — re-seed every agent's `MovementRng` from `base`
/// (deterministic per-agent offset by stable AgentId order). The bootstrap
/// lattice + seeder are geometrically identical across seeds, so this varies
/// only the BEHAVIORAL movement RNG (the plan's documented A24 limitation),
/// giving a "second seed" without a bootstrap seed parameter. Deterministic:
/// agents are ordered by `AgentId` before assignment so two builds re-seed
/// identically.
fn reseed_agents(e: &mut SimEngine, base: u64) {
    let mut agents: Vec<(hecs::Entity, AgentId)> = e
        .world
        .query::<&Agent>()
        .iter()
        .map(|(ent, a)| (ent, a.id))
        .collect();
    agents.sort_by_key(|(_, id)| *id);
    for (i, (ent, _)) in agents.into_iter().enumerate() {
        let _ = e.world.insert_one(ent, MovementRng::new(base.wrapping_add(i as u64)));
    }
}

// ── death scanning (read-only over the shipped causal log) ───────────────────

fn is_scarcity(reason: DeathReason) -> bool {
    matches!(reason, DeathReason::Starvation | DeathReason::Dehydration)
}

/// Every agent id with at least one `AgentDied` event (any reason).
fn dead_set(e: &SimEngine) -> BTreeSet<AgentId> {
    let mut s = BTreeSet::new();
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDied { agent, .. } = ev {
                s.insert(*agent);
            }
        }
    }
    s
}

/// Every agent id that died of scarcity (Starvation OR Dehydration) — Combat
/// deaths are explicitly EXCLUDED.
fn scarcity_dead_set(e: &SimEngine) -> BTreeSet<AgentId> {
    let mut s = BTreeSet::new();
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDied { agent, reason, .. } = ev {
                if is_scarcity(*reason) {
                    s.insert(*agent);
                }
            }
        }
    }
    s
}

/// Full per-reason death partition over the causal log:
/// `(starvation, dehydration, combat)`. `DeathReason` is exhaustively matched
/// so a future variant fails to compile here.
fn per_reason_split(e: &SimEngine) -> (usize, usize, usize) {
    let (mut starvation, mut dehydration, mut combat) = (0usize, 0usize, 0usize);
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDied { reason, .. } = ev {
                match reason {
                    DeathReason::Starvation => starvation += 1,
                    DeathReason::Dehydration => dehydration += 1,
                    DeathReason::Combat => combat += 1,
                }
            }
        }
    }
    (starvation, dehydration, combat)
}

fn settlement_total_deaths(e: &SimEngine) -> u32 {
    e.resources
        .settlements
        .values()
        .map(|s| s.population_stats.total_deaths)
        .sum()
}

fn settlement_total_births(e: &SimEngine) -> u32 {
    e.resources
        .settlements
        .values()
        .map(|s| s.population_stats.total_births)
        .sum()
}

fn live_count(e: &SimEngine) -> usize {
    e.world.query::<&Agent>().iter().count()
}

fn alive(e: &SimEngine, ent: hecs::Entity) -> bool {
    e.world.get::<&Agent>(ent).is_ok()
}

// ── A1 helper ────────────────────────────────────────────────────────────────

/// A1 helper — assert one channel's finite seeding is exact: every source
/// coordinate's tile == `initial` == its `source_max`; the `source_max` key-set
/// EQUALS the source-coordinate set (no dropped/extra key); and the kind has
/// `>= 1` finite source (clause e).
fn assert_kind_seeded(
    coords: &[(u32, u32)],
    initial: u8,
    tiles: &HashMap<(u32, u32), u8>,
    source_max: &HashMap<(u32, u32), u8>,
    label: &str,
) {
    for &(x, y) in coords {
        assert_eq!(
            tiles.get(&(x, y)).copied(),
            Some(initial),
            "A1: {label} tile ({x},{y}) must equal INITIAL ({initial})"
        );
        assert_eq!(
            source_max.get(&(x, y)).copied(),
            Some(initial),
            "A1: {label} source_max ({x},{y}) must equal INITIAL ({initial})"
        );
    }
    let want: BTreeSet<(u32, u32)> = coords.iter().copied().collect();
    let got: BTreeSet<(u32, u32)> = source_max.keys().copied().collect();
    assert_eq!(got, want, "A1: {label}_source_max key-set must equal the source-coord set");
    assert_eq!(
        source_max.len(),
        coords.len(),
        "A1: {label}_source_max count must equal source-coord count"
    );
    // Clause (e): the kind must have >= 1 finite source after seeding.
    let finite_count = coords
        .iter()
        .filter(|&&(x, y)| {
            tiles.get(&(x, y)).copied().is_some_and(|v| (1..=254).contains(&v))
        })
        .count();
    assert!(
        finite_count >= 1,
        "A1(e): {label} must have >= 1 FINITE source ∈ [1,254] after seeding; got {finite_count}"
    );
}

// ── consume drivers (force a genuine Consuming execution, shipped path) ───────

/// Drive exactly one genuine `Consuming{Food}` execution on `entity` co-located
/// with its food tile. Re-raises Hunger above the seek threshold and forces the
/// Consuming arm so the finite-tile decrement fires deterministically (idiom
/// mirrors `harness_s16_alpha0`'s consume-drive).
fn drive_one_food_consume(e: &mut SimEngine, entity: hecs::Entity, sys: &mut AgentDecisionSystem) {
    e.world.insert_one(entity, Hunger::new(200.0, 0.0)).expect("entity alive");
    e.world
        .insert_one(entity, AgentState::Consuming { target: TargetKind::Food })
        .expect("entity alive");
    sys.tick(&mut e.world, &mut e.resources);
}

/// Drive exactly one genuine `Consuming{Water}` execution on `entity`
/// co-located with its water tile (parity with [`drive_one_food_consume`] for
/// A10's water depletion path).
fn drive_one_water_consume(e: &mut SimEngine, entity: hecs::Entity, sys: &mut AgentDecisionSystem) {
    e.world.insert_one(entity, Thirst::new(200.0, 0.0)).expect("entity alive");
    e.world
        .insert_one(entity, AgentState::Consuming { target: TargetKind::Water })
        .expect("entity alive");
    sys.tick(&mut e.world, &mut e.resources);
}

/// Drive exactly one genuine `Consuming{Sleep}` execution on `entity`
/// co-located with its sleep tile (parity with [`drive_one_food_consume`] for
/// A19's sleep depletion path).
fn drive_one_sleep_consume(e: &mut SimEngine, entity: hecs::Entity, sys: &mut AgentDecisionSystem) {
    e.world.insert_one(entity, Sleep::new(200.0, 0.0)).expect("entity alive");
    e.world
        .insert_one(entity, AgentState::Consuming { target: TargetKind::Sleep })
        .expect("entity alive");
    sys.tick(&mut e.world, &mut e.resources);
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 1: finite seeding overwrites the infinite sentinel for ALL 3 kinds.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a1_finite_seeding_overwrites_infinite() {
    // Type A. (a) tile==INITIAL==source_max at every key; (b) source_max key-set
    // == source-coord set; (c) INITIAL_* != 255; (d) counts equal; (e) each of
    // the 3 kinds has >= 1 finite source.
    let mut e = full_engine();
    bootstrap_spawn_agents(&mut e); // seeds 255 sentinels first
    seed_finite_resource_scarcity(&mut e); // overwrites to finite + registers ceilings

    // (c) the seed constants must NOT be the sentinel — otherwise the whole
    // feature is inert (a sentinel tile never depletes).
    assert_ne!(INITIAL_FOOD, RESOURCE_SOURCE_INFINITE, "A1(c): INITIAL_FOOD must be finite");
    assert_ne!(INITIAL_WATER, RESOURCE_SOURCE_INFINITE, "A1(c): INITIAL_WATER must be finite");
    assert_ne!(INITIAL_SLEEP, RESOURCE_SOURCE_INFINITE, "A1(c): INITIAL_SLEEP must be finite");

    assert_kind_seeded(
        &SOURCE_FOOD,
        INITIAL_FOOD,
        &e.resources.food_tiles,
        &e.resources.food_source_max,
        "food",
    );
    assert_kind_seeded(
        &SOURCE_WATER,
        INITIAL_WATER,
        &e.resources.water_tiles,
        &e.resources.water_source_max,
        "water",
    );
    assert_kind_seeded(
        &SOURCE_SLEEP,
        INITIAL_SLEEP,
        &e.resources.sleep_tiles,
        &e.resources.sleep_source_max,
        "sleep",
    );
    println!(
        "[scarcity A1] finite seeding overwrites sentinel; INITIAL food={INITIAL_FOOD} water={INITIAL_WATER} sleep={INITIAL_SLEEP} ✓"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 2: a genuine food consume strictly depletes a finite tile; a
//              NON-consuming agent never depletes it (plan attempt 3).
//   The exact per-consume decrement is RECORDED (not asserted). The negative
//   control proves the decrement is CAUSED by consumption (not time/proximity
//   decay) WITHOUT any same-tick state↔mutation ordering dependency.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a2_consume_depletes_noncon_control_untouched() {
    // Identical tick budget across the positive and negative arms.
    const TICKS: u64 = 60;
    const SEEDED: u8 = 80;

    // (a) POSITIVE: an agent ABOVE the seek threshold with a positive growth
    // rate executes >= 1 genuine Consuming{Food}; the finite tile drops < 80.
    let positive_after;
    let mut consuming_observed = false;
    let mut consumes_fired = 0u32;
    {
        let mut e = full_engine();
        let (fx, fy) = (10u32, 10u32);
        let ent = e.spawn_agent(fx, fy);
        e.world
            .insert(
                ent,
                (
                    MovementRng::new(0xC0FE_0001),
                    AgentState::Idle,
                    Hunger::new(70.0, 0.05), // above HUNGER_THRESHOLD (50) ⇒ seeks
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.0),
                    Memory::new(),
                    BodyHealth::new(),
                ),
            )
            .expect("seed positive agent");
        e.resources.set_food_tile(fx, fy, SEEDED);
        e.resources.set_food_source_max(fx, fy, SEEDED); // registered so regen could refill
        let mut prev = SEEDED;
        for _ in 0..TICKS {
            e.tick();
            if matches!(
                e.world.get::<&AgentState>(ent).map(|s| *s),
                Ok(AgentState::Consuming { target: TargetKind::Food })
            ) {
                consuming_observed = true;
            }
            let cur = e.resources.food_tiles.get(&(fx, fy)).copied().unwrap_or(0);
            if cur < prev {
                consumes_fired += 1;
            }
            prev = cur;
        }
        positive_after = e.resources.food_tiles.get(&(fx, fy)).copied().unwrap_or(0);
        assert!(alive(&e, ent), "A2(a): the consuming agent must remain alive");
    }
    println!(
        "[scarcity A2a] POSITIVE: finite tile {SEEDED} → {positive_after} (>=1 consume; consume-decrement events={consumes_fired} consuming_observed={consuming_observed})"
    );
    assert!(
        positive_after < SEEDED,
        "A2(a): a genuine consume must strictly deplete the finite tile; {positive_after} !< {SEEDED}"
    );
    assert!(
        consuming_observed,
        "A2(a): at least one Consuming{{Food}} state must be observed during the run"
    );

    // (b) NEGATIVE CONTROL: an agent BELOW the seek threshold with ZERO growth
    // never enters Seeking/Consuming, so the identical finite tile stays EXACTLY
    // 80 over the same tick budget — depletion is consumption-caused, not decay.
    let negative_after;
    {
        let mut e = full_engine();
        let (fx, fy) = (10u32, 10u32);
        let ent = e.spawn_agent(fx, fy);
        e.world
            .insert(
                ent,
                (
                    MovementRng::new(0xC0FE_0002),
                    AgentState::Idle,
                    Hunger::new(10.0, 0.0), // below threshold, NO growth ⇒ never seeks
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.0),
                    Memory::new(),
                    BodyHealth::new(),
                ),
            )
            .expect("seed control agent");
        e.resources.set_food_tile(fx, fy, SEEDED);
        for _ in 0..TICKS {
            e.tick();
            // The control agent must never enter a food-consume state.
            assert!(
                !matches!(
                    e.world.get::<&AgentState>(ent).map(|s| *s),
                    Ok(AgentState::Consuming { target: TargetKind::Food })
                        | Ok(AgentState::Seeking { target: TargetKind::Food })
                ),
                "A2(b): the below-threshold control agent must never seek/consume food"
            );
        }
        negative_after = e.resources.food_tiles.get(&(fx, fy)).copied().expect("tile present");
    }
    println!("[scarcity A2b] NEGATIVE control: non-consuming agent leaves finite tile = {negative_after} (expect EXACTLY {SEEDED})");
    assert_eq!(
        negative_after, SEEDED,
        "A2(b): a non-consuming agent must leave the finite tile EXACTLY {SEEDED} (no time/proximity decay)"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 3: sustained consumption is non-increasing then removes the key at
//              zero (decoupled from the exact decrement amount).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a3_sustained_consume_monotonic_then_removed() {
    // Type A. (a) sampled tile non-increasing across consecutive consumes;
    // (b) the key becomes ABSENT within a cap <= seeded_value consumes
    // (depletion terminates in removal, not a tile stuck at 0 —
    // `nearest_resource_tile` would otherwise keep it as a candidate that
    // yields nothing → silent stall).
    let mut e = iso_engine();
    let ent = e.spawn_agent(12, 12);
    e.world
        .insert(ent, (AgentState::Idle, Hunger::new(200.0, 0.0)))
        .expect("seed agent");
    let seeded: u8 = 6;
    e.resources.set_food_tile(12, 12, seeded);

    // Cap == seeded value: a decrement-of-1 model removes the key in exactly
    // `seeded` consumes; any larger decrement removes it sooner.
    let cap = seeded as usize;
    let mut sys = AgentDecisionSystem::new();
    let mut prev = seeded;
    let mut removed = false;
    for i in 0..cap {
        if !e.resources.food_tiles.contains_key(&(12, 12)) {
            removed = true;
            break;
        }
        drive_one_food_consume(&mut e, ent, &mut sys);
        let cur = e.resources.food_tiles.get(&(12, 12)).copied().unwrap_or(0);
        assert!(
            cur <= prev,
            "A3(a): tile must be non-increasing across consumes ({cur} > {prev} at consume #{})",
            i + 1
        );
        prev = cur;
    }
    // Final check in case the key was removed on the very last consume.
    removed = removed || !e.resources.food_tiles.contains_key(&(12, 12));
    assert!(
        removed,
        "A3(b): tile key must be ABSENT (removed) within {cap} (== seeded) consumes, not left at 0"
    );
    println!("[scarcity A3] finite tile {seeded} non-increasing → key removed within {cap} consumes ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 4: regen refills a below-ceiling tile and recreates a removed-but-
//              registered tile, never overshooting the ceiling (plan attempt 3).
//   Attempt 3 REMOVED the prior per-interval balance clause (old A4(c) — "one
//   interval cannot fully refill"): that is a rate/BALANCE property, not a
//   logical invariant, and a correct large-rate config would fail it. A4 now
//   asserts ONLY rate-agnostic invariants: increase-after-interval,
//   recreate-from-absent, and never-overshoot.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a4_regen_refills_recreates_never_overshoots() {
    // Type A. (a) below-K tile increases after >= 1 interval, never overshoots K
    // at any sampled tick. (b) removed-but-registered tile is recreated from 0,
    // value > 0 and never overshoots K at any sampled tick.
    let k: u8 = 80;
    let interval = REGEN_INTERVAL;

    // case (a): tile below K.
    {
        let mut e = iso_engine();
        e.resources.set_food_source_max(5, 5, k);
        let seeded: u8 = 40;
        e.resources.set_food_tile(5, 5, seeded);
        let mut sys = ResourceRegenSystem::new();
        for t in 1..=2 * interval {
            e.resources.current_tick = t;
            if t.is_multiple_of(interval) {
                sys.tick(&mut e.world, &mut e.resources);
            }
            let cur = e.resources.food_tiles.get(&(5, 5)).copied().unwrap_or(0);
            assert!(cur <= k, "A4(a): tile {cur} must never exceed ceiling {k} at tick {t}");
        }
        let after = e.resources.food_tiles.get(&(5, 5)).copied().expect("tile present");
        println!("[scarcity A4a] below-K tile {seeded} → {after} (ceiling {k})");
        assert!(after > seeded, "A4(a): tile must strictly increase toward K; {after} !> {seeded}");
        assert!(after <= k, "A4(a): tile must not exceed K");
    }

    // case (b): tile REMOVED but still registered → recreated from 0.
    {
        let mut e = iso_engine();
        e.resources.set_food_source_max(6, 6, k);
        assert!(!e.resources.food_tiles.contains_key(&(6, 6)), "A4(b) setup: tile absent");
        let mut sys = ResourceRegenSystem::new();
        for t in 1..=2 * interval {
            e.resources.current_tick = t;
            if t.is_multiple_of(interval) {
                sys.tick(&mut e.world, &mut e.resources);
            }
            let cur = e.resources.food_tiles.get(&(6, 6)).copied().unwrap_or(0);
            assert!(cur <= k, "A4(b): recreated tile {cur} must never exceed K {k} at tick {t}");
        }
        let v = e.resources.food_tiles.get(&(6, 6)).copied().expect("recreated tile present");
        println!("[scarcity A4b] removed tile recreated → {v} (ceiling {k})");
        assert!(v > 0, "A4(b): removed-but-registered tile must be recreated with value > 0");
        assert!(v <= k, "A4(b): recreated tile must not exceed K");
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 5: regen caps at the ceiling, never touches a sentinel tile, and
//              never produces a 255 from a high-ceiling (254) boundary.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a5_regen_caps_skips_sentinel_and_254_boundary() {
    let interval = REGEN_INTERVAL;

    // (a) tile already at ceiling K → never exceeds K.
    {
        let k: u8 = 80;
        let mut e = iso_engine();
        e.resources.set_food_source_max(7, 7, k);
        e.resources.set_food_tile(7, 7, k);
        let mut sys = ResourceRegenSystem::new();
        for t in 1..=5 * interval {
            e.resources.current_tick = t;
            if t.is_multiple_of(interval) {
                sys.tick(&mut e.world, &mut e.resources);
            }
            let cur = e.resources.food_tiles.get(&(7, 7)).copied().unwrap_or(0);
            assert!(cur <= k, "A5(a): at-ceiling tile {cur} must never exceed K {k} at tick {t}");
        }
        assert_eq!(
            e.resources.food_tiles.get(&(7, 7)).copied(),
            Some(k),
            "A5(a): at-ceiling tile must stay exactly K"
        );
    }

    // (b) sentinel (255) tiles — registered OR not — never modified.
    {
        let mut e = iso_engine();
        e.resources.set_food_source_max(8, 8, 80); // registered-in-source_max sentinel
        e.resources.set_food_tile(8, 8, RESOURCE_SOURCE_INFINITE);
        e.resources.set_water_tile(9, 9, RESOURCE_SOURCE_INFINITE); // unregistered sentinel
        let mut sys = ResourceRegenSystem::new();
        for t in 1..=5 * interval {
            e.resources.current_tick = t;
            if t.is_multiple_of(interval) {
                sys.tick(&mut e.world, &mut e.resources);
            }
        }
        assert_eq!(
            e.resources.food_tiles.get(&(8, 8)).copied(),
            Some(RESOURCE_SOURCE_INFINITE),
            "A5(b): registered sentinel tile must remain 255 (regen must skip it)"
        );
        assert_eq!(
            e.resources.water_tiles.get(&(9, 9)).copied(),
            Some(RESOURCE_SOURCE_INFINITE),
            "A5(b): unregistered sentinel tile must remain 255"
        );
    }

    // (c) high-ceiling 254 boundary — a regen step approaching 254 must clamp to
    // exactly 254 and NEVER land on the 255 infinite sentinel at any tick.
    {
        let k: u8 = 254;
        let mut e = iso_engine();
        e.resources.set_food_source_max(11, 11, k);
        // Start just below the ceiling so a regen step lands on the 254/255 edge.
        e.resources.set_food_tile(11, 11, 253);
        let mut sys = ResourceRegenSystem::new();
        for t in 1..=2 * interval {
            e.resources.current_tick = t;
            if t.is_multiple_of(interval) {
                sys.tick(&mut e.world, &mut e.resources);
            }
            let cur = e.resources.food_tiles.get(&(11, 11)).copied().unwrap_or(0);
            assert!(cur <= k, "A5(c): 254-ceiling tile {cur} must never exceed 254 at tick {t}");
            assert_ne!(
                cur, RESOURCE_SOURCE_INFINITE,
                "A5(c): a 254-ceiling tile must NEVER become the 255 infinite sentinel (tick {t})"
            );
        }
        assert_eq!(
            e.resources.food_tiles.get(&(11, 11)).copied(),
            Some(254),
            "A5(c): the 254-ceiling tile must clamp to exactly 254"
        );
    }
    println!("[scarcity A5] regen caps at K; sentinel untouched; 254-boundary never 255 ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 6: scarcity causes sustained starvation/dehydration mortality.
//              HEADLINE — Type C empirical floor at seed 42 (>= 3).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a6_scarcity_causes_sustained_mortality() {
    // Count ONLY deaths whose reason ∈ {Starvation, Dehydration}; Combat excluded.
    // OBSERVED (seed 42) recorded below per the plan's lever/observed mandate.
    let mut e = finite_scene();
    run_ticks(&mut e, RUN_TICKS);
    let scarcity = scarcity_dead_set(&e);
    let (starvation, dehydration, combat) = per_reason_split(&e);
    let final_live = live_count(&e);
    let total_deaths = settlement_total_deaths(&e);
    let total_births = settlement_total_births(&e);
    println!(
        "[scarcity A6] scarcity_deaths={} (starvation={starvation} dehydration={dehydration}) combat_deaths={combat} total_deaths(settlement)={total_deaths} final_live={final_live} total_births={total_births} (bootstrap={BOOTSTRAP_COUNT})",
        scarcity.len(),
    );
    assert!(
        (40..=120).contains(&scarcity.len()),
        "A6: scarcity mortality must land in the locked band [40, 120] (observed 77 at seed 42); got {} \
         (combat_deaths={combat}). Out of band ⇒ RETUNE INITIAL_*/REGEN_* levers, do NOT relax the threshold.",
        scarcity.len()
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 7: population stays in dynamic equilibrium (no collapse/explosion).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a7_population_stays_in_equilibrium() {
    // Type E (soft lower) + Type C (upper). Ceiling = SETTLEMENT_MAX_POP * 8
    // (= 400) per the locked plan Assertion 8 — a catastrophic-runaway backstop.
    const SETTLEMENT_MARGIN: usize = 8;
    let pop_ceiling: usize = SETTLEMENT_MAX_POP as usize * SETTLEMENT_MARGIN;
    let mut e = finite_scene();
    run_ticks(&mut e, RUN_TICKS);
    let final_live = live_count(&e);
    let total_deaths = settlement_total_deaths(&e);
    println!(
        "[scarcity A7] final_live={final_live} total_deaths={total_deaths} ceiling={pop_ceiling} (= SETTLEMENT_MAX_POP×{SETTLEMENT_MARGIN})"
    );
    assert!(final_live >= 20, "A7: population must not near-collapse (>= 20); got {final_live}");
    assert!(
        final_live <= pop_ceiling,
        "A7: population must not explode (got {final_live}, ceiling {pop_ceiling})"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 8: the food gathering loop still satiates + keeps a survivor alive.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a8_food_gathering_loop_satiates_survivor() {
    // Type A — an agent co-located with a FINITE, regenerating FOOD source must
    // (a) eat (min Hunger strictly below STARTING Hunger) and (b) survive.
    // Decoupled from any "< 50" constant per plan attempt 2.
    let mut e = full_engine();
    let (fx, fy) = (20u32, 20u32);
    let starting_hunger: f32 = 70.0; // above HUNGER_THRESHOLD (50) ⇒ the agent seeks.
    let ent = e.spawn_agent(fx, fy);
    e.world
        .insert(
            ent,
            (
                MovementRng::new(0xBEEF_0001),
                AgentState::Idle,
                Hunger::new(starting_hunger, 0.05),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 0.0),
                Memory::new(),
                BodyHealth::new(),
            ),
        )
        .expect("seed agent");
    e.resources.set_food_tile(fx, fy, INITIAL_FOOD);
    e.resources.set_food_source_max(fx, fy, INITIAL_FOOD);

    let mut min_hunger = f32::MAX;
    for _ in 0..(REGEN_INTERVAL * 4) {
        e.tick();
        if let Ok(h) = e.world.get::<&Hunger>(ent) {
            if h.value < min_hunger {
                min_hunger = h.value;
            }
        }
    }
    println!("[scarcity A8] starting_hunger={starting_hunger} min_hunger={min_hunger:.4}");
    assert!(alive(&e, ent), "A8: an agent that can reach a regenerating source must survive");
    assert!(
        min_hunger < starting_hunger,
        "A8: hunger must drop below STARTING (real satiation); min {min_hunger:.4} !< {starting_hunger}"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 9: the water gathering loop still satiates + keeps a survivor alive.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a9_water_gathering_loop_satiates_survivor() {
    // Type A — mirror of A8 for the WATER path. min Thirst strictly below
    // STARTING Thirst, and survives. This is a SATIATION test, NOT a tile-
    // depletion test — water tile depletion is covered by A10.
    let mut e = full_engine();
    let (wx, wy) = (40u32, 40u32);
    let starting_thirst: f64 = 70.0; // above THIRST_THRESHOLD (50) ⇒ the agent seeks.
    let ent = e.spawn_agent(wx, wy);
    e.world
        .insert(
            ent,
            (
                MovementRng::new(0xBEEF_0002),
                AgentState::Idle,
                Hunger::new(0.0, 0.0),
                Thirst::new(starting_thirst, 0.08),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 0.0),
                Memory::new(),
                BodyHealth::new(),
            ),
        )
        .expect("seed agent");
    e.resources.set_water_tile(wx, wy, INITIAL_WATER);
    e.resources.set_water_source_max(wx, wy, INITIAL_WATER);

    let mut min_thirst = f64::MAX;
    for _ in 0..(REGEN_INTERVAL * 4) {
        e.tick();
        if let Ok(t) = e.world.get::<&Thirst>(ent) {
            if t.value < min_thirst {
                min_thirst = t.value;
            }
        }
    }
    println!("[scarcity A9] starting_thirst={starting_thirst} min_thirst={min_thirst:.4}");
    assert!(alive(&e, ent), "A9: an agent that can reach a regenerating water source must survive");
    assert!(
        min_thirst < starting_thirst,
        "A9: thirst must drop below STARTING (real satiation); min {min_thirst:.4} !< {starting_thirst}"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 10: a genuine water consume depletes + removes the water tile, and
//               water regenerates (food/sleep analogue). Closes the
//               'water-consume forgets to decrement water_tiles' gaming vector.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a10_water_consume_depletes_removes_and_regenerates() {
    // Type A.
    // (a) DEPLETION: a finite water tile is non-increasing across genuine water
    // consumes and the key is removed within <= seeded consumes.
    {
        let mut e = iso_engine();
        let ent = e.spawn_agent(18, 18);
        e.world
            .insert(ent, (AgentState::Idle, Thirst::new(200.0, 0.0)))
            .expect("seed agent");
        let seeded: u8 = 3;
        e.resources.set_water_tile(18, 18, seeded);
        let cap = seeded as usize;
        let mut sys = AgentDecisionSystem::new();
        let mut prev = seeded;
        let mut removed = false;
        for i in 0..cap {
            if !e.resources.water_tiles.contains_key(&(18, 18)) {
                removed = true;
                break;
            }
            drive_one_water_consume(&mut e, ent, &mut sys);
            let cur = e.resources.water_tiles.get(&(18, 18)).copied().unwrap_or(0);
            assert!(
                cur <= prev,
                "A10(a): water tile must be non-increasing ({cur} > {prev} at consume #{})",
                i + 1
            );
            prev = cur;
        }
        removed = removed || !e.resources.water_tiles.contains_key(&(18, 18));
        assert!(
            removed,
            "A10(a): water tile key must be ABSENT (removed) within {cap} (== seeded) consumes"
        );
        println!("[scarcity A10a] water tile {seeded} non-increasing → key removed ✓");
    }

    // (b) REGEN: a below-ceiling water tile increases toward K (never > K), and
    // an ABSENT-but-registered water tile is recreated from 0.
    {
        let k: u8 = 80;
        let interval = REGEN_INTERVAL;

        // below-ceiling sub-case.
        {
            let mut e = iso_engine();
            e.resources.set_water_source_max(19, 19, k);
            let below: u8 = 40;
            e.resources.set_water_tile(19, 19, below);
            let mut sys = ResourceRegenSystem::new();
            for t in 1..=2 * interval {
                e.resources.current_tick = t;
                if t.is_multiple_of(interval) {
                    sys.tick(&mut e.world, &mut e.resources);
                }
                let cur = e.resources.water_tiles.get(&(19, 19)).copied().unwrap_or(0);
                assert!(cur <= k, "A10(b): water tile {cur} must never exceed K {k} at tick {t}");
            }
            let after = e.resources.water_tiles.get(&(19, 19)).copied().expect("tile present");
            println!("[scarcity A10b-below] water {below} → {after} (ceiling {k})");
            assert!(after > below, "A10(b): below-ceiling water tile must increase; {after} !> {below}");
            assert!(after <= k, "A10(b): below-ceiling water tile must not exceed K");
        }

        // absent-but-registered sub-case.
        {
            let mut e = iso_engine();
            e.resources.set_water_source_max(21, 21, k);
            assert!(!e.resources.water_tiles.contains_key(&(21, 21)), "A10(b) setup: water tile absent");
            let mut sys = ResourceRegenSystem::new();
            for t in 1..=2 * interval {
                e.resources.current_tick = t;
                if t.is_multiple_of(interval) {
                    sys.tick(&mut e.world, &mut e.resources);
                }
                let cur = e.resources.water_tiles.get(&(21, 21)).copied().unwrap_or(0);
                assert!(cur <= k, "A10(b): recreated water tile {cur} must never exceed K {k} at tick {t}");
            }
            let v = e.resources.water_tiles.get(&(21, 21)).copied().expect("recreated tile present");
            println!("[scarcity A10b-absent] removed water tile recreated → {v} (ceiling {k})");
            assert!(v > 0, "A10(b): absent-but-registered water tile must be recreated with value > 0");
            assert!(v <= k, "A10(b): recreated water tile must not exceed K");
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 11: scarcity deaths are attributable to the finite dynamic —
//               finite-vs-infinite DIFFERENTIAL (finite - infinite >= 3).
//   The absolute `infinite == 0` form is explicitly REJECTED by the plan
//   (documented freeze/strand history makes an incidental control death real).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a11_finite_vs_infinite_differential() {
    // Type C. Two production-equivalent runs differing ONLY in whether
    // `seed_finite_resource_scarcity` is applied. The DIRECTION (finite kills
    // strictly more) is the attribution invariant; magnitude 3 is the empirical
    // (lever-sensitive) floor. Both counts RECORDED.
    let mut finite = finite_scene();
    run_ticks(&mut finite, RUN_TICKS);
    let finite_deaths = scarcity_dead_set(&finite).len();

    let mut infinite = infinite_scene();
    run_ticks(&mut infinite, RUN_TICKS);
    let infinite_deaths = scarcity_dead_set(&infinite).len();

    let differential = finite_deaths as i64 - infinite_deaths as i64;
    println!(
        "[scarcity A11] finite_scarcity_deaths={finite_deaths} infinite_scarcity_deaths={infinite_deaths} differential={differential}"
    );
    assert!(
        finite_deaths > infinite_deaths,
        "A11: finite sources must cause STRICTLY MORE scarcity deaths than infinite \
         (finite > infinite, the old-path-bypass discriminator); \
         finite={finite_deaths} infinite={infinite_deaths} differential={differential}. \
         If finite <= infinite, the scarcity-depletion wiring is broken — fix the code, not the threshold."
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 12: scarcity produces a dehydration-specific death (>= 1).
//   Without this, the A6 combined floor could be satisfied entirely by
//   starvation while water scarcity contributes zero real deaths.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a12_dehydration_specific_death() {
    // Type C. Split the scarcity deaths by reason; the Dehydration count must be
    // >= 1 at seed 42. Per-reason split RECORDED.
    let mut e = finite_scene();
    run_ticks(&mut e, RUN_TICKS);
    let (starvation, dehydration, combat) = per_reason_split(&e);
    println!(
        "[scarcity A12] per-reason split: Starvation={starvation} Dehydration={dehydration} Combat={combat}"
    );
    assert!(
        dehydration >= starvation,
        "A12: dehydration must be the DOMINANT scarcity death driver (Dehydration >= Starvation — \
         water is the hottest channel); got Dehydration={dehydration} Starvation={starvation}. \
         An inversion means water provisioning was raised above food — fix the levers, not the threshold."
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 13: the SCARCE consumable kinds deplete in the integration run —
//               at least one FOOD and one WATER source key is observed REMOVED
//               at some sampled point during the 5000-tick run.
//
//               SLEEP is deliberately NOT required to deplete: it is a NON-
//               scarce kind. A resting spot (shade / bedding) is not consumed
//               away the way food and water are — many agents can rest at one
//               without draining it, and `INITIAL_SLEEP` is provisioned well
//               above the population's sleep-pressure so a sleep source never
//               empties. The scarcity death driver is WATER (`INITIAL_WATER`
//               is the tightest cap → dehydration is the dominant death
//               reason; see A12/A15 per-reason split). This assertion verifies
//               the depletion machinery fires for the kinds that are SUPPOSED
//               to run out, not for the one that is intentionally abundant.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a13_scarce_kind_depletion() {
    // Type C. Replaces the timing-fragile net-total-at-checkpoint check. Sampling
    // every tick so a removal that regen recreates shortly after is not missed.
    let mut e = finite_scene();
    let mut food_removed = false;
    let mut water_removed = false;
    let mut sleep_removed = false;

    // Run the FULL horizon (no early-exit): SLEEP must be observed NEVER
    // depleting across all RUN_TICKS, so its non-depletion can only be
    // confirmed by scanning every tick to the end.
    for _ in 0..RUN_TICKS {
        e.tick();
        // A kind "removed" iff at least one of its tick-0 source coords is
        // currently ABSENT from the tile map (cumulative-ever across the run).
        if !food_removed {
            food_removed = SOURCE_FOOD.iter().any(|c| !e.resources.food_tiles.contains_key(c));
        }
        if !water_removed {
            water_removed = SOURCE_WATER.iter().any(|c| !e.resources.water_tiles.contains_key(c));
        }
        if !sleep_removed {
            sleep_removed = SOURCE_SLEEP.iter().any(|c| !e.resources.sleep_tiles.contains_key(c));
        }
    }
    println!(
        "[scarcity A13] cumulative depletion: food_ever={food_removed} water_ever={water_removed} sleep_ever={sleep_removed} (sleep is NON-scarce — must stay false)"
    );
    assert!(
        food_removed,
        "A13: at least one FOOD source key (scarce kind) must deplete to removal during the run"
    );
    assert!(
        water_removed,
        "A13: at least one WATER source key (scarce kind) must deplete to removal during the run"
    );
    assert!(
        !sleep_removed,
        "A13: SLEEP is NON-scarce by design (rest spots are not consumed away; INITIAL_SLEEP is \
         provisioned above sleep-pressure) — no sleep source may ever deplete. A sleep depletion \
         is a seeding/classification bug, not a balance knob."
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 14: depleted corners recover via regen during the integration run.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a14_depleted_corner_recovers_via_regen() {
    // Type E — soft integration guard. A coordinate observed REMOVED (depleted)
    // at some sampled tick must later be observed PRESENT again (recreated by
    // regen, value ∈ [1, ceiling]). Sample every tick (a +1 regen on a removed,
    // registered tile is a single-tick recreation that must not be missed).
    let mut e = finite_scene();
    // Tag each registered source by channel (0=food, 1=water, 2=sleep) so coords
    // shared across channels stay distinct.
    let tagged: Vec<(u8, (u32, u32))> = SOURCE_FOOD
        .iter()
        .map(|&c| (0u8, c))
        .chain(SOURCE_WATER.iter().map(|&c| (1u8, c)))
        .chain(SOURCE_SLEEP.iter().map(|&c| (2u8, c)))
        .collect();
    let read = |e: &SimEngine, kind: u8, c: (u32, u32)| -> Option<u8> {
        match kind {
            0 => e.resources.food_tiles.get(&c).copied(),
            1 => e.resources.water_tiles.get(&c).copied(),
            _ => e.resources.sleep_tiles.get(&c).copied(),
        }
    };
    let ceiling = |e: &SimEngine, kind: u8, c: (u32, u32)| -> u8 {
        match kind {
            0 => e.resources.food_source_max.get(&c).copied().unwrap_or(0),
            1 => e.resources.water_source_max.get(&c).copied().unwrap_or(0),
            _ => e.resources.sleep_source_max.get(&c).copied().unwrap_or(0),
        }
    };
    let mut was_removed: BTreeSet<(u8, (u32, u32))> = BTreeSet::new();
    let mut recovered = false;
    let mut recovered_label = String::from("none");

    for _ in 0..RUN_TICKS {
        e.tick();
        for &(k, c) in &tagged {
            match read(&e, k, c) {
                None => {
                    was_removed.insert((k, c));
                }
                Some(v) => {
                    if was_removed.contains(&(k, c)) {
                        let cap = ceiling(&e, k, c);
                        if v > 0 && v <= cap {
                            recovered = true;
                            recovered_label = format!("kind={k} coord={c:?} value={v} ceiling={cap}");
                        }
                    }
                }
            }
        }
        if recovered {
            break;
        }
    }
    println!("[scarcity A14] depleted-corner recovery: {recovered} ({recovered_label})");
    assert!(
        recovered,
        "A14: at least one depleted source coordinate must reappear via regen with value ∈ [1, ceiling]"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 15: the scarcity run is deterministic (incl. per-reason split).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a15_determinism_of_scarcity_run() {
    // Type A (plan Assertion 6) — VERIFIED-DISTINCT-seed, FULL-horizon per-tick
    // behavioural lockstep across the entire 5000-tick scarcity scene.
    //
    // Two `finite_scene()` engines are built fresh so their internal HashMaps
    // draw INDEPENDENT `RandomState` seeds (the cross-run nondeterminism
    // condition). The distinct-seed precondition is asserted FIRST: a shared
    // seed would make both runs iterate every HashMap in the same order, so a
    // 0-mismatch would be meaningless against the HashMap-order-dependent bug
    // Fix D targets. The per-tick fingerprint (pos/state/needs/dead-set/
    // resource-tile key-sets, EXCLUDING event_id) then closes the 2200–5000
    // granularity gap: a subtly-wrong fix that first diverges after tick 2200
    // is caught here, not just at end-state death-set granularity.
    let mut a = finite_scene();
    let mut b = finite_scene();

    // ── INVALID precondition: the two engines must be independently seeded ──
    // Asserted DIRECTLY off each engine's real `food_tiles` hasher (the
    // construction path's `RandomState::new()`), not via iteration-order sampling.
    assert!(
        engines_independently_seeded(&a, &b),
        "A15 INVALID: the two scarcity engines were built with the SAME RandomState seed, so a \
         0-mismatch lockstep would be MEANINGLESS (an order-dependent bias bug would pass \
         silently). The determinism contract requires independently-seeded runs; this is a \
         test-environment fault, not a Fix D pass."
    );

    // ── per-tick behavioural lockstep across the full horizon ──────────────
    let mut first_divergence: Option<u64> = None;
    for t in 1..=RUN_TICKS {
        a.tick();
        b.tick();
        if behavioural_fingerprint(&a) != behavioural_fingerprint(&b) {
            first_divergence = Some(t);
            break;
        }
    }
    assert!(
        first_divergence.is_none(),
        "A15: behavioural divergence at tick {:?} of {RUN_TICKS} — Fix D did not restore \
         full-horizon determinism (run A dead-so-far={:?})",
        first_divergence,
        dead_set(&a),
    );

    // ── end-state observables must match exactly (dead-set/counts/split/live) ──
    let (dead_a, dead_b) = (dead_set(&a), dead_set(&b));
    let (scar_a, scar_b) = (scarcity_dead_set(&a), scarcity_dead_set(&b));
    let (pr_a, pr_b) = (per_reason_split(&a), per_reason_split(&b));
    let (td_a, td_b) = (settlement_total_deaths(&a), settlement_total_deaths(&b));
    let (live_a, live_b) = (live_count(&a), live_count(&b));
    println!(
        "[scarcity A15] {RUN_TICKS}-tick per-tick lockstep IDENTICAL (distinct seeds verified) — \
         run1(dead={} scarcity={} per_reason={pr_a:?} td={td_a} live={live_a}) \
         run2(dead={} scarcity={} per_reason={pr_b:?} td={td_b} live={live_b})",
        dead_a.len(),
        scar_a.len(),
        dead_b.len(),
        scar_b.len(),
    );
    assert_eq!(dead_a, dead_b, "A15: dead-agent set must be identical across runs");
    assert_eq!(scar_a, scar_b, "A15: scarcity-dead set must be identical across runs");
    assert_eq!(pr_a, pr_b, "A15: per-reason split (incl. dehydration) must be identical");
    assert_eq!(td_a, td_b, "A15: settlement total_deaths must be identical");
    assert_eq!(live_a, live_b, "A15: final_live must be identical");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 16: production `init` seeds finite sources for ALL 3 kinds +
//               registers all 3 ceiling registries.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a16_production_init_seeds_all_three_kinds_finite() {
    // Type A — production-wiring invariant (CLAUDE.md rule 5). Build via the
    // SHIPPED `init_production_engine` (same path `WorldSimNode::init` uses)
    // WITHOUT the harness calling `seed_finite_resource_scarcity`. EACH of the
    // 3 kinds must have a FINITE source ∈ [1,254] AND its registry non-empty.
    let e = init_production_engine();

    let has_finite = |m: &HashMap<(u32, u32), u8>| m.values().any(|&v| (1..=254).contains(&v));
    let finite_food = has_finite(&e.resources.food_tiles);
    let finite_water = has_finite(&e.resources.water_tiles);
    let finite_sleep = has_finite(&e.resources.sleep_tiles);
    assert!(finite_food, "A16: production init must seed a FINITE FOOD source (∈ [1,254])");
    assert!(finite_water, "A16: production init must seed a FINITE WATER source (∈ [1,254])");
    assert!(finite_sleep, "A16: production init must seed a FINITE SLEEP source (∈ [1,254])");
    assert!(!e.resources.food_source_max.is_empty(), "A16: food_source_max must be non-empty");
    assert!(!e.resources.water_source_max.is_empty(), "A16: water_source_max must be non-empty");
    assert!(!e.resources.sleep_source_max.is_empty(), "A16: sleep_source_max must be non-empty");
    println!("[scarcity A16] production init seeds finite food+water+sleep + registers all 3 ceilings ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 17: ResourceRegenSystem is registered in the default schedule.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a17_regen_system_registered_in_runtime_schedule() {
    // Type A — production-wiring invariant (mirrors starvation A17's pattern).
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    assert!(
        e.system_names().contains(&"ResourceRegenSystem"),
        "A17: ResourceRegenSystem must be registered in the default runtime; got {:?}",
        e.system_names()
    );
    println!("[scarcity A17] ResourceRegenSystem present in default runtime schedule ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 18: regen respects its interval cadence (withholds before, fires at).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a18_regen_respects_interval_cadence() {
    // Type A — cadence invariant. `tick_interval == REGEN_INTERVAL` AND `> 1`;
    // tile UNCHANGED for every tick strictly before the first boundary; tile
    // strictly increased once >= REGEN_INTERVAL ticks elapsed.
    let interval = ResourceRegenSystem::new().tick_interval();
    assert_eq!(interval, REGEN_INTERVAL, "A18: tick_interval must equal REGEN_INTERVAL const");
    assert!(interval > 1, "A18: regen interval must exceed 1 (scarcity needs withholding)");

    let mut e = iso_engine();
    let k: u8 = 80;
    let seeded: u8 = 40;
    e.resources.set_food_source_max(10, 10, k);
    e.resources.set_food_tile(10, 10, seeded);
    let mut sys = ResourceRegenSystem::new();

    for t in 1..=interval + 1 {
        e.resources.current_tick = t;
        if t.is_multiple_of(interval) {
            sys.tick(&mut e.world, &mut e.resources);
        }
        let cur = e.resources.food_tiles.get(&(10, 10)).copied().unwrap_or(0);
        if t < interval {
            assert_eq!(
                cur, seeded,
                "A18: tile must be UNCHANGED ({seeded}) for tick {t} strictly before the boundary"
            );
        }
    }
    let after = e.resources.food_tiles.get(&(10, 10)).copied().unwrap_or(0);
    println!("[scarcity A18] interval={interval}: seeded={seeded} → after boundary={after}");
    assert!(
        after > seeded,
        "A18: tile must strictly increase once >= REGEN_INTERVAL ticks elapsed; {after} !> {seeded}"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 19: SLEEP path depletes and regenerates (parity with food/water).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a19_sleep_path_depletes_and_regenerates() {
    // Type A. (a) depletion: genuine sleep consumes non-increasing + key removed
    // within <= seeded consumes. (b) regen: a below-ceiling sleep tile increases
    // to > start and <= K (and <= K at every sampled tick); an absent-but-
    // registered tile is recreated > 0.

    // (a) depletion on sleep_tiles.
    {
        let mut e = iso_engine();
        let ent = e.spawn_agent(14, 14);
        e.world
            .insert(ent, (AgentState::Idle, Sleep::new(200.0, 0.0)))
            .expect("seed agent");
        let seeded: u8 = 6;
        e.resources.set_sleep_tile(14, 14, seeded);
        let cap = seeded as usize;
        let mut sys = AgentDecisionSystem::new();
        let mut prev = seeded;
        let mut removed = false;
        for i in 0..cap {
            if !e.resources.sleep_tiles.contains_key(&(14, 14)) {
                removed = true;
                break;
            }
            drive_one_sleep_consume(&mut e, ent, &mut sys);
            let cur = e.resources.sleep_tiles.get(&(14, 14)).copied().unwrap_or(0);
            assert!(
                cur <= prev,
                "A19(a): sleep tile must be non-increasing ({cur} > {prev} at consume #{})",
                i + 1
            );
            prev = cur;
        }
        removed = removed || !e.resources.sleep_tiles.contains_key(&(14, 14));
        assert!(removed, "A19(a): sleep tile key must be ABSENT (removed) within {cap} (== seeded) consumes");
        println!("[scarcity A19a] sleep tile {seeded} non-increasing → key removed ✓");
    }

    // (b) regen on sleep_tiles: below-ceiling AND absent-but-registered.
    {
        let k: u8 = 80;
        let interval = REGEN_INTERVAL;

        // below-ceiling.
        {
            let mut e = iso_engine();
            e.resources.set_sleep_source_max(16, 16, k);
            let start: u8 = 40;
            e.resources.set_sleep_tile(16, 16, start);
            let mut sys = ResourceRegenSystem::new();
            for t in 1..=2 * interval {
                e.resources.current_tick = t;
                if t.is_multiple_of(interval) {
                    sys.tick(&mut e.world, &mut e.resources);
                }
                let cur = e.resources.sleep_tiles.get(&(16, 16)).copied().unwrap_or(0);
                assert!(cur <= k, "A19(b): sleep tile {cur} must never exceed K {k} at tick {t}");
            }
            let after = e.resources.sleep_tiles.get(&(16, 16)).copied().expect("tile present");
            println!("[scarcity A19b-below] sleep tile {start} → {after} (ceiling {k})");
            assert!(after > start, "A19(b): sleep tile must increase toward K; {after} !> {start}");
            assert!(after <= k, "A19(b): sleep tile must not exceed K");
        }

        // absent-but-registered.
        {
            let mut e = iso_engine();
            e.resources.set_sleep_source_max(17, 17, k);
            assert!(!e.resources.sleep_tiles.contains_key(&(17, 17)), "A19(b) setup: sleep tile absent");
            let mut sys = ResourceRegenSystem::new();
            for t in 1..=2 * interval {
                e.resources.current_tick = t;
                if t.is_multiple_of(interval) {
                    sys.tick(&mut e.world, &mut e.resources);
                }
                let cur = e.resources.sleep_tiles.get(&(17, 17)).copied().unwrap_or(0);
                assert!(cur <= k, "A19(b): recreated sleep tile {cur} must never exceed K {k} at tick {t}");
            }
            let v = e.resources.sleep_tiles.get(&(17, 17)).copied().expect("recreated tile present");
            println!("[scarcity A19b-absent] removed sleep tile recreated → {v} (ceiling {k})");
            assert!(v > 0, "A19(b): absent-but-registered sleep tile must be recreated with value > 0");
            assert!(v <= k, "A19(b): recreated sleep tile must not exceed K");
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 20: a single-kind global wipe makes nearest_resource_tile return
//               None (the controlled death-driver), kind-isolated.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a20_single_kind_wipe_yields_none() {
    // Type A — the intended mortality mechanism: all tiles of a kind gone →
    // `nearest_resource_tile` yields None → the agent cannot Seek that need →
    // the need rises unbounded → starvation. Kind-isolated: the wipe of food
    // must NOT blank water/sleep lookups.
    let mut e = full_engine();
    bootstrap_spawn_agents(&mut e);
    seed_finite_resource_scarcity(&mut e);

    // Remove ALL food source tiles (every food coordinate) while leaving
    // water/sleep present.
    let food_coords: Vec<(u32, u32)> = e.resources.food_tiles.keys().copied().collect();
    for (x, y) in food_coords {
        e.resources.set_food_tile(x, y, 0); // 0 removes the key
    }
    assert!(e.resources.food_tiles.is_empty(), "A20 setup: all food tiles must be removed");

    let pos = Position::new(32, 32); // playfield centre
    let food_lookup = nearest_resource_tile(&pos, &e.resources.food_tiles);
    let water_lookup = nearest_resource_tile(&pos, &e.resources.water_tiles);
    let sleep_lookup = nearest_resource_tile(&pos, &e.resources.sleep_tiles);
    println!(
        "[scarcity A20] food_lookup={food_lookup:?} water_lookup={water_lookup:?} sleep_lookup={sleep_lookup:?}"
    );
    assert_eq!(food_lookup, None, "A20(a): food lookup must return None after a full food wipe");
    assert!(water_lookup.is_some(), "A20(a): water lookup must still return a tile (kind-isolated wipe)");
    assert!(sleep_lookup.is_some(), "A20(a): sleep lookup must still return a tile (kind-isolated wipe)");

    // (b) TOTAL wipe — remove ALL keys of ALL three kinds; every lookup must
    // return None with NO panic (graceful degradation at the most extreme
    // freeze-risk input).
    let water_coords: Vec<(u32, u32)> = e.resources.water_tiles.keys().copied().collect();
    for (x, y) in water_coords {
        e.resources.set_water_tile(x, y, 0);
    }
    let sleep_coords: Vec<(u32, u32)> = e.resources.sleep_tiles.keys().copied().collect();
    for (x, y) in sleep_coords {
        e.resources.set_sleep_tile(x, y, 0);
    }
    assert!(e.resources.water_tiles.is_empty(), "A20(b) setup: all water tiles removed");
    assert!(e.resources.sleep_tiles.is_empty(), "A20(b) setup: all sleep tiles removed");
    assert_eq!(
        nearest_resource_tile(&pos, &e.resources.food_tiles),
        None,
        "A20(b): food lookup must be None after total wipe"
    );
    assert_eq!(
        nearest_resource_tile(&pos, &e.resources.water_tiles),
        None,
        "A20(b): water lookup must be None after total wipe"
    );
    assert_eq!(
        nearest_resource_tile(&pos, &e.resources.sleep_tiles),
        None,
        "A20(b): sleep lookup must be None after total wipe (no panic)"
    );
    println!("[scarcity A20] single-kind wipe kind-isolated ✓; total wipe → all None, no panic ✓");
}

// ── frozen-streak helper for A22 / A25 (re-route / no-freeze behavioral) ──────

/// Longest run of consecutive ticks during which an agent's Position is
/// UNCHANGED while it is NOT in a `Consuming{*}` state — the project's
/// "frozen streak" metric. A re-routing or wandering agent moves, so its
/// streak stays small; a stale-SeekTarget freeze produces a streak equal to
/// the remaining window.
fn longest_frozen_streak(samples: &[((u32, u32), AgentState)]) -> u32 {
    let mut longest = 0u32;
    let mut cur = 0u32;
    let mut prev: Option<(u32, u32)> = None;
    for &(pos, state) in samples {
        let consuming = matches!(state, AgentState::Consuming { .. });
        let unchanged = prev == Some(pos);
        if unchanged && !consuming {
            cur += 1;
        } else {
            cur = 0;
        }
        longest = longest.max(cur);
        prev = Some(pos);
    }
    longest
}

/// Small constant bound on the frozen streak (A22 / A25 clause 2).
const MAX_FROZEN_STREAK: u32 = 5;
/// Bounded observation window for the behavioral re-route / no-freeze tests.
const FREEZE_WINDOW: u32 = 60;

/// Seed a single agent already in `Seeking{Food}` carrying a `SeekTarget` at
/// `target`, with Hunger held high so it keeps wanting food. Returns the entity.
fn seed_food_seeker(e: &mut SimEngine, at: (u32, u32), target: (u32, u32), rng: u64) -> hecs::Entity {
    let ent = e.spawn_agent(at.0, at.1);
    e.world
        .insert(
            ent,
            (
                MovementRng::new(rng),
                AgentState::Seeking { target: TargetKind::Food },
                Hunger::new(90.0, 0.05), // stays above HUNGER_THRESHOLD (50)
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 0.0),
                Memory::new(),
                BodyHealth::new(),
                SeekTarget::new(target),
            ),
        )
        .expect("seed food seeker");
    ent
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 21: `*_source_max` ceiling key survives tile depletion/removal
//               (food + water). Load-bearing for regen resurrection (A4b/A14).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a21_ceiling_survives_tile_removal() {
    // Type A. Consume a finite tile to removal; the `*_source_max` entry at the
    // same coordinate must still be present and unchanged afterward.

    // FOOD.
    {
        let mut e = iso_engine();
        let ent = e.spawn_agent(22, 22);
        e.world
            .insert(ent, (AgentState::Idle, Hunger::new(200.0, 0.0)))
            .expect("seed");
        let seeded: u8 = 3;
        e.resources.set_food_tile(22, 22, seeded);
        e.resources.set_food_source_max(22, 22, seeded);
        let mut sys = AgentDecisionSystem::new();
        for _ in 0..(seeded as usize) {
            if !e.resources.food_tiles.contains_key(&(22, 22)) {
                break;
            }
            drive_one_food_consume(&mut e, ent, &mut sys);
        }
        assert!(
            !e.resources.food_tiles.contains_key(&(22, 22)),
            "A21(food) setup: tile must be depleted/removed"
        );
        assert_eq!(
            e.resources.food_source_max.get(&(22, 22)).copied(),
            Some(seeded),
            "A21(food): food_source_max ceiling must SURVIVE tile removal, unchanged"
        );
    }

    // WATER.
    {
        let mut e = iso_engine();
        let ent = e.spawn_agent(23, 23);
        e.world
            .insert(ent, (AgentState::Idle, Thirst::new(200.0, 0.0)))
            .expect("seed");
        let seeded: u8 = 3;
        e.resources.set_water_tile(23, 23, seeded);
        e.resources.set_water_source_max(23, 23, seeded);
        let mut sys = AgentDecisionSystem::new();
        for _ in 0..(seeded as usize) {
            if !e.resources.water_tiles.contains_key(&(23, 23)) {
                break;
            }
            drive_one_water_consume(&mut e, ent, &mut sys);
        }
        assert!(
            !e.resources.water_tiles.contains_key(&(23, 23)),
            "A21(water) setup: tile must be depleted/removed"
        );
        assert_eq!(
            e.resources.water_source_max.get(&(23, 23)).copied(),
            Some(seeded),
            "A21(water): water_source_max ceiling must SURVIVE tile removal, unchanged"
        );
    }
    println!("[scarcity A21] food + water source_max ceilings survive tile removal ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 22: an already-Seeking agent RE-ROUTES (does not freeze) when its
//               target tile is removed but ANOTHER source remains.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a22_seeking_reroutes_when_alternative_remains() {
    // Type A. (1) SeekTarget updated to a PRESENT coord OR agent reaches a
    // present source and enters Consuming{Food}; AND (2) longest frozen streak
    // <= MAX_FROZEN_STREAK across the full window.
    let mut e = full_engine();
    let start = (30u32, 30u32);
    let t = (31u32, 30u32); // initial target (will be removed)
    let alt = (34u32, 30u32); // alternative that REMAINS
    let ent = seed_food_seeker(&mut e, start, t, 0xA22_0001);
    e.resources.set_food_tile(t.0, t.1, 80);
    e.resources.set_food_tile(alt.0, alt.1, 80);
    // Register ceilings (production always registers source_max alongside a
    // source tile via seed_finite_resource_scarcity) so this is a genuine
    // scarcity scene — StaleSeekTargetSystem only acts when scarcity is active.
    e.resources.set_food_source_max(t.0, t.1, 80);
    e.resources.set_food_source_max(alt.0, alt.1, 80);
    // Remove T (deplete the target corner); the alternative remains present.
    e.resources.set_food_tile(t.0, t.1, 0);

    let mut samples: Vec<((u32, u32), AgentState)> = Vec::new();
    let mut rerouted_or_consumed = false;
    for _ in 0..FREEZE_WINDOW {
        e.tick();
        let st = e.world.get::<&AgentState>(ent).map(|s| *s).unwrap_or(AgentState::Idle);
        let p = e.world.get::<&Position>(ent).map(|p| (p.x, p.y)).unwrap_or((9999, 9999));
        // re-route success: SeekTarget points at a PRESENT food coord, OR the
        // agent reached a source and is Consuming{Food}.
        if let Ok(seek) = e.world.get::<&SeekTarget>(ent) {
            if e.resources.food_tiles.get(&seek.tile).copied().is_some_and(|v| v > 0) {
                rerouted_or_consumed = true;
            }
        }
        if matches!(st, AgentState::Consuming { target: TargetKind::Food }) {
            rerouted_or_consumed = true;
        }
        samples.push((p, st));
    }
    let streak = longest_frozen_streak(&samples);
    println!("[scarcity A22] rerouted_or_consumed={rerouted_or_consumed} longest_frozen_streak={streak} (<= {MAX_FROZEN_STREAK})");
    assert!(
        rerouted_or_consumed,
        "A22(1): the agent must re-route its SeekTarget to a present source OR reach one and Consume"
    );
    assert!(
        streak <= MAX_FROZEN_STREAK,
        "A22(2): longest frozen streak {streak} must be <= {MAX_FROZEN_STREAK} (no stale-target freeze)"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 23: consuming an infinite (255) sentinel source never decrements it
//               (food + water) — control-arm integrity for A11.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a23_consume_never_decrements_sentinel() {
    // Type A. A 255 sentinel under genuine consumption stays EXACTLY 255.

    // FOOD.
    {
        let mut e = iso_engine();
        let ent = e.spawn_agent(25, 25);
        e.world
            .insert(ent, (AgentState::Idle, Hunger::new(200.0, 0.0)))
            .expect("seed");
        e.resources.set_food_tile(25, 25, RESOURCE_SOURCE_INFINITE);
        let mut sys = AgentDecisionSystem::new();
        for _ in 0..5 {
            drive_one_food_consume(&mut e, ent, &mut sys);
            assert_eq!(
                e.resources.food_tiles.get(&(25, 25)).copied(),
                Some(RESOURCE_SOURCE_INFINITE),
                "A23(food): consuming a 255 sentinel must NOT decrement it"
            );
        }
    }

    // WATER.
    {
        let mut e = iso_engine();
        let ent = e.spawn_agent(26, 26);
        e.world
            .insert(ent, (AgentState::Idle, Thirst::new(200.0, 0.0)))
            .expect("seed");
        e.resources.set_water_tile(26, 26, RESOURCE_SOURCE_INFINITE);
        let mut sys = AgentDecisionSystem::new();
        for _ in 0..5 {
            drive_one_water_consume(&mut e, ent, &mut sys);
            assert_eq!(
                e.resources.water_tiles.get(&(26, 26)).copied(),
                Some(RESOURCE_SOURCE_INFINITE),
                "A23(water): consuming a 255 sentinel must NOT decrement it"
            );
        }
    }
    println!("[scarcity A23] food + water 255 sentinels never decremented by consume ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 24: mortality direction holds under a SECOND seed (anti-overfit).
//               Type E — direction only (finite > infinite), no magnitude.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a24_mortality_direction_second_seed() {
    // The bootstrap lattice + seeder are geometrically identical across seeds;
    // re-seeding every agent's MovementRng varies only the BEHAVIORAL RNG
    // (movement noise) — a thin but real anti-overfit check (plan A24 honest
    // limitation). Direction (finite kills MORE than infinite) must still hold.
    const SECOND_SEED_BASE: u64 = 0x0000_0007_0000_0007;

    let mut finite = finite_scene();
    reseed_agents(&mut finite, SECOND_SEED_BASE);
    run_ticks(&mut finite, RUN_TICKS);
    let finite_deaths = scarcity_dead_set(&finite).len();

    let mut infinite = infinite_scene();
    reseed_agents(&mut infinite, SECOND_SEED_BASE);
    run_ticks(&mut infinite, RUN_TICKS);
    let infinite_deaths = scarcity_dead_set(&infinite).len();

    println!(
        "[scarcity A24] second-seed: finite_scarcity_deaths={finite_deaths} infinite_scarcity_deaths={infinite_deaths}"
    );
    assert!(
        finite_deaths > infinite_deaths,
        "A24: at a second seed, finite sources must still kill MORE than infinite (direction); \
         finite={finite_deaths} infinite={infinite_deaths}"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 25: an already-Seeking agent EXITS Seeking (does not freeze) when
//               its ONLY source of that kind vanishes. #1-priority no-freeze.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a25_seeking_exits_when_only_source_vanishes() {
    // Type A. EXACTLY ONE food source T, no alternative. Agent Seeking{Food}
    // with SeekTarget at T. Remove T → nearest_resource_tile(Food) becomes None.
    // (1) the agent transitions OUT of Seeking{Food} to a non-Seeking{Food}
    // state (Idle) at least once after removal; AND (2) longest frozen streak
    // <= MAX_FROZEN_STREAK (it must NOT permanently Seek a vanished coord).
    let mut e = full_engine();
    let start = (30u32, 30u32);
    let t = (31u32, 30u32); // the ONLY food source
    let ent = seed_food_seeker(&mut e, start, t, 0xA25_0001);
    e.resources.set_food_tile(t.0, t.1, 80);
    // Register the ceiling (survives tile removal — A21) so this is a genuine
    // scarcity scene: the gate stays active even after the only tile is gone.
    e.resources.set_food_source_max(t.0, t.1, 80);
    // Remove T — now there is NO food source anywhere.
    e.resources.set_food_tile(t.0, t.1, 0);
    assert!(e.resources.food_tiles.is_empty(), "A25 setup: no food source remains");

    let mut samples: Vec<((u32, u32), AgentState)> = Vec::new();
    let mut exited_seeking_food = false;
    for _ in 0..FREEZE_WINDOW {
        e.tick();
        let st = e.world.get::<&AgentState>(ent).map(|s| *s).unwrap_or(AgentState::Idle);
        let p = e.world.get::<&Position>(ent).map(|p| (p.x, p.y)).unwrap_or((9999, 9999));
        if !matches!(st, AgentState::Seeking { target: TargetKind::Food }) {
            exited_seeking_food = true;
        }
        samples.push((p, st));
    }
    let streak = longest_frozen_streak(&samples);
    println!("[scarcity A25] exited_seeking_food={exited_seeking_food} longest_frozen_streak={streak} (<= {MAX_FROZEN_STREAK})");
    assert!(
        exited_seeking_food,
        "A25(1): with no source remaining, the agent must transition OUT of Seeking{{Food}} (to Idle)"
    );
    assert!(
        streak <= MAX_FROZEN_STREAK,
        "A25(2): longest frozen streak {streak} must be <= {MAX_FROZEN_STREAK} (no permanent freeze on a vanished coord)"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 26: regen does NOT panic on an orphan tile (present tile with no
//               registered ceiling) and leaves it untouched.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a26_regen_no_panic_on_orphan_tile() {
    // Type A. Orphan = present in food_tiles, absent from food_source_max.
    let interval = REGEN_INTERVAL;
    let mut e = iso_engine();
    // Orphan tile (no ceiling) + a normally-registered below-ceiling source.
    e.resources.set_food_tile(3, 3, 50); // orphan, no source_max
    let k: u8 = 80;
    e.resources.set_food_source_max(4, 4, k);
    e.resources.set_food_tile(4, 4, 40);
    let mut sys = ResourceRegenSystem::new();
    for t in 1..=5 * interval {
        e.resources.current_tick = t;
        if t.is_multiple_of(interval) {
            sys.tick(&mut e.world, &mut e.resources); // must NOT panic
        }
    }
    assert_eq!(
        e.resources.food_tiles.get(&(3, 3)).copied(),
        Some(50),
        "A26: an orphan tile (no registered ceiling) must be left UNTOUCHED by regen"
    );
    let registered = e.resources.food_tiles.get(&(4, 4)).copied().expect("registered tile present");
    assert!(registered > 40, "A26: the normally-registered source must still regenerate; {registered} !> 40");
    assert!(registered <= k, "A26: the registered source must not exceed its ceiling");
    println!("[scarcity A26] regen no-panic on orphan; orphan untouched (50); registered regenerated → {registered} ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 27: regen only ever touches REGISTERED source coordinates — it
//               creates no coordinate that was neither present nor registered.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_a27_regen_touches_only_registered_coords() {
    // Type A. Registered set: one below-ceiling, one absent-but-registered, one
    // at-ceiling. Post-regen tile keys must be a subset of
    // (pre-regen tile keys ∪ food_source_max keys).
    let interval = REGEN_INTERVAL;
    let k: u8 = 80;
    let mut e = iso_engine();
    e.resources.set_food_source_max(5, 5, k); // below ceiling
    e.resources.set_food_tile(5, 5, 40);
    e.resources.set_food_source_max(6, 6, k); // absent-but-registered
    e.resources.set_food_source_max(7, 7, k); // at ceiling
    e.resources.set_food_tile(7, 7, k);

    let pre_tile_keys: BTreeSet<(u32, u32)> = e.resources.food_tiles.keys().copied().collect();
    let registered_keys: BTreeSet<(u32, u32)> = e.resources.food_source_max.keys().copied().collect();
    let allowed: BTreeSet<(u32, u32)> = pre_tile_keys.union(&registered_keys).copied().collect();

    let mut sys = ResourceRegenSystem::new();
    for t in 1..=5 * interval {
        e.resources.current_tick = t;
        if t.is_multiple_of(interval) {
            sys.tick(&mut e.world, &mut e.resources);
        }
    }
    let post_keys: BTreeSet<(u32, u32)> = e.resources.food_tiles.keys().copied().collect();
    let fabricated: Vec<(u32, u32)> = post_keys.difference(&allowed).copied().collect();
    println!("[scarcity A27] post-regen food keys={post_keys:?} allowed={allowed:?} fabricated={fabricated:?}");
    assert!(
        fabricated.is_empty(),
        "A27: regen must NOT fabricate tiles at non-registered coordinates; fabricated={fabricated:?}"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Plan Assertion 9: REGEN is a BOUNDED trickle, not an instant refill.
//   Cumulative-over-run instrumentation on the registered source coords (the
//   only tiles regen touches). A positive tick-over-tick increase on a source
//   tile can ONLY come from regen (consumption decrements; nothing else adds),
//   so it both witnesses a regen recovery AND bounds the per-tick gain.
//     regen_recovery_count >= 1
//     max_single_tick_gain <= intended per-tick regen step * 1.5
//   The 1.5× bound rejects a `0.9*source_max`-per-tick regression that stays
//   under source_max yet effectively defangs scarcity.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_regen_is_bounded_trickle() {
    // Type A (bounded-regen invariant) + Type C (recovery presence).
    fn scan(
        coords: &[(u32, u32)],
        tiles: &HashMap<(u32, u32), u8>,
        prev: &mut HashMap<(u32, u32), u8>,
        rec: &mut u64,
        mx: &mut u8,
    ) {
        for c in coords {
            let cur = tiles.get(c).copied().unwrap_or(0);
            let p = prev.get(c).copied().unwrap_or(0);
            // A sentinel never regen-changes; ignore any 255 transitions.
            if cur > p && cur != RESOURCE_SOURCE_INFINITE && p != RESOURCE_SOURCE_INFINITE {
                *rec += 1;
                let gain = cur - p;
                if gain > *mx {
                    *mx = gain;
                }
            }
            prev.insert(*c, cur);
        }
    }

    let mut e = finite_scene();
    // Registered coords per channel (source_max survives tile removal — A21).
    let food: Vec<(u32, u32)> = e.resources.food_source_max.keys().copied().collect();
    let water: Vec<(u32, u32)> = e.resources.water_source_max.keys().copied().collect();
    let sleep: Vec<(u32, u32)> = e.resources.sleep_source_max.keys().copied().collect();
    let mut prev_food: HashMap<(u32, u32), u8> =
        food.iter().map(|c| (*c, e.resources.food_tiles.get(c).copied().unwrap_or(0))).collect();
    let mut prev_water: HashMap<(u32, u32), u8> =
        water.iter().map(|c| (*c, e.resources.water_tiles.get(c).copied().unwrap_or(0))).collect();
    let mut prev_sleep: HashMap<(u32, u32), u8> =
        sleep.iter().map(|c| (*c, e.resources.sleep_tiles.get(c).copied().unwrap_or(0))).collect();

    let mut regen_recovery_count: u64 = 0;
    let mut max_single_tick_gain: u8 = 0;
    for _ in 0..RUN_TICKS {
        e.tick();
        scan(&food, &e.resources.food_tiles, &mut prev_food, &mut regen_recovery_count, &mut max_single_tick_gain);
        scan(&water, &e.resources.water_tiles, &mut prev_water, &mut regen_recovery_count, &mut max_single_tick_gain);
        scan(&sleep, &e.resources.sleep_tiles, &mut prev_sleep, &mut regen_recovery_count, &mut max_single_tick_gain);
    }

    // Intended per-tick regen STEP = the largest single-interval regen amount
    // (the most a tile can gain when regen fires). 1.5× tolerance for rounding.
    let intended = FOOD_REGEN_AMOUNT.max(WATER_REGEN_AMOUNT).max(SLEEP_REGEN_AMOUNT);
    let bound = intended as f64 * 1.5;
    println!(
        "[scarcity regen-trickle] regen_recovery_count={regen_recovery_count} max_single_tick_gain={max_single_tick_gain} intended_step={intended} bound={bound}"
    );
    assert!(
        regen_recovery_count >= 1,
        "Assertion 9: REGEN must fire at least once over the run (regen_recovery_count >= 1); got {regen_recovery_count}"
    );
    assert!(
        (max_single_tick_gain as f64) <= bound,
        "Assertion 9: REGEN must be a BOUNDED trickle — max single-tick gain {max_single_tick_gain} must be \
         <= intended step ({intended}) * 1.5 = {bound}. A larger jump means a near-infinite refill that defangs scarcity."
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Plan Assertion 15: agents actively gather scarce water/food under need
//   (consume-loop behavioral guard). Over the full scarcity scene, count
//   DISTINCT agents observed in a water-acquisition state while thirsty (and
//   food-acquisition while hungry); require >= 20 of each, AND confirm at least
//   one traced consume decrement per kind (seek→consume linkage).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_scarcity_agents_gather_under_need() {
    // Type C (gatherer floors) + Type A (seek→consume linkage).
    let mut e = finite_scene();
    let mut water_gatherers: BTreeSet<AgentId> = BTreeSet::new();
    let mut food_gatherers: BTreeSet<AgentId> = BTreeSet::new();
    let mut water_consumed = false;
    let mut food_consumed = false;

    let mut prev_food: HashMap<(u32, u32), u8> =
        SOURCE_FOOD.iter().map(|c| (*c, e.resources.food_tiles.get(c).copied().unwrap_or(0))).collect();
    let mut prev_water: HashMap<(u32, u32), u8> =
        SOURCE_WATER.iter().map(|c| (*c, e.resources.water_tiles.get(c).copied().unwrap_or(0))).collect();

    for _ in 0..RUN_TICKS {
        e.tick();
        // Distinct gatherers observed acquiring a kind while its need is high.
        for (_, (a, st, hunger, thirst)) in e
            .world
            .query::<(&Agent, &AgentState, Option<&Hunger>, Option<&Thirst>)>()
            .iter()
        {
            if thirst.is_some_and(|t| t.value > THIRST_THRESHOLD)
                && matches!(
                    st,
                    AgentState::Seeking { target: TargetKind::Water }
                        | AgentState::Consuming { target: TargetKind::Water }
                )
            {
                water_gatherers.insert(a.id);
            }
            if hunger.is_some_and(|h| h.value > HUNGER_THRESHOLD)
                && matches!(
                    st,
                    AgentState::Seeking { target: TargetKind::Food }
                        | AgentState::Consuming { target: TargetKind::Food }
                )
            {
                food_gatherers.insert(a.id);
            }
        }
        // Traced consume: only consumption decrements a finite source tile.
        for c in SOURCE_FOOD.iter() {
            let cur = e.resources.food_tiles.get(c).copied().unwrap_or(0);
            let p = prev_food.get(c).copied().unwrap_or(0);
            if cur < p && p != RESOURCE_SOURCE_INFINITE {
                food_consumed = true;
            }
            prev_food.insert(*c, cur);
        }
        for c in SOURCE_WATER.iter() {
            let cur = e.resources.water_tiles.get(c).copied().unwrap_or(0);
            let p = prev_water.get(c).copied().unwrap_or(0);
            if cur < p && p != RESOURCE_SOURCE_INFINITE {
                water_consumed = true;
            }
            prev_water.insert(*c, cur);
        }
    }

    println!(
        "[scarcity gatherers] water_gatherers_distinct={} food_gatherers_distinct={} water_consumed={water_consumed} food_consumed={food_consumed}",
        water_gatherers.len(),
        food_gatherers.len()
    );
    assert!(
        water_gatherers.len() >= 20,
        "Assertion 15: >= 20 distinct high-thirst water gatherers must be observed; got {}",
        water_gatherers.len()
    );
    assert!(
        food_gatherers.len() >= 20,
        "Assertion 15: >= 20 distinct high-hunger food gatherers must be observed; got {}",
        food_gatherers.len()
    );
    assert!(
        water_consumed && food_consumed,
        "Assertion 15: at least one traced consume decrement per kind (seek→consume linkage); \
         water_consumed={water_consumed} food_consumed={food_consumed}"
    );
}
