//! Winter horizons and food-cap procurement boundaries.
use hecs::World;
use sim_core::colony::food::PlantState;
use sim_core::colony::steward::{Action, FoodPlan};
use sim_core::colony::{Command, ItemKind, ItemSpec};
use sim_core::components::{Agent, Position, Settlement};
use sim_core::rules::WorldRules;
use sim_engine::SimResources;

fn fixture() -> (World, SimResources, FoodPlan) {
    let mut world = World::new();
    let mut res = SimResources::new(16, 16, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(1);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.current_tick = 14 * res.ticks_per_day;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (8, 8);
    res.settlements.insert(1, settlement);
    world.spawn((Agent { id: 1 }, Position::new(8, 8)));
    assert!(
        crate::commands::apply_command(
            &mut world,
            &mut res,
            Command::AddStockpile {
                x0: 7,
                y0: 7,
                x1: 8,
                y1: 7
            }
        )
        .accepted
    );
    let mut policy = sim_core::colony::steward::rules()
        .expect("rules")
        .rules
        .iter()
        .find_map(|rule| match &rule.action {
            Action::FoodPlan(p) => Some(p.clone()),
            _ => None,
        })
        .expect("food policy");
    policy.winter_keep_days = 6;
    policy.recipes.clear();
    policy.vessel_recipes.clear();
    (world, res, policy)
}

#[test]
fn winter_horizon_counts_expiry_at_end_but_not_the_previous_day() {
    let (world, mut res, policy) = fixture();
    let roots = ItemKind::from_name("roots").expect("roots");
    for (tile, born, count) in [((7, 7), 13, 3), ((8, 7), 14, 5)] {
        crate::runtime::food::items::create_ground(
            &mut res,
            tile,
            ItemSpec::raw(roots, None),
            count,
            born,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("actual stock");
    }
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.stored_units, 8);
    assert_eq!(
        facts.winter_units, 5,
        "expiry23 does not cover [18,24), expiry24 does"
    );
}

#[test]
fn winter_horizon_harvest_accepts_same_expiry_boundary_as_stored_food() {
    let (world, mut res, policy) = fixture();
    let catalog = sim_core::colony::content::installed_or_bundled();
    let species = catalog.plant_by_name("wild_root").expect("root plant");
    res.food.plants.insert(
        (9, 8),
        PlantState {
            species,
            units_left: 5,
            next_regrow_day: u32::MAX,
        },
    );
    let mut facts = super::summary(&world, &res, &policy);
    facts.usable_units = 1000;
    facts.groups.extend(
        ItemKind::from_name("roots")
            .expect("roots")
            .definition()
            .food
            .as_ref()
            .expect("food")
            .groups
            .iter()
            .cloned(),
    );
    let commands = super::harvest::plan(&world, &res, &policy, &facts);
    assert!(
        commands
            .iter()
            .any(|command| matches!(command, Command::DesignateHarvest { .. })),
        "fresh day14 roots expire24 and must fill the winter-only gap: {commands:?}"
    );
}

fn food_cap_fixture(stored: u16, reserve_roots: bool) -> (World, SimResources, FoodPlan) {
    let (mut world, mut res, mut policy) = fixture();
    res.current_tick = 6 * res.ticks_per_day;
    policy.reserve_days = 2;
    let roots = ItemKind::from_name("roots").expect("roots");
    if stored > 0 {
        crate::runtime::food::items::create_ground(
            &mut res,
            (7, 7),
            ItemSpec::raw(roots, None),
            stored,
            6,
            crate::runtime::food::items::FoodSource::Debug,
        )
        .expect("physical fresh stock");
    }
    let catalog = sim_core::colony::content::installed_or_bundled();
    let berry = catalog.plant_by_name("berry_bush").expect("berry plant");
    res.food.plants.insert(
        (9, 8),
        PlantState {
            species: berry,
            units_left: 4,
            next_regrow_day: u32::MAX,
        },
    );
    assert!(crate::runtime::food::plants::harvestable(&res, (9, 8)));
    if reserve_roots {
        let species = catalog.plant_by_name("wild_root").expect("root plant");
        res.food.plants.insert(
            (10, 8),
            PlantState {
                species,
                units_left: 5,
                next_regrow_day: u32::MAX,
            },
        );
        assert!(
            crate::commands::apply_command(
                &mut world,
                &mut res,
                Command::DesignateHarvest {
                    x0: 10,
                    y0: 8,
                    x1: 10,
                    y1: 8
                },
            )
            .accepted
        );
    }
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.adults, 1);
    assert_eq!(facts.daily_milli, 2400);
    assert_eq!(facts.usable_units, u64::from(stored));
    assert!(
        !facts.preparing,
        "day6 precedes the winter preparation window"
    );
    assert_eq!(facts.groups.contains("root"), stored > 0);
    assert!(!facts.groups.contains("fruit"));
    assert!(super::dependencies::ingredient_deficits(&world, &res, &policy, &facts).is_empty());
    (world, res, policy)
}

