//! Availability must equal the original ranked search under live invalidation.
use super::*;
use sim_core::colony::food::{FoodAge, PotStore};
use sim_core::colony::{Carry, ItemSpec, ItemStack};

// Frozen original boolean path: nearest is deliberately retained as the oracle.
fn reference(
    index: &FoodChoiceIndex,
    res: &SimResources,
    origin: Tile,
    agent: u64,
    carry: Option<Carry>,
) -> bool {
    if carry.is_some_and(|carry| {
        let owner = FoodOwner::Carried(agent);
        carry.count > 0
            && carry.kind.definition().food.as_ref().is_some_and(|food| {
                food.raw_edible
                    && res.food.ages.get(&owner).is_some_and(|age| {
                        age.born_day <= crate::runtime::food::day(res)
                            && age.remaining(
                                crate::runtime::food::day(res),
                                food.lifetime_days(crate::runtime::food::storage::class(
                                    res, owner,
                                )),
                            ) > 0
                    })
            })
    }) {
        return true;
    }
    index.kinds().any(|kind| {
        index
            .nearest(res, kind, origin, u32::MAX, |_, tile| {
                res.nav.reachable(origin, tile)
            })
            .is_some()
    })
}

fn spec(name: &str) -> ItemSpec {
    ItemSpec::raw(ItemKind::from_name(name).unwrap(), None)
}

fn scene() -> SimResources {
    let mut res = SimResources::new(40, 12, sim_core::MaterialRegistry::new());
    res.rules = sim_core::rules::WorldRules::colony(0);
    res.nav.install(sim_core::nav::CostMap::new(40, 12));
    res
}

fn check(index: &FoodChoiceIndex, res: &SimResources) {
    // Includes separate chunks, blocked origins, and positions outside the map.
    for y in [0, 2, 11, 12] {
        for x in 0..=40 {
            for agent in [7, 8] {
                for carry in [
                    None,
                    Some(Carry::of(spec("roots"), 1)),
                    Some(Carry::of(spec("roots"), 0)),
                    Some(Carry::of(spec("wild_grain"), 1)),
                ] {
                    assert_eq!(
                        index.available_to(res, (x, y), agent, carry),
                        reference(index, res, (x, y), agent, carry),
                        "origin=({x},{y}) agent={agent} carry={carry:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn availability_matches_nearest_presence_across_live_ground_and_navigation_changes() {
    let mut res = scene();
    let mut index = FoodChoiceIndex::default();
    index.refresh(&res);
    check(&index, &res);
    for (tile, name) in [
        ((2, 2), "roots"),
        ((35, 2), "berries"),
        ((5, 2), "wild_grain"),
    ] {
        super::super::items::create_ground(
            &mut res,
            tile,
            spec(name),
            2,
            0,
            super::super::items::FoodSource::Harvest,
        )
        .unwrap();
    }
    index.refresh(&res);
    res.food
        .ages
        .insert(FoodOwner::Carried(7), FoodAge { born_day: 0 });
    check(&index, &res);
    for y in 0..12 {
        res.nav.set_cost((20, y), 0);
    }
    check(&index, &res);
    // No refresh: an existing decision phase must consult live stack/age data.
    res.colony.ground_items.remove(&(2, 2));
    check(&index, &res);
    assert!(!index.available_to(&res, (2, 2), 8, None));
    res.colony.ground_items.get_mut(&(35, 2)).unwrap().count = 0;
    check(&index, &res);
    res.colony
        .ground_items
        .insert((35, 2), ItemStack::of(spec("roots"), 1));
    check(&index, &res); // Different kind at an old indexed owner is not admitted.
    assert!(!index.available_to(&res, (35, 2), 8, None));
    res.colony
        .ground_items
        .insert((35, 2), ItemStack::of(spec("berries"), 1));
    res.food
        .ages
        .insert(FoodOwner::Ground((35, 2)), FoodAge { born_day: 1 });
    check(&index, &res);
    res.food.ages.remove(&FoodOwner::Ground((35, 2)));
    check(&index, &res);
    res.food
        .ages
        .insert(FoodOwner::Ground((35, 2)), FoodAge { born_day: 0 });
    res.current_tick = res.ticks_per_day * 100;
    check(&index, &res);
    assert!(!index.available_to(&res, (35, 2), 7, Some(Carry::of(spec("roots"), 1))));
}

#[test]
fn availability_preserves_pot_keys_and_adjacent_impassable_goal_semantics() {
    let mut res = scene();
    let roots = spec("roots");
    let owner = FoodOwner::Pot {
        id: 1,
        key: FoodStackKey::from(roots),
    };
    res.food.pots.insert(
        1,
        PotStore {
            tile: (2, 2),
            vessel: spec("clay_pot"),
            contents: vec![ItemStack::of(roots, 1)],
        },
    );
    res.food.ages.insert(owner, FoodAge { born_day: 0 });
    let mut index = FoodChoiceIndex::default();
    index.refresh(&res);
    res.nav.set_cost((2, 2), 0);
    // Existing availability permits reaching an impassable goal's neighbour.
    assert!(index.available_to(&res, (3, 2), 8, None));
    check(&index, &res);
    res.food.pots.get_mut(&1).unwrap().contents.clear();
    check(&index, &res);
    assert!(!index.available_to(&res, (3, 2), 8, None));
}
