//! Stored ingredients must reach cooking through actual reserved supply work.
use crate::runtime::food::{self, items, storage};
use hecs::World;
use sim_core::colony::food::{FoodOwner, FoodStackKey};
use sim_core::colony::{
    Bill, BillMode, ItemKind, ItemSpec, Job, JobKind, JobStage, RecipeKind, Workshop,
};
use sim_core::components::{Agent, AgentState, Position, SeekTarget, Settlement, TargetKind};
use sim_core::rules::WorldRules;
use sim_engine::{FireState, RuntimeSystem, SimResources};

fn fixture() -> (World, SimResources, hecs::Entity, FoodOwner) {
    let mut world = World::new();
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.colony.as_mut().expect("colony").knowledge = None;
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.current_tick = res.ticks_per_day;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (4, 4);
    res.settlements.insert(1, settlement);
    res.fires.insert(1, FireState { lit: true });
    let recipe = RecipeKind::from_name("porridge").expect("recipe");
    res.colony.workshops.insert(
        1,
        Workshop {
            id: 1,
            kind: recipe.recipe().expect("definition").workshop,
            top_left: (4, 4),
            size: (1, 1),
            bills: vec![Bill {
                id: 1,
                recipe,
                mode: BillMode::Make(1),
                done: 0,
                suspended: false,
                auto: false,
            }],
            shelf: vec![],
            active: None,
        },
    );
    let vessel = ItemSpec::raw(ItemKind::from_name("clay_pot").expect("pot"), None);
    items::create_ground(&mut res, (4, 5), vessel, 1, 0, items::FoodSource::Debug).expect("vessel");
    let pot = storage::deploy_pot(&mut world, &mut res, FoodOwner::Ground((4, 5)), (4, 5), 1)
        .expect("deploy");
    let grain = ItemSpec::raw(ItemKind::from_name("wild_grain").expect("grain"), None);
    items::create_ground(&mut res, (2, 2), grain, 2, 0, items::FoodSource::Harvest).expect("grain");
    let owner = FoodOwner::Pot {
        id: pot,
        key: FoodStackKey::from(grain),
    };
    items::transfer(&mut world, &mut res, FoodOwner::Ground((2, 2)), owner, 2, 1).expect("store");
    let person = world.spawn((
        Agent { id: 7 },
        Position::new(1, 1),
        AgentState::Idle,
        crate::runtime::agent::MovementRng::new(7),
    ));
    (world, res, person, owner)
}

fn hold_supply(world: &mut World, res: &mut SimResources, person: hecs::Entity) -> Job {
    let job = Job {
        kind: JobKind::Supply {
            from: (4, 5),
            workshop: 1,
            to: (4, 5),
        },
        stage: JobStage::ToSource,
        taken_at: res.current_tick,
        amount: 2,
        item: ItemKind::from_name("wild_grain").expect("grain"),
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: res.current_tick,
    };
    world
        .insert(
            person,
            (
                job,
                Position::new(4, 5),
                AgentState::Seeking {
                    target: TargetKind::Job,
                },
                SeekTarget::new((4, 5)),
            ),
        )
        .expect("supply worker");
    for key in job.keys() {
        super::cancel::reserve(res, key, 7);
    }
    job
}

#[test]
fn pot_only_grain_is_supplied_by_worker_and_cooked_once() {
    let (mut world, mut res, person, owner) = fixture();
    let grain = ItemKind::from_name("wild_grain").expect("grain");
    assert_eq!(super::bills::available(&res, grain), 2);
    super::bills::schedule(&mut world, &mut res);
    assert_eq!(res.colony.workshops[&1].active, Some(1));
    let mut assign = super::JobAssignSystem::new();
    let mut execute = super::JobExecuteSystem::new();
    let mut movement = crate::runtime::agent::AgentMovementSystem::new();
    assign.tick(&mut world, &mut res);
    assert!(matches!(
        world.get::<&Job>(person).expect("supply job").kind,
        JobKind::Supply {
            from: (4, 5),
            workshop: 1,
            ..
        }
    ));
    let start = res.current_tick;
    let mut saw_dated_carry_or_shelf = false;
    for tick in start..start + 400 {
        res.current_tick = tick;
        movement.tick(&mut world, &mut res);
        super::bills::schedule(&mut world, &mut res);
        assign.tick(&mut world, &mut res);
        execute.tick(&mut world, &mut res);
        saw_dated_carry_or_shelf |= res.food.ages.iter().any(|(owner, age)| {
            matches!(
                owner,
                FoodOwner::Carried(7) | FoodOwner::Shelf { workshop: 1, .. }
            ) && age.born_day == 0
        });
        if res.colony.workshops[&1].bills[0].done == 1 {
            break;
        }
    }
    assert!(
        saw_dated_carry_or_shelf,
        "input age survives the real supply trip"
    );
    assert!(items::stack(&world, &res, owner).is_none());
    assert_eq!(res.colony.workshops[&1].bills[0].done, 1);
    assert_eq!(res.food.tallies.craft_input_units, 2);
    assert_eq!(res.food.tallies.craft_output_units, 3);
    food::validate(&world, &res).expect("food conservation");
    super::invariants::check(&world, &res).expect("item and reservation conservation");
}