#[test]
fn food_cap_stops_diversity_at_target_but_keeps_a_real_whole_plant_gap() {
    // One warm adult's two-day target is ceil(2.4 * 2) = 5 food units.
    for stored in [5, 4] {
        let (world, res, policy) = food_cap_fixture(stored, false);
        let facts = super::summary(&world, &res, &policy);
        let commands = super::harvest::plan(&world, &res, &policy, &facts);
        let expected = if stored == 5 {
            Vec::new()
        } else {
            // A one-unit gap permits one indivisible four-berry harvest.
            vec![Command::DesignateHarvest {
                x0: 9,
                y0: 8,
                x1: 9,
                y1: 8,
            }]
        };
        assert_eq!(
            commands, expected,
            "stored={stored}; target=5; berry yield=4"
        );
    }
}

#[test]
fn food_cap_counts_reserved_harvest_before_admitting_diversity() {
    let (world, res, policy) = food_cap_fixture(3, true);
    assert_eq!(
        res.colony.designations.get(&(10, 8)),
        Some(&sim_core::colony::Designation::Harvest),
    );
    // 3 fresh units + one already ordered 5-root plant = 8, covering target5.
    // Its existing whole-plant residual does not justify another 4-berry order.
    assert_eq!(res.food.plants[&(10, 8)].units_left, 5);
    let facts = super::summary(&world, &res, &policy);
    let commands = super::harvest::plan(&world, &res, &policy, &facts);
    assert!(
        commands.is_empty(),
        "stored3 + reserved5 >= target5: {commands:?}"
    );
}

#[test]
fn replacement_area_has_a_floor_even_when_wild_stores_are_full() {
    let catalog = sim_core::colony::content::installed_or_bundled();
    let crop = catalog.crop_by_name("emmer").expect("emmer data");
    let policy = sim_core::colony::steward::CultivationPlan {
        crop,
        replacement_permille: 300,
        net_units_per_tile: 12,
        area_cap_permille: 1250,
        batch: 4,
    };
    let (world, res, food_policy) = fixture();
    for (daily, annual, target, cap) in [
        (24000, 648000, 17, 21),
        (48000, 1296000, 33, 41),
        (96000, 2592000, 65, 81),
    ] {
        let mut facts = super::summary(&world, &res, &food_policy);
        facts.warm_daily_milli = daily;
        for current_daily in [daily, daily * 3 / 2] {
            facts.daily_milli = current_daily;
            assert_eq!(
                super::annual_food_milli(&res, &food_policy, &facts, 24),
                annual,
                "current Cold does not resize the yearly replacement target"
            );
            assert_eq!(super::cultivation_area(&policy, annual), (target, cap));
            assert!(cap * 1000 <= target * 1250);
        }
    }
}

#[test]
fn full_wild_food_store_still_accepts_replacement_fields_once() {
    let (mut world, mut res, policy) = food_cap_fixture(5, false);
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.usable_units, 5);
    let commands = super::field_orders(&res, &policy, &facts);
    assert_eq!(commands.len(), 2, "one adult needs ceil(64.8*0.3/12) tiles");
    for command in commands {
        assert!(crate::commands::apply_command(&mut world, &mut res, command).accepted);
    }
    assert_eq!(res.cultivation.fields.len(), 2);
    assert!(super::field_orders(&res, &policy, &facts).is_empty());
}

