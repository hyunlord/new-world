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

fn reserve_gather_scene() -> (World, SimResources, Entity) {
    let (world, mut res, person) = scene();
    res.steward.enabled = true;
    world.get::<&mut Hunger>(person).unwrap().value = 0.0;
    world.get::<&mut WorkPriorities>(person).unwrap().levels[WorkType::Split.code() as usize] = 1;
    let mut settlement = sim_core::components::Settlement::new_with_id(1, 0);
    settlement.formation_tile = (4, 4);
    let rule = res.rules.society.firewood.unwrap();
    use crate::runtime::settlement::firewood::{burn_hours, night_tick_at, winter_burn_per_hour};
    let night = night_tick_at(&res, &rule, res.current_tick);
    let tonight = u32::from(winter_burn_per_hour(&res, &rule, night)) * burn_hours(&rule);
    settlement.store(sim_core::components::ResourceKind::Wood, tonight);
    res.settlements.insert(1, settlement);
    res.wood_tiles.insert((2, 3), 20);
    assert!(super::assign::gather_shortfall(&world, &res) > 0);
    (world, res, person)
}

#[test]
fn urgent_food_defers_tied_reserve_gather_to_real_harvest() {
    let (mut world, mut res, person) = reserve_gather_scene();
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(person).unwrap().kind,
        JobKind::Harvest { plant: (3, 2) }
    ));
}

#[test]
fn urgent_food_gather_deferral_preserves_heat_priorities_and_legacy() {
    for case in [
        "cold_hearth",
        "steward_off",
        "haul_disabled",
        "split_priority",
        "no_harvest",
        "food_ready",
        "other_cold_hearth",
    ] {
        let (mut world, mut res, person) = reserve_gather_scene();
        match case {
            "cold_hearth" => {
                *res.settlements
                    .get_mut(&1)
                    .unwrap()
                    .stockpile
                    .get_mut(&sim_core::components::ResourceKind::Wood)
                    .unwrap() -= 1;
            }
            "steward_off" => res.steward.enabled = false,
            "haul_disabled" => {
                world.get::<&mut WorkPriorities>(person).unwrap().levels
                    [WorkType::Haul.code() as usize] = 0
            }
            "split_priority" => {
                world.get::<&mut WorkPriorities>(person).unwrap().levels
                    [WorkType::Haul.code() as usize] = 2
            }
            "no_harvest" => res.colony.designations.clear(),
            "food_ready" => {
                items::create_ground(
                    &mut res,
                    (2, 2),
                    ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None),
                    3,
                    0,
                    items::FoodSource::Harvest,
                )
                .unwrap();
            }
            "other_cold_hearth" => {
                let mut settlement = sim_core::components::Settlement::new_with_id(2, 0);
                settlement.formation_tile = (7, 7);
                res.settlements.insert(2, settlement);
            }
            _ => unreachable!(),
        }
        JobAssignSystem::new().tick(&mut world, &mut res);
        assert!(
            matches!(
                world.get::<&Job>(person).unwrap().kind,
                JobKind::Gather { .. }
            ),
            "{case}"
        );
    }
}

#[test]
fn urgent_food_harvest_selection_skips_nearer_inedible_crop() {
    let (mut world, mut res, person) = reserve_gather_scene();
    res.current_tick = 6 * res.ticks_per_day;
    res.rules.society.firewood.as_mut().unwrap().base_days = 10.0;
    assert!(super::assign::gather_shortfall(&world, &res) > 0);
    let species = sim_core::colony::content::installed_or_bundled()
        .plant_by_name("wild_flax")
        .unwrap();
    res.food.plants.insert(
        (2, 2),
        PlantState {
            species,
            units_left: 4,
            next_regrow_day: u32::MAX,
        },
    );
    res.colony.designations.insert((2, 2), Designation::Harvest);
    res.colony.touch(false);
    assert!(super::harvest::target_exists(&res, (2, 2)));
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(person).unwrap().kind,
        JobKind::Harvest { plant: (3, 2) }
    ));
}

#[test]
fn urgent_food_gather_keeps_actual_cold_snap_heating_reserve() {
    let (mut world, mut res, person) = reserve_gather_scene();
    res.rules.colony.as_mut().unwrap().story = Some(sim_core::rules::StoryRule::M5);
    res.colony
        .story
        .active
        .push(sim_core::colony::story::Pressure {
            kind: sim_core::colony::story::PressureKind::ColdSnap,
            start_day: 0,
            end_day: 3,
            began_entry: None,
            deaths_at_start: 0,
            intensity: 0,
            subject: None,
            cost: Default::default(),
        });
    assert!(crate::runtime::settlement::fire::effective_burn_per_hour(&res, 2) > 2);
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(person).unwrap().kind,
        JobKind::Gather { .. }
    ));
}

#[test]
fn urgent_food_gather_keeps_possible_positive_winter_night_cost() {
    let (world, mut res, _) = reserve_gather_scene();
    res.current_tick = 19 * res.ticks_per_day;
    let mut rule = res.rules.society.firewood.unwrap();
    rule.winter_severity_permille = Some([1000, 0, 0]);
    rule.winter_night_burn_permille = Some([0, 0, 1000]);
    rule.winter_night_burn_step = 1;
    res.rules.society.firewood = Some(rule);
    use crate::runtime::settlement::firewood::{burn_hours, night_tick_at, winter_burn_per_hour};
    let night = night_tick_at(&res, &rule, res.current_tick);
    let nominal = u32::from(winter_burn_per_hour(&res, &rule, night)) * burn_hours(&rule);
    res.settlements
        .get_mut(&1)
        .unwrap()
        .stockpile
        .insert(sim_core::components::ResourceKind::Wood, nominal);
    assert!(!super::assign::reserve_gather_food_urgent(&world, &res));
}
