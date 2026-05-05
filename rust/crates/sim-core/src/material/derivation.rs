//! `AutoDerivedStats` and `DerivedStatKind` — 23 stats, 1:1 correspondence.
//!
//! Field order locked by §3.G; variant order locked by §3.H. Iron baseline
//! constants live at the top per §3.I.

use serde::Serialize;

use crate::material::properties::MaterialProperties;

/// Iron axe cutting damage baseline (RimWorld factor normalisation reference).
pub const IRON_AXE_CUT: f64 = 11.6;
/// Iron axe blunt damage baseline.
pub const IRON_AXE_BLUNT: f64 = 62.96;
/// Iron axe durability baseline (HP per blade lifecycle).
pub const IRON_AXE_DURABILITY: f64 = 6745.0;

/// 23 derived stats per material. Field order locked by §3.G.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct AutoDerivedStats {
    /// Axe blunt damage component.
    pub axe_damage_blunt: f64,
    /// Axe cutting damage component.
    pub axe_damage_cut: f64,
    /// Axe durability (HP per lifecycle).
    pub axe_durability: f64,
    /// Axe swing speed factor.
    pub axe_speed: f64,
    /// Sword cutting damage.
    pub sword_damage_cut: f64,
    /// Sword durability.
    pub sword_durability: f64,
    /// Spear piercing damage.
    pub spear_damage_pierce: f64,
    /// Dagger cutting damage.
    pub dagger_damage_cut: f64,
    /// Armor blunt resistance.
    pub armor_blunt: f64,
    /// Armor sharp resistance.
    pub armor_sharp: f64,
    /// Armor heat resistance.
    pub armor_heat: f64,
    /// Wall structural strength.
    pub wall_strength: f64,
    /// Wall thermal insulation.
    pub wall_insulation: f64,
    /// Wall aesthetic score.
    pub wall_aesthetic: f64,
    /// Floor aesthetic score.
    pub floor_aesthetic: f64,
    /// Influence-grid warmth attenuation factor.
    pub blocking_warmth: f64,
    /// Influence-grid light attenuation factor.
    pub blocking_light: f64,
    /// Influence-grid noise attenuation factor.
    pub blocking_noise: f64,
    /// Crafting time multiplier.
    pub craft_time_factor: f64,
    /// Crafting quality multiplier.
    pub craft_quality_factor: f64,
    /// RimWorld-style sharp damage factor.
    pub sharp_damage_factor: f64,
    /// RimWorld-style blunt damage factor.
    pub blunt_damage_factor: f64,
    /// RimWorld-style max-HP factor.
    pub max_hit_points_factor: f64,
}

/// Tag enum for derived stats. 1:1 with `AutoDerivedStats`. Variant order
/// locked by §3.H.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DerivedStatKind {
    /// `axe_damage_blunt`.
    AxeDamageBlunt,
    /// `axe_damage_cut`.
    AxeDamageCut,
    /// `axe_durability`.
    AxeDurability,
    /// `axe_speed`.
    AxeSpeed,
    /// `sword_damage_cut`.
    SwordDamageCut,
    /// `sword_durability`.
    SwordDurability,
    /// `spear_damage_pierce`.
    SpearDamagePierce,
    /// `dagger_damage_cut`.
    DaggerDamageCut,
    /// `armor_blunt`.
    ArmorBlunt,
    /// `armor_sharp`.
    ArmorSharp,
    /// `armor_heat`.
    ArmorHeat,
    /// `wall_strength`.
    WallStrength,
    /// `wall_insulation`.
    WallInsulation,
    /// `wall_aesthetic`.
    WallAesthetic,
    /// `floor_aesthetic`.
    FloorAesthetic,
    /// `blocking_warmth`.
    BlockingWarmth,
    /// `blocking_light`.
    BlockingLight,
    /// `blocking_noise`.
    BlockingNoise,
    /// `craft_time_factor`.
    CraftTimeFactor,
    /// `craft_quality_factor`.
    CraftQualityFactor,
    /// `sharp_damage_factor`.
    SharpDamageFactor,
    /// `blunt_damage_factor`.
    BluntDamageFactor,
    /// `max_hit_points_factor`.
    MaxHitPointsFactor,
}

