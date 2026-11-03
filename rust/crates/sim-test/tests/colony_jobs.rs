//! M1 Part 2: the job layer (goal 7.8; ADR 0013). Mechanics on the small
//! `job_world` (movement, jobs and construction only); needs pre-empting a
//! job and the hut's beds on the live colony world. Every tick of every run
//! checks the 7.6 invariants (log ledger, reservations, stacks).

mod colony_support;
pub mod save_gate_observation;
use save_gate_observation::Watch;

use colony_support::*;
use sim_core::colony::{
    BlueprintKind, CancelReason, Command, ItemKind, JobKind, JobStage, ReserveKey,
};
use sim_core::components::{Agent, AgentState, BuildingKind, Child, Hunger, TargetKind};
use sim_engine::SimEngine;
use sim_systems::commands::{apply_command, check_blueprint, submit};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::jobs::invariants::log_ledger;

fn order(engine: &mut SimEngine, cmd: Command) {
    let result = apply_command(&mut engine.world, &mut engine.resources, cmd.clone());
    assert!(result.accepted, "{cmd:?} → {result:?}");
}

fn chop(tile: (u32, u32)) -> Command {
    Command::DesignateChopTiles { tiles: vec![tile] }
}

fn zone(x0: u32, y0: u32, x1: u32, y1: u32) -> Command {
    Command::AddStockpile { x0, y0, x1, y1 }
}

#[test]
fn jobs_chop_one_tree() {
    let mut engine = job_world(24, 24);
    let w = worker(&mut engine, 2, 2);
    let watch = Watch::begin("jobs_chop_one_tree", 0, "primary", &engine);
    plant(&mut engine, (14, 10), 4);
    order(&mut engine, chop((14, 10)));
    watch.annotate(&engine, "fixture_setup", || serde_json::json!({"recipe": "jobs_chop_one_tree", "founding_boundary": "post_worker_spawn_tick_zero", "world_seed": engine.resources.world_seed, "width": engine.resources.tile_grid.width, "height": engine.resources.tile_grid.height, "ground_logs": ground_logs(&engine), "log_ledger": log_ledger(&engine.world, &engine.resources)}));
    let mut working_ticks = 0;
    let used = run_until(&mut engine, 2_000, |e| {
        !e.resources.colony.trees.contains_key(&(14, 10))
    });
    assert!(used < 2_000, "the tree was never felled");
    // Re-run to count the work ticks independently of the walk.
    let mut again = job_world(24, 24);
    let w2 = worker(&mut again, 2, 2);
    let again_watch = Watch::begin("jobs_chop_one_tree", 0, "again", &again);
    plant(&mut again, (14, 10), 4);
    order(&mut again, chop((14, 10)));
    again_watch.annotate(&again, "fixture_setup", || serde_json::json!({"recipe": "jobs_chop_one_tree", "founding_boundary": "post_worker_spawn_tick_zero", "world_seed": again.resources.world_seed, "width": again.resources.tile_grid.width, "height": again.resources.tile_grid.height, "ground_logs": ground_logs(&again), "log_ledger": log_ledger(&again.world, &again.resources)}));
    while again.resources.colony.trees.contains_key(&(14, 10)) {
        if matches!(
            job_of(&again, w2).map(|j| j.stage),
            Some(JobStage::Working { .. })
        ) {
            working_ticks += 1;
        }
        tick_checked(&mut again);
    }
    again_watch.annotate(&again, "event_stop", || serde_json::json!({"predicate": "tree_felled", "tree_absent": !again.resources.colony.trees.contains_key(&(14, 10)), "working_ticks": working_ticks}));
    again_watch.finish(&again, "tree_felled");
    // Arrival sets Working{90}; ticks 90..1 then fell: 90 ticks seen in Working.
    assert_eq!(working_ticks, RULE.chop_ticks);
    let c = &engine.resources.colony;
    assert!(
        !c.designations.contains_key(&(14, 10)),
        "the mark goes with the tree"
    );
    assert_eq!(
        c.ground_items.get(&(14, 10)).map(|s| (s.kind, s.count)),
        Some((ItemKind::Log, 4))
    );
    assert_eq!(
        log_ledger(&engine.world, &engine.resources),
        (4, 4, 0, 0, 0)
    );
    assert!(job_of(&engine, w).is_none());
    assert!(c.reservations.is_empty());
    assert_eq!((c.tallies.trees_felled, c.tallies.jobs_done), (1, 1));
    assert!(c.trees_version > 0);
    watch.annotate(&engine, "event_stop", || serde_json::json!({"predicate": "tree_felled", "tree_absent": !engine.resources.colony.trees.contains_key(&(14, 10)), "used": used, "limit": 2000}));
    watch.finish(&engine, "tree_felled");
}

