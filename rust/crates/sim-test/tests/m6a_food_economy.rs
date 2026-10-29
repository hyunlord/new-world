//! Custom colony gates use physical reserves and authored seasonal raw capacity.
use sim_core::colony::{content, food::PlantState, ItemKind, ItemSpec};
use sim_core::components::{BuildingKind, Hunger, ResourceKind, Settlement, Thirst};
use sim_core::material::MaterialRegistry;
use sim_core::rules::{BirthRule, SustainRule, WorldRules};
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::{
    food,
    settlement::{food_economy as economy, SettlementSystem},
};

fn species(engine: &SimEngine, item: &str) -> PlantState {
    let item = ItemKind::from_name(item).unwrap();
    *engine
        .resources
        .food
        .plants
        .values()
        .find(|plant| {
            content::installed_or_bundled()
                .plant(plant.species)
                .unwrap()
                .yield_item
                == item
        })
        .unwrap()
}

#[test]
fn seasonal_raw_supply_excludes_grain_and_reuses_cache_until_geometry_changes() {
    let mut engine = sim_test::colony_engine(0);
    let root = species(&engine, "roots");
    let grain = species(&engine, "wild_grain");
    engine.resources.food.plants.clear();
    engine.resources.food.plants.insert((10, 10), root);
    engine.resources.food.plants.insert((11, 10), grain);
    let mut village = Settlement::new_with_id(1, 0);
    village.formation_tile = (8, 10);
    engine.resources.settlements.clear();
    engine.resources.settlements.insert(1, village);
    engine
        .resources
        .nav
        .install(sim_core::nav::CostMap::new(192, 192));
    // Five roots every two days: 15 raw units per six-day ripe season.
    assert_eq!(
        economy::season_supply_per_day(&engine.resources, 1, 16, 0),
        2.5
    );
    assert_eq!(
        economy::season_supply_per_day(&engine.resources, 1, 16, 1),
        2.5
    );
    assert_eq!(economy::annual_mean_supply(&engine.resources, 1, 16), 1.875);
    assert_eq!(
        economy::winter_supply_per_day(&engine.resources, 1, 16),
        0.0
    );
    assert_eq!(engine.resources.food_supply_cache.borrow().rebuilds, 1);
    engine.resources.food.touch();
    engine
        .resources
        .food
        .plants
        .get_mut(&(10, 10))
        .unwrap()
        .units_left = 0;
    assert_eq!(economy::annual_mean_supply(&engine.resources, 1, 16), 1.875);
    assert_eq!(engine.resources.food_supply_cache.borrow().rebuilds, 1);
    assert!(engine
        .resources
        .nav
        .set_cost((10, 10), sim_core::nav::IMPASSABLE));
    let home_region = engine.resources.nav.region_of((8, 10));
    // Harvest navigation deliberately permits adjacent work on blocked goals.
    assert!(engine.resources.nav.reaches(home_region, (10, 10)));
    assert_eq!(economy::annual_mean_supply(&engine.resources, 1, 16), 1.875);
    assert_eq!(engine.resources.food_supply_cache.borrow().rebuilds, 2);
    for x in 9..=11 {
        for y in 9..=11 {
            engine
                .resources
                .nav
                .set_cost((x, y), sim_core::nav::IMPASSABLE);
        }
    }
    assert!(engine.resources.nav.passable((8, 10)));
    assert!(!engine
        .resources
        .nav
        .reaches(engine.resources.nav.region_of((8, 10)), (10, 10)));
    assert_eq!(economy::annual_mean_supply(&engine.resources, 1, 16), 0.0);
    assert_eq!(engine.resources.food_supply_cache.borrow().rebuilds, 3);
    for x in 9..=11 {
        for y in 9..=11 {
            engine
                .resources
                .nav
                .set_cost((x, y), sim_core::nav::COST_PLAIN);
        }
    }
    engine
        .resources
        .settlements
        .get_mut(&1)
        .unwrap()
        .formation_tile = (100, 100);
    assert_eq!(economy::annual_mean_supply(&engine.resources, 1, 16), 0.0);
    assert_eq!(engine.resources.food_supply_cache.borrow().rebuilds, 4);
}

