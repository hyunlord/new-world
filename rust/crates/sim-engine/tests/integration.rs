//! Integration tests — sim-core + sim-engine baseline.
//!
//! V7 reset 후 첫 multi-crate integration (T7.5.5.C).
//! Phase 0 v0.1.3 patch Section 1.3.6 정합 검증.

use hecs::World;
use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine, SimResources};
use std::sync::{Arc, Mutex};

fn empty_registry() -> MaterialRegistry {
    MaterialRegistry::new()
}

#[test]
fn test_zero_systems_baseline() {
    let engine = SimEngine::new(256, 256, empty_registry());
    assert_eq!(engine.system_count(), 0);
    assert_eq!(engine.current_tick(), 0);
    assert!(engine.system_names().is_empty());
}

#[test]
fn test_tick_loop_no_panic() {
    let mut engine = SimEngine::new(256, 256, empty_registry());
    for _ in 0..100 {
        engine.tick();
    }
    assert_eq!(engine.current_tick(), 100);
}

#[test]
fn test_resources_initial_state() {
    let engine = SimEngine::new(256, 256, empty_registry());

    assert_eq!(engine.resources.tile_grid.width, 256);
    assert_eq!(engine.resources.tile_grid.height, 256);
    assert_eq!(engine.resources.tile_grid.len(), 256 * 256);

    for ch in InfluenceChannel::all() {
        assert_eq!(engine.resources.influence_grid.sample(0, 0, *ch), 0);
        assert_eq!(engine.resources.influence_grid.sample(128, 128, *ch), 0);
        assert_eq!(engine.resources.influence_grid.sample(255, 255, *ch), 0);
    }

    assert!(engine.resources.material_blocking_cache.is_empty());
    assert_eq!(engine.resources.current_tick, 0);
}

#[test]
fn test_resources_modification_via_tick() {
    struct WarmthWriter;
    impl RuntimeSystem for WarmthWriter {
        fn name(&self) -> &str {
            "WarmthWriter"
        }
        fn priority(&self) -> u32 {
            100
        }
        fn tick_interval(&self) -> u64 {
            1
        }
        fn tick(&mut self, _world: &mut World, resources: &mut SimResources) {
            let idx = resources.influence_grid.idx(10, 10);
            let buf = resources
                .influence_grid
                .pending_buf_mut(InfluenceChannel::Warmth);
            buf[idx] = 200;
            resources.influence_grid.swap();
        }
    }

    let mut engine = SimEngine::new(256, 256, empty_registry());
    engine.register_system(Box::new(WarmthWriter));
    assert_eq!(
        engine.resources.influence_grid.sample(10, 10, InfluenceChannel::Warmth),
        0
    );
    engine.tick();
    assert_eq!(
        engine.resources.influence_grid.sample(10, 10, InfluenceChannel::Warmth),
        200
    );
}

#[test]
fn test_multiple_systems_priority_order() {
    struct OrderTracker {
        name: &'static str,
        priority: u32,
        log: Arc<Mutex<Vec<&'static str>>>,
    }
    impl RuntimeSystem for OrderTracker {
        fn name(&self) -> &str {
            self.name
        }
        fn priority(&self) -> u32 {
            self.priority
        }
        fn tick_interval(&self) -> u64 {
            1
        }
        fn tick(&mut self, _world: &mut World, _resources: &mut SimResources) {
            self.log.lock().unwrap().push(self.name);
        }
    }

    let log = Arc::new(Mutex::new(Vec::new()));
    let mut engine = SimEngine::new(256, 256, empty_registry());
    engine.register_system(Box::new(OrderTracker {
        name: "high",
        priority: 1000,
        log: log.clone(),
    }));
    engine.register_system(Box::new(OrderTracker {
        name: "low",
        priority: 100,
        log: log.clone(),
    }));
    engine.register_system(Box::new(OrderTracker {
        name: "mid",
        priority: 500,
        log: log.clone(),
    }));

    assert_eq!(engine.system_count(), 3);
    assert_eq!(engine.system_names(), vec!["low", "mid", "high"]);

    engine.tick();
    let observed = log.lock().unwrap().clone();
    assert_eq!(observed, vec!["low", "mid", "high"]);
}
