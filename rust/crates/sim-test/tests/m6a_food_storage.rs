//! Food ownership regression tests: no reset, no failed-move loss, real storage and exact intake.
use hecs::World;
use sim_core::colony::food::{FoodOwner, FoodStackKey};
use sim_core::colony::{ItemKind, ItemSpec, Zone, ZoneKind};
use sim_core::components::{Agent, Hunger, Lineage};
use sim_core::rules::WorldRules;
use sim_core::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimResources};
use sim_systems::runtime::food::{self, items, meals, storage};
use std::collections::BTreeSet;

fn scene() -> (World, SimResources) {
    let mut res = SimResources::new(8, 8, MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    (World::new(), res)
}
fn spec(name: &str) -> ItemSpec {
    ItemSpec::raw(ItemKind::from_name(name).expect("test content"), None)
}
fn create(res: &mut SimResources, tile: (u32, u32), name: &str, count: u16, day: u32) {
    items::create_ground(
        res,
        tile,
        spec(name),
        count,
        day,
        items::FoodSource::Harvest,
    )
    .unwrap();
}
fn stockpile(res: &mut SimResources, tile: (u32, u32)) {
    res.colony.zones.insert(
        1,
        Zone {
            id: 1,
            kind: ZoneKind::Stockpile,
            tiles: BTreeSet::from([tile]),
        },
    );
    res.colony.touch(false);
}

#[test]
fn splitting_transferring_and_merging_preserve_the_oldest_birth_day() {
    let (mut world, mut res) = scene();
    create(&mut res, (2, 2), "roots", 5, 0);
    create(&mut res, (3, 2), "roots", 2, 2);
    let from = FoodOwner::Ground((2, 2));
    let to = FoodOwner::Ground((3, 2));
    items::transfer(&mut world, &mut res, from, to, 3, 2).unwrap();
    assert_eq!(res.food.ages[&from].born_day, 0);
    assert_eq!(res.food.ages[&to].born_day, 0);
    assert_eq!(res.colony.ground_items[&(2, 2)].count, 2);
    assert_eq!(res.colony.ground_items[&(3, 2)].count, 5);
    res.current_tick = res.ticks_per_day * 2;
    food::validate(&world, &res).unwrap();
}

#[test]
fn incompatible_transfer_leaves_source_quantity_and_age_unchanged() {
    let (mut world, mut res) = scene();
    create(&mut res, (2, 2), "roots", 3, 0);
    create(&mut res, (3, 2), "berries", 2, 0);
    let before = res.food.clone();
    let ground = res.colony.ground_items.clone();
    assert_eq!(
        items::transfer(
            &mut world,
            &mut res,
            FoodOwner::Ground((2, 2)),
            FoodOwner::Ground((3, 2)),
            2,
            0
        ),
        Err(items::FoodError::NoCapacity)
    );
    assert_eq!(res.food, before);
    assert_eq!(res.colony.ground_items, ground);
}

#[test]
fn storage_requires_a_real_zone_and_a_real_roofed_storehouse() {
    let (world, mut res) = scene();
    let tile = (2, 2);
    let owner = FoodOwner::Ground(tile);
    create(&mut res, tile, "roots", 2, 0);
    assert_eq!(storage::remaining_days(&world, &res, owner, 0).unwrap(), 6);
    stockpile(&mut res, tile);
    assert_eq!(storage::remaining_days(&world, &res, owner, 0).unwrap(), 10);
    res.colony.rooms.insert(
        1,
        sim_core::rooms::Room {
            id: 1,
            tiles: BTreeSet::from([tile]),
            doors: BTreeSet::new(),
            roofed: true,
            role: sim_core::rooms::RoomRole::Bedroom,
            dug: false,
        },
    );
    assert_eq!(storage::remaining_days(&world, &res, owner, 0).unwrap(), 10);
    res.colony.rooms.get_mut(&1).unwrap().role = sim_core::rooms::RoomRole::Storehouse;
    assert_eq!(storage::remaining_days(&world, &res, owner, 0).unwrap(), 14);
}

#[test]
fn a_pot_has_finite_capacity_and_spills_its_real_vessel_and_contents() {
    let (mut world, mut res) = scene();
    create(&mut res, (2, 2), "clay_pot", 1, 0);
    create(&mut res, (3, 2), "roots", 15, 0);
    let id =
        storage::deploy_pot(&mut world, &mut res, FoodOwner::Ground((2, 2)), (2, 2), 0).unwrap();
    let owner = FoodOwner::Pot {
        id,
        key: FoodStackKey::from(spec("roots")),
    };
    items::transfer(
        &mut world,
        &mut res,
        FoodOwner::Ground((3, 2)),
        owner,
        12,
        0,
    )
    .unwrap();
    assert_eq!(storage::remaining_days(&world, &res, owner, 0).unwrap(), 18);
    assert_eq!(
        items::transfer(&mut world, &mut res, FoodOwner::Ground((3, 2)), owner, 1, 0),
        Err(items::FoodError::NoCapacity)
    );
    storage::remove_pot(&mut world, &mut res, id, 1).unwrap();
    assert!(res.food.pots.is_empty());
    assert_eq!(res.colony.ground_total(spec("roots").kind), 15);
    assert_eq!(res.colony.ground_total(spec("clay_pot").kind), 1);
    assert!(res.food.ages.values().all(|age| age.born_day == 0));
    food::validate(&world, &res).unwrap();
}

#[test]
fn expired_food_cannot_be_saved_by_transfer_and_spoils_exactly_once() {
    let (mut world, mut res) = scene();
    create(&mut res, (2, 2), "berries", 4, 0);
    stockpile(&mut res, (3, 2));
    assert_eq!(
        items::transfer(
            &mut world,
            &mut res,
            FoodOwner::Ground((2, 2)),
            FoodOwner::Ground((3, 2)),
            4,
            2
        ),
        Err(items::FoodError::Expired)
    );
    assert_eq!(storage::spoil_day(&mut world, &mut res, 2).unwrap(), 4);
    assert_eq!(storage::spoil_day(&mut world, &mut res, 2).unwrap(), 0);
    assert_eq!(res.food.tallies.spoiled_units, 4);
    food::validate(&world, &res).unwrap();
}

#[test]
fn an_actual_meal_consumes_one_item_and_records_only_its_ingredient_groups() {
    let (mut world, mut res) = scene();
    let entity = world.spawn((
        Agent { id: 1 },
        Hunger::new(50.0, 0.05),
        Lineage::founder(0, 0),
    ));
    create(&mut res, (2, 2), "roasted_roots", 3, 0);
    let outcome =
        meals::consume(&mut world, &mut res, entity, FoodOwner::Ground((2, 2)), 0).unwrap();
    assert!(outcome.cooked);
    assert_eq!(outcome.units, 1);
    assert_eq!(world.get::<&Hunger>(entity).unwrap().value, 20.0);
    assert_eq!(
        res.food.meals[&1].groups_in(0, res.rules.calendar.days_to_ticks(3)),
        1
    );
    assert_eq!(res.food.tallies.meals, 1);
    assert_eq!(res.food.tallies.cooked_meals, 1);
    assert_eq!(res.colony.ground_items[&(2, 2)].count, 2);
    food::validate(&world, &res).unwrap();
}

#[test]
fn raw_inedible_grain_never_relieves_hunger_or_counts_as_a_meal() {
    let (mut world, mut res) = scene();
    let entity = world.spawn((Agent { id: 1 }, Hunger::new(50.0, 0.05)));
    create(&mut res, (2, 2), "wild_grain", 3, 0);
    assert_eq!(
        meals::consume(&mut world, &mut res, entity, FoodOwner::Ground((2, 2)), 0),
        Err(items::FoodError::Inedible)
    );
    assert_eq!(res.food.tallies.meals, 0);
    assert_eq!(world.get::<&Hunger>(entity).unwrap().value, 50.0);
    food::validate(&world, &res).unwrap();
}

#[test]
fn a_missing_birth_record_is_an_invariant_failure_not_fresh_food() {
    let (mut world, mut res) = scene();
    create(&mut res, (2, 2), "roots", 2, 0);
    res.food.ages.clear();
    assert!(food::validate(&world, &res)
        .unwrap_err()
        .contains("owner/age mismatch"));
    assert_eq!(
        items::take(&mut world, &mut res, FoodOwner::Ground((2, 2)), 1, 0),
        Err(items::FoodError::MissingAge)
    );
    assert_eq!(res.colony.ground_items[&(2, 2)].count, 2);
}

#[test]
fn maintenance_is_scheduled_on_the_world_calendar_boundary() {
    let (_, res) = scene();
    let system = food::FoodDailySystem::new(&res);
    assert_eq!(system.tick_interval(), res.ticks_per_day);
    assert_eq!(
        (system.tick_offset(&res) + res.rules.clock_offset_ticks) % res.ticks_per_day,
        0
    );
    assert_eq!(system.priority(), 119);
}

#[test]
fn a_full_map_drop_preserves_the_carrier_and_all_age_metadata() {
    let (mut world, mut res) = scene();
    let entity = world.spawn((Agent { id: 4 },));
    create(&mut res, (2, 2), "roots", 3, 0);
    items::transfer(
        &mut world,
        &mut res,
        FoodOwner::Ground((2, 2)),
        FoodOwner::Carried(4),
        3,
        0,
    )
    .unwrap();
    res.tile_grid
        .terrain_type
        .fill(sim_core::TerrainType::Mountain);
    let before = res.food.clone();
    assert_eq!(
        items::drop_ground(&mut world, &mut res, FoodOwner::Carried(4), (2, 2), 3, 0),
        Err(items::FoodError::NoCapacity)
    );
    assert_eq!(
        world.get::<&sim_core::colony::Carry>(entity).unwrap().count,
        3
    );
    assert_eq!(res.food, before);
    food::validate(&world, &res).unwrap();
}

#[test]
fn an_expired_carried_stack_is_booked_when_death_drops_it() {
    let (mut world, mut res) = scene();
    let entity = world.spawn((Agent { id: 4 },));
    create(&mut res, (2, 2), "berries", 3, 0);
    items::transfer(
        &mut world,
        &mut res,
        FoodOwner::Ground((2, 2)),
        FoodOwner::Carried(4),
        3,
        0,
    )
    .unwrap();
    assert_eq!(
        items::drop_ground(&mut world, &mut res, FoodOwner::Carried(4), (2, 2), 3, 2).unwrap(),
        3
    );
    assert!(world.get::<&sim_core::colony::Carry>(entity).is_err());
    assert_eq!(res.food.tallies.spoiled_units, 3);
    food::validate(&world, &res).unwrap();
}

#[test]
fn deployed_pot_items_are_counted_once_by_the_existing_item_ledger() {
    let (mut world, mut res) = scene();
    create(&mut res, (2, 2), "clay_pot", 1, 0);
    create(&mut res, (3, 2), "roots", 4, 0);
    let id =
        storage::deploy_pot(&mut world, &mut res, FoodOwner::Ground((2, 2)), (2, 2), 0).unwrap();
    items::transfer(
        &mut world,
        &mut res,
        FoodOwner::Ground((3, 2)),
        FoodOwner::Pot {
            id,
            key: FoodStackKey::from(spec("roots")),
        },
        4,
        0,
    )
    .unwrap();
    let pots =
        sim_systems::runtime::jobs::invariants::ledger_of(&world, &res, spec("clay_pot").kind);
    let roots = sim_systems::runtime::jobs::invariants::ledger_of(&world, &res, spec("roots").kind);
    assert_eq!((pots.ground, pots.held), (0, 1));
    assert_eq!((roots.ground, roots.shelved), (0, 4));
    assert!(pots.balanced() && roots.balanced());
    sim_systems::runtime::jobs::invariants::check(&world, &res).unwrap();
    storage::remove_pot(&mut world, &mut res, id, 0).unwrap();
    sim_systems::runtime::jobs::invariants::check(&world, &res).unwrap();
}

#[test]
fn diet_mood_uses_only_actual_groups_with_a_three_day_expiration() {
    use sim_core::colony::people::Modifier;
    let (mut world, mut res) = scene();
    let entity = world.spawn((Agent { id: 1 }, Hunger::new(100.0, 0.05)));
    create(&mut res, (2, 2), "roots", 1, 0);
    meals::consume(&mut world, &mut res, entity, FoodOwner::Ground((2, 2)), 0).unwrap();
    let facts = sim_systems::runtime::mood::facts_of(&world, &res, entity);
    assert_eq!(facts.food_groups, 1);
    assert!(sim_systems::runtime::mood::modifiers(&facts)
        .iter()
        .any(|(kind, _)| *kind == Modifier::FoodMonotonous));
    create(&mut res, (2, 2), "fruit_porridge", 1, 0);
    meals::consume(&mut world, &mut res, entity, FoodOwner::Ground((2, 2)), 0).unwrap();
    let facts = sim_systems::runtime::mood::facts_of(&world, &res, entity);
    assert_eq!(facts.food_groups, 3);
    assert!(sim_systems::runtime::mood::modifiers(&facts).contains(&(Modifier::FoodVaried, 60)));
    res.current_tick = res.rules.calendar.days_to_ticks(3);
    let facts = sim_systems::runtime::mood::facts_of(&world, &res, entity);
    assert_eq!(facts.food_groups, 0);
    assert!(!sim_systems::runtime::mood::modifiers(&facts)
        .iter()
        .any(|(kind, _)| matches!(kind, Modifier::FoodVaried | Modifier::FoodMonotonous)));
    assert_eq!(
        (Modifier::FoodVaried.code(), Modifier::FoodMonotonous.code()),
        (14, 15)
    );
}

#[test]
fn food_milestones_require_real_intake_and_real_preserved_storage() {
    use sim_core::chronicle::ChronicleKind;
    let (mut world, mut res) = scene();
    let entity = world.spawn((Agent { id: 1 }, Hunger::new(100.0, 0.05)));
    create(&mut res, (2, 2), "roasted_roots", 2, 0);
    assert!(res.food.firsts.cooked_meal.is_none());
    meals::consume(&mut world, &mut res, entity, FoodOwner::Ground((2, 2)), 0).unwrap();
    meals::consume(&mut world, &mut res, entity, FoodOwner::Ground((2, 2)), 0).unwrap();
    assert_eq!(
        res.chronicle
            .entries()
            .iter()
            .filter(|e| e.kind == ChronicleKind::FirstCookedMeal)
            .count(),
        1
    );
    create(&mut res, (2, 2), "dried_berries", 2, 0);
    assert!(res.food.firsts.dried_store.is_none());
    stockpile(&mut res, (3, 2));
    items::transfer(
        &mut world,
        &mut res,
        FoodOwner::Ground((2, 2)),
        FoodOwner::Ground((3, 2)),
        2,
        0,
    )
    .unwrap();
    assert_eq!(
        res.chronicle
            .entries()
            .iter()
            .filter(|e| e.kind == ChronicleKind::FirstDriedStore)
            .count(),
        1
    );
}
