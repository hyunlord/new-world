//! Catalogue validation and process-isolated bootstrap regression tests.
use super::*;

#[test]
fn bundled_catalogue_is_local_and_code_ordered() {
    let catalog =
        Catalog::from_sources(RawSources::bundled(), None).expect("valid bundled catalog");
    assert_eq!(
        catalog.items().map(|d| d.identity.code).collect::<Vec<_>>(),
        (0..101).collect::<Vec<_>>()
    );
    assert_eq!(
        catalog.item_by_name("flax_seed").map(|kind| kind.code()),
        Some(30)
    );
    assert_eq!(
        catalog
            .recipe_by_name("extract_flax_seed")
            .map(|kind| kind.code()),
        Some(18)
    );
    assert_eq!(catalog.blueprints().count(), 25);
    assert_eq!(catalog.recipes().count(), 70);
    assert_eq!(
        catalog
            .items()
            .skip(43)
            .map(|item| item.identity.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "flour",
            "bread",
            "beer",
            "milk",
            "cheese",
            "smoked_meat",
            "hay",
            "hoe",
            "sickle",
            "adze",
            "bow",
            "arrow",
            "dyed_tunic",
            "leather_armor",
            "herbs",
            "poultice",
            "barley",
            "barley_seed",
            "rye",
            "rye_seed",
            "lentil",
            "lentil_seed",
            "pea",
            "pea_seed",
            "millet",
            "millet_seed",
            "apple",
            "goat_hide",
            "wool",
            "charcoal",
            "copper_ore",
            "tin_ore",
            "iron_ore",
            "copper_ingot",
            "tin_ingot",
            "bronze_ingot",
            "clay_mould",
            "copper_awl",
            "copper_needle",
            "copper_axe",
            "bronze_axe",
            "bronze_sickle",
            "bronze_hoe",
            "bronze_spear",
            "bronze_dagger",
            "honey",
            "beeswax",
            "wool_yarn",
            "wool_cloth",
            "wool_tunic",
            "wheel",
            "mud_brick",
            "lime",
            "seal",
            "tablet",
            "glass_bead",
            "composite_bow",
            "limestone",
        ]
    );
    assert_eq!(catalog.item_by_name("firewood").map(|k| k.code()), Some(1));
    assert_eq!(
        catalog.item_by_serde_name("Firewood").map(|k| k.code()),
        Some(1)
    );
    assert!(catalog.item_by_serde_name("firewood").is_none());
}

#[test]
fn malformed_schema_and_duplicate_sources_are_rejected() {
    let mut sources = RawSources::bundled();
    let source = sources
        .sources
        .iter_mut()
        .find(|s| s.name == "items.ron")
        .expect("item source");
    source.bytes = String::from_utf8(source.bytes.clone())
        .expect("utf8")
        .replacen("schema_version: 1", "schema_version: 99", 1)
        .into_bytes();
    assert!(Catalog::from_sources(sources, None).is_err());
    let mut duplicate = RawSources::bundled();
    duplicate.sources.push(duplicate.sources[0].clone());
    assert!(Catalog::from_sources(duplicate, None).is_err());
}

#[test]
fn default_projection_and_recipe_material_term_are_explicit() {
    let c = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
    assert!(c.historical_stone_wall_material(crate::colony::materials::GRANITE));
    assert!(!c.historical_stone_wall_material(crate::colony::materials::FLINT));
    assert_eq!(
        c.recipe_by_code(0).expect("blade").material_term,
        MaterialTerm::Hardness {
            offset: 50,
            divisor: 5
        }
    );
    assert_eq!(
        c.recipe_by_code(3).expect("fine blade").material_term,
        MaterialTerm::None
    );
}

#[test]
fn bootstrap_negative_cases_run_in_fresh_processes() {
    for case in ["uninstalled", "failed", "bundled", "extension"] {
        let status = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                "--exact",
                "colony::content::tests::bootstrap_child",
                "--nocapture",
            ])
            .env("WORLDSIM_CONTENT_TEST_CASE", case)
            .status()
            .expect("child process");
        assert!(status.success(), "bootstrap case {case}");
    }
}

#[test]
fn bootstrap_child() {
    let Ok(case) = std::env::var("WORLDSIM_CONTENT_TEST_CASE") else {
        return;
    };
    match case.as_str() {
        "uninstalled" => {
            assert!(installed().is_err());
            assert!(ron::from_str::<ItemKind>("Firewood").is_err());
            assert!(installed().is_err(), "serde must not bootstrap");
        }
        "failed" => {
            let bad = BootSelection::Explicit {
                request_id: "bad".into(),
                sources: RawSources::default(),
            };
            assert!(bootstrap(bad).is_err());
            assert!(bootstrap(BootSelection::Bundled).is_err());
            assert!(installed().is_err());
        }
        "bundled" => {
            bootstrap(BootSelection::Bundled).expect("boot");
            let id: ItemKind = ron::from_str("Firewood").expect("legacy unit name");
            assert_eq!(id.code(), 1);
            assert_eq!(format!("{id:?}"), "Firewood");
            assert_eq!(serde_json::to_string(&id).expect("json"), "\"Firewood\"");
            assert!(ItemKind::checked(u32::MAX).is_err());
            assert!(bootstrap(BootSelection::Bundled).is_ok());
        }
        "extension" => {
            let sources = RawSources {
                sources: vec![NamedSource {
                    name: "items.ron".into(),
                    bytes: EXT_ITEM.as_bytes().to_vec(),
                }],
            };
            bootstrap(BootSelection::Explicit {
                request_id: "test:1".into(),
                sources,
            })
            .expect("extension boot");
            let item = ItemKind::checked(u32::MAX).expect("registered high code");
            assert_eq!(ron::to_string(&item).expect("ron"), "FixtureCopper");
            assert_eq!(
                ron::from_str::<ItemKind>("FixtureCopper").expect("roundtrip"),
                item
            );
            assert!(serde_json::from_str::<ItemKind>("4294967295").is_err());
            let map = std::collections::BTreeMap::from([(item, 3u16)]);
            let text = serde_json::to_string(&map).expect("unit-name map key");
            assert_eq!(
                serde_json::from_str::<std::collections::BTreeMap<ItemKind, u16>>(&text)
                    .expect("map"),
                map
            );
            assert!(
                bootstrap(BootSelection::Bundled).is_err(),
                "selected pack cannot be replaced"
            );
        }
        _ => panic!("unknown child case"),
    }
}

