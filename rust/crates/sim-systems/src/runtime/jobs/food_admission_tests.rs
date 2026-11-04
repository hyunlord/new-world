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
        "immediate_ready",
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
            "food_ready" | "immediate_ready" => {
                items::create_ground(
                    &mut res,
                    (2, 2),
                    ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None),
                    if case == "food_ready" { 3 } else { 5 },
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
        if matches!(case, "food_ready" | "immediate_ready") {
            assert!(!super::assign::reserve_gather_food_urgent(&world, &res));
            let policy = sim_core::colony::steward::rules()
                .unwrap()
                .rules
                .iter()
                .find_map(|r| {
                    if let sim_core::colony::steward::Action::FoodPlan(policy) = &r.action {
                        Some(policy.clone())
                    } else {
                        None
                    }
                })
                .unwrap();
            let facts = crate::runtime::steward::food::summary(&world, &res, &policy);
            assert_eq!(facts.daily_milli, 2400);
            assert_eq!(policy.reserve_days, 2);
            assert_eq!(
                (facts.daily_milli * u64::from(policy.reserve_days)).div_ceil(1000),
                5
            );
        }
        JobAssignSystem::new().tick(&mut world, &mut res);
        let kind = world.get::<&Job>(person).unwrap().kind;
        if case == "food_ready" {
            assert!(
                matches!(kind, JobKind::Harvest { .. }),
                "3 covers today but is below immediate target5"
            );
        } else {
            assert!(matches!(kind, JobKind::Gather { .. }), "{case}: {kind:?}");
        }
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

fn fuel_build_scene(supplied: bool) -> (World, SimResources, Entity) {
    use sim_core::colony::{
        Bill, BillMode, BlueprintKind, Command, ItemStack, RecipeKind, Workshop,
    };
    let (mut world, mut res, person) = scene();
    res.steward.enabled = true;
    res.colony.designations.clear();
    world.get::<&mut Hunger>(person).unwrap().value = 0.0;
    *world.get::<&mut WorkPriorities>(person).unwrap() = WorkPriorities::defaults(false);
    let mut settlement = sim_core::components::Settlement::new_with_id(1, 0);
    settlement.formation_tile = (4, 4);
    res.settlements.insert(1, settlement);
    let recipe = RecipeKind::from_name("split_firewood").expect("fuel recipe");
    let log = ItemKind::role("tree_output");
    res.colony
        .ground_items
        .insert((2, 3), ItemStack::of(ItemSpec::raw(log, None), 20));
    res.colony.workshops.insert(
        1,
        Workshop {
            id: 1,
            kind: recipe.recipe().unwrap().workshop,
            top_left: (3, 3),
            size: (1, 1),
            bills: vec![Bill {
                id: 1,
                recipe,
                mode: BillMode::Maintain(100),
                done: 0,
                suspended: false,
                auto: true,
            }],
            shelf: if supplied {
                vec![ItemStack::of(ItemSpec::raw(log, None), 1)]
            } else {
                vec![]
            },
            active: Some(1),
        },
    );
    assert!(
        crate::commands::apply_command(
            &mut world,
            &mut res,
            Command::PlaceBlueprint {
                kind: BlueprintKind::from_name("log_wall").unwrap(),
                x: 5,
                y: 5,
            }
        )
        .accepted
    );
    assert!(super::bills::runnable(
        &world,
        &res,
        &res.colony.workshops[&1],
        0
    ));
    res.colony.touch(false);
    (world, res, person)
}

#[test]
fn urgent_fuel_bill_wins_equal_priority_construction() {
    for supplied in [false, true] {
        let (mut world, mut res, person) = fuel_build_scene(supplied);
        JobAssignSystem::new().tick(&mut world, &mut res);
        let job = world.get::<&Job>(person).unwrap();
        assert!(
            matches!(
                job.kind,
                JobKind::Supply { workshop: 1, .. } | JobKind::Craft { workshop: 1, .. }
            ),
            "supplied={supplied}: {job:?}"
        );
        assert_eq!(res.colony.reservations.len(), job.keys().len());
    }
}

#[test]
fn urgent_fuel_bill_respects_stored_cover_off_disabled_and_higher_build() {
    for control in 0..4 {
        let (mut world, mut res, person) = fuel_build_scene(false);
        match control {
            0 => {
                res.settlements
                    .get_mut(&1)
                    .unwrap()
                    .stockpile
                    .insert(sim_core::components::ResourceKind::Wood, 1000);
            }
            1 => res.steward.enabled = false,
            2 => {
                world.get::<&mut WorkPriorities>(person).unwrap().levels
                    [WorkType::Split.code() as usize] = 0
            }
            _ => {
                world.get::<&mut WorkPriorities>(person).unwrap().levels
                    [WorkType::Build.code() as usize] = 1
            }
        }
        JobAssignSystem::new().tick(&mut world, &mut res);
        assert!(
            matches!(
                world.get::<&Job>(person).unwrap().kind,
                JobKind::Deliver { site: (5, 5), .. }
            ),
            "control={control}"
        );
    }
}

#[test]
fn urgent_fuel_bill_keeps_ordinary_craft_first() {
    use sim_core::colony::{Bill, BillMode, ItemStack, RecipeKind, Workshop};
    let (mut world, mut res, person) = fuel_build_scene(true);
    let recipe = RecipeKind::from_name("knap_blade").unwrap();
    res.colony.workshops.insert(
        2,
        Workshop {
            id: 2,
            kind: recipe.recipe().unwrap().workshop,
            top_left: (2, 4),
            size: (1, 1),
            bills: vec![Bill {
                id: 2,
                recipe,
                mode: BillMode::Make(1),
                done: 0,
                suspended: false,
                auto: false,
            }],
            shelf: vec![ItemStack::of(
                ItemSpec::raw(recipe.recipe().unwrap().inputs[0].0, None),
                1,
            )],
            active: Some(2),
        },
    );
    res.colony.designations.insert((3, 2), Designation::Harvest);
    res.colony.touch(false);
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(person).unwrap().kind,
        JobKind::Craft { workshop: 2, .. }
    ));
}

