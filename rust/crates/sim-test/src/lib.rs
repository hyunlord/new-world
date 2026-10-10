//! WorldSim test harness crate (T7.5.5.D scaffold).
//!
//! Hosts long-running simulation tests and cross-crate integration scenarios.
//! Tests live under `tests/` (Cargo integration tests).
//!
//! V7 reset 0.7.0 — Phase 0 v0.1.3 patch Section 4.4.

/// Headless measurements and trajectory fingerprints.
pub mod probe;
/// Shared production-scene fixtures.
pub mod scenes;

#[cfg(test)]
mod tests {
    #[test]
    fn shared_scene_starts_at_tick_zero() {
        assert_eq!(super::scenes::legacy_scene().current_tick(), 0);
    }
}
