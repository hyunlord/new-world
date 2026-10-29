//! Food plans distinguish edible storage, pending harvest and player overrides.
use sim_core::colony::steward::{rules, Action, FoodPlan, Target};
use sim_core::colony::{Command, ItemKind, ItemSpec};
use sim_systems::runtime::{food, steward};

fn policy() -> FoodPlan {
    rules()
        .unwrap()
        .rules
        .iter()
        .find_map(|rule| match &rule.action {
            Action::FoodPlan(policy) => Some(policy.clone()),
            _ => None,
        })
        .unwrap()
}

#[test]
fn winter_owner_bar_rounds_once_per_village() {
    assert_eq!(steward::food::winter_need(0), 0);
    assert_eq!(steward::food::winter_need(1), 15);
    assert_eq!(steward::food::winter_need(10), 144);
}

#[test]
fn inedible_grain_and_unstored_food_never_inflate_stored_units() {
    let mut engine = sim_test::colony_engine(0);
    let policy = policy();
    let tile = engine
        .resources
        .food
        .plants
        .keys()
        .copied()
        .find(|tile| {
            sim_systems::commands::camp_reaches(&engine.resources, *tile)
                && sim_systems::runtime::jobs::items::can_hold_items(&engine.resources, *tile)
        })
        .unwrap();
    let grain = ItemKind::from_name("wild_grain").unwrap();
    food::items::create_ground(
        &mut engine.resources,
        tile,
        ItemSpec::raw(grain, None),
        5,
        0,
        food::items::FoodSource::Harvest,
    )
    .unwrap();
    let result = sim_systems::commands::apply_command(
        &mut engine.world,
        &mut engine.resources,
        Command::AddStockpile {
            x0: tile.0,
            y0: tile.1,
            x1: tile.0,
            y1: tile.1,
        },
    );
    assert!(result.accepted);
    let facts = steward::food::summary(&engine.world, &engine.resources, &policy);
    assert_eq!(facts.stored_units, 0);
    assert_eq!(facts.usable_units, 0);
    assert_eq!(facts.pieces.get(&grain), Some(&5));
}

#[test]
fn vetoed_harvest_tile_is_never_reissued() {
    let mut engine = sim_test::colony_engine(0);
    let policy = policy();
    let commands = steward::food::plan(&engine.world, &engine.resources, &policy);
    let tiles: Vec<_> = commands
        .iter()
        .filter_map(|command| match command {
            Command::DesignateHarvest { x0, y0, .. } => Some((*x0, *y0)),
            _ => None,
        })
        .collect();
    assert!(!tiles.is_empty());
    for &(x, y) in &tiles {
        engine
            .resources
            .steward
            .vetoes
            .insert(Target::Tile(x, y), u64::MAX);
    }
    let again = steward::food::plan(&engine.world, &engine.resources, &policy);
    assert!(again.iter().all(|command| match command {
        Command::DesignateHarvest { x0, y0, .. } => !tiles.contains(&(*x0, *y0)),
        _ => true,
    }));
}

fn grant(engine: &mut sim_engine::SimEngine, name: &str) {
    use sim_core::colony::knowledge::{self, Knowledge, Source};
    let id = knowledge::id(name).unwrap();
    for (_, knowledge) in engine.world.query::<&mut Knowledge>().iter() {
        knowledge.set(id, 1000, Source::Seeded, 0);
    }
}

fn lit_camp(engine: &mut sim_engine::SimEngine) -> (u32, u32) {
    let tile = sim_systems::worldgen::colony::hearth(&engine.resources).unwrap();
    let mut settlement = sim_core::components::Settlement::new_with_id(1, 0);
    settlement.formation_tile = tile;
    engine.resources.settlements.insert(1, settlement);
    engine
        .resources
        .fires
        .insert(1, sim_engine::FireState { lit: true });
    sim_systems::runtime::nav::ensure(&mut engine.resources);
    tile
}

#[test]
fn undiscovered_pot_recipe_requests_authored_clay_discovery_supply() {
    let mut engine = sim_test::colony_engine(0);
    grant(&mut engine, "clay_digging");
    let policy = policy();
    let commands = steward::food::plan(&engine.world, &engine.resources, &policy);
    let clay = commands
        .into_iter()
        .find(|command| matches!(command, Command::DesignateClay { .. }))
        .expect("pot prerequisite stockpile clay is requested before form_pot is known");
    let result =
        sim_systems::commands::apply_command(&mut engine.world, &mut engine.resources, clay);
    assert!(result.accepted);
    assert!(engine
        .resources
        .colony
        .designations
        .values()
        .any(|designation| *designation == sim_core::colony::Designation::ClayPit));
}

