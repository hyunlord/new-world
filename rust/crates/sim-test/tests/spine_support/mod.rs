//! Production scene: only starting knowledge is a fixture; materials come from jobs.
#![allow(dead_code)]
use sim_core::colony::building_site::{BuildingStatus, Tile};
use sim_core::colony::knowledge::{self, Knowledge, KnowledgeEntry, Source};
use sim_core::colony::{Command, ItemKind};
use sim_core::components::Agent;
use sim_engine::SimEngine;
use sim_systems::commands::submit;
use sim_systems::runtime::jobs::{invariants, items};

pub mod ui_fixture;
mod ui_motion;

pub const DEADLINE: u64 = 48 * 1_440;
pub fn command(engine: &mut SimEngine, value: Command) {
    let result = submit(engine, value.clone());
    assert!(result.accepted, "{value:?}: {result:?}");
}

pub fn scene() -> SimEngine {
    let mut engine = sim_test::colony_engine(0);
    command(
        &mut engine,
        Command::SetSteward {
            on: false,
            scope: Default::default(),
        },
    );
    let entries: Vec<_> = knowledge::defs()
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.status == knowledge::ContentStatus::Connected)
        .map(|(index, _)| {
            KnowledgeEntry::new(
                u8::try_from(index).expect("validated knowledge count"),
                Source::Seeded,
                1_000,
                0,
            )
        })
        .collect();
    let entities: Vec<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    for entity in entities {
        engine
            .world
            .insert_one(entity, Knowledge::from_entries(entries.clone()))
            .expect("initial fixture knowledge");
    }
    engine
}

pub fn until(engine: &mut SimEngine, label: &str, ready: impl Fn(&SimEngine) -> bool) {
    while engine.current_tick() < DEADLINE && !ready(engine) {
        engine.tick();
        if engine.current_tick().is_multiple_of(1_440) {
            println!("SPINE_WAIT {label} tick={}", engine.current_tick());
        }
        invariants::check(&engine.world, &engine.resources)
            .expect("physical ledger and job invariants");
    }
    assert!(
        ready(engine),
        "{label} not reached at {}: sites={:?}",
        engine.current_tick(),
        engine.resources.colony.buildings.sites
    );
}

pub fn camp(engine: &SimEngine) -> Tile {
    sim_systems::worldgen::colony::hearth(&engine.resources).expect("initial colony hearth")
}

pub fn harvest_logs(engine: &mut SimEngine) {
    let centre = camp(engine);
    let mut trees: Vec<_> = engine
        .resources
        .colony
        .trees
        .keys()
        .copied()
        .filter(|&tile| sim_systems::commands::camp_reaches(&engine.resources, tile))
        .collect();
    trees.sort_by_key(|&tile| (items::chebyshev(tile, centre), tile.1, tile.0));
    trees.truncate(32);
    assert!(!trees.is_empty(), "seed0 has reachable raw trees");
    command(engine, Command::DesignateChopTiles { tiles: trees });
    until(engine, "logs harvested by workers", |engine| {
        engine.resources.colony.ground_total(ItemKind::Log) >= 50
    });
    assert!(engine.resources.colony.tallies.trees_felled > 0);
}

pub fn anchor(engine: &SimEngine, id: &str) -> Tile {
    let centre = camp(engine);
    let mut tiles: Vec<_> = (1..engine.resources.tile_grid.height - 1)
        .flat_map(|y| (1..engine.resources.tile_grid.width - 1).map(move |x| (x, y)))
        .filter(|&tile| items::chebyshev(tile, centre) <= 24)
        .collect();
    tiles.sort_by_key(|&tile| (items::chebyshev(tile, centre), tile.1, tile.0));
    tiles
        .into_iter()
        .find(|&tile| {
            sim_systems::runtime::building_templates::commands::preview(
                &engine.world,
                &engine.resources,
                id,
                tile,
                0,
                None,
            )
            .is_ok()
        })
        .expect("seed0 reachable cleared house footprint")
}

pub fn complete(engine: &SimEngine, id: u32) -> bool {
    engine
        .resources
        .colony
        .buildings
        .sites
        .get(&id)
        .is_some_and(|site| site.status == BuildingStatus::Complete)
}

pub fn checkpoint(engine: &mut SimEngine, label: &str, selected_site: Option<u32>) {
    invariants::check(&engine.world, &engine.resources).expect("capture physical invariants");
    if let Some(directory) = std::env::var_os("WORLDSIM_SPINE_SCENE_OUT") {
        let directory = std::path::PathBuf::from(directory);
        assert!(
            directory.is_absolute() && directory.is_dir(),
            "existing external capture directory"
        );
        let ui = ui_fixture::manifest(engine, label);
        let bytes = sim_systems::save::save(engine).expect("production scene save");
        let sites: Vec<_> = engine.resources.colony.buildings.sites.values().map(|site| serde_json::json!({
            "id": site.id, "template": site.template, "anchor": [site.anchor.0, site.anchor.1],
            "footprint": [site.footprint.0, site.footprint.1], "completion_entry": site.completion_entry,
            "source_record": site.source_record
        })).collect();
        let manifest = serde_json::to_vec_pretty(
            &serde_json::json!({"tick": engine.current_tick(), "selected_site": selected_site, "sites": sites, "ui": ui}),
        )
        .expect("capture manifest bytes");
        use std::io::Write;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(format!("{label}.wsave")))
            .expect("fresh evidence path")
            .write_all(&bytes)
            .expect("capture save bytes");
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(format!("{label}.json")))
            .expect("fresh manifest path")
            .write_all(&manifest)
            .expect("capture manifest");
    }
    println!("SPINE_STEP {label} tick={}", engine.current_tick());
}

/// Cancel only the retained plan's remaining physical work through the existing area order.
pub fn cancel_remaining(engine: &mut SimEngine, id: u32) {
    let site = engine.resources.colony.buildings.sites[&id].clone();
    command(
        engine,
        Command::CancelArea {
            x0: site.anchor.0,
            y0: site.anchor.1,
            x1: site.anchor.0 + site.footprint.0 - 1,
            y1: site.anchor.1 + site.footprint.1 - 1,
        },
    );
}
