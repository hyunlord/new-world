//! M2 Part 4: firewood through jobs (goal 3.8, §8) — the decision system's
//! `GatherWood` arm is off in the colony, `HearthFeedSystem` turns firewood
//! items into the hearth's wood, and the baseline keeps its arm.

mod colony_support;

use colony_support::tick_checked;
use sim_core::colony::materials::OAK;
use sim_core::colony::{Command, ItemKind, ItemSpec, JobKind};
use sim_core::components::{AgentState, ResourceKind, TargetKind};
use sim_engine::SimEngine;
use sim_systems::commands::submit;
use sim_systems::runtime::jobs::hearth::{feed, firewood_by_jobs, settlement_wood};
use sim_systems::runtime::jobs::invariants::ledger_of;
use sim_systems::runtime::jobs::items::put_items;
use sim_test::probe::colony::stockpile_block;

fn wood_seekers(engine: &SimEngine) -> usize {
    engine
        .world
        .query::<&AgentState>()
        .iter()
        .filter(|(_, s)| s.target() == Some(TargetKind::GatherWood))
        .count()
}

/// Goal §8: no `GatherWood` target ever appears in a colony run (3 days, two
/// nights with the hearth burning), and the hearth is fed from firewood items
/// that jobs gathered.
#[test]
fn a_colony_never_targets_gather_wood_and_jobs_feed_the_hearth() {
    let mut engine = sim_test::colony_engine(4);
    assert!(firewood_by_jobs(&engine.resources));
    let mut gathers = 0u64;
    for _ in 0..3 * 1_440 {
        tick_checked(&mut engine);
        assert_eq!(wood_seekers(&engine), 0, "tick {}", engine.current_tick());
        gathers += engine
            .world
            .query::<&sim_core::colony::Job>()
            .iter()
            .filter(|(_, j)| matches!(j.kind, JobKind::Gather { .. }))
            .count() as u64;
    }
    let c = &engine.resources.colony;
    assert!(gathers > 0, "nobody gathered deadwood");
    let fw = ledger_of(&engine.world, &engine.resources, ItemKind::Firewood);
    assert!(fw.created >= 40, "{} firewood gathered", fw.created);
    assert!(
        c.item_ledger.burned_of(ItemKind::Firewood) >= 20,
        "fed {}",
        c.item_ledger.burned_of(ItemKind::Firewood)
    );
    assert_eq!(engine.resources.tallies.deaths_exposure, 0);
    assert_eq!(
        engine.resources.tallies.wood_trips, 0,
        "no decision wood trips in the colony"
    );
}

/// Goal §8: `HearthFeedSystem` conservation — the firewood items taken equal
/// the wood units added (`burned` = settlement wood gained), nearest the hearth first.
#[test]
fn hearth_feed_turns_items_into_wood_one_for_one() {
    let mut engine = sim_test::colony_engine(1);
    let (x0, y0, x1, y1) = stockpile_block(&engine, 4, 3).expect("stockpile");
    assert!(submit(&mut engine, Command::AddStockpile { x0, y0, x1, y1 }).accepted);
    // Evening of day 0 (18:00 → the burn window): empty the hearth, stock 30 firewood.
    for _ in 0..1_000 {
        engine.tick();
    }
    for s in engine.resources.settlements.values_mut() {
        s.stockpile.insert(ResourceKind::Wood, 0);
    }
    let spec = ItemSpec::raw(ItemKind::Firewood, Some(OAK));
    engine
        .resources
        .colony
        .item_ledger
        .create(ItemKind::Firewood, 30);
    put_items(&mut engine.resources, (x0, y0), spec, 30);
    let before_items = engine.resources.colony.ground_total(ItemKind::Firewood);
    let before_burned = engine
        .resources
        .colony
        .item_ledger
        .burned_of(ItemKind::Firewood);
    feed(&mut engine.resources);
    let fed = engine
        .resources
        .colony
        .item_ledger
        .burned_of(ItemKind::Firewood)
        - before_burned;
    let taken = before_items - engine.resources.colony.ground_total(ItemKind::Firewood);
    let wood = u64::from(settlement_wood(&engine.resources));
    assert!(fed > 0, "the hearth took nothing");
    assert_eq!(
        (taken, wood),
        (fed, fed),
        "items taken = wood units gained = burned"
    );
    // A second feed with the night's wood in hand takes nothing.
    feed(&mut engine.resources);
    assert_eq!(
        engine
            .resources
            .colony
            .item_ledger
            .burned_of(ItemKind::Firewood)
            - before_burned,
        fed
    );
    let t = ledger_of(&engine.world, &engine.resources, ItemKind::Firewood);
    assert!(t.balanced());
}

/// Goal §8: the baseline keeps the decision system's firewood arm.
#[test]
fn the_baseline_still_gathers_wood_through_the_decision_system() {
    let mut engine = sim_test::baseline_engine(3);
    assert!(!firewood_by_jobs(&engine.resources));
    let mut seen = 0usize;
    for _ in 0..2 * 1_440 {
        engine.tick();
        seen += wood_seekers(&engine);
    }
    assert!(seen > 0, "no GatherWood target in the baseline");
    assert!(engine.resources.tallies.wood_trips > 0);
}
