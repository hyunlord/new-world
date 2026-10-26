//! M1 Part 1: player orders as recorded commands (goal 6.3, 6.4, 6.6; ADR 0012).
//!
//! One test per reject reason, the effect of each command, the item ledger
//! after commands alone, and replay: a run's command log applied to a fresh
//! world reproduces it exactly.

mod colony_support;

use colony_support::*;
use sim_core::colony::{BlueprintKind, ColonySite, Command, ItemKind, MaterialNeeds, RejectReason};
use sim_core::components::ConstructionSite;
use sim_core::TerrainType;
use sim_engine::SimEngine;
use sim_systems::commands::{apply_command, submit};
use sim_systems::runtime::jobs::invariants::log_ledger;

fn rejected(engine: &mut SimEngine, cmd: Command) -> RejectReason {
    let result = apply_command(&mut engine.world, &mut engine.resources, cmd);
    assert!(!result.accepted, "expected a rejection, got {result:?}");
    assert_eq!(result.count, 0);
    result.reason.expect("a rejection names its reason")
}

fn hut(x: u32, y: u32) -> Command {
    Command::PlaceBlueprint {
        kind: BlueprintKind::Hut,
        x,
        y,
    }
}

#[test]
fn commands_reject_without_the_colony_rule() {
    let mut engine = sim_test::baseline_engine(0);
    let before = engine.resources.colony.command_log.len();
    assert_eq!(
        rejected(
            &mut engine,
            Command::DesignateChop {
                x0: 0,
                y0: 0,
                x1: 63,
                y1: 63
            }
        ),
        RejectReason::NoColony
    );
    assert_eq!(rejected(&mut engine, hut(10, 10)), RejectReason::NoColony);
    assert!(engine.resources.colony.designations.is_empty());
    assert_eq!(engine.resources.colony.command_log.len(), before);
    assert_eq!(engine.resources.colony.tallies.commands_rejected, 2);
}

#[test]
fn commands_reject_out_of_map() {
    let mut engine = job_world(16, 16);
    plant(&mut engine, (3, 3), 4);
    assert_eq!(
        rejected(
            &mut engine,
            Command::DesignateChop {
                x0: 20,
                y0: 0,
                x1: 30,
                y1: 4
            }
        ),
        RejectReason::OutOfMap
    );
    assert_eq!(
        rejected(
            &mut engine,
            Command::DesignateChopTiles {
                tiles: vec![(16, 1), (99, 99)]
            }
        ),
        RejectReason::OutOfMap
    );
    // A hut whose 3×3 footprint ends past the edge (14..=16).
    assert_eq!(rejected(&mut engine, hut(14, 2)), RejectReason::OutOfMap);
}

#[test]
fn commands_reject_a_hut_on_water() {
    let mut engine = job_world(16, 16);
    let i = engine.resources.tile_grid.idx(6, 6);
    engine.resources.tile_grid.terrain_type[i] = TerrainType::River;
    assert_eq!(rejected(&mut engine, hut(5, 5)), RejectReason::Water);
    assert!(engine.resources.colony.occupied.is_empty());
}

#[test]
fn commands_reject_a_blocked_hut() {
    let mut engine = job_world(20, 20);
    // A tree, a zone tile, a mountain and a reservation each block a footprint.
    plant(&mut engine, (2, 2), 3);
    assert_eq!(rejected(&mut engine, hut(1, 1)), RejectReason::Blocked);
    let zone = submit(
        &mut engine,
        Command::AddStockpile {
            x0: 10,
            y0: 1,
            x1: 10,
            y1: 1,
        },
    );
    assert!(zone.accepted);
    assert_eq!(rejected(&mut engine, hut(9, 0)), RejectReason::Blocked);
    let i = engine.resources.tile_grid.idx(16, 16);
    engine.resources.tile_grid.terrain_type[i] = TerrainType::Mountain;
    assert_eq!(rejected(&mut engine, hut(15, 15)), RejectReason::Blocked);
    engine
        .resources
        .colony
        .reservations
        .insert(sim_core::colony::ReserveKey::Stack((6, 11)), 1);
    assert_eq!(rejected(&mut engine, hut(5, 10)), RejectReason::Blocked);
    // A placed site blocks an overlapping one.
    assert!(apply_command(&mut engine.world, &mut engine.resources, hut(4, 4)).accepted);
    assert_eq!(rejected(&mut engine, hut(5, 5)), RejectReason::Blocked);
}

