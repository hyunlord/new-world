//! Pass 12 Part 2: fission as a daily hazard (`game_v18`, PASS12_GOAL.md 7.1).
use sim_core::components::{Agent, AgentId, AgentState, Child, Elder, Lineage, Position, ResourceKind, Settlement};
use sim_core::material::MaterialRegistry;
use sim_core::rng::{keyed_below, NoiseFork, STREAM_FISSION_HAZARD};
use sim_core::rules::WorldRules;
use sim_engine::{FissionCheck, RuntimeSystem, SimEngine, WildRegion};
use sim_systems::runtime::settlement::fission::{hazard_terms, sustain_capacity};
use sim_systems::runtime::settlement::food_economy::adult_need_per_day;
use sim_systems::runtime::settlement::FissionSystem;
use sim_test::probe::fingerprint;
use sim_test::scenes::{game_v17_scene, game_v18_nofission_scene, game_v18_scene};

const HOME: (u32, u32) = (20, 20);
const SITE: (u32, u32) = (46, 20);

/// Tick at which the display clock (480-tick offset) reads `minute` on the
/// zero-based display day `day` (season = (day / 7) % 4).
fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

/// A bare world under `rules`: one settlement (id 1) at `HOME` founded at tick
/// 0 holding `food`, `bushes` capacity-4 bushes, one wild region at `SITE`,
/// and `members` residents (ids 1..=n; every fifth a child, every seventh an
/// elder) spawned in order, or in reverse order with `reverse`.
fn world(rules: WorldRules, seed: u64, bushes: u32, food: u32, members: u64, reverse: bool) -> SimEngine {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
    engine.resources.world_seed = seed;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = HOME;
    settlement.store(ResourceKind::Food, food);
    for i in 0..bushes {
        engine.resources.set_food_source_max(10 + i % 20, 10 + i / 20, 4);
    }
    let mut ids: Vec<u64> = (1..=members).collect();
    if reverse {
        ids.reverse();
    }
    for id in ids {
        let entity = engine.world.spawn((
            Position::new(HOME.0, HOME.1),
            Agent { id: id as AgentId },
            AgentState::Idle,
            Lineage { key: id.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x55, births: 0 },
        ));
        if id % 7 == 0 {
            engine.world.insert_one(entity, Elder).expect("elder");
        } else if id % 5 == 0 {
            engine.world.insert_one(entity, Child { born_tick: 0 }).expect("child");
        }
        settlement.add_member_agent(id as AgentId);
    }
    engine.resources.settlements.insert(1, settlement);
    engine.resources.wild_regions.push(WildRegion { centre: SITE, bushes: 8, deadwood: 12, water_distance: 3 });
    engine
}

/// Run a fresh `FissionSystem` at every tick of [from, to).
fn run(engine: &mut SimEngine, from: u64, to: u64) {
    let mut system = FissionSystem::new();
    for tick in from..to {
        engine.resources.current_tick = tick;
        system.tick(&mut engine.world, &mut engine.resources);
    }
}

/// Recompute a logged check from its inputs (7.1's formula and draw).
fn recompute(engine: &SimEngine, c: &FissionCheck, replicate: u32) -> (i64, i64, i64, i64, u64) {
    let hazard = engine.resources.rules.society.fission.and_then(|f| f.hazard).expect("hazard");
    let (g, q, s, h) = hazard_terms(&hazard, c.members, c.capacity, c.food, adult_need_per_day(engine.resources.ticks_per_day));
    let draw = keyed_below(engine.resources.world_seed, STREAM_FISSION_HAZARD, u64::from(c.settlement), c.tick, 0, replicate, 1000);
    (g, q, s, h, draw)
}

