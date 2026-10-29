//! M6a catalog contracts: stable identities, strict validation and data-only food species.
use sim_core::colony::content::{Catalog, FoodStorageClass, NamedSource, PlantPhase, RawSources};

fn catalog() -> Catalog {
    Catalog::from_sources(RawSources::bundled(), None).expect("bundled catalog")
}
fn replace_source(name: &str, old: &str, new: &str) -> RawSources {
    let mut sources = RawSources::bundled();
    let source = sources
        .sources
        .iter_mut()
        .find(|s| s.name == name)
        .expect("source");
    let text = String::from_utf8(source.bytes.clone()).expect("utf8");
    assert!(text.contains(old), "fixture substitution must match");
    source.bytes = text.replacen(old, new, 1).into_bytes();
    sources
}

#[test]
fn food_storage_and_cooking_are_data_capabilities() {
    let c = catalog();
    let roots = c
        .item(c.item_by_name("roots").expect("roots"))
        .expect("definition");
    let food = roots.food.as_ref().expect("food");
    assert_eq!(food.units, 1);
    assert_eq!(food.groups, ["root"]);
    assert_eq!(
        [
            FoodStorageClass::Ground,
            FoodStorageClass::Stockpile,
            FoodStorageClass::RoofedStore,
            FoodStorageClass::Pot
        ]
        .map(|class| food.lifetime_days(class)),
        [6, 10, 14, 18]
    );
    assert!(
        !c.item(c.item_by_name("wild_grain").expect("grain"))
            .expect("definition")
            .food
            .as_ref()
            .expect("food")
            .raw_edible
    );
    assert_eq!(c.item_by_code(9).expect("historical pot").pot_capacity, 12);
    assert_eq!(
        c.item_by_code(11).expect("historical meat").identity.id,
        "meat"
    );
    assert_eq!(
        c.blueprint_by_code(3)
            .expect("historical drying rack")
            .workshop_tags,
        ["hideworking"]
    );
    let recipe = c
        .recipe(c.recipe_by_name("fruit_porridge").expect("recipe"))
        .expect("definition");
    assert_eq!(recipe.output.1, 4);
    assert_eq!(
        recipe.food_requirements.knowledge_any,
        [c.knowledge_defs()
            .id_of("grain_parching")
            .expect("knowledge")]
    );
    assert!(recipe.food_requirements.pot_alternative);
    let cooked = c
        .item(recipe.output.0)
        .expect("output")
        .food
        .as_ref()
        .expect("food");
    assert!(cooked.cooked);
    assert_eq!(cooked.groups, ["grain", "fruit"]);
}

#[test]
fn seasonal_species_preserve_frozen_supply_and_category_fallback() {
    let c = catalog();
    assert_eq!(c.plants().count(), 5);
    let root = c.plant_by_name("wild_root").expect("root");
    let def = c.plant(root).expect("definition");
    assert_eq!(
        (def.yield_count, def.regrow_days, def.harvest_ticks),
        (5, 2, 60)
    );
    assert_eq!(
        def.phases,
        [
            PlantPhase::Ripe,
            PlantPhase::Ripe,
            PlantPhase::Ripe,
            PlantPhase::Dormant
        ]
    );
    assert_eq!(c.resolve_plant("missing", "root"), Some(root));
    assert_eq!(c.resolve_plant("wild_root", "missing"), Some(root));
    assert_eq!(c.resolve_plant("missing", "missing"), None);
    let grain = c
        .plant(c.plant_by_name("wild_grain").expect("grain"))
        .expect("definition");
    assert_eq!(grain.ripe_start_day, 3);
}

const APPLE_ITEM: &str = r#"(schema_version: 1, items: [(
    identity: (code: 1001, id: "apple", name_key: "ITEM_APPLE", serde_name: "Apple", serde_aliases: [], resolver_aliases: []),
    category: "food", material_format_key: "", tags: ["food"], stack_max: 30, carry: 10,
    stack_rule: None, carry_rule: None, value: 2, made_by: None, clue_nodes: [],
    chop: None, weapon: None, cutter: None, fuel_units: None, apparel: None,
    food: Some((units: 1, units_rule: None, spoil_days: 3, spoil_rule: None,
        groups: ["fruit"], storage_spoil_days: Some((ground: 3, stockpile: 5, roofed_store: 7, pot: 10)))),
)])"#;
const APPLE_PLANT: &str = r#"(schema_version: 1, plants: [(
    identity: (code: 1001, id: "wild_apple", name_key: "PLANT_WILD_APPLE", serde_name: "WildApple", serde_aliases: [], resolver_aliases: []),
    category: "bush", tags: ["bush"],
    placement: (terrains: [plain, forest], elevation: (0, 255), weight: 3, density_permille: 200),
    phases: [Growing, Ripe, Ripe, Dormant], ripe_start_day: 0, ripe_end_day: None,
    yield: (item: "apple", units: 4, regrow_days: 2), harvest_ticks: 60,
    knowledge: Some("foraging"), knowledge_speed_permille: 1500,
)])"#;

