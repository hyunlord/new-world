//! Focused physical heat, numeric Cold and bed-stage regressions; no balance bars.
mod colony_support;
use colony_support::{adults, bedroll, detect, open_area, walled_box};
use sim_core::colony::thermal::PaidHeat;
use sim_core::components::{Position, ResourceKind};
use sim_engine::RuntimeSystem;
use sim_systems::runtime::thermal;

fn room() -> (sim_engine::SimEngine, Vec<(u32, u32)>) {
    let mut engine = sim_test::colony_engine(2);
    for _ in 0..10 {
        engine.tick();
    }
    let at = open_area(&engine, 5, 5);
    let tiles = walled_box(&mut engine, at, 5, 5, true);
    detect(&mut engine);
    engine
        .resources
        .building_kinds
        .insert(tiles[4], sim_core::components::BuildingKind::Hearth);
    thermal::initialize(&mut engine.resources);
    (engine, tiles)
}

#[path = "colony_thermal/boundary.rs"]
mod boundary;
#[path = "colony_thermal/dwelling.rs"]
mod dwelling;
#[path = "colony_thermal/heat.rs"]
mod heat;