#[test]
fn commands_reject_when_nothing_changes() {
    let mut engine = job_world(16, 16);
    let chop = Command::DesignateChop {
        x0: 0,
        y0: 0,
        x1: 5,
        y1: 5,
    };
    assert_eq!(rejected(&mut engine, chop.clone()), RejectReason::Nothing);
    plant(&mut engine, (1, 1), 4);
    assert!(apply_command(&mut engine.world, &mut engine.resources, chop.clone()).accepted);
    // Designating the same tree again changes nothing.
    assert_eq!(rejected(&mut engine, chop), RejectReason::Nothing);
    assert_eq!(
        rejected(
            &mut engine,
            Command::RemoveZoneArea {
                x0: 0,
                y0: 0,
                x1: 15,
                y1: 15
            }
        ),
        RejectReason::Nothing
    );
    assert_eq!(
        rejected(
            &mut engine,
            Command::CancelArea {
                x0: 8,
                y0: 8,
                x1: 9,
                y1: 9
            }
        ),
        RejectReason::Nothing
    );
}

#[test]
fn commands_reject_drafting_in_m1() {
    let mut engine = job_world(16, 16);
    let agent = worker(&mut engine, 2, 2);
    let id = engine
        .world
        .get::<&sim_core::components::Agent>(agent)
        .map(|a| a.id)
        .expect("agent");
    assert_eq!(
        rejected(
            &mut engine,
            Command::SetDrafted {
                agent: id,
                drafted: true
            }
        ),
        RejectReason::NotYet
    );
}

#[test]
fn commands_designate_zone_place_and_cancel() {
    let mut engine = job_world(24, 24);
    for (x, y) in [(1, 1), (2, 1), (5, 5)] {
        plant(&mut engine, (x, y), 4);
    }
    let r = apply_command(
        &mut engine.world,
        &mut engine.resources,
        Command::DesignateChop {
            x0: 3,
            y0: 3,
            x1: 0,
            y1: 0,
        },
    );
    assert_eq!((r.accepted, r.count), (true, 2), "corners in any order");
    let r = apply_command(
        &mut engine.world,
        &mut engine.resources,
        Command::DesignateChopTiles {
            tiles: vec![(5, 5), (5, 5), (6, 6)],
        },
    );
    assert_eq!(
        (r.accepted, r.count),
        (true, 1),
        "one tree among the listed tiles"
    );
    // Two separate strips become two zones; a tile already zoned is skipped.
    let r = apply_command(
        &mut engine.world,
        &mut engine.resources,
        Command::AddStockpile {
            x0: 10,
            y0: 10,
            x1: 13,
            y1: 11,
        },
    );
    assert_eq!((r.accepted, r.count), (true, 8));
    let i = engine.resources.tile_grid.idx(16, 10);
    engine.resources.tile_grid.terrain_type[i] = TerrainType::River;
    let r = apply_command(
        &mut engine.world,
        &mut engine.resources,
        Command::AddStockpile {
            x0: 13,
            y0: 10,
            x1: 18,
            y1: 10,
        },
    );
    // (13,10) is zoned already and (16,10) is water: (14,10),(15,10) and (17,10),(18,10).
    assert_eq!((r.accepted, r.count), (true, 4));
    assert_eq!(engine.resources.colony.zones.len(), 3);
    let r = apply_command(
        &mut engine.world,
        &mut engine.resources,
        Command::RemoveZoneArea {
            x0: 17,
            y0: 0,
            x1: 23,
            y1: 23,
        },
    );
    assert_eq!((r.accepted, r.count), (true, 2));
    assert_eq!(
        engine.resources.colony.zones.len(),
        2,
        "an emptied zone is deleted"
    );
    // A hut: a site at the footprint centre with its materials.
    let r = apply_command(&mut engine.world, &mut engine.resources, hut(18, 18));
    assert!(r.accepted);
    let sites: Vec<(ConstructionSite, MaterialNeeds, ColonySite)> = engine
        .world
        .query::<(&ConstructionSite, &MaterialNeeds, &ColonySite)>()
        .iter()
        .map(|(_, (c, n, s))| (*c, *n, *s))
        .collect();
    assert_eq!(sites.len(), 1);
    let (site, needs, colony_site) = sites[0];
    assert_eq!((site.position.x, site.position.y), (19, 19));
    assert_eq!(site.blueprint.required_progress, RULE.hut_progress);
    assert_eq!(
        (needs.needed_total(), needs.delivered_total()),
        (RULE.hut_logs, 0)
    );
    assert_eq!(colony_site.tiles().len(), 9);
    assert_eq!(engine.resources.colony.occupied.len(), 9);
    // Cancel over one footprint corner and one mark: both go.
    let r = apply_command(
        &mut engine.world,
        &mut engine.resources,
        Command::CancelArea {
            x0: 0,
            y0: 0,
            x1: 1,
            y1: 1,
        },
    );
    assert_eq!((r.accepted, r.count), (true, 1));
    let r = apply_command(
        &mut engine.world,
        &mut engine.resources,
        Command::CancelArea {
            x0: 20,
            y0: 20,
            x1: 23,
            y1: 23,
        },
    );
    assert_eq!((r.accepted, r.count), (true, 1));
    assert_eq!(engine.world.query::<&ColonySite>().iter().count(), 0);
    assert!(engine.resources.colony.occupied.is_empty());
    assert_eq!(engine.resources.colony.designations.len(), 2);
}

