//! M3 Part 2: walls, doors, floors and digging (ADR 0022) — built from
//! delivered items by workers standing beside them, refused where they must
//! be, cleared of items first, taken apart for half their materials, and hill
//! tiles dug for stone; the nav map follows every change.

mod colony_support;

use colony_support::{drop_items, job_world_m2, run_until, tick_checked, worker};
use sim_core::colony::{BlueprintKind, Command, ItemKind, ItemSpec, RejectReason};
use sim_core::rules::ColonyRule;
use sim_core::TerrainType;
use sim_engine::SimEngine;
use sim_systems::commands::submit;
use sim_systems::runtime::jobs::invariants::ledger_of;
use sim_systems::runtime::nav;

/// A plain `w × h` M3 job world (the economy and built space on).
fn world(w: u32, h: u32) -> SimEngine {
    let mut engine = job_world_m2(w, h);
    engine.resources.rules.colony = Some(ColonyRule::M3);
    nav::ensure(&mut engine.resources);
    engine
}

fn stockpile(engine: &mut SimEngine, x0: u32, y0: u32, x1: u32, y1: u32) {
    let r = submit(engine, Command::AddStockpile { x0, y0, x1, y1 });
    assert!(r.accepted, "{r:?}");
}

fn logs(engine: &mut SimEngine, tile: (u32, u32), n: u16) {
    drop_items(engine, tile, ItemSpec::raw(ItemKind::Log, None), n);
}

fn is_wall(engine: &SimEngine, t: (u32, u32)) -> bool {
    let g = &engine.resources.tile_grid;
    g.wall_material[g.idx(t.0, t.1)].is_some()
}

fn is_door(engine: &SimEngine, t: (u32, u32)) -> bool {
    let g = &engine.resources.tile_grid;
    g.is_door(g.idx(t.0, t.1))
}

#[test]
fn a_walled_square_with_a_door_is_built_from_stockpiled_logs() {
    let mut engine = world(30, 30);
    for k in 0..3 {
        worker(&mut engine, 3 + k, 3);
    }
    stockpile(&mut engine, 2, 6, 4, 8);
    logs(&mut engine, (2, 6), 25);
    logs(&mut engine, (3, 6), 25);
    // The door first, then the hollow 5 × 5 around it: the door's tile is taken (15 walls).
    let v_start = engine.resources.nav.version;
    let door = submit(
        &mut engine,
        Command::PlaceBlueprint {
            kind: BlueprintKind::Door,
            x: 12,
            y: 14,
        },
    );
    assert_eq!(door.count, 1, "{door:?}");
    let walls = submit(
        &mut engine,
        Command::PlaceRect {
            kind: BlueprintKind::LogWall,
            x0: 10,
            y0: 10,
            x1: 14,
            y1: 14,
            hollow: true,
        },
    );
    assert_eq!(walls.count, 15, "{walls:?}");
    // Wall sites are impassable at once; the door site is not.
    assert!(!nav::passable(&engine.resources, (10, 10)));
    assert!(nav::passable(&engine.resources, (12, 14)));
    // Placing changed the map (wall sites impassable, the door site 12) ...
    let v0 = engine.resources.nav.version;
    assert_eq!(v0, v_start + 16);
    let border: Vec<(u32, u32)> = (10..=14)
        .flat_map(|y| (10..=14).map(move |x| (x, y)))
        .filter(|t| t.0 == 10 || t.0 == 14 || t.1 == 10 || t.1 == 14)
        .collect();
    let used = run_until(&mut engine, 12_000, |e| {
        border.iter().all(|t| is_wall(e, *t)) && e.resources.colony.space_sites.is_empty()
    });
    assert!(used < 12_000, "not built in 12,000 ticks");
    assert!(is_door(&engine, (12, 14)));
    assert_eq!(
        border.iter().filter(|t| is_door(&engine, **t)).count(),
        1,
        "one door"
    );
    // Ledger: 15 walls × 2 logs + 1 door × 1 log consumed, nothing lost.
    let l = ledger_of(&engine.world, &engine.resources, ItemKind::Log);
    assert_eq!(l.created, 50);
    assert_eq!(l.consumed, 31);
    assert_eq!(l.ground + l.carried, 19);
    assert_eq!(engine.resources.colony.tallies.space_built, 16);
    // The nav map: walls impassable, the door passable, inside and outside one region.
    // ... finishing did not: a wall costs what its site cost.
    assert_eq!(engine.resources.nav.version, v0);
    assert!(!nav::passable(&engine.resources, (11, 10)));
    assert!(nav::passable(&engine.resources, (12, 14)));
    assert!(engine.resources.nav.reachable((12, 12), (20, 20)));
    let p = engine
        .resources
        .nav
        .path((12, 12), (12, 20))
        .expect("out by the door");
    assert!(p.contains(&(12, 14)), "{p:?}");
    assert_eq!(*engine.resources.nav.map(), nav::build(&engine.resources));
}

