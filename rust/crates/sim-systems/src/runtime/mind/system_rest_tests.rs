//! Coordinator regressions for real routine-to-rest handoff and finite hearth seats.
use super::{ColonyMindSystem, MindProgram, SOURCE};
use crate::worldgen::{build_world_with, WorldGenConfig};
use sim_core::components::{Activity, ActivityKind, Agent, AgentState, Cold, Hunger, SeekTarget};
use sim_core::rules::WorldRules;
use sim_engine::{FireState, RuntimeSystem, SimEngine};

fn fixture() -> (SimEngine, Vec<(hecs::Entity, u64)>, (u32, u32)) {
    let mut engine = build_world_with(&WorldGenConfig::colony(0), WorldRules::colony(0));
    crate::runtime::influence::BuildingStampSystem::new()
        .tick(&mut engine.world, &mut engine.resources);
    crate::runtime::settlement::SettlementSystem::new()
        .tick(&mut engine.world, &mut engine.resources);
    let mut people: Vec<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(e, a)| (e, a.id))
        .collect();
    people.sort_by_key(|(_, agent)| *agent);
    let (settlement, hearth) = engine
        .resources
        .settlements
        .iter()
        .find(|(_, s)| s.member_agents.contains(&people[0].1))
        .map(|(sid, s)| (*sid, s.formation_tile))
        .expect("actual populated hearth");
    engine
        .resources
        .fires
        .insert(settlement, FireState { lit: true });
    for (entity, _) in &people {
        engine
            .world
            .get::<&mut Hunger>(*entity)
            .expect("hunger")
            .value = 0.0;
    }
    (engine, people, hearth)
}

#[test]
fn active_routine_to_warm_rest_observes_interruption_boundary_then_starts_next_tick() {
    let entry = r#"nodes: [
        (id: "initial_routine", band: "need", behaviour: "routine_tend_fire", target: Some("routine_tend_fire"),
         when: [(fact: "cold", op: "not"), (fact: "hunger", op: "==", value: Some(10000)),
                (fact: "routine_tend_fire_available", op: "has")],
         score: (constant: 1000, terms: []), hysteresis: 0,
         life_stages: ["adult", "child", "elder"], tags: [], name_key: "INITIAL_ROUTINE"),"#;
    let source = SOURCE.replacen("nodes: [", entry, 1);
    let program = MindProgram::load(&source).expect("registered routine and targeted rest");
    let warm = program.policy.node_id("warm_at_hearth").expect("warm node");
    let mut system = ColonyMindSystem::new(program);
    let (mut engine, people, hearth) = fixture();
    let (entity, agent) = people[0];
    engine
        .world
        .get::<&mut Hunger>(entity)
        .expect("hunger")
        .value = 10.0;
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("real routine entry");
    assert_eq!(
        engine
            .world
            .get::<&Activity>(entity)
            .expect("active routine")
            .kind,
        ActivityKind::TendFire
    );
    engine.world.insert_one(entity, Cold).expect("cold onset");
    engine.resources.current_tick += 1;
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("routine interruption is not a fatal admission error");
    assert_eq!(
        *engine.world.get::<&AgentState>(entity).expect("state"),
        AgentState::Idle
    );
    assert!(engine.world.get::<&Activity>(entity).is_err());
    assert!(engine.world.get::<&SeekTarget>(entity).is_err());
    let running = engine
        .resources
        .mind
        .running
        .get(&agent)
        .expect("warm choice");
    assert_eq!(running.node, warm);
    assert!(running.finished);
    engine.resources.current_tick += 1;
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("fresh transaction permits warmth");
    assert_eq!(
        engine
            .world
            .get::<&Activity>(entity)
            .expect("warm activity")
            .place,
        hearth
    );
    assert_eq!(
        engine
            .world
            .get::<&SeekTarget>(entity)
            .expect("warm target")
            .tile,
        hearth
    );
    assert!(
        !engine
            .resources
            .mind
            .running
            .get(&agent)
            .expect("running warmth")
            .finished
    );
}

