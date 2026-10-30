//! EntityRef reuse compared with the original ordered World::get capture.
use super::*;
use sim_core::colony::people::DraftOrder;
use sim_core::colony::{ItemKind, JobKind, JobStage};
use sim_core::components::{ActivityKind, Lineage, Memory, MemoryArm, MemoryEntry};

impl PersonSnapshot {
    fn capture_legacy(
        world: &World,
        entity: Entity,
        agent: AgentId,
        tick: u64,
    ) -> Result<Self, ContextError> {
        let position = world.get::<&Position>(entity).map(|v| *v).map_err(|_| {
            ContextError::MissingComponent {
                agent,
                component: "Position",
            }
        })?;
        let state = world.get::<&AgentState>(entity).map(|v| *v).map_err(|_| {
            ContextError::MissingComponent {
                agent,
                component: "AgentState",
            }
        })?;
        let life_stage = if world.get::<&Child>(entity).is_ok() {
            LifeStage::Child
        } else if world.get::<&Elder>(entity).is_ok() {
            LifeStage::Elder
        } else {
            LifeStage::Adult
        };
        let memory = world.get::<&sim_core::components::Memory>(entity).ok();
        let (memory_weights, memory_causes) = memory
            .as_ref()
            .map_or(([0.0; 6], [None; 6]), |memory| memory_values(memory, tick));
        let memory_entries = memory.as_ref().map(|memory| memory.entries.len());
        Ok(Self {
            agent,
            entity,
            position,
            state,
            life_stage,
            hunger: world
                .get::<&Hunger>(entity)
                .ok()
                .map(|v| f64::from(v.value)),
            thirst: world.get::<&Thirst>(entity).ok().map(|v| v.value),
            fatigue: world.get::<&Sleep>(entity).ok().map(|v| v.fatigue),
            social: world.get::<&Social>(entity).ok().map(|v| v.loneliness),
            temperament: world.get::<&Temperament>(entity).ok().map(|v| *v),
            inventory_total: world.get::<&Inventory>(entity).ok().map(|v| v.total()),
            inventory_food: world
                .get::<&Inventory>(entity)
                .ok()
                .map(|v| v.get(ResourceKind::Food)),
            carry: world.get::<&Carry>(entity).ok().map(|v| *v),
            job: world.get::<&Job>(entity).ok().map(|v| *v),
            haul_enabled: world
                .get::<&WorkPriorities>(entity)
                .ok()
                .is_none_or(|p| p.level(WorkType::Haul) > 0),
            drafted: world.get::<&Drafted>(entity).is_ok(),
            fleeing: world.get::<&Fleeing>(entity).ok().map(|v| *v),
            cold: world.get::<&Cold>(entity).is_ok(),
            sick: world.get::<&Sick>(entity).is_ok(),
            mood: world.get::<&Mood>(entity).ok().map(|v| *v),
            hp: world.get::<&BodyHealth>(entity).ok().map(|v| v.hp),
            activity: world.get::<&Activity>(entity).ok().map(|v| *v),
            activity_rest: world.get::<&ActivityRest>(entity).ok().map(|v| *v),
            lineage: world
                .get::<&sim_core::components::Lineage>(entity)
                .ok()
                .map(|v| v.key),
            memory_weights,
            memory_causes,
            memory_entries,
        })
    }
}

fn assert_snapshot(actual: &PersonSnapshot, expected: &PersonSnapshot) {
    macro_rules! fields {
        ($($name:ident),* $(,)?) => { $(assert_eq!(actual.$name, expected.$name, stringify!($name));)* };
    }
    fields!(
        agent,
        entity,
        position,
        state,
        life_stage,
        temperament,
        inventory_total,
        inventory_food,
        carry,
        job,
        haul_enabled,
        drafted,
        fleeing,
        cold,
        sick,
        mood,
        activity,
        activity_rest,
        memory_entries,
        lineage,
        memory_causes
    );
    for (a, b) in [
        (actual.hunger, expected.hunger),
        (actual.thirst, expected.thirst),
        (actual.fatigue, expected.fatigue),
        (actual.social, expected.social),
        (actual.hp, expected.hp),
    ] {
        assert_eq!(a.map(f64::to_bits), b.map(f64::to_bits));
    }
    assert_eq!(
        actual.memory_weights.map(f64::to_bits),
        expected.memory_weights.map(f64::to_bits)
    );
}

fn compare(world: &World, entity: Entity, tick: u64) -> PersonSnapshot {
    let actual = PersonSnapshot::capture(world, entity, 7, tick).unwrap();
    let expected = PersonSnapshot::capture_legacy(world, entity, 7, tick).unwrap();
    assert_snapshot(&actual, &expected);
    actual
}

