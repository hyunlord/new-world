//! Stock acquisition requires a real transport destination; urgent meals do not.
use super::harvest_stock_tests::{fixture, mind};
use sim_core::colony::{Job, JobKind, ReserveKey, WorkPriorities, WorkType};
use sim_core::components::{AgentState, Hunger};
use sim_engine::RuntimeSystem;

#[test]
fn stock_harvest_waits_for_the_scripted_stockpile() {
    let (mut world, mut res, entity) = fixture();
    res.colony.zones.clear();
    res.colony.zone_of_tile.clear();
    res.colony.touch(false);
    let mut policy = mind();
    let mut movement = crate::runtime::agent::AgentMovementSystem::new();
    let mut jobs = crate::runtime::jobs::JobExecuteSystem::new();
    for tick in 0..360 {
        res.current_tick = tick;
        movement.tick(&mut world, &mut res);
        policy
            .try_tick(&mut world, &mut res)
            .expect("stock decision");
        assert!(
            !world
                .get::<&Job>(entity)
                .is_ok_and(|job| matches!(job.kind, JobKind::Harvest { .. })),
            "stock harvest without storage at tick {tick}"
        );
        jobs.tick(&mut world, &mut res);
    }
    assert_eq!(res.food.tallies.harvested_units, 0);
    res.current_tick = 360;
    res.colony.zones.insert(
        1,
        sim_core::colony::Zone {
            id: 1,
            kind: sim_core::colony::ZoneKind::Stockpile,
            tiles: std::collections::BTreeSet::from([(6, 6)]),
        },
    );
    res.colony.zone_of_tile.insert((6, 6), 1);
    res.colony.touch(false);
    // The existing continuation contract re-evaluates work on the actor's
    // absolute eight-tick phase: agent7 first reaches it at tick367.
    for tick in 360..=367 {
        res.current_tick = tick;
        movement.tick(&mut world, &mut res);
        policy
            .try_tick(&mut world, &mut res)
            .expect("new stockpile decision");
        if world
            .get::<&Job>(entity)
            .is_ok_and(|job| matches!(job.kind, JobKind::Harvest { .. }))
        {
            return;
        }
        jobs.tick(&mut world, &mut res);
    }
    panic!("deliverable stock harvest must start by the next normal decision phase");
}

#[test]
fn stock_harvest_rejects_reserved_or_unreachable_storage() {
    for unreachable in [false, true] {
        let (mut world, mut res, entity) = fixture();
        if unreachable {
            let mut map = sim_core::nav::CostMap::new(8, 8);
            for y in 0..8 {
                map.set((4, y), sim_core::nav::IMPASSABLE);
            }
            res.nav.install(map);
        } else {
            res.colony.reservations.insert(ReserveKey::Cell((6, 6)), 99);
        }
        mind()
            .try_tick(&mut world, &mut res)
            .expect("blocked storage");
        assert!(!world
            .get::<&Job>(entity)
            .is_ok_and(|job| matches!(job.kind, JobKind::Harvest { .. })));
    }
}

#[test]
fn urgent_harvest_still_starts_without_storage() {
    let (mut world, mut res, entity) = fixture();
    res.colony.zones.clear();
    res.colony.zone_of_tile.clear();
    crate::runtime::food::items::take(
        &mut world,
        &mut res,
        sim_core::colony::food::FoodOwner::Ground((6, 6)),
        1,
        0,
    )
    .expect("remove fixture food");
    res.colony.touch(false);
    world.get::<&mut Hunger>(entity).expect("hunger").value = 60.0;
    mind()
        .try_tick(&mut world, &mut res)
        .expect("urgent decision");
    assert!(world
        .get::<&Job>(entity)
        .is_ok_and(|job| matches!(job.kind, JobKind::Harvest { .. })));
}

#[test]
fn disabled_haul_does_not_select_a_noop_stock_action() {
    let (mut world, mut res, entity) = fixture();
    let mut priorities = WorkPriorities::defaults(false);
    priorities.levels[WorkType::Haul.code() as usize] = 0;
    world.insert_one(entity, priorities).expect("priorities");
    mind()
        .try_tick(&mut world, &mut res)
        .expect("alternative decision");
    assert!(world.get::<&Job>(entity).is_err());
    assert_ne!(
        *world.get::<&AgentState>(entity).expect("state"),
        AgentState::Idle,
        "available wandering work must proceed instead of a rejected stock action"
    );
}

#[test]
fn stock_harvest_rejects_storage_with_no_compatible_room() {
    let (mut world, mut res, entity) = fixture();
    crate::runtime::food::items::take(
        &mut world,
        &mut res,
        sim_core::colony::food::FoodOwner::Ground((6, 6)),
        1,
        0,
    )
    .expect("remove fixture food");
    let log = sim_core::colony::ItemSpec::raw(
        sim_core::colony::ItemKind::from_name("log").expect("log"),
        None,
    );
    crate::runtime::food::items::create_ground(
        &mut res,
        (6, 6),
        log,
        1,
        0,
        crate::runtime::food::items::FoodSource::Debug,
    )
    .expect("incompatible storage");
    mind().try_tick(&mut world, &mut res).expect("full storage");
    assert!(!world
        .get::<&Job>(entity)
        .is_ok_and(|job| matches!(job.kind, JobKind::Harvest { .. })));
}

