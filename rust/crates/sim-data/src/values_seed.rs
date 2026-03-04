//! Compile-time HEXACO → Values seed mapping and heritability data.
//!
//! Academic references:
//! - Schwartz (1992) — 33 value theory
//! - Knafo & Schwartz (2004) — differential value heritability by type
//! - Ashton & Lee (2009) — HEXACO facet → value correlations
//!
//! Ported from:
//! - `scripts/core/social/value_defs.gd` HEXACO_SEED_MAP
//! - `scripts/core/simulation/game_config.gd` VALUE_HERITABILITY

use sim_core::enums::ValueType;

// ── Facet Index Mapping ───────────────────────────────────────────────────────

/// Map a HEXACO facet name string to its index in `Personality.facets[24]`.
///
/// Index = axis_index × 4 + facet_offset.
/// H: sincerity=0, fairness=1, greed_avoidance=2, modesty=3
/// E: fearfulness=4, anxiety=5, dependence=6, sentimentality=7
/// X: social_self_esteem=8, social_boldness=9, sociability=10, liveliness=11
/// A: forgivingness=12, gentleness=13, flexibility=14, patience=15
/// C: organization=16, diligence=17, perfectionism=18, prudence=19
/// O: aesthetic=20, inquisitiveness=21, creativity=22, unconventionality=23
pub fn facet_index(name: &str) -> Option<usize> {
    match name {
        "H_sincerity"         => Some(0),
        "H_fairness"          => Some(1),
        "H_greed_avoidance"   => Some(2),
        "H_modesty"           => Some(3),
        "E_fearfulness"       => Some(4),
        "E_anxiety"           => Some(5),
        "E_dependence"        => Some(6),
        "E_sentimentality"    => Some(7),
        "X_social_self_esteem"=> Some(8),
        "X_social_boldness"   => Some(9),
        "X_sociability"       => Some(10),
        "X_liveliness"        => Some(11),
        "A_forgivingness"     => Some(12),
        "A_gentleness"        => Some(13),
        "A_flexibility"       => Some(14),
        "A_patience"          => Some(15),
        "C_organization"      => Some(16),
        "C_diligence"         => Some(17),
        "C_perfectionism"     => Some(18),
        "C_prudence"          => Some(19),
        "O_aesthetic"         => Some(20),
        "O_inquisitiveness"   => Some(21),
        "O_creativity"        => Some(22),
        "O_unconventionality" => Some(23),
        _                     => None,
    }
}

// ── HEXACO Seed Map ───────────────────────────────────────────────────────────

