//! Private rich upgrade against unchanged full refresh after the same light boundary.
use super::*;
use crate::runtime::mind::{facts, MindProgram};
use sim_core::colony::mind::{choose_with_reader, EvalKey, EvalScratch};
use sim_core::colony::{ItemKind, JobKind, JobStage};
use sim_core::components::{Lineage, Memory, MemoryArm, MemoryEntry};

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

fn enrich(world: &mut World, entity: Entity) {
    optional_fields(world, entity);
    let memory = Memory {
        entries: [
            MemoryArm::Hunger,
            MemoryArm::Thirst,
            MemoryArm::Fatigue,
            MemoryArm::Construction,
            MemoryArm::Social,
            MemoryArm::Combat,
            MemoryArm::None,
        ]
        .into_iter()
        .enumerate()
        .map(|(index, arm)| MemoryEntry {
            event_id: u64::try_from(index).unwrap() + 11,
            arm,
            encoded_tick: 2,
            valence: if index == 4 { -0.0 } else { -0.375 },
            salience: 0.75,
            reinforcement_count: 2,
        })
        .collect(),
    };
    world
        .insert(
            entity,
            (
                Cold,
                Sick { until: 42 },
                Mood::default(),
                BodyHealth {
                    hp: 67.25,
                    max_hp: 100.0,
                },
                Lineage {
                    key: u64::MAX,
                    births: 3,
                },
                memory,
            ),
        )
        .unwrap();
}

fn assert_rows_equal(left: &PersonSnapshot, right: &PersonSnapshot) {
    assert_eq!(format!("{left:?}"), format!("{right:?}"));
    assert_eq!(
        left.memory_weights.map(f64::to_bits),
        right.memory_weights.map(f64::to_bits)
    );
    for (left, right) in [left.hunger, left.thirst, left.fatigue, left.social, left.hp]
        .into_iter()
        .zip([
            right.hunger,
            right.thirst,
            right.fatigue,
            right.social,
            right.hp,
        ])
    {
        assert_eq!(left.map(f64::to_bits), right.map(f64::to_bits));
    }
}

fn assert_contexts_equal(left: &TickContext, right: &TickContext) {
    assert_eq!(left.rows.len(), right.rows.len());
    for (left, right) in left.rows.iter().zip(&right.rows) {
        assert_rows_equal(left, right);
    }
    assert_eq!(left.row_refresh_states, right.row_refresh_states);
    assert_eq!(left.versions, right.versions);
    assert_eq!(
        format!("{:?}", left.derived_slots),
        format!("{:?}", right.derived_slots)
    );
}

