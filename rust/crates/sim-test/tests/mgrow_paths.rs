//! M-GROW 3.2: worn paths — speed bonus on dirt and paved tiles.
//!
//! An agent seeking a goal on a dirt-stage path should arrive faster than
//! one on plain grass, because the step interval is 2 instead of 3.

mod colony_support;

use sim_core::colony::paths::{STAGE_DIRT, STAGE_PAVED};
use sim_core::components::{Agent, AgentState, Position, SeekTarget, TargetKind};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::RuntimeSystem;
use sim_engine::SimEngine;
use sim_systems::runtime::agent::MovementRng;

fn colony_engine() -> SimEngine {
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules = WorldRules::colony(0);
    engine.resources.rules.agent.idle_pause_chance = 0.0;
    // move_interval_ticks = 3 (colony default).
    let grid_len = 32 * 32;
    engine.resources.colony.footfall.resize(grid_len, 0);
    engine.resources.colony.path_stage.resize(grid_len, 0);
    engine
        .resources
        .nav
        .install(sim_core::nav::CostMap::new(32, 32));
    engine
}

/// Ticks for a seeking agent to walk from (2, y) to (20, y) on a straight line.
fn walk_ticks(stage: u8) -> u64 {
    let mut engine = colony_engine();
    // Set path stage along the row y=5.
    for x in 0..32 {
        let idx = engine.resources.tile_grid.idx(x, 5);
        engine.resources.colony.path_stage[idx] = stage;
        // Also set nav cost from the stage.
        let cost = sim_core::colony::paths::installed()
            .stage_by_code(stage)
            .map_or(10, |s| s.nav_cost);
        engine.resources.nav.set_cost((x, 5), cost);
    }
    walk_engine(&mut engine)
}

fn walk_engine(engine: &mut SimEngine) -> u64 {
    engine.world.spawn((
        Agent { id: 1 },
        Position::new(2, 5),
        MovementRng::new(42),
        AgentState::Seeking {
            target: TargetKind::Job,
        },
        SeekTarget::new((20, 5)),
    ));
    let mut sys = sim_systems::runtime::agent::AgentMovementSystem::new();
    for tick in 1..=200 {
        engine.resources.current_tick = tick;
        sys.tick(&mut engine.world, &mut engine.resources);
        let pos = engine
            .world
            .query::<&Position>()
            .iter()
            .next()
            .map(|(_, p)| (p.x, p.y));
        if pos == Some((20, 5)) {
            return tick;
        }
    }
    panic!("agent did not arrive at (20,5) within 200 ticks");
}

#[test]
fn paved_and_dirt_speed_bonus() {
    let grass_ticks = walk_ticks(0);
    let dirt_ticks = walk_ticks(STAGE_DIRT);
    let paved_ticks = walk_ticks(STAGE_PAVED);
    // 18 steps: 3-tick grass cadence versus the authored 2-tick fast cadence.
    assert_eq!(grass_ticks, 54);
    assert_eq!(dirt_ticks, 36);
    assert_eq!(paved_ticks, 36);
    // Dirt and paved should be strictly faster than grass.
    assert!(
        dirt_ticks < grass_ticks,
        "dirt ({dirt_ticks}) should be faster than grass ({grass_ticks})"
    );
    assert!(
        paved_ticks < grass_ticks,
        "paved ({paved_ticks}) should be faster than grass ({grass_ticks})"
    );
    // Paved should be at least as fast as dirt.
    assert!(
        paved_ticks <= dirt_ticks,
        "paved ({paved_ticks}) should be <= dirt ({dirt_ticks})"
    );
}

#[test]
fn save_round_trip_preserves_footfall_and_path_stage() {
    let mut engine = colony_engine();
    let grid_len = 32 * 32;
    engine.resources.colony.footfall.resize(grid_len, 0);
    engine.resources.colony.path_stage.resize(grid_len, 0);
    // Set some footfall/stage data.
    let idx = engine.resources.tile_grid.idx(5, 5);
    engine.resources.colony.footfall[idx] = 42;
    engine.resources.colony.path_stage[idx] = STAGE_DIRT;
    let idx2 = engine.resources.tile_grid.idx(10, 10);
    engine.resources.colony.footfall[idx2] = 200;
    engine.resources.colony.path_stage[idx2] = STAGE_PAVED;
    // A placed-only stage requires its matching physical floor identity.
    engine.resources.tile_grid.floor_material[idx2] =
        Some(sim_core::material::MaterialId::from_str_hash("cut_stone"));
    let road = sim_core::colony::BlueprintKind::from_name("paved_road").expect("road blueprint");
    sim_systems::runtime::jobs::space::record_completion(&mut engine.resources, (10, 10), road);
    // Register systems for save/load.
    sim_systems::register_world_runtime_systems(
        &mut engine,
        sim_engine::registration::WorldKind::Colony,
    );
    let bytes = sim_systems::save::save(&mut engine).expect("save succeeds");
    let loaded = sim_systems::save::load(&bytes).expect("load succeeds");
    assert_eq!(loaded.resources.colony.footfall.len(), grid_len);
    assert_eq!(loaded.resources.colony.path_stage.len(), grid_len);
    let lidx = loaded.resources.tile_grid.idx(5, 5);
    assert_eq!(loaded.resources.colony.footfall[lidx], 42);
    assert_eq!(loaded.resources.colony.path_stage[lidx], STAGE_DIRT);
    let lidx2 = loaded.resources.tile_grid.idx(10, 10);
    assert_eq!(loaded.resources.colony.footfall[lidx2], 200);
    assert_eq!(loaded.resources.colony.path_stage[lidx2], STAGE_PAVED);
}

