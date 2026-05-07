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

fn animal_dir_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("material")
        .join("animal")
}

fn mineral_dir_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("material")
        .join("mineral")
}

fn plant_dir_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("material")
        .join("plant")
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

#[test]
fn test_load_animal_directory() {
    let defs = load_directory(&animal_dir_path()).expect("animal/ must load");
    assert_eq!(defs.len(), 20, "Animal 20 files 모두 로드 (T6.3)");
}

#[test]
fn test_registry_with_20_animals() {
    let defs = load_directory(&animal_dir_path()).expect("animal/ must load");

    let mut registry = MaterialRegistry::new();
    for def in defs {
        registry.register(def, None).expect("register animal");
    }

    assert_eq!(registry.count(), 20, "registry holds 20 animals");

    let animals: Vec<_> = registry.animals().collect();
    assert_eq!(animals.len(), 20, "by_category Animal dispatcher returns 20");

    assert_eq!(registry.stones().count(), 0);
    assert_eq!(registry.woods().count(), 0);
    assert_eq!(registry.minerals().count(), 0);
    assert_eq!(registry.plants().count(), 0);
}

#[test]
fn test_animal_property_ranges_valid() {
    let defs = load_directory(&animal_dir_path()).expect("animal/ must load");

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
            p.thermal_conductivity >= 0.04,
            "{} thermal_conductivity {} below boundary",
            def.name,
            p.thermal_conductivity
        );
    }
}

#[test]
fn test_load_specific_animals_by_name() {
    let defs = load_directory(&animal_dir_path()).expect("animal/ must load");

    let mut registry = MaterialRegistry::new();
    for def in defs {
        registry.register(def, None).expect("register");
    }

    let leather_cow = registry.animals().find(|m| m.name == "Leather Cow");
    assert!(leather_cow.is_some(), "leather_cow (Leather Cow) 찾기");

    // ivory 가장 cultural_value 높음 (Tier 1)
    let ivory = registry.animals().find(|m| m.name == "Ivory");
    assert!(ivory.is_some(), "ivory 찾기");
    assert!(
        ivory.unwrap().properties.cultural_value >= 0.9,
        "ivory cultural_value 매우 높음"
    );

    // ★ feather Q5 boundary clamp 명시 검증 (audit trail)
    let feather = registry.animals().find(|m| m.name == "Feather");
    assert!(feather.is_some(), "feather 찾기");
    assert!(
        (feather.unwrap().properties.density - 100.0).abs() < 1e-6,
        "feather density boundary clamp 100.0 정확 (Q5-1)"
    );
    assert!(
        (feather.unwrap().properties.thermal_conductivity - 0.04).abs() < 1e-6,
        "feather thermal_conductivity boundary clamp 0.04 정확 (Q5-2)"
    );

    // shell calcium carbonate, 비가연
    let shell = registry.animals().find(|m| m.name == "Shell");
    assert!(shell.is_some(), "shell 찾기");
    assert!(
        shell.unwrap().properties.flammability < 0.1,
        "shell calcium carbonate 비가연"
    );

    // bone calcium hydroxyapatite, melting 1670
    let bone = registry.animals().find(|m| m.name == "Bone");
    assert!(bone.is_some(), "bone 찾기");
    assert!(
        bone.unwrap().properties.melting_point > 1500.0,
        "bone calcium hydroxyapatite high melting"
    );
}

#[test]
fn test_load_mineral_directory() {
    let defs = load_directory(&mineral_dir_path()).expect("mineral/ must load");
    assert_eq!(defs.len(), 15, "Mineral 15 files 모두 로드 (T6.4)");
}

#[test]
fn test_registry_with_15_minerals() {
    let defs = load_directory(&mineral_dir_path()).expect("mineral/ must load");

    let mut registry = MaterialRegistry::new();
    for def in defs {
        registry.register(def, None).expect("register mineral");
    }

    assert_eq!(registry.count(), 15, "registry holds 15 minerals");

    let minerals: Vec<_> = registry.minerals().collect();
    assert_eq!(minerals.len(), 15, "by_category Mineral dispatcher returns 15");

    assert_eq!(registry.stones().count(), 0);
    assert_eq!(registry.woods().count(), 0);
    assert_eq!(registry.animals().count(), 0);
    assert_eq!(registry.plants().count(), 0);
}

#[test]
fn test_mineral_property_ranges_valid() {
    let defs = load_directory(&mineral_dir_path()).expect("mineral/ must load");

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
        // ★ thermal_conductivity boundary 검증 (silver_ore Q6 핵심)
        assert!(
            p.thermal_conductivity >= 0.04 && p.thermal_conductivity <= 400.0,
            "{} thermal_conductivity {} out of range (Q6 boundary)",
            def.name,
            p.thermal_conductivity
        );
    }
}

