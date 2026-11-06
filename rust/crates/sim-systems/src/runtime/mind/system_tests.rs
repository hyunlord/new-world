//! Data-only extension through the real registry, coordinator, target and record path.
use super::{ColonyMindSystem, MindProgram};
use crate::worldgen::{build_world_without_runtime_for_tests as build_world_with, WorldGenConfig};
use sim_core::causal::CausalEvent;
use sim_core::components::{Activity, Agent, AgentState, Cold, Position, SeekTarget, TargetKind};
use sim_core::rules::WorldRules;
use sim_engine::{FireState, RuntimeSystem};

const SOURCE: &str = r#"(
    schema_version: 1, bands: ["critical", "duty", "need", "work", "leisure", "idle"],
    nodes: [
        (id: "warm_at_hearth", band: "need", behaviour: "rest", target: Some("hearth"),
         when: [(fact: "cold", op: "has"), (fact: "hearth_lit", op: "has"),
                (fact: "hearth_available", op: "has")],
         score: (constant: 20, terms: []), hysteresis: 0,
         life_stages: ["adult", "child", "elder"], tags: [], name_key: "WARM_AT_HEARTH"),
        (id: "fallback", band: "idle", behaviour: "wander", when: [],
         score: (constant: 0, terms: [(fact: "hunger", weight_permille: 1)]), hysteresis: 0,
         life_stages: ["adult", "child", "elder"], tags: [], name_key: "FALLBACK"),
    ],
)"#;

#[test]
fn data_only_warm_at_hearth_reaches_real_target_and_retains_deciding_record() {
    for lit in [false, true] {
        // Extend a copy of the shipped policy; no provider, selector or effect code changes.
        let warm = r#"nodes: [
        (id: "warm_at_hearth", band: "need", behaviour: "rest", target: Some("hearth"),
         when: [(fact: "cold", op: "has"), (fact: "hearth_lit", op: "has"),
                (fact: "hearth_available", op: "has")],
         score: (constant: 1000000000000, terms: []), hysteresis: 0,
         life_stages: ["adult", "child", "elder"], tags: [], name_key: "WARM_AT_HEARTH"),"#;
        let shipped = include_str!("../../../../sim-core/data/colony/mind.ron");
        let source = shipped.replacen("nodes: [", warm, 1);
        assert!(source.contains("warm_at_hearth"));
        let program = MindProgram::load(&source).expect("extended shipped policy");
        let policy = program.policy.clone();
        let mut engine = build_world_with(&WorldGenConfig::colony(0), WorldRules::colony(0));
        // Worldgen queues camp buildings; real owners register them and admit residents.
        crate::runtime::influence::BuildingStampSystem::new()
            .tick(&mut engine.world, &mut engine.resources);
        crate::runtime::settlement::SettlementSystem::new()
            .tick(&mut engine.world, &mut engine.resources);
        let (entity, agent) = engine
            .world
            .query::<&Agent>()
            .iter()
            .map(|(entity, agent)| (entity, agent.id))
            .min_by_key(|(_, id)| *id)
            .expect("founder");
        let (settlement, hearth) = engine
            .resources
            .settlements
            .iter()
            .find(|(_, settlement)| settlement.member_agents.contains(&agent))
            .map(|(id, settlement)| (*id, settlement.formation_tile))
            .expect("founder admitted around real hearth after building registration");
        assert_eq!(
            engine.resources.building_kinds.get(&hearth),
            Some(&sim_core::components::BuildingKind::Hearth)
        );
        engine.resources.fires.insert(settlement, FireState { lit });
        engine.world.insert_one(entity, Cold).expect("cold fixture");
        assert!(
            engine.world.get::<&sim_core::colony::Job>(entity).is_err(),
            "fixture person must be free of duty"
        );
        engine
            .world
            .get::<&mut sim_core::components::Hunger>(entity)
            .expect("hunger")
            .value = 10.0;
        engine
            .world
            .get::<&mut sim_core::components::Thirst>(entity)
            .expect("thirst")
            .value = 10.0;
        engine
            .world
            .get::<&mut sim_core::components::Sleep>(entity)
            .expect("sleep")
            .fatigue = 10.0;
        engine
            .world
            .get::<&mut sim_core::components::Social>(entity)
            .expect("social")
            .loneliness = 0.0;
        let mut system = ColonyMindSystem::new(program);
        system
            .try_tick(&mut engine.world, &mut engine.resources)
            .expect("real coordinator");
        let history = engine
            .resources
            .mind
            .history
            .as_ref()
            .expect("retained history");
        let packed = history.latest(agent).expect("selected record");
        let record = packed
            .decode(&policy, history.dictionary())
            .expect("exact dictionary");
        let selected = policy.node(record.node).expect("selected node");
        if lit {
            assert_eq!(selected.name, "warm_at_hearth");
        } else {
            assert_ne!(selected.name, "warm_at_hearth");
        }
        assert!(record.facts.iter().any(|w| w.fact
            == policy.fact_id("hearth_lit").expect("fire fact")
            && w.value == i64::from(lit)));
        if !lit {
            continue;
        }
        assert_eq!(
            policy.behaviour(selected.behaviour).expect("binding").name,
            "rest"
        );
        assert_eq!(selected.target.as_deref(), Some("hearth"));
        assert_eq!(
            engine
                .world
                .get::<&SeekTarget>(entity)
                .expect("actual target")
                .tile,
            hearth
        );
        assert_eq!(
            engine
                .world
                .get::<&Activity>(entity)
                .expect("owned activity")
                .place,
            hearth
        );
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Seeking {
                target: TargetKind::Activity
            }
        );
        assert!(engine.resources.causal_log.iter().flat_map(|(_, log)| log.iter()).any(|event|
            matches!(event, CausalEvent::MindChoice { choice } if choice.id() == packed.id() && choice.shares_payload(packed))));
        // Arrival is a separate fixture transition; the real continuation owns consumption.
        *engine.world.get::<&mut Position>(entity).expect("position") =
            Position::new(hearth.0, hearth.1);
        engine.resources.current_tick += 1;
        system
            .try_tick(&mut engine.world, &mut engine.resources)
            .expect("actual continuation");
        assert_eq!(
            *engine
                .world
                .get::<&AgentState>(entity)
                .expect("arrived state"),
            AgentState::Consuming {
                target: TargetKind::Activity
            }
        );
        assert_eq!(
            engine
                .world
                .get::<&Activity>(entity)
                .expect("arrived activity")
                .arrived_tick,
            Some(engine.resources.current_tick)
        );
    }
}