#[test]
fn custom_provisioned_colony_uses_real_reserve_without_abstract_birth_tax() {
    let source = sim_test::colony_engine(0);
    let root = species(&source, "roots");
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules.colony = WorldRules::colony(0).colony;
    engine.resources.rules.calendar = sim_core::rules::CalendarRule::COLONY;
    engine.resources.rules.society.birth = BirthRule::Provisioned {
        cooldown_ticks: 1,
        food_cost: 500,
        max_mean_hunger: 60.0,
        max_mean_thirst: 60.0,
        sustain: Some(SustainRule {
            utilization: 1.0,
            reserve_days: 1.0,
            forage_radius: 16,
        }),
    };
    for _ in 0..3 {
        let person = engine.spawn_agent(10, 10);
        engine
            .world
            .insert(person, (Hunger::new(0.0, 0.05), Thirst::new(0.0, 0.0)))
            .unwrap();
    }
    engine.resources.building_registry.insert(100, (10, 10));
    engine
        .resources
        .building_kinds
        .insert((10, 10), BuildingKind::Hearth);
    engine.resources.building_registry.insert(101, (11, 10));
    engine
        .resources
        .building_kinds
        .insert((11, 10), BuildingKind::Shelter);
    for x in 12..28 {
        engine.resources.food.plants.insert((x, 12), root);
    }
    sim_systems::runtime::nav::ensure(&mut engine.resources);
    let mut system = SettlementSystem::new();
    system.tick(&mut engine.world, &mut engine.resources);
    let sid = *engine.resources.settlements.keys().next().unwrap();
    engine
        .resources
        .settlements
        .get_mut(&sid)
        .unwrap()
        .stockpile
        .insert(ResourceKind::Food, 999);
    engine.resources.current_tick = 1;
    system.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        engine.resources.tallies.births, 0,
        "abstract reserve cannot admit a colony birth"
    );
    let tile = (15, 15);
    assert!(
        sim_systems::commands::apply_command(
            &mut engine.world,
            &mut engine.resources,
            sim_core::colony::Command::AddStockpile {
                x0: tile.0,
                y0: tile.1,
                x1: tile.0,
                y1: tile.1
            }
        )
        .accepted
    );
    let kind = ItemKind::from_name("roots").unwrap();
    food::items::create_ground(
        &mut engine.resources,
        tile,
        ItemSpec::raw(kind, None),
        10,
        0,
        food::items::FoodSource::Debug,
    )
    .unwrap();
    engine.resources.rules.society.firewood = source.resources.rules.society.firewood;
    let members: Vec<_> = engine
        .world
        .query::<&Hunger>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    for &entity in &members {
        engine
            .world
            .insert_one(entity, sim_core::components::Cold)
            .unwrap();
    }
    engine.resources.current_tick = 2;
    system.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        engine.resources.tallies.births, 0,
        "ten stored units cannot cover three cold adults' 10.8-unit reserve"
    );
    for entity in members {
        engine
            .world
            .remove_one::<sim_core::components::Cold>(entity)
            .unwrap();
    }
    engine.resources.current_tick = 3;
    system.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(engine.resources.tallies.births, 1);
    assert_eq!(
        engine.resources.colony.ground_items[&tile].count, 10,
        "retired birth tax consumes no real items"
    );
    assert_eq!(
        engine.resources.settlements[&sid].stockpile[&ResourceKind::Food],
        999
    );
}

#[test]
fn baseline_source_arithmetic_does_not_populate_colony_cache() {
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    let mut village = Settlement::new_with_id(1, 0);
    village.formation_tile = (10, 10);
    engine.resources.settlements.insert(1, village);
    engine.resources.food_source_max.insert((11, 10), 4);
    assert_eq!(economy::supply_per_day(&engine.resources, 1, 16), 12.0);
    assert!(engine
        .resources
        .food_supply_cache
        .borrow()
        .entries
        .is_empty());
}

#[test]
fn colony_cold_demand_matches_actual_growth_and_stock_target() {
    use sim_core::components::{Agent, Cold};
    use sim_core::rules::StockTargetRule;
    use sim_systems::runtime::needs::HungerDecaySystem;

    let source = sim_test::colony_engine(0);
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules = source.resources.rules;
    engine.resources.rules.seasons = None;
    engine.resources.rules.society.stock_target = Some(StockTargetRule {
        base_days: 2.0,
        hoard_cap_factor: 1.0,
    });
    let adult = engine
        .world
        .spawn((Agent { id: 1 }, Hunger::new(0.0, 0.05), Cold));
    let child = engine
        .world
        .spawn((Agent { id: 2 }, Hunger::new(0.0, 0.03)));
    let mut village = Settlement::new_with_id(1, 0);
    village.add_member_agent(1);
    village.add_member_agent(2);
    engine.resources.settlements.insert(1, village);
    for multiplier in [1500, 1750] {
        engine
            .resources
            .rules
            .society
            .firewood
            .as_mut()
            .unwrap()
            .cold_hunger_permille = multiplier;
        engine.world.get::<&mut Hunger>(adult).unwrap().value = 0.0;
        engine.world.get::<&mut Hunger>(child).unwrap().value = 0.0;
        let projected = economy::members_need_per_day(&engine.world, &engine.resources, 1);
        HungerDecaySystem::new().tick(&mut engine.world, &mut engine.resources);
        let adult_growth = engine.world.get::<&Hunger>(adult).unwrap().value;
        let child_growth = engine.world.get::<&Hunger>(child).unwrap().value;
        assert_eq!(child_growth, 0.03, "warm child rate is unchanged");
        let actual_daily = (f64::from(adult_growth) + f64::from(child_growth))
            * engine.resources.ticks_per_day as f64
            / 30.0;
        assert!((projected - actual_daily).abs() < 1e-12);
        assert!(
            (economy::stock_target(&engine.world, &engine.resources, 1).unwrap()
                - 2.0 * actual_daily)
                .abs()
                < 1e-12
        );
    }
    engine.resources.rules.society.firewood = None;
    let warm = economy::need_per_day(&Hunger::new(0.0, 0.05), engine.resources.ticks_per_day)
        + economy::need_per_day(&Hunger::new(0.0, 0.03), engine.resources.ticks_per_day);
    assert_eq!(
        economy::members_need_per_day(&engine.world, &engine.resources, 1),
        warm
    );
    engine.resources.rules.society.firewood = source.resources.rules.society.firewood;
    engine.resources.rules.colony = None;
    assert_eq!(
        economy::members_need_per_day(&engine.world, &engine.resources, 1),
        warm,
        "baseline preserves its historical projection even with Cold"
    );
}
