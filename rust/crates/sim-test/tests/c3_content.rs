//! Iron recipes, stations and templates resolve through the C2 machinery by data alone.

use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::knowledge::ContentStatus;
use sim_core::colony::{ItemKind, RecipeKind};

const RECIPES: [&str; 15] = [
    "smelt_bloom",
    "smelt_bog_bloom",
    "refine_bloom",
    "forge_iron_axe",
    "forge_iron_hoe",
    "forge_iron_sickle",
    "forge_iron_saw",
    "forge_iron_knife",
    "forge_iron_adze",
    "forge_iron_spear",
    "forge_iron_sword",
    "forge_iron_helmet",
    "forge_iron_shield_boss",
    "forge_nails",
    "dress_stone",
];

#[test]
fn c3_recipes_resolve() {
    for (offset, name) in RECIPES.into_iter().enumerate() {
        let kind = RecipeKind::from_name(name).expect("iron recipe");
        assert_eq!(
            kind.code() as usize,
            70 + offset,
            "append-only code: {name}"
        );
        let recipe = kind.recipe().expect("definition");
        assert!(recipe.supports_workshop(recipe.workshop), "{name}");
        assert!(
            !recipe.food_requirements.knowledge_any.is_empty(),
            "{name} explicit knowledge guard"
        );
        // Bloom smelting is the authored long job.
        assert!(
            (45..=240).contains(&recipe.work_ticks),
            "{name} authored initial work"
        );
        for station in &recipe.compatible_workshops {
            assert!(recipe
                .workshop_tags
                .iter()
                .all(|tag| station.definition().workshop_tags.contains(tag)));
        }
    }
    // The distinct bloomery tag keeps copper smelting off the bloomery and bloom smelting off the smelter.
    let smelter = RecipeKind::from_name("smelt_copper")
        .and_then(|r| r.recipe())
        .expect("copper smelting");
    let bloom = RecipeKind::from_name("smelt_bloom")
        .and_then(|r| r.recipe())
        .expect("bloom smelting");
    assert!(!smelter.compatible_workshops.contains(&bloom.workshop));
    assert!(!bloom.compatible_workshops.contains(&smelter.workshop));
    let catalog = installed_or_bundled();
    let item = |name: &str| {
        catalog
            .item(catalog.item_by_name(name).expect(name))
            .expect(name)
    };
    assert_eq!(item("iron_ore").status, ContentStatus::Connected);
    for name in ["coin", "book", "steel_edge"] {
        assert_eq!(item(name).status, ContentStatus::Inert, "{name}");
        assert!(item(name)
            .debt
            .as_ref()
            .is_some_and(|debt| !debt.is_empty()));
    }
    for name in ["iron_bloom", "wrought_iron"] {
        assert!(item(name).tags.iter().any(|t| t == "ingot"), "{name}");
        assert!(item(name).tags.iter().any(|t| t == "metal_iron"), "{name}");
    }
    let chop = |name: &str| {
        ItemKind::from_name(name)
            .and_then(|k| k.definition().chop.as_ref().map(|c| c.quality_permille))
            .expect(name)
    };
    let (iron, bronze) = (chop("iron_axe"), chop("bronze_axe"));
    assert!(iron.iter().zip(bronze.iter()).all(|(i, b)| i < b));
    assert!(item("iron_knife").cutter.is_some());
    assert_eq!(item("iron_adze").tool_capabilities, ["adze"]);
}

#[test]
fn c3_buildings_resolve_and_extend_the_house_upgrade() {
    let catalog = installed_or_bundled();
    let templates = catalog.building_templates();
    for name in ["bloomery", "forge", "cut_stone_house"] {
        let template = templates.get(name).expect("iron building");
        assert_eq!(template.status, ContentStatus::Connected, "{name}");
        for rotation in 0..4 {
            let plan = template
                .expand(catalog, None, rotation)
                .expect("physical footprint and work tile");
            assert_eq!(
                sim_core::colony::building_template::material_cost(&plan, catalog)
                    .expect("material sum"),
                template.materials
            );
        }
    }
    let anvil = templates
        .get("forge")
        .expect("smithy")
        .expand(catalog, Some("anvil"), 0);
    assert!(anvil.is_ok(), "the smithy accepts the anvil alternative");
    let mud = templates.get("mud_brick_house").expect("mud house");
    let stone = templates.get("cut_stone_house").expect("stone house");
    assert_eq!(mud.upgrade_to.as_deref(), Some(stone.id.as_str()));
    assert_eq!(stone.upgrade_from.as_deref(), Some(mud.id.as_str()));
    assert!(
        stone.tier > mud.tier,
        "steward housing prefers the stone house"
    );
    // The historical stone house keeps its saved geometry and log-house predecessor.
    assert_eq!(
        templates
            .get("stone_house")
            .expect("old")
            .upgrade_from
            .as_deref(),
        Some("log_house")
    );
    let old_plan = mud.expand(catalog, None, 0).expect("old plan");
    let new_plan = stone.expand(catalog, None, 0).expect("new plan");
    assert_eq!(old_plan.len(), new_plan.len());
    assert_eq!(
        old_plan
            .iter()
            .zip(&new_plan)
            .filter(|(old, new)| old.kind != new.kind)
            .count(),
        11
    );
    for name in ["warehouse", "market"] {
        assert_eq!(
            templates.get(name).expect(name).status,
            ContentStatus::Inert,
            "{name}"
        );
    }
}
