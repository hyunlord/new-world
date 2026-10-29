//! Colony observers and shared legacy paths cannot recreate abstract food or double-credit cargo.
use hecs::World;
use sim_core::colony::{ItemKind, ItemSpec, Zone, ZoneKind};
use sim_core::components::{
    Agent, AgentState, Inventory, PorterKind, PorterStage, PorterTrip, Position, ResourceKind,
    SeekTarget, Settlement, TargetKind, TripRecord,
};
use sim_core::rules::{NeighbourRule, WorldRules};
use sim_core::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimResources};
use sim_systems::runtime::{
    agent_actions::NeedAvailability, food::items, neighbours,
    settlement::relations::RelationsSystem,
};
use std::collections::BTreeSet;

fn scene() -> (World, SimResources) {
    let mut res = SimResources::new(12, 12, MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    sim_systems::runtime::nav::ensure(&mut res);
    (World::new(), res)
}

#[test]
fn colony_food_availability_ignores_legacy_counters_and_tracks_actual_fresh_items() {
    let (_, mut res) = scene();
    res.food_tiles.insert((2, 2), 255);
    assert!(!NeedAvailability::from_resources(&res).food);
    let roots = ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None);
    items::create_ground(&mut res, (2, 2), roots, 2, 0, items::FoodSource::Harvest).unwrap();
    assert!(NeedAvailability::from_resources(&res).food);
    res.current_tick = res.ticks_per_day * 6;
    assert!(!NeedAvailability::from_resources(&res).food);
}

#[test]
fn fresh_raw_grain_is_tradable_but_never_advertised_as_edible() {
    let (_, mut res) = scene();
    let grain = ItemSpec::raw(ItemKind::from_name("wild_grain").unwrap(), None);
    items::create_ground(&mut res, (2, 2), grain, 2, 0, items::FoodSource::Harvest).unwrap();
    let owner = sim_core::colony::food::FoodOwner::Ground((2, 2));
    assert!(sim_systems::runtime::food::choice::fresh(
        &res, owner, grain.kind
    ));
    assert!(!sim_systems::runtime::food::choice::edible(
        &res, owner, grain.kind
    ));
    res.current_tick = res.ticks_per_day * 12;
    assert!(!sim_systems::runtime::food::choice::fresh(
        &res, owner, grain.kind
    ));
}

#[test]
fn neighbour_surplus_counts_physical_storage_once_across_two_settlements() {
    let (mut world, mut res) = scene();
    world.spawn((Agent { id: 1 },));
    world.spawn((Agent { id: 2 },));
    for (id, tile) in [(1, (1, 1)), (2, (9, 9))] {
        let mut settlement = Settlement::new_with_id(id, 0);
        settlement.formation_tile = tile;
        settlement.store(ResourceKind::Food, 1000);
        res.settlements.insert(id, settlement);
    }
    let tile = (2, 2);
    res.colony.zones.insert(
        1,
        Zone {
            id: 1,
            kind: ZoneKind::Stockpile,
            tiles: BTreeSet::from([tile]),
        },
    );
    let roots = ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None);
    items::create_ground(&mut res, tile, roots, 12, 0, items::FoodSource::Harvest).unwrap();
    let rule = NeighbourRule::m5_5().unwrap();
    assert_eq!(
        neighbours::surplus_days(&world, &res, &rule),
        12 / (2 * rule.food_per_adult_day)
    );
}

#[test]
fn retired_food_porter_cannot_credit_a_delivery_after_its_cargo_was_migrated() {
    let (mut world, mut res) = scene();
    let trip = PorterTrip {
        from: 1,
        to: 2,
        kind: PorterKind::Food,
        amount: 5,
        stage: PorterStage::Outbound,
        depart_tick: 0,
    };
    let entity = world.spawn((
        Agent { id: 1 },
        Position { x: 2, y: 2 },
        Inventory::default(),
        trip,
        AgentState::Seeking {
            target: TargetKind::Agent(1),
        },
        SeekTarget { tile: (2, 2) },
    ));
    for id in [1, 2] {
        let mut settlement = Settlement::new_with_id(id, 0);
        settlement.formation_tile = (2, 2);
        res.settlements.insert(id, settlement);
    }
    res.settlement_relations.active.insert(
        1,
        TripRecord {
            porter: 1,
            trip,
            cause: 0,
            delivered: 0,
            returned: 0,
            lost: 0,
            failed: false,
            end_tick: None,
        },
    );
    let roots = ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None);
    items::create_ground(&mut res, (2, 2), roots, 5, 0, items::FoodSource::Migration).unwrap();
    RelationsSystem::new().tick(&mut world, &mut res);
    assert!(world.get::<&PorterTrip>(entity).is_err());
    assert!(res.settlement_relations.active.is_empty());
    assert!(res.settlement_relations.credited.is_empty());
    assert_eq!(res.tallies.porter_food_delivered, 0);
    assert_eq!(res.colony.ground_total(roots.kind), 5);
    assert_eq!(res.food.tallies.initial_units, 5);
    assert!(res
        .settlements
        .values()
        .all(|s| !s.stockpile.contains_key(&ResourceKind::Food)));
    assert_eq!(res.settlement_relations.trips[0].lost, 5);
}

#[test]
fn food_import_receives_age_and_a_full_map_trade_keeps_its_goods_and_payment() {
    use sim_core::colony::food::FoodOwner;
    use sim_core::colony::neighbours::{Caravan, Goods, VisitState};
    for full in [false, true] {
        let (world, mut res) = scene();
        let roots = ItemKind::from_name("roots").unwrap();
        let payment = ItemSpec::raw(ItemKind::role("tree_output"), None);
        items::create_ground(&mut res, (2, 2), payment, 20, 0, items::FoodSource::Harvest).unwrap();
        res.colony.zone_of_tile.insert((2, 2), 1);
        res.colony.neighbours.caravans.push(Caravan {
            id: 1,
            neighbour: 0,
            goods: vec![Goods {
                kind: roots,
                count: 2,
                quality: 50,
            }],
            pos: (3, 3),
            path: Vec::new(),
            state: VisitState::Waiting { since: 0 },
            day: 0,
            spot: Some((3, 3)),
            owed: Vec::new(),
            trades: 0,
        });
        if full {
            res.tile_grid
                .terrain_type
                .fill(sim_core::TerrainType::Mountain);
        }
        let before = res.colony.neighbours.caravans[0].clone();
        let ledger = res.colony.item_ledger.clone();
        let result = neighbours::trade::trade(&mut res, 0, 1, &[((2, 2), 20)], &[(0, 2)]);
        if full {
            assert!(result.is_err());
            assert_eq!(res.colony.neighbours.caravans[0], before);
            assert_eq!(res.colony.item_ledger, ledger);
        } else {
            result.unwrap();
            assert_eq!(res.food.ages[&FoodOwner::Ground((3, 3))].born_day, 0);
            assert_eq!(res.food.tallies.traded_in_units, 2);
            sim_systems::runtime::food::validate(&world, &res).unwrap();
        }
    }
}