#[test]
fn urgent_fuel_bill_ground_cover_suppresses_production() {
    use sim_core::colony::ItemStack;
    let (mut world, mut res, person) = fuel_build_scene(false);
    let fuel = res.colony.workshops[&1].bills[0]
        .recipe
        .recipe()
        .unwrap()
        .output
        .0;
    res.colony
        .ground_items
        .insert((4, 4), ItemStack::of(ItemSpec::raw(fuel, None), 50));
    res.colony
        .ground_items
        .insert((4, 3), ItemStack::of(ItemSpec::raw(fuel, None), 50));
    // Disable hauling to observe bill suppression independently of the earlier fuel recovery round.
    world.get::<&mut WorkPriorities>(person).unwrap().levels[WorkType::Haul.code() as usize] = 0;
    res.colony.touch(false);
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(person).unwrap().kind,
        JobKind::Deliver { site: (5, 5), .. }
    ));
}

#[test]
fn urgent_fuel_bill_does_not_promote_gather_or_cancel_survival() {
    let (mut world, mut res, person) = fuel_build_scene(false);
    assert!(
        crate::commands::apply_command(
            &mut world,
            &mut res,
            sim_core::colony::Command::SetBillSuspended {
                workshop: 1,
                bill: 1,
                suspended: true
            }
        )
        .accepted
    );
    res.wood_tiles.insert((2, 4), 20);
    assert!(super::assign::gather_shortfall(&world, &res) > 0);
    // The same natural resource is actionable when construction is disabled.
    let (mut witness_world, mut witness_res, witness) = fuel_build_scene(false);
    assert!(
        crate::commands::apply_command(
            &mut witness_world,
            &mut witness_res,
            sim_core::colony::Command::SetBillSuspended {
                workshop: 1,
                bill: 1,
                suspended: true
            }
        )
        .accepted
    );
    witness_res.wood_tiles.insert((2, 4), 20);
    witness_world
        .get::<&mut WorkPriorities>(witness)
        .unwrap()
        .levels[WorkType::Build.code() as usize] = 0;
    JobAssignSystem::new().tick(&mut witness_world, &mut witness_res);
    assert!(matches!(
        witness_world.get::<&Job>(witness).unwrap().kind,
        JobKind::Gather { .. }
    ));
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(person).unwrap().kind,
        JobKind::Deliver { .. }
    ));
    let prior = *world.get::<&Job>(person).unwrap();
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert_eq!(*world.get::<&Job>(person).unwrap(), prior);
    let (mut world, mut res, person) = fuel_build_scene(false);
    *world.get::<&mut AgentState>(person).unwrap() = AgentState::Seeking {
        target: TargetKind::Food,
    };
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(world.get::<&Job>(person).is_err());
}

