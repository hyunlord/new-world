//! Pass 7 Part 2: winters that differ, trips that find wood.
//!
//! Each winter draws a severity (mild, normal, harsh) from a pure hash of the
//! world seed and the year; it sets the night burn and the winter deadwood
//! fall. A deadwood spot takes no more claims than the units it holds, and a
//! gatherer who has arrived keeps its claim until the pickup. Expected numbers
//! come from `WorldRules::baseline(SEED)` (V8-0; was `game_v13`);
//! `harness_winter_severity_table` pins the draw with explicit literals. The
//! baseline also changes each winter night's burn by a drawn -2/0/+2 and each
//! winter day's fall by a drawn -2..=+2 (Pass 8), so the burn and fall
//! assertions add the night's or day's draw to the severity's value.
use std::collections::BTreeSet;

use hecs::Entity;
use sim_core::components::{
    Agent, AgentState, Hunger, Inventory, Position, ResourceKind, SeekTarget, Settlement, Sleep,
    Social, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::{FirewoodRule, WorldRules};
use sim_engine::{RuntimeSystem, SimEngine, SimResources};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::decision::AgentDecisionSystem;
use sim_systems::runtime::resource_regen::{ResourceRegenSystem, REGEN_INTERVAL};
use sim_systems::runtime::settlement::firewood::{
    burn_hours, night_burn_delta, severity_hash, urgent_with, winter_burn_per_hour,
    winter_fall_delta, winter_severity, winter_severity_at, wood_target,
};
use sim_systems::runtime::settlement::FireSystem;

const HOME: (u32, u32) = (20, 20);
/// World seed of the fixture rules (climate 1: severity weights 350 / 400 /
/// 250) and of the fixtures' night and fall draws.
const SEED: u64 = 1;
const AUTUMN_DAY: u64 = 20;
/// Display day 22: a winter day of year 0 (seven-day seasons; winter is days 21–27).
const WINTER_DAY: u64 = 22;

/// Tick at which the display clock (480-tick offset) reads `minute` on display day `day`.
fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

fn rules() -> WorldRules {
    WorldRules::baseline(SEED)
}

fn baseline_rule() -> FirewoodRule {
    rules().society.firewood.expect("baseline firewood rule")
}

/// The baseline with every winter forced to `level` (a one-hot weight vector).
fn rules_at(level: usize) -> WorldRules {
    let mut rules = rules();
    if let Some(rule) = rules.society.firewood.as_mut() {
        let mut weights = [0u16; 3];
        weights[level] = 1000;
        rule.winter_severity_permille = Some(weights);
    }
    rules
}

/// A seed whose year-0 and year-1 winters draw `levels` under the fixture's weights.
fn seed_with(levels: [u8; 2]) -> u64 {
    let rule = baseline_rule();
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    (0..10_000)
        .find(|seed| {
            resources.world_seed = *seed;
            winter_severity(&resources, &rule, 0) == levels[0]
                && winter_severity(&resources, &rule, 1) == levels[1]
        })
        .expect("a seed with these winters")
}

/// One settlement at `HOME` holding `wood`, `adults` idle satisfied members on
/// the formation tile, and deadwood `spots` holding the given amounts.
fn village(
    rules: WorldRules,
    tick: u64,
    wood: u32,
    adults: usize,
    spots: &[((u32, u32), u8)],
) -> (SimEngine, Vec<Entity>) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
    engine.resources.world_seed = SEED;
    engine.resources.current_tick = tick;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = HOME;
    settlement.store(ResourceKind::Wood, wood);
    let mut entities = Vec::new();
    for index in 0..adults {
        let entity = engine.spawn_agent(HOME.0, HOME.1);
        let id = engine.world.get::<&Agent>(entity).expect("agent").id;
        engine
            .world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(0.0, 0.05),
                    Thirst::new(0.0, 0.08),
                    Sleep::new(0.0, 0.03),
                    Social::new(0.0, 0.04),
                    Inventory::default(),
                    MovementRng::new(index as u64 + 1),
                ),
            )
            .expect("components");
        settlement.add_member_agent(id);
        entities.push(entity);
    }
    engine.resources.settlements.insert(1, settlement);
    for &(spot, amount) in spots {
        engine.resources.wood_tiles.insert(spot, amount);
        engine.resources.wood_source_max.insert(spot, 6);
    }
    (engine, entities)
}

