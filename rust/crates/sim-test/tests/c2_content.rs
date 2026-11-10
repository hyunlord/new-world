//! Bronze recipes use ordinary jobs; authored geometry and knowledge guards remain explicit.
mod colony_support;

use colony_support::{build_workshop, job_world_m2, run_until, worker};
use sim_core::colony::content::{installed_or_bundled, CompletionMaterial};
use sim_core::colony::knowledge::{self, ContentStatus, Knowledge, KnowledgeEntry, Source};
use sim_core::colony::{BillMode, Command, ItemKind, ItemSpec, RecipeKind};
use sim_systems::commands::submit;
use sim_systems::runtime::food::items::{create_ground, FoodSource};
use sim_systems::runtime::jobs::invariants::ledger_of;

const RECIPES: [&str; 20] = [
    "burn_charcoal",
    "smelt_copper",
    "smelt_tin",
    "alloy_bronze",
    "make_mould",
    "cast_copper_axe",
    "cast_bronze_axe",
    "cast_bronze_sickle",
    "cast_bronze_hoe",
    "cast_bronze_spear",
    "cast_bronze_dagger",
    "hammer_awl",
    "hammer_needle",
    "spin_wool",
    "weave_wool",
    "sew_wool_tunic",
    "make_mud_brick",
    "burn_lime",
    "make_wheel",
    "make_composite_bow",
];

#[test]
fn c2_recipes_resolve() {
    let catalog = installed_or_bundled();
    for name in RECIPES {
        let recipe = RecipeKind::from_name(name)
            .expect("bronze recipe")
            .recipe()
            .expect("definition");
        assert!(recipe.supports_workshop(recipe.workshop), "{name}");
        assert!(
            !recipe.food_requirements.knowledge_any.is_empty(),
            "{name} explicit knowledge guard"
        );
        assert!(
            (45..=120).contains(&recipe.work_ticks),
            "{name} authored initial work"
        );
        for station in &recipe.compatible_workshops {
            assert!(recipe
                .workshop_tags
                .iter()
                .all(|tag| station.definition().workshop_tags.contains(tag)));
        }
    }
    let lime = RecipeKind::from_name("burn_lime")
        .expect("inert recipe")
        .recipe()
        .expect("definition");
    assert!(lime
        .food_requirements
        .knowledge_any
        .iter()
        .all(|id| knowledge::defs().nodes[usize::from(*id)].status == ContentStatus::Inert));
    for name in [
        "limestone",
        "lime",
        "honey",
        "beeswax",
        "seal",
        "tablet",
        "glass_bead",
        "iron_ore",
    ] {
        let item = catalog
            .item(catalog.item_by_name(name).expect("inert item"))
            .expect("definition");
        assert_eq!(item.status, ContentStatus::Inert, "{name}");
        assert!(item.debt.as_ref().is_some_and(|debt| !debt.is_empty()));
    }
    assert!(ItemKind::from_name("copper_axe")
        .expect("axe")
        .definition()
        .chop
        .is_some());
    assert!(ItemKind::from_name("bronze_axe")
        .expect("axe")
        .definition()
        .chop
        .is_some());
}

