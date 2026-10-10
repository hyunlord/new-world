use sim_core::components::{Agent, AgentState, Child, Hunger, Position, Sleep, TargetKind, Thirst};
use sim_core::{material::MaterialRegistry, rules::WorldRules};
use sim_engine::{RuntimeSystem, SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
use sim_systems::runtime::decision::{agent_decision::minute_in_window, AgentDecisionSystem};
use sim_test::scenes::game_v9_scene;

fn advance_to(engine: &mut SimEngine, tick: u64) {
    while engine.current_tick() < tick {
        engine.tick();
    }
}

#[test]
fn harness_rhythm_minute_window_wraps_midnight() {
    // Given the v9 night window, when each boundary is queried, then it is half-open.
    for (minute, expected) in [
        (1259, false),
        (1260, true),
        (1439, true),
        (0, true),
        (359, true),
        (360, false),
    ] {
        assert_eq!(minute_in_window(minute, 1260, 360), expected);
    }
}

#[test]
fn harness_rhythm_villagers_sleep_in_beds_at_night() {
    // Given the production v9 village.
    let mut engine = game_v9_scene(1);
    for day in [2, 3] {
        // When the displayed clock reaches 02:00 on each requested day.
        let tick = (day - 1) * 1440 + 120 - engine.resources.rules.clock_offset_ticks;
        advance_to(&mut engine, tick);
        // Then at least seventy percent of living agents sleep on their own beds.
        let mut alive = 0;
        let mut sleeping = 0;
        for (_, (agent, pos, state)) in engine
            .world
            .query::<(&Agent, &Position, &AgentState)>()
            .iter()
        {
            alive += 1;
            if matches!(
                state,
                AgentState::Consuming {
                    target: TargetKind::Sleep
                }
            ) && engine.resources.bed_of.get(&agent.id) == Some(&(pos.x, pos.y))
            {
                sleeping += 1;
            }
        }
        assert!(
            sleeping * 10 >= alive * 7,
            "day {day}: sleeping {sleeping}/{alive}"
        );
    }
}

#[test]
fn harness_rhythm_evening_near_hearth() {
    // Given the production v9 village.
    let mut engine = game_v9_scene(1);
    for day in [2, 3] {
        // When the displayed clock reaches 19:30.
        let tick = (day - 1) * 1440 + 1170 - engine.resources.rules.clock_offset_ticks;
        advance_to(&mut engine, tick);
        // Then at least half of idle adults are within the evening tether.
        let mut idle = 0;
        let mut near = 0;
        for (_, (agent, pos, state, child)) in engine
            .world
            .query::<(&Agent, &Position, &AgentState, Option<&Child>)>()
            .iter()
        {
            if child.is_some() || !matches!(state, AgentState::Idle) {
                continue;
            }
            idle += 1;
            if engine.resources.settlements.values().any(|settlement| {
                settlement.member_agents.contains(&agent.id)
                    && pos
                        .x
                        .abs_diff(settlement.formation_tile.0)
                        .max(pos.y.abs_diff(settlement.formation_tile.1))
                        <= 3
            }) {
                near += 1;
            }
        }
        assert!(
            idle > 0 && near * 2 >= idle,
            "day {day}: near {near}/{idle}"
        );
    }
}

#[test]
fn harness_rhythm_wake_for_critical_thirst() {
    // Given a sleeping villager with moderate hunger and independently critical thirst.
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules = WorldRules::game_v9();
    engine.resources.rules.clock_offset_ticks = 1260;
    engine.resources.rules.agent.move_interval_ticks = 1;
    let entity = engine.spawn_agent(2, 2);
    engine
        .world
        .insert(
            entity,
            (
                AgentState::Consuming {
                    target: TargetKind::Sleep,
                },
                Hunger::new(60.0, 0.0),
                Thirst::new(80.0, 0.0),
                Sleep::new(10.0, 0.0),
                MovementRng::new(1),
            ),
        )
        .unwrap();
    engine
        .resources
        .food_tiles
        .insert((2, 2), RESOURCE_SOURCE_INFINITE);
    engine
        .resources
        .water_tiles
        .insert((4, 2), RESOURCE_SOURCE_INFINITE);
    engine
        .resources
        .sleep_tiles
        .insert((2, 2), RESOURCE_SOURCE_INFINITE);
    engine.register_system(Box::new(AgentMovementSystem::new()));
    engine.register_system(Box::new(AgentDecisionSystem::new()));
    // When two simulation ticks run.
    advance_to(&mut engine, 2);
    // Then thirst wakes the sleeper despite the higher-priority, noncritical hunger.
    assert!(!matches!(
        *engine.world.get::<&AgentState>(entity).unwrap(),
        AgentState::Consuming {
            target: TargetKind::Sleep
        }
    ));
    advance_to(&mut engine, 600);
    assert!(engine.world.get::<&Thirst>(entity).unwrap().value < 80.0);
}

#[test]
fn harness_rhythm_children_grow_up() {
    // Given the first real newborn in the production village.
    let mut engine = game_v9_scene(1);
    let (entity, born_tick) = loop {
        engine.tick();
        let newborn = engine
            .world
            .query::<&Child>()
            .iter()
            .map(|(entity, child)| (entity, child.born_tick))
            .next();
        if let Some(newborn) = newborn {
            break newborn;
        }
        assert!(
            engine.current_tick() < 3 * 1440,
            "no newborn within three days"
        );
    };
    let rule = engine.resources.rules.society.life_stage.unwrap();
    assert_eq!(
        engine.world.get::<&Hunger>(entity).unwrap().growth_rate,
        rule.hunger_rate
    );
    assert_eq!(
        engine.world.get::<&Thirst>(entity).unwrap().growth_rate,
        rule.thirst_rate
    );
    assert_eq!(
        engine.world.get::<&Sleep>(entity).unwrap().growth_rate,
        rule.sleep_rate
    );
    // When the child lives through its configured childhood.
    while engine.world.get::<&Child>(entity).is_ok() {
        let state = *engine
            .world
            .get::<&AgentState>(entity)
            .expect("child remains alive");
        assert!(!matches!(
            state,
            AgentState::Seeking {
                target: TargetKind::ConstructionSite | TargetKind::GatherFood
            } | AgentState::Consuming {
                target: TargetKind::ConstructionSite | TargetKind::GatherFood
            }
        ));
        engine.tick();
        assert!(engine.current_tick() <= born_tick + rule.child_ticks + 61);
    }
    // Then maturation occurs on schedule and restores all founder need rates.
    assert!(engine.current_tick() >= born_tick + rule.child_ticks);
    assert_eq!(
        engine.world.get::<&Hunger>(entity).unwrap().growth_rate,
        0.05
    );
    assert_eq!(
        engine.world.get::<&Thirst>(entity).unwrap().growth_rate,
        0.08
    );
    assert_eq!(
        engine.world.get::<&Sleep>(entity).unwrap().growth_rate,
        0.03
    );
}

#[test]
fn harness_rhythm_bed_seek_ignores_intermediate_shelters() {
    // Given an assigned bed beyond another shelter on the same straight path.
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules = WorldRules::game_v9();
    engine.resources.current_tick = 780;
    let entity = engine.spawn_agent(2, 2);
    let id = engine.world.get::<&Agent>(entity).unwrap().id;
    engine
        .world
        .insert(
            entity,
            (AgentState::Idle, Sleep::new(10.0, 0.0), MovementRng::new(1)),
        )
        .unwrap();
    engine
        .resources
        .sleep_tiles
        .insert((2, 2), RESOURCE_SOURCE_INFINITE);
    engine
        .resources
        .sleep_tiles
        .insert((4, 2), RESOURCE_SOURCE_INFINITE);
    engine.resources.bed_of.insert(id, (4, 2));
    let mut decision = AgentDecisionSystem::new();
    // When seeking begins while already standing on the wrong shelter.
    decision.tick(&mut engine.world, &mut engine.resources);
    decision.tick(&mut engine.world, &mut engine.resources);
    // Then the agent must keep seeking its actual bed.
    assert!(matches!(
        *engine.world.get::<&AgentState>(entity).unwrap(),
        AgentState::Seeking {
            target: TargetKind::Sleep
        }
    ));
    assert_eq!(
        engine
            .world
            .get::<&sim_core::components::SeekTarget>(entity)
            .unwrap()
            .tile,
        (4, 2)
    );
    engine.world.get::<&mut Position>(entity).unwrap().x = 4;
    decision.tick(&mut engine.world, &mut engine.resources);
    decision.tick(&mut engine.world, &mut engine.resources);
    assert!(matches!(
        *engine.world.get::<&AgentState>(entity).unwrap(),
        AgentState::Consuming {
            target: TargetKind::Sleep
        }
    ));
    assert!((engine.world.get::<&Sleep>(entity).unwrap().fatigue - 9.88).abs() < 1e-10);
}
