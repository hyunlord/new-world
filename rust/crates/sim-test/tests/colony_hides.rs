//! M5 Part 3: hides, apparel and knife wear (ADR 0029) — scraping cures a hide
//! with the drying rack's knife, the knife breaks after 8 uses (ledger
//! consumed), hide wraps and fur cloaks are made and worn by the least
//! cold-tolerant first, a worn top spares its wearer cold hours (cold 700 ‰ of them),
//! and meat spoils six days after the kill.

mod colony_support;

use colony_support::{build_workshop, drop_items, job_world_m2, run_until, tick_checked, worker};
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::people::{Apparel, Top};
use sim_core::colony::{BillMode, BlueprintKind, Command, ItemKind, ItemSpec, RecipeKind};
use sim_core::components::Hardiness;
use sim_core::rules::{ColonyRule, WildlifeRule};
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::commands::submit;
use sim_systems::runtime::jobs::invariants::ledger_of;

/// The M2 job world with the M5 wildlife rule (knives, dressing), no knowledge gating.
fn hide_world() -> (SimEngine, u32) {
    let mut engine = job_world_m2(32, 32);
    engine.resources.rules.colony = Some(ColonyRule {
        wildlife: Some(WildlifeRule::M5),
        ..ColonyRule::M2
    });
    for x in [3, 5, 7] {
        let e = worker(&mut engine, x, 3);
        let _ = engine.world.insert_one(
            e,
            Hardiness {
                permille: 700 + 100 * x as u16,
            },
        );
    }
    assert!(
        submit(
            &mut engine,
            Command::AddStockpile {
                x0: 18,
                y0: 18,
                x1: 24,
                y1: 24
            }
        )
        .accepted
    );
    let shop = build_workshop(&mut engine, BlueprintKind::DryingRack, (12, 12), (4, 4));
    (engine, shop)
}

fn raw(kind: ItemKind) -> ItemSpec {
    ItemSpec::raw(kind, None)
}

fn bill(engine: &mut SimEngine, shop: u32, recipe: RecipeKind, n: u16) {
    assert!(
        submit(
            engine,
            Command::AddBill {
                workshop: shop,
                recipe,
                mode: BillMode::Make(n)
            }
        )
        .accepted
    );
}

#[test]
fn scraping_cures_hides_with_the_racks_knife_and_the_knife_breaks_after_8_uses() {
    let (mut engine, shop) = hide_world();
    drop_items(&mut engine, (20, 20), raw(ItemKind::Hide), 9);
    drop_items(&mut engine, (21, 20), raw(ItemKind::FlintBlade), 2);
    bill(&mut engine, shop, RecipeKind::ScrapeHide, 9);
    let used = run_until(&mut engine, 12_000, |e| {
        e.resources.colony.tallies.crafts_done >= 9
    });
    assert!(used < 12_000, "nine scrapes");
    assert_eq!(
        engine.resources.colony.knife_wear.get(&shop).copied(),
        Some(1),
        "the second knife has one use"
    );
    for _ in 0..300 {
        tick_checked(&mut engine);
    }
    let w = &engine.resources.colony.wildlife.tallies;
    assert_eq!(w.knife_uses, 9, "one knife use a scrape");
    assert_eq!(w.knives_broken, 1, "the 8th use broke the first blade");
    let blades = ledger_of(&engine.world, &engine.resources, ItemKind::FlintBlade);
    assert_eq!(
        blades.consumed, 1,
        "the broken knife is used up; the hides' blades are not inputs"
    );
    let cured = ledger_of(&engine.world, &engine.resources, ItemKind::CuredHide);
    let hides = ledger_of(&engine.world, &engine.resources, ItemKind::Hide);
    assert_eq!((cured.created, hides.consumed), (9, 9));
    assert!(engine
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|e| e.kind == ChronicleKind::ToolBroke
            && e.cause == ItemKind::FlintBlade.code()
            && e.value == 8));
    // No bill needs the second blade any more: it leaves the shelf, and its wear with it (code review m5).
    let shelved = engine.resources.colony.workshops[&shop].shelved(ItemKind::FlintBlade);
    assert_eq!(shelved, 0, "the unused blade was spilled");
    assert_eq!(
        engine.resources.colony.knife_wear.get(&shop),
        None,
        "the next blade starts unworn"
    );
}

