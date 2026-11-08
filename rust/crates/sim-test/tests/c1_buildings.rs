//! C1 templates physically complete through ordinary work and retain their storage/pen functions.
mod spine_support;
use sim_core::colony::content::FoodStorageClass;
use sim_core::colony::food::FoodOwner;
use sim_core::colony::{Command, ItemKind, ItemSpec};
use sim_systems::runtime::{building_templates, food, jobs, rooms};
use spine_support::*;

#[test]
fn c1_granary_construction_filters_stock_and_roundtrips_protection() {
    let mut engine = scene();
    harvest_logs(&mut engine);
    let anchor = anchor(&engine, "granary");
    let id = engine.resources.colony.buildings.next_site_id;
    command(
        &mut engine,
        Command::BuildTemplate {
            id: "granary".to_owned(),
            anchor,
            rotation: 0,
            station: None,
        },
    );
    until(&mut engine, "grain store completed", |engine| {
        complete(engine, id)
    });
    let tile = (anchor.1 + 1..anchor.1 + 3)
        .flat_map(|y| (anchor.0 + 1..anchor.0 + 3).map(move |x| (x, y)))
        .find(|tile| !engine.resources.colony.ground_items.contains_key(tile))
        .expect("empty physical granary cell");
    let owner = FoodOwner::Ground(tile);
    assert!(engine.resources.colony.zone_of_tile.contains_key(&tile));
    assert!(rooms::covered(&engine.resources, tile));
    assert_eq!(
        food::storage::class(&engine.resources, owner),
        FoodStorageClass::Granary
    );
    let grain = ItemKind::from_name("wild_grain").expect("grain");
    assert!(building_templates::storage::accepts(
        &engine.resources,
        tile,
        grain
    ));
    assert!(!building_templates::storage::accepts(
        &engine.resources,
        tile,
        ItemKind::Log
    ));
    assert_eq!(
        jobs::items::room_on(&engine.resources, tile, ItemSpec::raw(ItemKind::Log, None)),
        0
    );
    let born = food::day(&engine.resources);
    food::items::create_ground(
        &mut engine.resources,
        tile,
        ItemSpec::raw(grain, None),
        2,
        born,
        food::items::FoodSource::Debug,
    )
    .expect("dated grain fixture in physically built storage");
    let age = engine.resources.food.ages[&owner];
    assert_eq!(age.born_day, born);
    let saved = sim_systems::save::save(&mut engine).expect("grain store save");
    let mut loaded = sim_systems::save::load(&saved).expect("grain store load");
    assert_eq!(
        food::storage::class(&loaded.resources, owner),
        FoodStorageClass::Granary
    );
    assert_eq!(
        sim_systems::save::save(&mut loaded).expect("roundtrip"),
        saved
    );
    assert_eq!(loaded.resources.food.ages[&owner], age);
    let definition = grain.definition().food.as_ref().expect("grain food");
    let ordinary_end = born + definition.lifetime_days(FoodStorageClass::RoofedStore);
    let granary_end = born + definition.lifetime_days(FoodStorageClass::Granary);
    assert!(granary_end > ordinary_end);
    assert_eq!(
        age.remaining(
            ordinary_end,
            definition.lifetime_days(FoodStorageClass::RoofedStore)
        ),
        0
    );
    // Exercise the daily maintenance directly, with its resource clock at the same day.
    loaded.resources.current_tick = (u64::from(ordinary_end) * loaded.resources.ticks_per_day)
        .saturating_sub(loaded.resources.rules.clock_offset_ticks);
    assert_eq!(food::day(&loaded.resources), ordinary_end);
    food::storage::spoil_day(&mut loaded.world, &mut loaded.resources, ordinary_end)
        .expect("ordinary roofed expiry boundary");
    assert_eq!(
        food::items::stack(&loaded.world, &loaded.resources, owner)
            .expect("protected grain")
            .count,
        2
    );
    assert_eq!(
        loaded.resources.food.ages[&owner], age,
        "storage never resets age"
    );
    assert_eq!(
        food::storage::remaining_days(&loaded.world, &loaded.resources, owner, ordinary_end)
            .expect("protected lifetime"),
        granary_end - ordinary_end
    );
    food::validate(&loaded.world, &loaded.resources).expect("exact food ledger while protected");
    let spoiled_before = loaded
        .resources
        .food
        .tallies
        .spoiled
        .get(&grain)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        food::storage::remaining_days(&loaded.world, &loaded.resources, owner, granary_end)
            .expect("exhausted granary lifetime"),
        0
    );
    loaded.resources.current_tick = (u64::from(granary_end) * loaded.resources.ticks_per_day)
        .saturating_sub(loaded.resources.rules.clock_offset_ticks);
    assert_eq!(food::day(&loaded.resources), granary_end);
    food::storage::spoil_day(&mut loaded.world, &mut loaded.resources, granary_end)
        .expect("granary expiry boundary");
    assert!(food::items::stack(&loaded.world, &loaded.resources, owner).is_none());
    assert!(!loaded.resources.food.ages.contains_key(&owner));
    assert_eq!(
        loaded
            .resources
            .food
            .tallies
            .spoiled
            .get(&grain)
            .copied()
            .unwrap_or(0)
            - spoiled_before,
        2
    );
    food::validate(&loaded.world, &loaded.resources)
        .expect("exact food ledger after grain expires");
    sim_systems::runtime::jobs::invariants::check(&loaded.world, &loaded.resources)
        .expect("physical item ledger after grain expires");
    let wall = engine.resources.colony.buildings.sites[&id]
        .pieces
        .iter()
        .find(|piece| {
            piece.kind.definition().completion
                == sim_core::colony::content::CompletionCategory::Wall
        })
        .expect("actual enclosing wall")
        .tile;
    command(
        &mut engine,
        Command::DesignateDeconstruct {
            x0: wall.0,
            y0: wall.1,
            x1: wall.0,
            y1: wall.1,
        },
    );
    until(
        &mut engine,
        "grain store protection removed with its wall",
        |engine| {
            !building_templates::is_intact(
                &engine.resources,
                &engine.resources.colony.buildings.sites[&id],
            )
        },
    );
    assert_ne!(
        food::storage::class(&engine.resources, owner),
        FoodStorageClass::Granary
    );
}

