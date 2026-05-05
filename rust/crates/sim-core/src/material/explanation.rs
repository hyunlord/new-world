//! `PropertyKind` + `Explanation` — explain why each derived stat takes the
//! value it does, with input properties and a primary-source citation.

use crate::material::derivation::DerivedStatKind;

/// Tag enum for `MaterialProperties` fields. 1:1 with §3.E. Variant order
/// locked by §3.J.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyKind {
    /// `density`.
    Density,
    /// `hardness`.
    Hardness,
    /// `shear_yield`.
    ShearYield,
    /// `impact_yield`.
    ImpactYield,
    /// `fracture_toughness`.
    FractureToughness,
    /// `melting_point`.
    MeltingPoint,
    /// `flammability`.
    Flammability,
    /// `thermal_conductivity`.
    ThermalConductivity,
    /// `cultural_value`.
    CulturalValue,
    /// `rarity`.
    Rarity,
    /// `distribution`.
    Distribution,
    /// `work_difficulty`.
    WorkDifficulty,
    /// `aesthetic_value`.
    AestheticValue,
    /// `workability`.
    Workability,
    /// `preservation`.
    Preservation,
}

impl PropertyKind {
    /// Slice of all 15 variants in §3.J declaration order.
    pub const fn all_variants() -> &'static [PropertyKind] {
        use PropertyKind::*;
        &[
            Density,
            Hardness,
            ShearYield,
            ImpactYield,
            FractureToughness,
            MeltingPoint,
            Flammability,
            ThermalConductivity,
            CulturalValue,
            Rarity,
            Distribution,
            WorkDifficulty,
            AestheticValue,
            Workability,
            Preservation,
        ]
    }

    /// Snake-case name matching the corresponding `MaterialProperties` field.
    pub const fn snake_case(self) -> &'static str {
        use PropertyKind::*;
        match self {
            Density => "density",
            Hardness => "hardness",
            ShearYield => "shear_yield",
            ImpactYield => "impact_yield",
            FractureToughness => "fracture_toughness",
            MeltingPoint => "melting_point",
            Flammability => "flammability",
            ThermalConductivity => "thermal_conductivity",
            CulturalValue => "cultural_value",
            Rarity => "rarity",
            Distribution => "distribution",
            WorkDifficulty => "work_difficulty",
            AestheticValue => "aesthetic_value",
            Workability => "workability",
            Preservation => "preservation",
        }
    }
}

/// One derived stat's explanation: formula text, input property tags, and a
/// primary-source citation.
#[derive(Debug, Clone)]
pub struct Explanation {
    /// Stat being explained.
    pub stat: DerivedStatKind,
    /// Human-readable formula transcription.
    pub formula: &'static str,
    /// Property tags used by the formula, in argument order.
    pub inputs: Vec<PropertyKind>,
    /// Primary-source citation. MUST contain "DF" / "wiki" / "RimWorld" / "CRC".
    pub source: &'static str,
}

