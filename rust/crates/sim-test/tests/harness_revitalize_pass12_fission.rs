//! Pass 12 Part 2: fission as a daily hazard (PASS12_GOAL.md 7.1).
//!
//! V8-0: rebased from `game_v18` with the goal's start hazard on
//! `WorldRules::baseline(seed)`, whose hazard is Pass 13's (`V19_HAZARD`: size
//! 24..44, crowding weight 0, scarcity weight 1000 over 4 days, ceiling 300)
//! and whose fission accumulates the hazard as pressure against a hidden
//! threshold (Pass 13) instead of a daily draw: a logged check carries draw 0
//! and splits when its pressure reaches its threshold.
use sim_core::components::{
    Agent, AgentId, AgentState, Child, Elder, Lineage, Position, ResourceKind, Settlement,
};
use sim_core::fission_threshold::fission_threshold;
use sim_core::material::MaterialRegistry;
use sim_core::rules::{FissionHazard, WorldRules};
use sim_engine::{FissionCheck, RuntimeSystem, SimEngine, WildRegion};
use sim_systems::runtime::settlement::fission::{hazard_terms, sustain_capacity};
use sim_systems::runtime::settlement::food_economy::adult_need_per_day;
use sim_systems::runtime::settlement::FissionSystem;

const HOME: (u32, u32) = (20, 20);
const SITE: (u32, u32) = (46, 20);

/// The baseline fission hazard.
fn baseline_hazard() -> FissionHazard {
    WorldRules::baseline(0)
        .society
        .fission
        .and_then(|f| f.hazard)
        .expect("baseline hazard")
}

/// The baseline rules of world `seed`.
fn baseline_rules(seed: u64) -> WorldRules {
    WorldRules::baseline(seed)
}

/// Tick at which the display clock (480-tick offset) reads `minute` on the
/// zero-based display day `day` (season = (day / 7) % 4).
fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

/// A bare world under `rules`: one settlement (id 1) at `HOME` founded at tick
/// 0 holding `food`, `bushes` capacity-4 bushes, one wild region at `SITE`,
/// and `members` residents (ids 1..=n; every fifth a child, every seventh an
/// elder) spawned in order, or in reverse order with `reverse`.
fn world(
    rules: WorldRules,
    seed: u64,
    bushes: u32,
    food: u32,
    members: u64,
    reverse: bool,
) -> SimEngine {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
    engine.resources.world_seed = seed;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = HOME;
    settlement.store(ResourceKind::Food, food);
    for i in 0..bushes {
        engine
            .resources
            .set_food_source_max(10 + i % 20, 10 + i / 20, 4);
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
            Lineage {
                key: id.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x55,
                births: 0,
            },
        ));
        if id % 7 == 0 {
            engine.world.insert_one(entity, Elder).expect("elder");
        } else if id % 5 == 0 {
            engine
                .world
                .insert_one(entity, Child { born_tick: 0 })
                .expect("child");
        }
        settlement.add_member_agent(id as AgentId);
    }
    engine.resources.settlements.insert(1, settlement);
    engine.resources.wild_regions.push(WildRegion {
        centre: SITE,
        bushes: 8,
        deadwood: 12,
        water_distance: 3,
    });
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

/// Recompute a logged check's terms from its inputs (7.1's formula).
fn recompute(engine: &SimEngine, c: &FissionCheck) -> (i64, i64, i64, i64) {
    let hazard = engine
        .resources
        .rules
        .society
        .fission
        .and_then(|f| f.hazard)
        .expect("hazard");
    hazard_terms(
        &hazard,
        c.members,
        c.capacity,
        c.food,
        adult_need_per_day(engine.resources.ticks_per_day),
    )
}

