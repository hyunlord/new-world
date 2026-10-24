//! M3 bridge collectors (`sim_bridge::ffi::space`): the order path, the drag
//! preview, built space, rooms, a room's panel and the inspector's bed line,
//! on the colony world; their empty form in the baseline. M-ART2: the wall and
//! floor material, style and category columns against `TileGrid` and the
//! sim-core material files read from disk (MART2_DECISIONS D6).

use sim_bridge::ffi::{
    collect_agent_economy, collect_colony_overlay, collect_colony_versions, collect_place_preview,
    collect_room_detail, collect_rooms, collect_space, submit_space, SpaceCommand,
};
use sim_core::colony::materials::OAK;
use sim_core::components::{Agent, Child, Elder};
use sim_engine::{SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::runtime::jobs::items::can_hold_items;
use sim_systems::runtime::nav;

/// A free `w × h` area near the hearth (every tile can hold items, no tree or node).
fn open_area(engine: &SimEngine, w: u32, h: u32) -> (u32, u32) {
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    let r = &engine.resources;
    let free = |t: (u32, u32)| {
        can_hold_items(r, t) && !r.colony.trees.contains_key(&t) && !r.colony.nodes.contains_key(&t)
    };
    for d in 4..40u32 {
        for dx in 0..=2 * d {
            let (x, y) = ((hearth.0 + dx).saturating_sub(d), hearth.1 + d);
            if (y..y + h).all(|ty| (x..x + w).all(|tx| free((tx, ty)))) {
                return (x, y);
            }
        }
    }
    panic!("no free area");
}

#[test]
fn space_orders_preview_and_reads_follow_the_colony() {
    let mut engine = sim_test::colony_engine(1);
    for _ in 0..5 {
        engine.tick();
    }
    let (x, y) = open_area(&engine, 5, 5);
    // The preview of a hollow 5 × 5 of log walls: 16 tiles, all fit.
    let p = collect_place_preview(
        &engine.world,
        &engine.resources,
        "log_wall",
        (x as i32, y as i32, x as i32 + 4, y as i32 + 4),
        2,
    );
    assert_eq!(p.tiles.len(), 16);
    assert!(p.reasons.iter().all(|r| r.is_empty()), "{:?}", p.reasons);
    // A line preview follows Bresenham: 5 tiles.
    let line = collect_place_preview(
        &engine.world,
        &engine.resources,
        "log_wall",
        (x as i32, y as i32, x as i32 + 4, y as i32),
        1,
    );
    assert_eq!(line.tiles.len(), 5);
    // An unknown kind previews nothing; a building previews its top-left.
    assert!(
        collect_place_preview(&engine.world, &engine.resources, "castle", (0, 0, 1, 1), 2)
            .tiles
            .is_empty()
    );
    assert_eq!(
        collect_place_preview(&engine.world, &engine.resources, "hut", (3, 3, 9, 9), 2)
            .tiles
            .len(),
        1
    );
    // The order through the bridge path: placed and logged.
    let r = submit_space(
        &mut engine,
        SpaceCommand::Rect {
            kind: "log_wall".into(),
            rect: (x as i32, y as i32, x as i32 + 4, y as i32 + 4),
            hollow: true,
        },
    );
    assert_eq!((r.accepted, r.count), (true, 16));
    assert_eq!(
        engine
            .resources
            .colony
            .command_log
            .last()
            .map(|c| c.command.name()),
        Some("place_rect")
    );
    // The preview now refuses the same tiles (a site is there).
    let again = collect_place_preview(
        &engine.world,
        &engine.resources,
        "log_wall",
        (x as i32, y as i32, x as i32 + 4, y as i32 + 4),
        2,
    );
    assert!(again.reasons.iter().all(|r| *r == "blocked"));
    // Wrong names never reach the log.
    let before = engine.resources.colony.command_log.len();
    let bad = submit_space(
        &mut engine,
        SpaceCommand::Line {
            kind: "hut".into(),
            rect: (0, 0, 3, 0),
        },
    );
    assert!(!bad.accepted);
    let bad = submit_space(
        &mut engine,
        SpaceCommand::RoomRole {
            room: 1,
            role: "castle".into(),
        },
    );
    assert_eq!(bad.reason.map(|r| r.name()), Some("unknown"));
    assert_eq!(engine.resources.colony.command_log.len(), before);
    // Build the walls (as the job layer writes them), a door, two bedrolls.
    let sites: Vec<hecs::Entity> = engine
        .world
        .query::<(&sim_core::colony::ColonySite,)>()
        .iter()
        .filter(|(_, (s,))| s.kind.is_space())
        .map(|(e, _)| e)
        .collect();
    for site in sites {
        sim_systems::runtime::jobs::space::complete_site(
            &mut engine.world,
            &mut engine.resources,
            site,
        );
    }
    let door = (x + 2, y + 4);
    let g = &mut engine.resources.tile_grid;
    let i = g.idx(door.0, door.1);
    g.set_door(i, true);
    nav::refresh(&mut engine.resources, door);
    for t in [(x + 1, y + 1), (x + 2, y + 1)] {
        engine.resources.colony.beds.insert(t, OAK);
        engine
            .resources
            .set_sleep_tile(t.0, t.1, RESOURCE_SOURCE_INFINITE);
    }
    engine.resources.colony.rooms_dirty = true;
    let v0 = collect_colony_versions(&engine.resources);
    engine.tick();
    let v1 = collect_colony_versions(&engine.resources);
    assert!(v1.rooms > v0.rooms, "rooms_version moved");
    // get_space: 16 walls (one a door), the bedrolls.
    let space = collect_space(&engine.resources);
    assert_eq!(space.walls.len(), 16);
    assert_eq!(space.walls.iter().filter(|w| w.1 == 2).count(), 1);
    assert_eq!(space.walls.iter().filter(|w| w.1 == 0).count(), 15);
    assert_eq!(space.beds, vec![(x + 1, y + 1), (x + 2, y + 1)]);
    assert!(space
        .walls
        .windows(2)
        .all(|w| (w[0].0 .1, w[0].0 .0) < (w[1].0 .1, w[1].0 .0)));
    // get_rooms: one bedroom of 9 tiles with 2 beds and 1 door, its tiles listed.
    let rooms = collect_rooms(&engine.resources);
    assert_eq!(rooms.rows.len(), 1);
    let row = &rooms.rows[0];
    assert_eq!(
        (row.role.name(), row.tiles, row.beds, row.doors, row.roofed),
        ("bedroom", 9, 2, 1, true)
    );
    assert_eq!(rooms.tiles.len(), 9);
    assert!(rooms.tiles.iter().all(|t| t.1 == row.id));
    assert_eq!(row.centre, (x + 2, y + 2));
    let detail = collect_room_detail(&engine.world, &engine.resources, i64::from(row.id));
    assert!(detail.found);
    assert!(!collect_room_detail(&engine.world, &engine.resources, 999).found);
    // The override through the bridge.
    let set = submit_space(
        &mut engine,
        SpaceCommand::RoomRole {
            room: i64::from(row.id),
            role: "storehouse".into(),
        },
    );
    assert!(set.accepted);
    let rooms = collect_rooms(&engine.resources);
    assert_eq!(
        (rooms.rows[0].role.name(), rooms.rows[0].overridden),
        ("storehouse", true)
    );
    // The overlay's covered column matches its item rows.
    let overlay = collect_colony_overlay(&engine.world, &engine.resources);
    assert_eq!(overlay.item_covered.len(), overlay.items.len());
    // The inspector's bed line: someone with a bedroll bed shows its room.
    let mut adults: Vec<u64> = engine
        .world
        .query::<&Agent>()
        .without::<&Child>()
        .without::<&Elder>()
        .iter()
        .map(|(_, a)| a.id)
        .collect();
    adults.sort_unstable();
    engine.resources.bed_of.insert(adults[0], (x + 1, y + 1));
    let econ = collect_agent_economy(&engine.world, &engine.resources, adults[0]).expect("row");
    assert_eq!(econ.bed, Some((x + 1, y + 1)));
    assert_eq!(econ.bed_room.map(|r| (r.0, r.2)), Some((row.id, true)));
}

#[test]
fn the_baseline_has_no_built_space_or_rooms() {
    let engine = sim_test::baseline_engine(0);
    let space = collect_space(&engine.resources);
    assert!(space.walls.is_empty() && space.floors.is_empty() && space.beds.is_empty());
    assert!(collect_rooms(&engine.resources).rows.is_empty());
}

/// Every sim-core material file's id → category name, read from disk here (independent of the bridge's
/// embedded catalogue), with the number of files.
fn disk_categories() -> (std::collections::BTreeMap<u32, &'static str>, usize) {
    use sim_core::material::MaterialCategory as C;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../sim-core/data/material");
    let mut out = std::collections::BTreeMap::new();
    let mut files = 0;
    let mut dirs: Vec<_> = std::fs::read_dir(&root)
        .expect("material dir")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    for dir in dirs {
        files += std::fs::read_dir(&dir)
            .expect("dir")
            .filter(|e| {
                e.as_ref()
                    .is_ok_and(|e| e.path().extension().is_some_and(|x| x == "ron"))
            })
            .count();
        for def in sim_core::material::load_directory(&dir).expect("parses") {
            let name = match def.category {
                C::Stone => "Stone",
                C::Wood => "Wood",
                C::Animal => "Animal",
                C::Mineral => "Mineral",
                C::Plant => "Plant",
                C::Mod(_) => "Mod",
            };
            out.insert(def.id.raw(), name);
        }
    }
    (out, files)
}

#[test]
fn wall_and_floor_looks_match_the_tile_grid() {
    use sim_core::colony::materials::{GRANITE, LIMESTONE};
    use sim_systems::runtime::jobs::space::{complete_site, dig, PACKED_EARTH};
    let mut engine = sim_test::colony_engine(1);
    for _ in 0..5 {
        engine.tick();
    }
    let (x, y) = open_area(&engine, 6, 4);
    for (kind, row) in [
        ("log_wall", 0),
        ("stone_wall", 1),
        ("door", 2),
        ("packed_floor", 3),
    ] {
        let r = submit_space(
            &mut engine,
            SpaceCommand::Line {
                kind: kind.into(),
                rect: (x as i32, (y + row) as i32, x as i32 + 4, (y + row) as i32),
            },
        );
        assert_eq!((r.accepted, r.count), (true, 5), "{kind}");
    }
    let sites: Vec<hecs::Entity> = engine
        .world
        .query::<(&sim_core::colony::ColonySite,)>()
        .iter()
        .filter(|(_, (s,))| s.kind.is_space())
        .map(|(e, _)| e)
        .collect();
    assert_eq!(sites.len(), 20);
    for site in sites {
        complete_site(&mut engine.world, &mut engine.resources, site);
    }
    // A stone wall of limestone (the other stone the colony knows), as the job layer would write it.
    let lime = (x + 5, y + 1);
    let i = engine.resources.tile_grid.idx(lime.0, lime.1);
    engine.resources.tile_grid.wall_material[i] = Some(LIMESTONE);
    // Dig three hill tiles (a rock floor of granite).
    let hills: Vec<(u32, u32)> = {
        let g = &engine.resources.tile_grid;
        (0..g.height)
            .flat_map(|ty| (0..g.width).map(move |tx| (tx, ty)))
            .filter(|t| {
                let i = g.idx(t.0, t.1);
                g.terrain_type[i] == sim_core::TerrainType::Hill && !g.is_dug(i)
            })
            .take(3)
            .collect()
    };
    assert_eq!(hills.len(), 3, "the colony map has hills");
    for t in &hills {
        assert!(dig(&mut engine.resources, *t));
    }
    let (disk, files) = disk_categories();
    // The bridge embeds every material file of the directory (data only: a new file is picked up).
    assert_eq!(
        sim_bridge::ffi::material_category::embedded_file_count(),
        files
    );
    assert!(files >= 100, "{files}");
    let space = collect_space(&engine.resources);
    assert_eq!(space.wall_looks.len(), space.walls.len());
    assert_eq!(space.floor_looks.len(), space.floors.len());
    let g = &engine.resources.tile_grid;
    let mut styles = std::collections::BTreeSet::new();
    for ((tile, code), look) in space.walls.iter().zip(&space.wall_looks) {
        let i = g.idx(tile.0, tile.1);
        assert_eq!(look.material, g.wall_material[i], "wall {tile:?}");
        let raw = g.wall_material[i].expect("a wall row has a material").raw();
        assert_eq!(sim_bridge::ffi::material_column(look.material), raw as i32);
        let expected_style = if g.is_door(i) {
            "Door"
        } else if g.wall_material[i] == Some(GRANITE) || g.wall_material[i] == Some(LIMESTONE) {
            "StoneWall"
        } else {
            "LogWall"
        };
        assert_eq!(look.style, expected_style, "wall {tile:?}");
        let expected_code = match expected_style {
            "Door" => 2,
            "StoneWall" => 1,
            _ => 0,
        };
        assert_eq!(
            *code, expected_code,
            "wall_kinds agrees with wall_styles at {tile:?}"
        );
        assert_eq!(
            look.category,
            disk.get(&raw).copied().unwrap_or(""),
            "wall {tile:?}"
        );
        assert!(
            !look.category.is_empty(),
            "every wall material is known: {tile:?}"
        );
        styles.insert(look.style.clone());
    }
    for ((tile, code), look) in space.floors.iter().zip(&space.floor_looks) {
        let i = g.idx(tile.0, tile.1);
        assert_eq!(look.material, g.floor_material[i], "floor {tile:?}");
        assert_eq!(
            sim_bridge::ffi::material_column(look.material),
            g.floor_material[i].map_or(-1, |m| m.raw() as i32)
        );
        let (style, kind) = if g.is_dug(i) {
            ("DugFloor", 1)
        } else {
            ("PackedFloor", 0)
        };
        assert_eq!(
            (look.style.as_str(), *code),
            (style, kind),
            "floor {tile:?}"
        );
        let raw = g.floor_material[i].map(|m| m.raw());
        let expected = raw.and_then(|r| disk.get(&r).copied()).unwrap_or("");
        assert_eq!(look.category, expected, "floor {tile:?}");
        styles.insert(look.style.clone());
    }
    // Each built kind is listed with its art-data style name and the expected material.
    assert_eq!(
        styles.into_iter().collect::<Vec<_>>(),
        vec!["Door", "DugFloor", "LogWall", "PackedFloor", "StoneWall"]
    );
    let looks_at = |t: (u32, u32)| {
        space
            .walls
            .iter()
            .position(|w| w.0 == t)
            .map(|p| space.wall_looks[p].clone())
    };
    let stone = looks_at((x, y + 1)).expect("stone wall row");
    assert_eq!((stone.material, stone.category), (Some(GRANITE), "Stone"));
    let lime_look = looks_at(lime).expect("limestone wall row");
    assert_eq!(
        (lime_look.style.as_str(), lime_look.category),
        ("StoneWall", "Stone")
    );
    let log = looks_at((x, y)).expect("log wall row");
    assert_eq!((log.style.as_str(), log.category), ("LogWall", "Wood"));
    let packed = space
        .floors
        .iter()
        .position(|f| f.0 == (x, y + 3))
        .map(|p| space.floor_looks[p].clone())
        .expect("packed floor row");
    // `packed_earth` has no material file: no category (the renderer falls back by style).
    assert_eq!((packed.material, packed.category), (Some(PACKED_EARTH), ""));
    let dug = space
        .floors
        .iter()
        .position(|f| f.0 == hills[0])
        .map(|p| space.floor_looks[p].clone())
        .expect("dug floor row");
    assert_eq!(
        (dug.style.as_str(), dug.material, dug.category),
        ("DugFloor", Some(GRANITE), "Stone")
    );
    // `-1` (none) is never a real material: no known id has the bit pattern 0xFFFF_FFFF.
    assert_eq!(sim_bridge::ffi::material_column(None), -1);
    let mut known: Vec<u32> = disk.keys().copied().collect();
    known.extend(
        sim_bridge::ffi::material_category::catalogue()
            .keys()
            .map(|m| m.raw()),
    );
    known.extend(
        engine
            .resources
            .material_registry
            .all_ids()
            .map(|m| m.raw()),
    );
    known.extend(
        engine
            .resources
            .colony
            .materials
            .by_id
            .keys()
            .map(|m| m.raw()),
    );
    known.push(PACKED_EARTH.raw());
    assert!(known.len() > 100);
    assert!(
        known.iter().all(|r| *r != u32::MAX),
        "a material hashes to 0xFFFFFFFF"
    );
}

#[test]
fn the_baseline_has_no_wall_or_floor_looks() {
    let engine = sim_test::baseline_engine(0);
    let space = collect_space(&engine.resources);
    assert!(space.wall_looks.is_empty() && space.floor_looks.is_empty());
    assert_eq!(space.wall_looks.len(), space.walls.len());
}