#[test]
fn jobs_haul_to_stockpile() {
    let mut engine = job_world(30, 30);
    let a = worker(&mut engine, 1, 1);
    let b = worker(&mut engine, 2, 1);
    let watch = Watch::begin("jobs_haul_to_stockpile", 0, "primary", &engine);
    drop_logs(&mut engine, (4, 4), 25);
    drop_logs(&mut engine, (5, 4), 13);
    order(&mut engine, zone(20, 20, 21, 21));
    watch.annotate(&engine, "fixture_setup", || serde_json::json!({"recipe": "jobs_haul_to_stockpile", "founding_boundary": "post_worker_spawn_tick_zero", "world_seed": engine.resources.world_seed, "width": engine.resources.tile_grid.width, "height": engine.resources.tile_grid.height, "ground_logs": ground_logs(&engine), "log_ledger": log_ledger(&engine.world, &engine.resources)}));
    let in_zone = |e: &SimEngine| -> u64 {
        e.resources
            .colony
            .ground_items
            .iter()
            .filter(|(t, _)| e.resources.colony.zone_of_tile.contains_key(t))
            .map(|(_, s)| u64::from(s.count))
            .sum()
    };
    let mut max_cell = 0;
    let used = run_until(&mut engine, 6_000, |e| {
        for (tile, stack) in &e.resources.colony.ground_items {
            if e.resources.colony.zone_of_tile.contains_key(tile) {
                max_cell = max_cell.max(stack.count);
            }
        }
        in_zone(e) == 38
    });
    assert!(
        used < 6_000,
        "only {} of 38 logs reached the zone",
        in_zone(&engine)
    );
    assert!(max_cell <= RULE.stack_max_logs, "a cell held {max_cell}");
    assert_eq!(ground_logs(&engine), 38, "no log lost or left outside");
    assert!(engine.resources.colony.reservations.is_empty());
    assert!(job_of(&engine, a).is_none() && job_of(&engine, b).is_none());
    // 38 logs over 5-log trips: at least 8 haul jobs finished.
    assert!(engine.resources.colony.tallies.jobs_done >= 8);
    watch.annotate(&engine, "event_stop", || serde_json::json!({"predicate": "all_38_logs_in_zone", "used": used, "limit": 6000, "logs_in_zone": in_zone(&engine)}));
    watch.finish(&engine, "all_38_logs_in_zone");
}