#[test]
fn pot_ingredient_that_cannot_survive_carrying_keeps_its_owner_and_age() {
    let (mut world, mut res, person, owner) = fixture();
    res.current_tick = res.ticks_per_day * 13;
    let grain = ItemKind::from_name("wild_grain").expect("grain");
    assert!(storage::remaining_days(&world, &res, owner, 13).expect("stored lifetime") > 0);
    let job = hold_supply(&mut world, &mut res, person);
    let food_before = res.food.clone();
    let reservations = res.colony.reservations.clone();
    let overlay_version = res.colony.overlay_version;
    let ledger = super::invariants::ledger_of(&world, &res, grain);
    assert_eq!(
        super::food_transfers::pickup(&mut world, &mut res, person, (4, 5), grain, 2).0,
        0
    );
    assert_eq!(
        res.food, food_before,
        "failed admission preserves ages, tally and generation"
    );
    assert_eq!(res.colony.reservations, reservations);
    assert_eq!(res.colony.overlay_version, overlay_version);
    assert_eq!(*world.get::<&Job>(person).expect("job retained"), job);
    assert_eq!(super::invariants::ledger_of(&world, &res, grain), ledger);
    assert_eq!(
        items::stack(&world, &res, owner)
            .expect("source retained")
            .count,
        2
    );
    assert_eq!(res.food.ages[&owner].born_day, 0);
    assert!(world.get::<&sim_core::colony::Carry>(person).is_err());
    food::validate(&world, &res).expect("unchanged food conservation");
}

#[test]
fn full_carry_rejects_pot_supply_without_partial_take_or_reservation_change() {
    let (mut world, mut res, person, owner) = fixture();
    let grain = ItemKind::from_name("wild_grain").expect("grain");
    let spec = ItemSpec::raw(grain, None);
    let full = super::items::stack_max(&res, grain);
    items::create_ground(&mut res, (2, 2), spec, full, 0, items::FoodSource::Harvest)
        .expect("full load created");
    items::transfer(
        &mut world,
        &mut res,
        FoodOwner::Ground((2, 2)),
        FoodOwner::Carried(7),
        full,
        1,
    )
    .expect("full carried load");
    let job = hold_supply(&mut world, &mut res, person);
    let food_before = res.food.clone();
    let reservations = res.colony.reservations.clone();
    let overlay_version = res.colony.overlay_version;
    let ledger = super::invariants::ledger_of(&world, &res, grain);
    assert_eq!(
        super::food_transfers::pickup(&mut world, &mut res, person, (4, 5), grain, 2).0,
        0
    );
    assert_eq!(
        res.food, food_before,
        "failed admission preserves ages, tally and generation"
    );
    assert_eq!(res.colony.reservations, reservations);
    assert_eq!(res.colony.overlay_version, overlay_version);
    assert_eq!(*world.get::<&Job>(person).expect("job retained"), job);
    assert_eq!(
        items::stack(&world, &res, owner)
            .expect("pot retained")
            .count,
        2
    );
    assert_eq!(
        items::stack(&world, &res, FoodOwner::Carried(7))
            .expect("carry retained")
            .count,
        full
    );
    assert_eq!(super::invariants::ledger_of(&world, &res, grain), ledger);
    food::validate(&world, &res).expect("unchanged food conservation");
    super::invariants::check(&world, &res).expect("unchanged physical ledger and reservation");
}

