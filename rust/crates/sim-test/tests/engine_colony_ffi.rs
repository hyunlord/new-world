//! M1 bridge surface (goal 6.5): the colony command helpers and readers
//! behind `WorldSimNode`'s `cmd_*`, `can_place_blueprint`, `get_trees`,
//! `get_colony_overlay`, `get_colony_versions` and the agent snapshot's
//! `jobs` / `carry_kinds` / `carry_counts` columns, called headless.

use sim_bridge::ffi::{
    can_place, collect_agent_jobs, collect_agent_snapshot, collect_colony_overlay,
    collect_colony_versions, collect_trees, submit_blueprint, submit_rect, submit_tiles,
    RectCommand,
};
use sim_core::colony::{Carry, ItemKind, RejectReason};
use sim_systems::worldgen::colony::hearth;

#[test]
fn colony_reads_follow_the_orders() {
    let mut engine = sim_test::colony_engine(0);
    let (version, trees) = collect_trees(&engine.resources);
    assert_eq!(trees.len(), engine.resources.colony.trees.len());
    assert!(
        trees.windows(2).all(|w| w[0] < w[1]),
        "tiles in (x, y) order"
    );
    assert_eq!(version, engine.resources.colony.trees_version);
    let before = collect_colony_versions(&engine.resources);
    let overlay = collect_colony_overlay(&engine.world, &engine.resources);
    assert!(
        overlay.designations.is_empty() && overlay.sites.is_empty() && overlay.items.is_empty()
    );

    let h = hearth(&engine.resources).expect("hearth");
    let near: Vec<(u32, u32)> = sim_test::probe::colony::nearest_trees(&engine, 3);
    let xs: Vec<i32> = near.iter().map(|t| t.0 as i32).collect();
    let ys: Vec<i32> = near.iter().map(|t| t.1 as i32).collect();
    let r = submit_tiles(&mut engine, &xs, &ys);
    assert_eq!((r.accepted, r.count), (true, 3));
    let block = sim_test::probe::colony::stockpile_block(&engine, 2, 3).expect("block");
    let r = submit_rect(
        &mut engine,
        RectCommand::AddStockpile,
        block.0 as i32,
        block.1 as i32,
        block.2 as i32,
        block.3 as i32,
    );
    assert_eq!((r.accepted, r.count), (true, 4));
    let spot = sim_test::probe::colony::hut_spot(&engine, &[]).expect("hut spot");
    assert!(can_place(
        &engine.resources,
        "hut",
        spot.0 as i32,
        spot.1 as i32
    ));
    assert!(!can_place(
        &engine.resources,
        "wall",
        spot.0 as i32,
        spot.1 as i32
    ));
    let r = submit_blueprint(&mut engine, "hut", spot.0 as i32, spot.1 as i32);
    assert!(r.accepted);
    assert!(
        !can_place(&engine.resources, "hut", spot.0 as i32, spot.1 as i32),
        "the site blocks itself"
    );

    let after = collect_colony_versions(&engine.resources);
    assert!(after.overlay > before.overlay);
    assert_eq!(
        after.trees, before.trees,
        "marking trees does not change the tree layer"
    );
    let overlay = collect_colony_overlay(&engine.world, &engine.resources);
    let mut marked = near.clone();
    marked.sort();
    assert_eq!(overlay.designations, marked);
    assert_eq!(overlay.zones.len(), 4);
    assert!(overlay
        .zones
        .iter()
        .all(|(id, _)| *id == overlay.zones[0].0));
    assert_eq!(overlay.sites.len(), 1);
    let site = overlay.sites[0];
    assert_eq!((site.x, site.y, site.w, site.h), (spot.0, spot.1, 3, 3));
    assert_eq!((site.needed, site.delivered, site.progress), (12, 0, 0));
    assert_eq!(site.required, 240);
    // Off-map rectangles are rejected before they reach the simulation.
    let r = submit_rect(&mut engine, RectCommand::CancelArea, -5, -5, -1, -1);
    assert_eq!(r.reason, Some(RejectReason::OutOfMap));
    assert_eq!(
        engine.resources.colony.command_log.len(),
        3,
        "rejected off-map rects are not logged"
    );
    let _ = h;
}

#[test]
fn agent_job_columns_match_the_snapshot_rows() {
    let mut engine = sim_test::colony_engine(1);
    let rows = collect_agent_snapshot(&engine.world);
    let jobs = collect_agent_jobs(&engine.world);
    assert_eq!(rows.len(), jobs.len());
    assert!(
        jobs.iter().all(|j| *j == (0, -1, 0)),
        "no job, no load at tick 0"
    );
    // Give the third row's agent a load: its row (and only it) shows it.
    let entity = hecs::Entity::from_bits(rows[2].entity_bits).expect("entity");
    let _ = engine.world.insert_one(
        entity,
        Carry {
            kind: ItemKind::Log,
            count: 4,
        },
    );
    let rows = collect_agent_snapshot(&engine.world);
    let jobs = collect_agent_jobs(&engine.world);
    let marked: Vec<usize> = jobs
        .iter()
        .enumerate()
        .filter(|(_, j)| j.2 > 0)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(marked.len(), 1);
    assert_eq!(rows[marked[0]].entity_bits, entity.to_bits().get());
    assert_eq!(jobs[marked[0]], (0, 0, 4));
}