#[test]
fn hide_wraps_and_fur_cloaks_are_made_and_worn_by_the_least_cold_tolerant_first() {
    let (mut engine, shop) = hide_world();
    drop_items(&mut engine, (20, 20), raw(ItemKind::CuredHide), 2);
    drop_items(&mut engine, (21, 20), raw(ItemKind::WolfHide), 2);
    bill(&mut engine, shop, RecipeKind::MakeHideWrap, 1);
    bill(&mut engine, shop, RecipeKind::MakeFurCloak, 1);
    let used = run_until(&mut engine, 8_000, |e| {
        e.world.query::<&Apparel>().iter().count() >= 2
    });
    assert!(used < 8_000, "both tops made and put on");
    let mut worn: Vec<(u16, Top)> = engine
        .world
        .query::<(&Apparel, &Hardiness)>()
        .iter()
        .map(|(_, (a, h))| (h.permille, a.top))
        .collect();
    worn.sort();
    // Hardiness 1,000 / 1,200 / 1,400 ‰ (higher = less cold-tolerant): the cloak to 1,400, the wrap to 1,200.
    assert_eq!(worn, vec![(1_200, Top::HideWrap), (1_400, Top::FurCloak)]);
    for kind in [ItemKind::HideWrap, ItemKind::FurCloak] {
        let l = ledger_of(&engine.world, &engine.resources, kind);
        assert_eq!(
            (l.created, l.held, l.ground),
            (1, 1, 0),
            "{kind:?} is worn, counted as held"
        );
    }
}

#[test]
fn a_worn_top_spares_its_wearer_cold_hours_at_its_permille() {
    let mut engine = sim_test::colony_engine(2);
    for _ in 0..10 {
        engine.tick();
    }
    let rule = engine.resources.rules.society.firewood.expect("firewood");
    let ids: Vec<_> = engine.resources.settlements.keys().copied().collect();
    let (_, e) = colony_support::adults(&engine)[0];
    // One hour of FireSystem at `tick` with the hearth empty: (cold?, hp lost).
    let hour = |engine: &mut SimEngine, top: Option<Top>, tick: u64| {
        for sid in &ids {
            if let Some(s) = engine.resources.settlements.get_mut(sid) {
                s.withdraw(sim_core::components::ResourceKind::Wood, u32::MAX);
            }
        }
        let _ = engine.world.remove_one::<Apparel>(e);
        if let Some(top) = top {
            let _ = engine.world.insert_one(e, Apparel { top, since_day: 0 });
        }
        if let Ok(mut h) = engine.world.get::<&mut sim_core::components::BodyHealth>(e) {
            h.hp = 100.0;
        }
        engine.resources.current_tick = tick;
        sim_systems::runtime::settlement::FireSystem::new()
            .tick(&mut engine.world, &mut engine.resources);
        let cold = engine.world.get::<&sim_core::components::Cold>(e).is_ok();
        let lost = 100.0
            - engine
                .world
                .get::<&sim_core::components::BodyHealth>(e)
                .map_or(100.0, |h| h.hp);
        (cold, lost)
    };
    // Every hour of every winter night in 40 years in which a bare person is cold.
    let mut hours = Vec::new();
    for t in (0..40 * 24 * 1_440u64).step_by(60) {
        engine.resources.current_tick = t;
        if sim_systems::runtime::settlement::firewood::night_season(&engine.resources, &rule) == 3
            && hour(&mut engine, None, t).0
        {
            hours.push(t);
        }
    }
    let n = hours.len() as u64;
    assert!(n >= 400, "{n} cold hours to sample");
    let (_, bare_loss) = hour(&mut engine, None, hours[0]);
    assert!(bare_loss > 0.0, "the cold bites a bare person");
    for (top, permille) in [(Top::HideWrap, 700u64), (Top::FurCloak, 500)] {
        let mut cold = 0u64;
        for &t in &hours {
            let (c, lost) = hour(&mut engine, Some(top), t);
            if c {
                cold += 1;
                // A cold hour bites a wearer like anyone else; the top acts through the roll.
                assert!(
                    (lost - bare_loss).abs() < 1e-9,
                    "{top:?} at {t}: {lost} vs {bare_loss}"
                );
            } else {
                assert!(lost == 0.0, "a spared hour does no damage");
            }
        }
        // Binomial: sd = sqrt(p(1-p)/n) <= 25 per mille at n >= 400; 4 sd = 100 (seed 2: 678 and 481 of 1,920).
        let got = cold * 1000 / n;
        println!("{top:?}: {got} per mille of {n} cold hours");
        assert!(
            got.abs_diff(permille) <= 100,
            "{top:?}: {got} per mille of {n} hours cold, want {permille}"
        );
    }
}