#[test]
fn commands_keep_the_ledger_and_move_logs_out_of_a_new_footprint() {
    let mut engine = job_world(20, 20);
    drop_logs(&mut engine, (6, 6), 7);
    drop_logs(&mut engine, (7, 6), 4);
    // The footprint 5..=7 covers both stacks: they spill out, none is lost.
    assert!(apply_command(&mut engine.world, &mut engine.resources, hut(5, 5)).accepted);
    let (created, ground, carried, delivered, consumed) =
        log_ledger(&engine.world, &engine.resources);
    assert_eq!(
        (created, ground, carried, delivered, consumed),
        (11, 11, 0, 0, 0)
    );
    for tile in engine.resources.colony.ground_items.keys() {
        assert!(
            !engine.resources.colony.occupied.contains_key(tile),
            "a stack under the site at {tile:?}"
        );
    }
    // A site with delivered logs, cancelled: its logs return to the ground.
    {
        let mut q = engine.world.query::<&mut MaterialNeeds>();
        for (_, needs) in q.iter() {
            needs.lines[0].delivered = 5;
        }
    }
    engine.resources.colony.item_ledger.create(ItemKind::Log, 5);
    assert!(
        apply_command(
            &mut engine.world,
            &mut engine.resources,
            Command::CancelArea {
                x0: 6,
                y0: 6,
                x1: 6,
                y1: 6
            }
        )
        .accepted
    );
    let (created, ground, carried, delivered, consumed) =
        log_ledger(&engine.world, &engine.resources);
    assert_eq!(
        (created, ground, carried, delivered, consumed),
        (16, 16, 0, 0, 0)
    );
    sim_systems::runtime::jobs::invariants::check(&engine.world, &engine.resources)
        .expect("invariants");
}