const EXT_ITEM: &str = r#"(schema_version: 1, roles: {}, items: [(
    identity: (code: 4294967295, id: "fixture:copper", name_key: "FIXTURE_COPPER", serde_name: "FixtureCopper", serde_aliases: [], resolver_aliases: []),
    category: "material", material_format_key: "", tags: [], stack_max: 20, carry: 5,
    stack_rule: None, carry_rule: None, value: 3, made_by: None, clue_nodes: [],
    chop: None, weapon: None, cutter: None, food: None, fuel_units: None, apparel: None,
)])"#;

#[test]
fn high_sparse_codes_and_revision_inputs_are_deterministic() {
    let base = RawSources::bundled();
    let mut reversed = base.clone();
    reversed.sources.reverse();
    let first = Catalog::from_sources(base.clone(), None).expect("base");
    let reordered = Catalog::from_sources(reversed, None).expect("reordered sources");
    assert_eq!(first.revision(), reordered.revision());
    let extension = RawSources {
        sources: vec![NamedSource {
            name: "items".into(),
            bytes: EXT_ITEM.as_bytes().to_vec(),
        }],
    };
    let extended = Catalog::from_sources(base, Some(extension)).expect("sparse extension");
    assert_eq!(extended.items().count(), 102);
    assert_eq!(
        extended.items().take(101).collect::<Vec<_>>(),
        first.items().collect::<Vec<_>>()
    );
    assert_eq!(
        extended.item_by_name("fixture:copper").map(|id| id.code()),
        Some(u32::MAX)
    );
    assert_ne!(first.revision(), extended.revision());
}

#[test]
fn invalid_extension_cannot_hide_behind_category_fallback() {
    for (text, expected) in [
        (
            EXT_ITEM.replace("code: 4294967295", "code: 0"),
            "duplicate code",
        ),
        (
            EXT_ITEM.replace("value: 3", "value: 3, unknown: 4"),
            "Unexpected field named `unknown`",
        ),
        (
            EXT_ITEM.replace("fuel_units: None", "fuel_units: Some(2)"),
            "fuel conversion",
        ),
    ] {
        let extension = RawSources {
            sources: vec![NamedSource {
                name: "items.ron".into(),
                bytes: text.into_bytes(),
            }],
        };
        let error = Catalog::from_sources(RawSources::bundled(), Some(extension))
            .expect_err("invalid extension");
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn spine_items_keep_unwired_capabilities_inert_and_art_explicit() {
    // Given the bundled data, the five M-SPINE identities retain their stable codes.
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalogue");
    // When the new item family is resolved, metadata cannot install new mechanics.
    for (code, name, shape, connected) in [
        (38, "cord", ItemShape::Rod, true),
        (39, "basket", ItemShape::Vessel, true),
        (40, "sling", ItemShape::Cloth, true),
        (41, "bone_needle", ItemShape::Rod, false),
        (42, "fish", ItemShape::Lump, false),
    ] {
        let item = catalog.item_by_code(code).expect("authored item");
        // Then C1 connects crafting while preserving explicit debt for missing effects.
        assert_eq!(item.identity.id, name);
        assert_eq!(item.art.shape, shape);
        assert_eq!(
            item.status == crate::colony::knowledge::ContentStatus::Connected,
            connected
        );
        assert_eq!(item.debt.is_none(), name == "cord");
        assert!(item.food.is_none() && item.weapon.is_none() && item.apparel.is_none());
    }
}

#[test]
fn spine_cord_recipe_uses_real_flax_and_workbench_without_new_mechanics() {
    // Given the selected bundled catalogue.
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalogue");
    // When its newly authored cord recipe is resolved.
    let kind = catalog.recipe_by_name("twist_cord").expect("cord recipe");
    let recipe = catalog.recipe(kind).expect("resolved cord recipe");
    // Then two existing flax stalks yield one cord at the existing wood bench.
    assert_eq!(
        recipe.inputs,
        vec![(catalog.item_by_name("flax_stalks").expect("flax"), 2)]
    );
    assert_eq!(
        recipe.output,
        (catalog.item_by_name("cord").expect("cord"), 1)
    );
    assert_eq!(
        recipe.workshop,
        catalog.blueprint_by_name("wood_bench").expect("bench")
    );
    let basket = catalog
        .recipe(
            catalog
                .recipe_by_name("weave_basket")
                .expect("basket recipe"),
        )
        .expect("resolved basket recipe");
    assert_eq!(basket.workshop, recipe.workshop);
    assert_eq!(
        basket.inputs,
        vec![(catalog.item_by_name("cord").expect("cord"), 3)]
    );
    assert_eq!(
        basket.output,
        (catalog.item_by_name("basket").expect("basket"), 1)
    );
    assert_eq!(
        basket.food_requirements.knowledge_any,
        vec![catalog
            .knowledge_defs()
            .id_of("basketry")
            .expect("basketry")]
    );
}