impl DerivedStatKind {
    /// Slice of all 23 variants in §3.H declaration order.
    pub const fn all_variants() -> &'static [DerivedStatKind] {
        use DerivedStatKind::*;
        &[
            AxeDamageBlunt,
            AxeDamageCut,
            AxeDurability,
            AxeSpeed,
            SwordDamageCut,
            SwordDurability,
            SpearDamagePierce,
            DaggerDamageCut,
            ArmorBlunt,
            ArmorSharp,
            ArmorHeat,
            WallStrength,
            WallInsulation,
            WallAesthetic,
            FloorAesthetic,
            BlockingWarmth,
            BlockingLight,
            BlockingNoise,
            CraftTimeFactor,
            CraftQualityFactor,
            SharpDamageFactor,
            BluntDamageFactor,
            MaxHitPointsFactor,
        ]
    }

    /// Snake-case name matching the corresponding `AutoDerivedStats` field.
    pub const fn snake_case(self) -> &'static str {
        use DerivedStatKind::*;
        match self {
            AxeDamageBlunt => "axe_damage_blunt",
            AxeDamageCut => "axe_damage_cut",
            AxeDurability => "axe_durability",
            AxeSpeed => "axe_speed",
            SwordDamageCut => "sword_damage_cut",
            SwordDurability => "sword_durability",
            SpearDamagePierce => "spear_damage_pierce",
            DaggerDamageCut => "dagger_damage_cut",
            ArmorBlunt => "armor_blunt",
            ArmorSharp => "armor_sharp",
            ArmorHeat => "armor_heat",
            WallStrength => "wall_strength",
            WallInsulation => "wall_insulation",
            WallAesthetic => "wall_aesthetic",
            FloorAesthetic => "floor_aesthetic",
            BlockingWarmth => "blocking_warmth",
            BlockingLight => "blocking_light",
            BlockingNoise => "blocking_noise",
            CraftTimeFactor => "craft_time_factor",
            CraftQualityFactor => "craft_quality_factor",
            SharpDamageFactor => "sharp_damage_factor",
            BluntDamageFactor => "blunt_damage_factor",
            MaxHitPointsFactor => "max_hit_points_factor",
        }
    }

    /// Read the corresponding numeric field out of `AutoDerivedStats`.
    pub const fn read(self, s: &AutoDerivedStats) -> f64 {
        use DerivedStatKind::*;
        match self {
            AxeDamageBlunt => s.axe_damage_blunt,
            AxeDamageCut => s.axe_damage_cut,
            AxeDurability => s.axe_durability,
            AxeSpeed => s.axe_speed,
            SwordDamageCut => s.sword_damage_cut,
            SwordDurability => s.sword_durability,
            SpearDamagePierce => s.spear_damage_pierce,
            DaggerDamageCut => s.dagger_damage_cut,
            ArmorBlunt => s.armor_blunt,
            ArmorSharp => s.armor_sharp,
            ArmorHeat => s.armor_heat,
            WallStrength => s.wall_strength,
            WallInsulation => s.wall_insulation,
            WallAesthetic => s.wall_aesthetic,
            FloorAesthetic => s.floor_aesthetic,
            BlockingWarmth => s.blocking_warmth,
            BlockingLight => s.blocking_light,
            BlockingNoise => s.blocking_noise,
            CraftTimeFactor => s.craft_time_factor,
            CraftQualityFactor => s.craft_quality_factor,
            SharpDamageFactor => s.sharp_damage_factor,
            BluntDamageFactor => s.blunt_damage_factor,
            MaxHitPointsFactor => s.max_hit_points_factor,
        }
    }
}

