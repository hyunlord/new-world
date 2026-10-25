//! Pure flora admission, strict merge, physical capability and draw-boundary tests.
use super::*;
use crate::material::{MaterialId, TerrainType};

const BASE: &str = r#"(schema_version:1,default_material:"oak",
 species:[(material:"oak"),(material:"birch")],
 distributions:[(terrain:forest,elevation:(0,255),
 weights:[(material:"oak",weight:3),(material:"birch",weight:1)])])"#;
const EXTRA: &str = r#"(schema_version:1,species:[(material:"pine")],
 replace_distributions:Some([(terrain:forest,elevation:(0,255),
 weights:[(material:"pine",weight:1)])]))"#;
fn base(text: &str) -> RawSources {
    let mut sources = RawSources::bundled();
    sources
        .sources
        .iter_mut()
        .find(|s| s.name == "flora.ron")
        .expect("flora")
        .bytes = text.as_bytes().to_vec();
    sources
}
fn extra(text: &str) -> RawSources {
    RawSources {
        sources: vec![NamedSource {
            name: "flora".into(),
            bytes: text.as_bytes().to_vec(),
        }],
    }
}
fn rejected(text: &str, extension: Option<&str>, message: &str) {
    let error = Catalog::from_sources(base(text), extension.map(extra)).expect_err("must reject");
    assert!(error.to_string().contains(message), "{error}");
}
#[test]
fn flora_member_append_uses_already_registered_wood_and_explicit_replacement() {
    let original = Catalog::from_sources(base(BASE), None).expect("base");
    let extended = Catalog::from_sources(base(BASE), Some(extra(EXTRA))).expect("extension");
    let pine = MaterialId::from_str_hash("pine");
    assert!(crate::colony::materials::ColonyMaterials::load()
        .stats(pine)
        .is_some());
    assert!(original.flora().species_by_material(pine).is_none());
    assert_eq!(original.flora().species().len(), 2);
    assert_eq!(extended.flora().species().len(), 3);
    assert_eq!(
        extended.flora().default_material(),
        original.flora().default_material()
    );
    assert_ne!(extended.revision(), original.revision());
    for elevation in 0..=255 {
        let band = extended
            .flora()
            .distribution(TerrainType::Forest, elevation)
            .expect("band");
        assert_eq!(band.total_weight(), 1);
        assert_eq!(band.select(0), Some(pine));
        assert_eq!(band.select(1), None);
    }
}
#[test]
fn flora_draw_boundaries_follow_canonical_names_and_inherit_on_none() {
    let inherited = EXTRA.replace("replace_distributions:Some([(terrain:forest,elevation:(0,255),\n weights:[(material:\"pine\",weight:1)])])", "replace_distributions:None");
    assert_ne!(inherited, EXTRA);
    let catalog = Catalog::from_sources(base(BASE), Some(extra(&inherited))).expect("inherit");
    let band = catalog
        .flora()
        .distribution(TerrainType::Forest, 0)
        .expect("band");
    assert_eq!(band.select(0), Some(MaterialId::from_str_hash("birch")));
    for draw in 1..4 {
        assert_eq!(band.select(draw), Some(MaterialId::from_str_hash("oak")));
    }
    assert_eq!(band.select(4), None);
    assert_eq!(band.select(u64::MAX), None);
    assert!(catalog
        .flora()
        .distribution(TerrainType::Plain, 0)
        .is_none());
}
#[test]
fn flora_bundled_distribution_endpoints_and_weights_are_exact() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("bundled");
    let flora = catalog.flora();
    assert_eq!(
        flora
            .species()
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
        ["birch", "oak", "pine"]
    );
    for (elevation, expected) in [
        (0, [150, 600, 250]),
        (159, [150, 600, 250]),
        (160, [150, 200, 650]),
        (255, [150, 200, 650]),
    ] {
        let band = flora
            .distribution(TerrainType::Forest, elevation)
            .expect("band");
        assert_eq!(
            band.weights().iter().map(|w| w.weight).collect::<Vec<_>>(),
            expected
        );
        assert_eq!(band.total_weight(), 1000);
    }
}
#[test]
fn flora_extension_cannot_hide_invalid_base_or_readmit_members() {
    rejected(
        &BASE.replace("weight:3", "weight:0"),
        Some(EXTRA),
        "positive",
    );
    rejected(
        BASE,
        Some(&EXTRA.replace("species:[(material:\"pine\")]", "species:[]")),
        "unadmitted",
    );
    rejected(BASE, Some(&EXTRA.replace("pine", "oak")), "duplicate");
    rejected(
        BASE,
        Some(&EXTRA.replace(
            "schema_version:1",
            "schema_version:1,default_material:\"pine\"",
        )),
        "default_material",
    );
    rejected(
        BASE,
        Some("(schema_version:1,species:[],replace_distributions:Some([]))"),
        "coverage",
    );
}
#[test]
fn flora_rejects_invalid_schema_members_defaults_and_bands() {
    for (text, message) in [
        (
            BASE.replace("schema_version:1", "schema_version:2"),
            "schema",
        ),
        (
            BASE.replace("default_material:\"oak\"", "default_material:\"pine\""),
            "default",
        ),
        (BASE.replace("birch", "granite"), "Wood"),
        (BASE.replace("birch", "unregistered"), "material"),
        (BASE.replace("birch", "clay"), "physical"),
        (BASE.replace("birch", "oak"), "duplicate"),
        (BASE.replace("(0,255)", "(1,255)"), "coverage"),
        (BASE.replace("(0,255)", "(255,0)"), "interval"),
        (BASE.replace("terrain:forest", "terrain:plain"), "Forest"),
        (BASE.replace("weight:3", "weight:0"), "positive"),
        (BASE.replace("weight:3", "weight:4294967296"), "flora"),
        (BASE.replace("weight:3", "weight:3,unknown:1"), "unknown"),
    ] {
        rejected(&text, None, message);
    }
    rejected(
        BASE,
        Some("(schema_version:1,species:[])"),
        "replace_distributions",
    );
}
#[test]
fn flora_rejects_overlapping_empty_or_duplicate_distribution_rows() {
    let row = r#"(terrain:forest,elevation:(0,255),weights:[(material:"oak",weight:1)])"#;
    let prefix =
        r#"(schema_version:1,default_material:"oak",species:[(material:"oak")],distributions:"#;
    rejected(&format!("{prefix}[{row},{row}])"), None, "overlapping");
    rejected(
        &format!("{prefix}[(terrain:forest,elevation:(0,255),weights:[])])"),
        None,
        "nonempty",
    );
    rejected(&format!("{prefix}[(terrain:forest,elevation:(0,255),weights:[(material:\"oak\",weight:1),(material:\"oak\",weight:2)])])"), None, "duplicate");
}
#[test]
fn flora_requires_source_and_declared_physical_material_and_exact_names() {
    let mut missing = RawSources::bundled();
    missing.sources.retain(|source| source.name != "flora.ron");
    assert!(Catalog::from_sources(missing, None)
        .expect_err("required")
        .to_string()
        .contains("missing required"));
    let mut undeclared = base(BASE);
    let blueprints = undeclared
        .sources
        .iter_mut()
        .find(|source| source.name == "blueprints.ron")
        .expect("blueprints");
    let text = String::from_utf8(blueprints.bytes.clone()).expect("utf8");
    blueprints.bytes = text.replacen("\"birch\", ", "", 1).into_bytes();
    assert!(Catalog::from_sources(undeclared, None)
        .expect_err("declaration")
        .to_string()
        .contains("undeclared material birch"));
    rejected(BASE, Some(&EXTRA.replace("pine", "Pine")), "undeclared");
}
#[test]
fn flora_u32_weights_accumulate_in_u64_without_clipping() {
    let catalog = Catalog::from_sources(
        base(
            &BASE
                .replace("weight:3", "weight:4294967295")
                .replace("weight:1", "weight:4294967295"),
        ),
        None,
    )
    .expect("u64 total");
    let band = catalog
        .flora()
        .distribution(TerrainType::Forest, 0)
        .expect("band");
    assert_eq!(band.total_weight(), 2 * u64::from(u32::MAX));
    assert_eq!(
        band.select(u64::from(u32::MAX) - 1),
        Some(MaterialId::from_str_hash("birch"))
    );
    assert_eq!(
        band.select(u64::from(u32::MAX)),
        Some(MaterialId::from_str_hash("oak"))
    );
    assert_eq!(
        band.select(band.total_weight() - 1),
        Some(MaterialId::from_str_hash("oak"))
    );
    assert_eq!(band.select(band.total_weight()), None);
}
#[test]
fn fixed_physical_definitions_supply_identical_runtime_stats() {
    let physical = crate::colony::materials::embedded_definitions().expect("physical definitions");
    let runtime = crate::colony::materials::ColonyMaterials::load();
    assert_eq!(physical.len(), 7);
    for definition in physical {
        assert_eq!(runtime.name(definition.id), definition.name.to_lowercase());
        assert_eq!(
            runtime.stats(definition.id),
            Some(crate::colony::materials::MaterialStats::from_def(
                &definition
            ))
        );
    }
}
#[test]
fn flora_extension_material_declaration_cannot_repair_invalid_base() {
    let mut invalid = base(BASE);
    let blueprints = invalid
        .sources
        .iter_mut()
        .find(|source| source.name == "blueprints.ron")
        .expect("blueprints");
    let text = String::from_utf8(blueprints.bytes.clone()).expect("utf8");
    blueprints.bytes = text.replacen("\"birch\", ", "", 1).into_bytes();
    let extension = RawSources {
        sources: vec![NamedSource {
            name: "blueprints.ron".into(),
            bytes: br#"(schema_version:1,materials:["birch"],blueprints:[])"#.to_vec(),
        }],
    };
    assert!(Catalog::from_sources(invalid, Some(extension))
        .expect_err("invalid base stays invalid")
        .to_string()
        .contains("undeclared material birch"));
}
