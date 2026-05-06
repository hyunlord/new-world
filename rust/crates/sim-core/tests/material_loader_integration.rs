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
fn loads_stone_directory_with_one_file() {
    let defs = load_directory(&stone_dir_path()).expect("stone/ must load");
    assert_eq!(defs.len(), 1, "stone/ currently holds only granite.ron");
    assert_eq!(defs[0].name, "Granite");
}
