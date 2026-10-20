//! WorldSim test harness crate (T7.5.5.D scaffold).
//!
//! Hosts long-running simulation tests and cross-crate integration scenarios.
//! Tests live under `tests/` (Cargo integration tests).
//!
//! V7 reset 0.7.0 — Phase 0 v0.1.3 patch Section 4.4.

/// Revitalize Pass 15: tick timings, the stress world and snapshot costs (`worldsim_perf`).
pub mod perf;
/// Headless measurements and trajectory fingerprints.
pub mod probe;
/// Shared production-scene fixtures.
pub mod scenes;
/// Pass 18: `worldsim_perf`'s `subprofile` block (feature `subprofile` only).
#[cfg(feature = "subprofile")]
pub mod subprofile_report;

#[cfg(test)]
mod tests {
    #[test]
    fn shared_scene_starts_at_tick_zero() {
        assert_eq!(super::scenes::legacy_scene().current_tick(), 0);
    }
}
