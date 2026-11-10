//! Physical cooking heat and fuel admission regressions.
use crate::runtime::food::items;
use hecs::World;
use sim_core::colony::food::{FoodOwner, FoodStackKey};
use sim_core::colony::knowledge::KnowledgeMask;
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
    assert!(super::craft_food::supplyable(
        &res,
        shop,
        recipe,
        KnowledgeMask::MAX
    ));
    assert!(!super::craft_food::permitted(
        &res,
        shop,
        recipe,
        KnowledgeMask::MAX
    ));
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
        KnowledgeMask::MAX
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
            KnowledgeMask::MAX
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

#[test]
fn successful_porridge_credits_only_actual_auxiliary_practitioners() {
    use sim_core::colony::knowledge::{defs, Knowledge, Source};

    // Actual knowledge route, pot-only route, and rejected cold start.
    for (knows, wood) in [(true, 100), (false, 100), (true, 0)] {
        let (mut world, mut res) = fixture(wood);
        res.rules.colony.as_mut().expect("colony").knowledge =
            WorldRules::colony(1).colony.expect("colony").knowledge;
        res.current_tick += res.ticks_per_day;
        let recipe = RecipeKind::from_name("porridge").expect("porridge");
        res.colony.workshops.get_mut(&1).expect("shop").bills[0].recipe = recipe;
        let grain = ItemSpec::raw(ItemKind::from_name("wild_grain").expect("grain"), None);
        items::create_ground(&mut res, (2, 3), grain, 2, 1, items::FoodSource::Debug)
            .expect("grain inputs");
        items::transfer(
            &mut world,
            &mut res,
            FoodOwner::Ground((2, 3)),
            FoodOwner::Shelf {
                workshop: 1,
                key: FoodStackKey::from(grain),
            },
            2,
            1,
        )
        .expect("grain shelf");
        if !knows {
            let pot = ItemSpec::raw(ItemKind::from_name("clay_pot").expect("pot"), None);
            items::create_ground(&mut res, (4, 5), pot, 1, 1, items::FoodSource::Debug)
                .expect("physical pot");
            crate::runtime::food::storage::deploy_pot(
                &mut world,
                &mut res,
                FoodOwner::Ground((4, 5)),
                (4, 5),
                1,
            )
            .expect("deployed alternative");
        }
        let cooking = defs().id_of("cooking").expect("primary knowledge");
        let auxiliary = defs().id_of("grain_parching").expect("auxiliary knowledge");
        let mut knowledge = Knowledge::default();
        knowledge.raise_to(cooking, 600, Source::Oral, 0);
        if knows {
            knowledge.raise_to(auxiliary, 600, Source::Oral, 0);
        }
        let before = knowledge.clone();
        let person = world.spawn((Agent { id: 7 }, knowledge));
        crate::runtime::settlement::fire::FireSystem::new().tick(&mut world, &mut res);
        let mut job = craft_job(&res);
        let started = super::craft::start(&mut world, &mut res, person, &mut job, 1, 1);
        assert_eq!(started.is_some(), wood > 0);
        assert_eq!(
            crate::runtime::knowledge::knowledge_of(&world, person),
            before,
            "admission and input consumption alone never credit knowledge"
        );
        if wood == 0 {
            continue;
        }
        assert!(super::craft::finish_craft(
            &mut world, &mut res, 7, person, &job
        ));
        let after = crate::runtime::knowledge::knowledge_of(&world, person);
        let gain = crate::runtime::knowledge::rule(&res)
            .expect("knowledge")
            .use_gain;
        assert_eq!(
            after.proficiency(cooking),
            600 + gain,
            "primary credited once"
        );
        if knows {
            assert_eq!(after.proficiency(auxiliary), 600 + gain);
            assert_eq!(after.get(auxiliary).expect("held").last_used, 1);
        } else {
            assert!(
                after.get(auxiliary).is_none(),
                "pot does not invent knowledge"
            );
        }
        assert_eq!(res.colony.tallies.crafts_done, 1);
    }
}