#[test]
fn harness_hazard_inert_in_v17() {
    let rule = WorldRules::game_v17(1).society.fission.expect("v17 fission");
    assert_eq!((rule.hazard, rule.allowed_seasons), (None, [true, true, false, false]));
    // Seeds 1 and 2 of v17 split at 06:00 of display day 29: no hazard check is ever logged.
    for seed in [1, 2] {
        let mut engine = game_v17_scene(seed);
        for _ in 0..42 * 1440 {
            engine.tick();
        }
        assert!(!engine.resources.fission_log.is_empty(), "seed {seed} departs in v17");
        assert!(engine.resources.fission_checks.is_empty());
        assert_eq!(engine.resources.tallies.wet_burn_wood, 0);
    }
    let mut engine = game_v18_nofission_scene(1);
    for _ in 0..42 * 1440 {
        engine.tick();
    }
    assert!(engine.resources.fission_checks.is_empty() && engine.resources.fission_log.is_empty());
}

#[test]
fn harness_hazard_formula() {
    let hz = WorldRules::V18_HAZARD;
    // Pure terms (need per member 1.0): an ordinary case, capacity 0 (counts
    // as 1: q = 1000), an empty store (s = 1000) and size at size_lo (h = 0).
    assert_eq!(hazard_terms(&hz, 30, 30, 30, 1.0), (500, 750, 500, 37));
    assert_eq!(hazard_terms(&hz, 30, 0, 30, 1.0), (500, 1000, 500, 45));
    assert_eq!(hazard_terms(&hz, 30, 30, 0, 1.0), (500, 750, 1000, 52));
    assert_eq!(hazard_terms(&hz, 20, 0, 0, 1.0), (0, 1000, 1000, 0));
    // Constructed settlements: the logged check is the formula on the
    // settlement's members, sustain capacity and own store, and the draw is
    // the keyed draw (replicate 0). Four cases: no bushes (capacity 0), an
    // empty store, a roomy full store, the smallest eligible village.
    let seed = 900_412;
    let check = tick_at(28, 360);
    for (bushes, food, members) in [(0u32, 100u32, 30u64), (40, 0, 30), (200, 2000, 30), (40, 100, 22)] {
        let mut engine = world(WorldRules::game_v18(seed), seed, bushes, food, members, false);
        let capacity = sustain_capacity(&engine.resources, 1).expect("capacity");
        let resident_members = engine.resources.settlements[&1].member_agents.len() as u32;
        run(&mut engine, check, check + 1);
        let logged = engine.resources.fission_checks.clone();
        assert_eq!(logged.len(), 1, "case {bushes}/{food}/{members}");
        let c = logged[0];
        assert_eq!((c.tick, c.settlement, c.members, c.capacity, c.food), (check, 1, resident_members, capacity, food));
        assert_eq!((c.g, c.q, c.s, c.h, c.draw), recompute(&engine, &c, 0));
        assert_eq!(c.split, (c.draw as i64) < c.h);
        assert!(c.site_free);
        assert_eq!(engine.resources.fission_log.len(), usize::from(c.split));
        if bushes == 0 {
            assert_eq!((capacity, c.q), (0, 1000));
        }
        if food == 0 {
            assert_eq!(c.s, 1000);
        }
    }
}

#[test]
fn harness_hazard_keyed_with_replicate() {
    // A whole v18 world: a replicate-0 fork equals the unforked run.
    let seed = 900_007;
    let (mut a, mut b, mut c) = (game_v18_scene(seed), game_v18_scene(seed), game_v18_scene(seed));
    b.resources.noise_fork = Some(NoiseFork { tick: 0, replicate: 0 });
    c.resources.noise_fork = Some(NoiseFork { tick: 0, replicate: 3 });
    for _ in 0..60 * 1440 {
        a.tick();
        b.tick();
        c.tick();
    }
    assert_eq!(a.resources.fission_checks, b.resources.fission_checks);
    assert_eq!(fingerprint(&a), fingerprint(&b));
    assert!(!a.resources.fission_checks.is_empty(), "the world reaches the draw");
    for check in &c.resources.fission_checks {
        assert_eq!(check.draw, recompute(&c, check, 3).4, "replicate 3 draws with replicate 3");
    }
    // Constant h: the first split day of settlement 1 over the daily checks of
    // days 10..120 (winter excluded) changes with the replicate in at least
    // one of eight seeds.
    let h = 60u64;
    let first = |seed: u64, replicate: u32| {
        (10..120u64)
            .filter(|day| (day / 7) % 4 != 3)
            .find(|&day| keyed_below(seed, STREAM_FISSION_HAZARD, 1, tick_at(day, 360), 0, replicate, 1000) < h)
    };
    let changed = (0..8).filter(|&s| first(s, 0) != first(s, 3)).count();
    assert!(changed >= 1, "replicate 3 changes a split day");
}

