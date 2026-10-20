//! Pass 5 Part 4: critical needs interrupt gathering trips.
use hecs::Entity;
use sim_core::components::{
    Agent, AgentState, Hunger, Inventory, ResourceKind, SeekTarget, Settlement, Sleep, Social,
    TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::decision::AgentDecisionSystem;

/// World seed of the fixture rules; preemption does not depend on it.
const SEED: u64 = 1;

/// One member of a settlement at (20, 20) standing 15 tiles out at (35, 20),
/// already seeking `target` at `spot`; a bush at (36, 22), water on the river
/// bank at (35, 30); clock at 12:00 on display day 1 (inside the forage and
/// firewood windows).
fn seeker(
    rules: WorldRules,
    target: TargetKind,
    spot: (u32, u32),
    hunger: f32,
    thirst: f64,
) -> (SimEngine, Entity) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
    engine.resources.current_tick = 1440 + 720 - 480;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (20, 20);
    let entity = engine.spawn_agent(35, 20);
    let id = engine.world.get::<&Agent>(entity).expect("agent").id;
    engine
        .world
        .insert(
            entity,
            (
                AgentState::Seeking { target },
                SeekTarget { tile: spot },
                Hunger::new(hunger, 0.05),
                Thirst::new(thirst, 0.08),
                Sleep::new(0.0, 0.03),
                Social::new(0.0, 0.04),
                Inventory::default(),
                MovementRng::new(1),
            ),
        )
        .expect("components");
    settlement.add_member_agent(id);
    engine.resources.settlements.insert(1, settlement);
    engine.resources.wood_tiles.insert((38, 20), 6);
    engine.resources.wood_source_max.insert((38, 20), 6);
    // Baseline food availability is local (`local_food_availability`): the
    // bush lies on the catchment edge (Chebyshev 16 from the hearth); v11's
    // (38, 22) lies outside it, where no member forages.
    engine.resources.set_food_tile(36, 22, 4);
    engine.resources.set_food_source_max(36, 22, 4);
    engine.resources.set_water_tile(35, 30, 255);
    (engine, entity)
}

fn state(engine: &SimEngine, entity: Entity) -> AgentState {
    *engine.world.get::<&AgentState>(entity).expect("state")
}

#[test]
fn harness_gather_preempted_by_thirst() {
    let (mut engine, gatherer) = seeker(
        WorldRules::baseline(SEED),
        TargetKind::GatherWood,
        (38, 20),
        0.0,
        80.0,
    );
    let mut decision = AgentDecisionSystem::new();
    decision.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(state(&engine, gatherer), AgentState::Idle);
    assert!(
        engine.world.get::<&SeekTarget>(gatherer).is_err(),
        "the wood target is dropped"
    );
    assert_eq!(engine.resources.tallies.gather_preempted, 1);
    engine.resources.current_tick += 1;
    decision.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        state(&engine, gatherer),
        AgentState::Seeking {
            target: TargetKind::Water
        }
    );
    assert_eq!(
        engine
            .world
            .get::<&SeekTarget>(gatherer)
            .expect("water target")
            .tile,
        (35, 30)
    );

    // A forager is interrupted by critical hunger the same way (75 inclusive).
    let (mut engine, forager) = seeker(
        WorldRules::baseline(SEED),
        TargetKind::GatherFood,
        (36, 22),
        75.0,
        0.0,
    );
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(state(&engine, forager), AgentState::Idle);
    assert_eq!(engine.resources.tallies.gather_preempted, 1);
    engine.resources.current_tick += 1;
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        state(&engine, forager),
        AgentState::Seeking {
            target: TargetKind::Food
        }
    );

    // Below the critical level the trip continues.
    let (mut engine, gatherer) = seeker(
        WorldRules::baseline(SEED),
        TargetKind::GatherWood,
        (38, 20),
        60.0,
        74.0,
    );
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        state(&engine, gatherer),
        AgentState::Seeking {
            target: TargetKind::GatherWood
        }
    );
    assert_eq!(engine.resources.tallies.gather_preempted, 0);
    let carried = engine
        .world
        .get::<&Inventory>(gatherer)
        .expect("inv")
        .get(ResourceKind::Wood);
    assert_eq!(carried, 0);
}

#[test]
fn harness_preempted_gatherer_is_not_sent_out_again() {
    // Review fix: every bush empty (no reachable food need) and wood below
    // target; after the preemption the critical member starts no new trip.
    let (mut engine, gatherer) = seeker(
        WorldRules::baseline(SEED),
        TargetKind::GatherWood,
        (38, 20),
        80.0,
        0.0,
    );
    engine.resources.food_tiles.clear();
    let mut decision = AgentDecisionSystem::new();
    for _ in 0..20 {
        decision.tick(&mut engine.world, &mut engine.resources);
        engine.resources.current_tick += 1;
        assert_eq!(state(&engine, gatherer), AgentState::Idle);
    }
    assert_eq!(
        engine.resources.tallies.gather_preempted, 1,
        "preempted once, never re-sent"
    );
    assert_eq!(engine.resources.tallies.wood_trips, 0);
}
