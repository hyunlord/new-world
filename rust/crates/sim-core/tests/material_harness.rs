//! T6.7 — Material RON 105 harness tests.
//!
//! Eight assertions covering the full T6.1~T6.5 audit chain:
//!   load count, validate ranges, category balance, tier distribution,
//!   boundary clamp audit, fixture SSoT, id uniqueness, terrain validity.

use std::collections::HashSet;
use std::path::Path;

use sim_core::material::{load_directory, MaterialRegistry, TerrainType};

const DATA_DIR: &str = "data/material";

fn load_all_categories() -> MaterialRegistry {
    let mut registry = MaterialRegistry::default();
    for category in &["stone", "wood", "animal", "mineral", "plant"] {
        let dir = Path::new(DATA_DIR).join(category);
        let materials = load_directory(&dir).expect("load category");
        for def in materials {
            registry.register(def, None).expect("register");
        }
    }
    registry
}

#[test]
fn harness_material_load_all_105() {
    let registry = load_all_categories();
    assert_eq!(
        registry.count(),
        105,
        "Material RON 105 (실측, 30+25+20+15+15)"
    );
}

#[test]
fn harness_material_validate_no_failure() {
    let registry = load_all_categories();

    for id in registry.all_ids() {
        let def = registry.get(id).expect("def");
        let p = &def.properties;

        assert!(p.density >= 100.0 && p.density <= 25_000.0, "{} density", def.name);
        assert!(p.hardness >= 1.0 && p.hardness <= 10.0, "{} hardness", def.name);
        assert!(
            p.shear_yield >= 1_000.0 && p.shear_yield <= 600_000.0,
            "{} shear_yield",
            def.name
        );
        assert!(
            p.impact_yield >= 1_000.0 && p.impact_yield <= 1_500_000.0,
            "{} impact_yield",
            def.name
        );
        assert!(
            p.fracture_toughness >= 1_000.0 && p.fracture_toughness <= 800_000.0,
            "{} fracture_toughness",
            def.name
        );
        assert!(
            p.melting_point >= 0.0 && p.melting_point <= 3_500.0,
            "{} melting_point",
            def.name
        );
        assert!(
            p.flammability >= 0.0 && p.flammability <= 1.0,
            "{} flammability",
            def.name
        );
        assert!(
            p.thermal_conductivity >= 0.04 && p.thermal_conductivity <= 400.0,
            "{} thermal_conductivity",
            def.name
        );
        assert!(
            p.cultural_value >= 0.0 && p.cultural_value <= 1.0,
            "{} cultural_value",
            def.name
        );
        assert!(p.rarity >= 0.0 && p.rarity <= 1.0, "{} rarity", def.name);
        assert!(
            p.work_difficulty >= 0.0 && p.work_difficulty <= 1.0,
            "{} work_difficulty",
            def.name
        );
        assert!(
            p.aesthetic_value >= 0.0 && p.aesthetic_value <= 1.0,
            "{} aesthetic_value",
            def.name
        );
        assert!(
            p.workability >= 0.0 && p.workability <= 1.0,
            "{} workability",
            def.name
        );
        assert!(
            p.preservation >= 0.0 && p.preservation <= 1.0,
            "{} preservation",
            def.name
        );
    }
}

#[test]
fn harness_material_categories_balance() {
    let registry = load_all_categories();

    assert_eq!(registry.stones().count(), 30, "Stone 30");
    assert_eq!(registry.woods().count(), 25, "Wood 25");
    assert_eq!(registry.animals().count(), 20, "Animal 20");
    assert_eq!(registry.minerals().count(), 15, "Mineral 15");
    assert_eq!(registry.plants().count(), 15, "Plant 15");

    let total = registry.stones().count()
        + registry.woods().count()
        + registry.animals().count()
        + registry.minerals().count()
        + registry.plants().count();
    assert_eq!(total, 105, "총 105 (design '100'은 round number)");
}

