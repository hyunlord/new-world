//! Pass 6 Part 3: urgent mornings.
//!
//! When tonight's burn is not in hand a settlement is in urgent mode: its
//! members' firewood window opens at `urgent_start_minute` and two gatherers
//! may share a spot. Expected numbers come from `WorldRules::game_v12()`.
use hecs::Entity;
use sim_core::components::{
    Agent, AgentState, Hunger, Inventory, ResourceKind, SeekTarget, Settlement, Sleep, Social, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::{FirewoodRule, WorldRules};
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::decision::AgentDecisionSystem;
use sim_systems::runtime::settlement::firewood::{burn_per_day, is_urgent, night_season, wood_target};
use sim_test::probe::fingerprint;
use sim_test::scenes::game_v11_scene;

const HOME: (u32, u32) = (20, 20);
/// Display day 22: a winter day (seven-day seasons; winter is days 21–27).
const WINTER_DAY: u64 = 22;

/// Tick at which the display clock (480-tick offset) reads `minute` on display day `day`.
fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

fn v12_rule() -> FirewoodRule {
    WorldRules::game_v12().society.firewood.expect("v12 keeps the firewood rule")
}

/// Bare world under `rules`: one settlement at `HOME` holding `wood`,
/// `adults` idle satisfied members on the formation tile, full deadwood spots.
fn fixture(rules: WorldRules, adults: usize, spots: &[(u32, u32)], tick: u64, wood: u32) -> (SimEngine, Vec<Entity>) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
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
    for &spot in spots {
        engine.resources.wood_tiles.insert(spot, 6);
        engine.resources.wood_source_max.insert(spot, 6);
    }
    (engine, entities)
}

fn wood_targets(engine: &SimEngine, entities: &[Entity]) -> Vec<Option<(u32, u32)>> {
    entities
        .iter()
        .map(|e| {
            let state = *engine.world.get::<&AgentState>(*e).expect("state");
            matches!(state, AgentState::Seeking { target: TargetKind::GatherWood })
                .then(|| engine.world.get::<&SeekTarget>(*e).ok().map(|s| s.tile))
                .flatten()
        })
        .collect()
}

/// Tonight's burn on `day` at `minute` under v12.
fn tonight(day: u64, minute: u64) -> u32 {
    let (engine, _) = fixture(WorldRules::game_v12(), 0, &[], tick_at(day, minute), 0);
    let rule = v12_rule();
    burn_per_day(&rule, night_season(&engine.resources, &rule)) as u32
}

#[test]
fn harness_urgent_window_opens_at_dawn() {
    let rule = v12_rule();
    let start = u64::from(rule.urgent_start_minute.expect("v12 urgent mode"));
    let gather = u64::from(rule.gather_start_minute);
    assert!(start < gather, "urgent mornings start before the normal window");
    let burn = tonight(WINTER_DAY, start);
    // Tonight's burn is not in hand: an adult leaves for wood at the urgent start.
    let (mut engine, entities) = fixture(WorldRules::game_v12(), 1, &[(26, 20)], tick_at(WINTER_DAY, start), burn - 1);
    assert!(is_urgent(&engine.world, &engine.resources, 1));
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_targets(&engine, &entities), vec![Some((26, 20))]);
    assert_eq!((engine.resources.tallies.wood_trips, engine.resources.tallies.urgent_wood_trips), (1, 1));
    // One minute before the urgent start nobody goes.
    let (mut engine, entities) = fixture(WorldRules::game_v12(), 1, &[(26, 20)], tick_at(WINTER_DAY, start - 1), 0);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_targets(&engine, &entities), vec![None]);
    // Tonight's burn in hand (still below the target): not before the normal window.
    let target = wood_target(&engine.resources).expect("target");
    assert!(f64::from(burn) < target);
    for minute in [start, gather - 1] {
        let (mut engine, entities) = fixture(WorldRules::game_v12(), 1, &[(26, 20)], tick_at(WINTER_DAY, minute), burn);
        assert!(!is_urgent(&engine.world, &engine.resources, 1));
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(wood_targets(&engine, &entities), vec![None], "minute {minute}");
    }
    let (mut engine, entities) = fixture(WorldRules::game_v12(), 1, &[(26, 20)], tick_at(WINTER_DAY, gather), burn);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_targets(&engine, &entities), vec![Some((26, 20))]);
    assert_eq!((engine.resources.tallies.wood_trips, engine.resources.tallies.urgent_wood_trips), (1, 0));
    // The urgent window closes with the normal one.
    let end = u64::from(rule.gather_end_minute);
    let (mut engine, entities) = fixture(WorldRules::game_v12(), 1, &[(26, 20)], tick_at(WINTER_DAY, end), 0);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_targets(&engine, &entities), vec![None]);
}

