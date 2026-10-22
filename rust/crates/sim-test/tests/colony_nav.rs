//! M3 Part 1: pathfinding in the colony (ADR 0021) — agents walk around walls,
//! never stand on an impassable tile, are never sent to an unreachable target,
//! and a walk cut by a new wall ends as `Unreachable`.

mod colony_support;

use colony_support::{job_of, job_world, plant, tick_checked, tile_of, worker};
use sim_core::colony::{CancelReason, Command, JobKind};
use sim_core::material::MaterialId;
use sim_engine::SimEngine;
use sim_systems::commands::submit;
use sim_systems::runtime::nav;

/// Put a built wall on `tile` and refresh the nav map (as the job layer will).
fn wall(engine: &mut SimEngine, tile: (u32, u32)) {
    let g = &mut engine.resources.tile_grid;
    let i = g.idx(tile.0, tile.1);
    g.wall_material[i] = Some(MaterialId::from_str_hash("oak"));
    nav::refresh(&mut engine.resources, tile);
}

fn designate(engine: &mut SimEngine, tile: (u32, u32)) {
    let r = submit(engine, Command::DesignateChopTiles { tiles: vec![tile] });
    assert!(r.accepted, "{r:?}");
}

#[test]
fn movement_follows_path_around_a_wall() {
    let mut engine = job_world(30, 30);
    let w = worker(&mut engine, 5, 10);
    // A wall at x = 15 from y = 0 to 25: the way to the tree goes through the gap below.
    nav::ensure(&mut engine.resources);
    for y in 0..=25 {
        wall(&mut engine, (15, y));
    }
    plant(&mut engine, (25, 10), 4);
    designate(&mut engine, (25, 10));
    let mut visited = Vec::new();
    let mut reached = None;
    for t in 0..400 {
        tick_checked(&mut engine);
        let at = tile_of(&engine, w);
        assert!(
            nav::passable(&engine.resources, at),
            "tick {t}: on impassable {at:?}"
        );
        visited.push(at);
        if reached.is_none()
            && job_of(&engine, w).is_some_and(|j| {
                matches!(j.kind, JobKind::Chop { .. })
                    && matches!(j.stage, sim_core::colony::JobStage::Working { .. })
            })
        {
            reached = Some(t);
        }
    }
    let reached = reached.expect("the chopper reached the tree within 400 ticks");
    // It crossed x = 15 only through the gap (y >= 26).
    assert!(
        visited.iter().any(|t| t.0 == 15 && t.1 >= 26),
        "never used the gap"
    );
    // The straight line is 20 steps; around the wall it is at least 2 × 16 (down and up again).
    assert!(reached >= 32, "reached the tree after {reached} ticks");
    assert_eq!(
        engine
            .resources
            .colony
            .tallies
            .cancels_of(CancelReason::Stuck),
        0
    );
    let stats = *engine.resources.nav.stats.borrow();
    assert!(stats.searches >= 1 && stats.expanded > 0, "{stats:?}");
}

#[test]
fn a_target_behind_a_sealed_wall_is_never_assigned() {
    let mut engine = job_world(30, 30);
    let w = worker(&mut engine, 5, 10);
    nav::ensure(&mut engine.resources);
    for y in 0..30 {
        wall(&mut engine, (15, y));
    }
    plant(&mut engine, (25, 10), 4);
    designate(&mut engine, (25, 10));
    // A reachable tree too, so the check cannot pass on a worker who takes nothing at all.
    plant(&mut engine, (8, 20), 4);
    for _ in 0..30 {
        tick_checked(&mut engine);
        if let Some(job) = job_of(&engine, w) {
            assert!(
                !matches!(job.kind, JobKind::Chop { tree: (25, 10) }),
                "assigned the unreachable tree"
            );
        }
    }
    designate(&mut engine, (8, 20));
    let mut took = false;
    for _ in 0..30 {
        tick_checked(&mut engine);
        took |= job_of(&engine, w).is_some_and(|j| j.kind == JobKind::Chop { tree: (8, 20) });
    }
    assert!(took, "the reachable tree was taken");
    assert_eq!(
        engine
            .resources
            .colony
            .tallies
            .cancels_of(CancelReason::Unreachable),
        0
    );
}

#[test]
fn a_wall_cutting_the_way_cancels_the_walk_as_unreachable() {
    let mut engine = job_world(30, 30);
    let w = worker(&mut engine, 5, 10);
    plant(&mut engine, (25, 10), 4);
    designate(&mut engine, (25, 10));
    for _ in 0..4 {
        tick_checked(&mut engine);
    }
    assert!(job_of(&engine, w).is_some(), "walking to the tree");
    // Seal the way while it walks.
    for y in 0..30 {
        wall(&mut engine, (15, y));
    }
    for _ in 0..4 {
        tick_checked(&mut engine);
    }
    assert!(job_of(&engine, w).is_none(), "the walk ended");
    let c = &engine.resources.colony;
    assert_eq!(c.tallies.cancels_of(CancelReason::Unreachable), 1);
    let id = engine
        .world
        .get::<&sim_core::components::Agent>(w)
        .map(|a| a.id)
        .expect("agent");
    let until = c
        .unreachable
        .get(&(id, (25, 10)))
        .copied()
        .expect("cooldown");
    let now = engine.current_tick();
    assert!(
        until > now && until + 4 >= now + 1_440 && until <= now + 1_440,
        "one day from the cancel: until {until}, now {now}"
    );
    assert!(c.reservations.is_empty(), "{:?}", c.reservations);
}

#[test]
fn idle_wanderers_never_leave_a_walled_box() {
    let mut engine = job_world(20, 20);
    nav::ensure(&mut engine.resources);
    for i in 4..=12 {
        for t in [(i, 4), (i, 12), (4, i), (12, i)] {
            wall(&mut engine, t);
        }
    }
    let people: Vec<_> = (0..6).map(|k| worker(&mut engine, 6 + k, 8)).collect();
    let mut moved = 0;
    let start: Vec<_> = people.iter().map(|p| tile_of(&engine, *p)).collect();
    for _ in 0..2_000 {
        tick_checked(&mut engine);
        for p in &people {
            let (x, y) = tile_of(&engine, *p);
            assert!(
                (5..=11).contains(&x) && (5..=11).contains(&y),
                "left the box at {x},{y}"
            );
        }
    }
    for (p, s) in people.iter().zip(start) {
        moved += usize::from(tile_of(&engine, *p) != s);
    }
    assert!(moved >= 3, "only {moved} of 6 wandered at all");
}
