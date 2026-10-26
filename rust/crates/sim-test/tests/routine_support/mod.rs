use hecs::Entity;
use sim_core::components::{
    Activity, ActivityKind, Agent, AgentState, BuildingKind, Inventory, Lineage, SeekTarget,
    Settlement, TargetKind, Temperament,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::baseline::decision::AgentDecisionSystem;

pub fn fixture() -> SimEngine {
    let mut engine = SimEngine::new(48, 48, MaterialRegistry::new());
    // The live routine (R1): `baseline` carries the Pass 19 routine, with each
    // person's own day end (`end_stagger`).
    engine.resources.rules.agent.routine = WorldRules::baseline(7).agent.routine;
    engine.resources.rules.agent.keyed_draws = true;
    engine.resources.current_tick = 600;
    engine.resources.world_seed = 7;
    let mut home = Settlement::new_with_id(1, 0);
    home.formation_tile = (24, 24);
    engine.resources.settlements.insert(1, home);
    engine
        .resources
        .building_kinds
        .insert((24, 24), BuildingKind::Hearth);
    engine
}

pub fn resident(engine: &mut SimEngine, key: u64) -> Entity {
    let entity = engine.spawn_agent(24, 24);
    engine
        .world
        .insert(
            entity,
            (
                AgentState::Idle,
                Lineage { key, births: 0 },
                Temperament::from_axes([500; 4]),
                Inventory::default(),
                MovementRng::new(key),
            ),
        )
        .expect("resident components");
    let id = engine.world.get::<&Agent>(entity).expect("agent").id;
    engine
        .resources
        .settlements
        .get_mut(&1)
        .expect("settlement")
        .member_agents
        .insert(id);
    entity
}

pub fn decide(engine: &mut SimEngine) {
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
}

pub fn activity(engine: &mut SimEngine, entity: Entity, kind: ActivityKind, arrived: bool) {
    let tick = engine.resources.current_tick;
    engine
        .world
        .insert(
            entity,
            (
                Activity {
                    kind,
                    settlement: 1,
                    place: (24, 23),
                    chosen_tick: tick,
                    arrived_tick: arrived.then_some(tick),
                    until_tick: arrived.then_some(tick + 90),
                },
                if arrived {
                    AgentState::Consuming {
                        target: TargetKind::Activity,
                    }
                } else {
                    AgentState::Seeking {
                        target: TargetKind::Activity,
                    }
                },
                SeekTarget { tile: (24, 23) },
            ),
        )
        .expect("activity components");
}
