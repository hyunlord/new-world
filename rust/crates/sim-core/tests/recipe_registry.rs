//! Recipe registry regressions: explicit IDs, ordered requirements and compatibility.
use sim_core::colony::content::{Catalog, MaterialTerm, NamedSource, RawSources};

fn sources_with_recipe_change(from: &str, to: &str) -> RawSources {
    let mut sources = RawSources::bundled();
    let source = sources
        .sources
        .iter_mut()
        .find(|s| s.name == "recipes.ron")
        .expect("recipes");
    let text = String::from_utf8(source.bytes.clone()).expect("utf8");
    assert!(
        text.contains(from),
        "mutation must reach its intended field"
    );
    source.bytes = text.replacen(from, to, 1).into_bytes();
    sources
}

#[test]
fn recipe_requirements_preserve_order_material_distinction_and_reusable_count() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("bundled");
    let axe = catalog
        .recipe_by_name("haft_axe")
        .and_then(|id| catalog.recipe(id))
        .expect("axe");
    assert_eq!(
        axe.inputs
            .iter()
            .map(|(id, n)| (id.code(), *n))
            .collect::<Vec<_>>(),
        vec![(4, 1), (0, 1)]
    );
    let blade = catalog.recipe_by_code(0).expect("blade");
    let fine = catalog.recipe_by_code(3).expect("fine");
    assert_eq!(
        blade.material_term,
        MaterialTerm::Hardness {
            offset: 50,
            divisor: 5
        }
    );
    assert_eq!(fine.material_term, MaterialTerm::None);
    assert_eq!(fine.quality_add, 15);
    let scrape = catalog.recipe_by_code(8).expect("scrape");
    assert_eq!(
        scrape
            .needs()
            .iter()
            .map(|(id, n)| (id.code(), *n))
            .collect::<Vec<_>>(),
        vec![(6, 1), (4, 1)]
    );
    let mut already_required = scrape.clone();
    let cutter = scrape.tool.as_ref().expect("cutter").preferred_item;
    already_required.inputs.push((cutter, 2));
    assert_eq!(
        already_required.needs(),
        already_required.inputs,
        "append only if absent"
    );
}

#[test]
fn malformed_recipe_metadata_is_rejected_before_world_creation() {
    for (from, to) in [
        ("schema_version: 1", "schema_version: 2"),
        ("code: 1, id: \"haft_axe\"", "code: 0, id: \"haft_axe\""),
        ("id: \"haft_axe\"", "id: \"knap_blade\""),
        (
            "inputs: [(\"flint\", 1)]",
            "inputs: [(\"undefined_input\", 1)]",
        ),
        ("divisor: 5", "divisor: 0"),
        ("count: 1, reusable: true", "count: 2, reusable: true"),
        ("count: 1, reusable: true", "count: 1, reusable: false"),
        ("preferred_item: \"flint_blade\"", "preferred_item: \"log\""),
        (
            "workshop_tags: [\"knapping\"]",
            "workshop_tags: [\"undefined_capability\"]",
        ),
    ] {
        assert!(
            Catalog::from_sources(sources_with_recipe_change(from, to), None).is_err(),
            "must reject {to}"
        );
    }
}

#[test]
fn full_width_recipe_is_ordered_and_uses_declared_compatibility() {
    let extension = RawSources {
        sources: vec![NamedSource {
            name: "recipes.ron".into(),
            bytes: br#"
        (schema_version: 1, roles: {}, recipes: [(
            identity: (code: 4294967295, id: "fixture_recipe", name_key: "FIXTURE_RECIPE",
                serde_name: "FixtureRecipe", serde_aliases: [], resolver_aliases: []),
            workshop: "wood_bench", workshop_tags: ["woodworking"],
            inputs: [("log", 2), ("flint", 1)], output: ("firewood", 2),
            work_ticks: 7, skill: "woodwork", work: "craft", quality_add: 0,
            material_term: None, tool: None, auto_bill_role: None,
        )])
    "#
            .to_vec(),
        }],
    };
    let catalog = Catalog::from_sources(RawSources::bundled(), Some(extension)).expect("extension");
    let recipe = catalog.recipe_by_code(u32::MAX).expect("full width recipe");
    assert_eq!(
        catalog.recipes().last().map(|r| r.id.code()),
        Some(u32::MAX)
    );
    assert_eq!(
        recipe
            .inputs
            .iter()
            .map(|(id, n)| (id.code(), *n))
            .collect::<Vec<_>>(),
        vec![(0, 2), (3, 1)]
    );
    assert!(recipe.supports_workshop(catalog.blueprint_by_name("wood_bench").expect("wood bench")));
    assert!(!recipe.supports_workshop(
        catalog
            .blueprint_by_name("knapping_spot")
            .expect("knapping spot")
    ));
}

#[test]
fn second_workshop_with_the_capability_supports_the_same_recipe() {
    let mut sources = RawSources::bundled();
    let source = sources
        .sources
        .iter_mut()
        .find(|s| s.name == "blueprints.ron")
        .expect("blueprints");
    let text = String::from_utf8(source.bytes.clone()).expect("utf8");
    assert!(text.contains("workshop_tags: [\"grinding\"]"));
    source.bytes = text
        .replacen(
            "workshop_tags: [\"grinding\"]",
            "workshop_tags: [\"grinding\", \"woodworking\"]",
            1,
        )
        .into_bytes();
    let catalog = Catalog::from_sources(sources, None).expect("compatible workstation");
    let recipe = catalog.recipe_by_code(1).expect("haft axe");
    let preferred = catalog.blueprint_by_name("wood_bench").expect("preferred");
    let additional = catalog
        .blueprint_by_name("grinding_stone")
        .expect("additional");
    assert_eq!(
        recipe.workshop, preferred,
        "compatibility must not change construction preference"
    );
    assert!(recipe.supports_workshop(preferred));
    assert!(recipe.supports_workshop(additional));
    assert!(recipe
        .compatible_workshops
        .windows(2)
        .all(|pair| pair[0].code() < pair[1].code()));
}

#[test]
fn steward_rule_parsing_requires_boot_without_poisoning_later_rules() {
    let status = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", "steward_rule_boot_child", "--nocapture"])
        .env("WORLDSIM_STEWARD_BOOT_CHILD", "1")
        .status()
        .expect("child");
    assert!(status.success());
}

#[test]
fn steward_rule_boot_child() {
    if std::env::var_os("WORLDSIM_STEWARD_BOOT_CHILD").is_none() {
        return;
    }
    use sim_core::colony::{content, steward};
    assert!(content::installed().is_err());
    assert!(steward::rules().is_err());
    let text = include_str!("../data/colony/steward.ron");
    assert!(steward::parse_rules(text).is_err());
    assert!(
        content::installed().is_err(),
        "typed rule parsing must not initialize the catalog"
    );
    content::bootstrap(content::BootSelection::Bundled).expect("explicit boot");
    assert_eq!(steward::rules().expect("rules after boot").rules.len(), 7);
    assert_eq!(
        steward::parse_rules(text).expect("typed rules").rules.len(),
        7
    );
}