fn partial_night_fuel_scene(tile: (u32, u32), count: u16) -> (World, SimResources, Entity) {
    use sim_core::colony::ItemStack;
    let (world, mut res, person) = fuel_build_scene(true);
    let firewood = res.rules.society.firewood.as_mut().unwrap();
    firewood.burn_per_hour = [2; 4];
    firewood.winter_severity_permille = None;
    firewood.winter_night_burn_permille = None;
    assert_eq!(
        crate::runtime::settlement::firewood::burn_hours(firewood),
        13
    );
    assert_eq!(
        crate::runtime::settlement::fire::effective_burn_per_hour(&res, 2),
        2
    );
    res.settlements
        .get_mut(&1)
        .unwrap()
        .stockpile
        .insert(sim_core::components::ResourceKind::Wood, 24);
    let fuel = res.colony.workshops[&1].bills[0]
        .recipe
        .recipe()
        .unwrap()
        .output
        .0;
    res.colony
        .ground_items
        .insert(tile, ItemStack::of(ItemSpec::raw(fuel, None), count));
    world.get::<&mut WorkPriorities>(person).unwrap().levels[WorkType::Haul.code() as usize] = 0;
    assert!(super::bills::runnable(
        &world,
        &res,
        &res.colony.workshops[&1],
        0
    ));
    res.colony.touch(false);
    (world, res, person)
}

#[test]
fn urgent_fuel_partial_night_cover_below_seasonal_target_defers_to_build() {
    use sim_core::colony::{Zone, ZoneKind};
    for stockpile in [false, true] {
        let tile = if stockpile { (0, 0) } else { (4, 3) };
        let (mut world, mut res, person) = partial_night_fuel_scene(tile, 10);
        if stockpile {
            res.colony.zones.insert(
                1,
                Zone {
                    id: 1,
                    kind: ZoneKind::Stockpile,
                    tiles: std::collections::BTreeSet::from([tile]),
                },
            );
            res.colony.zone_of_tile.insert(tile, 1);
        }
        JobAssignSystem::new().tick(&mut world, &mut res);
        assert!(
            matches!(
                world.get::<&Job>(person).unwrap().kind,
                JobKind::Deliver { site: (5, 5), .. }
            ),
            "stockpile={stockpile}: {:?}",
            world.get::<&Job>(person).unwrap()
        );
    }
}

#[test]
fn urgent_fuel_outside_pile_does_not_cover_night() {
    let (mut world, mut res, person) = partial_night_fuel_scene((0, 0), 10);
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(person).unwrap().kind,
        JobKind::Craft { workshop: 1, .. }
    ));
}

#[test]
fn urgent_fuel_shared_stack_covers_each_hearth_only_once() {
    for (count, urgent) in [(2, true), (4, false)] {
        let (mut world, mut res, person) = partial_night_fuel_scene((4, 3), count);
        let mut second = sim_core::components::Settlement::new_with_id(2, 0);
        second.formation_tile = (5, 3);
        second
            .stockpile
            .insert(sim_core::components::ResourceKind::Wood, 24);
        res.settlements.insert(2, second);
        JobAssignSystem::new().tick(&mut world, &mut res);
        let kind = world.get::<&Job>(person).unwrap().kind;
        assert_eq!(
            matches!(kind, JobKind::Craft { workshop: 1, .. }),
            urgent,
            "count={count}: {kind:?}"
        );
        if !urgent {
            assert!(matches!(kind, JobKind::Deliver { site: (5, 5), .. }));
        }
    }
}