#[test]
fn data_only_food_and_species_resolve_without_global_installation() {
    let extension = RawSources {
        sources: vec![
            NamedSource {
                name: "items".into(),
                bytes: APPLE_ITEM.as_bytes().to_vec(),
            },
            NamedSource {
                name: "plants".into(),
                bytes: APPLE_PLANT.as_bytes().to_vec(),
            },
        ],
    };
    let c = Catalog::from_sources(RawSources::bundled(), Some(extension)).expect("data extension");
    let plant = c
        .plant(c.plant_by_name("wild_apple").expect("apple plant"))
        .expect("definition");
    assert_eq!(
        plant.yield_item,
        c.item_by_name("apple").expect("apple item")
    );
    assert!(
        c.item(plant.yield_item)
            .expect("item")
            .food
            .as_ref()
            .expect("food")
            .raw_edible
    );
    assert_eq!(plant.identity.code, 1001);
    assert_eq!(c.plants().count(), 6);
}

#[test]
fn invalid_plant_references_windows_and_density_are_rejected() {
    for (old, new) in [
        ("density_permille: 200", "density_permille: 1001"),
        ("item: \"berries\"", "item: \"missing\""),
        (
            "knowledge: Some(\"foraging\")",
            "knowledge: Some(\"missing\")",
        ),
        ("ripe_end_day: None", "ripe_end_day: Some(0)"),
        ("weight: 3", "weight: 0"),
        ("units: 4", "units: 0"),
    ] {
        assert!(
            Catalog::from_sources(replace_source("plants.ron", old, new), None).is_err(),
            "{new}"
        );
    }
}

#[test]
fn food_validation_rejects_zero_lifetimes_and_duplicate_groups() {
    for (old, new) in [
        ("sick_permille: 100", "sick_permille: 1001"),
        ("groups: [\"meat\"]", "groups: [\"meat\", \"meat\"]"),
        ("ground: 2, stockpile: 3", "ground: 0, stockpile: 3"),
    ] {
        assert!(
            Catalog::from_sources(replace_source("items.ron", old, new), None).is_err(),
            "{new}"
        );
    }
}

#[test]
fn perishable_construction_needs_fail_explicitly_without_blocking_material_extensions() {
    const NEED: &str = "needs: [(item: \"log\", count: 12, count_rule: Some(\"hut_logs\"))]";
    for name in ["meat", "berries", "wild_grain"] {
        let replacement = NEED.replace("\"log\"", &format!("\"{name}\""));
        let sources = replace_source("blueprints.ron", NEED, &replacement);
        let error = Catalog::from_sources(sources, None).expect_err("food construction must fail");
        assert!(
            error.to_string().contains("cannot be a construction need"),
            "{error}"
        );
    }
    let sources = replace_source(
        "blueprints.ron",
        NEED,
        &NEED.replace("\"log\"", "\"apple\""),
    );
    let extension = |item: &str| RawSources {
        sources: vec![NamedSource {
            name: "items".into(),
            bytes: item.as_bytes().to_vec(),
        }],
    };
    let error = Catalog::from_sources(sources.clone(), Some(extension(APPLE_ITEM)))
        .expect_err("novel food capability must also fail");
    assert!(
        error.to_string().contains("cannot be a construction need"),
        "{error}"
    );
    let material = format!(
        "{}food: None,\n)])",
        APPLE_ITEM.split("food: Some").next().expect("fixture")
    );
    let c = Catalog::from_sources(sources, Some(extension(&material))).expect("nonfood extension");
    assert_eq!(c.blueprints().count(), catalog().blueprints().count());
    assert_eq!(
        c.blueprint_by_code(0).expect("hut").needs[0].item,
        c.item_by_name("apple").expect("extension material")
    );
}