#[test]
fn sparse_and_full_capture_match_oracle_including_memory_bits() {
    let mut world = World::new();
    let entity = world.spawn((Agent { id: 7 }, Position::new(1, 2), AgentState::Idle));
    let sparse = compare(&world, entity, 10);
    assert_eq!(sparse.hunger, None);
    assert_eq!(sparse.inventory_total, None);
    assert_eq!(sparse.memory_entries, None);
    let mut inventory = Inventory::default();
    inventory.add(ResourceKind::Food, 3);
    let mut memory = Memory::new();
    for (id, arm) in [
        MemoryArm::Hunger,
        MemoryArm::Thirst,
        MemoryArm::Fatigue,
        MemoryArm::Construction,
        MemoryArm::Social,
        MemoryArm::Combat,
    ]
    .into_iter()
    .enumerate()
    {
        memory.insert(MemoryEntry::new(id as u64, 2, -0.25, 0.8, arm));
        memory.insert(MemoryEntry::new(id as u64 + 10, 2, 0.25, 0.8, arm));
    }
    world
        .insert(
            entity,
            (
                Hunger::new(12.5, 0.0),
                Thirst::new(-0.0, 0.0),
                Sleep::new(23.5, 0.0),
                Social::new(34.5, 0.0),
                Temperament::from_axes([100, 200, 300, 400]),
                inventory,
                BodyHealth::new(),
                memory,
                Lineage { key: 99, births: 2 },
            ),
        )
        .unwrap();
    world
        .insert(
            entity,
            (
                Child { born_tick: 1 },
                Elder,
                Cold,
                Sick { until: 30 },
                Mood::default(),
                Drafted {
                    since: 1,
                    order: DraftOrder::Hold,
                    arming: None,
                },
                Fleeing {
                    since: 1,
                    until: 30,
                    from: 4,
                    to: (3, 4),
                },
                ActivityRest { until_tick: 20 },
                Activity {
                    kind: ActivityKind::Play,
                    settlement: 1,
                    place: (3, 4),
                    chosen_tick: 2,
                    arrived_tick: Some(3),
                    until_tick: Some(40),
                },
            ),
        )
        .unwrap();
    world
        .insert(
            entity,
            (
                Carry {
                    kind: ItemKind::Log,
                    count: 2,
                    quality: 1,
                    material: None,
                },
                Job {
                    kind: JobKind::Chop { tree: (3, 3) },
                    stage: JobStage::ToSource,
                    taken_at: 2,
                    amount: 0,
                    item: ItemKind::Log,
                    material: None,
                    fetch: None,
                    best_distance: 4,
                    progress_tick: 2,
                },
            ),
        )
        .unwrap();
    for tick in [0, 10, u64::MAX] {
        let actual = compare(&world, entity, tick);
        assert_eq!(actual.life_stage, LifeStage::Child);
        assert_eq!(actual.memory_entries, Some(12));
        assert_eq!(actual.inventory_food, Some(3));
    }
    world.remove_one::<Child>(entity).unwrap();
    assert_eq!(compare(&world, entity, 10).life_stage, LifeStage::Elder);
}

#[test]
fn missing_required_and_despawned_keep_exact_errors() {
    let mut world = World::new();
    let neither = world.spawn(());
    let no_position = world.spawn((AgentState::Idle,));
    let no_state = world.spawn((Position::new(1, 1),));
    let gone = world.spawn((Position::new(1, 1), AgentState::Idle));
    world.despawn(gone).unwrap();
    for (entity, component) in [
        (neither, "Position"),
        (no_position, "Position"),
        (no_state, "AgentState"),
        (gone, "Position"),
    ] {
        let expected = ContextError::MissingComponent {
            agent: 7,
            component,
        };
        assert_eq!(
            PersonSnapshot::capture(&world, entity, 7, 10).unwrap_err(),
            expected
        );
        assert_eq!(
            PersonSnapshot::capture_legacy(&world, entity, 7, 10).unwrap_err(),
            expected
        );
    }
}

#[test]
fn refresh_reads_live_changes_and_failure_preserves_row_and_version() {
    let mut world = World::new();
    let entity = world.spawn((
        Agent { id: 7 },
        Position::new(1, 1),
        AgentState::Idle,
        Hunger::new(1.0, 0.0),
    ));
    let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let mut context = TickContext::capture(&world, &resources).unwrap();
    world.get::<&mut Position>(entity).unwrap().x = 4;
    world.get::<&mut Hunger>(entity).unwrap().value = 75.0;
    let version = context.versions[Domain::Person as usize];
    context.refresh_person(&world, 7).unwrap();
    assert_eq!(context.versions[Domain::Person as usize], version + 1);
    assert_eq!(context.row(7).unwrap().hunger, Some(75.0));
    assert_snapshot(
        context.row(7).unwrap(),
        &PersonSnapshot::capture_legacy(&world, entity, 7, context.tick).unwrap(),
    );
    world.remove_one::<Hunger>(entity).unwrap();
    context.refresh_person(&world, 7).unwrap();
    assert_eq!(context.row(7).unwrap().hunger, None);
    let saved = context.row(7).unwrap().clone();
    let version = context.versions[Domain::Person as usize];
    world.remove_one::<AgentState>(entity).unwrap();
    assert_eq!(
        context.refresh_person(&world, 7),
        Err(ContextError::MissingComponent {
            agent: 7,
            component: "AgentState"
        })
    );
    assert_eq!(context.versions[Domain::Person as usize], version);
    assert_snapshot(context.row(7).unwrap(), &saved);
}

#[test]
fn borrow_conflicts_still_panic_and_child_still_short_circuits_elder() {
    let mut world = World::new();
    let entity = world.spawn((
        Position::new(1, 1),
        AgentState::Idle,
        Child { born_tick: 0 },
        Elder,
    ));
    let elder = world.get::<&mut Elder>(entity).unwrap();
    assert_eq!(compare(&world, entity, 0).life_stage, LifeStage::Child);
    drop(elder);
    let state = world.get::<&mut AgentState>(entity).unwrap();
    for old in [false, true] {
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if old {
                PersonSnapshot::capture_legacy(&world, entity, 7, 0)
            } else {
                PersonSnapshot::capture(&world, entity, 7, 0)
            }
        }))
        .is_err());
    }
    drop(state);
    compare(&world, entity, 0);
}