#[test]
fn three_cold_people_share_two_hearth_seats_without_aborting_the_tick() {
    let program = MindProgram::load(SOURCE).expect("targeted rest policy");
    let warm = program.policy.node_id("warm_at_hearth").expect("warm node");
    let mut system = ColonyMindSystem::new(program);
    let (mut engine, people, hearth) = fixture();
    for (entity, _) in &people[..3] {
        engine
            .world
            .insert_one(*entity, Cold)
            .expect("cold fixture");
    }
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("full hearth is ordinary unavailable admission");
    for (index, (entity, agent)) in people[..3].iter().enumerate() {
        let running = engine
            .resources
            .mind
            .running
            .get(agent)
            .expect("choice for every person");
        assert_eq!(running.node, warm);
        if index < 2 {
            assert!(!running.finished);
            assert_eq!(
                engine
                    .world
                    .get::<&Activity>(*entity)
                    .expect("admitted seat")
                    .place,
                hearth
            );
            assert_eq!(
                engine
                    .world
                    .get::<&SeekTarget>(*entity)
                    .expect("admitted target")
                    .tile,
                hearth
            );
        } else {
            assert!(running.finished);
            assert_eq!(
                *engine.world.get::<&AgentState>(*entity).expect("state"),
                AgentState::Idle
            );
            assert!(engine.world.get::<&Activity>(*entity).is_err());
            assert!(engine.world.get::<&SeekTarget>(*entity).is_err());
        }
    }
}