#[test]
fn urgent_fuel_rechecks_after_ordinary_craft_reserves_ground_cover() {
    use sim_core::colony::{Bill, BillMode, ItemStack, RecipeKind, Workshop};
    let (mut world, mut res, crafter) = partial_night_fuel_scene((4, 3), 10);
    let mut craft_only = WorkPriorities {
        levels: [0; WorkType::ALL.len()],
    };
    craft_only.levels[WorkType::Craft.code() as usize] = 2;
    *world.get::<&mut WorkPriorities>(crafter).unwrap() = craft_only;
    let mut priorities = WorkPriorities::defaults(false);
    priorities.levels[WorkType::Craft.code() as usize] = 0;
    priorities.levels[WorkType::Haul.code() as usize] = 0;
    let person = world.spawn((
        Agent { id: 2 },
        Position::new(2, 2),
        AgentState::Idle,
        Hunger::new(0.0, 0.0),
        priorities,
    ));
    let recipe = RecipeKind::from_name("fire_pot").unwrap();
    res.colony.workshops.insert(
        2,
        Workshop {
            id: 2,
            kind: recipe.recipe().unwrap().workshop,
            top_left: (2, 4),
            size: (1, 1),
            bills: vec![Bill {
                id: 2,
                recipe,
                mode: BillMode::Make(1),
                done: 0,
                suspended: false,
                auto: false,
            }],
            shelf: vec![ItemStack::of(
                ItemSpec::raw(ItemKind::from_name("raw_pot").unwrap(), None),
                1,
            )],
            active: Some(2),
        },
    );
    res.colony.touch(false);
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(crafter).unwrap().kind,
        JobKind::Supply {
            workshop: 2,
            from: (4, 3),
            ..
        }
    ));
    assert!(res
        .colony
        .reservations
        .contains_key(&sim_core::colony::ReserveKey::Stack((4, 3))));
    assert!(matches!(
        world.get::<&Job>(person).unwrap().kind,
        JobKind::Craft { workshop: 1, .. }
    ));
}

#[test]
fn winter_possible_cold_night_admits_fuel_before_build() {
    let (mut world, mut res, person) = partial_night_fuel_scene((4, 3), 10);
    res.rules.clock_offset_ticks = 0;
    res.current_tick = u64::from(res.rules.calendar.days_per_season) * 3 * res.ticks_per_day + 720;
    assert_eq!(
        crate::runtime::resource_regen::season_at(&res).unwrap().0,
        3
    );
    assert!(res.colony.story.active.is_empty());
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(person).unwrap().kind,
        JobKind::Craft { workshop: 1, .. }
    ));
}

fn food_build_scene() -> (World, SimResources, Entity) {
    let (world, mut res, person) = fuel_build_scene(false);
    res.settlements
        .get_mut(&1)
        .unwrap()
        .stockpile
        .insert(sim_core::components::ResourceKind::Wood, 1000);
    res.colony.designations.insert((3, 2), Designation::Harvest);
    res.colony.touch(false);
    (world, res, person)
}

#[test]
fn immediate_food_harvest_precedes_equal_priority_build() {
    let (mut world, mut res, person) = food_build_scene();
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(person).unwrap().kind,
        JobKind::Harvest { plant: (3, 2) }
    ));
}

#[test]
fn immediate_food_admission_preserves_player_priority_scope_and_target() {
    for case in [
        "higher_build",
        "disabled_haul",
        "off",
        "housing",
        "at_target",
        "inedible",
    ] {
        let (mut world, mut res, person) = food_build_scene();
        match case {
            "higher_build" => {
                world.get::<&mut WorkPriorities>(person).unwrap().levels
                    [WorkType::Build.code() as usize] = 1
            }
            "disabled_haul" => {
                world.get::<&mut WorkPriorities>(person).unwrap().levels
                    [WorkType::Haul.code() as usize] = 0
            }
            "off" => res.steward.enabled = false,
            "housing" => res.steward.scope = sim_core::colony::steward::StewardScope::Housing,
            "at_target" => {
                items::create_ground(
                    &mut res,
                    (1, 1),
                    ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None),
                    5,
                    0,
                    items::FoodSource::Harvest,
                )
                .unwrap();
            }
            "inedible" => {
                res.food.plants.get_mut(&(3, 2)).unwrap().species =
                    sim_core::colony::content::installed_or_bundled()
                        .plant_by_name("wild_flax")
                        .unwrap()
            }
            _ => unreachable!(),
        }
        JobAssignSystem::new().tick(&mut world, &mut res);
        assert!(
            matches!(
                world.get::<&Job>(person).unwrap().kind,
                JobKind::Deliver { site: (5, 5), .. }
            ),
            "{case}"
        );
    }
}

