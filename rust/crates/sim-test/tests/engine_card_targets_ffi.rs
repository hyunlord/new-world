//! M-LOOP card reads: `get_card_harvest_target` offers only a rectangle the
//! harvest order accepts with exactly the counted plants, and `get_command_log`
//! lists the log in order with the submitted command's kind and tick.

use sim_bridge::ffi::food::submit_harvest;
use sim_bridge::ffi::{collect_command_log, collect_harvest_target};

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