#[test]
fn harness_material_tier_distribution() {
    let registry = load_all_categories();

    let mut tier_0 = 0usize;
    let mut tier_1 = 0usize;
    for id in registry.all_ids() {
        let def = registry.get(id).expect("def");
        match def.tier {
            0 => tier_0 += 1,
            1 => tier_1 += 1,
            other => panic!("{} unexpected tier {}", def.name, other),
        }
    }

    assert_eq!(tier_0, 81, "Tier 0 (Stone Age EA materials)");
    assert_eq!(tier_1, 24, "Tier 1 (Bronze Age+ materials)");
    assert_eq!(tier_0 + tier_1, 105);
}

#[test]
fn harness_material_boundary_clamp_audit() {
    let registry = load_all_categories();

    let feather = registry
        .animals()
        .find(|m| m.name == "Feather")
        .expect("feather");
    assert!(
        (feather.properties.density - 100.0).abs() < 1e-6,
        "T6.3 Q5-1: feather density boundary clamp (100, was 80)"
    );
    assert!(
        (feather.properties.thermal_conductivity - 0.04).abs() < 1e-6,
        "T6.3 Q5-2: feather thermal_conductivity boundary clamp (0.04, was 0.034)"
    );

    let silver_ore = registry
        .minerals()
        .find(|m| m.name == "Silver Ore")
        .expect("silver_ore");
    assert!(
        (silver_ore.properties.thermal_conductivity - 400.0).abs() < 1e-6,
        "T6.4 Q6: silver_ore thermal_conductivity boundary clamp (400, was 420)"
    );

    let straw = registry
        .plants()
        .find(|m| m.name == "Straw")
        .expect("straw");
    assert!(
        (straw.properties.density - 100.0).abs() < 1e-6,
        "T6.5 Q7: straw density boundary clamp (100, was 80)"
    );
}

#[test]
fn harness_material_fixture_consistency() {
    let registry = load_all_categories();

    let granite = registry
        .stones()
        .find(|m| m.name == "Granite")
        .expect("granite");
    assert!(
        (granite.properties.density - 2700.0).abs() < 1e-6,
        "T6.6 granite fixture: density"
    );
    assert!(
        (granite.properties.hardness - 7.0).abs() < 1e-6,
        "T6.6 granite fixture: hardness"
    );
    assert!(
        (granite.properties.fracture_toughness - 1500.0).abs() < 1e-6,
        "T6.6 granite fixture: fracture_toughness"
    );

    let oak = registry.woods().find(|m| m.name == "Oak").expect("oak");
    assert!(
        (oak.properties.density - 750.0).abs() < 1e-6,
        "T6.2 oak fixture: density"
    );
    assert!(
        (oak.properties.hardness - 4.0).abs() < 1e-6,
        "T6.2 oak fixture: hardness"
    );
    assert!(
        (oak.properties.fracture_toughness - 4000.0).abs() < 1e-6,
        "T6.2 oak fixture: fracture_toughness"
    );
}

#[test]
fn harness_material_no_duplicate_id() {
    let registry = load_all_categories();

    let ids: Vec<_> = registry.all_ids().collect();
    let unique = ids.iter().collect::<HashSet<_>>().len();

    assert_eq!(ids.len(), unique, "105 materials no duplicate id");
    assert_eq!(ids.len(), 105);
}

#[test]
fn harness_material_terrain_distribution_valid() {
    let registry = load_all_categories();

    let valid: HashSet<TerrainType> = [
        TerrainType::Plain,
        TerrainType::Forest,
        TerrainType::Mountain,
        TerrainType::Hill,
        TerrainType::River,
        TerrainType::Coast,
        TerrainType::Desert,
        TerrainType::Tundra,
        TerrainType::Swamp,
        TerrainType::Cave,
    ]
    .into_iter()
    .collect();

    for id in registry.all_ids() {
        let def = registry.get(id).expect("def");
        for terrain in &def.properties.distribution {
            assert!(
                valid.contains(terrain),
                "{} distribution {:?} invalid TerrainType",
                def.name,
                terrain
            );
        }
        for terrain in &def.natural_in {
            assert!(
                valid.contains(terrain),
                "{} natural_in {:?} invalid TerrainType",
                def.name,
                terrain
            );
        }
    }
}
