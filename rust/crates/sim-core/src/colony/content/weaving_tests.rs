//! Authored textile capability and content-family regression tests.
use super::*;

#[test]
fn bundled_weaving_chain_has_physical_tools_and_passive_inputs() {
    // Given the bundled starting values, not an outcome-tuned pack.
    let sources = RawSources::bundled();
    // When the catalogue resolves the physical chain.
    let catalog = Catalog::from_sources(sources, None).expect("weaving catalog");
    // Then the entire authored chain exists and the spindle is not a knife.
    for name in [
        "ret_flax",
        "spin_thread",
        "weave_cloth",
        "make_linen_tunic",
        "make_linen_wrap",
        "make_hand_spindle",
    ] {
        assert!(catalog.recipe_by_name(name).is_some(), "missing {name}");
    }
    let spindle = catalog
        .item(catalog.item_by_name("hand_spindle").expect("spindle id"))
        .expect("spindle");
    assert!(spindle.cutter.is_none());
    assert!(spindle
        .tool_capabilities
        .iter()
        .any(|capability| capability == "spindle"));
    let ret = catalog
        .recipe(catalog.recipe_by_name("ret_flax").expect("ret id"))
        .expect("ret");
    assert_eq!(
        ret.process
            .as_ref()
            .map(|p| (p.duration_days, p.capacity, p.requires_river)),
        Some((2, 1, true))
    );
    let tunic = catalog
        .item(catalog.item_by_name("linen_tunic").expect("tunic id"))
        .expect("tunic");
    assert_eq!(
        tunic.apparel.as_ref().and_then(|apparel| apparel.wear_days),
        Some(48)
    );
}

fn extension(items: &str, recipes: &str) -> RawSources {
    RawSources {
        sources: vec![
            NamedSource {
                name: "items.ron".into(),
                bytes: items.as_bytes().to_vec(),
            },
            NamedSource {
                name: "recipes.ron".into(),
                bytes: recipes.as_bytes().to_vec(),
            },
        ],
    }
}

const ITEMS: &str = r#"(schema_version: 1, roles: {}, items: [
    (identity: (code: 1000, id: "reed_comb", name_key: "ITEM_REED_COMB", serde_name: "ReedComb", serde_aliases: [], resolver_aliases: []),
     category: "tool", material_format_key: "", tags: [], stack_max: 2, carry: 1,
     stack_rule: None, carry_rule: None, value: 1, made_by: None, clue_nodes: [],
     chop: None, weapon: None, cutter: None, tool_capabilities: ["combing"],
     food: None, fuel_units: None, apparel: None),
    (identity: (code: 1001, id: "reed_top", name_key: "ITEM_REED_TOP", serde_name: "ReedTop", serde_aliases: [], resolver_aliases: []),
     category: "apparel", material_format_key: "", tags: [], stack_max: 1, carry: 1,
     stack_rule: None, carry_rule: None, value: 1, made_by: None, clue_nodes: [],
     chop: None, weapon: None, cutter: None, food: None, fuel_units: None,
     apparel: Some((legacy_top_code: 5, warmth_rank: 1, cold_permille: 800,
                   cold_snap_work_immune: false, wear_days: Some(12), comfort_permille: 7))),
])"#;

const RECIPES: &str = r#"(schema_version: 1, roles: {}, recipes: [
    (identity: (code: 1000, id: "comb_reed", name_key: "RECIPE_COMB_REED", serde_name: "CombReed", serde_aliases: [], resolver_aliases: []),
     workshop: "retting_pit", workshop_tags: ["retting"],
     inputs: [("flax_stalks", 1)], output: ("reed_top", 1),
     work_ticks: 17, skill: "woodwork", work: "craft", quality_add: 0, material_term: None,
     tool: Some((capability: "combing", preferred_item: "reed_comb", count: 1, reusable: true)),
     process: Some((duration_days: 3, capacity: 2, requires_river: false)), auto_bill_role: None),
])"#;

