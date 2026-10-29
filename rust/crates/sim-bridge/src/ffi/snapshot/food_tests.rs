//! Product snapshot food columns ignore obsolete colony counters and preserve baseline reads.
use super::*;
use crate::ffi::reads_settlement::collect_settlement_snapshot_with_resources;
use sim_core::colony::food::FoodOwner;
use sim_core::colony::{ItemKind, ItemSpec, Zone, ZoneKind};
use sim_systems::runtime::food::items::{self, FoodSource};
use std::collections::BTreeSet;

#[test]
fn colony_snapshot_uses_fresh_physical_carry_and_stores_despite_poisoned_counters() {
    let mut world = hecs::World::new();
    let mut res = SimResources::new(8, 8, MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.nav.install(sim_core::nav::CostMap::new(8, 8));
    let mut inventory = Inventory::default();
    inventory.add(ResourceKind::Food, 9);
    world.spawn((Agent { id: 7 }, Position::new(2, 2), inventory));
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (2, 2);
    settlement.add_member_agent(7);
    settlement.stockpile.insert(ResourceKind::Food, 999);
    res.settlements.insert(1, settlement);
    res.colony.zones.insert(
        1,
        Zone {
            id: 1,
            kind: ZoneKind::Stockpile,
            tiles: BTreeSet::from([(2, 2)]),
        },
    );
    let roots = ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None);
    items::create_ground(&mut res, (2, 2), roots, 7, 0, FoodSource::Harvest).unwrap();
    items::transfer(
        &mut world,
        &mut res,
        FoodOwner::Ground((2, 2)),
        FoodOwner::Carried(7),
        2,
        0,
    )
    .unwrap();
    let before = res.food.clone();
    assert_eq!(
        collect_agent_snapshot_with_resources(&world, &res)[0].carried_food,
        2
    );
    assert_eq!(
        collect_settlement_snapshot_with_resources(&world, &res)[0].food_stock,
        5
    );
    assert_eq!(res.food, before);
    assert_eq!(collect_agent_snapshot(&world)[0].carried_food, 9);
    res.current_tick = 6 * res.ticks_per_day;
    assert_eq!(
        collect_agent_snapshot_with_resources(&world, &res)[0].carried_food,
        0
    );
    assert_eq!(
        collect_settlement_snapshot_with_resources(&world, &res)[0].food_stock,
        5
    );
    res.current_tick = 10 * res.ticks_per_day;
    assert_eq!(
        collect_settlement_snapshot_with_resources(&world, &res)[0].food_stock,
        0
    );
    res.rules = WorldRules::baseline(0);
    assert_eq!(
        collect_agent_snapshot_with_resources(&world, &res)[0].carried_food,
        9
    );
    assert_eq!(
        collect_settlement_snapshot_with_resources(&world, &res)[0].food_stock,
        999
    );
}

#[test]
fn raw_grain_and_untracked_food_do_not_display_as_edible_carried_reserves() {
    let mut world = hecs::World::new();
    let mut res = SimResources::new(8, 8, MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    let entity = world.spawn((Agent { id: 7 }, Position::new(2, 2)));
    let grain = ItemSpec::raw(ItemKind::from_name("wild_grain").unwrap(), None);
    items::create_ground(&mut res, (2, 2), grain, 2, 0, FoodSource::Harvest).unwrap();
    items::transfer(
        &mut world,
        &mut res,
        FoodOwner::Ground((2, 2)),
        FoodOwner::Carried(7),
        2,
        0,
    )
    .unwrap();
    assert_eq!(
        collect_agent_snapshot_with_resources(&world, &res)[0].carried_food,
        0
    );
    let roots = ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None);
    world
        .insert_one(entity, sim_core::colony::Carry::of(roots, 2))
        .unwrap();
    res.food.ages.remove(&FoodOwner::Carried(7));
    assert_eq!(
        collect_agent_snapshot_with_resources(&world, &res)[0].carried_food,
        0
    );
}
