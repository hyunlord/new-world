//! Scripted production-scene acceptance: planted flax becomes physically worn linen.
//! Starting knowledge and raw supplies are explicit scenario fixtures, not natural discovery.
//! Optional WORLDSIM_WEAVING_SCENE_OUT exports real saves at each completed step.
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::cultivation::FieldWork;
use sim_core::colony::knowledge::{self, Knowledge, KnowledgeEntry, Source};
use sim_core::colony::people::Apparel;
use sim_core::colony::{BillMode, BlueprintKind, Command, ItemKind, ItemSpec, RecipeKind};
use sim_core::components::Agent;
use sim_engine::SimEngine;
use sim_systems::commands::{check_blueprint, submit};
use sim_systems::runtime::food::items::{create_ground, FoodSource};
use sim_systems::runtime::jobs::{invariants, items};
use std::collections::BTreeSet;

type Tile = (u32, u32);
const DAY: u64 = 1_440;

fn command(engine: &mut SimEngine, command: Command) {
    let result = submit(engine, command.clone());
    assert!(
        result.accepted,
        "scripted command rejected: {command:?}: {result:?}"
    );
}

fn item(name: &str) -> ItemKind {
    ItemKind::from_name(name).expect("scripted content exists")
}

fn created(engine: &SimEngine, name: &str) -> u64 {
    engine
        .resources
        .colony
        .item_ledger
        .created
        .get(&item(name))
        .copied()
        .unwrap_or(0)
}

fn ordered_tiles(engine: &SimEngine, centre: Tile) -> Vec<Tile> {
    let mut tiles: Vec<_> = (1..engine.resources.tile_grid.height - 1)
        .flat_map(|y| (1..engine.resources.tile_grid.width - 1).map(move |x| (x, y)))
        .filter(|&tile| items::chebyshev(tile, centre) <= 24)
        .collect();
    tiles.sort_by_key(|&tile| (items::chebyshev(tile, centre), tile.1, tile.0));
    tiles
}