#[test]
fn jobs_deliver_and_build_hut() {
    let mut engine = job_world(30, 30);
    for x in 1..4 {
        worker(&mut engine, x, 1);
    }
    let watch = Watch::begin("jobs_deliver_and_build_hut", 0, "primary", &engine);
    order(&mut engine, zone(3, 20, 4, 20));
    drop_logs(&mut engine, (3, 20), 9);
    drop_logs(&mut engine, (10, 2), 6);
    order(
        &mut engine,
        Command::PlaceBlueprint {
            kind: BlueprintKind::Hut,
            x: 15,
            y: 15,
        },
    );
    watch.annotate(&engine, "fixture_setup", || serde_json::json!({"recipe": "jobs_deliver_and_build_hut", "founding_boundary": "post_worker_spawn_tick_zero", "world_seed": engine.resources.world_seed, "width": engine.resources.tile_grid.width, "height": engine.resources.tile_grid.height, "ground_logs": ground_logs(&engine), "log_ledger": log_ledger(&engine.world, &engine.resources)}));
    let used = run_until(&mut engine, 8_000, |e| {
        e.resources.building_kinds.get(&(16, 16)) == Some(&BuildingKind::Hut)
    });
    assert!(used < 8_000, "the hut was never finished");
    watch.annotate(&engine, "construction_stop", || serde_json::json!({"used": used, "limit": 8000, "hut_present": engine.resources.building_kinds.get(&(16, 16)) == Some(&BuildingKind::Hut)}));
    // 12 logs went into the hut; the other 3 are on the ground or being hauled.
    let (created, ground, carried, delivered, consumed) =
        log_ledger(&engine.world, &engine.resources);
    assert_eq!(
        (created, delivered, consumed, ground + carried),
        (15, 0, 12, 3)
    );
    assert!(
        engine.resources.sleep_tiles.contains_key(&(16, 16)),
        "the hut's centre is a sleep tile"
    );
    let occupied = engine.resources.colony.occupied.len();
    assert_eq!(occupied, 9, "the footprint stays covered by the hut");
    assert_eq!(engine.resources.colony.tallies.huts_built, 1);
    // The builder's job ends at the next tick's JobExecute (127), before which
    // its Site reservation is still held.
    tick_checked(&mut engine);
    // Only the leftover logs' haul may still hold a reservation (a cell or a stack).
    assert!(engine
        .resources
        .colony
        .reservations
        .keys()
        .all(|k| matches!(k, ReserveKey::Cell(_) | ReserveKey::Stack(_))));
    assert!(!engine
        .resources
        .colony
        .reservations
        .contains_key(&ReserveKey::Site((16, 16))));
    watch.annotate(
        &engine,
        "event_stop",
        || serde_json::json!({"predicate": "hut_completion_plus1", "used": used}),
    );
    watch.finish(&engine, "hut_completion_plus1");
}

#[test]
fn jobs_two_agents_one_tree() {
    let mut engine = job_world(24, 24);
    let a = worker(&mut engine, 5, 5);
    let b = worker(&mut engine, 15, 5);
    let watch = Watch::begin("jobs_two_agents_one_tree", 0, "primary", &engine);
    plant(&mut engine, (10, 12), 3);
    order(&mut engine, chop((10, 12)));
    watch.annotate(&engine, "fixture_setup", || serde_json::json!({"recipe": "jobs_two_agents_one_tree", "founding_boundary": "post_worker_spawn_tick_zero", "world_seed": engine.resources.world_seed, "width": engine.resources.tile_grid.width, "height": engine.resources.tile_grid.height, "ground_logs": ground_logs(&engine), "log_ledger": log_ledger(&engine.world, &engine.resources)}));
    tick_checked(&mut engine);
    let holders: Vec<_> = [a, b]
        .into_iter()
        .filter(|e| job_of(&engine, *e).is_some())
        .collect();
    assert_eq!(holders.len(), 1, "exactly one takes the reserved tree");
    // The lower AgentId (a) is served first.
    assert_eq!(holders[0], a);
    let owner_id = engine.world.get::<&Agent>(a).map(|x| x.id).expect("id");
    assert_eq!(
        engine
            .resources
            .colony
            .reservations
            .get(&ReserveKey::Tree((10, 12))),
        Some(&owner_id)
    );
    let mut b_ever = false;
    run_until(&mut engine, 2_000, |e| {
        b_ever |= job_of(e, b).is_some();
        !e.resources.colony.trees.contains_key(&(10, 12))
    });
    assert!(!b_ever, "the second worker never took the reserved tree");
    assert_eq!(engine.resources.colony.tallies.trees_felled, 1);
    watch.annotate(&engine, "event_stop", || serde_json::json!({"predicate": "reserved_tree_felled", "trees_felled": engine.resources.colony.tallies.trees_felled, "second_worker_took_job": b_ever}));
    watch.finish(&engine, "reserved_tree_felled");
}

