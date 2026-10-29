//! Storage planning must make room for normal haul jobs without relocating food.
use hecs::World;
use sim_core::colony::steward::{Action, FoodPlan, Target};
use sim_core::colony::{Command, ItemKind, ItemSpec};
use sim_core::components::{Agent, Position, Settlement};
use sim_core::rules::WorldRules;
use sim_engine::SimResources;

fn fixture(full: bool) -> (World, SimResources, FoodPlan) {
    fixture_with_food(full, "roots")
}

fn fixture_with_food(full: bool, food_name: &str) -> (World, SimResources, FoodPlan) {
    let mut world = World::new();
    let mut res = SimResources::new(24, 24, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(1);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.current_tick = 14 * res.ticks_per_day;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (12, 12);
    res.settlements.insert(1, settlement);
    for id in 0..10 {
        world.spawn((Agent { id }, Position::new(12, 12)));
    }
    assert!(
        crate::commands::apply_command(
            &mut world,
            &mut res,
            Command::AddStockpile {
                x0: 8,
                y0: 14,
                x1: 11,
                y1: 17
            }
        )
        .accepted
    );
    if full {
        let wood = ItemKind::from_name("firewood").expect("firewood");
        let count = crate::runtime::jobs::items::stack_max(&res, wood);
        let tiles: Vec<_> = res.colony.zone_of_tile.keys().copied().collect();
        for tile in tiles {
            crate::runtime::food::items::create_ground(
                &mut res,
                tile,
                ItemSpec::raw(wood, None),
                count,
                0,
                crate::runtime::food::items::FoodSource::Debug,
            )
            .expect("occupied stock cell");
        }
    }
    let food = ItemKind::from_name(food_name).expect("fixture food");
    crate::runtime::food::items::create_ground(
        &mut res,
        (6, 12),
        ItemSpec::raw(food, None),
        20,
        14,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("actual loose food");
    let policy = sim_core::colony::steward::rules()
        .expect("rules")
        .rules
        .iter()
        .find_map(|rule| match &rule.action {
            Action::FoodPlan(policy) => Some(policy.clone()),
            _ => None,
        })
        .expect("food plan");
    (world, res, policy)
}

fn stock_commands(world: &World, res: &SimResources, policy: &FoodPlan) -> Vec<Command> {
    super::plan(world, res, policy)
        .into_iter()
        .filter(|command| matches!(command, Command::AddStockpile { .. }))
        .collect()
}

#[test]
fn full_nonfood_store_plans_bounded_empty_space_for_actual_food_hauling() {
    let (world, res, policy) = fixture(true);
    let commands = stock_commands(&world, &res, &policy);
    assert!(
        !commands.is_empty(),
        "full nonfood stockpile must not block all food storage"
    );
    let mut tiles = std::collections::BTreeSet::new();
    for command in commands {
        let Command::AddStockpile { x0, y0, x1, y1 } = command else {
            unreachable!()
        };
        for y in y0..=y1 {
            for x in x0..=x1 {
                assert!(tiles.insert((x, y)), "do not allocate the same cell twice");
                assert!(
                    !res.colony.ground_items.contains_key(&(x, y)),
                    "food must arrive by hauling"
                );
                assert!(!res.colony.zone_of_tile.contains_key(&(x, y)));
                assert!(crate::runtime::jobs::items::can_hold_items(&res, (x, y)));
                assert!(crate::commands::camp_reaches(&res, (x, y)));
                assert!(!res.steward.vetoed(&Target::Tile(x, y), res.current_tick));
                assert!(x.abs_diff(12).max(y.abs_diff(12)) <= policy.forage_radius);
            }
        }
    }
    assert!(
        tiles.len() <= policy.harvest_batch as usize,
        "one firing is bounded"
    );
    assert_eq!(
        res.colony.ground_items[&(6, 12)].count,
        20,
        "planning cannot move or consume food"
    );
}

#[test]
fn enough_empty_compatible_capacity_does_not_expand_the_stockpile() {
    let (world, res, policy) = fixture(false);
    assert!(stock_commands(&world, &res, &policy).is_empty());
}

#[test]
fn canceled_storage_tiles_are_not_recreated_by_food_planning() {
    let (world, mut res, policy) = fixture(true);
    for y in 0..24 {
        for x in 0..24 {
            res.steward
                .vetoes
                .insert(Target::Tile(x, y), res.current_tick + res.ticks_per_day);
        }
    }
    assert!(stock_commands(&world, &res, &policy).is_empty());
}

#[test]
fn actual_food_above_winter_minimum_gets_its_real_stack_capacity() {
    let (world, mut res, policy) = fixture(true);
    let roots = ItemKind::from_name("roots").expect("roots");
    crate::runtime::food::items::create_ground(
        &mut res,
        (6, 12),
        ItemSpec::raw(roots, None),
        347,
        14,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("actual surplus");
    let stack_max = u64::from(crate::runtime::jobs::items::stack_max(&res, roots));
    let expected = 367_u64
        .div_ceil(stack_max)
        .min(u64::from(policy.harvest_batch));
    let commands = stock_commands(&world, &res, &policy);
    let cells: u64 = commands
        .iter()
        .map(|command| match command {
            Command::AddStockpile { x0, y0, x1, y1 } => {
                u64::from(x1 - x0 + 1) * u64::from(y1 - y0 + 1)
            }
            _ => unreachable!(),
        })
        .sum();
    assert_eq!(
        cells, expected,
        "real inventory, not only minimum demand, needs room"
    );
}

#[test]
fn ingredient_only_loose_food_gets_space_without_edible_reserve_credit() {
    let (world, res, policy) = fixture_with_food(true, "wild_grain");
    let grain = ItemKind::from_name("wild_grain").expect("grain");
    assert!(!grain.definition().food.as_ref().unwrap().raw_edible);
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.usable_units, 0, "raw grain is not edible reserve");
    assert_eq!(facts.pieces.get(&grain), Some(&20));
    let items_before = res.colony.ground_items.clone();
    let food_before = res.food.clone();
    let commands = stock_commands(&world, &res, &policy);
    let expected = 20_u64
        .div_ceil(u64::from(crate::runtime::jobs::items::stack_max(
            &res, grain,
        )))
        .min(u64::from(policy.harvest_batch));
    assert_eq!(
        commands.len() as u64,
        expected,
        "actual ingredient inventory needs haul capacity"
    );
    for command in commands {
        let Command::AddStockpile { x0, y0, x1, y1 } = command else {
            unreachable!()
        };
        assert_eq!((x0, y0), (x1, y1));
        assert!(!res.colony.ground_items.contains_key(&(x0, y0)));
        assert!(!res.colony.zone_of_tile.contains_key(&(x0, y0)));
        assert!(crate::runtime::jobs::items::can_hold_items(&res, (x0, y0)));
        assert!(crate::commands::camp_reaches(&res, (x0, y0)));
    }
    assert_eq!(
        res.colony.ground_items, items_before,
        "normal hauling must move the grain"
    );
    assert_eq!(
        res.food, food_before,
        "planning cannot consume or credit food"
    );
    crate::runtime::food::validate(&world, &res).expect("fresh conserved ingredient inventory");
}