#[test]
fn alternate_tool_apparel_and_process_require_only_authored_data() {
    // Given unrelated names and different physical parameters in an extension.
    let mut sources = extension(ITEMS, RECIPES);
    sources.sources.push(NamedSource {
        name: "knowledge.ron".into(),
        bytes: br#"(schema_version: 1, absent_markers: [], knowledge: [
            (id: "reed_combing", lane: "craft", era: "neolithic", threshold: 300,
             prereqs: ["spinning"], difficulty: 1, discover: [Stockpile("flax_stalks", 1)],
             gates: [Recipe("comb_reed")], marker_weight: 0, seed: None, name_key: "KNOWLEDGE_REED_COMBING")
        ])"#.to_vec(),
    });
    // When the normal catalogue boundary resolves them.
    let catalog =
        Catalog::from_sources(RawSources::bundled(), Some(sources)).expect("data-only extension");
    // Then both identity and mechanics come from the selected data.
    let recipe = catalog.recipe_by_code(1000).expect("alternate recipe");
    assert_eq!(catalog.knowledge_defs().id_of("reed_combing"), Some(31));
    assert_eq!(catalog.knowledge_defs().nodes.len(), 32);
    assert_eq!(
        recipe.tool.as_ref().map(|tool| tool.capability.as_str()),
        Some("combing")
    );
    assert_eq!(
        recipe.process.as_ref().map(|process| (
            process.duration_days,
            process.capacity,
            process.requires_river
        )),
        Some((3, 2, false))
    );
    let apparel = catalog
        .item(catalog.item_by_name("reed_top").expect("alternate top id"))
        .expect("alternate top");
    assert_eq!(
        apparel.apparel.as_ref().map(|top| (
            top.legacy_top_code,
            top.wear_days,
            top.comfort_permille
        )),
        Some((5, Some(12), 7))
    );
}

#[test]
fn ambiguous_or_unsupported_physical_metadata_is_rejected() {
    // Given valid extension data with one deliberately malformed field per case.
    let mutations = [
        (
            ITEMS.replace("legacy_top_code: 5", "legacy_top_code: 1"),
            RECIPES.into(),
        ),
        (ITEMS.replace("Some(12)", "Some(0)"), RECIPES.into()),
        (
            ITEMS.replace("comfort_permille: 7", "comfort_permille: 1001"),
            RECIPES.into(),
        ),
        (
            ITEMS.replace("[\"combing\"]", "[\"combing\", \"combing\"]"),
            RECIPES.into(),
        ),
        (
            ITEMS.replace("[\"combing\"]", "[\"cutter\"]"),
            RECIPES.into(),
        ),
        (ITEMS.into(), RECIPES.replace("capacity: 2", "capacity: 0")),
        (
            ITEMS.into(),
            RECIPES.replace("duration_days: 3", "duration_days: 0"),
        ),
        (
            ITEMS.into(),
            RECIPES.replace("capability: \"combing\"", "capability: \"missing\""),
        ),
        (
            ITEMS.into(),
            RECIPES.replace("reusable: true", "reusable: false"),
        ),
        (
            ITEMS.into(),
            RECIPES.replace("(\"reed_top\", 1)", "(\"meat\", 1)"),
        ),
        (
            ITEMS.into(),
            RECIPES.replace("(\"flax_stalks\", 1)", "(\"meat\", 1)"),
        ),
    ];
    for (items, recipes) in mutations {
        // When the catalogue parses the malformed physical ownership contract.
        let result =
            Catalog::from_sources(RawSources::bundled(), Some(extension(&items, &recipes)));
        // Then it rejects the pack before simulation begins.
        assert!(result.is_err(), "unexpectedly accepted {items} {recipes}");
    }
}

#[test]
fn legacy_metadata_stays_inert_and_era_denominator_is_preserved() {
    // Given the bundled catalogue with unchanged historical definitions.
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
    // When historical garments and recipes are read.
    let old = ["hide_wrap", "fur_cloak"].map(|name| {
        catalog
            .item(catalog.item_by_name(name).expect("legacy id"))
            .expect("legacy item")
    });
    // Then new wear, comfort and passive processing are absent for old content.
    for item in old {
        let apparel = item.apparel.as_ref().expect("legacy apparel");
        assert_eq!((apparel.wear_days, apparel.comfort_permille), (None, 0));
        assert!(item.tool_capabilities.is_empty());
    }
    assert!(catalog
        .recipes()
        .filter(|recipe| recipe.identity.code < 20)
        .all(|recipe| recipe.process.is_none()));
    let knowledge = catalog.knowledge_defs();
    assert_eq!(knowledge.nodes.len(), 31);
    assert_eq!(knowledge.id_of("cultivation"), Some(27));
    assert_eq!(knowledge.id_of("spinning"), Some(28));
    assert_eq!(knowledge.id_of("weaving"), Some(29));
    assert_eq!(knowledge.id_of("tailoring"), Some(30));
    let spinning = knowledge.get(28).expect("spinning");
    assert_eq!(spinning.prereqs, vec![27]);
    assert_eq!(
        spinning.discover,
        vec![crate::colony::knowledge::Cond::Stockpile(
            catalog.item_by_name("flax_stalks").expect("raw flax"),
            2,
        )]
    );
    let weaving = knowledge
        .nodes
        .iter()
        .find(|node| node.name == "weaving")
        .expect("real weaving node");
    assert_eq!(weaving.marker_weight, 2);
    assert_eq!(
        knowledge
            .nodes
            .iter()
            .map(|node| u32::from(node.marker_weight))
            .sum::<u32>(),
        14
    );
}
