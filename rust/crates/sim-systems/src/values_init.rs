//! Values initialization: port of value_system.gd::initialize_values().
//!
//! Computes initial Schwartz values from HEXACO personality facets
//! using seed weights, noise, and optional parental heritability.
//!
//! References:
//! - Schwartz (1992): 33-value theory
//! - Knafo & Schwartz (2004): differential value heritability by value type
//! - Ashton & Lee (2009): HEXACO facet–value correlations

use rand::Rng;
use rand_distr::{Distribution, Normal};
use sim_core::components::{Personality, Values};
use sim_data::{facet_index, hexaco_seed_map, value_heritability};

/// Initialize 33 Schwartz values from HEXACO personality facets.
///
/// # Formula
/// ```text
/// hexaco_seed[v] = Σ(z(facet) × weight) / Σ|weight|
///   where z(facet) = clamp(2 × (facet − 0.5), −1, 1)
///
/// final[v] = hs × 0.40 + noise × 0.40 + (genetic − hs) × h_v
///   where noise   ~ N(0, 0.5)
///         genetic = midparent + N(0, 0.1)   [if parents]
///                 = hs                        [no parents]
///         h_v     = per-value heritability coefficient
/// All values clamped to [−1.0, 1.0].
/// ```
pub fn initialize_values(
    personality: &Personality,
    parent_a_values: Option<&Values>,
    parent_b_values: Option<&Values>,
    rng: &mut impl Rng,
) -> Values {
    let noise_dist =
        Normal::new(0.0_f64, 0.5).expect("N(0, 0.5): std-dev is a valid positive constant");
    let genetic_noise_dist =
        Normal::new(0.0_f64, 0.1).expect("N(0, 0.1): std-dev is a valid positive constant");

    let mut values = Values::default();

    for (v, facet_weights) in hexaco_seed_map() {
        // Step 1: Compute HEXACO seed for this value type.
        let mut raw = 0.0_f64;
        let mut total_weight = 0.0_f64;

        for (facet_name, weight) in *facet_weights {
            // Default to 0 (H_sincerity) on unknown facet name — should never happen
            // with canonical data from values_seed.rs.
            let idx = facet_index(facet_name).unwrap_or(0);
            let facet_val = personality.facets[idx]; // 0.0..=1.0
            let z = (2.0 * (facet_val - 0.5)).clamp(-1.0, 1.0);
            raw += z * weight;
            total_weight += weight.abs();
        }

        let hs = if total_weight > 0.0 {
            (raw / total_weight).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        // Step 2: Genetic midparent value (or fall back to hs for first generation).
        let genetic = match (parent_a_values, parent_b_values) {
            (Some(pa), Some(pb)) => {
                let midparent = (pa.get(*v) + pb.get(*v)) * 0.5;
                midparent + genetic_noise_dist.sample(rng)
            }
            _ => hs,
        };

        // Step 3: Combine seed, noise, and genetic contribution.
        let noise: f64 = noise_dist.sample(rng);
        let h_v = value_heritability(*v);
        let final_val = (hs * 0.40 + noise * 0.40 + (genetic - hs) * h_v).clamp(-1.0, 1.0);

        values.values[*v as usize] = final_val;
    }

    values
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use sim_core::enums::ValueType;

    fn make_personality(facet_idx: usize, val: f64) -> Personality {
        let mut p = Personality::default();
        p.facets[facet_idx] = val;
        p.recalculate_axes();
        p
    }

    #[test]
    fn most_values_nonzero_with_default_personality() {
        // Even with a neutral personality, noise ensures most values are non-zero.
        let personality = Personality::default();
        let mut rng = SmallRng::seed_from_u64(42);
        let values = initialize_values(&personality, None, None, &mut rng);
        let nonzero = values.values.iter().filter(|v| v.abs() > 0.001).count();
        assert!(
            nonzero >= 20,
            "Expected ≥20 non-zero values, got {}",
            nonzero
        );
    }

    #[test]
    fn all_values_in_range() {
        let personality = Personality::default();
        let mut rng = SmallRng::seed_from_u64(99);
        let values = initialize_values(&personality, None, None, &mut rng);
        for (i, v) in values.values.iter().enumerate() {
            assert!(
                *v >= -1.0 && *v <= 1.0,
                "values[{}] = {} is out of [-1.0, 1.0]",
                i,
                v
            );
        }
    }

    #[test]
    fn high_h_fairness_produces_positive_fairness_in_majority_of_trials() {
        // H_fairness (facet index 1) has weight 0.40 on FAIRNESS (ValueType::Fairness).
        // With facet = 0.95, z = +0.90, seed should be strongly positive.
        let personality = make_personality(1, 0.95);
        let mut positive = 0;
        for seed in 0..30u64 {
            let mut rng = SmallRng::seed_from_u64(seed);
            let v = initialize_values(&personality, None, None, &mut rng);
            if v.get(ValueType::Fairness) > 0.0 {
                positive += 1;
            }
        }
        assert!(
            positive >= 18,
            "High H_fairness should produce positive FAIRNESS in ≥18/30 trials, got {}",
            positive
        );
    }

    #[test]
    fn parental_values_in_range() {
        let personality = Personality::default();
        let mut parent_values = Values::default();
        parent_values.values[ValueType::Power as usize] = 0.9;
        parent_values.values[ValueType::Harmony as usize] = -0.8;

        let mut rng = SmallRng::seed_from_u64(42);
        let child = initialize_values(
            &personality,
            Some(&parent_values),
            Some(&parent_values),
            &mut rng,
        );
        for v in &child.values {
            assert!(*v >= -1.0 && *v <= 1.0, "Parental value {} out of range", v);
        }
    }
}