#[test]
fn immediate_food_admission_reserves_once_and_yields_to_urgent_fuel() {
    let (mut world, mut res, first) = food_build_scene();
    let second = world.spawn((
        Agent { id: 2 },
        Position::new(2, 2),
        AgentState::Idle,
        Hunger::new(0.0, 0.0),
        WorkPriorities::defaults(false),
    ));
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(first).unwrap().kind,
        JobKind::Harvest { .. }
    ));
    assert!(matches!(
        world.get::<&Job>(second).unwrap().kind,
        JobKind::Deliver { .. }
    ));
    let (mut world, mut res, person) = food_build_scene();
    res.settlements.get_mut(&1).unwrap().stockpile.clear();
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(person).unwrap().kind,
        JobKind::Supply { workshop: 1, .. }
    ));
}

#[test]
fn winter_forecast_keeps_story_off_and_hourly_rounding() {
    for (story, stored, expect_fuel) in [(false, 40, false), (true, 40, true), (true, 52, false)] {
        let (mut world, mut res, person) = partial_night_fuel_scene((4, 3), 0);
        res.rules.clock_offset_ticks = 0;
        res.current_tick =
            u64::from(res.rules.calendar.days_per_season) * 3 * res.ticks_per_day + 720;
        res.settlements
            .get_mut(&1)
            .unwrap()
            .stockpile
            .insert(sim_core::components::ResourceKind::Wood, stored);
        if story {
            res.rules
                .colony
                .as_mut()
                .unwrap()
                .story
                .as_mut()
                .unwrap()
                .snap_burn_permille = 1501;
        } else {
            res.rules.colony.as_mut().unwrap().story = None;
        }
        JobAssignSystem::new().tick(&mut world, &mut res);
        let kind = world.get::<&Job>(person).unwrap().kind;
        assert_eq!(
            matches!(kind, JobKind::Craft { workshop: 1, .. }),
            expect_fuel,
            "story={story},stored={stored}: {kind:?}"
        );
    }
}

#[test]
fn winter_forecast_uses_enabled_scheduled_cold_data_not_identity() {
    use sim_core::colony::content::{PressureDraw, PressureMechanic, PressureSchedule};
    let catalog = sim_core::colony::content::installed_or_bundled();
    let mut def = catalog
        .pressures()
        .find(|d| d.mechanic == PressureMechanic::Cold)
        .unwrap()
        .clone();
    let alternate = sim_core::colony::story::PressureKind::Beasts;
    def.identity.code = alternate.code();
    def.enabled_rule = Some("beasts".into());
    def.cold.as_mut().unwrap().burn.rule = None;
    def.cold.as_mut().unwrap().burn.value = 2301;
    let mut rule = sim_core::rules::StoryRule::M5;
    let mut schedule = PressureSchedule::default();
    let factor = |rule: &_, schedule: &_| {
        super::assign::possible_winter_cold_factor(rule, schedule, std::iter::once(&def))
    };
    assert_eq!(factor(&rule, &schedule), 1000);
    schedule.seasons.insert(
        3,
        vec![
            PressureDraw {
                upper_exclusive: 500,
                kind: None,
            },
            PressureDraw {
                upper_exclusive: 500,
                kind: Some(alternate),
            },
            PressureDraw {
                upper_exclusive: 1000,
                kind: None,
            },
        ],
    );
    assert_eq!(
        factor(&rule, &schedule),
        1000,
        "zero-width draw is impossible"
    );
    schedule.minor = Some(alternate);
    schedule.minor_seasons = vec![3];
    assert_eq!(factor(&rule, &schedule), 2301);
    rule.beasts = false;
    assert_eq!(factor(&rule, &schedule), 1000);
    rule.beasts = true;
    schedule.minor = None;
    schedule.due_order = vec![alternate];
    assert_eq!(
        factor(&rule, &schedule),
        2301,
        "authored due eligibility also counts"
    );
}

#[test]
fn immediate_food_without_fuel_rule_or_hearth_keeps_safe_admission() {
    for no_rule in [false, true] {
        let (mut world, mut res, person) = food_build_scene();
        if no_rule {
            res.rules.society.firewood = None;
        } else {
            res.settlements.clear();
        }
        JobAssignSystem::new().tick(&mut world, &mut res);
        assert!(matches!(
            world.get::<&Job>(person).unwrap().kind,
            JobKind::Harvest { .. }
        ));
    }
}