fn wet(engine: &SimEngine, tile: Tile) -> bool {
    let (x, y) = tile;
    !items::is_water(&engine.resources, tile)
        && [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
            .into_iter()
            .any(|near| items::is_water(&engine.resources, near))
}

fn place(engine: &mut SimEngine, name: &str, centre: Tile, river: bool) -> Tile {
    let kind = BlueprintKind::from_name(name).expect("workshop content");
    let tile = ordered_tiles(engine, centre)
        .into_iter()
        .find(|&tile| {
            (!river || wet(engine, tile))
                && sim_systems::commands::camp_reaches(&engine.resources, tile)
                && check_blueprint(&engine.resources, kind, tile.0, tile.1).is_ok()
        })
        .expect("scripted site has a real reachable footprint");
    command(
        engine,
        Command::PlaceBlueprint {
            kind,
            x: tile.0,
            y: tile.1,
        },
    );
    tile
}

fn checkpoint(engine: &mut SimEngine, label: &str) {
    invariants::check(&engine.world, &engine.resources)
        .expect("exact physical ledger at scene step");
    println!("WEAVING_STEP {label} tick={}", engine.current_tick());
    if let Some(directory) = std::env::var_os("WORLDSIM_WEAVING_SCENE_OUT") {
        let directory = std::path::PathBuf::from(directory);
        assert!(
            directory.is_absolute() && directory.is_dir(),
            "existing external evidence directory"
        );
        let bytes = sim_systems::save::save(engine).expect("real production scene save");
        let path = directory.join(format!("{label}.wsave"));
        use std::io::Write;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .expect("never overwrite a scene capture input")
            .write_all(&bytes)
            .expect("scene save bytes");
    }
}

fn until(engine: &mut SimEngine, deadline: u64, label: &str, ready: impl Fn(&SimEngine) -> bool) {
    while engine.current_tick() < deadline && !ready(engine) {
        engine.tick();
    }
    assert!(
        ready(engine),
        "scene did not reach {label} by tick {}; fields={:?}; workshops={:?}; pending={:?}",
        engine.current_tick(),
        engine.resources.cultivation.fields,
        engine.resources.colony.workshops,
        engine.resources.processing.pending
    );
    checkpoint(engine, label);
}

fn bill(engine: &mut SimEngine, workshop: u32, recipe: &str, count: u16) {
    command(
        engine,
        Command::AddBill {
            workshop,
            recipe: RecipeKind::from_name(recipe).expect("recipe content"),
            mode: BillMode::Make(count),
        },
    );
}

#[test]
fn production_scene_harvests_rets_spins_weaves_tailors_and_wears_both_garments() {
    let mut engine = sim_test::colony_engine(0);
    // An explicitly knowledgeable demonstration colony; needs, movement, weather,
    // construction, crop growth, production and clothing all retain production rules.
    let entries: Vec<_> = knowledge::defs()
        .nodes
        .iter()
        .enumerate()
        .map(|(index, _)| KnowledgeEntry::new(index as u8, Source::Seeded, 1000, 0))
        .collect();
    let mut people: Vec<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(e, a)| (a.id, e))
        .collect();
    people.sort_by_key(|&(id, _)| id);
    for (_, entity) in people {
        engine
            .world
            .insert_one(entity, Knowledge::from_entries(entries.clone()))
            .expect("fixture knowledge");
    }
    command(
        &mut engine,
        Command::SetSteward {
            on: false,
            scope: Default::default(),
        },
    );
    command(&mut engine, Command::SetDressOrder { on: false });
    // Settlement membership is established by the real first runtime ticks.
    while engine.resources.settlements.is_empty() && engine.current_tick() < DAY {
        engine.tick();
    }
    let home = engine
        .resources
        .settlements
        .values()
        .min_by_key(|s| s.settlement_id)
        .expect("production founders have a home")
        .formation_tile;
    let pit = place(&mut engine, "retting_pit", home, true);
    let bench = place(&mut engine, "wood_bench", home, false);
    let loom = place(&mut engine, "loom", home, false);
    let crop = installed_or_bundled()
        .crop_by_name("flax")
        .expect("flax crop");
    let mut fields = Vec::new();
    for tile in ordered_tiles(&engine, home) {
        if sim_systems::runtime::cultivation::commands::zoneable(&engine.resources, tile) {
            command(
                &mut engine,
                Command::ZoneField {
                    min: tile,
                    max: tile,
                    crop,
                },
            );
            command(
                &mut engine,
                Command::SetFieldWork {
                    tile,
                    mode: FieldWork::Now,
                },
            );
            fields.push(tile);
            if fields.len() == 2 {
                break;
            }
        }
    }
    assert_eq!(fields.len(), 2);
    let stock = ordered_tiles(&engine, home)
        .into_iter()
        .find(|&(x, y)| {
            x + 3 < engine.resources.tile_grid.width
                && y + 3 < engine.resources.tile_grid.height
                && (y..y + 4).all(|sy| {
                    (x..x + 4).all(|sx| {
                        sim_systems::runtime::cultivation::commands::zoneable(
                            &engine.resources,
                            (sx, sy),
                        )
                    })
                })
        })
        .expect("reachable clear raw supply stockpile");
    command(
        &mut engine,
        Command::AddStockpile {
            x0: stock.0,
            y0: stock.1,
            x1: stock.0 + 3,
            y1: stock.1 + 3,
        },
    );
    // Raw initial stores only: no stalks, fibres, thread, cloth, spindle or garment.
    for (name, count) in [
        ("log", 40),
        ("stone", 10),
        ("wild_grain", 200),
        ("flax_seed", 4),
    ] {
        create_ground(
            &mut engine.resources,
            stock,
            ItemSpec::raw(item(name), None),
            count,
            0,
            FoodSource::Debug,
        )
        .expect("accounted raw starting inventory");
    }
    for name in [
        "flax_stalks",
        "flax_fibre",
        "linen_thread",
        "linen_cloth",
        "hand_spindle",
        "linen_tunic",
        "linen_wrap",
    ] {
        assert_eq!(
            created(&engine, name),
            0,
            "no injected intermediate: {name}"
        );
    }
    engine.tick(); // Commit this tick's command provenance before saving the scene.
    checkpoint(&mut engine, "00-start");
    let sites = [pit, bench, loom];
    until(&mut engine, 3 * DAY, "01-workshops", |e| {
        sites.iter().all(|tile| {
            e.resources
                .colony
                .workshops
                .values()
                .any(|shop| shop.top_left == *tile)
        })
    });
    let shops: Vec<_> = sites
        .iter()
        .map(|tile| {
            engine
                .resources
                .colony
                .workshops
                .values()
                .find(|shop| shop.top_left == *tile)
                .expect("completed workshop")
                .id
        })
        .collect();
    bill(&mut engine, shops[1], "make_hand_spindle", 1);
    bill(&mut engine, shops[0], "ret_flax", 3);
    bill(&mut engine, shops[1], "spin_thread", 3);
    bill(&mut engine, shops[2], "weave_cloth", 3);
    bill(&mut engine, shops[2], "make_linen_tunic", 1);
    bill(&mut engine, shops[2], "make_linen_wrap", 1);
    until(&mut engine, 8 * DAY, "02-harvested", |e| {
        created(e, "flax_stalks") >= 6
    });
    until(&mut engine, 11 * DAY, "03-retted", |e| {
        created(e, "flax_fibre") >= 2
    });
    until(&mut engine, 12 * DAY, "04-spun", |e| {
        created(e, "linen_thread") >= 2
    });
    until(&mut engine, 13 * DAY, "05-woven", |e| {
        created(e, "linen_cloth") >= 1
    });
    until(&mut engine, 17 * DAY, "06-tailored", |e| {
        created(e, "linen_tunic") == 1 && created(e, "linen_wrap") == 1
    });
    assert_eq!(created(&engine, "hand_spindle"), 1);
    assert_eq!(created(&engine, "flax_fibre"), 6);
    assert_eq!(created(&engine, "linen_thread"), 6);
    assert_eq!(created(&engine, "linen_cloth"), 3);
    assert!(
        !engine
            .world
            .query::<&Apparel>()
            .iter()
            .any(|(_, a)| { [item("linen_tunic"), item("linen_wrap")].contains(&a.top.item()) }),
        "player's Dress pause holds through tailoring"
    );
    // The initial raw stores fill the small stockpile. The player opens storage
    // for completed garments before requesting dressing; loose goods are hauled.
    command(
        &mut engine,
        Command::AddStockpile {
            x0: stock.0,
            y0: stock.1,
            x1: stock.0 + 5,
            y1: stock.1 + 5,
        },
    );
    command(&mut engine, Command::SetDressOrder { on: true });
    until(&mut engine, 18 * DAY, "07-worn", |e| {
        let worn: BTreeSet<_> = e
            .world
            .query::<&Apparel>()
            .iter()
            .map(|(_, a)| a.top.item())
            .collect();
        worn.contains(&item("linen_tunic")) && worn.contains(&item("linen_wrap"))
    });
    for kind in [
        ChronicleKind::FirstHarvest,
        ChronicleKind::FirstCloth,
        ChronicleKind::FirstTunic,
    ] {
        assert_eq!(
            engine
                .resources
                .chronicle
                .entries()
                .iter()
                .filter(|entry| entry.kind == kind)
                .count(),
            1,
            "one truthful committed first event: {kind:?}"
        );
    }
    for (_, apparel) in engine.world.query::<&Apparel>().iter() {
        if let Some(days) = apparel
            .top
            .item()
            .definition()
            .apparel
            .as_ref()
            .and_then(|a| a.wear_days)
        {
            assert!(apparel
                .wear_remaining_hours
                .is_some_and(|hours| hours > 0 && hours <= days * 24));
        }
    }
    let bytes = sim_systems::save::save(&mut engine).expect("completed scene save");
    let mut loaded = sim_systems::save::load(&bytes).expect("completed scene reload");
    for _ in 0..120 {
        engine.tick();
        loaded.tick();
    }
    assert!(
        sim_systems::save::save(&mut engine).expect("continuous")
            == sim_systems::save::save(&mut loaded).expect("reloaded"),
        "completed physical scene roundtrip"
    );
    println!("WEAVING_CHAIN_COMPLETE seed=0 tick={} fields={fields:?} pit={pit:?} bench={bench:?} loom={loom:?} stock={stock:?}", engine.current_tick());
}
