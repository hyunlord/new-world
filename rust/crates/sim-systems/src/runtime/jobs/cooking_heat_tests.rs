use crate::runtime::food::items;
use hecs::World;
use sim_core::colony::food::{FoodOwner, FoodStackKey};
use sim_core::colony::{
    Bill, BillMode, ItemKind, ItemSpec, Job, JobKind, JobStage, RecipeKind, Workshop,
};
use sim_core::components::{Agent, ResourceKind, Settlement};
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimResources};

fn fixture(wood: u32) -> (World, SimResources) {
    let mut world = World::new();
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(1);
    res.rules.colony.as_mut().expect("colony").knowledge = None;
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.current_tick = res.ticks_per_day / 2;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (4, 4);
    settlement.store(ResourceKind::Wood, wood);
    res.settlements.insert(1, settlement);
    let recipe = RecipeKind::from_name("roasted_roots").expect("recipe");
    res.colony.workshops.insert(
        1,
        Workshop {
            id: 1,
            kind: recipe.recipe().expect("recipe").workshop,
            top_left: (4, 4),
            size: (1, 1),
            active: Some(1),
            bills: vec![Bill {
                id: 1,
                recipe,
                mode: BillMode::Maintain(1),
                done: 0,
                suspended: false,
                auto: false,
            }],
            shelf: vec![],
        },
    );
    let roots = ItemSpec::raw(ItemKind::from_name("roots").expect("roots"), None);
    items::create_ground(&mut res, (2, 2), roots, 2, 0, items::FoodSource::Debug).expect("inputs");
    items::transfer(
        &mut world,
        &mut res,
        FoodOwner::Ground((2, 2)),
        FoodOwner::Shelf {
            workshop: 1,
            key: FoodStackKey::from(roots),
        },
        2,
        0,
    )
    .expect("ordinary aged transfer");
    (world, res)
}

#[test]
fn daylight_supply_can_plan_heat_without_claiming_execution_permission() {
    let (_world, res) = fixture(0);
    let shop = &res.colony.workshops[&1];
    let recipe = shop.bills[0].recipe.recipe().expect("recipe");
    assert!(super::craft_food::supplyable(&res, shop, recipe, u32::MAX));
    assert!(!super::craft_food::permitted(&res, shop, recipe, u32::MAX));
}

#[test]
fn daylight_unsatisfied_ready_cooking_bill_burns_actual_fuel() {
    let (mut world, mut res) = fixture(100);
    let rule = res.rules.society.firewood.expect("firewood");
    let rate = u32::from(crate::runtime::settlement::firewood::winter_burn_per_hour(
        &res,
        &rule,
        res.current_tick,
    ));
    assert!(
        rate > 0,
        "daylight heat must never use the zero outside-night rate"
    );
    crate::runtime::settlement::fire::FireSystem::new().tick(&mut world, &mut res);
    assert!(
        res.fires[&1].lit,
        "ready daytime cooking requires real heat service"
    );
    assert_eq!(
        res.settlements[&1].stockpile[&ResourceKind::Wood],
        100 - rate
    );
    let shop = &res.colony.workshops[&1];
    assert!(super::craft_food::permitted(
        &res,
        shop,
        shop.bills[0].recipe.recipe().expect("recipe"),
        u32::MAX
    ));
    let person = world.spawn((Agent { id: 7 },));
    let mut job = craft_job(&res);
    assert!(super::craft::start(&mut world, &mut res, person, &mut job, 1, 1).is_some());
    assert!(res.colony.workshops[&1].shelf.is_empty());
    assert_eq!(res.food.tallies.craft_input_units, 2);
}

#[test]
fn daylight_zero_fuel_paused_and_fulfilled_bills_do_not_create_fire() {
    for state in 0..3 {
        let (mut world, mut res) = fixture(if state == 0 { 0 } else { 100 });
        if state == 1 {
            res.colony.workshops.get_mut(&1).expect("shop").bills[0].suspended = true;
        }
        if state == 2 {
            let food = ItemSpec::raw(ItemKind::from_name("roasted_roots").expect("output"), None);
            items::create_ground(&mut res, (3, 3), food, 1, 0, items::FoodSource::Debug)
                .expect("existing output");
        }
        crate::runtime::settlement::fire::FireSystem::new().tick(&mut world, &mut res);
        assert!(!res.fires[&1].lit, "negative state {state}");
        assert_eq!(
            res.settlements[&1]
                .stockpile
                .get(&ResourceKind::Wood)
                .copied()
                .unwrap_or(0),
            if state == 0 { 0 } else { 100 }
        );
        let shop = &res.colony.workshops[&1];
        assert!(!super::craft_food::permitted(
            &res,
            shop,
            shop.bills[0].recipe.recipe().expect("recipe"),
            u32::MAX
        ));
        if state == 0 {
            let person = world.spawn((Agent { id: 7 },));
            let mut job = craft_job(&res);
            assert!(super::craft::start(&mut world, &mut res, person, &mut job, 1, 1).is_none());
            assert_eq!(res.food.tallies.craft_input_units, 0);
            assert_eq!(res.food.tallies.craft_output_units, 0);
        }
        let roots = ItemSpec::raw(ItemKind::from_name("roots").expect("roots"), None);
        assert_eq!(
            items::stack(
                &world,
                &res,
                FoodOwner::Shelf {
                    workshop: 1,
                    key: FoodStackKey::from(roots)
                }
            )
            .expect("unconsumed input")
            .count,
            2
        );
    }
}

fn craft_job(res: &SimResources) -> Job {
    Job {
        kind: JobKind::Craft {
            workshop: 1,
            bill: 1,
            at: (4, 5),
        },
        stage: JobStage::ToSource,
        taken_at: res.current_tick,
        amount: 0,
        item: ItemKind::from_name("roots").expect("roots"),
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: res.current_tick,
    }
}

#[test]
fn non_colony_daytime_fire_remains_off_even_with_legacy_workshop_data() {
    let (mut world, mut res) = fixture(100);
    res.rules.colony = None;
    crate::runtime::settlement::fire::FireSystem::new().tick(&mut world, &mut res);
    assert!(!res.fires[&1].lit);
    assert_eq!(res.settlements[&1].stockpile[&ResourceKind::Wood], 100);
}