#[test]
fn jobs_cancel_on_hunger() {
    // The live colony: a worker carrying logs far from camp gets hungry.
    let mut engine = sim_test::colony_engine(1);
    let watch = Watch::begin("jobs_cancel_on_hunger", 1, "primary", &engine);
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    // Logs 14 tiles east, a zone 14 tiles south-east: hauls far from the camp
    // (its meal radius is 8).
    // M3: tiles the camp can walk to (the river or a hill may lie east).
    sim_systems::runtime::nav::ensure(&mut engine.resources);
    let open = |engine: &sim_engine::SimEngine, t: (u32, u32)| {
        sim_systems::runtime::jobs::items::can_hold_items(&engine.resources, t)
            && engine.resources.nav.reachable(hearth, t)
    };
    let source = (0..30)
        .flat_map(|d| {
            [
                (hearth.0 + 14 + d, hearth.1),
                (hearth.0.saturating_sub(14 + d), hearth.1),
            ]
        })
        .find(|t| open(&engine, *t))
        .expect("a free tile east or west of the camp");
    let east = source.0 > hearth.0;
    watch.annotate(&engine, "before_log_gift", || serde_json::json!({"tile": source, "count": 20, "log_ledger": log_ledger(&engine.world, &engine.resources)}));
    drop_logs(&mut engine, source, 20);
    let cell = (0..30)
        .flat_map(|d| {
            (0..6).map(move |e| {
                let x = if east {
                    hearth.0 + 14 + d
                } else {
                    hearth.0.saturating_sub(14 + d)
                };
                (x, hearth.1 + 14 + e)
            })
        })
        .find(|t| open(&engine, *t))
        .expect("a free tile for the zone");
    order(&mut engine, zone(cell.0, cell.1, cell.0, cell.1));
    watch.annotate(&engine, "fixture_setup", || serde_json::json!({"recipe": "jobs_cancel_on_hunger", "source": source, "cell": cell, "log_ledger": log_ledger(&engine.world, &engine.resources)}));
    let mut carrier = None;
    for _ in 0..4_000 {
        tick_checked(&mut engine);
        // M3: a carrier of logs (a firewood gatherer also carries).
        let found = engine
            .world
            .query::<(&sim_core::colony::Carry, &sim_core::colony::Job)>()
            .iter()
            .find(|(_, (c, _))| c.kind == sim_core::colony::ItemKind::Log)
            .map(|(e, _)| e);
        if let Some(e) = found {
            let pos = tile_of(&engine, e);
            if pos.0.abs_diff(hearth.0).max(pos.1.abs_diff(hearth.1)) > 10 {
                carrier = Some(e);
                break;
            }
        }
    }
    let carrier = carrier.expect("someone carried logs far from camp");
    let id = engine
        .world
        .get::<&Agent>(carrier)
        .map(|a| a.id)
        .expect("id");
    let load = carry_of(&engine, carrier).expect("load").count;
    let ground_before = ground_logs(&engine);
    // Hunger interrupts for a reachable physical meal; an empty colony cannot
    // satisfy hunger by cancelling the very jobs that produce its food.
    let position = tile_of(&engine, carrier);
    let meal_tile = (0..engine.resources.tile_grid.height)
        .flat_map(|y| (0..engine.resources.tile_grid.width).map(move |x| (x, y)))
        .filter(|tile| position.0.abs_diff(tile.0).max(position.1.abs_diff(tile.1)) >= 3)
        .filter(|tile| {
            open(&engine, *tile)
                && engine.resources.nav.reachable(position, *tile)
                && !engine.resources.colony.ground_items.contains_key(tile)
        })
        .min_by_key(|tile| {
            (
                position.0.abs_diff(tile.0).max(position.1.abs_diff(tile.1)),
                tile.1,
                tile.0,
            )
        })
        .expect("reachable empty meal tile");
    watch.annotate(&engine, "before_hunger_intervention", || serde_json::json!({"predicate": "log_carrier_far_from_hearth", "carrier_id": id, "load": load, "position": position, "hearth": hearth, "meal_tile": meal_tile, "food_count": 10, "requested_hunger": 80.0, "meal_tile_empty": !engine.resources.colony.ground_items.contains_key(&meal_tile)}));
    let day = sim_systems::runtime::food::day(&engine.resources);
    sim_systems::runtime::food::items::create_ground(
        &mut engine.resources,
        meal_tile,
        sim_core::colony::ItemSpec::raw(ItemKind::from_name("porridge").expect("meal"), None),
        10,
        day,
        sim_systems::runtime::food::items::FoodSource::Debug,
    )
    .expect("tracked edible food");
    sim_systems::runtime::food::validate(&engine.world, &engine.resources)
        .expect("physical fixture accounting");
    if let Ok(mut hunger) = engine.world.get::<&mut Hunger>(carrier) {
        hunger.value = 80.0;
    }
    watch.annotate(&engine, "after_hunger_intervention", || serde_json::json!({"carrier_id": id, "actual_hunger": engine.world.get::<&Hunger>(carrier).ok().map(|h| h.value), "meal_tile": meal_tile, "direct_food_source": "Debug", "requested_food_count": 10, "meal_count": engine.resources.colony.ground_items.get(&meal_tile).map(|s| s.count), "meal_is_porridge": engine.resources.colony.ground_items.get(&meal_tile).is_some_and(|s| s.kind == ItemKind::from_name("porridge").expect("meal"))}));
    let cancels_before = engine
        .resources
        .colony
        .tallies
        .cancels_of(CancelReason::Needs);
    tick_checked(&mut engine);
    assert!(
        job_of(&engine, carrier).is_none(),
        "the job is dropped within one tick"
    );
    assert!(carry_of(&engine, carrier).is_none());
    assert_eq!(
        ground_logs(&engine),
        ground_before + u64::from(load),
        "the load lands on the ground"
    );
    assert!(
        engine
            .resources
            .colony
            .reservations
            .values()
            .all(|holder| *holder != id),
        "its reservations are released"
    );
    assert_eq!(
        engine
            .resources
            .colony
            .tallies
            .cancels_of(CancelReason::Needs),
        cancels_before + 1
    );
    assert!(
        matches!(
            state_of(&engine, carrier),
            AgentState::Seeking {
                target: TargetKind::Food | TargetKind::GatherFood
            }
        ),
        "it went for food: {:?}",
        state_of(&engine, carrier)
    );
    watch.annotate(&engine, "event_stop", || serde_json::json!({"predicate": "needs_cancel_within_one_tick", "carrier_id": id, "cancel_before": cancels_before, "cancel_after": engine.resources.colony.tallies.cancels_of(CancelReason::Needs)}));
    watch.finish(&engine, "hunger_intervention_plus1");
}