#[test]
fn seed_bootstrap_uses_real_grain_harvest_despite_full_food_budget() {
    let (world, mut res, policy) = food_cap_fixture(5, false);
    res.current_tick = 9 * res.ticks_per_day;
    let catalog = sim_core::colony::content::installed_or_bundled();
    let grain = catalog.plant_by_name("wild_grain").expect("grain plant");
    res.food.plants.insert(
        (10, 8),
        PlantState {
            species: grain,
            units_left: 6,
            next_regrow_day: u32::MAX,
        },
    );
    assert!(crate::runtime::food::plants::harvestable(&res, (10, 8)));
    let mut facts = super::summary(&world, &res, &policy);
    facts.preparing = false;
    let seed = ItemKind::from_name("emmer_seed").expect("seed");
    assert_eq!(super::seed_reserve(&res, &policy, &facts), Some((seed, 4)));
    assert_eq!(
        super::harvest::plan(&world, &res, &policy, &facts),
        vec![Command::DesignateHarvest {
            x0: 10,
            y0: 8,
            x1: 10,
            y1: 8,
        }]
    );
    assert_eq!(
        facts.pieces.get(&seed),
        None,
        "planner never invents seed stock"
    );
}

#[test]
fn cooking_shares_the_raw_food_reserve_without_double_counting_cooked_stock() {
    let output = ItemKind::from_name("porridge").expect("cooked output");
    for (raw, cooked_target) in [(5, 0), (4, 1)] {
        let (world, res, policy) = food_cap_fixture(raw, false);
        let facts = super::summary(&world, &res, &policy);
        assert_eq!(
            super::cooking::daily_target(&policy, &facts, 1, output),
            cooked_target
        );
    }
    let (world, mut res, policy) = food_cap_fixture(0, false);
    crate::runtime::food::items::create_ground(
        &mut res,
        (7, 7),
        ItemSpec::raw(output, None),
        3,
        6,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("physical cooked stock");
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.usable_units, 3);
    assert_eq!(facts.cooked_units, 3);
    assert_eq!(
        super::cooking::daily_target(&policy, &facts, 1, output),
        3,
        "existing cooked portions remain part of Maintain, not raw-stock credit"
    );
}

#[test]
fn nonfood_seed_bootstrap_harvest_has_no_edible_credit_and_stops_with_physical_input() {
    let (mut world, mut res, mut policy) = food_cap_fixture(5, false);
    let catalog = sim_core::colony::content::installed_or_bundled();
    policy
        .cultivation
        .as_mut()
        .expect("cultivation policy")
        .crop = catalog.crop_by_name("flax").expect("flax");
    res.current_tick = 9 * res.ticks_per_day;
    let species = catalog.plant_by_name("wild_flax").expect("wild flax");
    let item = catalog.plant(species).expect("plant").yield_item;
    assert!(item.definition().food.is_none());
    res.food.plants.insert(
        (10, 8),
        PlantState {
            species,
            units_left: 4,
            next_regrow_day: u32::MAX,
        },
    );
    assert!(crate::runtime::food::plants::harvestable(&res, (10, 8)));
    let mut facts = super::summary(&world, &res, &policy);
    facts.preparing = false;
    let discovery_grain =
        ItemKind::from_name("wild_grain").expect("authored cultivation discovery");
    let reserved = super::dependencies::seed_inputs(&world, &res, &policy, &facts);
    assert_eq!(
        reserved.get(&discovery_grain),
        Some(&2),
        "flax conversion inputs must not erase the separate cultivation discovery floor"
    );
    let commands = super::harvest::plan(&world, &res, &policy, &facts);
    assert_eq!(
        commands,
        vec![Command::DesignateHarvest {
            x0: 10,
            y0: 8,
            x1: 10,
            y1: 8
        }]
    );
    assert!(crate::commands::apply_command(&mut world, &mut res, commands[0].clone()).accepted);
    assert!(
        super::harvest::plan(&world, &res, &policy, &facts).is_empty(),
        "accepted input harvest counts once"
    );
    res.colony.designations.clear();
    res.colony.ground_items.insert(
        (10, 7),
        sim_core::colony::ItemStack::of(ItemSpec::raw(item, None), 100),
    );
    let after = super::summary(&world, &res, &policy);
    assert_eq!(after.usable_units, facts.usable_units);
    let mut after = after;
    after.preparing = false;
    assert!(
        super::harvest::plan(&world, &res, &policy, &after).is_empty(),
        "physical stalks end bootstrap harvesting"
    );
}
