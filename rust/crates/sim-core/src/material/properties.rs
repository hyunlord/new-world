//! `MaterialProperties` — exactly 15 named fields (14 numeric `f64` + 1 `Vec<TerrainType>`).
//!
//! Field order, names, types and inclusive ranges are locked by §3.E of the
//! material schema spec. Add/remove/reorder X.

use serde::{Deserialize, Serialize};

use crate::material::error::MaterialError;
use crate::material::terrain::TerrainType;

/// Physical, ecological and cultural properties of a material.
///
/// 14 numeric fields are bounded to inclusive ranges (see `validate`). The
/// 15th field, `distribution`, has no range constraint — empty and
/// duplicate entries are allowed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProperties {
    /// Mass per unit volume, kg/m³. Range `100..=25000`.
    pub density: f64,
    /// Mohs hardness. Range `1.0..=10.0`.
    pub hardness: f64,
    /// Shear yield strength, kPa. Range `1000..=600_000`.
    pub shear_yield: f64,
    /// Impact yield strength, kPa. Range `1000..=1_500_000`.
    pub impact_yield: f64,
    /// Fracture toughness, kPa. Range `1000..=800_000`.
    pub fracture_toughness: f64,
    /// Melting point, °C. Range `0.0..=3500.0`.
    pub melting_point: f64,
    /// Flammability (0 = noncombustible, 1 = highly flammable). `0.0..=1.0`.
    pub flammability: f64,
    /// Thermal conductivity, W/m·K. Range `0.04..=400.0`.
    pub thermal_conductivity: f64,
    /// Cultural value (subjective). Range `0.0..=1.0`.
    pub cultural_value: f64,
    /// Rarity (0 = common, 1 = mythic). Range `0.0..=1.0`.
    pub rarity: f64,
    /// Terrains in which the material naturally occurs. No range check.
    #[serde(default)]
    pub distribution: Vec<TerrainType>,
    /// Work difficulty (0 = easy, 1 = nearly intractable). Range `0.0..=1.0`.
    pub work_difficulty: f64,
    /// Aesthetic value. Range `0.0..=1.0`.
    pub aesthetic_value: f64,
    /// Workability (1 = easy to shape, 0 = unworkable). Range `0.0..=1.0`.
    pub workability: f64,
    /// Preservation factor (0 = decays fast, 1 = lasts forever). `0.0..=1.0`.
    pub preservation: f64,
}

impl MaterialProperties {
    /// Validate every numeric field against its inclusive range. NaN /
    /// non-finite values are rejected.
    pub fn validate(&self) -> Result<(), MaterialError> {
        check_range("density", self.density, 100.0, 25_000.0, "100..=25000")?;
        check_range("hardness", self.hardness, 1.0, 10.0, "1.0..=10.0")?;
        check_range("shear_yield", self.shear_yield, 1_000.0, 600_000.0, "1000..=600000")?;
        check_range("impact_yield", self.impact_yield, 1_000.0, 1_500_000.0, "1000..=1500000")?;
        check_range(
            "fracture_toughness",
            self.fracture_toughness,
            1_000.0,
            800_000.0,
            "1000..=800000",
        )?;
        check_range("melting_point", self.melting_point, 0.0, 3_500.0, "0.0..=3500.0")?;
        check_range("flammability", self.flammability, 0.0, 1.0, "0.0..=1.0")?;
        check_range(
            "thermal_conductivity",
            self.thermal_conductivity,
            0.04,
            400.0,
            "0.04..=400.0",
        )?;
        check_range("cultural_value", self.cultural_value, 0.0, 1.0, "0.0..=1.0")?;
        check_range("rarity", self.rarity, 0.0, 1.0, "0.0..=1.0")?;
        check_range("work_difficulty", self.work_difficulty, 0.0, 1.0, "0.0..=1.0")?;
        check_range("aesthetic_value", self.aesthetic_value, 0.0, 1.0, "0.0..=1.0")?;
        check_range("workability", self.workability, 0.0, 1.0, "0.0..=1.0")?;
        check_range("preservation", self.preservation, 0.0, 1.0, "0.0..=1.0")?;
        Ok(())
    }
}

