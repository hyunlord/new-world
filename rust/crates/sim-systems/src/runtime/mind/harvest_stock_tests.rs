//! Proactive food work restores a complete physical harvest-to-storage transaction.
use super::{ColonyMindSystem, MindProgram};
use crate::runtime::food::items::{self, FoodSource};
use sim_core::colony::food::{FoodOwner, PlantState};
use sim_core::colony::{
    content::installed_or_bundled, Designation, ItemKind, ItemSpec, Job, JobKind, JobStage,
    ReserveKey, WorkPriorities, WorkType, Zone, ZoneKind,
};
use sim_core::components::{
    Agent, AgentState, Hunger, Inventory, Lineage, Position, SeekTarget, Settlement, Sleep, Social,
    TargetKind, Temperament, Thirst,
};
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimResources};

const PLANT: (u32, u32) = (2, 2);
const STORE: (u32, u32) = (6, 6);

pub(super) fn fixture() -> (hecs::World, SimResources, hecs::Entity) {
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.current_tick = 720;
    res.nav.install(sim_core::nav::CostMap::new(8, 8));
    let mut world = hecs::World::new();
    let entity = world.spawn((
        Agent { id: 7 },
        Position::new(2, 2),
        AgentState::Idle,
        Hunger::new(10.0, 0.05),
        Thirst::new(0.0, 0.0),
        Sleep::new(0.0, 0.0),
        Social::new(0.0, 0.0),
        Inventory::default(),
        Lineage { key: 7, births: 0 },
        Temperament::from_axes([500; 4]),
        crate::runtime::agent::MovementRng::new(7),
    ));
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = STORE;
    settlement.add_member_agent(7);
    res.settlements.insert(1, settlement);
    res.colony.zones.insert(
        1,
        Zone {
            id: 1,
            kind: ZoneKind::Stockpile,
            tiles: std::collections::BTreeSet::from([STORE]),
        },
    );
    res.colony.zone_of_tile.insert(STORE, 1);
    let roots = ItemSpec::raw(ItemKind::from_name("roots").expect("roots"), None);
    items::create_ground(&mut res, STORE, roots, 1, 0, FoodSource::Debug).expect("initial store");
    for tile in [PLANT, (3, 2)] {
        res.food.plants.insert(
            tile,
            PlantState {
                species: installed_or_bundled()
                    .plant_by_name("wild_root")
                    .expect("plant"),
                units_left: 5,
                next_regrow_day: 0,
            },
        );
    }
    res.spatial.colony.refresh(&res.colony);
    (world, res, entity)
}

pub(super) fn mind() -> ColonyMindSystem {
    ColonyMindSystem::new(
        MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron"))
            .expect("bundled policy"),
    )
}

pub(super) fn ready_harvest(world: &mut hecs::World, res: &mut SimResources, entity: hecs::Entity) {
    let job = Job {
        kind: JobKind::Harvest { plant: PLANT },
        stage: JobStage::Working { ticks_left: 1 },
        taken_at: 710,
        amount: 5,
        item: ItemKind::from_name("roots").expect("roots"),
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: 720,
    };
    world
        .insert(
            entity,
            (
                job,
                AgentState::Consuming {
                    target: TargetKind::Job,
                },
                SeekTarget::new(PLANT),
            ),
        )
        .expect("harvest");
    res.colony.designations.insert(PLANT, Designation::Harvest);
    res.colony.reservations.insert(ReserveKey::Plant(PLANT), 7);
    res.colony.tallies.jobs_taken += 1;
}

