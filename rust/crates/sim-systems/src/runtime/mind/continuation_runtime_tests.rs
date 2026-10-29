//! Actual scheduled effects, ownership exclusions and frozen snapshot readiness.
use super::*;
use crate::runtime::mind::{facts, ColonyMindSystem};
use hecs::World;
use sim_core::colony::mind_records::{ChoiceCandidate, MindChoiceRecord, MindHistory};
use sim_core::components::{Agent, Hunger, Position, Sleep, Thirst};
use sim_core::rules::WorldRules;

const POLICY: &str = r#"(
 schema_version:1, bands:["critical","duty","need","work","leisure","idle"], nodes:[
 (id:"eat_now",band:"need",behaviour:"eat",target:Some("food"),when:[(fact:"hunger",op:">",value:Some(50000))],score:(constant:100,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"EAT"),
 (id:"keep_sleep",band:"need",behaviour:"sleep",target:Some("sleep"),when:[(fact:"target_sleep",op:"has")],score:(constant:0,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"SLEEP"),
 (id:"idle",band:"idle",behaviour:"wander",when:[(fact:"hunger",op:">=",value:Some(0))],score:(constant:0,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;

#[test]
fn retained_tick_still_consumes_and_phase_tick_reconsiders_without_fabricated_record() {
    for (tick, retained) in [(1, true), (7, false)] {
        let program = MindProgram::load(POLICY).expect("policy");
        let policy = program.policy.clone();
        let node = policy.node_id("keep_sleep").expect("sleep");
        let mut world = World::new();
        let entity = world.spawn((
            Agent { id: 7 },
            Position::new(1, 1),
            AgentState::Consuming {
                target: TargetKind::Sleep,
            },
            Hunger::new(60.0, 0.0),
            Thirst::new(10.0, 0.0),
            Sleep::new(90.0, 0.0),
        ));
        let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        resources.rules = WorldRules::colony(0);
        resources.rules.agent.night_rest = None;
        resources.rules.agent.routine = None;
        resources.current_tick = tick;
        resources.nav.install(sim_core::nav::CostMap::new(8, 8));
        crate::runtime::food::items::create_ground(
            &mut resources,
            (2, 2),
            sim_core::colony::ItemSpec::raw(
                sim_core::colony::ItemKind::from_name("berries").expect("food"),
                None,
            ),
            10,
            0,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("physical food fixture");
        resources.sleep_tiles.insert((1, 1), 10);
        let mut history = MindHistory::new(&policy).expect("history");
        history
            .push_record(
                &policy,
                &MindChoiceRecord {
                    revision: policy.revision(),
                    id: 1,
                    parent: None,
                    agent: 7,
                    tick: 0,
                    node,
                    band: policy.node(node).expect("node").band,
                    score: 0,
                    top3: vec![ChoiceCandidate { node, score: 0 }],
                    facts: Vec::new(),
                },
            )
            .expect("initiating choice");
        resources.mind.policy_source = Some(POLICY.into());
        resources.mind.history = Some(history);
        resources.mind.running.insert(
            7,
            RunningChoice {
                revision: policy.revision(),
                node,
                finished: false,
            },
        );
        resources
            .next_event_id
            .store(2, std::sync::atomic::Ordering::Relaxed);
        ColonyMindSystem::new(program)
            .try_tick(&mut world, &mut resources)
            .expect("actual tick");
        let running = resources.mind.running[&7];
        if retained {
            assert_eq!(running.node, node);
            assert!(
                running.finished,
                "the existing one-step consume completion still runs"
            );
            assert_eq!(
                *world.get::<&AgentState>(entity).expect("state"),
                AgentState::Idle
            );
            assert!(world.get::<&Sleep>(entity).expect("sleep").fatigue < 90.0);
            assert_eq!(
                resources
                    .mind
                    .history
                    .as_ref()
                    .expect("history")
                    .records(7)
                    .expect("records")
                    .len(),
                1
            );
        } else {
            assert_eq!(running.node, policy.node_id("eat_now").expect("eat"));
            assert_eq!(
                world.get::<&SeekTarget>(entity).expect("food target").tile,
                (2, 2)
            );
            assert_eq!(
                *world.get::<&AgentState>(entity).expect("state"),
                AgentState::Seeking {
                    target: TargetKind::Food
                }
            );
            assert_eq!(
                resources
                    .mind
                    .history
                    .as_ref()
                    .expect("history")
                    .records(7)
                    .expect("records")
                    .len(),
                2
            );
        }
    }
}

#[test]
fn social_missing_peer_has_live_none_target_and_idle_requires_no_other_owner() {
    let mut world = World::new();
    world.spawn((
        Agent { id: 7 },
        Position::new(1, 1),
        AgentState::Seeking {
            target: TargetKind::Agent(99),
        },
        Hunger::new(10.0, 0.0),
    ));
    let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let context = facts::prepare_context(&world, &resources).expect("context");
    assert!(matches!(
        owned_target(
            LifecycleBinding::Social,
            context.row(7).expect("person"),
            &context,
            None
        ),
        Some(ActionTarget::None)
    ));
    let owner = OwnerState {
        state: AgentState::Idle,
        activity: None,
        seek: None,
        has_job: false,
        fleeing: false,
        drafted: false,
    };
    assert!(owner_admitted(LifecycleBinding::Idle, owner));
    assert!(!owner_admitted(
        LifecycleBinding::Idle,
        OwnerState {
            has_job: true,
            ..owner
        }
    ));
    assert!(!owner_admitted(
        LifecycleBinding::Idle,
        OwnerState {
            drafted: true,
            ..owner
        }
    ));
    assert!(!owner_admitted(
        LifecycleBinding::Idle,
        OwnerState {
            seek: Some(SeekTarget::new((2, 2))),
            ..owner
        }
    ));
}

#[test]
fn lightweight_capture_preserves_frozen_social_evidence_and_rejects_unready_full_reader() {
    use sim_core::components::{Memory, MemoryArm, MemoryEntry, Social};
    let program = MindProgram::load(POLICY).expect("policy");
    let mut memory = Memory::new();
    memory.entries.push(MemoryEntry {
        event_id: 9,
        arm: MemoryArm::Social,
        encoded_tick: 0,
        valence: 1.0,
        salience: 1.0,
        reinforcement_count: 0,
    });
    memory.entries.push(MemoryEntry {
        event_id: 10,
        arm: MemoryArm::Hunger,
        encoded_tick: 0,
        valence: -1.0,
        salience: 1.0,
        reinforcement_count: 0,
    });
    let mut world = World::new();
    for id in [7, 8] {
        world.spawn((
            Agent { id },
            Position::new(1, 1),
            AgentState::Idle,
            Hunger::new(10.0, 0.0),
            Thirst::new(10.0, 0.0),
            Sleep::new(10.0, 0.0),
            Social::new(60.0, 0.0),
            memory.clone(),
        ));
    }
    let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let eager = facts::prepare_context(&world, &resources).expect("eager");
    let mut light = facts::prepare_continuation_context(&world, &resources).expect("light");
    for id in [7, 8] {
        for name in [
            "social_candidate",
            "social_memory_candidate",
            "social_handshake",
        ] {
            assert_eq!(
                eager.derived(id, name).expect("eager"),
                light.derived(id, name).expect("light")
            );
        }
        for name in ["social_peer", "social_memory_peer"] {
            assert_eq!(
                eager.target(id, name).expect("eager"),
                light.target(id, name).expect("light")
            );
        }
        assert_eq!(
            eager.row(id).expect("eager").memory_causes[4],
            light.row(id).expect("light").memory_causes[4]
        );
        assert!(light.require_rich_row(id).is_err());
    }
    let mut memo = Vec::new();
    assert!(program
        .providers
        .lazy_row(&mut light, &resources, None, 7, &mut memo)
        .is_err());
    light.refresh_person(&world, 7).expect("full refresh");
    assert_eq!(
        eager.row(7).expect("eager").memory_causes,
        light.row(7).expect("light").memory_causes
    );
    assert!(program
        .providers
        .lazy_row(&mut light, &resources, None, 7, &mut memo)
        .is_ok());
}

#[path = "continuation_save_tests.rs"]
mod save_tests;

#[test]
fn retained_social_effect_keeps_prior_cause_and_phase_effect_uses_current_record() {
    use sim_core::causal::CausalEvent;
    let source = POLICY
        .replace(
            "behaviour:\"sleep\",target:Some(\"sleep\")",
            "behaviour:\"socialise\",target:Some(\"social_current_peer\")",
        )
        .replace("fact:\"target_sleep\"", "fact:\"target_social\"")
        .replace(
            "behaviour:\"eat\",target:Some(\"food\")",
            "behaviour:\"socialise\",target:Some(\"social_current_peer\")",
        );
    for tick in [1, 7] {
        let program = MindProgram::load(&source).unwrap();
        let policy = program.policy.clone();
        let node = policy.node_id("keep_sleep").unwrap();
        let mut world = World::new();
        for (agent, peer) in [(7, 8), (8, 7)] {
            world.spawn((
                Agent { id: agent },
                Position::new(1, 1),
                AgentState::Seeking {
                    target: TargetKind::Agent(peer),
                },
                Hunger::new(60.0, 0.0),
                Thirst::new(10.0, 0.0),
                Sleep::new(10.0, 0.0),
            ));
        }
        let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        resources.rules = WorldRules::colony(0);
        resources.rules.agent.routine = None;
        resources.current_tick = tick;
        resources.mind.policy_source = Some(source.clone());
        let mut history = MindHistory::new(&policy).unwrap();
        history
            .push_record(
                &policy,
                &MindChoiceRecord {
                    revision: policy.revision(),
                    id: 1,
                    parent: None,
                    agent: 7,
                    tick: 0,
                    node,
                    band: policy.node(node).unwrap().band,
                    score: 0,
                    top3: vec![ChoiceCandidate { node, score: 0 }],
                    facts: Vec::new(),
                },
            )
            .unwrap();
        resources.mind.history = Some(history);
        resources.mind.running.insert(
            7,
            RunningChoice {
                revision: policy.revision(),
                node,
                finished: false,
            },
        );
        resources
            .next_event_id
            .store(2, std::sync::atomic::Ordering::Relaxed);
        ColonyMindSystem::new(program)
            .try_tick(&mut world, &mut resources)
            .unwrap();
        let expected = resources
            .mind
            .history
            .as_ref()
            .unwrap()
            .latest(7)
            .unwrap()
            .id();
        if tick == 1 {
            assert_eq!(expected, 1);
        } else {
            assert_ne!(expected, 1);
        }
        let parents: Vec<_> = resources
            .causal_log
            .iter()
            .flat_map(|(_, log)| log.iter())
            .filter_map(|event| match event {
                CausalEvent::SocialInteractionStarted {
                    agents: (7, 8),
                    parent,
                    ..
                } => Some(*parent),
                _ => None,
            })
            .collect();
        assert_eq!(parents, [Some(expected)]);
    }
}
