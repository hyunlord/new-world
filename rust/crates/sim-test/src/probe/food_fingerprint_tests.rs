//! Regressions for hidden food ownership and baseline fingerprint isolation.
use super::*;
use sim_core::colony::food::{FoodAge, FoodOwner, PotStore};
use sim_core::colony::{BlueprintKind, ItemKind, ItemSpec, ItemStack, Workshop};
use sim_core::components::Position;
use sim_core::rooms::{Room, RoomRole};
use sim_core::rules::WorldRules;
use sim_core::MaterialRegistry;
use std::collections::BTreeSet;

fn food() -> ItemSpec {
    ItemSpec::raw(ItemKind::from_name("berries").expect("bundled food"), None)
}
fn fixture() -> SimEngine {
    let mut engine = SimEngine::new(4, 4, MaterialRegistry::new());
    engine.resources.rules = WorldRules::colony(0);
    engine
        .resources
        .colony
        .ground_items
        .insert((1, 1), ItemStack::of(food(), 3));
    engine.resources.colony.workshops.insert(
        1,
        Workshop {
            id: 1,
            kind: BlueprintKind::WoodBench,
            top_left: (0, 0),
            size: (1, 1),
            bills: vec![],
            shelf: vec![ItemStack::of(food(), 2)],
            active: None,
        },
    );
    engine
        .resources
        .food
        .ages
        .insert(FoodOwner::Ground((1, 1)), FoodAge { born_day: 0 });
    engine.resources.food.pots.insert(
        1,
        PotStore {
            tile: (2, 2),
            vessel: ItemSpec::raw(ItemKind::from_name("clay_pot").expect("pot"), None),
            contents: vec![ItemStack::of(food(), 1)],
        },
    );
    engine
        .world
        .spawn((Agent { id: 1 }, Position::new(0, 0), Carry::of(food(), 2)));
    engine
}
fn hashes(engine: &SimEngine) -> (String, String) {
    (
        crate::probe::fingerprint(engine),
        crate::probe::colony::colony_fingerprint(engine),
    )
}
fn changed(edit: impl FnOnce(&mut SimEngine)) {
    let mut engine = fixture();
    let before = hashes(&engine);
    edit(&mut engine);
    let after = hashes(&engine);
    assert_ne!(before.0, after.0);
    assert_ne!(before.1, after.1);
}

#[test]
fn baseline_stream_is_untouched_even_with_accidentally_populated_sidecar() {
    let mut engine = fixture();
    engine.resources.rules.colony = None;
    let mut hash = 0x1234_5678;
    append(&mut hash, &engine).expect("baseline bypass");
    assert_eq!(hash, 0x1234_5678);
    let before = crate::probe::fingerprint(&engine);
    engine.resources.food.tallies.meals = 999;
    engine.resources.food.ages.clear();
    engine.resources.food.pots.clear();
    assert_eq!(crate::probe::fingerprint(&engine), before);
}

#[test]
fn full_sidecar_and_each_physical_owner_change_both_probe_fingerprints() {
    changed(|engine| {
        engine
            .resources
            .food
            .ages
            .get_mut(&FoodOwner::Ground((1, 1)))
            .expect("age")
            .born_day = 2
    });
    changed(|engine| engine.resources.food.next_pot_id += 1);
    changed(|engine| engine.resources.food.firsts.last_plan_day = Some(3));
    changed(|engine| engine.resources.food.last_daily_day = Some(1));
    changed(|engine| engine.resources.food.tallies.wasted_tenths += 1);
    changed(|engine| engine.resources.food.version += 1);
    changed(|engine| {
        engine
            .resources
            .food
            .pots
            .get_mut(&1)
            .expect("pot")
            .contents[0]
            .count += 1
    });
    changed(|engine| {
        engine
            .resources
            .colony
            .ground_items
            .get_mut(&(1, 1))
            .expect("ground")
            .count += 1
    });
    changed(|engine| {
        engine
            .resources
            .colony
            .workshops
            .get_mut(&1)
            .expect("shelf")
            .shelf[0]
            .count += 1
    });
    changed(|engine| {
        for (_, carry) in engine.world.query::<&mut Carry>().iter() {
            carry.count += 1;
        }
    });
    changed(|engine| engine.resources.colony.item_ledger.create(food().kind, 1));
    changed(|engine| {
        engine.resources.colony.rooms.insert(
            1,
            Room {
                id: 1,
                tiles: BTreeSet::from([(1, 1)]),
                doors: BTreeSet::new(),
                roofed: true,
                role: RoomRole::Storehouse,
                dug: false,
            },
        );
    });
}

#[test]
fn ecs_and_map_insertion_order_do_not_change_food_stream() {
    let mut left = fixture();
    let mut right = fixture();
    for (engine, ids) in [(&mut left, [2, 3]), (&mut right, [3, 2])] {
        for id in ids {
            engine
                .world
                .spawn((Agent { id }, Position::new(0, 0), Carry::of(food(), 1)));
            engine
                .resources
                .food
                .ages
                .insert(FoodOwner::Carried(id), FoodAge { born_day: 0 });
        }
    }
    assert_eq!(hashes(&left), hashes(&right));
}

#[test]
fn malformed_carrier_cannot_silently_disappear_from_the_food_digest() {
    let mut engine = fixture();
    engine.world.spawn((Carry::of(food(), 1),));
    assert!(append(&mut 0, &engine).is_err());
    let mut engine = fixture();
    engine.world.spawn((Agent { id: 1 }, Carry::of(food(), 1)));
    assert!(append(&mut 0, &engine).is_err());
}
