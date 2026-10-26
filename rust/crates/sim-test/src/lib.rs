//! WorldSim test harness crate (T7.5.5.D scaffold).
//!
//! Hosts long-running simulation tests and cross-crate integration scenarios.
//! Tests live under `tests/` (Cargo integration tests).
//!
//! V7 reset 0.7.0 — Phase 0 v0.1.3 patch Section 4.4.

/// M5: the allocator byte counter of `worldsim_perf` (measurement only).
pub mod alloc;
/// Feature-only decision-arm profiling, exported separately from simulation outputs.
#[cfg(feature = "perf_arms")]
pub mod arm_profile;
/// M1.5: the standing colony benchmark (`worldsim_perf bench`, ADR 0016 C3).
pub mod bench;
/// Explicit immutable content selection for headless CLI entry points.
pub mod content_boot;
/// M3 pathfinding benchmark (`worldsim_perf nav-bench`).
pub mod nav_bench;
/// Revitalize Pass 15: tick timings, the stress world and snapshot costs (`worldsim_perf`).
pub mod perf;
/// Headless measurements and trajectory fingerprints.
pub mod probe;
/// Shared production-scene fixtures.
pub mod scenes;
/// Read-only annual story metrics over durable chronicle causes.
pub mod story_yield;

/// The live game's engine for world `seed` (`scenes::baseline_scene`): the
/// starting point of every test that runs the game's rules.
pub fn baseline_engine(seed: u64) -> sim_engine::SimEngine {
    scenes::baseline_scene(seed)
}

/// The M1 colony world's engine for world `seed` (`scenes::colony_scene`):
/// the starting point of every test that runs the colony (ADR 0014).
pub fn colony_engine(seed: u64) -> sim_engine::SimEngine {
    scenes::colony_scene(seed)
}
/// Pass 18: `worldsim_perf`'s `subprofile` block (feature `subprofile` only).
#[cfg(feature = "subprofile")]
pub mod subprofile_report;

#[cfg(test)]
mod tests {
    #[test]
    fn baseline_engine_is_the_baseline_scene_at_tick_zero() {
        let engine = super::baseline_engine(3);
        assert_eq!(engine.current_tick(), 0);
        assert_eq!(
            engine.resources.rules,
            sim_core::rules::WorldRules::baseline(3)
        );
        assert_eq!(
            super::probe::fingerprint(&engine),
            super::probe::fingerprint(&super::scenes::baseline_scene(3))
        );
    }
}

/// Explicit legacy-world shadow comparison; never enabled by default.
#[cfg(feature = "mind_shadow")]
pub mod mind_shadow;

/// Independent runtime mind reason coverage for colony probes.
pub mod mind_coverage;

/// Explicit post-timing synthetic mind history capacity companion.
pub mod mind_memory;