#[test]
fn test_load_specific_minerals_by_name() {
    let defs = load_directory(&mineral_dir_path()).expect("mineral/ must load");

    let mut registry = MaterialRegistry::new();
    for def in defs {
        registry.register(def, None).expect("register");
    }

    // clay (Stone Age 핵심, workability 가장 높음)
    let clay = registry.minerals().find(|m| m.name == "Clay");
    assert!(clay.is_some(), "clay 찾기");
    assert!(
        clay.unwrap().properties.workability >= 0.9,
        "clay 가장 workable (0.95)"
    );

    // ★ silver_ore Q6 boundary clamp 명시 검증 (audit trail, 1e-6)
    let silver_ore = registry.minerals().find(|m| m.name == "Silver Ore");
    assert!(silver_ore.is_some(), "silver_ore 찾기");
    assert!(
        (silver_ore.unwrap().properties.thermal_conductivity - 400.0).abs() < 1e-6,
        "silver_ore thermal_conductivity Q6 boundary clamp 정확 400.0 (Q6)"
    );

    // gold_ore 가장 cultural_value (Tier 1, 화폐)
    let gold_ore = registry.minerals().find(|m| m.name == "Gold Ore");
    assert!(gold_ore.is_some(), "gold_ore 찾기");
    assert!(
        gold_ore.unwrap().properties.cultural_value >= 0.9,
        "gold_ore cultural_value 매우 높음 (0.95)"
    );

    // sulfur 가장 flammability (mineral 중, 화약)
    let sulfur = registry.minerals().find(|m| m.name == "Sulfur");
    assert!(sulfur.is_some(), "sulfur 찾기");
    assert!(
        sulfur.unwrap().properties.flammability >= 0.9,
        "sulfur 매우 가연 (0.95, 화약)"
    );

    // soil 가장 흔함 (rarity 0.05)
    let soil = registry.minerals().find(|m| m.name == "Soil");
    assert!(soil.is_some(), "soil 찾기");
    assert!(
        soil.unwrap().properties.rarity <= 0.1,
        "soil 가장 흔함 (rarity 0.05)"
    );
}

#[test]
fn test_load_plant_directory() {
    let defs = load_directory(&plant_dir_path()).expect("plant/ must load");
    assert_eq!(defs.len(), 15, "Plant 15 files 모두 로드 (T6.5)");
}

#[test]
fn test_registry_with_15_plants() {
    let defs = load_directory(&plant_dir_path()).expect("plant/ must load");

    let mut registry = MaterialRegistry::new();
    for def in defs {
        registry.register(def, None).expect("register plant");
    }

    assert_eq!(registry.count(), 15, "registry holds 15 plants");

    let plants: Vec<_> = registry.plants().collect();
    assert_eq!(plants.len(), 15, "by_category Plant dispatcher returns 15");

    assert_eq!(registry.stones().count(), 0);
    assert_eq!(registry.woods().count(), 0);
    assert_eq!(registry.animals().count(), 0);
    assert_eq!(registry.minerals().count(), 0);
}

#[test]
fn test_plant_property_ranges_valid() {
    let defs = load_directory(&plant_dir_path()).expect("plant/ must load");

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
        // 모든 plant 가연 (flammability >= 0.6)
        assert!(
            p.flammability >= 0.6,
            "{} flammability {} 의외 (plant는 모두 가연)",
            def.name,
            p.flammability
        );
        // 모든 plant 단열 (thermal_conductivity <= 0.6, root_food 0.55가 최대)
        assert!(
            p.thermal_conductivity <= 0.6,
            "{} thermal_conductivity {} 의외 (plant는 단열)",
            def.name,
            p.thermal_conductivity
        );
    }
}

#[test]
fn test_load_specific_plants_by_name() {
    let defs = load_directory(&plant_dir_path()).expect("plant/ must load");

    let mut registry = MaterialRegistry::new();
    for def in defs {
        registry.register(def, None).expect("register");
    }

    // ★ straw Q7 boundary clamp 명시 검증 (audit trail, 1e-6)
    let straw = registry.plants().find(|m| m.name == "Straw");
    assert!(straw.is_some(), "straw 찾기");
    assert!(
        (straw.unwrap().properties.density - 100.0).abs() < 1e-6,
        "straw density Q7 boundary clamp 정확 100.0 (Q7)"
    );

    // papyrus 가장 cultural_value 높음 (이집트 기록)
    let papyrus = registry.plants().find(|m| m.name == "Papyrus");
    assert!(papyrus.is_some(), "papyrus 찾기");
    assert!(
        papyrus.unwrap().properties.cultural_value >= 0.8,
        "papyrus cultural_value 높음 (기록 매체)"
    );

    // hemp shear_yield 가장 강함 (밧줄)
    let hemp = registry.plants().find(|m| m.name == "Hemp");
    assert!(hemp.is_some(), "hemp 찾기");
    assert!(
        hemp.unwrap().properties.shear_yield >= 7000.0,
        "hemp 밧줄 강도"
    );

    // resin 가장 가연 (송진)
    let resin = registry.plants().find(|m| m.name == "Resin");
    assert!(resin.is_some(), "resin 찾기");
    assert!(
        resin.unwrap().properties.flammability >= 0.9,
        "resin 매우 가연 (송진)"
    );

    // berries preservation 가장 낮음 (부패)
    let berries = registry.plants().find(|m| m.name == "Berries");
    assert!(berries.is_some(), "berries 찾기");
    assert!(
        berries.unwrap().properties.preservation <= 0.3,
        "berries preservation 낮음 (부패)"
    );
}
