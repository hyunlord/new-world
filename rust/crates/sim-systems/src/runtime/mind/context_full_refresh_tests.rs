//! Exclusive full capture against the unchanged shared-world capture and refresh.
use super::*;
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

#[test]
fn full_refresh_preserves_fields_errors_and_atomic_assignment() {
    for case in 0..13 {
        let mut world = World::new();
        let entity = world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
        enrich(&mut world, entity);
        let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        let mut old = TickContext::capture_continuation(&world, &resources).unwrap();
        let mut new = TickContext::capture_continuation(&world, &resources).unwrap();
        let original_row = format!("{:?}", old.rows[0]);
        *world.get::<&mut Position>(entity).unwrap() = Position::new(5, 6);
        let mut agent = 7;
        match case {
            0 => {}
            1 => {
                world.remove_one::<Position>(entity).unwrap();
            }
            2 => {
                world.remove_one::<AgentState>(entity).unwrap();
            }
            3 => {
                world.remove::<(Position, AgentState)>(entity).unwrap();
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
                world
                    .remove::<(Cold, Sick, Mood, BodyHealth, Lineage, Memory)>(entity)
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
            10 => {
                agent = 999;
            }
            11 => {
                world.remove::<(Child, Elder)>(entity).unwrap();
            }
            12 => {
                world.insert_one(entity, Memory::new()).unwrap();
            }
            _ => unreachable!(),
        }
        let expected = old.refresh_person(&world, agent);
        let got = new.refresh_person_exclusive(&mut world, agent);
        assert_eq!(got, expected, "case {case}");
        assert_rows_equal(&new.rows[0], &old.rows[0]);
        assert_eq!(new.row_refresh_states, old.row_refresh_states);
        assert_eq!(new.versions, old.versions);
        if got.is_err() {
            assert_eq!(
                format!("{:?}", new.rows[0]),
                original_row,
                "no partial assignment"
            );
            assert!(!new.row_refresh_states[0].rich);
        }
        match case {
            0 => assert_eq!(new.rows[0].life_stage, LifeStage::Child),
            7 => assert_eq!(new.rows[0].life_stage, LifeStage::Elder),
            11 => assert_eq!(new.rows[0].life_stage, LifeStage::Adult),
            _ => {}
        }
    }
}

#[test]
fn exclusive_capture_preserves_memory_bits_and_missing_vs_empty_distinction() {
    for tick in [0, 2, 3, crate::runtime::memory::MAX_RECENCY_TICKS, u64::MAX] {
        for populated in [false, true] {
            let mut world = World::new();
            let entity = world.spawn((Position::new(2, 3), AgentState::Idle));
            if populated {
                enrich(&mut world, entity);
            }
            let expected = PersonSnapshot::capture(&world, entity, 7, tick).unwrap();
            let got = PersonSnapshot::capture_exclusive(&mut world, entity, 7, tick).unwrap();
            assert_rows_equal(&got, &expected);
            world.insert_one(entity, Memory::new()).unwrap();
            let expected = PersonSnapshot::capture(&world, entity, 7, tick).unwrap();
            let got = PersonSnapshot::capture_exclusive(&mut world, entity, 7, tick).unwrap();
            assert_rows_equal(&got, &expected);
            assert_eq!(got.memory_entries, Some(0));
            assert_eq!(
                got.memory_weights.map(f64::to_bits),
                [(-0.0_f64).to_bits(); 6]
            );
        }
    }
}
