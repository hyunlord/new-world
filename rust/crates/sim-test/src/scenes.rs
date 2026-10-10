//! Shared, frozen simulation fixtures used by probes and integration tests.
use sim_engine::SimEngine;

/// The shipped legacy world, including the three former renderer boot buildings.
pub fn legacy_scene() -> SimEngine {
    let mut engine = sim_bridge::ffi::world_node::init_production_engine();
    for x in [32, 24, 40] {
        sim_bridge::ffi::world_node::enqueue_building_placed(&mut engine.resources, x, 32, 8);
    }
    engine
}

#[cfg(test)]
mod tests {
    #[test]
    fn legacy_boot_buildings_are_queued_without_advancing_time() {
        let engine = super::legacy_scene();
        assert_eq!(engine.current_tick(), 0);
        assert_eq!(engine.resources.building_event_queue.len(), 3);
    }
}