/// A scripted colony run: three orders at fixed ticks.
fn scripted_run(seed: u64, ticks: u64) -> SimEngine {
    let mut engine = sim_test::colony_engine(seed);
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    for tick in 0..ticks {
        match tick {
            60 => {
                submit(
                    &mut engine,
                    Command::DesignateChop {
                        x0: hearth.0.saturating_sub(12),
                        y0: hearth.1.saturating_sub(12),
                        x1: hearth.0 + 12,
                        y1: hearth.1 + 12,
                    },
                );
            }
            61 => {
                submit(
                    &mut engine,
                    Command::AddStockpile {
                        x0: hearth.0 + 3,
                        y0: hearth.1,
                        x1: hearth.0 + 6,
                        y1: hearth.1 + 3,
                    },
                );
            }
            300 => {
                let spot = (3..20)
                    .flat_map(|dy| (0..8).map(move |dx| (hearth.0 + dx, hearth.1 + dy)))
                    .find(|&(x, y)| {
                        sim_systems::commands::check_blueprint(
                            &engine.resources,
                            BlueprintKind::Hut,
                            x,
                            y,
                        )
                        .is_ok()
                    })
                    .expect("a valid hut spot south of the hearth");
                submit(&mut engine, hut(spot.0, spot.1));
            }
            _ => {}
        }
        engine.tick();
    }
    engine
}

fn digest(engine: &SimEngine) -> String {
    let c = &engine.resources.colony;
    format!(
        "{} trees={} marks={:?} ground={:?} zones={:?} occupied={} res={:?} ledger={:?} tallies={:?}",
        sim_test::probe::fingerprint(engine),
        c.trees.len(),
        c.designations.keys().collect::<Vec<_>>(),
        c.ground_items,
        c.zone_of_tile,
        c.occupied.len(),
        c.reservations,
        c.item_ledger,
        c.tallies,
    )
}

#[test]
fn replay_of_the_command_log_reproduces_the_run() {
    let ticks = 1_200;
    let original = scripted_run(3, ticks);
    let log = original.resources.colony.command_log.clone();
    assert_eq!(log.len(), 3, "every order is logged");
    assert!(log.iter().all(|r| r.result.accepted), "{log:?}");
    let mut replay = sim_test::colony_engine(3);
    let mut next = 0;
    for tick in 0..ticks {
        while next < log.len() && log[next].tick == tick {
            let result = submit(&mut replay, log[next].command.clone());
            assert_eq!(result, log[next].result);
            next += 1;
        }
        replay.tick();
    }
    assert_eq!(replay.resources.colony.command_log, log);
    assert_eq!(digest(&replay), digest(&original));
    // The orders mattered: the run without them differs.
    let mut idle = sim_test::colony_engine(3);
    for _ in 0..ticks {
        idle.tick();
    }
    assert_ne!(digest(&idle), digest(&original));
}

/// Use M3 to make the fixture hut available to ordinary job builders.
/// The production mind intentionally dropped natural V7 construction entry
/// (CORE_INVENTORY); legacy seeker cleanup is staged explicitly below.
fn colony_engine_without_knowledge(seed: u64) -> sim_engine::SimEngine {
    let mut engine = sim_test::colony_engine(seed);
    engine.resources.rules.colony = Some(sim_core::rules::ColonyRule::M3);
    assert!(engine.system_names().contains(&"ColonyMindSystem"));
    assert!(!engine.system_names().contains(&"AgentDecisionSystem"));
    engine
}

/// Inject a legacy state after the real mind/jobs and before construction.
/// The witness ensures the cleanup assertion cannot pass because no seeker
/// reached the construction pass. This is fixture setup, not a second selector.
fn stage_legacy_seeker(
    engine: &mut SimEngine,
    entity: hecs::Entity,
    centre: (u32, u32),
) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    use sim_core::components::{AgentState, Position, SeekTarget, TargetKind};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    struct LegacySeekerBoundary {
        entity: Option<hecs::Entity>,
        centre: (u32, u32),
        observed: Arc<AtomicBool>,
    }
    impl sim_engine::RuntimeSystem for LegacySeekerBoundary {
        fn name(&self) -> &str {
            "LegacySeekerFixtureBoundary"
        }
        fn priority(&self) -> u32 {
            132
        }
        fn tick_interval(&self) -> u64 {
            1
        }
        fn tick(&mut self, world: &mut hecs::World, _: &mut sim_engine::SimResources) {
            let Some(entity) = self.entity.take() else {
                return;
            };
            let seeking = AgentState::Seeking {
                target: TargetKind::ConstructionSite,
            };
            *world
                .get::<&mut AgentState>(entity)
                .expect("live fixture adult") = seeking;
            {
                let mut position = world.get::<&mut Position>(entity).unwrap();
                position.x = self.centre.0;
                position.y = self.centre.1;
            }
            let _ = world.remove_one::<SeekTarget>(entity);
            assert_eq!(*world.get::<&AgentState>(entity).unwrap(), seeking);
            self.observed.store(true, Ordering::SeqCst);
        }
    }
    let observed = Arc::new(AtomicBool::new(false));
    engine.register_system(Box::new(LegacySeekerBoundary {
        entity: Some(entity),
        centre,
        observed: observed.clone(),
    }));
    observed
}