#[test]
fn cooking_bill_can_request_reachable_vessel_and_spare_still_deploys() {
    use sim_core::colony::{ItemStack, RecipeKind};
    let mut engine = sim_test::colony_engine(0);
    let tile = lit_camp(&mut engine);
    grant(&mut engine, "cooking");
    let mut policy = policy();
    let stew = RecipeKind::from_name("stew").unwrap();
    policy.recipes = vec![stew];
    policy.vessel_recipes.clear();
    for (name, count) in [("meat", 1), ("roots", 1)] {
        let kind = ItemKind::from_name(name).unwrap();
        food::items::create_ground(
            &mut engine.resources,
            tile,
            ItemSpec::raw(kind, None),
            count,
            0,
            food::items::FoodSource::Debug,
        )
        .unwrap();
    }
    let pot_tile = engine
        .resources
        .food
        .plants
        .keys()
        .copied()
        .find(|at| {
            !engine.resources.colony.ground_items.contains_key(at)
                && sim_systems::commands::camp_reaches(&engine.resources, *at)
                && sim_systems::runtime::jobs::items::can_hold_items(&engine.resources, *at)
        })
        .unwrap();
    let pot = ItemKind::from_name("clay_pot").unwrap();
    engine
        .resources
        .colony
        .ground_items
        .insert(pot_tile, ItemStack::of(ItemSpec::raw(pot, None), 2));
    engine.resources.colony.item_ledger.create(pot, 2);
    assert!(
        sim_systems::commands::apply_command(
            &mut engine.world,
            &mut engine.resources,
            Command::AddStockpile {
                x0: pot_tile.0,
                y0: pot_tile.1,
                x1: pot_tile.0,
                y1: pot_tile.1
            }
        )
        .accepted
    );
    let commands = steward::food::plan(&engine.world, &engine.resources, &policy);
    assert!(commands
        .iter()
        .any(|cmd| matches!(cmd, Command::AddBill { recipe, .. } if *recipe == stew)));
    assert!(!commands
        .iter()
        .any(|cmd| matches!(cmd, Command::DeployFoodPot { tile } if *tile == pot_tile)));
    let workshop = engine
        .resources
        .colony
        .workshops
        .iter()
        .find(|(_, shop)| stew.recipe().unwrap().supports_workshop(shop.kind))
        .map(|(id, _)| *id)
        .unwrap();
    let owner = sim_core::colony::food::FoodOwner::Shelf {
        workshop,
        key: sim_core::colony::food::FoodStackKey::from(ItemSpec::raw(pot, None)),
    };
    food::items::transfer(
        &mut engine.world,
        &mut engine.resources,
        sim_core::colony::food::FoodOwner::Ground(pot_tile),
        owner,
        1,
        0,
    )
    .unwrap();
    let after = steward::food::plan(&engine.world, &engine.resources, &policy);
    assert!(after
        .iter()
        .any(|cmd| matches!(cmd, Command::DeployFoodPot { tile } if *tile == pot_tile)));
}

#[test]
fn one_leftover_grain_does_not_block_recipe_ingredient_harvest() {
    use sim_core::colony::{content, RecipeKind};
    let mut engine = sim_test::colony_engine(0);
    let tile = lit_camp(&mut engine);
    grant(&mut engine, "cooking");
    grant(&mut engine, "grain_parching");
    engine.resources.current_tick = 9 * engine.resources.ticks_per_day;
    let mut policy = policy();
    policy.recipes = vec![RecipeKind::from_name("porridge").unwrap()];
    policy.vessel_recipes.clear();
    let grain = ItemKind::from_name("wild_grain").unwrap();
    let plant_tile = engine
        .resources
        .food
        .plants
        .iter()
        .filter(|(at, plant)| {
            content::installed_or_bundled()
                .plant(plant.species)
                .unwrap()
                .yield_item
                == grain
                && at.0.abs_diff(tile.0).max(at.1.abs_diff(tile.1)) <= policy.forage_radius
                && sim_systems::commands::camp_reaches(&engine.resources, **at)
        })
        .map(|(at, _)| *at)
        .next()
        .unwrap();
    engine
        .resources
        .food
        .plants
        .retain(|at, _| *at == plant_tile);
    engine
        .resources
        .food
        .plants
        .get_mut(&plant_tile)
        .unwrap()
        .units_left = 6;
    let day = food::day(&engine.resources);
    for (kind, count) in [(grain, 1), (ItemKind::role("legacy_food"), 200)] {
        food::items::create_ground(
            &mut engine.resources,
            tile,
            ItemSpec::raw(kind, None),
            count,
            day,
            food::items::FoodSource::Debug,
        )
        .unwrap();
    }
    assert!(food::plants::harvestable(&engine.resources, plant_tile));
    let commands = steward::food::plan(&engine.world, &engine.resources, &policy);
    assert!(commands
        .iter()
        .any(|cmd| matches!(cmd, Command::DesignateHarvest { x0, y0, .. }
        if (*x0, *y0) == plant_tile)));
}