#[test]
fn placements_are_refused_where_they_must_be() {
    let mut engine = world(20, 20);
    let w = worker(&mut engine, 5, 5);
    let _ = w;
    let i = engine.resources.tile_grid.idx(8, 8);
    engine.resources.tile_grid.terrain_type[i] = TerrainType::Hill;
    nav::refresh(&mut engine.resources, (8, 8));
    let place =
        |engine: &mut SimEngine, kind, x, y| submit(engine, Command::PlaceBlueprint { kind, x, y });
    assert_eq!(
        place(&mut engine, BlueprintKind::LogWall, 5, 5).reason,
        Some(RejectReason::Person)
    );
    assert!(
        place(&mut engine, BlueprintKind::Door, 5, 5).accepted,
        "a door may go under a person"
    );
    assert_eq!(
        place(&mut engine, BlueprintKind::LogWall, 8, 8).reason,
        Some(RejectReason::Hill)
    );
    assert!(place(&mut engine, BlueprintKind::LogWall, 2, 2).accepted);
    assert_eq!(
        place(&mut engine, BlueprintKind::PackedFloor, 2, 2).reason,
        Some(RejectReason::Blocked),
        "a site is already there"
    );
    // A line through a refused tile places the rest.
    let line = submit(
        &mut engine,
        Command::PlaceLine {
            kind: BlueprintKind::LogWall,
            x0: 1,
            y0: 2,
            x1: 6,
            y1: 2,
        },
    );
    assert_eq!(line.count, 5, "{line:?}");
    // Digging: only undug hill, never on the map edge.
    let e = engine.resources.tile_grid.idx(0, 8);
    engine.resources.tile_grid.terrain_type[e] = TerrainType::Hill;
    assert_eq!(
        submit(
            &mut engine,
            Command::DesignateDig {
                x0: 0,
                y0: 8,
                x1: 0,
                y1: 8
            }
        )
        .reason,
        Some(RejectReason::Edge)
    );
    assert_eq!(
        submit(
            &mut engine,
            Command::DesignateDig {
                x0: 7,
                y0: 7,
                x1: 9,
                y1: 9
            }
        )
        .count,
        1
    );
    // Without the space rule (M2) no space order is taken.
    engine.resources.rules.colony = Some(ColonyRule::M2);
    assert_eq!(
        place(&mut engine, BlueprintKind::LogWall, 3, 3).reason,
        Some(RejectReason::NotYet)
    );
}

#[test]
fn a_wall_site_on_items_has_them_hauled_off_first() {
    let mut engine = world(30, 30);
    worker(&mut engine, 5, 5);
    worker(&mut engine, 6, 5);
    stockpile(&mut engine, 2, 10, 4, 12);
    logs(&mut engine, (2, 10), 10);
    // Three stones lie where the wall goes.
    drop_items(
        &mut engine,
        (15, 15),
        ItemSpec::raw(ItemKind::Stone, None),
        3,
    );
    let r = submit(
        &mut engine,
        Command::PlaceBlueprint {
            kind: BlueprintKind::LogWall,
            x: 15,
            y: 15,
        },
    );
    assert!(r.accepted, "{r:?}");
    assert!(engine.resources.colony.ground_items.contains_key(&(15, 15)));
    let used = run_until(&mut engine, 4_000, |e| is_wall(e, (15, 15)));
    assert!(used < 4_000, "the wall was not built");
    let c = &engine.resources.colony;
    let clears: u64 = c
        .tallies
        .taken_by
        .iter()
        .filter(|((_, k), _)| *k == 11)
        .map(|(_, n)| n)
        .sum();
    assert!(clears >= 1, "a haul-away job was taken");
    assert!(!c.ground_items.contains_key(&(15, 15)));
    let stone = ledger_of(&engine.world, &engine.resources, ItemKind::Stone);
    assert_eq!((stone.created, stone.ground + stone.carried), (3, 3));
}

