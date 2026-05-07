//! Integration tests: RON file → loader → registry pipeline using the
//! checked-in `data/material/stone/granite.ron` fixture.
//!
//! Values must match `derivation::test_support::granite()` (single source
//! of truth for granite property values).

use std::path::PathBuf;

use sim_core::material::category::MaterialCategory;
use sim_core::material::id::MaterialId;
use sim_core::material::loader::{load_directory, load_ron};
use sim_core::material::registry::MaterialRegistry;
use sim_core::material::terrain::TerrainType;

fn granite_ron_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("material")
        .join("stone")
        .join("granite.ron")
}

fn stone_dir_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("material")
        .join("stone")
}

fn wood_dir_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("material")
        .join("wood")
}

#[test]
fn loads_granite_from_ron() {
    let defs = load_ron(&granite_ron_path()).expect("granite.ron must load");
    assert_eq!(defs.len(), 1, "granite.ron has exactly one material");
    let g = &defs[0];
    assert_eq!(g.id, MaterialId::from_str_hash("granite"));
    assert_eq!(g.name, "Granite");
    assert_eq!(g.category, MaterialCategory::Stone);
    assert_eq!(g.tier, 0);
    assert_eq!(g.natural_in, vec![TerrainType::Mountain]);
    assert!(g.mod_source.is_none());

    // Property values must match derivation::test_support::granite().
    let p = &g.properties;
    assert_eq!(p.density, 2_700.0);
    assert_eq!(p.hardness, 7.0);
    assert_eq!(p.shear_yield, 130_000.0);
    assert_eq!(p.impact_yield, 200_000.0);
    assert_eq!(p.fracture_toughness, 1_500.0);
    assert_eq!(p.melting_point, 1_260.0);
    assert_eq!(p.flammability, 0.0);
    assert_eq!(p.thermal_conductivity, 2.8);
    assert_eq!(p.cultural_value, 0.3);
    assert_eq!(p.rarity, 0.4);
    assert_eq!(p.distribution, vec![TerrainType::Mountain]);
    assert_eq!(p.work_difficulty, 0.6);
    assert_eq!(p.aesthetic_value, 0.5);
    assert_eq!(p.workability, 0.3);
    assert_eq!(p.preservation, 0.9);
}

#[test]
fn registry_with_loaded_granite() {
    let defs = load_ron(&granite_ron_path()).expect("granite.ron must load");
    let mut registry = MaterialRegistry::new();
    for def in defs {
        registry.register(def, None).expect("granite registers cleanly");
    }
    assert_eq!(registry.count(), 1);

    let granite_id = MaterialId::from_str_hash("granite");
    let g = registry.get(granite_id).expect("granite is in registry");
    assert_eq!(g.category, MaterialCategory::Stone);

    let stones: Vec<_> = registry.stones().collect();
    assert_eq!(stones.len(), 1);
    assert_eq!(stones[0].id, granite_id);

    assert_eq!(registry.woods().count(), 0);
    assert_eq!(registry.minerals().count(), 0);

    let counts = registry.category_counts();
    assert_eq!(counts.get(&MaterialCategory::Stone).copied(), Some(1));
}

#[test]
fn test_load_stone_directory() {
    let defs = load_directory(&stone_dir_path()).expect("stone/ must load");
    assert_eq!(
        defs.len(),
        30,
        "Stone 30 files 모두 로드 (granite + 29 new in T6.1)"
    );
}

#[test]
fn test_registry_with_30_stones() {
    let defs = load_directory(&stone_dir_path()).expect("stone/ must load");

    let mut registry = MaterialRegistry::new();
    for def in defs {
        registry.register(def, None).expect("register stone");
    }

    assert_eq!(registry.count(), 30, "registry holds 30 stones");

    let stones: Vec<_> = registry.stones().collect();
    assert_eq!(stones.len(), 30, "by_category Stone dispatcher returns 30");

    assert_eq!(registry.woods().count(), 0);
    assert_eq!(registry.animals().count(), 0);
    assert_eq!(registry.minerals().count(), 0);
    assert_eq!(registry.plants().count(), 0);
}

#[test]
fn test_stone_property_ranges_valid() {
    let defs = load_directory(&stone_dir_path()).expect("stone/ must load");

    for def in &defs {
        let p = &def.properties;
        assert!(
            p.density >= 100.0 && p.density <= 25_000.0,
            "{} density {} out of range",
            def.name,
            p.density
        );
        assert!(
            p.hardness >= 1.0 && p.hardness <= 10.0,
            "{} hardness {} out of range",
            def.name,
            p.hardness
        );
        assert!(
            p.shear_yield >= 1_000.0 && p.shear_yield <= 600_000.0,
            "{} shear_yield {} out of range",
            def.name,
            p.shear_yield
        );
        assert!(
            p.fracture_toughness >= 1_000.0 && p.fracture_toughness <= 800_000.0,
            "{} fracture_toughness {} out of range",
            def.name,
            p.fracture_toughness
        );
        assert!(
            p.flammability >= 0.0 && p.flammability <= 1.0,
            "{} flammability {} out of range",
            def.name,
            p.flammability
        );
    }
}