#[test]
fn jobs_cancel_area_mid_delivery() {
    let mut engine = job_world(30, 30);
    let w = worker(&mut engine, 2, 2);
    let watch = Watch::begin("jobs_cancel_area_mid_delivery", 0, "primary", &engine);
    drop_logs(&mut engine, (3, 3), 10);
    order(
        &mut engine,
        Command::PlaceBlueprint {
            kind: BlueprintKind::Hut,
            x: 20,
            y: 20,
        },
    );
    watch.annotate(&engine, "fixture_setup", || serde_json::json!({"recipe": "jobs_cancel_area_mid_delivery", "founding_boundary": "post_worker_spawn_tick_zero", "world_seed": engine.resources.world_seed, "width": engine.resources.tile_grid.width, "height": engine.resources.tile_grid.height, "ground_logs": ground_logs(&engine), "log_ledger": log_ledger(&engine.world, &engine.resources)}));
    let used = run_until(&mut engine, 2_000, |e| {
        carry_of(e, w).is_some()
            && matches!(job_of(e, w).map(|j| j.kind), Some(JobKind::Deliver { .. }))
    });
    assert!(used < 2_000, "the worker never picked logs up");
    let carried = carry_of(&engine, w).expect("load").count;
    assert_eq!(carried, RULE.carry_logs);
    watch.annotate(&engine, "before_cancel", || serde_json::json!({"used": used, "limit": 2000, "carried": carried, "worker_id": engine.world.get::<&Agent>(w).expect("worker").id}));
    let result = submit(
        &mut engine,
        Command::CancelArea {
            x0: 20,
            y0: 20,
            x1: 22,
            y1: 22,
        },
    );
    assert_eq!((result.accepted, result.count), (true, 1));
    sim_systems::runtime::jobs::invariants::check(&engine.world, &engine.resources)
        .expect("invariants");
    assert!(job_of(&engine, w).is_none());
    assert!(carry_of(&engine, w).is_none());
    assert_eq!(state_of(&engine, w), AgentState::Idle);
    assert_eq!(
        log_ledger(&engine.world, &engine.resources),
        (10, 10, 0, 0, 0)
    );
    assert_eq!(
        engine
            .resources
            .colony
            .tallies
            .cancels_of(CancelReason::Command),
        1
    );
    // Nothing is left to deliver to: the worker stays without a job.
    for _ in 0..50 {
        tick_checked(&mut engine);
    }
    assert!(job_of(&engine, w).is_none_or(|j| !matches!(j.kind, JobKind::Deliver { .. })));
    watch.annotate(&engine, "event_stop", || serde_json::json!({"predicate": "cancel_delivery_plus50", "used": used, "cancel_count": engine.resources.colony.tallies.cancels_of(CancelReason::Command)}));
    watch.finish(&engine, "cancel_delivery_plus50");
}