fn store_grain_fixture(fresh: bool) -> (World, SimResources, hecs::Entity) {
    let (mut world, mut res, person, pot_owner) = fixture();
    items::transfer(
        &mut world,
        &mut res,
        pot_owner,
        FoodOwner::Ground((2, 2)),
        2,
        1,
    )
    .expect("original dated grain leaves pot while portable");
    res.colony.rooms.insert(
        90,
        sim_core::rooms::Room {
            dwelling_kind: None,
            id: 90,
            tiles: std::collections::BTreeSet::from([(2, 2)]),
            doors: std::collections::BTreeSet::new(),
            roofed: true,
            role: sim_core::rooms::RoomRole::Storehouse,
            dug: false,
        },
    );
    res.colony.room_of_tile.insert((2, 2), 90);
    res.colony.zones.insert(
        90,
        sim_core::colony::Zone {
            id: 90,
            kind: sim_core::colony::ZoneKind::Stockpile,
            tiles: std::collections::BTreeSet::from([(2, 2)]),
        },
    );
    res.colony.zone_of_tile.insert((2, 2), 90);
    res.current_tick = 13 * res.ticks_per_day;
    if fresh {
        items::create_ground(
            &mut res,
            (3, 2),
            ItemSpec::raw(ItemKind::from_name("wild_grain").expect("grain"), None),
            2,
            12,
            items::FoodSource::Harvest,
        )
        .expect("fresh unzoned grain");
    }
    res.colony.touch(false);
    assert_eq!(
        storage::remaining_days(&world, &res, FoodOwner::Ground((2, 2)), 13).expect("stored age"),
        11
    );
    food::validate(&world, &res).expect("physical fixture");
    (world, res, person)
}

#[test]
fn supply_freshness_excludes_stored_grain_that_cannot_survive_carry() {
    let (world, res, _) = store_grain_fixture(false);
    let grain = ItemKind::from_name("wild_grain").expect("grain");
    assert_eq!(
        super::bills::available(&res, grain),
        0,
        "Store life24 does not grant Carried life beyond12"
    );
    assert!(!super::bills::runnable(
        &world,
        &res,
        &res.colony.workshops[&1],
        0
    ));
}

#[test]
fn supply_freshness_selects_fresh_unzoned_grain_and_preserves_age_and_ledger() {
    let (mut world, mut res, person) = store_grain_fixture(true);
    let grain = ItemKind::from_name("wild_grain").expect("grain");
    super::bills::schedule(&mut world, &mut res);
    assert_eq!(res.colony.workshops[&1].active, Some(1));
    super::JobAssignSystem::new().tick(&mut world, &mut res);
    assert!(
        matches!(
            world.get::<&Job>(person).expect("actual Supply").kind,
            JobKind::Supply {
                from: (3, 2),
                workshop: 1,
                ..
            }
        ),
        "old stocked grain must not hide portable unzoned grain"
    );
    let tallies = res.food.tallies.clone();
    let created = res.colony.item_ledger.created_of(grain);
    let consumed = res.colony.item_ledger.consumed_of(grain);
    let mut movement = crate::runtime::agent::AgentMovementSystem::new();
    let mut execute = super::JobExecuteSystem::new();
    let start = res.current_tick;
    for tick in start..start + 400 {
        res.current_tick = tick;
        movement.tick(&mut world, &mut res);
        execute.tick(&mut world, &mut res);
        if res.colony.workshops[&1].shelved(grain) == 2 {
            break;
        }
    }
    assert_eq!(res.colony.workshops[&1].shelved(grain), 2);
    let shelf = FoodOwner::Shelf {
        workshop: 1,
        key: FoodStackKey::from(ItemSpec::raw(grain, None)),
    };
    assert_eq!(res.food.ages[&shelf].born_day, 12);
    assert_eq!(res.food.ages[&FoodOwner::Ground((2, 2))].born_day, 0);
    assert_eq!(res.colony.ground_items[&(2, 2)].count, 2);
    assert_eq!(
        res.food.tallies, tallies,
        "transport neither creates nor consumes food"
    );
    assert_eq!(res.colony.item_ledger.created_of(grain), created);
    assert_eq!(res.colony.item_ledger.consumed_of(grain), consumed);
    food::validate(&world, &res).expect("completed Supply ownership");
}

#[test]
fn supply_freshness_keeps_nonfood_ground_inputs_available_without_food_age() {
    let (mut world, mut res, _) = store_grain_fixture(false);
    let flint = ItemKind::from_name("flint").expect("flint");
    items::create_ground(
        &mut res,
        (3, 2),
        ItemSpec::raw(flint, None),
        2,
        0,
        items::FoodSource::Debug,
    )
    .expect("nonfood");
    assert_eq!(super::bills::available(&res, flint), 2);
    let shop = res.colony.workshops.get_mut(&1).expect("workshop");
    let recipe = RecipeKind::from_name("knap_blade").expect("blade");
    shop.kind = recipe.recipe().expect("definition").workshop;
    shop.bills[0].recipe = recipe;
    assert!(super::bills::runnable(
        &world,
        &res,
        &res.colony.workshops[&1],
        0
    ));
    super::bills::schedule(&mut world, &mut res);
    assert_eq!(res.colony.workshops[&1].active, Some(1));
}