#[test]
fn deconstruction_returns_half_the_materials() {
    let mut engine = world(30, 30);
    worker(&mut engine, 5, 5);
    stockpile(&mut engine, 2, 10, 4, 12);
    logs(&mut engine, (2, 10), 10);
    for (kind, x) in [(BlueprintKind::LogWall, 15), (BlueprintKind::Door, 17)] {
        assert!(submit(&mut engine, Command::PlaceBlueprint { kind, x, y: 15 }).accepted);
    }
    let used = run_until(&mut engine, 4_000, |e| {
        is_wall(e, (15, 15)) && is_door(e, (17, 15))
    });
    assert!(used < 4_000);
    let before = ledger_of(&engine.world, &engine.resources, ItemKind::Log);
    let r = submit(
        &mut engine,
        Command::DesignateDeconstruct {
            x0: 15,
            y0: 15,
            x1: 17,
            y1: 15,
        },
    );
    assert_eq!(r.count, 2, "{r:?}");
    let used = run_until(&mut engine, 2_000, |e| {
        !is_wall(e, (15, 15)) && !is_wall(e, (17, 15))
    });
    assert!(used < 2_000, "not taken apart");
    let after = ledger_of(&engine.world, &engine.resources, ItemKind::Log);
    // A log wall (2 logs) gives 1 back, a door (1 log) none.
    assert_eq!(after.created, before.created + 1);
    assert_eq!(engine.resources.colony.tallies.space_removed, 2);
    assert!(nav::passable(&engine.resources, (15, 15)));
    assert!(engine.resources.colony.designations.is_empty());
    for _ in 0..3 {
        tick_checked(&mut engine);
    }
}

#[test]
fn digging_a_pocket_into_a_hill_yields_stone_and_opens_it() {
    let mut engine = world(30, 30);
    worker(&mut engine, 5, 15);
    worker(&mut engine, 6, 15);
    // A hill block x 12..=20, y 10..=20; dig a 3 × 3 pocket at its west edge.
    for y in 10..=20u32 {
        for x in 12..=20u32 {
            let i = engine.resources.tile_grid.idx(x, y);
            engine.resources.tile_grid.terrain_type[i] = TerrainType::Hill;
        }
    }
    engine.resources.nav.install(nav::build(&engine.resources));
    assert!(!engine.resources.nav.reachable((5, 15), (14, 15)));
    let r = submit(
        &mut engine,
        Command::DesignateDig {
            x0: 12,
            y0: 14,
            x1: 14,
            y1: 16,
        },
    );
    assert_eq!(r.count, 9, "{r:?}");
    let pocket: Vec<(u32, u32)> = (14..=16)
        .flat_map(|y| (12..=14).map(move |x| (x, y)))
        .collect();
    let dug = |e: &SimEngine, t: (u32, u32)| {
        let g = &e.resources.tile_grid;
        g.is_dug(g.idx(t.0, t.1))
    };
    let used = run_until(&mut engine, 12_000, |e| pocket.iter().all(|t| dug(e, *t)));
    assert!(used < 12_000, "pocket not dug");
    for t in &pocket {
        assert!(nav::passable(&engine.resources, *t));
        let g = &engine.resources.tile_grid;
        assert_eq!(
            g.terrain_type[g.idx(t.0, t.1)],
            TerrainType::Hill,
            "stays a hill"
        );
        assert!(g.floor_material[g.idx(t.0, t.1)].is_some(), "rock floor");
    }
    assert!(engine.resources.nav.reachable((5, 15), (14, 15)));
    let stone = ledger_of(&engine.world, &engine.resources, ItemKind::Stone);
    assert_eq!(stone.created, 27, "3 stone per tile");
    assert_eq!(engine.resources.colony.tallies.tiles_dug, 9);
    assert_eq!(*engine.resources.nav.map(), nav::build(&engine.resources));
    // Mining trained someone.
    let mined = engine
        .world
        .query::<&sim_core::colony::Skills>()
        .iter()
        .map(|(_, s)| s.get(sim_core::colony::SkillKind::Mining).xp)
        .max()
        .unwrap_or(0);
    assert!(mined > 0, "mining XP");
}