/// A legacy seeker at a waiting hut remains there while the site exists,
/// then must be freed when cancellation removes it. Exercise the real colony
/// runtime, without requiring the dropped V7 natural-construction selector.
#[test]
fn a_cancelled_site_frees_its_legacy_seeker() {
    use sim_core::components::{Agent, AgentState, Child, Elder, TargetKind};
    let seeking = AgentState::Seeking {
        target: TargetKind::ConstructionSite,
    };
    let mut engine = colony_engine_without_knowledge(3);
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    let spot = (3..24)
        .flat_map(|dy| (0..10).map(move |dx| (hearth.0 + dx, hearth.1 + dy)))
        .find(|&(x, y)| {
            sim_systems::commands::check_blueprint(&engine.resources, BlueprintKind::Hut, x, y)
                .is_ok()
        })
        .expect("a hut spot near the hearth");
    for _ in 0..400 {
        engine.tick();
    }
    assert!(submit(&mut engine, hut(spot.0, spot.1)).accepted);
    let centre = (spot.0 + 1, spot.1 + 1);
    let mut adults: Vec<(u64, hecs::Entity)> = engine
        .world
        .query::<&Agent>()
        .without::<&Child>()
        .without::<&Elder>()
        .iter()
        .map(|(e, a)| (a.id, e))
        .collect();
    adults.sort();
    let adult = adults[0].1;
    let entered = stage_legacy_seeker(&mut engine, adult, centre);
    engine.tick();
    assert!(entered.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(
        state_of(&engine, adult),
        seeking,
        "a waiting site keeps its seeker"
    );
    assert_eq!(tile_of(&engine, adult), centre);
    let cancel = Command::CancelArea {
        x0: spot.0,
        y0: spot.1,
        x1: spot.0 + 2,
        y1: spot.1 + 2,
    };
    assert!(submit(&mut engine, cancel).accepted);
    let observed = stage_legacy_seeker(&mut engine, adult, centre);
    engine.tick();
    assert!(observed.load(std::sync::atomic::Ordering::SeqCst));
    assert_ne!(
        state_of(&engine, adult),
        seeking,
        "the seeker of a cancelled site must be freed in the next tick"
    );
    // Round 1's frozen agent died about 1,618 ticks after the cancel.
    for _ in 0..2_000 {
        engine.tick();
    }
    assert!(
        engine.world.contains(adult),
        "the freed adult must still be alive"
    );
}

/// A real job builder completes the hut while a staged legacy seeker reaches
/// construction at the same tick boundary. The following construction pass
/// must free the seeker of the now-absent site.
#[test]
fn a_completed_site_frees_its_legacy_seeker() {
    use hecs::Entity;
    use sim_core::colony::Job;
    use sim_core::components::{
        Agent, AgentState, Child, Elder, Hunger, Position, SeekTarget, Sleep, TargetKind, Thirst,
    };
    let seek = AgentState::Seeking {
        target: TargetKind::ConstructionSite,
    };
    let build = AgentState::Consuming {
        target: TargetKind::ConstructionSite,
    };
    let calm = |engine: &mut SimEngine, e: Entity| {
        if let Ok(mut h) = engine.world.get::<&mut Hunger>(e) {
            h.value = 0.0;
        }
        if let Ok(mut t) = engine.world.get::<&mut Thirst>(e) {
            t.value = 0.0;
        }
        if let Ok(mut s) = engine.world.get::<&mut Sleep>(e) {
            s.fatigue = 0.0;
        }
    };
    let put = |engine: &mut SimEngine, e: Entity, tile: (u32, u32)| {
        if let Ok(mut p) = engine.world.get::<&mut Position>(e) {
            p.x = tile.0;
            p.y = tile.1;
        }
    };
    for seed in [3u64, 0] {
        let mut engine = colony_engine_without_knowledge(seed);
        let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
        let spot = (3..24)
            .flat_map(|dy| (0..10).map(move |dx| (hearth.0 + dx, hearth.1 + dy)))
            .find(|&(x, y)| {
                sim_systems::commands::check_blueprint(&engine.resources, BlueprintKind::Hut, x, y)
                    .is_ok()
            })
            .expect("a hut spot near the hearth");
        for _ in 0..400 {
            engine.tick();
        }
        assert!(submit(&mut engine, hut(spot.0, spot.1)).accepted);
        let centre = (spot.0 + 1, spot.1 + 1);
        let site = engine
            .world
            .query::<&ConstructionSite>()
            .iter()
            .find(|(_, s)| (s.position.x, s.position.y) == centre)
            .map(|(e, _)| e)
            .expect("the site");
        // Deliver the logs by hand (and credit the ledger) so the site is ready.
        let needed = engine
            .world
            .get::<&MaterialNeeds>(site)
            .unwrap()
            .needed_total();
        engine
            .resources
            .colony
            .item_ledger
            .create(ItemKind::Log, u64::from(needed));
        engine.world.get::<&mut MaterialNeeds>(site).unwrap().lines[0].delivered = needed;
        let mut adults: Vec<(u64, Entity)> = engine
            .world
            .query::<&Agent>()
            .without::<&Child>()
            .without::<&Elder>()
            .without::<&Job>()
            .iter()
            .map(|(e, a)| (a.id, e))
            .collect();
        adults.sort();
        let (a, b) = (adults[0].1, adults[1].1);
        // The production job systems assign A and start work on the ready site.
        let mut building = false;
        for _ in 0..200 {
            put(&mut engine, a, centre);
            calm(&mut engine, a);
            engine.tick();
            if state_of(&engine, a) == build && tile_of(&engine, a) == centre {
                building = true;
                break;
            }
        }
        assert!(
            building,
            "seed {seed}: A never started building the ready site"
        );
        assert!(matches!(
            engine.world.get::<&Job>(a).unwrap().kind,
            sim_core::colony::JobKind::Build { site } if site == centre
        ));
        // Complete the real job-owned build with B staged at the old race boundary.
        {
            let mut s = engine.world.get::<&mut ConstructionSite>(site).unwrap();
            s.progress = s.blueprint.required_progress - 1;
        }
        put(&mut engine, b, centre);
        calm(&mut engine, a);
        calm(&mut engine, b);
        *engine.world.get::<&mut AgentState>(b).unwrap() = AgentState::Idle;
        let _ = engine.world.remove_one::<SeekTarget>(b);
        let huts = engine.resources.colony.tallies.huts_built;
        let entered = stage_legacy_seeker(&mut engine, b, centre);
        engine.tick();
        assert!(entered.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(
            engine.resources.colony.tallies.huts_built,
            huts + 1,
            "seed {seed}"
        );
        assert_eq!(
            state_of(&engine, b),
            seek,
            "seed {seed}: completion must leave the staged seeker for next-tick cleanup"
        );
        let observed = stage_legacy_seeker(&mut engine, b, centre);
        engine.tick();
        assert!(observed.load(std::sync::atomic::Ordering::SeqCst));
        assert_ne!(
            state_of(&engine, b),
            seek,
            "seed {seed}: the seeker of a completed site must be freed in the next tick"
        );
    }
}