#[test]
fn meat_spoils_six_days_after_the_kill() {
    let mut engine = sim_test::colony_engine(4);
    engine.tick();
    let mut animals = sim_systems::runtime::wildlife::AnimalSystem;
    animals.tick(&mut engine.world, &mut engine.resources);
    // A carcass far from any stockpile or hearth (no larder takes it).
    let h = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    let far = (h.0.saturating_sub(30), h.1);
    engine.resources.current_tick = 1_440;
    sim_systems::runtime::jobs::hunt::drop_carcass(&mut engine.resources, far, 3, ItemKind::Hide);
    for (day, left) in [(6u64, 3u64), (7, 0)] {
        engine.resources.current_tick = day * 1_440;
        animals.tick(&mut engine.world, &mut engine.resources);
        let meat = ledger_of(&engine.world, &engine.resources, ItemKind::Meat);
        assert_eq!(meat.ground, left, "day {day}");
    }
    let w = &engine.resources.colony.wildlife;
    assert_eq!((w.tallies.meat_spoiled, w.meat_lots.len()), (3, 0));
    let meat = ledger_of(&engine.world, &engine.resources, ItemKind::Meat);
    assert_eq!(
        (meat.created, meat.consumed),
        (3, 3),
        "spoilage is consumed"
    );
}

/// M5.5 (ADR 0029 amendment): under the M5.5 wildlife rule a knife lasts 4 uses, and a hide wrap takes one cured hide.
#[test]
fn an_m55_knife_breaks_after_4_uses_and_a_wrap_takes_one_cured_hide() {
    let (mut engine, shop) = hide_world();
    if let Some(c) = engine.resources.rules.colony.as_mut() {
        c.wildlife = WildlifeRule::m5_5();
    }
    drop_items(&mut engine, (20, 20), raw(ItemKind::Hide), 5);
    drop_items(&mut engine, (21, 20), raw(ItemKind::FlintBlade), 2);
    bill(&mut engine, shop, RecipeKind::ScrapeHide, 5);
    let used = run_until(&mut engine, 8_000, |e| {
        e.resources.colony.tallies.crafts_done >= 5
    });
    assert!(used < 8_000, "five scrapes");
    let w = &engine.resources.colony.wildlife.tallies;
    // Five uses: the first blade broke at its 4th, the second has one (M5's 8-use knife would have none broken).
    assert_eq!((w.knife_uses, w.knives_broken), (5, 1));
    assert_eq!(
        engine.resources.colony.knife_wear.get(&shop).copied(),
        Some(1)
    );
    assert!(engine.resources.chronicle.entries().iter().any(|e| {
        e.kind == ChronicleKind::ToolBroke && e.cause == ItemKind::FlintBlade.code() && e.value == 4
    }));
    // Three wraps from three of the five cured hides (M5: two hides a wrap — only two wraps).
    bill(&mut engine, shop, RecipeKind::MakeHideWrap, 3);
    let used = run_until(&mut engine, 8_000, |e| {
        ledger_of(&e.world, &e.resources, ItemKind::HideWrap).created >= 3
    });
    assert!(used < 8_000, "three wraps");
    for _ in 0..200 {
        tick_checked(&mut engine);
    }
    let cured = ledger_of(&engine.world, &engine.resources, ItemKind::CuredHide);
    let wraps = ledger_of(&engine.world, &engine.resources, ItemKind::HideWrap);
    assert_eq!(
        (wraps.created, cured.consumed),
        (3, 3),
        "one cured hide a wrap"
    );
}
