//! Pass 6 Part 4: elders at the evening fire.
//!
//! Under `AgingRule::elder_evening_tether` an idle elder's tether inside the
//! evening-gather window is `min(elder_tether_radius, evening tether)`;
//! outside the window it stays `elder_tether_radius`. Numbers come from the rules.
use sim_core::components::{Agent, AgentState, Elder, Position, Settlement};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};

const HOME: (u32, u32) = (20, 20);

/// Tick at which the display clock (480-tick offset) reads `minute` on display day `day`.
fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

/// Idle elders of one settlement at `HOME`, each starting `start` tiles east of it.
fn elders(rules: WorldRules, count: u64, start: u32) -> (SimEngine, Vec<hecs::Entity>) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = HOME;
    let mut entities = Vec::new();
    for seed in 0..count {
        let entity = engine.spawn_agent(HOME.0 + start, HOME.1);
        engine.world.insert(entity, (AgentState::Idle, Elder, MovementRng::new(seed + 11))).expect("elder");
        settlement.add_member_agent(engine.world.get::<&Agent>(entity).expect("agent").id);
        entities.push(entity);
    }
    engine.resources.settlements.insert(1, settlement);
    (engine, entities)
}

/// Run the movement system from `from` for `ticks` ticks; the largest Chebyshev
/// distance from `HOME` observed after each tick.
fn widest(engine: &mut SimEngine, entities: &[hecs::Entity], from: u64, ticks: u64) -> u32 {
    let mut movement = AgentMovementSystem::new();
    let mut widest = 0;
    for tick in from..from + ticks {
        engine.resources.current_tick = tick;
        movement.tick(&mut engine.world, &mut engine.resources);
        for entity in entities {
            let pos = *engine.world.get::<&Position>(*entity).expect("position");
            widest = widest.max(pos.x.abs_diff(HOME.0).max(pos.y.abs_diff(HOME.1)));
        }
    }
    widest
}

#[test]
fn harness_elder_evening_tether_v12() {
    let rules = WorldRules::game_v12();
    let aging = rules.society.aging.expect("v12 aging");
    let evening = rules.agent.evening_gather.expect("v12 evening gather");
    assert!(aging.elder_evening_tether);
    let tight = aging.elder_tether_radius.min(evening.tether_radius);
    assert!(tight < aging.elder_tether_radius, "the evening tether is the tighter one");
    // 19:00: idle elders inside the evening tether stay there for 120 ticks.
    let (mut engine, entities) = elders(rules, 6, tight);
    assert_eq!(widest(&mut engine, &entities, tick_at(3, 1140), 120), tight);
    // One standing at the elder tether at 19:00 is drawn in to the evening tether.
    let (mut engine, entities) = elders(rules, 6, aging.elder_tether_radius);
    widest(&mut engine, &entities, tick_at(3, 1140), 60);
    assert!(widest(&mut engine, &entities, tick_at(3, 1200), 60) <= tight);
    // 12:00: the elder tether alone; elders may stand at it, never beyond.
    let (mut engine, entities) = elders(rules, 6, tight);
    assert_eq!(widest(&mut engine, &entities, tick_at(3, 720), 300), aging.elder_tether_radius);
}

#[test]
fn harness_elder_tether_v11_unchanged() {
    let rules = WorldRules::game_v11();
    let aging = rules.society.aging.expect("v11 aging");
    assert!(!aging.elder_evening_tether);
    // In v11 the elder tether holds at 19:00 too: elders may stand at 4.
    let (mut engine, entities) = elders(rules, 6, aging.elder_tether_radius - 1);
    assert_eq!(widest(&mut engine, &entities, tick_at(3, 1140), 120), aging.elder_tether_radius);
    assert_eq!(WorldRules::game_v10().society.aging.map(|rule| rule.elder_evening_tether), Some(false));
}

#[test]
fn harness_elder_tether_returns_after_the_evening() {
    // At 21:00 the evening window closes and the elder tether alone holds again.
    let rules = WorldRules::game_v12();
    let aging = rules.society.aging.expect("v12 aging");
    let evening = rules.agent.evening_gather.expect("evening");
    let (mut engine, entities) = elders(rules, 6, aging.elder_tether_radius.min(evening.tether_radius));
    let after = tick_at(3, u64::from(evening.end_minute));
    assert_eq!(widest(&mut engine, &entities, after, 300), aging.elder_tether_radius);
}