fn wood_store(engine: &SimEngine) -> u32 {
    engine.resources.settlements[&1]
        .stockpile
        .get(&ResourceKind::Wood)
        .copied()
        .unwrap_or(0)
}

/// Wood burned by one hourly fire update at `tick`.
fn burned_at(rules: WorldRules, tick: u64) -> u32 {
    let (mut engine, _) = village(rules, tick, 100, 0, &[]);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    100 - wood_store(&engine)
}

fn targets(engine: &SimEngine, entities: &[Entity]) -> Vec<Option<(u32, u32)>> {
    entities
        .iter()
        .map(|e| {
            let state = *engine.world.get::<&AgentState>(*e).expect("state");
            matches!(
                state,
                AgentState::Seeking {
                    target: TargetKind::GatherWood
                }
            )
            .then(|| engine.world.get::<&SeekTarget>(*e).ok().map(|s| s.tile))
            .flatten()
        })
        .collect()
}

#[test]
fn harness_winter_severity_is_a_pure_draw() {
    let rule = baseline_rule();
    let weights = rule
        .winter_severity_permille
        .expect("the baseline draws winter severity");
    assert_eq!(weights, [350, 400, 250]);
    assert_eq!(weights.iter().map(|w| u32::from(*w)).sum::<u32>(), 1000);
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    resources.rules = rules();
    resources.world_seed = 7;
    // Same seed and year, same level; the draw reads no state but the seed.
    let first = winter_severity(&resources, &rule, 3);
    resources.current_tick = 99_999;
    assert_eq!(winter_severity(&resources, &rule, 3), first);
    // Over 10,000 years of one seed each level is within 2 % of its weight.
    let mut counts = [0u32; 3];
    for year in 0..10_000 {
        counts[usize::from(winter_severity(&resources, &rule, year))] += 1;
    }
    for (count, weight) in counts.iter().zip(weights) {
        let share = f64::from(*count) / 10_000.0;
        assert!(
            (share - f64::from(weight) / 1000.0).abs() <= 0.02,
            "{counts:?} against {weights:?}"
        );
    }
    // The 128 probe seed-winters (seeds 0–31, years 0–3) are 128 distinct draws.
    let draws: BTreeSet<u64> = (0..32)
        .flat_map(|seed| (0..4).map(move |year| severity_hash(seed, year)))
        .collect();
    assert_eq!(draws.len(), 128);
    // Only winter hours carry a severity: from tick 0 to 07:00 of display
    // day 21 (the last autumn night ends then) every hour reads -1; from there
    // to 07:00 of display day 28 (the last winter night) every hour a level.
    for tick in (0..tick_at(21, 420)).step_by(60) {
        assert_eq!(winter_severity_at(&resources, tick), -1, "tick {tick}");
    }
    for tick in (tick_at(21, 420)..tick_at(28, 420)).step_by(60) {
        assert!(winter_severity_at(&resources, tick) >= 0, "tick {tick}");
    }
    assert_eq!(winter_severity_at(&resources, tick_at(28, 420)), -1);
}

#[test]
fn harness_winter_severity_table() {
    // u = severity_hash(seed, year) % 1000 against weights 300 / 400 / 300.
    let rule = FirewoodRule {
        winter_severity_permille: Some([300, 400, 300]),
        ..baseline_rule()
    };
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    for (seed, year, u, level) in TABLE {
        assert_eq!(
            severity_hash(seed, year) % 1000,
            u,
            "seed {seed} year {year}"
        );
        resources.world_seed = seed;
        assert_eq!(
            winter_severity(&resources, &rule, year),
            level,
            "seed {seed} year {year}"
        );
    }
}

