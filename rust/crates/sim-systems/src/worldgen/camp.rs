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
        let lineage = engine
            .resources
            .rules
            .agent
            .keyed_draws
            .then(|| sim_core::components::Lineage::founder(cfg.seed, i));
        if let Some(lineage) = lineage {
            let _ = engine.world.insert_one(entity, lineage);
            // Pass 11: under exposure v2 every founder carries its hardiness,
            // keyed on its lineage (replicate 0: an identity trait).
            if engine
                .resources
                .rules
                .society
                .exposure
                .is_some_and(|rule| rule.use_hardiness)
            {
                let _ = engine.world.insert_one(
                    entity,
                    sim_core::components::Hardiness::draw(cfg.seed, lineage.key),
                );
            }
            // Pass 16: under the temperament rule (`game_v20`) every founder
            // carries its temperament, keyed on its lineage (replicate 0: an
            // identity trait); with effects its loneliness rate leans with RD
            // (a mutation of the `Social` inserted above: no archetype change).
            if let Some(rule) = engine.resources.rules.agent.temperament {
                let temperament = sim_core::components::Temperament::founder(cfg.seed, lineage.key);
                let _ = engine.world.insert_one(entity, temperament);
                if rule.effects {
                    if let Ok(mut social) = engine.world.get::<&mut Social>(entity) {
                        social.growth_rate = rule.social_rate(social.growth_rate, &temperament);
                    }
                }
            }
            if engine.resources.rules.society.family.is_some() {
                let _ = engine.world.insert_one(
                    entity,
                    sim_core::components::Sex::from_lineage(cfg.seed, lineage.key),
                );
                let _ = engine
                    .world
                    .insert_one(entity, sim_core::components::Kin::default());
            }
        }
        // Pass 4: founders start already aged, from their own aging stream.
        if let Some(rule) = engine.resources.rules.society.aging {
            let id = engine
                .world
                .get::<&sim_core::components::Agent>(entity)
                .map(|a| a.id);
            if let Ok(id) = id {
                let life = match lineage {
                    Some(lineage) => crate::runtime::agent::aging::keyed_founder_life(
                        cfg.seed,
                        lineage.key,
                        engine.resources.noise_replicate_for(0),
                        &rule,
                    ),
                    None => crate::runtime::agent::aging::founder_life(cfg.seed, id, &rule),
                };
                let _ = engine.world.insert_one(entity, life);
            }
        }
        // Pass 19 chronicle: every founder's person record (every scene).
        let id = engine
            .world
            .get::<&sim_core::components::Agent>(entity)
            .map(|a| a.id);
        if let Ok(id) = id {
            crate::runtime::chronicle_log::ensure_entity(
                &engine.world,
                &mut engine.resources,
                id,
                entity,
            );
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
/// Innermost Chebyshev distance of a wild-region centre from the camp hearth (Pass 11).
pub const REGION_MIN_DISTANCE: u32 = 22;
/// Outermost Chebyshev distance of a wild-region centre from the camp hearth.
pub const REGION_MAX_DISTANCE: u32 = 28;
/// Smallest Chebyshev distance between two wild-region centres.
pub const REGION_SEPARATION: u32 = 20;
/// Innermost ring of a region's bushes and deadwood around its centre.
pub const REGION_RING_MIN: u32 = 2;
/// Outermost ring of a region's bushes and deadwood around its centre.
pub const REGION_RING_MAX: u32 = 6;
/// Innermost ring of the Pass 12 fill's outer tier (`region_fill`).
pub const REGION_FILL_RING_MIN: u32 = 7;
/// Outermost ring of the Pass 12 fill's outer tier.
pub const REGION_FILL_RING_MAX: u32 = 9;

/// Fisher–Yates shuffle of `tiles` (already in (y, x) order) keyed on
/// `(seed, stream, slot = region)`: swap index i uses the draw at tick i.
fn keyed_shuffle(tiles: &mut [Tile], seed: u64, stream: u64, region: u32) {
    for i in (1..tiles.len()).rev() {
        let j =
            sim_core::rng::keyed_below(seed, stream, 0, i as u64, region, 0, i as u64 + 1) as usize;
        tiles.swap(i, j);
    }
}

/// Pass 11 (`game_v17`): place `cfg.wild_regions` wild regions after the camp,
/// its berries and its deadwood (which stay exactly v16's).
///
/// Region r (in order): its centre is drawn uniformly by
/// `keyed_below(seed, STREAM_REGION, 0, 0, r, 0, n)` among the walkable,
/// non-water tiles at Chebyshev distance 22–28 from the camp hearth and at
/// least 20 from every earlier centre, listed in (y, x) order; with no
/// candidate the region is skipped (counted). Then `region_bushes` bushes of
/// `berry_capacity` and `region_deadwood` deadwood spots of
/// `deadwood_capacity` at Chebyshev 2–6 from the centre, chosen with the
/// candidate filters of the camp's berries (Forest with a Plain neighbour,
/// no building, water or food source) and deadwood (Forest, then Plain; no
/// water, building or food; two tiles from every deadwood spot), each list
/// shuffled by its own keyed stream (`STREAM_REGION_BUSH` /
/// `STREAM_REGION_WOOD`, slot r); both lie farther from the camp hearth than
/// the camp's forage and wood radii (Part 3). Shortfalls are counted, never asserted;
/// terrain is never edited. Every draw uses replicate 0.
///
/// Pass 12 (`game_v18`): with `region_water_max` a centre candidate needs a
/// water tile within that Chebyshev distance (filtered before the draw). With
/// `region_fill`, right after a region's bushes are chosen and before its
/// deadwood, a short region takes more bushes in tiers, each tier's list
/// shuffled by `STREAM_REGION_FILL_BUSH` with slot `r + (tier << 16)`, never
/// on water, a building, a food source or a deadwood spot and always beyond
/// the camp's reach: tier 2 Forest in the ring 2–6 without the Plain-neighbour
/// condition, tier 3 Plain with a Forest neighbour in the ring, tier 4 either
/// of the two in the ring 7–9, and (Part 2 finding: some regions hold no
/// Forest at all within reach) tier 5 any Plain tile in the ring 2–9. After
/// the deadwood pass a short region takes
/// more deadwood in tier 2: ring 7–9, Forest then Plain
/// (`STREAM_REGION_FILL_WOOD`, slots `r + (2 << 16)` and `r + (3 << 16)`),
/// with the existing two-tile spacing. Tier counts go to the worldgen report.
pub(super) fn regions(cfg: &WorldGenConfig, r: &mut SimResources) {
    use sim_core::rng::{
        keyed_below, STREAM_REGION, STREAM_REGION_BUSH, STREAM_REGION_FILL_BUSH,
        STREAM_REGION_FILL_WOOD, STREAM_REGION_WOOD,
    };
    let Some(hearth) = r
        .building_kinds
        .iter()
        .find(|(_, kind)| **kind == BuildingKind::Hearth)
        .map(|(tile, _)| *tile)
    else {
        return;
    };
    for spot in r.wood_source_max.keys().copied().collect::<Vec<_>>() {
        r.wood_spot_area.insert(spot, 0);
    }
    let water: Vec<Tile> = r.water_tiles.keys().copied().collect();
    // Part 3: region bushes and deadwood lie beyond the camp's forage and wood
    // radii, so the original village's economy is v16's (a region bush at 16
    // or deadwood at 16-18 tiles would be the camp's too).
    let camp_reach = r
        .rules
        .agent
        .forage
        .map_or(0, |f| f.forage_radius)
        .max(r.rules.society.firewood.map_or(0, |f| f.wood_radius));
    let beyond_camp = |p: Tile| distance(p, hearth) > camp_reach;
    r.worldgen_report.regions_requested = cfg.wild_regions;
    let mut centres: Vec<Tile> = Vec::new();
    for region in 0..cfg.wild_regions {
        let candidates: Vec<Tile> = tiles(r)
            .filter(|&p| {
                (REGION_MIN_DISTANCE..=REGION_MAX_DISTANCE).contains(&distance(p, hearth))
                    && walkable(r, p)
                    && !r.water_tiles.contains_key(&p)
                    && centres.iter().all(|&c| distance(p, c) >= REGION_SEPARATION)
                    && cfg
                        .region_water_max
                        .is_none_or(|max| water.iter().any(|&q| distance(p, q) <= max))
            })
            .collect();
        if candidates.is_empty() {
            continue;
        }
        let centre = candidates[keyed_below(
            cfg.seed,
            STREAM_REGION,
            0,
            0,
            region,
            0,
            candidates.len() as u64,
        ) as usize];
        centres.push(centre);
        let ring = |p: Tile| {
            (REGION_RING_MIN..=REGION_RING_MAX).contains(&distance(p, centre)) && beyond_camp(p)
        };
        // Bushes: the camp berries' candidate filter inside the ring.
        let mut bushes: Vec<Tile> = tiles(r)
            .filter(|&p| {
                ring(p)
                    && terrain(r, p) == TerrainType::Forest
                    && !r.building_kinds.contains_key(&p)
                    && !r.water_tiles.contains_key(&p)
                    && !r.food_source_max.contains_key(&p)
                    && r.tile_grid
                        .neighbors_4(p.0, p.1)
                        .iter()
                        .any(|&q| terrain(r, q) == TerrainType::Plain)
            })
            .collect();
        keyed_shuffle(&mut bushes, cfg.seed, STREAM_REGION_BUSH, region);
        bushes.truncate(cfg.region_bushes as usize);
        // Pass 12: fill a short region from looser tiers (none in v17).
        if cfg.region_fill {
            let far = |p: Tile| {
                (REGION_FILL_RING_MIN..=REGION_FILL_RING_MAX).contains(&distance(p, centre))
                    && beyond_camp(p)
            };
            let forest_edge = |p: Tile| {
                terrain(r, p) == TerrainType::Plain
                    && r.tile_grid
                        .neighbors_4(p.0, p.1)
                        .iter()
                        .any(|&q| terrain(r, q) == TerrainType::Forest)
            };
            let mut added = [0u32; 4];
            for (tier, slot) in [(2u32, 0usize), (3, 1), (4, 2), (5, 3)] {
                if bushes.len() >= cfg.region_bushes as usize {
                    break;
                }
                let mut extra: Vec<Tile> = tiles(r)
                    .filter(|&p| {
                        let place = match tier {
                            2 => ring(p) && terrain(r, p) == TerrainType::Forest,
                            3 => ring(p) && forest_edge(p),
                            4 => far(p) && (terrain(r, p) == TerrainType::Forest || forest_edge(p)),
                            _ => (ring(p) || far(p)) && terrain(r, p) == TerrainType::Plain,
                        };
                        place
                            && !r.water_tiles.contains_key(&p)
                            && !r.building_kinds.contains_key(&p)
                            && !r.food_source_max.contains_key(&p)
                            && !r.wood_source_max.contains_key(&p)
                            && !bushes.contains(&p)
                    })
                    .collect();
                keyed_shuffle(
                    &mut extra,
                    cfg.seed,
                    STREAM_REGION_FILL_BUSH,
                    region + (tier << 16),
                );
                let take = (cfg.region_bushes as usize - bushes.len()).min(extra.len());
                bushes.extend_from_slice(&extra[..take]);
                added[slot] = take as u32;
            }
            for (total, n) in r.worldgen_report.region_fill_bushes.iter_mut().zip(added) {
                *total += n;
            }
        }
        bushes.sort_by_key(|&(x, y)| (y, x));
        for &(x, y) in &bushes {
            r.set_food_tile(x, y, cfg.berry_capacity);
            r.set_food_source_max(x, y, cfg.berry_capacity);
        }
        // Deadwood: the camp deadwood's candidate filter inside the ring.
        let mut chosen: Vec<Tile> = Vec::new();
        for kind in [TerrainType::Forest, TerrainType::Plain] {
            let mut candidates: Vec<Tile> = tiles(r)
                .filter(|&p| {
                    ring(p)
                        && terrain(r, p) == kind
                        && !r.water_tiles.contains_key(&p)
                        && !r.building_kinds.contains_key(&p)
                        && !r.food_source_max.contains_key(&p)
                        && !r.food_tiles.contains_key(&p)
                })
                .collect();
            keyed_shuffle(
                &mut candidates,
                cfg.seed,
                STREAM_REGION_WOOD,
                region
                    + if kind == TerrainType::Plain {
                        1 << 16
                    } else {
                        0
                    },
            );
            for p in candidates {
                if chosen.len() >= cfg.region_deadwood as usize {
                    break;
                }
                if chosen.iter().all(|&q| distance(p, q) >= 2)
                    && r.wood_source_max.keys().all(|&q| distance(p, q) >= 2)
                {
                    chosen.push(p);
                }
            }
        }
        // Pass 12: fill a short region's deadwood from the ring 7–9 (none in v17).
        if cfg.region_fill && chosen.len() < cfg.region_deadwood as usize {
            let before = chosen.len();
            for (kind, tier) in [(TerrainType::Forest, 2u32), (TerrainType::Plain, 3)] {
                let mut candidates: Vec<Tile> = tiles(r)
                    .filter(|&p| {
                        (REGION_FILL_RING_MIN..=REGION_FILL_RING_MAX).contains(&distance(p, centre))
                            && beyond_camp(p)
                            && terrain(r, p) == kind
                            && !r.water_tiles.contains_key(&p)
                            && !r.building_kinds.contains_key(&p)
                            && !r.food_source_max.contains_key(&p)
                            && !r.food_tiles.contains_key(&p)
                    })
                    .collect();
                keyed_shuffle(
                    &mut candidates,
                    cfg.seed,
                    STREAM_REGION_FILL_WOOD,
                    region + (tier << 16),
                );
                for p in candidates {
                    if chosen.len() >= cfg.region_deadwood as usize {
                        break;
                    }
                    if chosen.iter().all(|&q| distance(p, q) >= 2)
                        && r.wood_source_max.keys().all(|&q| distance(p, q) >= 2)
                    {
                        chosen.push(p);
                    }
                }
            }
            r.worldgen_report.region_fill_deadwood += (chosen.len() - before) as u32;
        }
        for &p in &chosen {
            r.wood_tiles.insert(p, cfg.deadwood_capacity);
            r.wood_source_max.insert(p, cfg.deadwood_capacity);
            r.wood_spot_area.insert(p, region as u8 + 1);
        }
        r.worldgen_report.regions_placed += 1;
        r.worldgen_report.region_bushes_requested += cfg.region_bushes;
        r.worldgen_report.region_bushes_placed += bushes.len() as u32;
        r.worldgen_report.region_deadwood_requested += cfg.region_deadwood;
        r.worldgen_report.region_deadwood_placed += chosen.len() as u32;
        let water_distance = water
            .iter()
            .map(|&q| distance(centre, q))
            .min()
            .unwrap_or(u32::MAX);
        r.wild_regions.push(sim_engine::WildRegion {
            centre,
            bushes: bushes.len() as u32,
            deadwood: chosen.len() as u32,
            water_distance,
        });
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