#[test]
fn actual_program_load_rejects_unknown_fact_behaviour_and_target_names() {
    for (source, needle) in [
        (
            SOURCE.replace("fact: \"cold\"", "fact: \"unknown_cold\""),
            "unknown_cold",
        ),
        (
            SOURCE.replace("behaviour: \"rest\"", "behaviour: \"unknown_rest\""),
            "unknown_rest",
        ),
        (
            SOURCE.replace("Some(\"hearth\")", "Some(\"unknown_hearth\")"),
            "unknown_hearth",
        ),
    ] {
        let error = MindProgram::load(&source)
            .err()
            .expect("unknown binding must fail before effects");
        assert!(error.to_string().contains(needle), "{error}");
    }
}

#[test]
fn authored_finish_dispatches_effect_once_and_keeps_idle_tick_boundary() {
    use sim_core::components::Hunger;
    for gather in [false, true] {
        let source = SOURCE.replace(
        "id: \"warm_at_hearth\", band: \"need\", behaviour: \"rest\", target: Some(\"hearth\"),\n         when: [(fact: \"cold\", op: \"has\"), (fact: \"hearth_lit\", op: \"has\"),\n                (fact: \"hearth_available\", op: \"has\")]",
        "id: \"finish_food\", band: \"need\", behaviour: \"eat\", finish: Some(\"interrupted\"),\n         when: [(fact: \"consuming\", op: \"has\"), (fact: \"target_food\", op: \"has\")]",
    );
        let source = if gather {
            source
                .replace("behaviour: \"eat\"", "behaviour: \"gather\"")
                .replace("fact: \"consuming\"", "fact: \"seeking\"")
                .replace("fact: \"target_food\"", "fact: \"target_gather\"")
        } else {
            source
        };
        assert!(
            source.contains("finish_food"),
            "fixture must install the authored finish node"
        );
        let program = MindProgram::load(&source).expect("finish policy");
        let finish_node = program.policy.node_id("finish_food").expect("finish node");
        let fallback = program.policy.node_id("fallback").expect("fallback node");
        let mut system = ColonyMindSystem::new(program);
        let mut engine = build_world_with(&WorldGenConfig::colony(0), WorldRules::colony(0));
        let (entity, agent) = engine
            .world
            .query::<&Agent>()
            .iter()
            .map(|(entity, agent)| (entity, agent.id))
            .min_by_key(|(_, id)| *id)
            .expect("founder");
        *engine.world.get::<&mut AgentState>(entity).expect("state") = if gather {
            AgentState::Seeking {
                target: TargetKind::GatherFood,
            }
        } else {
            AgentState::Consuming {
                target: TargetKind::Food,
            }
        };
        let preempted = engine.resources.tallies.gather_preempted;
        let hunger = engine.world.get::<&Hunger>(entity).expect("hunger").value;
        system
            .try_tick(&mut engine.world, &mut engine.resources)
            .expect("finish transaction");
        let running = engine
            .resources
            .mind
            .running
            .get(&agent)
            .expect("finished choice");
        assert_eq!(running.node, finish_node);
        assert!(running.finished);
        assert_eq!(
            engine.resources.tallies.gather_preempted,
            preempted + u64::from(gather)
        );
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Idle
        );
        assert_eq!(
            engine.world.get::<&Hunger>(entity).expect("hunger").value,
            hunger,
            "finish must not run eating start/continue"
        );
        assert_eq!(
            engine
                .resources
                .mind
                .history
                .as_ref()
                .expect("history")
                .records(agent)
                .expect("records")
                .len(),
            1
        );
        engine.resources.current_tick += 1;
        system
            .try_tick(&mut engine.world, &mut engine.resources)
            .expect("next tick");
        assert_eq!(
            engine
                .resources
                .mind
                .running
                .get(&agent)
                .expect("fallback")
                .node,
            fallback
        );
        assert_eq!(
            engine.resources.tallies.gather_preempted,
            preempted + u64::from(gather),
            "finish tally must not repeat on the next tick"
        );
    }
}