/// (seed, year, u, level) written out by hand from the formula of section 7.2.
const TABLE: [(u64, u64, u64, u8); 8] = [
    (0, 0, 456, 1),
    (0, 1, 182, 0),
    (1, 0, 134, 0),
    (1, 1, 710, 2),
    (2, 0, 556, 1),
    (2, 1, 605, 1),
    (3, 0, 211, 0),
    (3, 1, 691, 1),
];

#[test]
fn harness_winter_burn_by_severity() {
    let rule = baseline_rule();
    let hours = f64::from(burn_hours(&rule));
    for level in 0..3 {
        let burn = rule.winter_severity_burn_per_hour[level];
        let (engine, _) = village(rules_at(level), tick_at(WINTER_DAY, 1320), 0, 0, &[]);
        let r = &engine.resources;
        // The fire at 22:00 of a winter evening burns the severity's rate
        // changed by that evening's draw (seed 1, display day 22: +0).
        let delta = night_burn_delta(r, &rule_of(r), WINTER_DAY);
        assert_eq!(delta, 0);
        assert_eq!(
            burned_at(rules_at(level), tick_at(WINTER_DAY, 1320)),
            u32::from(burn)
                .saturating_add_signed(i32::from(delta))
                .max(1),
            "level {level}"
        );
        assert_eq!(winter_burn_per_hour(r, &rule_of(r), r.current_tick), burn);
        // The winter target keeps `winter_base_days` of the severity's burn.
        let base = rule.winter_base_days.expect("baseline winter base");
        assert_eq!(
            wood_target(r),
            Some(base * f64::from(burn) * hours),
            "level {level}"
        );
        // Urgency compares wood in hand with tonight's severity burn.
        let tonight = (f64::from(burn) * hours) as u64;
        assert!(urgent_with(r, tonight - 1));
        assert!(!urgent_with(r, tonight));
    }
    // Outside winter the season's burn, whatever the weights say.
    assert_eq!(
        burned_at(rules_at(2), tick_at(AUTUMN_DAY, 1320)),
        u32::from(rule.burn_per_hour[2])
    );
}

fn rule_of(resources: &SimResources) -> FirewoodRule {
    resources.rules.society.firewood.expect("firewood")
}

#[test]
fn harness_winter_fall_by_severity() {
    let rule = baseline_rule();
    for level in 0..3 {
        let spots: Vec<((u32, u32), u8)> = (0..20).map(|i| ((30 + i, 30), 0)).collect();
        let (mut engine, _) = village(rules_at(level), 0, 0, 0, &[]);
        for (spot, _) in &spots {
            engine.resources.wood_source_max.insert(*spot, 6);
        }
        // Twelve pulses: the whole of display day 22 (00:00 to 22:00), whose
        // fall is the severity's changed by the day's draw (floor 0).
        let start = tick_at(WINTER_DAY, 0);
        let mut world = hecs::World::new();
        for pulse in 0..12 {
            engine.resources.current_tick = start + pulse * REGEN_INTERVAL;
            ResourceRegenSystem::new().tick(&mut world, &mut engine.resources);
        }
        let fallen: u32 = engine
            .resources
            .wood_tiles
            .values()
            .map(|v| u32::from(*v))
            .sum();
        let delta = winter_fall_delta(&engine.resources, &rule, WINTER_DAY);
        assert!((-2..=2).contains(&delta), "delta {delta}");
        let want = i64::from(rule.winter_severity_regen_units_per_day[level]) + i64::from(delta);
        assert_eq!(i64::from(fallen), want.max(0), "level {level}");
        // The severities' falls stay ordered: mild 8, normal 6, harsh 4 (+-2).
        assert!(fallen >= 2, "level {level}: {fallen}");
    }
}