#[test]
fn arrival_on_walk_timeout_tick_arrives_instead_of_ending_the_activity() {
    use crate::runtime::agent_actions::routine::{PlacesCache, RoutineTick};
    use crate::runtime::mind::facts;
    use sim_core::components::{ActivityRest, Position, TargetKind};
    const CONTINUE: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
    (id:"continue",band:"leisure",behaviour:"routine_play",when:[(fact:"activity_active",op:"has")],score:(constant:10,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"PLAY"),
    (id:"idle",band:"idle",behaviour:"wander",when:[],score:(constant:0,terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;
    let (mut engine, people, hearth) = fixture();
    let (entity, agent) = people[0];
    let member = engine
        .resources
        .settlements
        .iter()
        .find(|(_, settlement)| settlement.member_agents.contains(&agent))
        .map(|(id, _)| *id)
        .expect("member");
    let mut rule = engine.resources.rules.agent.routine.expect("routine rule");
    rule.walk_timeout_minutes = 5;
    engine.resources.rules.agent.routine = Some(rule);
    engine.resources.current_tick = 5;
    engine.resources.ticks_per_day = 0; // Isolate walk timeout from the daily closing boundary.
    engine
        .world
        .insert(
            entity,
            (
                Position::new(hearth.0, hearth.1),
                AgentState::Seeking {
                    target: TargetKind::Activity,
                },
                Activity {
                    kind: ActivityKind::Play,
                    settlement: member,
                    place: hearth,
                    chosen_tick: 0,
                    arrived_tick: None,
                    until_tick: None,
                },
            ),
        )
        .expect("arriving activity");
    let mut transaction = RoutineTick::new(
        &mut engine.world,
        &engine.resources,
        rule,
        PlacesCache::default(),
    );
    transaction.prepare_preview_places(&engine.resources);
    let mut context = facts::prepare_context(&engine.world, &engine.resources).expect("context");
    facts::refresh_routine(&mut context, Some(&transaction), agent).expect("routine facts");
    assert_eq!(
        context
            .derived(agent, "activity_deadline_reached")
            .expect("deadline"),
        0
    );
    drop(transaction);
    let mut system = ColonyMindSystem::new(MindProgram::load(CONTINUE).expect("policy"));
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("arrival tick");
    assert_eq!(
        *engine.world.get::<&AgentState>(entity).expect("state"),
        AgentState::Consuming {
            target: TargetKind::Activity
        }
    );
    assert_eq!(
        engine
            .world
            .get::<&Activity>(entity)
            .expect("active reservation")
            .arrived_tick,
        Some(5)
    );
    assert!(engine.world.get::<&ActivityRest>(entity).is_err());
}

#[test]
fn admitted_routine_without_weighted_place_records_denial_and_thirty_tick_cooldown() {
    use crate::runtime::agent_actions::routine::{PlacesCache, RoutineTick};
    use sim_core::components::{
        ActivityRest, Lineage, RoutineObservation, RoutineOutcome, Sleep, Social, Temperament,
        Thirst,
    };
    let (mut engine, people, _) = fixture();
    engine.resources.current_tick = 600;
    engine.resources.rules.clock_offset_ticks = 0;
    let rule = engine
        .resources
        .rules
        .agent
        .routine
        .as_mut()
        .expect("routine");
    rule.day_start_minute = 0;
    rule.day_end_minute = 1440;
    rule.end_stagger = None;
    rule.adult_weights = [0; 5];
    rule.elder_weights = [0; 2];
    engine.resources.food_tiles.clear();
    for (entity, _) in &people {
        engine
            .world
            .get::<&mut Thirst>(*entity)
            .expect("thirst")
            .value = 0.0;
        engine
            .world
            .get::<&mut Sleep>(*entity)
            .expect("sleep")
            .fatigue = 0.0;
        engine
            .world
            .get::<&mut Social>(*entity)
            .expect("social")
            .loneliness = 0.0;
        engine
            .world
            .insert_one(*entity, AgentState::Idle)
            .expect("idle");
    }
    let (entity, agent) = people[0];
    assert!(engine
        .world
        .get::<&sim_core::components::Child>(entity)
        .is_err());
    let lineage = engine.world.get::<&Lineage>(entity).expect("lineage").key;
    let temperament = *engine
        .world
        .get::<&Temperament>(entity)
        .expect("temperament");
    let member = engine
        .resources
        .settlements
        .iter()
        .find(|(_, settlement)| settlement.member_agents.contains(&agent))
        .map(|(id, _)| *id);
    // Establish the legacy lifecycle expectation using its real admission/start path.
    let mut old = RoutineTick::new(
        &mut engine.world,
        &engine.resources,
        engine.resources.rules.agent.routine.expect("routine"),
        PlacesCache::default(),
    );
    assert!(old.can_enter(agent, Some(lineage), member, Some(&temperament)));
    assert!(!old.start(
        agent,
        entity,
        Some(lineage),
        member,
        false,
        false,
        Some(&temperament),
        &engine.resources
    ));
    old.finish(&mut engine.world);
    let expected = *engine
        .world
        .get::<&ActivityRest>(entity)
        .expect("legacy cooldown");
    let observation = *engine
        .world
        .get::<&RoutineObservation>(entity)
        .expect("legacy denial");
    assert_eq!(expected.until_tick, 630);
    assert!(matches!(
        observation.outcome,
        RoutineOutcome::NoPlace { redrawn: true }
    ));
    engine
        .world
        .remove_one::<ActivityRest>(entity)
        .expect("reset legacy cooldown");
    engine
        .world
        .remove_one::<RoutineObservation>(entity)
        .expect("reset legacy observation");
    let _ = engine
        .world
        .remove_one::<sim_core::components::RoutineObservationBatch>(entity);
    let shipped = include_str!("../../../../sim-core/data/colony/mind.ron");
    let mut system = ColonyMindSystem::new(MindProgram::load(shipped).expect("shipped policy"));
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("mind tick");
    assert_eq!(
        engine
            .world
            .get::<&ActivityRest>(entity)
            .ok()
            .map(|rest| rest.until_tick),
        Some(expected.until_tick),
        "denied routine must retain the legacy retry delay"
    );
    assert_eq!(
        engine
            .world
            .get::<&RoutineObservation>(entity)
            .ok()
            .map(|event| *event),
        Some(observation)
    );
    assert_eq!(
        *engine.world.get::<&AgentState>(entity).expect("state"),
        AgentState::Idle
    );
}
