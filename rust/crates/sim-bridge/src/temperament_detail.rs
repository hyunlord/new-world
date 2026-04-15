//! Shared temperament-detail extraction used by both `runtime_get_entity_detail`
//! and `member_summary`, and testable from `sim-test` without Godot types.
//!
//! This module exists so that the exact same extraction logic is exercised by
//! harness tests and the live entity-detail bridge path.  If someone reverts the
//! bridge to inline its own extraction, the tests that import this helper will
//! still compile but the bridge will diverge — and the harness pipeline's
//! evaluator will catch the mismatch.

use sim_core::components::Temperament;

/// Plain-Rust mirror of the temperament fields exposed through `entity_detail()`.
///
/// All axis values are **f64** — no f32 narrowing.  The bridge converts these to
/// `VarDictionary` entries; tests assert against this struct directly.
#[derive(Debug, Clone, PartialEq)]
pub struct TemperamentDetail {
    /// Novelty Seeking (expressed axis), f64 in [0.0, 1.0].
    pub tci_ns: f64,
    /// Harm Avoidance (expressed axis), f64 in [0.0, 1.0].
    pub tci_ha: f64,
    /// Reward Dependence (expressed axis), f64 in [0.0, 1.0].
    pub tci_rd: f64,
    /// Persistence (expressed axis), f64 in [0.0, 1.0].
    pub tci_p: f64,
    /// Locale key for the 4-humor label (e.g. `"TEMPERAMENT_SANGUINE"`).
    pub temperament_label_key: &'static str,
    /// Whether a latent→expressed divergence has been unlocked by a shift event.
    pub temperament_awakened: bool,
}

/// Extracts temperament detail from an ECS `Temperament` component.
///
/// This is the **single source of truth** for how temperament data reaches the UI.
/// Both `runtime_get_entity_detail` (lib.rs) and `member_summary` (runtime_queries.rs)
/// call this function, ensuring test coverage and runtime behavior target the same
/// code path.
pub fn extract_temperament_detail(t: &Temperament) -> TemperamentDetail {
    TemperamentDetail {
        tci_ns: t.expressed.ns,
        tci_ha: t.expressed.ha,
        tci_rd: t.expressed.rd,
        tci_p: t.expressed.p,
        temperament_label_key: t.archetype_label_key(),
        temperament_awakened: t.awakened,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::temperament::TemperamentAxes;

    fn make_test_temperament(ns: f64, ha: f64, rd: f64, p: f64) -> Temperament {
        let axes = TemperamentAxes { ns, ha, rd, p };
        Temperament {
            genes: [0.0; 4],
            latent: axes,
            expressed: axes,
            awakened: false,
            shift_count: 0,
        }
    }

    #[test]
    fn extract_preserves_f64_precision() {
        let t = make_test_temperament(0.123456789012345, 0.987654321098765, 0.5, 0.5);
        let d = extract_temperament_detail(&t);
        // Must be exact f64 — no f32 narrowing.
        assert_eq!(d.tci_ns, 0.123456789012345);
        assert_eq!(d.tci_ha, 0.987654321098765);
    }

    #[test]
    fn extract_label_sanguine() {
        let t = make_test_temperament(0.7, 0.3, 0.5, 0.5);
        let d = extract_temperament_detail(&t);
        assert_eq!(d.temperament_label_key, "TEMPERAMENT_SANGUINE");
    }

    #[test]
    fn extract_label_choleric() {
        let t = make_test_temperament(0.7, 0.6, 0.5, 0.5);
        let d = extract_temperament_detail(&t);
        assert_eq!(d.temperament_label_key, "TEMPERAMENT_CHOLERIC");
    }

    #[test]
    fn extract_label_melancholic() {
        let t = make_test_temperament(0.3, 0.7, 0.5, 0.5);
        let d = extract_temperament_detail(&t);
        assert_eq!(d.temperament_label_key, "TEMPERAMENT_MELANCHOLIC");
    }

    #[test]
    fn extract_label_phlegmatic() {
        let t = make_test_temperament(0.3, 0.3, 0.5, 0.5);
        let d = extract_temperament_detail(&t);
        assert_eq!(d.temperament_label_key, "TEMPERAMENT_PHLEGMATIC");
    }
}