#[test]
fn harness_hazard_formula() {
    let hz = baseline_hazard();
    assert_eq!(
        (
            hz.size_lo,
            hz.size_hi,
            hz.scarcity_days,
            hz.max_daily_permille
        ),
        (24, 44, 4, 300)
    );
    assert_eq!(
        (hz.weight_crowd_permille, hz.weight_scarcity_permille),
        (0, 1000)
    );
    // Pure terms (need per member 1.0), by hand: an ordinary case (g = 6/20,
    // c = 1000 so q = 150/200, s = 1 - 30/120, h = 300 x 0.3 x 0.75 = 67),
    // capacity 0 (counts as 1: q = 1000; crowding weighs 0 so h is unchanged),
    // an empty store (s = 1000, h = 90) and size at size_lo (h = 0).
    assert_eq!(hazard_terms(&hz, 30, 30, 30, 1.0), (300, 750, 750, 67));
    assert_eq!(hazard_terms(&hz, 30, 0, 30, 1.0), (300, 1000, 750, 67));
    assert_eq!(hazard_terms(&hz, 30, 30, 0, 1.0), (300, 750, 1000, 90));
    assert_eq!(hazard_terms(&hz, 24, 0, 0, 1.0), (0, 1000, 1000, 0));
    // Constructed settlements: the logged check is the formula on the
    // settlement's members, sustain capacity and own store; with pressure the
    // first check's pressure is its hazard, against the settlement's drawn
    // threshold, and no draw is made. Four cases: no bushes (capacity 0), an
    // empty store, a roomy full store, the smallest eligible village (28).
    let seed = 900_412;
    let check = tick_at(28, 360);
    let law = baseline_rules(seed)
        .society
        .fission
        .and_then(|f| f.pressure)
        .expect("baseline pressure")
        .threshold_table;
    for (bushes, food, members) in [
        (0u32, 100u32, 30u64),
        (40, 0, 30),
        (200, 2000, 30),
        (40, 100, 28),
    ] {
        let mut engine = world(baseline_rules(seed), seed, bushes, food, members, false);
        let capacity = sustain_capacity(&engine.resources, 1).expect("capacity");
        let resident_members = engine.resources.settlements[&1].member_agents.len() as u32;
        run(&mut engine, check, check + 1);
        let logged = engine.resources.fission_checks.clone();
        assert_eq!(logged.len(), 1, "case {bushes}/{food}/{members}");
        let c = logged[0];
        assert_eq!(
            (c.tick, c.settlement, c.members, c.capacity, c.food),
            (check, 1, resident_members, capacity, food)
        );
        assert_eq!((c.g, c.q, c.s, c.h), recompute(&engine, &c));
        assert_eq!(c.draw, 0, "pressure makes no draw");
        assert_eq!(c.pressure, c.h.max(0) as u64, "first day: P = h");
        assert_eq!(c.threshold, fission_threshold(law, seed, 1, 0));
        assert!(c.threshold > 0);
        assert_eq!(c.split, c.pressure > 0 && c.pressure >= c.threshold);
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
fn harness_hazard_rises_with_shortfall() {
    let hz = baseline_hazard();
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
        let mut engine = world(baseline_rules(seed), seed, 40, food, 30, false);
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
    // Baseline allowed seasons [spring, summer, autumn]: spring (day 28),
    // summer (day 35) and autumn (day 14) checks are logged; winter (day 21)
    // never is.
    assert_eq!(
        baseline_rules(seed)
            .society
            .fission
            .expect("fission")
            .allowed_seasons,
        [true, true, true, false]
    );
    for (day, logged) in [(14u64, true), (21, false), (28, true), (35, true)] {
        let mut engine = world(baseline_rules(seed), seed, 0, 0, 30, false);
        run(&mut engine, tick_at(day, 360), tick_at(day, 360) + 1);
        assert_eq!(
            !engine.resources.fission_checks.is_empty(),
            logged,
            "day {day}"
        );
    }
    // No allowed season: no check at all.
    let mut rules = baseline_rules(seed);
    if let Some(fission) = rules.society.fission.as_mut() {
        fission.allowed_seasons = [false; 4];
    }
    let mut engine = world(rules, seed, 0, 0, 30, false);
    run(&mut engine, tick_at(10, 0), tick_at(40, 0));
    assert!(engine.resources.fission_checks.is_empty() && engine.resources.fission_log.is_empty());
}