#[test]
fn completed_physical_road_activates_speed_and_deconstruction_removes_it() {
    use colony_support::{drop_items, job_world_m2, run_until, worker};
    use sim_core::colony::knowledge::{self, Knowledge, Source};
    use sim_core::colony::{BlueprintKind, Command, ItemKind, ItemSpec};
    use sim_core::rules::ColonyRule;
    use sim_systems::runtime::{jobs::space, nav};
    let mut engine = job_world_m2(32, 32);
    engine.resources.rules.colony = Some(ColonyRule::M4);
    engine.resources.rules.agent.move_interval_ticks = 3;
    engine.resources.rules.agent.idle_pause_chance = 0.0;
    let builder = worker(&mut engine, 2, 3);
    let mut knowledge = Knowledge::default();
    knowledge.set(
        knowledge::id("paved_road").expect("road knowledge"),
        800,
        Source::Seeded,
        0,
    );
    engine
        .world
        .insert_one(builder, knowledge)
        .expect("builder practice");
    let stone = ItemKind::from_name("cut_stone").expect("construction material");
    drop_items(&mut engine, (2, 4), ItemSpec::raw(stone, None), 19);
    let road = BlueprintKind::from_name("paved_road").expect("road");
    let result = sim_systems::commands::submit(
        &mut engine,
        Command::PlaceLine {
            kind: road,
            x0: 2,
            y0: 5,
            x1: 20,
            y1: 5,
        },
    );
    assert_eq!(result.count, 19, "{result:?}");
    assert_eq!(
        nav::tile_cost(&engine.resources, (10, 5)),
        10,
        "unfinished sites confer no bonus"
    );
    let ticks = run_until(&mut engine, 40_000, |engine| {
        engine.resources.colony.tallies.space_built == 19
    });
    assert!(
        ticks < 40_000,
        "road never built by ordinary haul/build jobs"
    );
    for x in 2..=20 {
        let index = engine.resources.tile_grid.idx(x, 5);
        assert!(engine.resources.tile_grid.floor_material[index].is_some());
        assert_eq!(engine.resources.colony.path_stage[index], STAGE_PAVED);
        assert_eq!(nav::tile_cost(&engine.resources, (x, 5)), 6);
    }
    assert_eq!(*engine.resources.nav.map(), nav::build(&engine.resources));
    let ledger =
        sim_systems::runtime::jobs::invariants::ledger_of(&engine.world, &engine.resources, stone);
    assert_eq!(ledger.created, 19);
    assert_eq!(ledger.consumed, 19);
    engine.world.clear();
    assert_eq!(
        walk_engine(&mut engine),
        36,
        "physically completed road activates real cadence"
    );
    let version = engine.resources.colony.path_version;
    assert_eq!(
        space::deconstruct(&mut engine.resources, (10, 5)),
        Some(road)
    );
    let index = engine.resources.tile_grid.idx(10, 5);
    let natural = sim_core::colony::paths::installed()
        .stage_for_footfall(engine.resources.colony.footfall[index]);
    assert_eq!(engine.resources.colony.path_stage[index], natural.code);
    assert_eq!(nav::tile_cost(&engine.resources, (10, 5)), natural.nav_cost);
    assert!(engine.resources.colony.path_version > version);
}

#[test]
fn waiting_for_surface_cadence_keeps_the_walk_and_counts_only_completed_steps() {
    let mut engine = colony_engine();
    engine.world.spawn((
        Agent { id: 1 },
        Position::new(2, 5),
        MovementRng::new(42),
        AgentState::Seeking {
            target: TargetKind::Job,
        },
        SeekTarget::new((20, 5)),
    ));
    let mut movement = sim_systems::runtime::agent::AgentMovementSystem::new();
    engine.resources.current_tick = 3;
    movement.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        engine
            .resources
            .nav
            .paths
            .get(&1)
            .expect("planned route")
            .planned,
        3
    );
    assert_eq!(
        engine
            .resources
            .colony
            .footfall
            .iter()
            .copied()
            .sum::<u16>(),
        1
    );
    engine.resources.current_tick = 4;
    movement.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        engine
            .resources
            .nav
            .paths
            .get(&1)
            .expect("cadence wait retains route")
            .planned,
        3
    );
    assert_eq!(
        engine
            .resources
            .colony
            .footfall
            .iter()
            .copied()
            .sum::<u16>(),
        1
    );
    engine.resources.current_tick = 6;
    movement.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        engine
            .resources
            .nav
            .paths
            .get(&1)
            .expect("route reused")
            .planned,
        3
    );
    assert_eq!(
        engine
            .resources
            .colony
            .footfall
            .iter()
            .copied()
            .sum::<u16>(),
        2
    );
}
