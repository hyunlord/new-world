//! Red-first tests use only P7 APIs; receipts must arise from actual operations.
use super::{events, items, meals};
use hecs::World;
use sim_core::chronicle::{CauseRef, ChronicleKind};
use sim_core::colony::food::{FoodOwner, PlantState};
use sim_core::colony::{
    content::installed_or_bundled, Designation, ItemKind, ItemSpec, Job, JobKind, JobStage,
    ReserveKey, Zone, ZoneKind,
};
use sim_core::components::{Agent, AgentState, Hunger, Position, TargetKind};
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimResources};

fn scene() -> (World, SimResources, hecs::Entity) {
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.nav.install(sim_core::nav::CostMap::new(8, 8));
    let mut world = World::new();
    let entity = world.spawn((
        Agent { id: 7 },
        Position::new(2, 2),
        AgentState::Idle,
        Hunger::new(100.0, 0.05),
    ));
    (world, res, entity)
}

fn assert_operation_parent(res: &SimResources, kind: ChronicleKind, code: u32) {
    let (index, _) = res
        .chronicle
        .entries()
        .iter()
        .enumerate()
        .find(|(_, entry)| entry.kind == kind)
        .expect("actual milestone");
    let Some(CauseRef::Entry(parent)) = res.chronicle.cause_of(index as u32) else {
        panic!("successful milestone must name its actual operation parent");
    };
    let parent = res.chronicle.entry(parent).expect("existing operation");
    assert_eq!(parent.kind.as_str(), "food_operation");
    assert_eq!(parent.cause, code);
    assert_eq!(parent.tick, res.current_tick);
}

#[test]
fn first_cooked_meal_has_actual_intake_parent() {
    let (mut world, mut res, entity) = scene();
    let item = ItemKind::from_name("roast_meat").expect("cooked food");
    items::create_ground(
        &mut res,
        (2, 2),
        ItemSpec::raw(item, None),
        1,
        0,
        items::FoodSource::Debug,
    )
    .unwrap();
    meals::consume(&mut world, &mut res, entity, FoodOwner::Ground((2, 2)), 0).unwrap();
    assert_eq!(res.food.tallies.meals, 1);
    assert!(!res.colony.ground_items.contains_key(&(2, 2)));
    assert_operation_parent(&res, ChronicleKind::FirstCookedMeal, 1);
}

#[test]
fn first_dried_storage_has_successful_placement_parent() {
    let (mut world, mut res, _) = scene();
    let item = ItemKind::from_name("dried_berries").expect("preserved food");
    res.colony.zones.insert(
        1,
        Zone {
            id: 1,
            kind: ZoneKind::Stockpile,
            tiles: [(3, 2)].into_iter().collect(),
        },
    );
    res.colony.zone_of_tile.insert((3, 2), 1);
    items::create_ground(
        &mut res,
        (2, 2),
        ItemSpec::raw(item, None),
        1,
        0,
        items::FoodSource::Debug,
    )
    .unwrap();
    assert!(res.food.firsts.dried_store.is_none());
    items::transfer(
        &mut world,
        &mut res,
        FoodOwner::Ground((2, 2)),
        FoodOwner::Ground((3, 2)),
        1,
        0,
    )
    .unwrap();
    assert_operation_parent(&res, ChronicleKind::FirstDriedStore, 2);
}

#[test]
fn packed_map_harvest_failure_survives_generic_cancel_into_hunger_chain() {
    let (mut world, mut res, entity) = scene();
    let species = installed_or_bundled().plant_by_name("wild_root").unwrap();
    res.food.plants.insert(
        (2, 2),
        PlantState {
            species,
            units_left: 5,
            next_regrow_day: 0,
        },
    );
    res.colony.designations.insert((2, 2), Designation::Harvest);
    let wood = ItemKind::from_name("firewood").unwrap();
    let count = crate::runtime::jobs::items::stack_max(&res, wood);
    for y in 0..8 {
        for x in 0..8 {
            items::create_ground(
                &mut res,
                (x, y),
                ItemSpec::raw(wood, None),
                count,
                0,
                items::FoodSource::Debug,
            )
            .unwrap();
        }
    }
    let item = installed_or_bundled().plant(species).unwrap().yield_item;
    world
        .insert(
            entity,
            (
                Job {
                    kind: JobKind::Harvest { plant: (2, 2) },
                    stage: JobStage::Working { ticks_left: 1 },
                    taken_at: 0,
                    amount: 5,
                    item,
                    material: None,
                    fetch: None,
                    best_distance: 0,
                    progress_tick: 0,
                },
                AgentState::Consuming {
                    target: TargetKind::Job,
                },
            ),
        )
        .unwrap();
    res.colony.reservations.insert(ReserveKey::Plant((2, 2)), 7);
    res.current_tick = 1;
    crate::runtime::jobs::JobExecuteSystem::new().tick(&mut world, &mut res);
    assert!(world.get::<&Job>(entity).is_err());
    assert_eq!(res.food.plants[&(2, 2)].units_left, 5);
    assert_eq!(res.food.tallies.harvested_units, 0);
    let root = events::hunger_death(&world, &mut res, entity, 7, (2, 2), 1).unwrap();
    let mut entry = root;
    let mut found = false;
    for _ in 0..12 {
        match res.chronicle.cause_of(entry) {
            Some(CauseRef::Entry(parent)) => {
                entry = parent;
                let row = res.chronicle.entry(entry).unwrap();
                if row.kind.as_str() == "food_operation" {
                    assert_eq!(row.subject, Some(7));
                    assert_eq!(row.cause, 0);
                    found = true;
                    break;
                }
            }
            Some(CauseRef::Sample { sample, .. }) => {
                entry = res.chronicle.sample(sample).unwrap().entry.unwrap();
            }
            _ => break,
        }
    }
    assert!(
        found,
        "actual failed harvest must be reachable, not replaced by UNKNOWN"
    );
}