#[test]
fn someone_about_to_drink_gets_no_new_job() {
    let mut engine = world(30, 30);
    let calm = worker(&mut engine, 5, 5);
    let thirsty = worker(&mut engine, 6, 5);
    for (e, value) in [(calm, 10.0), (thirsty, 55.0)] {
        let _ = engine
            .world
            .insert_one(e, sim_core::components::Thirst::new(value, 0.0));
    }
    colony_support::plant(&mut engine, (20, 5), 4);
    colony_support::plant(&mut engine, (20, 8), 4);
    assert!(
        submit(
            &mut engine,
            Command::DesignateChopTiles {
                tiles: vec![(20, 5), (20, 8)]
            }
        )
        .accepted
    );
    tick_checked(&mut engine);
    let has_job = |engine: &SimEngine, e| colony_support::job_of(engine, e).is_some();
    assert!(has_job(&engine, calm), "the calm worker takes a tree");
    assert!(
        !has_job(&engine, thirsty),
        "the thirsty one (55 >= 50) is left to drink"
    );
    // Under M2 rules the old behaviour stands: both take one.
    engine.resources.rules.colony = Some(ColonyRule::M2);
    tick_checked(&mut engine);
    assert!(has_job(&engine, thirsty));
}

#[test]
fn review_fixes_beds_lines_and_open_goals() {
    // Code review m-3: a building never goes over a finished bedroll.
    let mut engine = world(30, 30);
    worker(&mut engine, 3, 3);
    logs(&mut engine, (4, 4), 5);
    let r = submit(
        &mut engine,
        Command::PlaceBlueprint {
            kind: BlueprintKind::Bedroll,
            x: 10,
            y: 10,
        },
    );
    assert!(r.accepted, "{r:?}");
    run_until(&mut engine, 600, |e| {
        e.resources.colony.beds.contains_key(&(10, 10))
    });
    assert!(engine.resources.colony.beds.contains_key(&(10, 10)));
    let over = submit(
        &mut engine,
        Command::PlaceBlueprint {
            kind: BlueprintKind::KnappingSpot,
            x: 10,
            y: 10,
        },
    );
    assert!(!over.accepted, "{over:?}");
    // Code review m-5: a line with an end off the map is refused whole (it
    // used to rasterise as many tiles as the coordinates).
    let off = submit(
        &mut engine,
        Command::PlaceLine {
            kind: BlueprintKind::LogWall,
            x0: 2,
            y0: 20,
            x1: u32::MAX,
            y1: 20,
        },
    );
    assert_eq!(off.reason, Some(RejectReason::OutOfMap), "{off:?}");
}

