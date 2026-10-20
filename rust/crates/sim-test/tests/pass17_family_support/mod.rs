use hecs::Entity;
use sim_core::components::{
    Agent, AgentState, BodyHealth, Hunger, Kin, Lineage, Settlement, Sex, Temperament, Thirst,
};
use sim_core::{rules::WorldRules, MaterialRegistry};
use sim_engine::SimEngine;

pub fn fixture() -> SimEngine {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.world_seed = 7;
    // The live family and temperament rules (R1): `baseline` carries the
    // Pass 18 family rule (pair at familiarity 0.200, elder partners follow).
    let live = WorldRules::baseline(7);
    engine.resources.rules.society.family = live.society.family;
    engine.resources.rules.agent.keyed_draws = true;
    engine.resources.rules.agent.temperament = live.agent.temperament;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (20, 20);
    engine.resources.settlements.insert(1, settlement);
    engine
}

pub fn id(engine: &SimEngine, entity: Entity) -> u64 {
    engine.world.get::<&Agent>(entity).expect("agent").id
}

pub fn person(engine: &mut SimEngine, female: bool) -> Entity {
    let entity = engine.spawn_agent(20, 20);
    let id = id(engine, entity);
    engine
        .world
        .insert(
            entity,
            (
                AgentState::Idle,
                Kin::default(),
                Sex { female },
                BodyHealth::new(),
                Lineage {
                    key: id + 10,
                    births: 0,
                },
                Temperament::from_axes([500; 4]),
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
            ),
        )
        .expect("family resident");
    engine
        .resources
        .settlements
        .get_mut(&1)
        .expect("settlement")
        .member_agents
        .insert(id);
    entity
}

pub fn pair(engine: &mut SimEngine, mother: Entity, father: Entity) {
    let (mother_id, father_id) = (id(engine, mother), id(engine, father));
    engine
        .world
        .get::<&mut Kin>(mother)
        .expect("mother")
        .partner = Some(father_id);
    engine
        .world
        .get::<&mut Kin>(father)
        .expect("father")
        .partner = Some(mother_id);
}