#[test]
fn jobs_stuck_cancel() {
    let mut engine = job_world(24, 24);
    let w = worker(&mut engine, 2, 2);
    let watch = Watch::begin("jobs_stuck_cancel", 0, "primary", &engine);
    // Without its movement stream the worker never moves: no progress.
    let _ = engine.world.remove_one::<MovementRng>(w);
    plant(&mut engine, (20, 20), 3);
    order(&mut engine, chop((20, 20)));
    watch.annotate(&engine, "fixture_setup", || serde_json::json!({"recipe": "jobs_stuck_cancel", "founding_boundary": "post_worker_spawn_tick_zero", "world_seed": engine.resources.world_seed, "width": engine.resources.tile_grid.width, "height": engine.resources.tile_grid.height, "ground_logs": ground_logs(&engine), "log_ledger": log_ledger(&engine.world, &engine.resources)}));
    watch.annotate(
        &engine,
        "removed_movement",
        || serde_json::json!({"movement_rng_absent": engine.world.get::<&MovementRng>(w).is_err()}),
    );
    tick_checked(&mut engine);
    let job = job_of(&engine, w).expect("the worker took the tree");
    for _ in 0..RULE.stuck_ticks - 1 {
        tick_checked(&mut engine);
    }
    assert!(job_of(&engine, w).is_some(), "not before stuck_ticks");
    tick_checked(&mut engine);
    // The cancel frees the tree; the same tick's assign runs before execute,
    // so the worker is job-less now and the stuck cancel is counted once.
    assert_eq!(
        engine
            .resources
            .colony
            .tallies
            .cancels_of(CancelReason::Stuck),
        1
    );
    assert!(job.taken_at < engine.current_tick());
    assert!(engine.resources.colony.trees.contains_key(&(20, 20)));
    watch.annotate(&engine, "event_stop", || serde_json::json!({"predicate": "stuck_cancel", "stuck_ticks": RULE.stuck_ticks, "cancel_count": engine.resources.colony.tallies.cancels_of(CancelReason::Stuck), "tree_present": engine.resources.colony.trees.contains_key(&(20, 20))}));
    watch.finish(&engine, "stuck_cancel");
}