fn check_range(
    property: &'static str,
    value: f64,
    min: f64,
    max: f64,
    expected: &'static str,
) -> Result<(), MaterialError> {
    if !value.is_finite() || value < min || value > max {
        return Err(MaterialError::PropertyOutOfRange {
            property,
            value,
            expected,
        });
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    /// Mid-range valid fixture used as a base for boundary / OOR tests.
    pub fn valid_props() -> MaterialProperties {
        MaterialProperties {
            density: 2_700.0,
            hardness: 5.0,
            shear_yield: 100_000.0,
            impact_yield: 200_000.0,
            fracture_toughness: 50_000.0,
            melting_point: 1_000.0,
            flammability: 0.5,
            thermal_conductivity: 5.0,
            cultural_value: 0.5,
            rarity: 0.5,
            distribution: vec![],
            work_difficulty: 0.5,
            aesthetic_value: 0.5,
            workability: 0.5,
            preservation: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::valid_props;
    use super::*;

    #[test]
    fn fifteen_named_fields_destructure_with_exact_types() {
        let props = valid_props();
        // Exhaustive type-annotated destructure — no `..`. Adding/removing/
        // retyping any field breaks compilation.
        let MaterialProperties {
            density: _x0,
            hardness: _x1,
            shear_yield: _x2,
            impact_yield: _x3,
            fracture_toughness: _x4,
            melting_point: _x5,
            flammability: _x6,
            thermal_conductivity: _x7,
            cultural_value: _x8,
            rarity: _x9,
            distribution: _x10,
            work_difficulty: _x11,
            aesthetic_value: _x12,
            workability: _x13,
            preservation: _x14,
        } = props;
        let _: f64 = _x0;
        let _: f64 = _x1;
        let _: f64 = _x2;
        let _: f64 = _x3;
        let _: f64 = _x4;
        let _: f64 = _x5;
        let _: f64 = _x6;
        let _: f64 = _x7;
        let _: f64 = _x8;
        let _: f64 = _x9;
        let _: Vec<TerrainType> = _x10;
        let _: f64 = _x11;
        let _: f64 = _x12;
        let _: f64 = _x13;
        let _: f64 = _x14;
    }

    #[test]
    fn valid_fixture_passes_validate() {
        assert!(valid_props().validate().is_ok());
    }

    // -------- Per-field boundary + OOR tests (14 × 3) --------

    macro_rules! boundary_case {
        ($name:ident, $field:ident, $min:expr, $max:expr, $oor:expr) => {
            #[test]
            fn $name() {
                let mut p = valid_props();
                p.$field = $min;
                assert!(p.validate().is_ok(), "min boundary should pass");
                let mut p = valid_props();
                p.$field = $max;
                assert!(p.validate().is_ok(), "max boundary should pass");
                let mut p = valid_props();
                p.$field = $oor;
                let err = p.validate().expect_err("OOR should fail");
                match err {
                    MaterialError::PropertyOutOfRange { property, value, expected } => {
                        assert_eq!(property, stringify!($field));
                        assert_eq!(value, $oor);
                        assert!(!expected.is_empty());
                    }
                    other => panic!("expected PropertyOutOfRange, got {other:?}"),
                }
            }
        };
    }

    boundary_case!(boundary_density, density, 100.0, 25_000.0, 99.0);
    boundary_case!(boundary_hardness, hardness, 1.0, 10.0, 10.5);
    boundary_case!(boundary_shear_yield, shear_yield, 1_000.0, 600_000.0, 600_001.0);
    boundary_case!(boundary_impact_yield, impact_yield, 1_000.0, 1_500_000.0, 1_500_001.0);
    boundary_case!(
        boundary_fracture_toughness,
        fracture_toughness,
        1_000.0,
        800_000.0,
        800_001.0
    );
    boundary_case!(boundary_melting_point, melting_point, 0.0, 3_500.0, 3_500.5);
    boundary_case!(boundary_flammability, flammability, 0.0, 1.0, 1.1);
    boundary_case!(
        boundary_thermal_conductivity,
        thermal_conductivity,
        0.04,
        400.0,
        400.1
    );
    boundary_case!(boundary_cultural_value, cultural_value, 0.0, 1.0, 1.5);
    boundary_case!(boundary_rarity, rarity, 0.0, 1.0, -0.1);
    boundary_case!(boundary_work_difficulty, work_difficulty, 0.0, 1.0, 1.1);
    boundary_case!(boundary_aesthetic_value, aesthetic_value, 0.0, 1.0, 1.1);
    boundary_case!(boundary_workability, workability, 0.0, 1.0, 1.1);
    boundary_case!(boundary_preservation, preservation, 0.0, 1.0, 1.1);

    // -------- Non-finite rejection (A15) --------

    fn assert_oor(err: MaterialError, expected_property: &str) {
        match err {
            MaterialError::PropertyOutOfRange { property, .. } => {
                assert_eq!(property, expected_property);
            }
            other => panic!("expected PropertyOutOfRange({expected_property}), got {other:?}"),
        }
    }

    #[test]
    fn density_nan_rejected() {
        let mut p = valid_props();
        p.density = f64::NAN;
        let err = p.validate().expect_err("nan must reject");
        assert_oor(err, "density");
    }

    #[test]
    fn hardness_infinity_rejected() {
        let mut p = valid_props();
        p.hardness = f64::INFINITY;
        let err = p.validate().expect_err("inf must reject");
        assert_oor(err, "hardness");
    }

    #[test]
    fn shear_yield_neg_infinity_rejected() {
        let mut p = valid_props();
        p.shear_yield = f64::NEG_INFINITY;
        let err = p.validate().expect_err("neg-inf must reject");
        assert_oor(err, "shear_yield");
    }

    // -------- Distribution skipped by validate (A16) --------

    #[test]
    fn distribution_empty_passes_validate() {
        let mut p = valid_props();
        p.distribution = vec![];
        assert!(p.validate().is_ok());
    }

    #[test]
    fn distribution_with_duplicates_passes_validate() {
        let mut p = valid_props();
        p.distribution = vec![TerrainType::Plain, TerrainType::Forest, TerrainType::Plain];
        assert!(p.validate().is_ok());
    }
}
