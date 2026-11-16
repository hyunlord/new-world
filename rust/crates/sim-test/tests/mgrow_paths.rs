//! M-GROW 3.2: worn paths — speed bonus on dirt and paved tiles.
//!
//! An agent seeking a goal on a dirt-stage path should arrive faster than
//! one on plain grass, because the step interval is 2 instead of 3.

use sim_core::colony::paths::{STAGE_DIRT, STAGE_PAVED};
use sim_engine::RuntimeSystem;
use sim_core::components::{Agent, AgentState, Position, SeekTarget, TargetKind};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
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
    engine.resources.nav.install(sim_core::nav::CostMap::new(32, 32));
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
