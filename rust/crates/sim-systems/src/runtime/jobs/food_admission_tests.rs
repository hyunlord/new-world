//! Reservation admission must match actionable food, retaining urgent needs and safety.
use super::JobAssignSystem;
use crate::runtime::food::{choice::FoodChoiceIndex, items};
use hecs::{Entity, World};
use sim_core::colony::food::PlantState;
use sim_core::colony::{Designation, ItemKind, ItemSpec, Job, JobKind, WorkPriorities, WorkType};
use sim_core::components::{Agent, AgentState, Hunger, Position, Sleep, TargetKind, Thirst};
use sim_core::rules::WorldRules;
use sim_core::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimResources};

fn scene() -> (World, SimResources, Entity) {
    let mut world = World::new();
    let mut res = SimResources::new(8, 8, MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.colony.as_mut().unwrap().knowledge = None;
    res.current_tick = 1;
    let species = sim_core::colony::content::installed_or_bundled()
        .plant_by_name("wild_root")
        .unwrap();
    res.food.plants.insert(
        (3, 2),
        PlantState {
            species,
            units_left: 2,
            next_regrow_day: u32::MAX,
        },
    );
    res.colony.designations.insert((3, 2), Designation::Harvest);
    res.colony.touch(false);
    let mut priorities = WorkPriorities {
        levels: [0; WorkType::ALL.len()],
    };
    priorities.levels[WorkType::Haul.code() as usize] = 1;
    let person = world.spawn((
        Agent { id: 1 },
        Position::new(2, 2),
        AgentState::Idle,
        Hunger::new(90.0, 0.0),
        priorities,
    ));
    (world, res, person)
}

#[test]
fn hungry_worker_can_claim_ripe_harvest_when_no_meal_exists() {
    let (mut world, mut res, person) = scene();
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(person).unwrap().kind,
        JobKind::Harvest { plant: (3, 2) }
    ));
    assert_eq!(world.get::<&Hunger>(person).unwrap().value, 90.0);
}

#[test]
fn edible_food_defers_new_reservation_to_authored_meal_policy() {
    let (mut world, mut res, person) = scene();
    let roots = ItemKind::from_name("roots").unwrap();
    items::create_ground(
        &mut res,
        (2, 2),
        ItemSpec::raw(roots, None),
        2,
        0,
        items::FoodSource::Harvest,
    )
    .unwrap();
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(world.get::<&Job>(person).is_err());
    assert_eq!(*world.get::<&AgentState>(person).unwrap(), AgentState::Idle);
}

#[test]
fn missing_food_does_not_override_thirst_sleep_or_active_survival() {
    let (mut world, mut res, person) = scene();
    world.insert_one(person, Thirst::new(90.0, 0.0)).unwrap();
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(world.get::<&Job>(person).is_err());
    world.remove_one::<Thirst>(person).unwrap();
    world.insert_one(person, Sleep::new(100.0, 0.0)).unwrap();
    res.sleep_tiles.insert((2, 2), 1);
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(world.get::<&Job>(person).is_err());
    world.remove_one::<Sleep>(person).unwrap();
    world
        .insert_one(
            person,
            AgentState::Seeking {
                target: TargetKind::Water,
            },
        )
        .unwrap();
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(world.get::<&Job>(person).is_err());
}

#[test]
fn unreachable_or_expired_food_does_not_hold_a_hungry_worker_idle() {
    let (mut world, mut res, person) = scene();
    let roots = ItemKind::from_name("roots").unwrap();
    items::create_ground(
        &mut res,
        (6, 2),
        ItemSpec::raw(roots, None),
        2,
        0,
        items::FoodSource::Harvest,
    )
    .unwrap();
    for y in 0..8 {
        let i = res.tile_grid.idx(4, y);
        res.tile_grid.terrain_type[i] = sim_core::TerrainType::River;
    }
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(world.get::<&Job>(person).is_ok());
    let mut index = FoodChoiceIndex::default();
    index.refresh(&res);
    assert!(!super::assign::needy(&world, &res, person, &index));
    res.current_tick = res.ticks_per_day * 100;
    index.refresh(&res);
    assert!(!index.available_to(&res, (6, 2), 1, None));
}