#[test]
fn nonhungry_member_harvests_before_stores_run_out() {
    for assigned in [false, true] {
        let (mut world, mut res, entity) = fixture();
        if assigned {
            world
                .insert(
                    entity,
                    (
                        Job {
                            kind: JobKind::Chop { tree: (4, 4) },
                            stage: JobStage::ToSource,
                            taken_at: 700,
                            amount: 0,
                            item: ItemKind::role("default_placeholder"),
                            material: None,
                            fetch: None,
                            best_distance: 2,
                            progress_tick: 720,
                        },
                        AgentState::Seeking {
                            target: TargetKind::Job,
                        },
                        SeekTarget::new((4, 4)),
                    ),
                )
                .expect("old work");
            res.colony.designations.insert((4, 4), Designation::Chop);
            res.colony.reservations.insert(ReserveKey::Tree((4, 4)), 7);
        }
        assert!(!res.steward.enabled);
        assert_eq!(
            crate::runtime::food::reserves::units(&world, &res, 1, false),
            1
        );
        mind()
            .try_tick(&mut world, &mut res)
            .expect("stock decision");
        if assigned {
            assert!(world
                .get::<&Job>(entity)
                .is_ok_and(|job| matches!(job.kind, JobKind::Chop { tree: (4, 4) })));
            assert_eq!(
                res.colony.reservations.get(&ReserveKey::Tree((4, 4))),
                Some(&7)
            );
            assert_eq!(res.colony.designations[&(4, 4)], Designation::Chop);
        } else {
            assert!(world
                .get::<&Job>(entity)
                .is_ok_and(|job| matches!(job.kind, JobKind::Harvest { plant: PLANT })));
            assert_eq!(
                res.colony.reservations.get(&ReserveKey::Plant(PLANT)),
                Some(&7)
            );
        }
    }
}

#[test]
fn completed_food_harvest_reaches_storage_without_new_orders_or_haul_preemption() {
    let (mut world, mut res, entity) = fixture();
    let mut policy = mind();
    policy
        .try_tick(&mut world, &mut res)
        .expect("start stock harvest");
    ready_harvest(&mut world, &mut res, entity);
    let mut jobs = crate::runtime::jobs::JobExecuteSystem::new();
    jobs.tick(&mut world, &mut res);
    assert!(
        world.get::<&Job>(entity).is_ok_and(|job| matches!(
            job.kind,
            JobKind::Haul {
                from: PLANT,
                to: STORE
            }
        )),
        "finished harvest must hand actual output to normal hauling"
    );
    assert!(!res
        .colony
        .reservations
        .contains_key(&ReserveKey::Plant(PLANT)));
    assert_eq!(
        res.colony.reservations.get(&ReserveKey::Stack(PLANT)),
        Some(&7)
    );
    assert_eq!(
        res.colony.reservations.get(&ReserveKey::Cell(STORE)),
        Some(&7)
    );
    let mut movement = crate::runtime::agent::AgentMovementSystem::new();
    for tick in 721..=960 {
        res.current_tick = tick;
        movement.tick(&mut world, &mut res);
        policy
            .try_tick(&mut world, &mut res)
            .expect("transport survives work policy");
        jobs.tick(&mut world, &mut res);
        if crate::runtime::food::reserves::units(&world, &res, 1, false) >= 6 {
            break;
        }
    }
    assert_eq!(
        crate::runtime::food::reserves::units(&world, &res, 1, false),
        6
    );
    assert_eq!(
        res.food.tallies.harvested_units, 5,
        "one harvest, no unbounded loose output"
    );
    assert_eq!(res.food.ages[&FoodOwner::Ground(STORE)].born_day, 0);
    assert!(!res.colony.reservations.values().any(|id| *id == 7));
    crate::runtime::jobs::invariants::check(&world, &res).expect("physical conservation");
}

#[test]
fn harvest_without_storage_or_with_disabled_haul_keeps_actual_ground_output() {
    for disabled in [false, true] {
        let (mut world, mut res, entity) = fixture();
        if disabled {
            let mut priorities = WorkPriorities::defaults(false);
            priorities.levels[WorkType::Haul.code() as usize] = 0;
            world.insert_one(entity, priorities).expect("priority");
        } else {
            res.colony.zones.clear();
            res.colony.zone_of_tile.clear();
            res.colony.touch(false);
        }
        ready_harvest(&mut world, &mut res, entity);
        crate::runtime::jobs::JobExecuteSystem::new().tick(&mut world, &mut res);
        assert!(world.get::<&Job>(entity).is_err());
        assert_eq!(res.colony.ground_items[&PLANT].count, 5);
        assert_eq!(res.food.tallies.harvested_units, 5);
        assert!(!res.colony.reservations.values().any(|id| *id == 7));
        crate::runtime::jobs::invariants::check(&world, &res).expect("conserved fallback");
    }
}
