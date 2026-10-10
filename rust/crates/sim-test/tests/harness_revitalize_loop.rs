use sim_core::components::{
    Agent, AgentState, BuildingKind, Hunger, Position, ResourceKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::{BirthRule, WorldRules};
use sim_engine::{RuntimeSystem, SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::runtime::settlement::SettlementSystem;
use sim_test::scenes::game_scene;
use std::collections::BTreeMap;

fn home_distance(engine: &SimEngine) -> f64 {
    let mut homes = BTreeMap::new();
    let mut settlements: Vec<_> = engine.resources.settlements.iter().collect();
    settlements.sort_by_key(|(id, _)| **id);
    for (_, settlement) in settlements {
        for id in &settlement.member_agents {
            homes.entry(*id).or_insert(settlement.formation_tile);
        }
    }
    let distances: Vec<_> = engine
        .world
        .query::<(&Agent, &Position)>()
        .iter()
        .filter_map(|(_, (a, p))| {
            homes
                .get(&a.id)
                .map(|h| p.x.abs_diff(h.0).max(p.y.abs_diff(h.1)) as f64)
        })
        .collect();
    assert!(!distances.is_empty());
    distances.iter().sum::<f64>() / distances.len() as f64
}

#[test]
fn harness_population_game_world_survives_3_days() {
    for seed in 1..=3 {
        let mut engine = game_scene(seed);
        let (mut samples, mut breach) = (0_u64, 0_u64);
        for _ in 0..4320 {
            engine.tick();
            for (_, (state, hunger, thirst)) in engine
                .world
                .query::<(&AgentState, &Hunger, &Thirst)>()
                .iter()
            {
                samples += 1;
                if matches!(state, AgentState::Idle) && (hunger.value > 50.0 || thirst.value > 50.0)
                {
                    breach += 1;
                }
            }
        }
        assert_eq!(
            engine.resources.tallies.deaths_dehydration, 0,
            "seed {seed}"
        );
        assert!(
            engine.world.query::<&Agent>().iter().count() >= 20,
            "seed {seed}"
        );
        assert!(
            breach as f64 / samples as f64 <= 0.10,
            "seed {seed}: {breach}/{samples}"
        );
    }
}

#[test]
fn harness_building_shelters_planned_and_completed() {
    let mut engine = game_scene(1);
    let initial = engine.resources.building_kinds.clone();
    let mut planned = std::collections::BTreeSet::new();
    for _ in 0..4320 {
        engine.tick();
        planned.extend(engine.resources.site_owner.keys().copied());
    }
    let completed: Vec<_> = engine
        .resources
        .building_kinds
        .iter()
        .filter(|(tile, kind)| **kind == BuildingKind::Shelter && !initial.contains_key(tile))
        .collect();
    assert!(!completed.is_empty());
    for (tile, _) in completed {
        assert!(planned.contains(tile));
        assert_eq!(
            engine.resources.sleep_tiles.get(tile),
            Some(&RESOURCE_SOURCE_INFINITE)
        );
        assert!(!engine.resources.site_owner.contains_key(tile));
    }
    let sites: std::collections::BTreeSet<_> = engine
        .world
        .query::<&sim_core::components::ConstructionSite>()
        .iter()
        .map(|(_, s)| (s.position.x, s.position.y))
        .collect();
    assert!(engine
        .resources
        .site_owner
        .keys()
        .all(|tile| sites.contains(tile)));
}

#[test]
fn harness_population_births_gated_by_food_and_housing() {
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules = WorldRules::game_v8();
    engine.resources.rules.society.birth = BirthRule::Provisioned {
        cooldown_ticks: 10,
        food_cost: 6,
        max_mean_hunger: 60.0,
        max_mean_thirst: 60.0,
    };
    engine
        .resources
        .rules
        .society
        .shelter_planning
        .as_mut()
        .expect("planning")
        .residents_per_shelter = 4;
    for _ in 0..3 {
        let agent = engine.spawn_agent(10, 10);
        engine
            .world
            .insert(agent, (Hunger::new(0.0, 0.0), Thirst::new(0.0, 0.0)))
            .expect("components");
    }
    engine.resources.building_registry.insert(100, (10, 10));
    engine.resources.building_registry.insert(101, (11, 10));
    engine
        .resources
        .building_kinds
        .insert((11, 10), BuildingKind::Shelter);
    let mut system = SettlementSystem::new();
    for tick in 0..=20 {
        engine.resources.current_tick = tick;
        system.tick(&mut engine.world, &mut engine.resources);
    }
    assert_eq!(engine.resources.tallies.births, 0);
    assert_eq!(engine.resources.settlements.len(), 1);
    let id = *engine
        .resources
        .settlements
        .keys()
        .next()
        .expect("settlement");
    engine
        .resources
        .settlements
        .get_mut(&id)
        .expect("settlement")
        .store(ResourceKind::Food, 12);
    engine.resources.current_tick = 21;
    system.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(engine.resources.tallies.births, 1);
    assert_eq!(
        engine.resources.settlements[&id].stockpile[&ResourceKind::Food],
        6
    );
    for tick in 22..=42 {
        engine.resources.current_tick = tick;
        system.tick(&mut engine.world, &mut engine.resources);
    }
    assert_eq!(engine.resources.tallies.births, 1, "housing is now full");
    assert_eq!(
        engine.resources.settlements[&id].stockpile[&ResourceKind::Food],
        6
    );
}

#[test]
fn harness_population_rules_default_is_legacy() {
    let rules = WorldRules::default();
    assert_eq!(rules.agent.move_interval_ticks, 1);
    assert_eq!(rules.agent.idle_pause_chance, 0.0);
    assert_eq!(rules.agent.idle_tether_radius, None);
    assert_eq!(rules.society.birth, BirthRule::Legacy);
    assert_eq!(rules.society.shelter_planning, None);
}

#[test]
fn harness_social_idle_agents_stay_near_home() {
    let mut engine = game_scene(1);
    for _ in 0..2880 {
        engine.tick();
    }
    assert!(home_distance(&engine) <= 6.0);
}

#[test]
fn harness_building_assigned_builder_walks_and_clears_missing_site() {
    use sim_core::components::{
        building_kind::shelter_blueprint, ConstructionSite, SeekTarget, Settlement, TargetKind,
    };
    use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
    use sim_systems::runtime::decision::AgentDecisionSystem;
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules = WorldRules::game_v8();
    engine.resources.rules.agent.move_interval_ticks = 1;
    engine.resources.rules.agent.idle_pause_chance = 1.0;
    let entity = engine.spawn_agent(10, 10);
    engine
        .world
        .insert(
            entity,
            (
                AgentState::Idle,
                MovementRng::new(1),
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
            ),
        )
        .expect("components");
    let id = engine.world.get::<&Agent>(entity).expect("agent").id;
    let mut settlement = Settlement::new_with_id(0, 0);
    settlement.formation_tile = (10, 10);
    settlement.add_member_agent(id);
    engine.resources.settlements.insert(0, settlement);
    let site = engine.world.spawn((ConstructionSite::new(
        shelter_blueprint(240),
        Position::new(14, 10),
    ),));
    engine.resources.site_owner.insert((14, 10), 0);
    let mut decisions = AgentDecisionSystem::new();
    let mut movement = AgentMovementSystem::new();
    decisions.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        *engine.world.get::<&AgentState>(entity).expect("state"),
        AgentState::Seeking {
            target: TargetKind::ConstructionSite
        }
    );
    assert_eq!(
        engine
            .world
            .get::<&SeekTarget>(entity)
            .expect("target")
            .tile,
        (14, 10)
    );
    for _ in 0..3 {
        movement.tick(&mut engine.world, &mut engine.resources);
        decisions.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Seeking {
                target: TargetKind::ConstructionSite
            }
        );
    }
    assert_eq!(
        engine.world.get::<&Position>(entity).expect("position").x,
        13
    );
    engine.world.despawn(site).expect("site");
    decisions.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        *engine.world.get::<&AgentState>(entity).expect("state"),
        AgentState::Idle
    );
    assert!(engine.world.get::<&SeekTarget>(entity).is_err());
}

#[test]
fn harness_population_movement_only_steps_on_configured_tick_multiples() {
    use sim_core::components::{SeekTarget, TargetKind};
    use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules.agent.move_interval_ticks = 6;
    let rng = MovementRng::new(123);
    // Identity remains optional: the legacy movement surface also handles
    // entities carrying Position/MovementRng without the Agent component.
    let entity = engine.world.spawn((
        Position::new(3, 4),
        rng,
        AgentState::Seeking {
            target: TargetKind::Water,
        },
        SeekTarget::new((15, 4)),
    ));
    let mut movement = AgentMovementSystem::new();
    for tick in 0..=12 {
        engine.resources.current_tick = tick;
        movement.tick(&mut engine.world, &mut engine.resources);
        let expected_x = match tick {
            0..=5 => 4,
            6..=11 => 5,
            _ => 6,
        };
        assert_eq!(
            *engine.world.get::<&Position>(entity).expect("position"),
            Position::new(expected_x, 4),
            "tick {tick}"
        );
        assert_eq!(
            *engine.world.get::<&MovementRng>(entity).expect("rng"),
            rng,
            "directed movement and skipped ticks must consume no random samples"
        );
    }
}