#[test]
fn c2_buildings_resolve_rotations_and_preserve_upgrade_geometry() {
    let catalog = installed_or_bundled();
    let templates = catalog.building_templates();
    for name in [
        "charcoal_clamp",
        "smelter",
        "casting_bench",
        "mud_brick_house",
        "kiln",
    ] {
        let template = templates.get(name).expect("bronze building");
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
    let old = templates.get("log_house").expect("old dwelling");
    let new = templates.get("mud_brick_house").expect("new dwelling");
    assert_eq!(old.upgrade_to.as_deref(), Some(new.id.as_str()));
    assert_eq!(new.upgrade_from.as_deref(), Some(old.id.as_str()));
    let old_plan = old.expand(catalog, None, 0).expect("old plan");
    let new_plan = new.expand(catalog, None, 0).expect("new plan");
    assert_eq!(old_plan.len(), new_plan.len());
    assert_eq!(
        old_plan
            .iter()
            .zip(&new_plan)
            .filter(|(old, new)| old.kind != new.kind)
            .count(),
        11
    );
    let wall = catalog
        .blueprint(catalog.blueprint_by_name("mud_brick_wall").expect("wall"))
        .expect("wall definition");
    assert!(
        matches!(&wall.material, CompletionMaterial::Fixed { material } if material == "mud_brick")
    );
    assert_eq!(
        templates.get("bee_skep").expect("future hive").status,
        ContentStatus::Inert
    );
}

#[test]
fn c2_connected_recipes_physically_craft_with_exact_ledger() {
    for name in RECIPES {
        let recipe = RecipeKind::from_name(name)
            .expect("recipe")
            .recipe()
            .expect("definition");
        let mut engine = job_world_m2(32, 32);
        engine.resources.rules.colony = Some(sim_core::rules::ColonyRule::M4);
        let entries: Vec<_> = knowledge::defs()
            .nodes
            .iter()
            .filter(|node| node.status == ContentStatus::Connected)
            .map(|node| KnowledgeEntry::new(node.id, Source::Seeded, 1000, 0))
            .collect();
        for x in [3, 5, 7] {
            let person = worker(&mut engine, x, 3);
            engine
                .world
                .insert_one(person, Knowledge::from_entries(entries.clone()))
                .expect("fixture knowledge");
        }
        let shop = build_workshop(&mut engine, recipe.workshop, (12, 12), (4, 4));
        let before: Vec<_> = recipe
            .inputs
            .iter()
            .map(|(kind, _)| ledger_of(&engine.world, &engine.resources, *kind).consumed)
            .collect();
        for (index, &(kind, count)) in recipe.inputs.iter().enumerate() {
            create_ground(
                &mut engine.resources,
                (5 + index as u32, 6),
                ItemSpec::raw(kind, None),
                count,
                0,
                FoodSource::Debug,
            )
            .expect("raw recipe fixture input");
        }
        if let Some(tool) = &recipe.tool {
            create_ground(
                &mut engine.resources,
                (9, 6),
                ItemSpec::raw(tool.preferred_item, None),
                tool.count,
                0,
                FoodSource::Debug,
            )
            .expect("reusable tool fixture");
        }
        assert!(
            submit(
                &mut engine,
                Command::AddBill {
                    workshop: shop,
                    recipe: recipe.id,
                    mode: BillMode::Make(1)
                }
            )
            .accepted
        );
        if name == "burn_lime" {
            // Even deliberately supplied reserved limestone cannot bypass the Inert knowledge guard.
            assert_eq!(
                run_until(&mut engine, 400, |e| e.resources.colony.workshops[&shop]
                    .bills[0]
                    .done
                    > 0),
                400
            );
            assert_eq!(
                engine
                    .resources
                    .colony
                    .item_ledger
                    .crafted_of(recipe.output.0),
                0
            );
            for ((kind, _), consumed) in recipe.inputs.iter().zip(before) {
                assert_eq!(
                    ledger_of(&engine.world, &engine.resources, *kind).consumed,
                    consumed
                );
            }
            sim_systems::runtime::jobs::invariants::check(&engine.world, &engine.resources)
                .expect("blocked recipe ledger");
            continue;
        }
        assert!(
            run_until(&mut engine, 4000, |e| e.resources.colony.workshops[&shop]
                .bills[0]
                .done
                == 1)
                < 4000,
            "{name} completed"
        );
        for ((kind, count), consumed) in recipe.inputs.iter().zip(before) {
            assert_eq!(
                ledger_of(&engine.world, &engine.resources, *kind).consumed - consumed,
                u64::from(*count),
                "{name} input {}",
                kind.name()
            );
        }
        assert_eq!(
            engine
                .resources
                .colony
                .item_ledger
                .crafted_of(recipe.output.0),
            u64::from(recipe.output.1),
            "{name} output"
        );
        sim_systems::runtime::jobs::invariants::check(&engine.world, &engine.resources)
            .expect("exact physical ledger");
    }
}
