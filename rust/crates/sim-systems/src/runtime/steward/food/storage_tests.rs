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

#[test]
fn open_stockpile_capacity_does_not_block_finite_covered_food_storage() {
    use sim_engine::RuntimeSystem;
    let (mut world, mut res, policy) = fixture(false);
    crate::runtime::food::items::transfer(
        &mut world,
        &mut res,
        sim_core::colony::food::FoodOwner::Ground((6, 12)),
        sim_core::colony::food::FoodOwner::Ground((8, 14)),
        20,
        14,
    )
    .expect("food actually stored outdoors");
    assert!(
        stock_commands(&world, &res, &policy).is_empty(),
        "outdoor capacity already meets old bound"
    );
    // Physical built perimeter, then the actual room detector and role command.
    for y in 8..=13 {
        for x in 15..=20 {
            if x == 15 || x == 20 || y == 8 || y == 13 {
                let i = res.tile_grid.idx(x, y);
                res.tile_grid.wall_material[i] = Some(sim_core::colony::materials::OAK);
            }
        }
    }
    let door = res.tile_grid.idx(15, 11);
    res.tile_grid.set_door(door, true);
    res.colony.rooms_dirty = true;
    crate::runtime::rooms::RoomSystem::new().tick(&mut world, &mut res);
    let room = res.colony.room_of_tile[&(16, 9)];
    for role in [
        sim_core::rooms::RoomRole::Bedroom,
        sim_core::rooms::RoomRole::Workshop,
        sim_core::rooms::RoomRole::Hall,
    ] {
        assert!(
            crate::commands::apply_command(
                &mut world,
                &mut res,
                Command::SetRoomRole {
                    room,
                    role: Some(role)
                }
            )
            .accepted
        );
        assert!(
            stock_commands(&world, &res, &policy).is_empty(),
            "extra food demand must not zone a {role:?} or spill outdoors"
        );
    }
    assert!(
        crate::commands::apply_command(
            &mut world,
            &mut res,
            Command::SetRoomRole {
                room,
                role: Some(sim_core::rooms::RoomRole::Storehouse)
            }
        )
        .accepted
    );
    let covered = res.colony.rooms[&room].tiles.clone();
    assert!(res.colony.rooms[&room].roofed);
    assert!(covered
        .iter()
        .all(|tile| !res.colony.zone_of_tile.contains_key(tile)));
    for &(x, y) in &covered {
        res.steward
            .vetoes
            .insert(Target::Tile(x, y), res.current_tick + res.ticks_per_day);
    }
    assert!(
        stock_commands(&world, &res, &policy).is_empty(),
        "covered demand cannot bypass player veto or spill outdoors"
    );
    for &(x, y) in &covered {
        res.steward.vetoes.remove(&Target::Tile(x, y));
    }
    let occupied = (16, 9);
    crate::runtime::food::items::create_ground(
        &mut res,
        occupied,
        ItemSpec::raw(ItemKind::from_name("firewood").expect("fuel"), None),
        1,
        14,
        crate::runtime::food::items::FoodSource::Debug,
    )
    .expect("occupied covered cell");
    let field = (17, 9);
    res.cultivation.fields.insert(
        field,
        sim_core::colony::cultivation::FieldTile {
            owner: 1,
            crop: policy.cultivation.as_ref().expect("crop plan").crop,
            phase: sim_core::colony::cultivation::FieldPhase::Sown { planted_day: 14 },
            active: false,
            work: sim_core::colony::cultivation::FieldWork::Paused,
            tended: false,
        },
    );
    let wall_index = res.tile_grid.idx(16, 8);
    let wall = res.tile_grid.wall_material[wall_index].take();
    assert!(
        stock_commands(&world, &res, &policy).is_empty(),
        "stale roof metadata without a complete physical Store cannot add capacity"
    );
    res.tile_grid.wall_material[wall_index] = wall;
    let old_zones = res.colony.zone_of_tile.clone();
    let old_food = res.food.clone();
    let old_items = res.colony.ground_items.clone();
    let commands = stock_commands(&world, &res, &policy);
    assert!(
        !commands.is_empty(),
        "fresh outdoor food needs usable covered destination cells despite ample open zones"
    );
    assert!(
        commands.len() <= usize::try_from(policy.harvest_batch).expect("bounded harvest batch")
    );
    for command in commands {
        let Command::AddStockpile { x0, y0, x1, y1 } = command else {
            unreachable!()
        };
        assert_eq!((x0, y0), (x1, y1));
        assert_ne!((x0, y0), occupied);
        assert_ne!((x0, y0), field);
        assert!(
            covered.contains(&(x0, y0)),
            "covered demand must not create more outdoor zones"
        );
        assert!(crate::commands::apply_command(&mut world, &mut res, command).accepted);
    }
    assert!(old_zones
        .iter()
        .all(|(tile, id)| res.colony.zone_of_tile.get(tile) == Some(id)));
    assert_eq!(
        res.food, old_food,
        "planning cannot relocate or rejuvenate food"
    );
    assert_eq!(res.colony.ground_items, old_items);
    // Accepted empty capacity must count on subsequent firings, before any hauling.
    for _ in 0..covered.len() {
        let next = stock_commands(&world, &res, &policy);
        if next.is_empty() {
            break;
        }
        for command in next {
            let Command::AddStockpile { x0, y0, .. } = command else {
                unreachable!()
            };
            assert!(covered.contains(&(x0, y0)));
            assert_ne!((x0, y0), occupied);
            assert_ne!((x0, y0), field);
            assert!(crate::commands::apply_command(&mut world, &mut res, command).accepted);
        }
    }
    assert!(
        stock_commands(&world, &res, &policy).is_empty(),
        "finite covered target converges without moving food"
    );
    let stable = res.colony.zone_of_tile.clone();
    for _ in 0..3 {
        assert!(stock_commands(&world, &res, &policy).is_empty());
    }
    assert_eq!(res.colony.zone_of_tile, stable);
    assert_eq!(res.colony.ground_items, old_items);
    assert_eq!(res.food, old_food);
}

#[test]
fn warm_people_get_storage_for_the_cold_winter_forecast() {
    let (mut world, res, mut policy) = fixture(true);
    for id in 10..12 {
        world.spawn((
            Agent { id },
            Position::new(12, 12),
            sim_core::components::Child { born_tick: 0 },
        ));
    }
    policy.harvest_batch = 64;
    assert_eq!(policy.reserve_days, 2);
    let roots = ItemKind::from_name("roots").expect("roots");
    let stack = u64::from(crate::runtime::jobs::items::stack_max(&res, roots));
    let expected = |units: u64| (1 + (units - 20).div_ceil(stack)).min(1 + units.div_ceil(stack));
    assert!(
        expected(242) > expected(162),
        "forecast crosses actual stack capacity"
    );
    let mut facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.daily_milli, 26880);
    assert!(facts.preparing);
    for (preparing, units) in [(true, 242), (false, 54)] {
        facts.preparing = preparing;
        let commands = super::storage::plan(&res, &policy, &facts);
        let cells: u64 = commands
            .iter()
            .map(|command| match command {
                Command::AddStockpile { x0, y0, x1, y1 } => {
                    u64::from(x1 - x0 + 1) * u64::from(y1 - y0 + 1)
                }
                _ => unreachable!(),
            })
            .sum();
        assert_eq!(cells, expected(units), "preparing={preparing}");
    }
}