#[path = "system_rest_tests.rs"]
mod rest_regressions;

#[test]
fn initialized_system_rejects_replaced_history_before_tick_effects() {
    let program = MindProgram::load(SOURCE).unwrap();
    let mut system = ColonyMindSystem::new(program);
    let mut resources = sim_engine::SimResources::new(4, 4, sim_core::MaterialRegistry::new());
    resources.rules = WorldRules::colony(0);
    system.initialize(&mut resources).unwrap();
    let history = resources.mind.history.as_ref().unwrap();
    let name = history.dictionary().node_label(0).unwrap().name.as_bytes();
    let mut wire = bincode::serialize(history).unwrap();
    // Fixed-width bincode prefix: revision, node count, then first name length.
    assert_eq!(&wire[24..24 + name.len()], name);
    wire[24] ^= 1; // Same-length valid ASCII name corruption; revision stays intact.
    resources.mind.history = Some(bincode::deserialize(&wire).unwrap());
    let mut world = hecs::World::new();
    let entity = world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
    let before = bincode::serialize(&resources.mind).unwrap();
    let events = bincode::serialize(&resources.causal_log).unwrap();
    let chronicle = bincode::serialize(&resources.chronicle).unwrap();
    let next = resources
        .next_event_id
        .load(std::sync::atomic::Ordering::Relaxed);
    let error = system.try_tick(&mut world, &mut resources).unwrap_err();
    assert!(error.to_string().contains("policy"), "{error}");
    assert_eq!(*world.get::<&AgentState>(entity).unwrap(), AgentState::Idle);
    assert_eq!(bincode::serialize(&resources.mind).unwrap(), before);
    assert_eq!(bincode::serialize(&resources.causal_log).unwrap(), events);
    assert_eq!(bincode::serialize(&resources.chronicle).unwrap(), chronicle);
    assert_eq!(
        resources
            .next_event_id
            .load(std::sync::atomic::Ordering::Relaxed),
        next
    );
}

