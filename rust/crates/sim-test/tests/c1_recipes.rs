//! C1 registry closure, physical recipe accounting and optional native scene fixture.
mod colony_support;
mod spine_support;

use colony_support::{build_workshop, job_world_m2, run_until, worker};
use sim_core::colony::knowledge::{self, Knowledge, KnowledgeEntry, Source};
use sim_core::colony::{BillMode, BlueprintKind, Command, ItemSpec, RecipeKind};
use sim_systems::commands::submit;
use sim_systems::runtime::food::items::{create_ground, FoodSource};
use sim_systems::runtime::jobs::invariants::ledger_of;

#[test]
fn c1_recipes_resolve() {
    for recipe in sim_core::colony::workshop::recipes() {
        assert!(
            !recipe.compatible_workshops.is_empty(),
            "{}",
            recipe.identity.id
        );
        assert!(recipe.supports_workshop(recipe.workshop));
        for station in &recipe.compatible_workshops {
            assert!(
                recipe
                    .workshop_tags
                    .iter()
                    .all(|tag| station.definition().workshop_tags.contains(tag)),
                "{} station tags",
                recipe.identity.id
            );
        }
    }
    for name in [
        "grind_flour",
        "bake_bread",
        "brew_beer",
        "ferment_milk",
        "smoke_meat",
        "weave_basket",
        "make_hoe",
        "make_sickle",
        "make_adze",
        "make_bow",
        "make_arrows",
        "dye_tunic",
        "make_leather_armor",
        "prepare_poultice",
    ] {
        let recipe = RecipeKind::from_name(name)
            .expect("requested recipe")
            .recipe()
            .expect("definition");
        assert!(
            !recipe.food_requirements.knowledge_any.is_empty(),
            "{name} gated"
        );
        assert!((45..=120).contains(&recipe.work_ticks));
    }
}

#[test]
fn c1_food_recipes_physically_craft_with_exact_ledger() {
    for name in [
        "grind_flour",
        "bake_bread",
        "brew_beer",
        "ferment_milk",
        "smoke_meat",
    ] {
        let recipe = RecipeKind::from_name(name)
            .expect("recipe")
            .recipe()
            .expect("definition");
        let mut engine = job_world_m2(32, 32);
        engine.resources.rules.colony = Some(sim_core::rules::ColonyRule::M4);
        let entries: Vec<_> = knowledge::defs()
            .nodes
            .iter()
            .enumerate()
            .map(|(i, _)| KnowledgeEntry::new(i as u8, Source::Seeded, 1000, 0))
            .collect();
        for x in [3, 5, 7] {
            let entity = worker(&mut engine, x, 3);
            engine
                .world
                .insert_one(entity, Knowledge::from_entries(entries.clone()))
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
            .expect("authored raw fixture input");
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
        assert!(
            run_until(&mut engine, 3000, |e| e.resources.colony.workshops[&shop]
                .bills[0]
                .done
                == 1)
                < 3000,
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
        sim_systems::runtime::food::validate(&engine.world, &engine.resources)
            .expect("exact food units and provenance");
    }
}

#[test]
fn c1_native_recipe_station_scene() {
    let Some(directory) = std::env::var_os("C1_SCENE_OUT") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    assert!(directory.is_absolute() && directory.is_dir());
    let mut engine = spine_support::scene();
    let camp = spine_support::camp(&engine);
    let mut workshops = std::collections::BTreeMap::new();
    for (station, names) in [
        ("clay_oven", vec!["bake_bread"]),
        ("pottery_bench", vec!["brew_beer", "ferment_milk"]),
        ("smoking_rack", vec!["smoke_meat"]),
    ] {
        let kind = BlueprintKind::from_name(station).expect("station");
        let mut tiles: Vec<_> = (camp.1.saturating_sub(12)..=camp.1 + 12)
            .flat_map(|y| (camp.0.saturating_sub(12)..=camp.0 + 12).map(move |x| (x, y)))
            .collect();
        tiles.sort_by_key(|&(x, y)| (x.abs_diff(camp.0) + y.abs_diff(camp.1), y, x));
        let at = tiles
            .into_iter()
            .find(|&(x, y)| submit(&mut engine, Command::PlaceBlueprint { kind, x, y }).accepted)
            .expect("placeable station near camp");
        let rule = engine.resources.rules.colony.expect("colony");
        let (_, needs, _) = kind.plan(&rule);
        let day = sim_systems::runtime::food::day(&engine.resources);
        for line in needs.used() {
            create_ground(
                &mut engine.resources,
                camp,
                ItemSpec::raw(line.kind, None),
                line.needed,
                day,
                FoodSource::Debug,
            )
            .expect("station construction fixture inputs");
        }
        spine_support::until(&mut engine, station, |e| {
            e.resources
                .colony
                .workshops
                .values()
                .any(|s| s.top_left == at)
        });
        let id = engine
            .resources
            .colony
            .workshops
            .values()
            .find(|s| s.top_left == at)
            .expect("constructed station")
            .id;
        for name in names {
            workshops.insert(name, id);
        }
    }
    let ui = spine_support::ui_fixture::manifest(&mut engine, "00-recipes");
    let bytes = sim_systems::save::save(&mut engine).expect("native save");
    let manifest = serde_json::to_vec_pretty(
        &serde_json::json!({"tick":engine.current_tick(), "workshops":workshops, "ui":ui}),
    )
    .expect("manifest");
    use std::io::Write;
    for (name, data) in [("00-recipes.wsave", bytes), ("00-recipes.json", manifest)] {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(name))
            .expect("fresh fixture output")
            .write_all(&data)
            .expect("fixture output");
    }
}