#[test]
fn c1_pen_construction_is_open_and_person_passable() {
    let mut engine = scene();
    harvest_logs(&mut engine);
    let anchor = anchor(&engine, "pen");
    let id = engine.resources.colony.buildings.next_site_id;
    command(
        &mut engine,
        Command::BuildTemplate {
            id: "pen".to_owned(),
            anchor,
            rotation: 0,
            station: None,
        },
    );
    until(&mut engine, "pen physically completed", |engine| {
        complete(engine, id)
    });
    let site = &engine.resources.colony.buildings.sites[&id];
    assert!(building_templates::is_intact(&engine.resources, site));
    let barriers: Vec<_> = site
        .pieces
        .iter()
        .filter(|piece| piece.kind.definition().blocks_livestock)
        .collect();
    assert_eq!(barriers.len(), 15);
    assert!(barriers
        .iter()
        .all(|piece| sim_systems::runtime::nav::passable(&engine.resources, piece.tile)));
    let interior = (anchor.0 + 2, anchor.1 + 2);
    assert!(
        !rooms::roofed(&engine.resources, interior),
        "an open pen grants no shelter"
    );
    rooms::detect_rooms(&engine.world, &mut engine.resources);
    assert!(
        !rooms::roofed(&engine.resources, interior),
        "physical fences grant no roof before template association"
    );
    building_templates::associate_rooms(&mut engine.resources);
    let saved = sim_systems::save::save(&mut engine).expect("pen save");
    let loaded = sim_systems::save::load(&saved).expect("pen load");
    assert!(building_templates::is_intact(
        &loaded.resources,
        &loaded.resources.colony.buildings.sites[&id]
    ));
    assert!(!rooms::roofed(&loaded.resources, interior));
}