#[test]
fn winter_forecast_never_reduces_active_cold_outside_winter() {
    let (mut world, mut res, person) = partial_night_fuel_scene((4, 3), 0);
    res.rules
        .colony
        .as_mut()
        .unwrap()
        .story
        .as_mut()
        .unwrap()
        .snap_burn_permille = 2500;
    res.settlements
        .get_mut(&1)
        .unwrap()
        .stockpile
        .insert(sim_core::components::ResourceKind::Wood, 52);
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
    assert_ne!(
        crate::runtime::resource_regen::season_at(&res).unwrap().0,
        3
    );
    assert_eq!(
        crate::runtime::settlement::fire::effective_burn_per_hour(&res, 2),
        5
    );
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(person).unwrap().kind,
        JobKind::Craft { workshop: 1, .. }
    ));
}

#[test]
fn winter_forecast_first_morning_uses_winter_rate_without_changing_booked_night() {
    let (mut world, mut res, person) = partial_night_fuel_scene((4, 3), 0);
    res.rules.clock_offset_ticks = 0;
    res.current_tick = u64::from(res.rules.calendar.days_per_season) * 3 * res.ticks_per_day + 60;
    let rule = res.rules.society.firewood.as_mut().unwrap();
    rule.night_season_from_evening = true;
    rule.winter_severity_permille = Some([1000, 0, 0]);
    rule.winter_severity_burn_per_hour = [3; 3];
    res.settlements
        .get_mut(&1)
        .unwrap()
        .stockpile
        .insert(sim_core::components::ResourceKind::Wood, 52);
    let rule = res.rules.society.firewood.unwrap();
    let booked = crate::runtime::settlement::firewood::night_tick_at(&res, &rule, res.current_tick);
    assert_eq!(
        crate::runtime::settlement::firewood::winter_burn_per_hour(&res, &rule, booked),
        2
    );
    assert_eq!(
        crate::runtime::settlement::firewood::winter_burn_per_hour(&res, &rule, res.current_tick),
        3
    );
    assert!(
        super::assign::reserve_gather_food_urgent(&world, &res),
        "current-only26night remains covered"
    );
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(
        matches!(
            world.get::<&Job>(person).unwrap().kind,
            JobKind::Craft { workshop: 1, .. }
        ),
        "possible winter Cold night requires78, not prior-season52"
    );
}

#[test]
fn urgent_fuel_prefetch_overlaps_committed_craft() {
    let (mut world, mut res, crafter) = fuel_build_scene(true);
    JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(crafter).unwrap().kind,
        JobKind::Craft {
            workshop: 1,
            bill: 1,
            ..
        }
    ));
    let worker = world.spawn((
        Agent { id: 2 },
        Position::new(2, 2),
        AgentState::Idle,
        Hunger::new(0.0, 0.0),
        WorkPriorities::defaults(false),
    ));
    JobAssignSystem::new().tick(&mut world, &mut res);
    let job = world.get::<&Job>(worker).unwrap();
    assert!(
        matches!(job.kind, JobKind::Supply { workshop: 1, .. }),
        "{job:?}"
    );
    assert_eq!(job.amount, 4, "five future crafts minus one shelved log");
    assert!(matches!(
        world.get::<&Job>(crafter).unwrap().kind,
        JobKind::Craft {
            workshop: 1,
            bill: 1,
            ..
        }
    ));
}

