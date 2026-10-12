//! Deterministic riverside camp, residents, and clustered renewable food.
use super::{
    noise::{h64, splitmix64},
    WorldGenConfig,
};
use crate::runtime::agent::MovementRng;
use sim_core::{
    components::{
        AgentState, BodyHealth, BuildingKind, Hunger, Inventory, Memory, Sleep, Social, Thirst,
    },
    TerrainType,
};
use sim_engine::{BuildingPlacedEvent, SimEngine, SimResources, RESOURCE_SOURCE_INFINITE};
use std::collections::BTreeSet;
type Tile = (u32, u32);
/// Salt of the deadwood placement stream (Pass 5); distinct from every other
/// worldgen, birth and aging salt.
pub const DEADWOOD_SALT: u64 = 0xDEAD_0D00_F1E1_0005;
pub(super) fn distance(a: Tile, b: Tile) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}
fn terrain(r: &SimResources, p: Tile) -> TerrainType {
    r.tile_grid.terrain_type[r.tile_grid.idx(p.0, p.1)]
}
fn walkable(r: &SimResources, p: Tile) -> bool {
    !matches!(terrain(r, p), TerrainType::River | TerrainType::Mountain)
}
fn tiles(r: &SimResources) -> impl Iterator<Item = Tile> + '_ {
    (0..r.tile_grid.height).flat_map(move |y| (0..r.tile_grid.width).map(move |x| (x, y)))
}
pub(super) fn shore_water(r: &mut SimResources) {
    let shores: Vec<_> = tiles(r)
        .filter(|&p| {
            walkable(r, p)
                && r.tile_grid
                    .neighbors_4(p.0, p.1)
                    .iter()
                    .any(|&n| terrain(r, n) == TerrainType::River)
        })
        .collect();
    for (x, y) in shores {
        r.set_water_tile(x, y, RESOURCE_SOURCE_INFINITE);
    }
}
fn choose_camp(r: &SimResources) -> Tile {
    let rivers: Vec<_> = tiles(r)
        .filter(|&p| terrain(r, p) == TerrainType::River)
        .collect();
    let center = (r.tile_grid.width / 2, r.tile_grid.height / 2);
    let mut ranked: Vec<_> = tiles(r)
        .filter(|&(x, y)| {
            x >= 6 && y >= 6 && x + 6 < r.tile_grid.width && y + 6 < r.tile_grid.height
        })
        .filter(|&p| terrain(r, p) == TerrainType::Plain)
        .filter(|&(x, y)| {
            (y - 3..=y + 3)
                .flat_map(|ny| (x - 3..=x + 3).map(move |nx| (nx, ny)))
                .filter(|&p| terrain(r, p) == TerrainType::Plain)
                .count()
                >= 6
        })
        .map(|p| {
            let river_distance = rivers
                .iter()
                .map(|&q| distance(p, q))
                .min()
                .unwrap_or(u32::MAX);
            let score =
                -(f64::from(river_distance.abs_diff(3))) - 0.05 * f64::from(distance(p, center));
            (score, p)
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.0.total_cmp(&a.0)
            .then((a.1 .1, a.1 .0).cmp(&(b.1 .1, b.1 .0)))
    });
    ranked
        .first()
        .map(|&(_, p)| p)
        .expect("world must contain a valid inland Plain camp")
}
pub(super) fn populate(cfg: &WorldGenConfig, engine: &mut SimEngine) {
    let c = choose_camp(&engine.resources);
    for (dx, dy, kind, radius) in [
        (0, 0, BuildingKind::Hearth, 6),
        (-2, -1, BuildingKind::Shelter, 2),
        (2, -1, BuildingKind::Shelter, 2),
        (0, 2, BuildingKind::Shelter, 2),
        (2, 2, BuildingKind::Stockpile, 0),
    ] {
        let target = ((i64::from(c.0) + dx) as u32, (i64::from(c.1) + dy) as u32);
        let r = &mut engine.resources;
        let p = tiles(r)
            .filter(|&p| walkable(r, p) && !r.building_kinds.contains_key(&p))
            .min_by_key(|&p| (distance(p, target), p.1, p.0))
            .expect("camp requires five buildable tiles");
        r.building_kinds.insert(p, kind);
        r.building_event_queue.push_back(BuildingPlacedEvent {
            position: p,
            radius,
        });
        if kind == BuildingKind::Shelter {
            r.set_sleep_tile(p.0, p.1, RESOURCE_SOURCE_INFINITE);
        }
    }
    let mut positions: Vec<_> = tiles(&engine.resources)
        .filter(|&p| {
            (1..=3).contains(&distance(p, c))
                && walkable(&engine.resources, p)
                && !engine.resources.building_kinds.contains_key(&p)
        })
        .collect();
    positions.sort_by_key(|&p| (distance(p, c), p.1, p.0));
    assert!(
        !positions.is_empty(),
        "camp requires at least one resident tile"
    );
    for i in 0..cfg.starting_agents {
        let p = positions[i as usize % positions.len()];
        let entity = engine.spawn_agent(p.0, p.1);
        let seed = splitmix64(cfg.seed ^ 0xA9E7_0000_0000_0001 ^ u64::from(i));
        let mut needs = MovementRng::new(seed ^ 0x5EED_5A66_E0DE_0001);
        // Hunger's legacy component is f32; all worldgen math remains f64.
        let hunger = (needs.next_u64() % 46) as f64;
        let thirst = (needs.next_u64() % 46) as f64;
        let sleep = (needs.next_u64() % 46) as f64;
        engine
            .world
            .insert(
                entity,
                (
                    MovementRng::new(seed),
                    AgentState::Idle,
                    Hunger::new(hunger as f32, 0.05),
                    Thirst::new(thirst, 0.08),
                    Sleep::new(sleep, 0.03),
                    Social::new(0.0, 0.04),
                    Memory::new(),
                    BodyHealth::new(),
                    Inventory::default(),
                ),
            )
            .expect("newly spawned worldgen resident must exist");
        // Pass 9: under keyed draws a founder's lineage key is its spawn index,
        // and its age and lifespan are keyed on it (born before tick 0: the
        // replicate in force at tick 0).
        let lineage = engine.resources.rules.agent.keyed_draws.then(|| sim_core::components::Lineage::founder(cfg.seed, i));
        if let Some(lineage) = lineage {
            let _ = engine.world.insert_one(entity, lineage);
        }
        // Pass 4: founders start already aged, from their own aging stream.
        if let Some(rule) = engine.resources.rules.society.aging {
            let id = engine.world.get::<&sim_core::components::Agent>(entity).map(|a| a.id);
            if let Ok(id) = id {
                let life = match lineage {
                    Some(lineage) => crate::runtime::agent::aging::keyed_founder_life(
                        cfg.seed, lineage.key, engine.resources.noise_replicate_for(0), &rule,
                    ),
                    None => crate::runtime::agent::aging::founder_life(cfg.seed, id, &rule),
                };
                let _ = engine.world.insert_one(entity, life);
            }
        }
    }
    berries(cfg, &mut engine.resources, c);
}
pub(super) fn berry_candidates(r: &SimResources, c: Tile) -> Vec<Tile> {
    tiles(r)
        .filter(|&p| {
            (6..=16).contains(&distance(p, c))
                && terrain(r, p) == TerrainType::Forest
                && !r.building_kinds.contains_key(&p)
                && !r.water_tiles.contains_key(&p)
                && r.tile_grid
                    .neighbors_4(p.0, p.1)
                    .iter()
                    .any(|&q| terrain(r, q) == TerrainType::Plain)
        })
        .collect()
}
pub(super) fn cluster_centers(cfg: &WorldGenConfig, candidates: &[Tile]) -> Vec<Tile> {
    let mut ranked = candidates.to_vec();
    ranked.sort_by_key(|&(x, y)| (h64(cfg.seed, 8, x, y), y, x));
    let mut centers = Vec::new();
    for p in ranked {
        if centers.iter().all(|&q| distance(p, q) >= 6) {
            centers.push(p);
        }
        if centers.len() == 3 {
            break;
        }
    }
    centers
}
fn berries(cfg: &WorldGenConfig, r: &mut SimResources, c: Tile) {
    super::terrain::repair_berry_edges(cfg, r, c);
    let candidates = berry_candidates(r, c);
    let centers = cluster_centers(cfg, &candidates);
    assert_eq!(
        centers.len(),
        3,
        "worldgen requires three separated berry clusters"
    );
    let mut chosen = BTreeSet::new();
    for (cluster, &center) in centers.iter().enumerate() {
        let count =
            cfg.berry_bushes as usize / 3 + usize::from(cluster < cfg.berry_bushes as usize % 3);
        let mut ranked: Vec<_> = candidates
            .iter()
            .copied()
            .filter(|p| !chosen.contains(p))
            .collect();
        ranked.sort_by_key(|&p| (distance(p, center), p.1, p.0));
        chosen.extend(ranked.into_iter().take(count));
    }
    assert_eq!(
        chosen.len(),
        cfg.berry_bushes as usize,
        "worldgen cannot silently omit requested berries"
    );
    for (x, y) in chosen {
        r.set_food_tile(x, y, cfg.berry_capacity);
        r.set_food_source_max(x, y, cfg.berry_capacity);
    }
}
/// Place `cfg.deadwood_spots` deadwood spots at full capacity (Pass 5).
///
/// Candidates are Forest tiles at Chebyshev `[min, max]` from the hearth,
/// then Plain tiles once Forest runs out, never water, a building or a bush.
/// Each list is sorted by `(y, x)` and Fisher–Yates shuffled with the
/// dedicated `DEADWOOD_SALT` stream; a candidate is taken when it lies at
/// least two tiles from every spot already chosen.
pub(super) fn deadwood(cfg: &WorldGenConfig, r: &mut SimResources) {
    let Some(hearth) = r
        .building_kinds
        .iter()
        .find(|(_, kind)| **kind == BuildingKind::Hearth)
        .map(|(tile, _)| *tile)
    else {
        return;
    };
    let mut rng = MovementRng::new(cfg.seed ^ DEADWOOD_SALT);
    let band = cfg.deadwood_min_radius..=cfg.deadwood_max_radius;
    let mut chosen: Vec<Tile> = Vec::new();
    for kind in [TerrainType::Forest, TerrainType::Plain] {
        let mut candidates: Vec<Tile> = tiles(r)
            .filter(|&p| {
                band.contains(&distance(p, hearth))
                    && terrain(r, p) == kind
                    && !r.water_tiles.contains_key(&p)
                    && !r.building_kinds.contains_key(&p)
                    && !r.food_source_max.contains_key(&p)
                    && !r.food_tiles.contains_key(&p)
            })
            .collect();
        candidates.sort_by_key(|&(x, y)| (y, x));
        for i in (1..candidates.len()).rev() {
            let j = (rng.next_u64() % (i as u64 + 1)) as usize;
            candidates.swap(i, j);
        }
        for p in candidates {
            if chosen.len() >= cfg.deadwood_spots as usize {
                break;
            }
            if chosen.iter().all(|&q| distance(p, q) >= 2) {
                chosen.push(p);
            }
        }
    }
    // Pass 10: counted for measurement only (fewer than requested when the
    // radius band runs out of tiles two apart).
    r.worldgen_report.deadwood_spots_requested = cfg.deadwood_spots;
    r.worldgen_report.deadwood_spots_placed = chosen.len() as u32;
    for p in chosen {
        r.wood_tiles.insert(p, cfg.deadwood_capacity);
        r.wood_source_max.insert(p, cfg.deadwood_capacity);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chebyshev_distance_matches_rings() {
        assert_eq!(distance((10, 10), (7, 12)), 3);
    }
}