#[test]
fn steward_immediate_budget_caps_proactive_harvest_but_off_keeps_legacy_target() {
    for enabled in [false, true] {
        let (mut world, mut res, entity) = fixture();
        res.steward.enabled = enabled;
        res.steward.scope = sim_core::colony::steward::StewardScope::Food;
        crate::runtime::food::items::create_ground(
            &mut res,
            (6, 6),
            sim_core::colony::ItemSpec::raw(
                sim_core::colony::ItemKind::from_name("roots").expect("roots"),
                None,
            ),
            3,
            0,
            crate::runtime::food::items::FoodSource::Debug,
        )
        .expect("four edible units exceed the one-adult unroofed target");
        mind().try_tick(&mut world, &mut res).expect("decision");
        let harvesting = world
            .get::<&Job>(entity)
            .is_ok_and(|job| matches!(job.kind, JobKind::Harvest { .. }));
        assert_eq!(
            harvesting, !enabled,
            "only steward food scope replaces the legacy stock target"
        );
    }
}

#[test]
fn steward_budget_preserves_hunger_acquisition_when_accepted_output_is_pending() {
    let (mut world, mut res, entity) = fixture();
    res.steward.enabled = true;
    res.steward.scope = sim_core::colony::steward::StewardScope::Food;
    crate::runtime::food::items::take(
        &mut world,
        &mut res,
        sim_core::colony::food::FoodOwner::Ground((6, 6)),
        1,
        0,
    )
    .expect("remove meal");
    res.colony
        .designations
        .insert((3, 2), sim_core::colony::Designation::Harvest);
    world.get::<&mut Hunger>(entity).expect("hunger").value = 60.0;
    mind()
        .try_tick(&mut world, &mut res)
        .expect("urgent decision");
    assert!(world
        .get::<&Job>(entity)
        .is_ok_and(|job| matches!(job.kind, JobKind::Harvest { plant: (2, 2) })));
}

#[test]
fn steward_budget_admits_only_one_of_two_same_tick_harvesters() {
    use sim_core::components::{
        Agent, Inventory, Lineage, Position, Sleep, Social, Temperament, Thirst,
    };
    let (mut world, mut res, _) = fixture();
    res.steward.enabled = true;
    res.steward.scope = sim_core::colony::steward::StewardScope::Food;
    world.spawn((
        Agent { id: 8 },
        Position::new(3, 2),
        AgentState::Idle,
        Hunger::new(10.0, 0.05),
        Thirst::new(0.0, 0.0),
        Sleep::new(0.0, 0.0),
        Social::new(0.0, 0.0),
        Inventory::default(),
        Lineage { key: 8, births: 0 },
        Temperament::from_axes([500; 4]),
        crate::runtime::agent::MovementRng::new(8),
    ));
    res.settlements
        .get_mut(&1)
        .expect("settlement")
        .add_member_agent(8);
    mind()
        .try_tick(&mut world, &mut res)
        .expect("two-person decision");
    assert_eq!(
        world
            .query::<&Job>()
            .iter()
            .filter(|(_, job)| matches!(job.kind, JobKind::Harvest { .. }))
            .count(),
        1
    );
}

#[test]
fn steward_budget_does_not_double_credit_designated_running_harvest() {
    let (mut world, mut res, entity) = fixture();
    res.steward.enabled = true;
    res.steward.scope = sim_core::colony::steward::StewardScope::Food;
    world.spawn((sim_core::components::Agent { id: 8 },));
    super::harvest_stock_tests::ready_harvest(&mut world, &mut res, entity);
    res.food.plants.get_mut(&(2, 2)).expect("plant").units_left = 2;
    let budget = super::harvest_food::HarvestBudget::capture(&world, &res).expect("budget");
    assert!(
        budget.permits(&res, (3, 2)),
        "target5 - stored1 - unique output2 still leaves2; double credit would close it"
    );
}

#[test]
fn accepted_harvest_keeps_proactive_labour_without_opening_new_commitments() {
    let (mut world, mut res, entity) = fixture();
    res.steward.enabled = true;
    res.steward.scope = sim_core::colony::steward::StewardScope::Food;
    // The farther marked plant covers the remaining budget. The nearer unmarked
    // plant must not win merely because proactive work becomes eligible again.
    res.colony
        .designations
        .insert((3, 2), sim_core::colony::Designation::Harvest);
    let read = |world: &hecs::World, res: &sim_engine::SimResources| {
        let context = super::context::TickContext::capture(world, res).expect("context");
        let provider = super::facts::BUILTIN_PROVIDERS
            .iter()
            .find(|provider| provider.name == "stock_shortfall")
            .expect("registered fact");
        (provider.read)(
            context.row(7).expect("person"),
            &context,
            res,
            provider.name,
        )
        .expect("fact")
    };
    assert_eq!(
        read(&world, &res),
        1,
        "accepted unassigned output still needs labour"
    );
    mind()
        .try_tick(&mut world, &mut res)
        .expect("execute accepted work");
    assert!(world
        .get::<&Job>(entity)
        .is_ok_and(|job| matches!(job.kind, JobKind::Harvest { plant: (3, 2) })));
    assert!(
        !res.colony.designations.contains_key(&(2, 2)),
        "no excess new commitment"
    );
    assert_eq!(
        read(&world, &res),
        0,
        "assigned output cannot recruit a duplicate worker"
    );
}
