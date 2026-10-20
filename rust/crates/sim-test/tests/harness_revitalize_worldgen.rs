//! Seeded-world contracts independent of the Godot runtime, on the live
//! generator (`WorldGenConfig::baseline` under `WorldRules::baseline`).
use sim_core::{
    components::{Agent, BuildingKind, Position},
    rules::WorldRules,
    TerrainType,
};
use sim_engine::SimEngine;
use sim_systems::worldgen::{build_world_with, WorldGenConfig};

fn build(seed: u64) -> SimEngine {
    build_world_with(&WorldGenConfig::baseline(seed), WorldRules::baseline(seed))
}

fn digest(engine: &SimEngine) -> Vec<u64> {
    let r = &engine.resources;
    let mut bytes = vec![u64::from(r.tile_grid.width), u64::from(r.tile_grid.height)];
    bytes.extend(r.tile_grid.terrain_type.iter().map(|&t| t as u64));
    bytes.extend(r.tile_grid.elevation.iter().map(|&e| u64::from(e)));
    for (&(x, y), &kind) in &r.building_kinds {
        bytes.extend([u64::from(x), u64::from(y), u64::from(kind.code())]);
    }
    let mut agents: Vec<_> = engine
        .world
        .query::<(&Agent, &Position)>()
        .iter()
        .map(|(_, (a, p))| (a.id, p.x, p.y))
        .collect();
    agents.sort();
    for (id, x, y) in agents {
        bytes.extend([id, u64::from(x), u64::from(y)]);
    }
    for map in [
        &r.food_tiles,
        &r.water_tiles,
        &r.sleep_tiles,
        &r.food_source_max,
        &r.water_source_max,
        &r.sleep_source_max,
    ] {
        let mut entries: Vec<_> = map.iter().map(|(&p, &v)| (p, v)).collect();
        entries.sort();
        bytes.push(entries.len() as u64);
        for ((x, y), v) in entries {
            bytes.extend([u64::from(x), u64::from(y), u64::from(v)]);
        }
    }
    bytes
}
fn distance(a: (u32, u32), b: (u32, u32)) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}
#[test]
fn harness_territory_worldgen_deterministic() {
    let a = build(1);
    let b = build(1);
    let c = build(2);
    assert_eq!(digest(&a), digest(&b));
    assert_ne!(digest(&a), digest(&c));
}
#[test]
fn harness_territory_worldgen_invariants() {
    for seed in (1..=8).chain([247, 338, 345, 383, 426, 728, 759, 773]) {
        let cfg = WorldGenConfig::baseline(seed);
        let mut engine = build(seed);
        let r = &engine.resources;
        let g = &r.tile_grid;
        for row in g.terrain_type.chunks(cfg.width as usize) {
            assert!(
                row.contains(&TerrainType::River),
                "seed {seed}: broken river"
            );
        }
        let nonriver = g
            .terrain_type
            .iter()
            .filter(|&&t| t != TerrainType::River)
            .count() as f64;
        for (kind, fraction) in [
            (TerrainType::Forest, cfg.forest_coverage),
            (TerrainType::Hill, cfg.hill_coverage),
            (TerrainType::Mountain, cfg.mountain_coverage),
        ] {
            let count = g.terrain_type.iter().filter(|&&t| t == kind).count() as f64;
            assert!(
                (count - nonriver * fraction).abs() <= 1.0,
                "seed {seed}: coverage {kind:?}"
            );
        }
        let hearth = r
            .building_kinds
            .iter()
            .find_map(|(&p, &k)| (k == BuildingKind::Hearth).then_some(p))
            .expect("Hearth");
        assert!(
            r.water_tiles.keys().any(|&p| distance(p, hearth) <= 4),
            "seed {seed}: distant shore"
        );
        assert!(r.water_source_max.is_empty());
        for (&(x, y), &amount) in &r.water_tiles {
            assert_eq!(amount, sim_engine::RESOURCE_SOURCE_INFINITE);
            assert!(g
                .neighbors_4(x, y)
                .iter()
                .any(|&(nx, ny)| g.terrain_type[g.idx(nx, ny)] == TerrainType::River));
        }
        for (kind, count) in [
            (BuildingKind::Hearth, 1),
            (BuildingKind::Shelter, 3),
            (BuildingKind::Stockpile, 1),
        ] {
            assert_eq!(
                r.building_kinds.values().filter(|&&k| k == kind).count(),
                count
            );
        }
        let mut occupied: Vec<_> = r.building_kinds.keys().copied().collect();
        occupied.extend(r.food_tiles.keys().copied());
        occupied.extend(
            engine
                .world
                .query::<&Position>()
                .iter()
                .map(|(_, p)| (p.x, p.y)),
        );
        for (x, y) in occupied {
            assert!(
                !matches!(
                    g.terrain_type[g.idx(x, y)],
                    TerrainType::River | TerrainType::Mountain
                ),
                "seed {seed}: blocked spawn"
            );
        }
        // Camp bushes lie in the 6..=16 ring around the hearth; wild-region
        // bushes (Pass 11) lie beyond the camp's forage and wood reach (18 on
        // the baseline rules), at most `region_bushes` per region.
        let rules = &r.rules;
        let camp_reach = rules
            .agent
            .forage
            .map_or(0, |f| f.forage_radius)
            .max(rules.society.firewood.map_or(0, |f| f.wood_radius));
        assert_eq!(camp_reach, 18);
        let camp: Vec<_> = r
            .food_tiles
            .keys()
            .filter(|&&p| distance(p, hearth) <= 16)
            .collect();
        assert_eq!(
            camp.len(),
            cfg.berry_bushes as usize,
            "seed {seed}: camp bushes"
        );
        let region = r.food_tiles.len() - camp.len();
        assert!(
            region <= (cfg.wild_regions * cfg.region_bushes) as usize,
            "seed {seed}: {region} region bushes"
        );
        for (&p, &amount) in &r.food_tiles {
            let d = distance(p, hearth);
            if d <= 16 {
                assert!(d >= 6, "seed {seed}: camp bush at {d}");
                assert_eq!(g.terrain_type[g.idx(p.0, p.1)], TerrainType::Forest);
                assert!(g
                    .neighbors_4(p.0, p.1)
                    .iter()
                    .any(|&(x, y)| g.terrain_type[g.idx(x, y)] == TerrainType::Plain));
            } else {
                assert!(d > camp_reach, "seed {seed}: region bush at {d}");
            }
            assert_eq!(amount, cfg.berry_capacity);
            assert_eq!(r.food_source_max.get(&p), Some(&cfg.berry_capacity));
            assert!(!r.building_kinds.contains_key(&p) && !r.water_tiles.contains_key(&p));
        }
        for (&p, &kind) in &r.building_kinds {
            if kind == BuildingKind::Shelter {
                assert_eq!(
                    r.sleep_tiles.get(&p),
                    Some(&sim_engine::RESOURCE_SOURCE_INFINITE)
                );
            }
        }
        engine.tick();
        engine.tick();
        assert!(
            engine
                .resources
                .settlements
                .values()
                .any(|s| s.member_agents.len() == cfg.starting_agents as usize),
            "seed {seed}: no full camp settlement at tick 2"
        );
    }
}

#[test]
fn harness_building_snapshot_matches_registry() {
    use sim_bridge::ffi::world_node::collect_building_snapshot;
    let mut engine = sim_test::baseline_engine(1);
    assert!(engine.resources.building_registry.is_empty());
    assert!(collect_building_snapshot(&engine.resources).is_empty());
    engine.tick();
    engine.tick();
    let rows = collect_building_snapshot(&engine.resources);
    assert_eq!(rows.len(), engine.resources.building_registry.len());
    assert_eq!(rows.len(), 5);
    assert!(rows.windows(2).all(|pair| pair[0].id < pair[1].id));
    for row in rows {
        let position = (row.x, row.y);
        assert_eq!(
            engine.resources.building_registry.get(&row.id),
            Some(&position)
        );
        let kind = engine
            .resources
            .building_kinds
            .get(&position)
            .expect("generated building has kind");
        assert_eq!(row.kind, kind.code());
    }
}