#[test]
fn test_load_specific_stones_by_name() {
    let defs = load_directory(&stone_dir_path()).expect("stone/ must load");

    let mut registry = MaterialRegistry::new();
    for def in defs {
        registry.register(def, None).expect("register");
    }

    let basalt = registry.stones().find(|m| m.name == "Basalt");
    assert!(basalt.is_some(), "basalt 찾기");

    let obsidian = registry.stones().find(|m| m.name == "Obsidian");
    assert!(obsidian.is_some(), "obsidian 찾기");

    let granite = registry.stones().find(|m| m.name == "Granite");
    assert!(granite.is_some(), "granite (T6.6 land) 찾기");

    let anthracite = registry.stones().find(|m| m.name == "Anthracite");
    assert!(anthracite.is_some(), "anthracite 찾기");
    assert!(
        anthracite.unwrap().properties.flammability > 0.9,
        "anthracite is combustible (석탄)"
    );
}

#[test]
fn test_load_wood_directory() {
    let defs = load_directory(&wood_dir_path()).expect("wood/ must load");
    assert_eq!(defs.len(), 25, "Wood 25 files 모두 로드 (T6.2)");
}

#[test]
fn test_registry_with_25_woods() {
    let defs = load_directory(&wood_dir_path()).expect("wood/ must load");

    let mut registry = MaterialRegistry::new();
    for def in defs {
        registry.register(def, None).expect("register wood");
    }

    assert_eq!(registry.count(), 25, "registry holds 25 woods");

    let woods: Vec<_> = registry.woods().collect();
    assert_eq!(woods.len(), 25, "by_category Wood dispatcher returns 25");

    assert_eq!(registry.stones().count(), 0);
    assert_eq!(registry.animals().count(), 0);
    assert_eq!(registry.minerals().count(), 0);
    assert_eq!(registry.plants().count(), 0);
}

#[test]
fn test_wood_property_ranges_valid() {
    let defs = load_directory(&wood_dir_path()).expect("wood/ must load");

    for def in &defs {
        let p = &def.properties;
        assert!(
            p.density >= 100.0 && p.density <= 25_000.0,
            "{} density {} out of range",
            def.name,
            p.density
        );
        assert!(
            p.hardness >= 1.0 && p.hardness <= 10.0,
            "{} hardness {} out of range",
            def.name,
            p.hardness
        );
        assert!(
            p.flammability >= 0.5,
            "{} flammability {} 의외 (wood는 모두 가연)",
            def.name,
            p.flammability
        );
        assert!(
            p.thermal_conductivity < 0.3,
            "{} thermal_conductivity {} 의외 (wood는 단열)",
            def.name,
            p.thermal_conductivity
        );
    }
}

#[test]
fn test_load_specific_woods_by_name() {
    let defs = load_directory(&wood_dir_path()).expect("wood/ must load");

    let mut registry = MaterialRegistry::new();
    for def in defs {
        registry.register(def, None).expect("register");
    }

    // T6.2.T1.1 발견: oak.ron must mirror derivation::test_support::oak()
    let oak = registry.woods().find(|m| m.name == "Oak");
    assert!(oak.is_some(), "oak 찾기");
    let oak_props = &oak.unwrap().properties;
    assert!(
        (oak_props.density - 750.0).abs() < 1e-6,
        "oak fixture density"
    );
    assert!(
        (oak_props.fracture_toughness - 4000.0).abs() < 1e-6,
        "oak fixture fracture_toughness (SSoT 일관)"
    );

    let pine = registry.woods().find(|m| m.name == "Pine");
    assert!(pine.is_some(), "pine 찾기");

    // bamboo 특이 case (grass 아닌 wood, 매우 높은 fracture_toughness)
    let bamboo = registry.woods().find(|m| m.name == "Bamboo");
    assert!(bamboo.is_some(), "bamboo (특이 case) 찾기");
    assert!(
        bamboo.unwrap().properties.fracture_toughness >= 35_000.0,
        "bamboo toughness 매우 높음"
    );

    // ebony 가장 dense
    let ebony = registry.woods().find(|m| m.name == "Ebony");
    assert!(ebony.is_some(), "ebony 찾기");
    assert!(
        ebony.unwrap().properties.density >= 1_000.0,
        "ebony density 가장 높음"
    );
}