#[test]
fn urgent_fuel_prefetch_bounds_remaining_work_and_incoming_inputs() {
    use sim_core::colony::BillMode;
    for (mode, expected) in [
        (BillMode::Make(3), 1),
        (BillMode::Maintain(12), 1),
        (BillMode::Make(2), 0),
        (BillMode::Maintain(8), 0),
    ] {
        let (mut world, mut res, crafter) = fuel_build_scene(true);
        res.colony.workshops.get_mut(&1).unwrap().bills[0].mode = mode;
        JobAssignSystem::new().tick(&mut world, &mut res);
        assert!(matches!(
            world.get::<&Job>(crafter).unwrap().kind,
            JobKind::Craft { .. }
        ));
        let worker = world.spawn((
            Agent { id: 2 },
            Position::new(2, 2),
            AgentState::Idle,
            Hunger::new(0.0, 0.0),
            WorkPriorities::defaults(false),
        ));
        JobAssignSystem::new().tick(&mut world, &mut res);
        let job = *world.get::<&Job>(worker).unwrap();
        if expected == 0 {
            assert!(
                matches!(job.kind, JobKind::Deliver { .. }),
                "{mode:?}: {job:?}"
            );
        } else {
            assert!(
                matches!(job.kind, JobKind::Supply { workshop: 1, .. }),
                "{mode:?}: {job:?}"
            );
            assert_eq!(job.amount, expected);
            // A second unreserved source makes the incoming-count control meaningful.
            res.colony.ground_items.insert(
                (1, 3),
                sim_core::colony::ItemStack::of(
                    ItemSpec::raw(ItemKind::role("tree_output"), None),
                    20,
                ),
            );
            res.colony.touch(false);
            let third = world.spawn((
                Agent { id: 3 },
                Position::new(2, 2),
                AgentState::Idle,
                Hunger::new(0.0, 0.0),
                WorkPriorities::defaults(false),
            ));
            JobAssignSystem::new().tick(&mut world, &mut res);
            assert!(
                !matches!(
                    world.get::<&Job>(third).unwrap().kind,
                    JobKind::Supply { workshop: 1, .. }
                ),
                "incoming already covers remaining inputs"
            );
        }
    }
}

#[test]
fn urgent_fuel_prefetch_requires_urgent_enabled_bill() {
    use sim_core::colony::Command;
    for control in 0..4 {
        let (mut world, mut res, crafter) = fuel_build_scene(true);
        JobAssignSystem::new().tick(&mut world, &mut res);
        assert!(matches!(
            world.get::<&Job>(crafter).unwrap().kind,
            JobKind::Craft { .. }
        ));
        match control {
            0 => {
                res.settlements
                    .get_mut(&1)
                    .unwrap()
                    .stockpile
                    .insert(sim_core::components::ResourceKind::Wood, 1000);
            }
            1 => res.steward.enabled = false,
            2 => res.rules.colony.as_mut().unwrap().space = None,
            _ => {
                assert!(
                    crate::commands::apply_command(
                        &mut world,
                        &mut res,
                        Command::SetBillSuspended {
                            workshop: 1,
                            bill: 1,
                            suspended: true
                        }
                    )
                    .accepted
                );
            }
        }
        let worker = world.spawn((
            Agent { id: 2 },
            Position::new(2, 2),
            AgentState::Idle,
            Hunger::new(0.0, 0.0),
            WorkPriorities::defaults(false),
        ));
        JobAssignSystem::new().tick(&mut world, &mut res);
        assert!(
            matches!(
                world.get::<&Job>(worker).unwrap().kind,
                JobKind::Deliver { .. }
            ),
            "control {control}"
        );
    }
}

#[test]
fn urgent_fuel_prefetch_overlaps_started_craft_after_consumption() {
    let (mut world, mut res, crafter) = fuel_build_scene(true);
    let log = ItemKind::role("tree_output");
    res.colony.workshops.get_mut(&1).unwrap().shelf[0].count = 2;
    res.colony.item_ledger.created.insert(log, 22);
    JobAssignSystem::new().tick(&mut world, &mut res);
    let tile = res.colony.workshops[&1].work_tile();
    *world.get::<&mut Position>(crafter).unwrap() = Position::new(tile.0, tile.1);
    super::JobExecuteSystem::new().tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(crafter).unwrap().stage,
        sim_core::colony::JobStage::Working { .. }
    ));
    assert_eq!(res.colony.workshops[&1].shelved(log), 1);
    let worker = world.spawn((
        Agent { id: 2 },
        Position::new(2, 2),
        AgentState::Idle,
        Hunger::new(0.0, 0.0),
        WorkPriorities::defaults(false),
    ));
    JobAssignSystem::new().tick(&mut world, &mut res);
    let job = world.get::<&Job>(worker).unwrap();
    assert!(
        matches!(job.kind, JobKind::Supply { workshop: 1, .. }),
        "{job:?}"
    );
    assert_eq!(job.amount, 4);
}