#[test]
fn a_hauler_whose_goal_opens_walks_onto_it() {
    // Code review m-1: a Haul from a stack on a wall site (impassable: taken
    // from beside); the site is cancelled on the way, so the goal opens. The
    // kept path ended beside the goal and the hauler stood still for 240 ticks
    // until a Stuck cancel; now it plans again and finishes.
    use sim_core::colony::WorkType;
    use sim_core::components::{Agent, Position};
    let mut e = world(30, 30);
    let w = worker(&mut e, 2, 15);
    let id = e.world.get::<&Agent>(w).map(|a| a.id).expect("agent");
    let r = submit(
        &mut e,
        Command::SetWorkPriority {
            agent: id,
            work: WorkType::Build,
            level: 0,
        },
    );
    assert!(r.accepted, "{r:?}");
    stockpile(&mut e, 2, 2, 3, 3);
    logs(&mut e, (20, 15), 3);
    let r = submit(
        &mut e,
        Command::PlaceBlueprint {
            kind: BlueprintKind::LogWall,
            x: 20,
            y: 15,
        },
    );
    assert!(r.accepted, "{r:?}");
    let near = run_until(&mut e, 200, |e| {
        e.world
            .get::<&Position>(w)
            .is_ok_and(|p| p.x.abs_diff(20) <= 4 && p.y.abs_diff(15) <= 4)
    });
    assert!(near < 200, "the hauler walks toward the stack");
    let r = submit(
        &mut e,
        Command::CancelArea {
            x0: 20,
            y0: 15,
            x1: 20,
            y1: 15,
        },
    );
    assert!(r.accepted, "{r:?}");
    let stocked = |e: &SimEngine| {
        (2..=3)
            .flat_map(|y| (2..=3).map(move |x| (x, y)))
            .filter_map(|t| e.resources.colony.ground_items.get(&t))
            .map(|s| s.count)
            .sum::<u16>()
    };
    // 3 logs over ~18 tiles at 3 ticks a step: well under 200 ticks; the old
    // freeze alone lasted 240.
    run_until(&mut e, 200, |e| stocked(e) == 3);
    assert_eq!(stocked(&e), 3);
    let stuck = e
        .resources
        .colony
        .tallies
        .cancels
        .get(&sim_core::colony::CancelReason::Stuck)
        .copied()
        .unwrap_or(0);
    assert_eq!(stuck, 0);
}

#[test]
fn a_command_right_after_a_load_spills_like_an_uninterrupted_run() {
    // Code review M-1: the nav map is derived and was rebuilt only at the
    // first tick after a load, so a command before it spilled items without
    // the region rule (a 25-log stack landed across a wall).
    let build = || {
        let mut e = world(16, 16);
        let r = submit(
            &mut e,
            Command::PlaceLine {
                kind: BlueprintKind::LogWall,
                x0: 6,
                y0: 0,
                x1: 6,
                y1: 15,
            },
        );
        assert_eq!(r.count, 16, "{r:?}");
        let max = sim_systems::runtime::jobs::items::stack_max(&e.resources, ItemKind::Log);
        for y in 0..=15u32 {
            for x in 0..=5u32 {
                if (x, y) != (0, 15) {
                    logs(&mut e, (x, y), max);
                }
            }
        }
        e
    };
    let mut a = build();
    let mut b = build();
    let bytes = sim_systems::save::save(&mut b).expect("save");
    // The fixture's system list (job_world_m2), as the load must match it.
    let mut b = sim_systems::save::load_with(&bytes, |e| {
        use sim_systems::runtime::jobs::{BillSystem, JobAssignSystem, JobExecuteSystem};
        e.register_system(Box::new(
            sim_systems::runtime::agent::AgentMovementSystem::new(),
        ));
        e.register_system(Box::new(JobAssignSystem::new()));
        e.register_system(Box::new(JobExecuteSystem::new()));
        e.register_system(Box::new(
            sim_systems::runtime::construction::ConstructionSystem::new(),
        ));
        e.register_system(Box::new(BillSystem::new()));
    })
    .expect("load");
    assert!(b.resources.nav.is_built(), "the map is rebuilt on load");
    let place = Command::PlaceBlueprint {
        kind: BlueprintKind::KnappingSpot,
        x: 3,
        y: 8,
    };
    assert!(submit(&mut a, place.clone()).accepted);
    assert!(submit(&mut b, place).accepted);
    let landed = |e: &SimEngine| -> Vec<((u32, u32), u16)> {
        e.resources
            .colony
            .ground_items
            .iter()
            .filter(|(t, _)| t.0 > 6 || **t == (0, 15))
            .map(|(t, s)| (*t, s.count))
            .collect()
    };
    assert_eq!(landed(&a), vec![((0, 15), 25)]);
    assert_eq!(landed(&a), landed(&b));
}