#[test]
fn harness_last_night_keeps_its_winter_severity() {
    let rule = baseline_rule();
    // Year 0 harsh, year 1 mild: each winter's last night keeps its own winter.
    let seed = seed_with([2, 0]);
    let rules = rules();
    for (year, level) in [(0u64, 2usize), (1, 0)] {
        // 02:00 of the first spring day closes the last winter evening.
        let tick = tick_at(28 * (year + 1), 120);
        let (mut engine, _) = village(rules, tick, 100, 0, &[]);
        engine.resources.world_seed = seed;
        assert_eq!(winter_severity_at(&engine.resources, tick), level as i64);
        let delta = night_burn_delta(&engine.resources, &rule, 28 * (year + 1) - 1);
        FireSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            100 - wood_store(&engine),
            u32::from(rule.winter_severity_burn_per_hour[level])
                .saturating_add_signed(i32::from(delta))
                .max(1),
            "year {year}"
        );
        // 07:00 that morning: spring.
        assert_eq!(
            winter_severity_at(&engine.resources, tick_at(28 * (year + 1), 420)),
            -1
        );
    }
}

#[test]
fn harness_autumn_prep_ignores_severity() {
    let rule = baseline_rule();
    let hours = f64::from(burn_hours(&rule));
    let prep = rule.winter_prep_days.expect("baseline autumn prep");
    let autumn = rule.base_days * f64::from(rule.burn_per_hour[2]) * hours
        + prep * f64::from(rule.burn_per_hour[3]) * hours;
    for level in 0..3 {
        let (engine, _) = village(rules_at(level), tick_at(AUTUMN_DAY, 480), 0, 0, &[]);
        assert_eq!(
            wood_target(&engine.resources),
            Some(autumn),
            "level {level}"
        );
    }
}

#[test]
fn harness_claims_bounded_by_amount() {
    let rule = baseline_rule();
    assert!(rule.claims_bounded_by_amount);
    let start = u64::from(rule.urgent_start_minute.expect("urgent mode"));
    let cap = 2 * rule.max_gatherers_per_spot as usize;
    let urgent = tick_at(WINTER_DAY, start);
    // A one-unit spot takes one claim even under the urgent cap.
    let (mut engine, entities) = village(rules(), urgent, 0, cap + 1, &[((26, 20), 1)]);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        targets(&engine, &entities)
            .iter()
            .filter(|t| t.is_some())
            .count(),
        1
    );
    // A three-unit spot under the urgent cap takes the cap.
    let (mut engine, entities) = village(rules(), urgent, 0, cap + 1, &[((26, 20), 3)]);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        targets(&engine, &entities)
            .iter()
            .filter(|t| t.is_some())
            .count(),
        cap.min(3)
    );
    // A gatherer who has arrived (`Consuming{GatherWood}`) holds its claim: the
    // other member walks to the farther spot.
    let near = (22, 20);
    let far = (30, 20);
    let (mut engine, entities) = village(rules(), urgent, 0, 2, &[(near, 1), (far, 1)]);
    {
        let mut pos = engine
            .world
            .get::<&mut Position>(entities[0])
            .expect("position");
        (pos.x, pos.y) = near;
    }
    engine
        .world
        .insert_one(
            entities[0],
            AgentState::Consuming {
                target: TargetKind::GatherWood,
            },
        )
        .expect("state");
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(targets(&engine, &entities)[1], Some(far));
    // Re-targeting obeys the same bound: a gatherer whose spot is gone finds
    // the only other one-unit spot claimed and stops.
    let (mut engine, entities) = village(rules(), urgent, 0, 2, &[(far, 1)]);
    for (entity, tile) in [(entities[0], near), (entities[1], far)] {
        engine
            .world
            .insert_one(
                entity,
                AgentState::Seeking {
                    target: TargetKind::GatherWood,
                },
            )
            .expect("state");
        engine
            .world
            .insert_one(entity, SeekTarget { tile })
            .expect("target");
    }
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(targets(&engine, &entities), vec![None, Some(far)]);
    assert_eq!(engine.resources.tallies.wood_retargets, 1);
}