#[test]
fn harness_urgent_two_per_spot() {
    let rule = v12_rule();
    let start = u64::from(rule.urgent_start_minute.expect("v12 urgent mode"));
    let per_spot = rule.max_gatherers_per_spot as usize;
    let adults = 2 * per_spot + 1;
    // Urgent: twice the claim cap on the only spot.
    let (mut engine, entities) = fixture(WorldRules::game_v12(), adults, &[(26, 20)], tick_at(WINTER_DAY, start), 0);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    let chosen = wood_targets(&engine, &entities);
    assert_eq!(chosen.iter().filter(|t| **t == Some((26, 20))).count(), 2 * per_spot, "{chosen:?}");
    assert_eq!(engine.resources.tallies.urgent_wood_trips, 2 * per_spot as u64);
    // Not urgent (10:00, tonight's burn in hand): the Pass 5 cap.
    let burn = tonight(WINTER_DAY, u64::from(rule.gather_start_minute));
    let (mut engine, entities) =
        fixture(WorldRules::game_v12(), adults, &[(26, 20)], tick_at(WINTER_DAY, u64::from(rule.gather_start_minute)), burn);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    let chosen = wood_targets(&engine, &entities);
    assert_eq!(chosen.iter().filter(|t| **t == Some((26, 20))).count(), per_spot, "{chosen:?}");
    // Re-targeting: gatherers whose spot emptied share the remaining spot up to the doubled cap.
    for (wood, minute, want) in [(0, start, 2 * per_spot), (burn, u64::from(rule.gather_start_minute), per_spot)] {
        let (mut engine, entities) = fixture(WorldRules::game_v12(), adults, &[(30, 20)], tick_at(WINTER_DAY, minute), wood);
        for entity in &entities {
            engine.world.insert_one(*entity, AgentState::Seeking { target: TargetKind::GatherWood }).expect("seek");
            engine.world.insert_one(*entity, SeekTarget::new((40, 40))).expect("empty spot");
        }
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        let chosen = wood_targets(&engine, &entities);
        assert_eq!(chosen.iter().filter(|t| **t == Some((30, 20))).count(), want, "wood {wood}: {chosen:?}");
        assert_eq!(chosen.iter().filter(|t| t.is_none()).count(), adults - want, "the rest idle: {chosen:?}");
        assert_eq!(engine.resources.tallies.wood_trips, 0, "re-targeting starts no trip");
    }
}

#[test]
fn harness_urgent_ends_when_wood_is_in_hand() {
    let rule = v12_rule();
    let start = u64::from(rule.urgent_start_minute.expect("v12 urgent mode"));
    let burn = tonight(WINTER_DAY, start);
    let (mut engine, entities) = fixture(WorldRules::game_v12(), 1, &[], tick_at(WINTER_DAY, start), burn - 2);
    assert!(is_urgent(&engine.world, &engine.resources, 1));
    // A gatherer walking home with one unit is wood in hand too.
    engine.world.get::<&mut Inventory>(entities[0]).expect("inv").add(ResourceKind::Wood, 1);
    assert!(is_urgent(&engine.world, &engine.resources, 1));
    engine.resources.settlements.get_mut(&1).expect("village").store(ResourceKind::Wood, 1);
    assert!(!is_urgent(&engine.world, &engine.resources, 1), "tonight's burn is in hand");
    // With the flag cleared, a new trip waits for the normal window.
    engine.resources.wood_tiles.insert((26, 20), 6);
    engine.resources.wood_source_max.insert((26, 20), 6);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_targets(&engine, &entities), vec![None]);
    assert_eq!(engine.resources.tallies.urgent_wood_trips, 0);
    // The night ahead decides: after midnight the evening's season still counts.
    let (engine, _) = fixture(WorldRules::game_v12(), 0, &[], tick_at(WINTER_DAY + 6, 120), 0);
    assert_eq!(night_season(&engine.resources, &rule), 3, "the last winter night is a winter night");
}

#[test]
fn harness_urgent_v11_unchanged() {
    let v11 = WorldRules::game_v11().society.firewood.expect("v11");
    assert_eq!(v11.urgent_start_minute, None);
    // No wood at all at 07:00 in v11: nobody leaves before 10:00 and nothing is urgent.
    let (mut engine, entities) = fixture(WorldRules::game_v11(), 2, &[(26, 20)], tick_at(WINTER_DAY, 420), 0);
    assert!(!is_urgent(&engine.world, &engine.resources, 1));
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_targets(&engine, &entities), vec![None, None]);
    // At 10:00 the Pass 5 cap of one gatherer per spot holds.
    let (mut engine, entities) = fixture(WorldRules::game_v11(), 2, &[(26, 20)], tick_at(WINTER_DAY, 600), 0);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_targets(&engine, &entities).iter().filter(|t| t.is_some()).count(), 1);
    assert_eq!(engine.resources.tallies.urgent_wood_trips, 0);
    // The frozen v11 day: fingerprint of seed 0 after 1,440 ticks (Part 0).
    let mut engine = game_v11_scene(0);
    for _ in 0..1440 {
        engine.tick();
    }
    assert_eq!(fingerprint(&engine), "270000e767fea504");
    assert_eq!(engine.resources.tallies.urgent_wood_trips, 0);
}