#[test]
fn harness_hazard_rises_with_shortfall() {
    let hz = WorldRules::V18_HAZARD;
    let mut last = i64::MAX;
    for food in (0..=200).step_by(10) {
        let h = hazard_terms(&hz, 30, 30, food, 1.0).3;
        assert!(h <= last, "h never rises as the store grows");
        last = h;
    }
    assert!(hazard_terms(&hz, 30, 30, 0, 1.0).3 > hazard_terms(&hz, 30, 30, 200, 1.0).3);
    // In a constructed settlement: the same village with an empty store has
    // s = 1000 and a higher hazard than with a full one.
    let seed = 900_413;
    let check = tick_at(28, 360);
    let hazard_with = |food: u32| {
        let mut engine = world(WorldRules::game_v18(seed), seed, 40, food, 30, false);
        run(&mut engine, check, check + 1);
        engine.resources.fission_checks[0]
    };
    let (empty, full) = (hazard_with(0), hazard_with(5000));
    assert_eq!((empty.s, full.s), (1000, 0));
    assert!(empty.h > full.h);
}

#[test]
fn harness_allowed_seasons() {
    let seed = 900_414;
    // v18: spring (day 28), summer (day 35) and autumn (day 14) checks reach
    // the draw; winter (day 21) never does.
    for (day, logged) in [(14u64, true), (21, false), (28, true), (35, true)] {
        let mut engine = world(WorldRules::game_v18(seed), seed, 0, 0, 30, false);
        run(&mut engine, tick_at(day, 360), tick_at(day, 360) + 1);
        assert_eq!(!engine.resources.fission_checks.is_empty(), logged, "day {day}");
    }
    // No allowed season: no check at all.
    let mut rules = WorldRules::game_v18(seed);
    if let Some(fission) = rules.society.fission.as_mut() {
        fission.allowed_seasons = [false; 4];
    }
    let mut engine = world(rules, seed, 0, 0, 30, false);
    run(&mut engine, tick_at(10, 0), tick_at(40, 0));
    assert!(engine.resources.fission_checks.is_empty() && engine.resources.fission_log.is_empty());
    // A whole v18 world never logs a winter check.
    let mut engine = game_v18_scene(900_008);
    for _ in 0..120 * 1440 {
        engine.tick();
    }
    let checks = &engine.resources.fission_checks;
    assert!(!checks.is_empty());
    for c in checks {
        assert_ne!(((c.tick + 480) / 1440 / 7) % 4, 3, "winter check at tick {}", c.tick);
    }
}

#[test]
fn harness_hazard_order_independent() {
    // Spawn order changes nothing.
    let seed = 900_415;
    let check = tick_at(28, 360);
    let outcome = |reverse: bool| {
        let mut engine = world(WorldRules::game_v18(seed), seed, 0, 0, 30, reverse);
        run(&mut engine, check, check + 1);
        (engine.resources.fission_checks.clone(), engine.resources.fission_log.clone())
    };
    assert_eq!(outcome(false), outcome(true));
    // The draw of (sid, tick) is its keyed draw alone: in a whole v18 world
    // with several settlements every logged draw is recomputed from (seed,
    // sid, tick, replicate 0), whichever other settlements exist or were checked.
    let mut engine = game_v18_scene(900_009);
    for _ in 0..120 * 1440 {
        engine.tick();
    }
    let checks = engine.resources.fission_checks.clone();
    assert!(!checks.is_empty());
    for c in &checks {
        assert_eq!((c.g, c.q, c.s, c.h, c.draw), recompute(&engine, c, 0));
    }
    let mut twin = game_v18_scene(900_009);
    for _ in 0..120 * 1440 {
        twin.tick();
    }
    assert_eq!(twin.resources.fission_checks, checks);
    assert_eq!(fingerprint(&twin), fingerprint(&engine));
}
