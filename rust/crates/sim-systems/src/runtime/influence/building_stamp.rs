//! `BuildingStampSystem` — priority 90, every tick.
//!
//! Phase 0 Section 2.5.1 base. Phase 2 land (T7.6) is a *no-op shell*:
//! Building components do not exist yet (they land in Phase 11). The shell
//! exists so the priority slot (90 < 100 < 110) is locked from day 1 and
//! cannot be reassigned later.
//!
//! Isolation invariant: this system must **not** clear or modify the
//! influence pending buffers. Any value written into pending before this
//! system runs must still be present when it exits. This invariant is
//! tested by `harness_building_stamp_isolation` in the integration suite.

use hecs::World;
use sim_engine::{RuntimeSystem, SimResources};

/// Phase 2 building → influence stamper (no-op shell).
pub struct BuildingStampSystem;

impl BuildingStampSystem {
    /// Construct a new shell.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BuildingStampSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSystem for BuildingStampSystem {
    fn name(&self) -> &str {
        "BuildingStampSystem"
    }

    fn priority(&self) -> u32 {
        90
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, _world: &mut World, _resources: &mut SimResources) {
        // No buildings in Phase 2 — shell only.
        // MUST NOT touch pending buffers (isolation invariant).
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    #[test]
    fn metadata() {
        let s = BuildingStampSystem::new();
        assert_eq!(s.name(), "BuildingStampSystem");
        assert_eq!(s.priority(), 90);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn tick_does_not_panic() {
        let mut e = SimEngine::new(32, 32, MaterialRegistry::new());
        e.register_system(Box::new(BuildingStampSystem::new()));
        for _ in 0..5 {
            e.tick();
        }
        assert_eq!(e.current_tick(), 5);
    }

    #[test]
    fn shell_does_not_mutate_tile_grid() {
        let mut e = SimEngine::new(32, 32, MaterialRegistry::new());
        let before = e.resources.tile_grid.len();
        e.register_system(Box::new(BuildingStampSystem::new()));
        for _ in 0..3 {
            e.tick();
        }
        assert_eq!(e.resources.tile_grid.len(), before);
    }
}
