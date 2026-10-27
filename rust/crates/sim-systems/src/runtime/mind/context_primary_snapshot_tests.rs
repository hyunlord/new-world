//! One-shot primary capture acknowledgement against the original refresh.
use super::*;
use crate::runtime::mind::facts;
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

fn capture(world: &World, resources: &SimResources) -> TickContext {
    TickContext::capture_continuation_with_claims(world, resources, BTreeMap::new(), None).unwrap()
}
fn same(a: &TickContext, b: &TickContext) {
    for (a, b) in a.rows.iter().zip(&b.rows) {
        assert_rows_equal(a, b);
    }
    assert_eq!(a.row_refresh_states, b.row_refresh_states);
    assert_eq!(a.versions, b.versions);
}

#[test]
fn verified_primary_matches_eight_query_and_is_one_shot() {
    assert_eq!(std::mem::size_of::<RowRefreshState>(), 2);
    for reverse in [false, true] {
        for populated in [false, true] {
            let mut world = World::new();
            for id in if reverse { [91, 7, 2] } else { [2, 7, 91] } {
                let entity = world.spawn((Agent { id }, Position::new(1, 1), AgentState::Idle));
                if populated {
                    enrich(&mut world, entity);
                }
            }
            let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
            let mut old = capture(&world, &resources);
            let mut new = capture(&world, &resources);
            for id in [2, 7, 91] {
                old.refresh_primary_continuation_exclusive(&mut world, id)
                    .unwrap();
                assert_eq!(new.acknowledge_primary_capture(id), Ok(true));
                same(&new, &old);
                let before = new.versions;
                assert_eq!(new.acknowledge_primary_capture(id), Ok(false));
                assert_eq!(before, new.versions);
            }
            // A fresh capture has fresh one-shot state; the old one is never rearmed.
            let mut next = capture(&world, &resources);
            assert_eq!(next.acknowledge_primary_capture(7), Ok(true));
            assert_eq!(new.acknowledge_primary_capture(7), Ok(false));
            let mut shared = TickContext::capture_continuation(&world, &resources).unwrap();
            assert_eq!(shared.acknowledge_primary_capture(7), Ok(false));
            let mut rich = TickContext::capture(&world, &resources).unwrap();
            assert_eq!(rich.acknowledge_primary_capture(7), Ok(false));
        }
    }
}

#[test]
fn any_secondary_alias_keeps_capture_and_social_payload_then_falls_back() {
    let mut observed_real_first = BTreeSet::new();
    for missing in 0..3 {
        for alias_first in [false, true] {
            let mut world = World::new();
            let spawn_alias = |world: &mut World| {
                let alias = world.spawn((Agent { id: 7 },));
                enrich(world, alias);
                if missing == 1 {
                    world.insert_one(alias, Position::new(6, 6)).unwrap();
                }
                if missing == 2 {
                    world.insert_one(alias, AgentState::Idle).unwrap();
                }
            };
            if alias_first {
                spawn_alias(&mut world);
            }
            let real = world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
            if !alias_first {
                spawn_alias(&mut world);
            }
            let first = world.query::<&Agent>().iter().next().unwrap().0;
            observed_real_first.insert(first == real);
            let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
            let mut old = TickContext::capture_continuation(&world, &resources).unwrap();
            let mut new = capture(&world, &resources);
            assert_rows_equal(&old.rows[0], &new.rows[0]);
            for context in [&mut old, &mut new] {
                facts::prepare_social_snapshot(context, &world, &resources).unwrap();
            }
            assert_eq!(
                format!("{:?}", old.derived_slots),
                format!("{:?}", new.derived_slots)
            );
            assert_eq!(
                format!("{:?}", old.target_slots),
                format!("{:?}", new.target_slots)
            );
            assert_eq!(new.acknowledge_primary_capture(7), Ok(false));
            old.refresh_primary_continuation_exclusive(&mut world, 7)
                .unwrap();
            new.refresh_primary_continuation_exclusive(&mut world, 7)
                .unwrap();
            same(&new, &old);
            assert_eq!(new.rows[0].life_stage, LifeStage::Adult);
            assert!(new.rows[0].activity.is_none());
            assert!(!new.rows[0].drafted);
        }
    }
    assert_eq!(observed_real_first, BTreeSet::from([false, true]));
}

