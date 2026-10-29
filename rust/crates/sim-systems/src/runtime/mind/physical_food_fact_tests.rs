//! Registered colony food facts observe physical owners, including custom-policy inputs.
use super::*;
use crate::runtime::food::items::{self, FoodSource};
use sim_core::colony::food::FoodOwner;
use sim_core::colony::{ItemKind, ItemSpec, Zone, ZoneKind};
use sim_core::components::{Agent, Hunger, Inventory, Position, ResourceKind, Settlement};
use sim_core::rules::{ForageRule, StockTargetRule, WorldRules};
use std::collections::BTreeSet;

fn fixture() -> (hecs::World, SimResources) {
    let mut world = hecs::World::new();
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res.rules.seasons = None;
    res.rules.agent.forage = Some(ForageRule {
        start_minute: 0,
        end_minute: 1440,
        forage_radius: 8,
        haul_threshold: 5,
        max_foragers_per_bush: 2,
    });
    res.rules.society.stock_target = Some(StockTargetRule {
        base_days: 10.0,
        hoard_cap_factor: 1.5,
    });
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.nav.install(sim_core::nav::CostMap::new(8, 8));
    let mut inventory = Inventory::default();
    inventory.add(ResourceKind::Food, 9);
    inventory.add(ResourceKind::Wood, 1);
    world.spawn((
        Agent { id: 7 },
        Position::new(2, 2),
        AgentState::Idle,
        Hunger::new(50.0, 0.05),
        inventory,
    ));
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (2, 2);
    settlement.add_member_agent(7);
    settlement.stockpile.insert(ResourceKind::Food, 999);
    res.settlements.insert(1, settlement);
    res.food_tiles.insert((3, 2), 5);
    res.colony.zones.insert(
        1,
        Zone {
            id: 1,
            kind: ZoneKind::Stockpile,
            tiles: BTreeSet::from([(2, 2)]),
        },
    );
    let roots = ItemSpec::raw(ItemKind::from_name("roots").expect("roots"), None);
    items::create_ground(&mut res, (2, 2), roots, 7, 0, FoodSource::Harvest).expect("food");
    items::transfer(
        &mut world,
        &mut res,
        FoodOwner::Ground((2, 2)),
        FoodOwner::Carried(7),
        2,
        0,
    )
    .expect("carry");
    (world, res)
}

fn read(context: &TickContext, res: &SimResources, name: &str) -> i64 {
    let provider = BUILTIN_PROVIDERS
        .iter()
        .find(|p| p.name == name)
        .expect("registered");
    (provider.read)(context.row(7).expect("person"), context, res, provider.name).expect("fact")
}

#[test]
fn physical_food_facts_ignore_legacy_poison_and_keep_nonfood_inventory() {
    let (world, mut res) = fixture();
    let before = res.food.clone();
    let context = prepare_context(&world, &res).expect("context");
    for (name, expected) in [
        ("inventory_food", 2),
        ("inventory_total", 3),
        ("inventory_room", 7),
        ("food_carry_loaded", 1),
        ("camp_stocked", 1),
        ("stock_in_hand", 7),
        ("stock_shortfall", 1),
    ] {
        assert_eq!(read(&context, &res, name), expected, "{name}");
    }
    assert_eq!(res.food, before, "projection cannot mutate ownership");
    res.current_tick = 6 * res.ticks_per_day;
    let context = prepare_context(&world, &res).expect("context");
    assert_eq!(read(&context, &res, "inventory_food"), 0);
    assert_eq!(read(&context, &res, "food_carry_loaded"), 0);
    assert_eq!(read(&context, &res, "stock_in_hand"), 5);
    // Expired items still occupy their physical carry until the daily removal.
    assert_eq!(read(&context, &res, "inventory_total"), 3);
    res.current_tick = 10 * res.ticks_per_day;
    let context = prepare_context(&world, &res).expect("context");
    assert_eq!(read(&context, &res, "stock_in_hand"), 0);
    assert_eq!(read(&context, &res, "camp_stocked"), 0);
    res.rules = WorldRules::baseline(0);
    let context = prepare_context(&world, &res).expect("baseline context");
    assert_eq!(read(&context, &res, "inventory_food"), 9);
    assert_eq!(read(&context, &res, "inventory_total"), 10);
    assert_eq!(read(&context, &res, "inventory_room"), 0);
    assert_eq!(read(&context, &res, "stock_in_hand"), 1008);
}

#[test]
fn custom_colony_policy_cannot_reactivate_abstract_food_targets() {
    let (world, res) = fixture();
    let mut context = prepare_context(&world, &res).expect("context");
    for name in ["gather_available", "forage_available", "forage_window"] {
        assert_eq!(read(&context, &res, name), 0, "{name}");
    }
    for target in ["gather_food", "forage_bush"] {
        assert!(resolve_target(&mut context, &res, 7, target)
            .expect("target")
            .is_none());
    }
}

#[test]
fn colony_food_facts_require_edibility_and_age_but_carry_still_takes_space() {
    let (mut world, mut res) = fixture();
    let entity = world.query::<&Agent>().iter().next().expect("agent").0;
    let grain = ItemSpec::raw(ItemKind::from_name("wild_grain").expect("grain"), None);
    world
        .insert_one(entity, sim_core::colony::Carry::of(grain, 2))
        .expect("grain carry");
    let context = prepare_context(&world, &res).expect("context");
    assert_eq!(read(&context, &res, "inventory_food"), 0);
    assert_eq!(read(&context, &res, "inventory_total"), 3);
    assert_eq!(read(&context, &res, "stock_in_hand"), 5);
    let roots = ItemSpec::raw(ItemKind::from_name("roots").expect("roots"), None);
    world
        .insert_one(entity, sim_core::colony::Carry::of(roots, 2))
        .expect("roots carry");
    res.food.ages.remove(&FoodOwner::Carried(7));
    let context = prepare_context(&world, &res).expect("context");
    assert_eq!(read(&context, &res, "inventory_food"), 0);
    assert_eq!(read(&context, &res, "stock_in_hand"), 5);
}