/// Returns the static HEXACO facet → ValueType seed weight mapping.
///
/// For each `(ValueType, &[(facet_name, weight)])`:
/// - `facet_name`: use `facet_index()` to convert to Personality.facets[] index
/// - `weight`: positive = facet promotes value, negative = facet suppresses value
///
/// Weights are normalized per-value during initialization.
/// Source: value_defs.gd HEXACO_SEED_MAP [GPT 수치 설계 2025]
pub fn hexaco_seed_map() -> &'static [(ValueType, &'static [(&'static str, f64)])] {
    &[
        (ValueType::Law,          &[("H_fairness", 0.35), ("C_prudence", 0.25), ("C_organization", 0.20), ("H_sincerity", 0.20)]),
        (ValueType::Loyalty,      &[("E_sentimentality", 0.30), ("E_dependence", 0.25), ("H_sincerity", 0.25), ("A_forgivingness", 0.20)]),
        (ValueType::Family,       &[("E_sentimentality", 0.40), ("E_dependence", 0.25), ("H_sincerity", 0.20), ("A_patience", 0.15)]),
        (ValueType::Friendship,   &[("X_sociability", 0.30), ("E_sentimentality", 0.25), ("H_sincerity", 0.25), ("A_forgivingness", 0.20)]),
        (ValueType::Power,        &[("X_social_boldness", 0.30), ("H_modesty", -0.35), ("H_greed_avoidance", -0.25), ("X_social_self_esteem", 0.10)]),
        (ValueType::Truth,        &[("H_sincerity", 0.35), ("H_fairness", 0.30), ("H_greed_avoidance", 0.20), ("C_prudence", 0.15)]),
        (ValueType::Cunning,      &[("H_sincerity", -0.30), ("O_unconventionality", 0.25), ("O_creativity", 0.25), ("X_social_boldness", 0.20)]),
        (ValueType::Eloquence,    &[("X_social_boldness", 0.30), ("X_social_self_esteem", 0.25), ("X_sociability", 0.25), ("O_creativity", 0.20)]),
        (ValueType::Fairness,     &[("H_fairness", 0.40), ("H_sincerity", 0.25), ("H_greed_avoidance", 0.20), ("A_patience", 0.15)]),
        (ValueType::Decorum,      &[("H_modesty", 0.30), ("A_patience", 0.25), ("A_gentleness", 0.25), ("C_prudence", 0.20)]),
        (ValueType::Tradition,    &[("C_prudence", 0.30), ("O_unconventionality", -0.35), ("H_modesty", 0.20), ("A_patience", 0.15)]),
        (ValueType::Artwork,      &[("O_aesthetic", 0.50), ("O_creativity", 0.30), ("O_unconventionality", 0.20)]),
        (ValueType::Cooperation,  &[("A_flexibility", 0.30), ("A_gentleness", 0.25), ("A_patience", 0.25), ("A_forgivingness", 0.20)]),
        (ValueType::Independence, &[("E_dependence", -0.35), ("X_social_self_esteem", 0.25), ("X_social_boldness", 0.20), ("O_unconventionality", 0.20)]),
        (ValueType::Stoicism,     &[("E_anxiety", -0.25), ("E_fearfulness", -0.25), ("E_sentimentality", -0.25), ("A_patience", 0.25)]),
        (ValueType::Introspection,&[("O_inquisitiveness", 0.30), ("O_aesthetic", 0.25), ("O_creativity", 0.25), ("C_prudence", 0.20)]),
        (ValueType::SelfControl,  &[("C_prudence", 0.35), ("C_perfectionism", 0.25), ("C_organization", 0.20), ("A_patience", 0.20)]),
        (ValueType::Tranquility,  &[("E_anxiety", -0.30), ("E_fearfulness", -0.20), ("A_patience", 0.30), ("A_gentleness", 0.20)]),
        (ValueType::Harmony,      &[("A_forgivingness", 0.25), ("A_flexibility", 0.25), ("A_gentleness", 0.25), ("A_patience", 0.25)]),
        (ValueType::Merriment,    &[("X_liveliness", 0.40), ("X_sociability", 0.25), ("X_social_self_esteem", 0.20), ("O_creativity", 0.15)]),
        (ValueType::Craftsmanship,&[("C_perfectionism", 0.35), ("C_diligence", 0.30), ("O_aesthetic", 0.20), ("C_organization", 0.15)]),
        (ValueType::MartialProwess,&[("X_social_boldness", 0.30), ("E_fearfulness", -0.30), ("E_anxiety", -0.20), ("C_diligence", 0.20)]),
        (ValueType::Skill,        &[("C_diligence", 0.30), ("C_perfectionism", 0.25), ("O_inquisitiveness", 0.25), ("O_creativity", 0.20)]),
        (ValueType::HardWork,     &[("C_diligence", 0.45), ("C_perfectionism", 0.25), ("C_organization", 0.15), ("C_prudence", 0.15)]),
        (ValueType::Sacrifice,    &[("H_greed_avoidance", 0.30), ("H_modesty", 0.25), ("E_sentimentality", 0.25), ("H_sincerity", 0.20)]),
        (ValueType::Competition,  &[("X_social_boldness", 0.30), ("X_social_self_esteem", 0.25), ("H_modesty", -0.25), ("X_liveliness", 0.20)]),
        (ValueType::Perseverance, &[("C_diligence", 0.35), ("A_patience", 0.30), ("E_fearfulness", -0.20), ("C_prudence", 0.15)]),
        (ValueType::Leisure,      &[("X_liveliness", 0.30), ("X_sociability", 0.25), ("C_diligence", -0.25), ("O_aesthetic", 0.20)]),
        (ValueType::Commerce,     &[("X_social_boldness", 0.25), ("H_greed_avoidance", -0.30), ("X_sociability", 0.25), ("C_organization", 0.20)]),
        (ValueType::Romance,      &[("E_sentimentality", 0.35), ("O_aesthetic", 0.25), ("X_sociability", 0.20), ("O_creativity", 0.20)]),
        (ValueType::Knowledge,    &[("O_inquisitiveness", 0.45), ("O_creativity", 0.25), ("C_diligence", 0.15), ("O_unconventionality", 0.15)]),
        (ValueType::Nature,       &[("O_aesthetic", 0.35), ("E_sentimentality", 0.25), ("O_inquisitiveness", 0.20), ("O_unconventionality", 0.20)]),
        (ValueType::Peace,        &[("A_gentleness", 0.30), ("A_forgivingness", 0.25), ("A_patience", 0.25), ("E_fearfulness", 0.20)]),
    ]
}

// ── Heritability ──────────────────────────────────────────────────────────────

/// Per-value heritability coefficient [Knafo & Schwartz 2004].
///
/// Range: 0.10 (conformity values, low genetic component)
///        to 0.20 (power/status values, higher genetic component).
/// Source: game_config.gd VALUE_HERITABILITY
pub fn value_heritability(v: ValueType) -> f64 {
    match v {
        ValueType::Tradition      => 0.10,
        ValueType::Law            => 0.10,
        ValueType::Decorum        => 0.11,
        ValueType::Loyalty        => 0.11,
        ValueType::Stoicism       => 0.11,
        ValueType::Harmony        => 0.12,
        ValueType::Peace          => 0.12,
        ValueType::Family         => 0.12,
        ValueType::Cooperation    => 0.12,
        ValueType::Sacrifice      => 0.12,
        ValueType::Fairness       => 0.13,
        ValueType::Friendship     => 0.13,
        ValueType::Truth          => 0.14,
        ValueType::Introspection  => 0.14,
        ValueType::Tranquility    => 0.14,
        ValueType::Commerce       => 0.15,
        ValueType::Knowledge      => 0.15,
        ValueType::Independence   => 0.15,
        ValueType::Nature         => 0.15,
        ValueType::Skill          => 0.15,
        ValueType::Romance        => 0.15,
        ValueType::Craftsmanship  => 0.16,
        ValueType::Eloquence      => 0.16,
        ValueType::Artwork        => 0.16,
        ValueType::Merriment      => 0.16,
        ValueType::Leisure        => 0.17,
        ValueType::Perseverance   => 0.17,
        ValueType::SelfControl    => 0.18,
        ValueType::HardWork       => 0.18,
        ValueType::Cunning        => 0.18,
        ValueType::Competition    => 0.19,
        ValueType::MartialProwess => 0.19,
        ValueType::Power          => 0.20,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn all_33_value_types_have_seed_entries() {
        let map = hexaco_seed_map();
        let map_types: std::collections::HashSet<_> = map.iter().map(|(vt, _)| *vt).collect();
        let mut missing = Vec::new();
        for vt in ValueType::iter() {
            if !map_types.contains(&vt) {
                missing.push(format!("{:?}", vt));
            }
        }
        assert!(missing.is_empty(), "Missing ValueTypes in hexaco_seed_map: {:?}", missing);
        assert_eq!(map.len(), 33, "Expected exactly 33 entries, got {}", map.len());
    }

    #[test]
    fn all_24_facet_names_resolve_to_valid_indices() {
        let map = hexaco_seed_map();
        let mut all_facet_names = std::collections::HashSet::new();
        for (_, weights) in map {
            for (facet_name, _) in *weights {
                all_facet_names.insert(*facet_name);
            }
        }
        for name in &all_facet_names {
            let idx = facet_index(name);
            assert!(idx.is_some(), "facet_index({:?}) returned None", name);
            assert!(idx.expect("is_some checked on line above") < 24, "facet_index({:?}) out of range: {:?}", name, idx);
        }
    }

    #[test]
    fn heritability_values_in_expected_range() {
        for vt in ValueType::iter() {
            let h = value_heritability(vt);
            assert!(
                (0.10..=0.20).contains(&h),
                "value_heritability({:?}) = {} is outside [0.10, 0.20]",
                vt, h
            );
        }
    }

    #[test]
    fn all_33_value_types_have_heritability() {
        // If any variant is missing from the match, this panics at compile time via non-exhaustive.
        // This test ensures the coverage is runtime-verified too.
        for vt in ValueType::iter() {
            let h = value_heritability(vt);
            assert!(h > 0.0, "heritability for {:?} is zero", vt);
        }
    }

    #[test]
    fn facet_index_all_24_canonical_names() {
        let names = [
            "H_sincerity", "H_fairness", "H_greed_avoidance", "H_modesty",
            "E_fearfulness", "E_anxiety", "E_dependence", "E_sentimentality",
            "X_social_self_esteem", "X_social_boldness", "X_sociability", "X_liveliness",
            "A_forgivingness", "A_gentleness", "A_flexibility", "A_patience",
            "C_organization", "C_diligence", "C_perfectionism", "C_prudence",
            "O_aesthetic", "O_inquisitiveness", "O_creativity", "O_unconventionality",
        ];
        let mut indices = std::collections::HashSet::new();
        for (i, name) in names.iter().enumerate() {
            let idx = facet_index(name).unwrap_or_else(|| panic!("facet_index({:?}) returned None", name));
            assert_eq!(idx, i, "facet_index({:?}) = {} but expected {}", name, idx, i);
            indices.insert(idx);
        }
        assert_eq!(indices.len(), 24, "Duplicate indices found");
    }
}