/// Return the locked explanation for a derived stat. Stable across runs.
pub fn explain(stat: DerivedStatKind) -> Explanation {
    use DerivedStatKind::*;
    use PropertyKind::*;
    let (formula, inputs, source) = match stat {
        AxeDamageBlunt => (
            "(impact_yield / 10000) * (density / 7800)",
            vec![ImpactYield, Density],
            "DF wiki Material_science: blunt damage scales with impact yield and density",
        ),
        AxeDamageCut => (
            "(shear_yield / 10000) * (1 + hardness / 20)",
            vec![ShearYield, Hardness],
            "DF wiki Material_science: cutting damage from shear yield, hardness multiplier",
        ),
        AxeDurability => (
            "hardness * fracture_toughness * (density / 1000)",
            vec![Hardness, FractureToughness, Density],
            "DF wiki Material_science: blade durability composite (Callister yield/fracture)",
        ),
        AxeSpeed => (
            "5 / (1 + density / 2500)",
            vec![Density],
            "DF wiki Material_science: swing speed inverse-proportional to mass",
        ),
        SwordDamageCut => (
            "(shear_yield / 9000) * (1 + hardness / 18)",
            vec![ShearYield, Hardness],
            "DF wiki Material_science: sword cut variant of shear-yield base formula",
        ),
        SwordDurability => (
            "hardness * fracture_toughness * 0.85",
            vec![Hardness, FractureToughness],
            "DF wiki Material_science: sword durability (Callister fracture toughness)",
        ),
        SpearDamagePierce => (
            "(shear_yield / 11000) * (1 + hardness / 22)",
            vec![ShearYield, Hardness],
            "DF wiki Material_science: pierce damage from shear yield + Mohs hardness",
        ),
        DaggerDamageCut => (
            "(shear_yield / 12000) * (1 + hardness / 25)",
            vec![ShearYield, Hardness],
            "DF wiki Material_science: dagger cut variant of shear-yield base formula",
        ),
        ArmorBlunt => (
            "(impact_yield / 10000) * (density / 8000)",
            vec![ImpactYield, Density],
            "DF wiki Material_science: armor blunt resistance from impact yield + density",
        ),
        ArmorSharp => (
            "(shear_yield / 9000) * (1 + hardness / 22)",
            vec![ShearYield, Hardness],
            "DF wiki Material_science: armor sharp resistance from shear yield + hardness",
        ),
        ArmorHeat => (
            "clamp(melting_point / 1500, 0, 4)",
            vec![MeltingPoint],
            "DF wiki Material_science: heat resistance (CRC Handbook melting points)",
        ),
        WallStrength => (
            "hardness * fracture_toughness / 1000",
            vec![Hardness, FractureToughness],
            "DF wiki Material_science: wall HP composite (Callister fracture toughness)",
        ),
        WallInsulation => (
            "(1 / (thermal_conductivity + 0.1)) * (density / 2000)",
            vec![ThermalConductivity, Density],
            "CRC Handbook of Chemistry and Physics: thermal R-value approximation",
        ),
        WallAesthetic => (
            "aesthetic_value * (1 + cultural_value)",
            vec![AestheticValue, CulturalValue],
            "RimWorld wiki Stuff: beauty multiplier blends aesthetic + cultural inputs",
        ),
        FloorAesthetic => (
            "aesthetic_value * (1 + workability * 0.5)",
            vec![AestheticValue, Workability],
            "RimWorld wiki Stuff: floor beauty rewards workable, attractive materials",
        ),
        BlockingWarmth => (
            "clamp(1 - thermal_conductivity / 400, 0, 1)",
            vec![ThermalConductivity],
            "CRC Handbook of Chemistry and Physics: insulation = inverse conductivity",
        ),
        BlockingLight => (
            "clamp(density / 25000, 0, 1)",
            vec![Density],
            "DF wiki Material_science: opaque-mass approximation (denser = darker)",
        ),
        BlockingNoise => (
            "clamp(density / 25000 + hardness / 20, 0, 1)",
            vec![Density, Hardness],
            "DF wiki Material_science: STL composite (mass + stiffness)",
        ),
        CraftTimeFactor => (
            "1 + work_difficulty",
            vec![WorkDifficulty],
            "RimWorld wiki StuffProperties: craft time scales with work difficulty",
        ),
        CraftQualityFactor => (
            "0.5 + workability * 0.75 + aesthetic_value * 0.25",
            vec![Workability, AestheticValue],
            "RimWorld wiki StuffProperties: quality multiplier blend (workability + beauty)",
        ),
        SharpDamageFactor => (
            "axe_damage_cut / IRON_AXE_CUT",
            vec![ShearYield, Hardness],
            "RimWorld wiki StuffProperties: sharp damage factor normalised to iron axe",
        ),
        BluntDamageFactor => (
            "axe_damage_blunt / IRON_AXE_BLUNT",
            vec![ImpactYield, Density],
            "RimWorld wiki StuffProperties: blunt damage factor normalised to iron axe",
        ),
        MaxHitPointsFactor => (
            "axe_durability / IRON_AXE_DURABILITY",
            vec![Hardness, FractureToughness, Density],
            "RimWorld wiki StuffProperties: max-HP factor normalised to iron baseline",
        ),
    };
    Explanation { stat, formula, inputs, source }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_kind_has_15_variants() {
        assert_eq!(PropertyKind::all_variants().len(), 15);
    }

    #[test]
    fn property_kind_snake_case_order_matches_spec() {
        let expected = [
            "density",
            "hardness",
            "shear_yield",
            "impact_yield",
            "fracture_toughness",
            "melting_point",
            "flammability",
            "thermal_conductivity",
            "cultural_value",
            "rarity",
            "distribution",
            "work_difficulty",
            "aesthetic_value",
            "workability",
            "preservation",
        ];
        let actual: Vec<&str> = PropertyKind::all_variants().iter().map(|k| k.snake_case()).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn explain_returns_well_formed_for_every_variant() {
        for k in DerivedStatKind::all_variants() {
            let exp = explain(*k);
            assert_eq!(exp.stat, *k);
            assert!(!exp.formula.is_empty());
            assert!(!exp.inputs.is_empty(), "{:?} has no inputs", k);
            assert!(
                exp.source.contains("DF")
                    || exp.source.contains("wiki")
                    || exp.source.contains("RimWorld")
                    || exp.source.contains("CRC"),
                "{:?} source missing keyword: {}",
                k,
                exp.source
            );
        }
    }

    #[test]
    fn explain_locks_specific_input_lists() {
        use DerivedStatKind::*;
        use PropertyKind::*;
        let cases = [
            (AxeDamageCut, vec![ShearYield, Hardness]),
            (AxeDamageBlunt, vec![ImpactYield, Density]),
            (AxeDurability, vec![Hardness, FractureToughness, Density]),
            (AxeSpeed, vec![Density]),
            (SwordDamageCut, vec![ShearYield, Hardness]),
            (ArmorBlunt, vec![ImpactYield, Density]),
            (WallStrength, vec![Hardness, FractureToughness]),
            (WallInsulation, vec![ThermalConductivity, Density]),
            (SharpDamageFactor, vec![ShearYield, Hardness]),
            (BluntDamageFactor, vec![ImpactYield, Density]),
        ];
        for (k, expected) in cases {
            let exp = explain(k);
            assert_eq!(exp.inputs, expected, "input mismatch for {:?}", k);
        }
    }

    #[test]
    fn explain_sources_substantive_distinct_with_keywords() {
        let sources: Vec<&str> = DerivedStatKind::all_variants()
            .iter()
            .map(|k| explain(*k).source)
            .collect();
        assert_eq!(sources.len(), 23);
        for s in &sources {
            assert!(s.len() >= 20, "source too short: {s}");
            assert!(s.contains(':'), "source missing colon: {s}");
        }
        let unique: std::collections::HashSet<&&str> = sources.iter().collect();
        assert!(
            unique.len() >= 18,
            "expected ≥18 distinct sources, got {}",
            unique.len()
        );
        let with_keywords = sources
            .iter()
            .filter(|s| {
                s.contains("DF")
                    || s.contains("wiki")
                    || s.contains("RimWorld")
                    || s.contains("Cloninger")
                    || s.contains("Phase 0")
            })
            .count();
        assert!(with_keywords >= 15, "only {with_keywords} sources have keywords");
    }
}