#[test]
fn colony_water_binding_coordinator_retains_walk_then_reselects_on_navigation_change() {
    let program = MindProgram::load(r#"(
        schema_version: 1, bands: ["critical", "duty", "need", "work", "leisure", "idle"],
        nodes: [
            (id: "water", band: "need", behaviour: "drink", target: Some("water"), when: [(fact: "water_available", op: "has")],
             score: (constant: 1, terms: []), hysteresis: 0,
             life_stages: ["adult", "child", "elder"], tags: [], name_key: "WATER"),
            (id: "fallback", band: "idle", behaviour: "wander", when: [],
             score: (constant: 0, terms: [(fact: "thirst", weight_permille: 0)]), hysteresis: 0,
             life_stages: ["adult", "child", "elder"], tags: [], name_key: "FALLBACK")
        ])"#).unwrap();
    let mut system = ColonyMindSystem::new(program);
    let mut world = hecs::World::new();
    let person = world.spawn((
        Agent { id: 7 },
        Position::new(2, 2),
        AgentState::Idle,
        sim_core::components::Thirst::new(80.0, 0.08),
        crate::runtime::agent::MovementRng::new(7),
    ));
    let mut r = sim_engine::SimResources::new(12, 12, sim_core::MaterialRegistry::new());
    r.rules = WorldRules::colony(0);
    r.nav.install(sim_core::nav::CostMap::new(12, 12));
    r.water_tiles.insert((8, 2), 255);
    system.try_tick(&mut world, &mut r).unwrap();
    assert_eq!(world.get::<&SeekTarget>(person).unwrap().tile, (8, 2));
    let path = r.nav.path((2, 2), (8, 2)).unwrap();
    r.nav.paths.insert(
        7,
        sim_core::nav::AgentPath::new((8, 2), r.nav.version, (2, 2), path.to_vec(), 0),
    );
    r.water_tiles.insert((2, 3), 255);
    let stats = *r.nav.stats.borrow();
    for tick in 1..4 {
        r.current_tick = tick;
        system.try_tick(&mut world, &mut r).unwrap();
        assert_eq!(world.get::<&SeekTarget>(person).unwrap().tile, (8, 2));
    }
    assert_eq!(
        *r.nav.stats.borrow(),
        stats,
        "valid binding performs no path queries"
    );
    r.nav.set_cost((9, 9), sim_core::nav::COST_DOOR);
    r.current_tick += 1;
    system.try_tick(&mut world, &mut r).unwrap();
    assert_eq!(world.get::<&SeekTarget>(person).unwrap().tile, (2, 3));
    r.rules.agent.move_interval_ticks = 1;
    crate::runtime::agent::AgentMovementSystem::new().tick(&mut world, &mut r);
    assert_eq!(
        *world.get::<&Position>(person).unwrap(),
        Position::new(2, 3)
    );
    assert_eq!(
        r.nav.paths.get(&7).unwrap().goal,
        (2, 3),
        "movement replaces the old water route"
    );
    r.water_tiles.insert((2, 3), 2);
    r.current_tick += 1;
    system.try_tick(&mut world, &mut r).unwrap();
    assert_eq!(
        *world.get::<&AgentState>(person).unwrap(),
        AgentState::Consuming {
            target: TargetKind::Water
        }
    );
    let thirst = world
        .get::<&sim_core::components::Thirst>(person)
        .unwrap()
        .value;
    r.current_tick += 1;
    system.try_tick(&mut world, &mut r).unwrap();
    assert_eq!(r.water_tiles.get(&(2, 3)), Some(&1));
    assert_eq!(
        world
            .get::<&sim_core::components::Thirst>(person)
            .unwrap()
            .value,
        (thirst - 30.0).max(0.0)
    );
}

fn food_walk_saturation_scene() -> (
    hecs::World,
    sim_engine::SimResources,
    hecs::Entity,
    ColonyMindSystem,
) {
    use crate::runtime::food::items::{self, FoodSource};
    use sim_core::colony::{ItemKind, ItemSpec};
    use sim_core::components::{Hunger, Inventory, Lineage, Thirst};

    let mut world = hecs::World::new();
    let person = world.spawn((
        Agent { id: 7 },
        Position::new(2, 2),
        AgentState::Idle,
        Hunger::new(87.0, 0.0),
        Thirst::new(20.0, 0.0),
        Inventory::default(),
        Lineage { key: 7, births: 0 },
        crate::runtime::agent::MovementRng::new(7),
    ));
    let mut r = sim_engine::SimResources::new(16, 8, sim_core::MaterialRegistry::new());
    r.rules = WorldRules::colony(0);
    r.rules.agent.move_interval_ticks = 1;
    r.nav.install(sim_core::nav::CostMap::new(16, 8));
    r.water_tiles.insert((2, 3), 2);
    items::create_ground(
        &mut r,
        (13, 2),
        ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None),
        2,
        0,
        FoodSource::Harvest,
    )
    .unwrap();
    let program =
        MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron")).unwrap();
    let mut mind = ColonyMindSystem::new(program);
    mind.try_tick(&mut world, &mut r).unwrap();
    assert_eq!(
        *world.get::<&AgentState>(person).unwrap(),
        AgentState::Seeking {
            target: TargetKind::Food
        }
    );
    assert_eq!(world.get::<&SeekTarget>(person).unwrap().tile, (13, 2));
    let path = r.nav.path((2, 2), (13, 2)).unwrap();
    assert_eq!(path.len(), 11);
    r.nav.paths.insert(
        7,
        sim_core::nav::AgentPath::new((13, 2), r.nav.version, (2, 2), path.to_vec(), 0),
    );
    (world, r, person, mind)
}

