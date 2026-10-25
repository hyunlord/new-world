//! Local-catalog knowledge resolution, sparse masks and bootstrap isolation regressions.
use super::*;
use crate::colony::content::{Catalog, RawSources};

const BUNDLED: &str = include_str!("../../../data/colony/knowledge.ron");

fn extension(count: usize) -> String {
    let nodes = (0..count)
        .map(|i| {
            format!(
                "(id: \"extension_{i}\", lane: \"craft\", era: \"neolithic\", threshold: 500, \
         prereqs: [\"flint_knapping\"], difficulty: 1, discover: [Stockpile(\"flint\", 1)], \
         gates: [Recipe(\"knap_blade\")], marker_weight: 0, seed: None, name_key: \"EXT_{i}\")"
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("(schema_version: 1, absent_markers: [], knowledge: [{nodes}])")
}

#[test]
fn appended_nodes_keep_base_indices_and_use_the_last_bit() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
    let base = parse_with_catalog(BUNDLED, &catalog).expect("base");
    let merged = merge_sources(BUNDLED, &extension(8)).expect("merge");
    let defs = parse_with_catalog(&merged, &catalog).expect("32 nodes");
    assert_eq!(&defs.nodes[..24], base.nodes.as_slice());
    assert_eq!(defs.nodes.len(), MAX_NODES);
    assert_eq!(defs.id_of("extension_7"), Some(31));
    let masks = gates_with_catalog(&defs, &catalog).expect("masks");
    let recipe = catalog.recipe_by_name("knap_blade").expect("recipe");
    assert_eq!(masks.recipe(recipe), bit(3) | 0xff00_0000);
    assert!(merge_sources(BUNDLED, &extension(9)).is_err());
}

#[test]
fn unknown_references_duplicate_nodes_and_cycles_reject() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
    assert!(parse_with_catalog(
        "(schema_version: 1, absent_markers: [], knowledge: [])",
        &catalog
    )
    .is_err());
    for malformed in [
        BUNDLED.replace("Recipe(\"haft_axe\")", "Recipe(\"missing\")"),
        BUNDLED.replace("Stockpile(\"raw_pot\", 2)", "Stockpile(\"missing\", 2)"),
        BUNDLED.replace("(id: \"foraging\"", "(id: \"fire_keeping\""),
        BUNDLED.replacen("prereqs: []", "prereqs: [\"fire_keeping\"]", 1),
        BUNDLED.replacen("threshold: 300", "threshold: 1001", 1),
        BUNDLED.replacen("schema_version: 1", "schema_version: 1, typo: 1", 1),
    ] {
        assert_ne!(
            malformed, BUNDLED,
            "negative control actually changed source"
        );
        assert!(parse_with_catalog(&malformed, &catalog).is_err());
    }
    let duplicate = merge_sources(BUNDLED, BUNDLED);
    assert!(duplicate.is_err(), "48 nodes cannot alias the 32-bit map");
}

#[test]
fn gate_builder_rejects_shifted_node_identity() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
    let mut defs = parse_with_catalog(BUNDLED, &catalog).expect("defs");
    defs.nodes[0].id = 31;
    assert!(gates_with_catalog(&defs, &catalog).is_err());
}

#[test]
fn sparse_full_width_recipe_codes_do_not_index_gate_vectors() {
    let mut sources = RawSources::bundled();
    let source = sources
        .sources
        .iter_mut()
        .find(|s| s.name == "recipes.ron")
        .expect("recipes");
    let text = String::from_utf8(source.bytes.clone()).expect("utf8");
    let sparse = text.replacen("code: 0,", "code: 4294967295,", 1);
    assert_ne!(sparse, text, "the fixture moved a real recipe identity");
    source.bytes = sparse.into_bytes();
    let catalog = Catalog::from_sources(sources, None).expect("sparse catalog");
    let recipe = catalog.recipe_by_name("knap_blade").expect("recipe");
    assert_eq!(recipe.code(), u32::MAX);
    assert_eq!(catalog.knowledge_gates().recipe(recipe), bit(3));
}

#[test]
fn local_catalog_and_knowledge_decode_are_bootstrap_isolated() {
    let status = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args([
            "--exact",
            "colony::knowledge::registry_tests::uninstalled_child",
            "--nocapture",
        ])
        .env("WORLDSIM_KNOWLEDGE_TEST_CHILD", "1")
        .status()
        .expect("child");
    assert!(status.success());
}

#[test]
fn uninstalled_child() {
    if std::env::var("WORLDSIM_KNOWLEDGE_TEST_CHILD").as_deref() != Ok("1") {
        return;
    }
    assert!(crate::colony::content::installed().is_err());
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("pure catalog");
    assert_eq!(catalog.knowledge_defs().nodes.len(), 24);
    assert!(crate::colony::content::installed().is_err());
    assert!(ron::from_str::<Knowledge>("(entries: [], practice: 0)").is_err());
    assert!(
        crate::colony::content::installed().is_err(),
        "knowledge serde never installs"
    );
}
