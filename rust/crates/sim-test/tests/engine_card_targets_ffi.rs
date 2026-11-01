//! M-LOOP card reads: `get_card_harvest_target` offers only a rectangle the
//! harvest order accepts with exactly the counted plants, and `get_command_log`
//! lists the log in order with the submitted command's kind and tick.

use sim_bridge::ffi::card_targets::{collect_chop_target, collect_stockpile_target};
use sim_bridge::ffi::food::submit_harvest;
use sim_bridge::ffi::{collect_command_log, collect_harvest_target};
use sim_core::colony::Command;

#[test]
fn harvest_target_is_accepted_with_its_count_until_none_remain() {
    let mut engine = sim_test::colony_engine(0);
    let mut offered = 0;
    loop {
        let t = collect_harvest_target(&engine.resources);
        if !t.found {
            break;
        }
        assert!(t.count > 0, "a found target marks at least one plant");
        let r = submit_harvest(
            &mut engine,
            (t.x0 as i32, t.y0 as i32, t.x1 as i32, t.y1 as i32),
        );
        assert!(r.accepted, "offered rectangle rejected: {r:?}");
        assert_eq!(
            r.count, t.count,
            "the order marks exactly the counted plants"
        );
        offered += 1;
        assert!(offered < 10_000, "target loop must end");
    }
    assert!(offered > 0, "seed 0 has harvestable plants near camp");
    let again = collect_harvest_target(&engine.resources);
    assert!(!again.found && again.count == 0);
}

#[test]
fn harvest_target_is_empty_without_a_colony() {
    let engine = sim_test::baseline_engine(0);
    assert_eq!(
        collect_harvest_target(&engine.resources),
        Default::default()
    );
}

#[test]
fn command_log_lists_submitted_orders_in_order() {
    let mut engine = sim_test::colony_engine(0);
    let before = engine.resources.colony.command_log.len();
    let tick = engine.current_tick();
    let t = collect_harvest_target(&engine.resources);
    submit_harvest(
        &mut engine,
        (t.x0 as i32, t.y0 as i32, t.x1 as i32, t.y1 as i32),
    );
    submit_harvest(&mut engine, (100_000, 100_000, 100_005, 100_005));
    let rows = collect_command_log(&engine.resources, before);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].kind, "DesignateHarvest");
    assert_eq!(rows[0].tick, tick);
    assert!(rows[0].accepted);
    assert!(
        !rows[1].accepted,
        "an out-of-map order is logged as rejected"
    );
    assert_eq!(collect_command_log(&engine.resources, usize::MAX).len(), 0);
}

#[test]
fn stockpile_target_is_empty_without_a_colony() {
    let engine = sim_test::baseline_engine(0);
    assert_eq!(
        collect_stockpile_target(&engine.resources),
        Default::default()
    );
}

#[test]
fn stockpile_target_counts_only_native_admissible_cells() {
    // Given a real colony and its current stockpile offer.
    let mut engine = sim_test::colony_engine(0);
    let target = collect_stockpile_target(&engine.resources);
    assert!(target.found);
    let expected = (target.y0..=target.y1)
        .flat_map(|y| (target.x0..=target.x1).map(move |x| (x, y)))
        .filter(|tile| {
            !engine.resources.colony.zone_of_tile.contains_key(tile)
                && sim_systems::runtime::jobs::items::can_hold_items(&engine.resources, *tile)
        })
        .count();
    assert!(expected > 0);
    // When the actual command receives the offered rectangle.
    let result = sim_systems::commands::submit(
        &mut engine,
        Command::AddStockpile {
            x0: target.x0,
            y0: target.y0,
            x1: target.x1,
            y1: target.y1,
        },
    );
    // Then admission succeeds with precisely those cells.
    assert!(result.accepted, "offered stockpile rejected: {result:?}");
    assert_eq!(result.count as usize, expected);
}

#[test]
fn stockpile_target_is_empty_when_every_admissible_cell_is_zoned() {
    // Given a colony whose item-holding tiles are all designated through the native command.
    let mut engine = sim_test::colony_engine(0);
    let x1 = engine.resources.tile_grid.width - 1;
    let y1 = engine.resources.tile_grid.height - 1;
    let result = sim_systems::commands::submit(
        &mut engine,
        Command::AddStockpile {
            x0: 0,
            y0: 0,
            x1,
            y1,
        },
    );
    assert!(result.accepted);
    // When observing a new offer, no positive rectangle can be admitted.
    let target = collect_stockpile_target(&engine.resources);
    assert_eq!(target, Default::default());
}

#[test]
fn card_target_observers_preserve_full_future_state_and_command_log() {
    // Given identical live colonies, repeatedly inspect only one of them.
    let mut observed = sim_test::colony_engine(0);
    let mut control = sim_test::colony_engine(0);
    // When observation and matching simulation ticks are interleaved.
    for _ in 0..120 {
        let first = collect_stockpile_target(&observed.resources);
        assert_eq!(first, collect_stockpile_target(&observed.resources));
        let chop = collect_chop_target(&observed.resources);
        assert_eq!(chop, collect_chop_target(&observed.resources));
        observed.tick();
        control.tick();
    }
    // Then complete serialized state and order history remain identical.
    assert_eq!(
        observed.resources.colony.command_log,
        control.resources.colony.command_log
    );
    assert_eq!(
        sim_systems::save::save(&mut observed).expect("observed world"),
        sim_systems::save::save(&mut control).expect("control world")
    );
}

#[test]
fn chop_target_is_empty_without_a_colony() {
    let engine = sim_test::baseline_engine(0);
    assert_eq!(collect_chop_target(&engine.resources), Default::default());
}

#[test]
fn chop_target_is_accepted_and_immediate_repeat_never_offers_same_tree() {
    // Given a real reachable unmarked tree selected by the observer.
    let mut engine = sim_test::colony_engine(0);
    let target = collect_chop_target(&engine.resources);
    assert!(target.found);
    assert_eq!((target.x0, target.y0), (target.x1, target.y1));
    // When the real command designates it.
    let result = sim_systems::commands::submit(
        &mut engine,
        Command::DesignateChop {
            x0: target.x0,
            y0: target.y0,
            x1: target.x1,
            y1: target.y1,
        },
    );
    // Then it marks exactly one tree, which is not offered again without a tick.
    assert!(result.accepted, "offered chop rejected: {result:?}");
    assert_eq!(result.count, 1);
    let next = collect_chop_target(&engine.resources);
    assert!(!next.found || (next.x0, next.y0) != (target.x0, target.y0));
}

#[test]
fn chop_target_is_empty_when_all_reachable_trees_are_marked() {
    // Given all currently reachable trees designated through the real command.
    let mut engine = sim_test::colony_engine(0);
    let x1 = engine.resources.tile_grid.width - 1;
    let y1 = engine.resources.tile_grid.height - 1;
    let result = sim_systems::commands::submit(
        &mut engine,
        Command::DesignateChop {
            x0: 0,
            y0: 0,
            x1,
            y1,
        },
    );
    assert!(result.accepted);
    // When another target is requested, unreachable or marked trees are not offered.
    assert_eq!(collect_chop_target(&engine.resources), Default::default());
}