/// Derive every stat from a material's properties. Each formula transcribes
/// Phase 0 v0.3 §3.2 verbatim — DF for absolute physical values, RimWorld
/// for the factor-normalisation pattern.
pub fn derive_all(p: &MaterialProperties) -> AutoDerivedStats {
    // Absolute weapon stats (DF-style).
    let axe_damage_cut = (p.shear_yield / 10_000.0) * (1.0 + p.hardness / 20.0);
    let axe_damage_blunt = (p.impact_yield / 10_000.0) * (p.density / 7_800.0);
    let axe_durability = p.hardness * p.fracture_toughness * (p.density / 1_000.0);
    let axe_speed = 5.0 / (1.0 + p.density / 2_500.0);

    let sword_damage_cut = (p.shear_yield / 9_000.0) * (1.0 + p.hardness / 18.0);
    let sword_durability = p.hardness * p.fracture_toughness * 0.85;
    let spear_damage_pierce = (p.shear_yield / 11_000.0) * (1.0 + p.hardness / 22.0);
    let dagger_damage_cut = (p.shear_yield / 12_000.0) * (1.0 + p.hardness / 25.0);

    // Armor.
    let armor_blunt = (p.impact_yield / 10_000.0) * (p.density / 8_000.0);
    let armor_sharp = (p.shear_yield / 9_000.0) * (1.0 + p.hardness / 22.0);
    let armor_heat = (p.melting_point / 1_500.0).clamp(0.0, 4.0);

    // Building.
    let wall_strength = p.hardness * p.fracture_toughness / 1_000.0;
    let wall_insulation = (1.0 / (p.thermal_conductivity + 0.1)) * (p.density / 2_000.0);
    let wall_aesthetic = p.aesthetic_value * (1.0 + p.cultural_value);
    let floor_aesthetic = p.aesthetic_value * (1.0 + p.workability * 0.5);

    // Influence-grid blocking.
    let blocking_warmth = (1.0 - (p.thermal_conductivity / 400.0)).clamp(0.0, 1.0);
    let blocking_light = (p.density / 25_000.0).clamp(0.0, 1.0);
    let blocking_noise = ((p.density / 25_000.0) + (p.hardness / 20.0)).clamp(0.0, 1.0);

    // Crafting.
    let craft_time_factor = 1.0 + p.work_difficulty;
    let craft_quality_factor = 0.5 + p.workability * 0.75 + p.aesthetic_value * 0.25;

    // RimWorld-style normalised factors (iron axe baseline = 1.0).
    let sharp_damage_factor = axe_damage_cut / IRON_AXE_CUT;
    let blunt_damage_factor = axe_damage_blunt / IRON_AXE_BLUNT;
    let max_hit_points_factor = axe_durability / IRON_AXE_DURABILITY;

    AutoDerivedStats {
        axe_damage_blunt,
        axe_damage_cut,
        axe_durability,
        axe_speed,
        sword_damage_cut,
        sword_durability,
        spear_damage_pierce,
        dagger_damage_cut,
        armor_blunt,
        armor_sharp,
        armor_heat,
        wall_strength,
        wall_insulation,
        wall_aesthetic,
        floor_aesthetic,
        blocking_warmth,
        blocking_light,
        blocking_noise,
        craft_time_factor,
        craft_quality_factor,
        sharp_damage_factor,
        blunt_damage_factor,
        max_hit_points_factor,
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use crate::material::terrain::TerrainType;

    pub fn granite() -> MaterialProperties {
        MaterialProperties {
            density: 2_700.0,
            hardness: 7.0,
            shear_yield: 130_000.0,
            impact_yield: 200_000.0,
            fracture_toughness: 1_500.0,
            melting_point: 1_260.0,
            flammability: 0.0,
            thermal_conductivity: 2.8,
            cultural_value: 0.3,
            rarity: 0.4,
            distribution: vec![TerrainType::Mountain],
            work_difficulty: 0.6,
            aesthetic_value: 0.5,
            workability: 0.3,
            preservation: 0.9,
        }
    }

    pub fn oak() -> MaterialProperties {
        MaterialProperties {
            density: 750.0,
            hardness: 4.0,
            shear_yield: 40_000.0,
            impact_yield: 70_000.0,
            fracture_toughness: 4_000.0,
            melting_point: 300.0,
            flammability: 0.8,
            thermal_conductivity: 0.17,
            cultural_value: 0.4,
            rarity: 0.5,
            distribution: vec![TerrainType::Forest],
            work_difficulty: 0.3,
            aesthetic_value: 0.6,
            workability: 0.7,
            preservation: 0.4,
        }
    }

    /// Iron-equivalent fixture: properties chosen so that `derive_all`
    /// reproduces the locked baselines `IRON_AXE_CUT`, `IRON_AXE_BLUNT`,
    /// `IRON_AXE_DURABILITY` exactly (within f64 rounding).
    ///
    /// Derivation (each line documents one closed-form back-solve):
    ///
    /// - `axe_durability = hardness * fracture_toughness * (density / 1000)`
    ///   so `1.0 * 1349.0 * (5000.0 / 1000.0) = 6745.0`.
    /// - `axe_damage_blunt = (impact_yield / 10000) * (density / 7800)`
    ///   so `(982176.0 / 10000) * (5000.0 / 7800.0) = 62.96`.
    /// - `axe_damage_cut = (shear_yield / 10000) * (1 + hardness / 20)`
    ///   so `(110476.19 / 10000) * 1.05 = 11.6` (within f64 rounding).
    ///
    /// All 14 numeric fields stay inside their §3.E ranges.
    pub fn iron_equivalent() -> MaterialProperties {
        MaterialProperties {
            density: 5_000.0,
            hardness: 1.0,
            shear_yield: 110_476.190_476_190_47,
            impact_yield: 982_176.0,
            fracture_toughness: 1_349.0,
            melting_point: 1_500.0,
            flammability: 0.0,
            thermal_conductivity: 80.0,
            cultural_value: 0.5,
            rarity: 0.5,
            distribution: vec![],
            work_difficulty: 0.5,
            aesthetic_value: 0.5,
            workability: 0.5,
            preservation: 0.5,
        }
    }

    pub fn obsidian() -> MaterialProperties {
        MaterialProperties {
            density: 2_400.0,
            hardness: 5.5,
            shear_yield: 80_000.0,
            impact_yield: 100_000.0,
            fracture_toughness: 1_000.0,
            melting_point: 1_000.0,
            flammability: 0.0,
            thermal_conductivity: 1.4,
            cultural_value: 0.5,
            rarity: 0.6,
            distribution: vec![TerrainType::Mountain],
            work_difficulty: 0.7,
            aesthetic_value: 0.7,
            workability: 0.4,
            preservation: 0.95,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;

    #[test]
    fn iron_baselines_have_locked_values() {
        assert_eq!(IRON_AXE_CUT, 11.6);
        assert_eq!(IRON_AXE_BLUNT, 62.96);
        assert_eq!(IRON_AXE_DURABILITY, 6745.0);
    }

    #[test]
    fn all_variants_has_23_entries() {
        assert_eq!(DerivedStatKind::all_variants().len(), 23);
    }

    #[test]
    fn snake_case_matches_struct_field_for_each_variant() {
        // Order parity with §3.G.
        let expected = [
            "axe_damage_blunt",
            "axe_damage_cut",
            "axe_durability",
            "axe_speed",
            "sword_damage_cut",
            "sword_durability",
            "spear_damage_pierce",
            "dagger_damage_cut",
            "armor_blunt",
            "armor_sharp",
            "armor_heat",
            "wall_strength",
            "wall_insulation",
            "wall_aesthetic",
            "floor_aesthetic",
            "blocking_warmth",
            "blocking_light",
            "blocking_noise",
            "craft_time_factor",
            "craft_quality_factor",
            "sharp_damage_factor",
            "blunt_damage_factor",
            "max_hit_points_factor",
        ];
        let actual: Vec<&str> = DerivedStatKind::all_variants().iter().map(|k| k.snake_case()).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn derive_iron_equivalent_matches_locked_baselines_within_tolerance() {
        let props = iron_equivalent();
        assert!(props.validate().is_ok(), "iron_equivalent fixture must validate");
        let stats = derive_all(&props);
        let tol = 1e-6;
        assert!(
            (stats.axe_damage_cut - IRON_AXE_CUT).abs() < tol,
            "axe_damage_cut = {} expected {}",
            stats.axe_damage_cut,
            IRON_AXE_CUT,
        );
        assert!(
            (stats.axe_damage_blunt - IRON_AXE_BLUNT).abs() < tol,
            "axe_damage_blunt = {} expected {}",
            stats.axe_damage_blunt,
            IRON_AXE_BLUNT,
        );
        assert!(
            (stats.axe_durability - IRON_AXE_DURABILITY).abs() < tol,
            "axe_durability = {} expected {}",
            stats.axe_durability,
            IRON_AXE_DURABILITY,
        );
    }

    #[test]
    fn derive_all_produces_finite_values_for_three_fixtures() {
        for (name, props) in [("granite", granite()), ("oak", oak()), ("obsidian", obsidian())] {
            assert!(props.validate().is_ok(), "{name} fixture must be valid before derive");
            let stats = derive_all(&props);
            for kind in DerivedStatKind::all_variants() {
                let v = kind.read(&stats);
                assert!(v.is_finite() && !v.is_nan(), "{name}.{} = {v}", kind.snake_case());
            }
        }
    }

    #[test]
    fn derive_outputs_are_pairwise_distinct_for_six_representative_stats() {
        let g = derive_all(&granite());
        let o = derive_all(&oak());
        let b = derive_all(&obsidian());
        let representatives = [
            DerivedStatKind::AxeDamageCut,
            DerivedStatKind::AxeDurability,
            DerivedStatKind::SwordDamageCut,
            DerivedStatKind::WallStrength,
            DerivedStatKind::ArmorBlunt,
            DerivedStatKind::CraftQualityFactor,
        ];
        for k in representatives {
            let gv = k.read(&g);
            let ov = k.read(&o);
            let bv = k.read(&b);
            let name = k.snake_case();
            assert!((gv - ov).abs() > 1e-6, "{name}: granite vs oak");
            assert!((gv - bv).abs() > 1e-6, "{name}: granite vs obsidian");
            assert!((ov - bv).abs() > 1e-6, "{name}: oak vs obsidian");
        }
    }
}
