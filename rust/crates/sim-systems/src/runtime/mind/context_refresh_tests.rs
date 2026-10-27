//! Exclusive refresh versus the unchanged immutable-host algorithm.
use super::*;
use crate::runtime::mind::{facts, MindProgram};
use sim_core::colony::mind::{choose_with_reader, EvalKey, EvalScratch};
use sim_core::colony::{ItemKind, JobKind, JobStage};

fn optional_fields(world: &mut World, entity: Entity) {
    world
        .insert(
            entity,
            (
                Child { born_tick: 1 },
                Elder,
                Hunger::new(61.25, 0.0),
                Thirst::new(42.0, 0.0),
                Sleep::new(32.0, 0.0),
                Social::new(51.0, 0.0),
                Temperament::from_axes([100, 200, 300, 400]),
                Inventory {
                    items: BTreeMap::from([(ResourceKind::Food, 3)]),
                },
            ),
        )
        .unwrap();
    world
        .insert(
            entity,
            (
                Job {
                    kind: JobKind::Chop { tree: (3, 3) },
                    stage: JobStage::ToSource,
                    taken_at: 0,
                    amount: 0,
                    item: ItemKind::Log,
                    material: None,
                    fetch: None,
                    best_distance: 2,
                    progress_tick: 0,
                },
                Carry {
                    kind: ItemKind::Log,
                    count: 2,
                    quality: 1,
                    material: None,
                },
                Drafted {
                    since: 0,
                    order: sim_core::colony::people::DraftOrder::Hold,
                    arming: None,
                },
                Fleeing {
                    until: 10,
                    from: 2,
                    to: (4, 4),
                    since: 1,
                },
                Activity {
                    kind: sim_core::components::ActivityKind::Explore,
                    settlement: 1,
                    place: (2, 2),
                    chosen_tick: 0,
                    arrived_tick: None,
                    until_tick: None,
                },
                ActivityRest { until_tick: 3 },
            ),
        )
        .unwrap();
}

#[test]
fn exclusive_refresh_matches_all_fields_and_partial_errors() {
    for case in 0..10 {
        let mut world = World::new();
        let entity = world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
        optional_fields(&mut world, entity);
        let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        let mut old = TickContext::capture(&world, &resources).unwrap();
        let mut new = TickContext::capture(&world, &resources).unwrap();
        *world.get::<&mut Position>(entity).unwrap() = Position::new(5, 6);
        match case {
            0 => {}
            1 => {
                world.remove_one::<Position>(entity).unwrap();
            }
            2 => {
                world.remove_one::<AgentState>(entity).unwrap();
            }
            3 => {
                world.remove_one::<Position>(entity).unwrap();
                world.remove_one::<AgentState>(entity).unwrap();
            }
            4 => {
                world.despawn(entity).unwrap();
            }
            5 => {
                old.versions[Domain::Person as usize] = u64::MAX;
                new.versions[Domain::Person as usize] = u64::MAX;
            }
            6 => {
                world
                    .remove::<(
                        Child,
                        Elder,
                        Hunger,
                        Thirst,
                        Sleep,
                        Social,
                        Temperament,
                        Inventory,
                    )>(entity)
                    .unwrap();
                world
                    .remove::<(Job, Carry, Drafted, Fleeing, Activity, ActivityRest)>(entity)
                    .unwrap();
            }
            7 => {
                world.remove_one::<Child>(entity).unwrap();
            }
            8 => {
                world.insert_one(entity, Inventory::default()).unwrap();
            }
            9 => {
                old.versions[Domain::Person as usize] = u64::MAX;
                new.versions[Domain::Person as usize] = u64::MAX;
                world.remove_one::<Position>(entity).unwrap();
            }
            _ => unreachable!(),
        }
        let expected = old.refresh_continuation(&world, 7);
        let got = new.refresh_continuation_exclusive(&mut world, 7);
        assert_eq!(got, expected, "case {case}");
        assert_eq!(
            format!("{:?}", new.rows),
            format!("{:?}", old.rows),
            "case {case}"
        );
        assert_eq!(new.row_refresh_states, old.row_refresh_states);
        assert_eq!(new.versions, old.versions);
        if case == 2 {
            assert_eq!(new.rows[0].position, Position::new(5, 6));
        }
        if case == 0 {
            assert_eq!(new.rows[0].life_stage, LifeStage::Child);
        }
    }
}

