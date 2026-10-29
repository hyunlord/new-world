//! Regional candidate indexes preserve the unrestricted linear nearest choice.
use super::{
    context::TickContext,
    harvest_food::{self, PlantCache, PlantCandidates},
};
use sim_core::colony::{content::installed_or_bundled, food::PlantState, ReserveKey};
use sim_core::components::{Agent, AgentState, Hunger, Inventory, Position};
use sim_core::nav::{CostMap, COST_PLAIN, IMPASSABLE};
use sim_core::rules::WorldRules;
use sim_engine::SimResources;

type Tile = (u32, u32);

fn fixture(origin: Tile, tiles: &[Tile]) -> (hecs::World, SimResources) {
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.nav.install(CostMap::new(8, 8));
    for y in 0..8 {
        res.nav.set_cost((3, y), IMPASSABLE);
    }
    let species = installed_or_bundled().plant_by_name("wild_root").unwrap();
    for &tile in tiles {
        res.food.plants.insert(
            tile,
            PlantState {
                species,
                units_left: 5,
                next_regrow_day: 0,
            },
        );
    }
    let mut world = hecs::World::new();
    world.spawn((
        Agent { id: 7 },
        Position::new(origin.0, origin.1),
        AgentState::Idle,
        Hunger::new(80.0, 0.05),
        Inventory::default(),
    ));
    (world, res)
}

fn nearest(index: &PlantCandidates, world: &hecs::World, res: &SimResources) -> Option<Tile> {
    let context = TickContext::capture(world, res).expect("context");
    index
        .nearest(context.row(7).expect("person"), res)
        .map(|target| target.tile)
}

fn linear(origin: Tile, res: &SimResources) -> Option<Tile> {
    res.food
        .plants
        .keys()
        .copied()
        .filter(|&tile| harvest_food::eligible(res, tile) && res.nav.reachable(origin, tile))
        .min_by_key(|&tile| (sim_core::spatial::chebyshev(origin, tile), tile.1, tile.0))
}

#[test]
fn disconnected_region_repeated_queries_visit_no_plant_candidates() {
    let (world, mut res) = fixture((1, 2), &[(5, 1), (5, 2), (6, 4)]);
    let mut cache = PlantCache::default();
    let original = cache.snapshot(&res);
    assert_eq!(original.len(), 3);
    for tick in 1..=5 {
        res.current_tick = tick;
        let index = cache.snapshot(&res);
        assert!(std::sync::Arc::ptr_eq(&original, &index));
        assert_eq!(nearest(&index, &world, &res), linear((1, 2), &res));
        assert_eq!(index.visits(), 0, "unreachable regions are never searched");
    }
}

#[test]
fn blocked_goal_is_indexed_for_both_neighbouring_regions() {
    for origin in [(2, 2), (4, 2)] {
        let (world, res) = fixture(origin, &[(3, 2)]);
        assert_eq!(res.nav.goal_regions((3, 2)).len(), 2);
        let index = PlantCandidates::capture(&res);
        assert_eq!(nearest(&index, &world, &res), Some((3, 2)));
        assert_eq!(nearest(&index, &world, &res), linear(origin, &res));
    }
    let (world, res) = fixture((3, 1), &[(3, 2)]);
    let index = PlantCandidates::capture(&res);
    assert_eq!(
        nearest(&index, &world, &res),
        None,
        "impassable origin has no region"
    );
    assert_eq!(index.visits(), 0);
}

#[test]
fn regional_nearest_keeps_ties_and_live_reservation_exclusions() {
    let origin = (1, 2);
    let (world, mut res) = fixture(origin, &[(2, 1), (0, 1), (1, 3), (3, 2), (5, 2)]);
    let index = PlantCandidates::capture(&res);
    assert_eq!(nearest(&index, &world, &res), Some((0, 1)));
    for tile in [(0, 1), (2, 1), (1, 3), (3, 2)] {
        assert_eq!(nearest(&index, &world, &res), linear(origin, &res));
        res.colony.reservations.insert(ReserveKey::Plant(tile), 9);
    }
    assert_eq!(nearest(&index, &world, &res), None);
    res.colony.reservations.remove(&ReserveKey::Plant((1, 3)));
    assert_eq!(
        nearest(&index, &world, &res),
        Some((1, 3)),
        "same-tick release stays live"
    );
}

#[test]
fn navigation_revision_repartitions_opened_and_closed_regions() {
    let origin = (1, 2);
    let (world, mut res) = fixture(origin, &[(5, 2)]);
    let mut cache = PlantCache::default();
    let blocked = cache.snapshot(&res);
    assert_eq!(nearest(&blocked, &world, &res), None);
    assert_eq!(blocked.visits(), 0);
    res.current_tick = 1;
    assert!(res.nav.set_cost((3, 2), COST_PLAIN));
    let opened = cache.snapshot(&res);
    assert!(!std::sync::Arc::ptr_eq(&blocked, &opened));
    assert_eq!(nearest(&opened, &world, &res), Some((5, 2)));
    assert_eq!(nearest(&opened, &world, &res), linear(origin, &res));
    res.current_tick = 2;
    assert!(res.nav.set_cost((3, 2), IMPASSABLE));
    let closed = cache.snapshot(&res);
    assert!(!std::sync::Arc::ptr_eq(&opened, &closed));
    assert_eq!(nearest(&closed, &world, &res), None);
    assert_eq!(closed.visits(), 0);
    let fresh = PlantCache::default().snapshot(&res);
    assert_eq!(
        nearest(&closed, &world, &res),
        nearest(&fresh, &world, &res)
    );
}
