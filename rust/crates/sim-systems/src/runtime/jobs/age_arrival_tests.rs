//! Owner-level age allocation keeps necessary merges and storage bounds intact.
use super::age_haul_tests::{cell, put, scene};
use crate::runtime::food::{items as food, storage};
use sim_core::colony::{
    food::{FoodOwner, FoodStackKey},
    Carry, ItemKind, ItemSpec, JobKind, ReserveKey,
};
use sim_core::components::Agent;

#[test]
fn clean_pot_beats_old_ground_and_required_merge_keeps_oldest_age() {
    for clean in [true, false] {
        let (mut world, mut res, spec) = scene();
        let tile = (5, 5);
        cell(&mut res, tile);
        let vessel = ItemSpec::raw(ItemKind::from_name("clay_pot").unwrap(), None);
        food::create_ground(&mut res, tile, vessel, 1, 0, food::FoodSource::Debug).unwrap();
        let pot =
            storage::deploy_pot(&mut world, &mut res, FoodOwner::Ground(tile), tile, 1).unwrap();
        let owner = FoodOwner::Pot {
            id: pot,
            key: FoodStackKey::from(spec),
        };
        if !clean {
            put(&mut res, (1, 1), spec, 0);
            food::transfer(&mut world, &mut res, FoodOwner::Ground((1, 1)), owner, 2, 1).unwrap();
        }
        put(&mut res, tile, spec, 0);
        put(&mut res, (2, 2), spec, 1);
        let entity = world.spawn((Agent { id: 7 },));
        food::transfer(
            &mut world,
            &mut res,
            FoodOwner::Ground((2, 2)),
            FoodOwner::Carried(7),
            2,
            1,
        )
        .unwrap();
        let carry = *world.get::<&Carry>(entity).unwrap();
        super::super::food_transfers::arrive(
            &mut world,
            &mut res,
            entity,
            JobKind::Haul {
                from: (2, 2),
                to: tile,
            },
            tile,
            carry,
        );
        assert_eq!(res.colony.ground_items[&tile].count, 2);
        assert_eq!(
            food::stack(&world, &res, owner).unwrap().count,
            if clean { 2 } else { 4 }
        );
        assert_eq!(res.food.ages[&owner].born_day, if clean { 1 } else { 0 });
        assert!(world.get::<&Carry>(entity).is_err());
        super::super::invariants::check(&world, &res).expect("owner conservation");
    }
}

#[test]
fn clean_capacity_respects_reservations_regions_and_missing_ages() {
    let (_, mut res, spec) = scene();
    put(&mut res, (3, 2), spec, 0);
    cell(&mut res, (3, 2));
    cell(&mut res, (7, 2));
    res.spatial.colony.refresh(&res.colony);
    let region = res.nav.region_of((2, 2));
    res.colony.reservations.insert(ReserveKey::Cell((7, 2)), 99);
    assert_eq!(
        super::super::food_storage::destination(&res, region, (2, 2), spec, 1, false)
            .unwrap()
            .0,
        (3, 2)
    );
    res.food.ages.remove(&FoodOwner::Ground((3, 2)));
    assert!(
        super::super::food_storage::destination(&res, region, (2, 2), spec, 1, false).is_none()
    );
    res.colony.reservations.clear();
    for y in 0..12 {
        res.nav.set_cost((6, y), 0);
    }
    let region = res.nav.region_of((2, 2));
    assert!(
        super::super::food_storage::destination(&res, region, (2, 2), spec, 1, false).is_none()
    );
}