#[test]
fn jobs_children_never_work() {
    let mut engine = job_world(24, 24);
    let kid = worker(&mut engine, 5, 5);
    let watch = Watch::begin("jobs_children_never_work", 0, "primary", &engine);
    let _ = engine.world.insert_one(kid, Child { born_tick: 0 });
    plant(&mut engine, (8, 8), 3);
    order(&mut engine, chop((8, 8)));
    drop_logs(&mut engine, (6, 6), 5);
    order(&mut engine, zone(12, 12, 12, 12));
    watch.annotate(&engine, "fixture_setup", || serde_json::json!({"recipe": "jobs_children_never_work", "founding_boundary": "post_worker_spawn_tick_zero", "world_seed": engine.resources.world_seed, "width": engine.resources.tile_grid.width, "height": engine.resources.tile_grid.height, "ground_logs": ground_logs(&engine), "log_ledger": log_ledger(&engine.world, &engine.resources)}));
    watch.annotate(
        &engine,
        "child_inserted",
        || serde_json::json!({"child_component_present": engine.world.get::<&Child>(kid).is_ok()}),
    );
    for _ in 0..600 {
        tick_checked(&mut engine);
        assert!(job_of(&engine, kid).is_none(), "a child took a job");
    }
    assert_eq!(engine.resources.colony.tallies.jobs_taken, 0);
    // The same world with an adult does take work (the test can fail).
    watch.annotate(
        &engine,
        "before_adult_spawn",
        || serde_json::json!({"direct_spawn": true, "position": [5, 6]}),
    );
    let adult = worker(&mut engine, 5, 6);
    watch.annotate(&engine, "after_adult_spawn", || serde_json::json!({"direct_spawn": true, "position": [5, 6], "adult_id": engine.world.get::<&Agent>(adult).expect("adult").id}));

    tick_checked(&mut engine);
    assert!(job_of(&engine, adult).is_some());
    watch.annotate(&engine, "event_stop", || serde_json::json!({"predicate": "child_600_then_adult1", "adult_has_job": job_of(&engine, adult).is_some()}));
    watch.finish(&engine, "child_600_then_adult1");
}

#[test]
fn hut_sleeps_four() {
    let mut engine = sim_test::colony_engine(2);
    let watch = Watch::begin("hut_sleeps_four", 2, "gifted_hut", &engine);
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    let spot = (3..24)
        .flat_map(|dy| (0..10).map(move |dx| (hearth.0 + dx, hearth.1 + dy)))
        .find(|&(x, y)| check_blueprint(&engine.resources, BlueprintKind::Hut, x, y).is_ok())
        .expect("a hut spot");
    let pile = (2..12)
        .map(|d| (hearth.0.saturating_sub(d), hearth.1))
        .find(|t| sim_systems::runtime::jobs::items::can_hold_items(&engine.resources, *t))
        .expect("a pile tile");
    watch.annotate(&engine, "before_log_gift", || serde_json::json!({
        "intervention": "direct_ledger_create_and_ground_stack_insert", "item": "log", "count": 12, "tile": pile,
        "ground_logs": ground_logs(&engine), "log_ledger": log_ledger(&engine.world, &engine.resources)}));
    drop_logs(&mut engine, pile, 12);
    watch.annotate(&engine, "after_log_gift", || {
        serde_json::json!({
        "ground_logs": ground_logs(&engine), "log_ledger": log_ledger(&engine.world, &engine.resources),
        "requested_count": 12, "tile": pile})
    });
    order(
        &mut engine,
        Command::PlaceBlueprint {
            kind: BlueprintKind::Hut,
            x: spot.0,
            y: spot.1,
        },
    );
    watch.annotate(&engine, "accepted_direct_blueprint", || serde_json::json!({
        "intervention": "direct_apply_command", "accepted_assertion_completed": true,
        "command": sim_test::probe::colony::command_json(engine.current_tick(), &Command::PlaceBlueprint {
            kind: BlueprintKind::Hut, x: spot.0, y: spot.1})}));
    let centre = (spot.0 + 1, spot.1 + 1);
    let used = run_until(&mut engine, 3 * 1_440, |e| {
        e.resources.building_kinds.get(&centre) == Some(&BuildingKind::Hut)
    });
    assert!(used < 3 * 1_440, "the hut was not finished in 3 days");
    assert!(
        engine.resources.building_owner.contains_key(&centre),
        "the colony owns the hut"
    );
    watch.annotate(&engine, "hut_completion", || {
        serde_json::json!({
        "actual_completed_ticks": used, "limit_ticks": 3 * 1_440, "centre": centre,
        "owned_hut_assertion_completed": true})
    });
    // Beds are assigned at 20:00 (display) or when the dwellings change.
    for _ in 0..1_440 {
        tick_checked(&mut engine);
    }
    let sleepers = engine
        .resources
        .bed_of
        .values()
        .filter(|b| **b == centre)
        .count();
    assert_eq!(sleepers, RULE.hut_beds as usize, "four beds at the hut");
    watch.finish(&engine, "actual_hut_completion_plus_1440");
}