#[test]
fn fallback_without_duplicate_running_write_keeps_full_witnesses() {
    let source = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
      (id:"rest",band:"need",behaviour:"rest",when:[(fact:"idle",op:"has")],score:(constant:1,terms:[(fact:"hunger",weight_permille:1000)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"REST"),
      (id:"idle",band:"idle",behaviour:"wander",when:[(fact:"fatigue",op:">=",value:Some(0))],score:(constant:0,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;
    let program = MindProgram::load(source).unwrap();
    let mut world = World::new();
    world.spawn((
        Agent { id: 7 },
        Position::new(1, 1),
        AgentState::Idle,
        Hunger::new(61.0, 0.0),
        Sleep::new(21.0, 0.0),
    ));
    let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    for prior in [
        None,
        Some(RunningChoice {
            revision: program.policy.revision(),
            node: program.policy.node_id("idle").unwrap(),
            finished: false,
        }),
        Some(RunningChoice {
            revision: program.policy.revision(),
            node: program.policy.node_id("idle").unwrap(),
            finished: true,
        }),
    ] {
        let mut old = facts::prepare_continuation_context(&world, &resources).unwrap();
        let mut new = facts::prepare_continuation_context(&world, &resources).unwrap();
        old.refresh_continuation(&world, 7).unwrap();
        new.refresh_continuation_exclusive(&mut world, 7).unwrap();
        for context in [&mut old, &mut new] {
            context.set_program_running(7, prior, &program).unwrap();
            facts::refresh_routine_lifecycle(context, None, 7).unwrap();
        }
        let mut guard_results = Vec::new();
        for context in [&old, &new] {
            let person = context.row(7).unwrap();
            let mut gate = crate::runtime::mind::scheduling::ContinuationGate::new(&program.policy);
            let mut trace = Vec::new();
            let result = gate.evaluate_traced(
                &program,
                context,
                &resources,
                7,
                prior,
                crate::runtime::mind::scheduling::ContinuationEvidence {
                    state: person.state,
                    activity: person.activity,
                    seek: None,
                    history_matches: true,
                    replay_or_invitation: false,
                },
                Some(&mut trace),
            );
            guard_results.push((format!("{result:?}"), trace));
        }
        assert_eq!(guard_results[0], guard_results[1]);
        old.refresh_person(&world, 7).unwrap();
        new.refresh_person(&world, 7).unwrap();
        old.set_program_running(7, prior, &program).unwrap();
        let mut results = Vec::new();
        for context in [&mut old, &mut new] {
            facts::refresh_routine_lifecycle(context, None, 7).unwrap();
            let mut values = Vec::new();
            let mut scratch = EvalScratch::default();
            let mut row = program
                .providers
                .lazy_row(context, &resources, None, 7, &mut values)
                .unwrap();
            let selected = choose_with_reader(
                &program.policy,
                &mut row,
                prior,
                EvalKey {
                    world_seed: 0,
                    agent: 7,
                    tick: 0,
                    replicate: 0,
                },
                &mut scratch,
            )
            .unwrap();
            results.push((selected, scratch.witnesses().to_vec()));
        }
        assert_eq!(results[0], results[1]);
        assert_eq!(old.versions, new.versions);
        assert_eq!(
            format!("{:?}", old.running(7)),
            format!("{:?}", new.running(7))
        );
    }
}

fn same_refresh_state(a: &TickContext, b: &TickContext) {
    assert_eq!(format!("{:?}", a.rows), format!("{:?}", b.rows));
    for (a, b) in a.rows.iter().zip(&b.rows) {
        for (a, b) in [
            (a.hunger, b.hunger),
            (a.thirst, b.thirst),
            (a.fatigue, b.fatigue),
            (a.social, b.social),
        ] {
            assert_eq!(a.map(f64::to_bits), b.map(f64::to_bits));
        }
    }
    assert_eq!(a.row_refresh_states, b.row_refresh_states);
    assert_eq!(a.versions, b.versions);
}

#[test]
fn primary_refresh_matches_full_with_optional_fields_and_error_order() {
    for populated in [false, true] {
        for case in 0..10 {
            let mut world = World::new();
            let entity = world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
            if populated {
                optional_fields(&mut world, entity);
                world.get::<&mut Thirst>(entity).unwrap().value = -0.0;
                world.get::<&mut Social>(entity).unwrap().loneliness =
                    f64::from_bits(0x7ff8000000000001);
            }
            let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
            let mut full = TickContext::capture_continuation(&world, &resources).unwrap();
            let mut primary = TickContext::capture_continuation(&world, &resources).unwrap();
            // Force the rich flag to prove both helpers clear it at the same point.
            full.row_refresh_states[0].rich = true;
            primary.row_refresh_states[0].rich = true;
            *world.get::<&mut Position>(entity).unwrap() = Position::new(5, 6);
            match case {
                0 => {}
                1 => {
                    world.remove_one::<Position>(entity).unwrap();
                }
                2 => {
                    world.remove_one::<AgentState>(entity).unwrap();
                }
                3 => {
                    world.remove_one::<Position>(entity).unwrap();
                    world.remove_one::<AgentState>(entity).unwrap();
                }
                4 => {
                    world.despawn(entity).unwrap();
                }
                5 | 6 => {
                    full.versions[Domain::Person as usize] = u64::MAX;
                    primary.versions[Domain::Person as usize] = u64::MAX;
                    if case == 6 {
                        world.remove_one::<Position>(entity).unwrap();
                    }
                }
                7 => {
                    for context in [&mut full, &mut primary] {
                        context.rows[0].life_stage = LifeStage::Elder;
                        context.rows[0].drafted = !populated;
                        context.rows[0].activity = None;
                    }
                }
                8 => {
                    world
                        .insert_one(
                            entity,
                            AgentState::Seeking {
                                target: sim_core::components::TargetKind::Food,
                            },
                        )
                        .unwrap();
                }
                9 => {}
                _ => unreachable!(),
            }
            let agent = if case == 9 { 99 } else { 7 };
            assert_eq!(
                primary.refresh_primary_continuation_exclusive(&mut world, agent),
                full.refresh_continuation_exclusive(&mut world, agent),
                "populated={populated} case={case}"
            );
            same_refresh_state(&primary, &full);
        }
    }
}

#[test]
fn primary_refresh_corrects_uncaptured_duplicate_secondary_fields() {
    for missing in 0..3 {
        for duplicate_first in [false, true] {
            let mut world = World::new();
            if duplicate_first {
                let duplicate = world.spawn((Agent { id: 7 },));
                optional_fields(&mut world, duplicate);
                if missing == 1 {
                    world.insert_one(duplicate, Position::new(7, 7)).unwrap();
                }
                if missing == 2 {
                    world.insert_one(duplicate, AgentState::Idle).unwrap();
                }
            }
            world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
            if !duplicate_first {
                let duplicate = world.spawn((Agent { id: 7 },));
                optional_fields(&mut world, duplicate);
                if missing == 1 {
                    world.insert_one(duplicate, Position::new(7, 7)).unwrap();
                }
                if missing == 2 {
                    world.insert_one(duplicate, AgentState::Idle).unwrap();
                }
            }
            let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
            let mut full = TickContext::capture_continuation(&world, &resources).unwrap();
            let mut primary = TickContext::capture_continuation(&world, &resources).unwrap();
            assert_eq!(primary.rows.len(), 1);
            full.refresh_continuation_exclusive(&mut world, 7).unwrap();
            primary
                .refresh_primary_continuation_exclusive(&mut world, 7)
                .unwrap();
            same_refresh_state(&primary, &full);
            let row = primary.row(7).unwrap();
            assert_eq!(row.life_stage, LifeStage::Adult);
            assert!(!row.drafted);
            assert!(row.fleeing.is_none() && row.activity.is_none() && row.activity_rest.is_none());
            assert!(row.hunger.is_none() && row.job.is_none() && row.carry.is_none());
        }
    }
}

#[test]
fn primary_refresh_after_other_actor_consumption_keeps_exact_snapshot() {
    use crate::runtime::agent_actions::consume;
    let mut world = World::new();
    let earlier = world.spawn((
        Agent { id: 2 },
        Position::new(1, 1),
        AgentState::Consuming {
            target: sim_core::components::TargetKind::Food,
        },
    ));
    optional_fields(&mut world, earlier);
    let later = world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
    optional_fields(&mut world, later);
    let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    resources.food_tiles.insert((1, 1), 4);
    let mut full = TickContext::capture_continuation(&world, &resources).unwrap();
    let mut primary = TickContext::capture_continuation(&world, &resources).unwrap();
    let mut version = 0;
    {
        let mut state = world.get::<&mut AgentState>(earlier).unwrap();
        let mut hunger = world.get::<&mut Hunger>(earlier).unwrap();
        consume::consume_food(
            &mut resources,
            (1, 1),
            &mut state,
            Some(&mut hunger),
            &mut version,
        );
        world
            .get::<&mut Inventory>(earlier)
            .unwrap()
            .add(ResourceKind::Food, 1);
    }
    full.refresh_continuation_exclusive(&mut world, 7).unwrap();
    primary
        .refresh_primary_continuation_exclusive(&mut world, 7)
        .unwrap();
    same_refresh_state(&primary, &full);
}
