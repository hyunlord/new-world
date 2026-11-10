//! Native C2-a scene: fixture raw stocks, ordinary crafting/building, and optional fresh saves.
mod spine_support;

use sim_core::colony::{BillMode, BlueprintKind, Command, ItemKind, ItemSpec, RecipeKind};
use sim_engine::SimEngine;
use sim_systems::runtime::food::items::{create_ground, FoodSource};
use spine_support::{anchor, camp, command, complete, scene, until};

fn raw_stock(engine: &mut SimEngine, name: &str, mut count: u16) {
    let kind = ItemKind::from_name(name).expect("raw fixture item");
    let centre = camp(engine);
    let mut tiles: Vec<_> = (centre.1.saturating_sub(12)..=centre.1 + 12)
        .flat_map(|y| (centre.0.saturating_sub(12)..=centre.0 + 12).map(move |x| (x, y)))
        .collect();
    tiles.sort_by_key(|&(x, y)| (x.abs_diff(centre.0) + y.abs_diff(centre.1), y, x));
    for tile in tiles {
        if count == 0 {
            break;
        }
        if !sim_systems::runtime::nav::passable(&engine.resources, tile)
            || engine.resources.colony.ground_items.contains_key(&tile)
            || !engine
                .resources
                .nav
                .reaches(engine.resources.nav.region_of(centre), tile)
        {
            continue;
        }
        let take = count.min(kind.definition().stack_max);
        let day = sim_systems::runtime::food::day(&engine.resources);
        if create_ground(
            &mut engine.resources,
            tile,
            ItemSpec::raw(kind, None),
            take,
            day,
            FoodSource::Debug,
        )
        .is_ok()
        {
            count -= take;
        }
    }
    assert_eq!(count, 0, "enough real ground owners for fixture {name}");
}

fn capture(engine: &mut SimEngine, label: &str, site: u32) {
    sim_systems::runtime::jobs::invariants::check(&engine.world, &engine.resources)
        .expect("physical accounting");
    let Some(directory) = std::env::var_os("C2_CONTENT_SCENE_OUT") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    assert!(directory.is_absolute() && directory.is_dir());
    let ui = spine_support::ui_fixture::manifest(engine, label);
    let bytes = sim_systems::save::save(engine).expect("native C2 save");
    let manifest = serde_json::to_vec_pretty(&serde_json::json!({
        "fixture": "C2-a raw log/stone/clay/flax stocks and connected starting knowledge; all bricks and stations are made by jobs",
        "tick": engine.current_tick(), "selected_site": site, "ui": ui,
        "sites": engine.resources.colony.buildings.sites.values().map(|site| serde_json::json!({
            "id": site.id, "template": site.template, "anchor": site.anchor, "footprint": site.footprint,
        })).collect::<Vec<_>>(),
    })).expect("scene manifest");
    use std::io::Write;
    for (suffix, data) in [("wsave", bytes), ("json", manifest)] {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(format!("{label}.{suffix}")))
            .expect("fresh scene output")
            .write_all(&data)
            .expect("scene bytes");
    }
}

fn build(engine: &mut SimEngine, name: &str) -> u32 {
    let at = anchor(engine, name);
    let id = engine.resources.colony.buildings.next_site_id;
    command(
        engine,
        Command::BuildTemplate {
            id: name.into(),
            anchor: at,
            rotation: 0,
            station: None,
        },
    );
    until(engine, name, |e| complete(e, id));
    id
}

#[test]
fn c2_buildings_physically_complete_and_bricks_are_crafted() {
    let mut engine = scene();
    // Only source materials are provided by this content scene; no brick, fuel or metal output is injected.
    for (name, count) in [
        ("log", 100),
        ("stone", 20),
        ("clay", 60),
        ("flax_stalks", 17),
    ] {
        raw_stock(&mut engine, name, count);
    }
    let clamp = build(&mut engine, "charcoal_clamp");
    let clamp_kind = BlueprintKind::from_name("charcoal_clamp").expect("clamp");
    let shop = engine
        .resources
        .colony
        .workshops
        .values()
        .find(|shop| shop.kind == clamp_kind)
        .expect("physical clamp")
        .id;
    command(
        &mut engine,
        Command::AddBill {
            workshop: shop,
            recipe: RecipeKind::from_name("burn_charcoal").expect("charcoal recipe"),
            mode: BillMode::Make(1),
        },
    );
    until(&mut engine, "charcoal from actual logs", |e| {
        e.resources.colony.workshops[&shop].bills[0].done == 1
    });
    capture(&mut engine, "00-charcoal-clamp", clamp);
    let smelter = build(&mut engine, "smelter");
    capture(&mut engine, "01-smelter-house", smelter);
    let casting = build(&mut engine, "casting_bench");
    capture(&mut engine, "02-casting-house", casting);
    let pottery = BlueprintKind::from_name("pottery_bench").expect("outdoor brick work spot");
    let tile = spine_support::ui_fixture::free_tile(&engine);
    command(
        &mut engine,
        Command::PlaceBlueprint {
            kind: pottery,
            x: tile.0,
            y: tile.1,
        },
    );
    until(&mut engine, "outdoor pottery bench", |e| {
        e.resources
            .colony
            .workshops
            .values()
            .any(|shop| shop.kind == pottery)
    });
    let shop = engine
        .resources
        .colony
        .workshops
        .values()
        .find(|shop| shop.kind == pottery)
        .expect("built pottery bench")
        .id;
    command(
        &mut engine,
        Command::AddBill {
            workshop: shop,
            recipe: RecipeKind::from_name("make_mud_brick").expect("brick recipe"),
            mode: BillMode::Make(17),
        },
    );
    until(&mut engine, "34 bricks from clay and flax stalks", |e| {
        e.resources.colony.workshops[&shop].bills[0].done == 17
    });
    let brick = ItemKind::from_name("mud_brick").expect("brick");
    assert_eq!(engine.resources.colony.item_ledger.crafted_of(brick), 34);
    let dwelling = build(&mut engine, "mud_brick_house");
    assert_eq!(engine.resources.colony.item_ledger.consumed_of(brick), 33);
    capture(&mut engine, "03-mud-brick-house", dwelling);
    let saved = sim_systems::save::save(&mut engine).expect("built scene save");
    let mut loaded = sim_systems::save::load(&saved).expect("built scene load");
    assert_eq!(
        sim_systems::save::save(&mut loaded).expect("roundtrip"),
        saved
    );
    assert!(sim_systems::runtime::building_templates::is_intact(
        &loaded.resources,
        &loaded.resources.colony.buildings.sites[&dwelling]
    ));
}