#[test]
fn overflow_and_unknown_agent_preserve_acknowledgement_atomicity() {
    let mut world = World::new();
    world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
    let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let mut context = capture(&world, &resources);
    context.versions[Domain::Person as usize] = u64::MAX;
    let before = format!("{:?}", context.rows);
    let metadata = context.row_refresh_states.clone();
    assert_eq!(
        context.acknowledge_primary_capture(999),
        Err(ContextError::UnknownAgent(999))
    );
    assert_eq!(
        context.acknowledge_primary_capture(7),
        Err(ContextError::VersionOverflow(Domain::Person))
    );
    assert_eq!(metadata, context.row_refresh_states);
    assert_eq!(before, format!("{:?}", context.rows));
    context.versions[Domain::Person as usize] = u64::MAX - 1;
    assert_eq!(context.acknowledge_primary_capture(7), Ok(true));
    assert_eq!(context.versions[Domain::Person as usize], u64::MAX);
    assert_eq!(context.acknowledge_primary_capture(7), Ok(false));
}

#[test]
fn every_person_mutator_revokes_pending_even_for_noop_or_failed_refresh() {
    for action in 0..9 {
        let mut world = World::new();
        let entity = world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
        let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        let mut context = capture(&world, &resources);
        match action {
            0 => context.set_activity_snapshot(7, None).unwrap(),
            1 => context
                .patch_person(
                    7,
                    PersonUpdate {
                        position: Position::new(1, 1),
                        state: AgentState::Idle,
                        hunger: None,
                        thirst: None,
                        fatigue: None,
                        social: None,
                        inventory_total: None,
                        inventory_food: None,
                    },
                )
                .unwrap(),
            2 => context.refresh_continuation(&world, 7).unwrap(),
            3 => context
                .refresh_continuation_exclusive(&mut world, 7)
                .unwrap(),
            4 => context
                .refresh_primary_continuation_exclusive(&mut world, 7)
                .unwrap(),
            5 => context.refresh_person(&world, 7).unwrap(),
            6 => context.refresh_person_exclusive(&mut world, 7).unwrap(),
            7 => {
                context
                    .refresh_continuation_exclusive(&mut world, 7)
                    .unwrap();
                context
                    .upgrade_continuation_exclusive(&mut world, 7)
                    .unwrap();
            }
            8 => {
                world.despawn(entity).unwrap();
                assert_eq!(
                    context.refresh_primary_continuation_exclusive(&mut world, 7),
                    Err(ContextError::UnknownAgent(7))
                );
            }
            _ => unreachable!(),
        }
        assert_eq!(
            context.acknowledge_primary_capture(7),
            Ok(false),
            "action {action}"
        );
    }
}

#[test]
fn unverified_fallback_preserves_missing_component_and_entity_errors() {
    for missing in 0..3 {
        let mut world = World::new();
        let entity = world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
        let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        let mut old = TickContext::capture_continuation(&world, &resources).unwrap();
        let mut new = TickContext::capture_continuation(&world, &resources).unwrap();
        match missing {
            0 => {
                world.remove_one::<Position>(entity).unwrap();
            }
            1 => {
                world.remove_one::<AgentState>(entity).unwrap();
            }
            2 => {
                world.despawn(entity).unwrap();
            }
            _ => unreachable!(),
        }
        assert_eq!(new.acknowledge_primary_capture(7), Ok(false));
        assert_eq!(
            new.refresh_primary_continuation_exclusive(&mut world, 7),
            old.refresh_primary_continuation_exclusive(&mut world, 7)
        );
        same(&new, &old);
    }
}

#[test]
fn duplicate_captured_ids_still_fail_before_metadata_is_used() {
    let mut world = World::new();
    for _ in 0..2 {
        world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
    }
    let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let result =
        TickContext::capture_continuation_with_claims(&world, &resources, BTreeMap::new(), None);
    assert!(matches!(result, Err(ContextError::DuplicateAgent(7))));
}