#[test]
fn upgrade_matches_full_fields_memory_bits_and_activity_overlay() {
    for tick in [0, 2, 3, crate::runtime::memory::MAX_RECENCY_TICKS, u64::MAX] {
        for case in 0..5 {
            let mut world = World::new();
            let entity = world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
            enrich(&mut world, entity);
            if case == 4 {
                // A malformed secondary-only duplicate must never supply rich values.
                let duplicate = world.spawn((Agent { id: 7 },));
                enrich(&mut world, duplicate);
                world
                    .insert_one(
                        duplicate,
                        BodyHealth {
                            hp: -17.0,
                            max_hp: 100.0,
                        },
                    )
                    .unwrap();
                world
                    .insert_one(duplicate, Lineage { key: 99, births: 0 })
                    .unwrap();
            }
            let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
            let mut old = TickContext::capture_continuation(&world, &resources).unwrap();
            let mut new = TickContext::capture_continuation(&world, &resources).unwrap();
            old.tick = tick;
            new.tick = tick;
            // Changes before light refresh must be observed, not reused from tick capture.
            *world.get::<&mut Position>(entity).unwrap() = Position::new(5, 6);
            *world.get::<&mut Hunger>(entity).unwrap() = Hunger::new(11.5, 0.0);
            match case {
                0 | 4 => {}
                1 => {
                    world.insert_one(entity, Memory::new()).unwrap();
                }
                2 => {
                    world
                        .remove::<(Cold, Sick, Mood, BodyHealth, Memory, Lineage, Activity)>(entity)
                        .unwrap();
                }
                3 => {
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
                _ => unreachable!(),
            }
            for context in [&mut old, &mut new] {
                context
                    .refresh_continuation_exclusive(&mut world, 7)
                    .unwrap();
                facts::refresh_routine_lifecycle(context, None, 7).unwrap();
            }
            assert!(new.rows[0].activity.is_none());
            let before = new.versions[Domain::Person as usize];
            old.refresh_person_exclusive(&mut world, 7).unwrap();
            new.upgrade_continuation_exclusive(&mut world, 7).unwrap();
            assert_contexts_equal(&old, &new);
            assert_eq!(new.versions[Domain::Person as usize], before + 1);
            let restored = new.rows[0].activity.is_some();
            for context in [&mut old, &mut new] {
                facts::refresh_routine_lifecycle(context, None, 7).unwrap();
            }
            assert_contexts_equal(&old, &new);
            assert_eq!(
                new.versions[Domain::Person as usize],
                before + 1 + u64::from(restored)
            );
        }
    }
}

#[test]
fn upgrade_and_second_routine_preserve_overflow_atomicity() {
    for version in [u64::MAX, u64::MAX - 1, u64::MAX - 2] {
        let mut world = World::new();
        let entity = world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
        enrich(&mut world, entity);
        let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        let mut old = TickContext::capture_continuation(&world, &resources).unwrap();
        let mut new = TickContext::capture_continuation(&world, &resources).unwrap();
        for context in [&mut old, &mut new] {
            context
                .refresh_continuation_exclusive(&mut world, 7)
                .unwrap();
            facts::refresh_routine_lifecycle(context, None, 7).unwrap();
            context.versions[Domain::Person as usize] = version;
        }
        let before = new.rows[0].clone();
        let expected = old.refresh_person_exclusive(&mut world, 7);
        let actual = new.upgrade_continuation_exclusive(&mut world, 7);
        assert_eq!(actual, expected);
        assert_contexts_equal(&old, &new);
        if version == u64::MAX {
            assert_eq!(actual, Err(ContextError::VersionOverflow(Domain::Person)));
            assert_rows_equal(&before, &new.rows[0]);
            assert!(!new.row_refresh_states[0].rich);
        } else {
            assert!(new.rows[0].activity.is_some());
            let expected = facts::refresh_routine_lifecycle(&mut old, None, 7);
            let actual = facts::refresh_routine_lifecycle(&mut new, None, 7);
            match (&actual, &expected) {
                (Ok(()), Ok(())) => {}
                (Err(facts::FactError::Context(a)), Err(facts::FactError::Context(b))) => {
                    assert_eq!(a, b);
                }
                _ => panic!("unexpected lifecycle results: {actual:?} / {expected:?}"),
            }
            assert_eq!(actual.is_err(), version == u64::MAX - 1);
            assert_contexts_equal(&old, &new);
        }
    }
}

#[test]
fn upgrade_keeps_unknown_and_dead_entity_error_order() {
    for dead in [false, true] {
        let mut world = World::new();
        let entity = world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
        let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        let mut old = TickContext::capture_continuation(&world, &resources).unwrap();
        let mut new = TickContext::capture_continuation(&world, &resources).unwrap();
        for context in [&mut old, &mut new] {
            context
                .refresh_continuation_exclusive(&mut world, 7)
                .unwrap();
            context.versions[Domain::Person as usize] = u64::MAX;
        }
        let agent = if dead {
            world.despawn(entity).unwrap();
            7
        } else {
            999
        };
        // Despawn violates the normal caller contract; retain its defensive error mapping.
        let expected = old.refresh_person_exclusive(&mut world, agent);
        let actual = new.upgrade_continuation_exclusive(&mut world, agent);
        assert_eq!(actual, expected);
        assert_eq!(
            actual,
            if dead {
                Err(ContextError::MissingComponent {
                    agent,
                    component: "Position",
                })
            } else {
                Err(ContextError::UnknownAgent(agent))
            }
        );
        assert_contexts_equal(&old, &new);
    }
}

#[test]
fn missing_required_components_fail_at_light_boundary_before_upgrade() {
    for missing_position in [false, true] {
        let mut world = World::new();
        let entity = world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
        let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        let mut context = TickContext::capture_continuation(&world, &resources).unwrap();
        if missing_position {
            world.remove_one::<Position>(entity).unwrap();
        } else {
            world.remove_one::<AgentState>(entity).unwrap();
        }
        let result = context
            .refresh_continuation_exclusive(&mut world, 7)
            .and_then(|()| context.upgrade_continuation_exclusive(&mut world, 7));
        assert_eq!(
            result,
            Err(ContextError::MissingComponent {
                agent: 7,
                component: if missing_position {
                    "Position"
                } else {
                    "AgentState"
                },
            })
        );
        assert_eq!(context.versions[Domain::Person as usize], 1);
        assert!(!context.row_refresh_states[0].rich);
    }
}

#[test]
fn upgrade_preserves_guard_and_full_selector_witnesses() {
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
        old.refresh_continuation_exclusive(&mut world, 7).unwrap();
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
        old.refresh_person_exclusive(&mut world, 7).unwrap();
        new.upgrade_continuation_exclusive(&mut world, 7).unwrap();
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