#[test]
fn saturated_thirst_interrupts_food_walk_before_merely_critical_hunger() {
    use sim_core::components::{BodyHealth, Thirst};
    let (mut world, mut r, person, mut mind) = food_walk_saturation_scene();
    world
        .insert_one(
            person,
            BodyHealth {
                hp: 1.58,
                max_hp: 100.0,
            },
        )
        .unwrap();
    // Unsaturated thirst preserves the existing hunger-first choice and route.
    world.get::<&mut Thirst>(person).unwrap().value = 99.0;
    r.current_tick = 1;
    mind.try_tick(&mut world, &mut r).unwrap();
    assert_eq!(world.get::<&SeekTarget>(person).unwrap().tile, (13, 2));
    assert_eq!(r.nav.paths[&7].goal, (13, 2));

    // This intentionally specifies a new saturation boundary, not legacy policy parity.
    world.get::<&mut Thirst>(person).unwrap().value = 100.0;
    r.current_tick = 2;
    mind.try_tick(&mut world, &mut r).unwrap();
    assert_eq!(
        *world.get::<&AgentState>(person).unwrap(),
        AgentState::Seeking {
            target: TargetKind::Water
        }
    );
    assert_eq!(world.get::<&SeekTarget>(person).unwrap().tile, (2, 3));
    crate::runtime::agent::AgentMovementSystem::new().tick(&mut world, &mut r);
    assert_eq!(
        *world.get::<&Position>(person).unwrap(),
        Position::new(2, 3)
    );
    assert_eq!(r.nav.paths[&7].goal, (2, 3));
    for tick in 3..=4 {
        r.current_tick = tick;
        mind.try_tick(&mut world, &mut r).unwrap();
        crate::runtime::survival::StarvationSystem::new().tick(&mut world, &mut r);
    }
    assert_eq!(r.water_tiles[&(2, 3)], 1);
    assert_eq!(world.get::<&Thirst>(person).unwrap().value, 70.0);
    assert_eq!(r.food.tallies.meals, 0);
    assert_eq!(r.colony.ground_items[&(13, 2)].count, 2);
    for tick in 5..=15 {
        r.current_tick = tick;
        crate::runtime::agent::AgentMovementSystem::new().tick(&mut world, &mut r);
        mind.try_tick(&mut world, &mut r).unwrap();
        crate::runtime::survival::StarvationSystem::new().tick(&mut world, &mut r);
    }
    assert!(world.get::<&BodyHealth>(person).unwrap().hp > 0.0);
    assert_eq!(r.tallies.deaths_dehydration, 0);
}

#[test]
fn saturated_thirst_food_walk_guard_preserves_ineligible_cases() {
    use sim_core::components::{Hunger, Thirst};
    for case in [
        "empty",
        "blocked",
        "disconnected",
        "both_saturated",
        "consuming",
        "disabled",
    ] {
        let (mut world, mut r, person, mut mind) = food_walk_saturation_scene();
        world.get::<&mut Thirst>(person).unwrap().value = 100.0;
        match case {
            "empty" => r.water_tiles.clear(),
            "blocked" => {
                r.nav.set_cost((2, 3), 0);
            }
            "disconnected" => {
                r.water_tiles.clear();
                r.water_tiles.insert((2, 6), 2);
                for x in 0..16 {
                    r.nav.set_cost((x, 4), 0);
                }
            }
            "both_saturated" => world.get::<&mut Hunger>(person).unwrap().value = 100.0,
            "consuming" => {
                *world.get::<&mut Position>(person).unwrap() = Position::new(13, 2);
                *world.get::<&mut AgentState>(person).unwrap() = AgentState::Consuming {
                    target: TargetKind::Food,
                };
            }
            "disabled" => r.rules.agent.critical_preemption = None,
            _ => unreachable!(),
        }
        r.current_tick = 1;
        mind.try_tick(&mut world, &mut r).unwrap();
        assert_ne!(
            *world.get::<&AgentState>(person).unwrap(),
            AgentState::Seeking {
                target: TargetKind::Water
            },
            "{case}"
        );
        if case == "consuming" {
            assert_eq!(r.food.tallies.meals, 1);
        } else {
            assert_eq!(
                world.get::<&SeekTarget>(person).unwrap().tile,
                (13, 2),
                "{case}"
            );
        }
        if matches!(case, "empty" | "blocked" | "disconnected") {
            for tick in 2..=16 {
                r.current_tick = tick;
                crate::runtime::agent::AgentMovementSystem::new().tick(&mut world, &mut r);
                mind.try_tick(&mut world, &mut r).unwrap();
                if r.food.tallies.meals > 0 {
                    break;
                }
            }
            assert_eq!(
                r.food.tallies.meals, 1,
                "valid food journey must finish: {case}"
            );
        }
    }
}