#[test]
fn food_cannot_enter_unaged_equipment_or_resource_pipelines() {
    for id in ["stone_axe", "flint_blade", "spear", "hide_wrap"] {
        let mut sources = RawSources::bundled();
        let source = sources
            .sources
            .iter_mut()
            .find(|s| s.name == "items.ron")
            .expect("items");
        let mut text = String::from_utf8(source.bytes.clone()).expect("utf8");
        let start = text.find(&format!("id: \"{id}\"")).expect("item");
        let food = start
            + text[start..]
                .find("food: None")
                .expect("nonfood capability");
        text.replace_range(
            food..food + "food: None".len(),
            "food: Some((units: 1, units_rule: None, spoil_days: 3, spoil_rule: None))",
        );
        source.bytes = text.into_bytes();
        let error = Catalog::from_sources(sources, None).expect_err("food equipment");
        assert!(
            error.to_string().contains("food cannot also provide"),
            "{error}"
        );
    }
    for (old, new) in [
        ("fuel_units: None", "fuel_units: Some(1)"),
        ("apparel: None", "apparel: None, pot_capacity: 12"),
    ] {
        let extension = RawSources {
            sources: vec![NamedSource {
                name: "items".into(),
                bytes: APPLE_ITEM.replace(old, new).into_bytes(),
            }],
        };
        let error = Catalog::from_sources(RawSources::bundled(), Some(extension))
            .expect_err("mixed capability");
        assert!(
            error.to_string().contains("food cannot also provide"),
            "{error}"
        );
    }
    for role in [
        "tree_output",
        "deadwood_output",
        "boulder_output",
        "outcrop_output",
        "clay_output",
        "excavated_stone",
        "legacy_axe_return",
        "default_placeholder",
        "legacy_axe",
        "preferred_hunt_weapon",
        "preferred_cutter",
        "hearth_fuel",
        "grazer_hide",
        "predator_hide",
    ] {
        let mut sources = RawSources::bundled();
        let source = sources
            .sources
            .iter_mut()
            .find(|s| s.name == "items.ron")
            .expect("items");
        let mut text = String::from_utf8(source.bytes.clone()).expect("utf8");
        let prefix = format!("\"{role}\": \"");
        let start = text.find(&prefix).expect("role") + prefix.len();
        let end = start + text[start..].find('"').expect("role end");
        text.replace_range(start..end, "berries");
        source.bytes = text.into_bytes();
        let error = Catalog::from_sources(sources, None).expect_err("nonfood role");
        assert!(error.to_string().contains("food cannot fill"), "{error}");
    }
}

#[test]
fn duplicate_resolved_recipe_inputs_and_nonfood_drying_are_rejected() {
    let sources = replace_source(
        "recipes.ron",
        "inputs: [(\"flint\", 1)]",
        "inputs: [(\"flint\", 1), (\"flint\", 2)]",
    );
    let error = Catalog::from_sources(sources, None).expect_err("duplicate inputs");
    assert!(
        error
            .to_string()
            .contains("duplicate resolved recipe input"),
        "{error}"
    );

    let mut aliases = replace_source("items.ron",
        "name_key: \"ITEM_FLINT\", serde_name: \"Flint\", serde_aliases: [], resolver_aliases: []",
        "name_key: \"ITEM_FLINT\", serde_name: \"Flint\", serde_aliases: [], resolver_aliases: [\"alias_flint\"]");
    let recipes = aliases
        .sources
        .iter_mut()
        .find(|s| s.name == "recipes.ron")
        .expect("recipes");
    recipes.bytes = String::from_utf8(recipes.bytes.clone())
        .expect("utf8")
        .replacen(
            "inputs: [(\"flint\", 1)]",
            "inputs: [(\"flint\", 1), (\"alias_flint\", 1)]",
            1,
        )
        .into_bytes();
    let error = Catalog::from_sources(aliases, None)
        .expect_err("aliases must resolve before duplicate check");
    assert!(
        error
            .to_string()
            .contains("duplicate resolved recipe input"),
        "{error}"
    );

    let sources = replace_source(
        "recipes.ron",
        "output: (\"dried_meat\", 2)",
        "output: (\"stone\", 2)",
    );
    let error = Catalog::from_sources(sources, None).expect_err("unaged nonfood output");
    assert!(
        error
            .to_string()
            .contains("positive recipe duration requires a food output"),
        "{error}"
    );

    let sources = replace_source(
        "recipes.ron",
        "output: (\"dried_meat\", 2)",
        "output: (\"apple\", 2)",
    );
    let extension = RawSources {
        sources: vec![NamedSource {
            name: "items".into(),
            bytes: APPLE_ITEM.as_bytes().to_vec(),
        }],
    };
    let c = Catalog::from_sources(sources, Some(extension))
        .expect("novel food drying remains supported");
    let dry = c
        .recipe(c.recipe_by_name("dry_meat").expect("dry recipe"))
        .expect("definition");
    assert_eq!(dry.output.0, c.item_by_name("apple").expect("novel food"));
    assert_eq!(dry.food_requirements.duration_days, 2);
}
