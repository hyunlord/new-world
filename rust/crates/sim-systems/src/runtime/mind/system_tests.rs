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
