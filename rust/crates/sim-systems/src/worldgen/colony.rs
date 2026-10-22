//! The M1 colony world's extra worldgen steps (after the baseline steps):
//! area-scaled bushes and deadwood beyond the camp, the founder adult check,
//! the trees and (M2, under `ColonyRule::economy`) the material table and the
//! resource nodes. Called by `build_world_with` only when `cfg.colony` is set.
//!
//! Every step draws from its own keyed stream (`STREAM_COLONY_BUSH`,
//! `STREAM_COLONY_WOOD`, `STREAM_COLONY_FOUNDER_FIX`, `STREAM_TREE_LOGS`),
//! so the camp, its berries, deadwood and wild regions are exactly what the
//! baseline steps placed for this config.
use super::camp::distance;
use super::{ColonyGen, WorldGenConfig};
use sim_core::colony::materials::{FLINT, GRANITE, LIMESTONE};
use sim_core::colony::{ColonyMaterials, NodeKind, ResourceNode, Tree};
use sim_core::components::{Agent, BuildingKind, Child, Elder, Life};
use sim_core::rng::{
    keyed_below, STREAM_BOULDER, STREAM_COLONY_BUSH, STREAM_COLONY_FOUNDER_FIX, STREAM_COLONY_WOOD,
    STREAM_OUTCROP, STREAM_TREE_LOGS,
};
use sim_core::TerrainType;
use sim_engine::{SimEngine, SimResources};

type Tile = (u32, u32);

/// Innermost Chebyshev distance of a bush beyond the camp ring (the camp's
/// berries lie at 6–16).
pub const OUTER_BUSH_MIN: u32 = 17;
/// Innermost Chebyshev distance of a deadwood spot beyond the camp band (5–18).
pub const OUTER_WOOD_MIN: u32 = 19;

fn terrain(r: &SimResources, p: Tile) -> TerrainType {
    r.tile_grid.terrain_type[r.tile_grid.idx(p.0, p.1)]
}

fn tiles(r: &SimResources) -> Vec<Tile> {
    (0..r.tile_grid.height)
        .flat_map(|y| (0..r.tile_grid.width).map(move |x| (x, y)))
        .collect()
}

/// The camp hearth (the first Hearth in `building_kinds` key order).
pub fn hearth(r: &SimResources) -> Option<Tile> {
    r.building_kinds
        .iter()
        .find(|(_, kind)| **kind == BuildingKind::Hearth)
        .map(|(tile, _)| *tile)
}

/// Fisher–Yates over `list` (already in (y, x) order) keyed on `(seed, stream, slot)`.
fn shuffle(list: &mut [Tile], seed: u64, stream: u64, slot: u32) {
    for i in (1..list.len()).rev() {
        let j = keyed_below(seed, stream, 0, i as u64, slot, 0, i as u64 + 1) as usize;
        list.swap(i, j);
    }
}

/// Run the colony steps (see the module header).
pub(super) fn generate(cfg: &WorldGenConfig, gen: &ColonyGen, engine: &mut SimEngine) {
    let Some(camp) = hearth(&engine.resources) else {
        return;
    };
    outer_bushes(cfg, gen, &mut engine.resources, camp);
    outer_deadwood(cfg, gen, &mut engine.resources, camp);
    let fixed = ensure_adults(cfg, gen, engine);
    engine.resources.colony.tallies.founder_fixes = fixed;
    trees(cfg, gen, &mut engine.resources, camp);
    nodes(cfg, gen, &mut engine.resources, camp);
    founders_on_open_ground(engine, camp);
}

/// M3 (ADR 0021, DECISIONS D9): the camp's resident tiles allow hill tiles,
/// which the colony cannot walk on; every founder standing on a tile it could
/// not leave (or outside the hearth's region) moves to the nearest tile of the
/// hearth's region, by `(Chebyshev distance, y, x)`, founders in `AgentId` order.
fn founders_on_open_ground(engine: &mut SimEngine, camp: Tile) {
    if engine.resources.rules.colony.is_none() {
        return;
    }
    // The colony's nav map exists from tick 0 (the probe's resolvers read it).
    crate::runtime::nav::ensure(&mut engine.resources);
    let nav = &engine.resources.nav;
    let home = nav.region_of(camp);
    if home == 0 {
        return;
    }
    let mut people: Vec<(u64, hecs::Entity, Tile)> = engine
        .world
        .query::<(
            &sim_core::components::Agent,
            &sim_core::components::Position,
        )>()
        .iter()
        .map(|(e, (a, p))| (a.id, e, (p.x, p.y)))
        .filter(|(_, _, t)| nav.region_of(*t) != home)
        .collect();
    people.sort_by_key(|row| row.0);
    let (w, h) = (
        engine.resources.tile_grid.width,
        engine.resources.tile_grid.height,
    );
    let mut moves = Vec::new();
    for (_, entity, at) in people {
        let mut best: Option<(u32, u32, u32)> = None;
        for y in at.1.saturating_sub(12)..(at.1 + 13).min(h) {
            for x in at.0.saturating_sub(12)..(at.0 + 13).min(w) {
                if nav.region_of((x, y)) == home {
                    let key = (distance((x, y), at), y, x);
                    if best.is_none_or(|b| key < b) {
                        best = Some(key);
                    }
                }
            }
        }
        moves.push((entity, best));
    }
    for (entity, best) in moves {
        if let (Some((_, y, x)), Ok(mut pos)) = (
            best,
            engine
                .world
                .get::<&mut sim_core::components::Position>(entity),
        ) {
            pos.x = x;
            pos.y = y;
        }
    }
}

/// M2 (goal 3.2): the material table, boulders and flint outcrops.
///
/// - Boulders: a keyed draw `(seed, STREAM_BOULDER, x, y, slot 0) % 1000` below
///   `hill_boulder_permille` on hill tiles or `grass_boulder_permille` on plain
///   tiles; stone `boulder_stone` (slot 1); granite on hills, limestone elsewhere.
/// - Outcrops: `bank / bank_tiles_per_outcrop` of them, where `bank` counts the
///   tiles that are not water or mountain and lie within `bank_reach` of a
///   river tile; placed on the free bank tiles shuffled with `STREAM_OUTCROP`
///   (slot 0), first ones first; flint `outcrop_flint` (slot 1).
///
/// Never on the camp clearing, a building, a water source, a bush, a deadwood
/// spot or a tree. Tiles in `(y, x)` order throughout.
fn nodes(cfg: &WorldGenConfig, gen: &ColonyGen, r: &mut SimResources, camp: Tile) {
    let Some(rule) = r.rules.colony.and_then(|rule| rule.economy) else {
        return;
    };
    r.colony.materials = ColonyMaterials::load();
    let free = |r: &SimResources, p: Tile| {
        distance(p, camp) > gen.clearing_radius
            && !r.building_kinds.contains_key(&p)
            && !r.water_tiles.contains_key(&p)
            && !r.food_source_max.contains_key(&p)
            && !r.food_tiles.contains_key(&p)
            && !r.wood_source_max.contains_key(&p)
            && !r.colony.trees.contains_key(&p)
    };
    let amount = |stream: u64, p: Tile, range: (u8, u8)| {
        let lo = range.0.min(range.1);
        let span = u64::from(range.1.max(lo) - lo) + 1;
        lo + keyed_below(cfg.seed, stream, u64::from(p.0), u64::from(p.1), 1, 0, span) as u8
    };
    for p in tiles(r) {
        let permille = match terrain(r, p) {
            TerrainType::Hill => rule.hill_boulder_permille,
            TerrainType::Plain => rule.grass_boulder_permille,
            _ => continue,
        };
        if !free(r, p) {
            continue;
        }
        let draw = keyed_below(
            cfg.seed,
            STREAM_BOULDER,
            u64::from(p.0),
            u64::from(p.1),
            0,
            0,
            1000,
        );
        if draw < u64::from(permille) {
            let material = if terrain(r, p) == TerrainType::Hill {
                GRANITE
            } else {
                LIMESTONE
            };
            let left = amount(STREAM_BOULDER, p, rule.boulder_stone);
            r.colony.nodes.insert(
                p,
                ResourceNode {
                    kind: NodeKind::Boulder,
                    left,
                    material,
                },
            );
        }
    }
    let reach = rule.bank_reach;
    let (w, h) = (r.tile_grid.width, r.tile_grid.height);
    let near_water = |r: &SimResources, p: Tile| {
        let (x0, y0) = (p.0.saturating_sub(reach), p.1.saturating_sub(reach));
        let (x1, y1) = ((p.0 + reach).min(w - 1), (p.1 + reach).min(h - 1));
        (y0..=y1).any(|y| (x0..=x1).any(|x| terrain(r, (x, y)) == TerrainType::River))
    };
    let bank: Vec<Tile> = tiles(r)
        .into_iter()
        .filter(|&p| {
            !matches!(terrain(r, p), TerrainType::River | TerrainType::Mountain) && near_water(r, p)
        })
        .collect();
    // One outcrop per `bank_tiles_per_outcrop` bank tiles, placed on the free ones.
    let count = bank.len() / rule.bank_tiles_per_outcrop.max(1) as usize;
    let mut bank: Vec<Tile> = bank
        .into_iter()
        .filter(|&p| free(r, p) && !r.colony.nodes.contains_key(&p))
        .collect();
    shuffle(&mut bank, cfg.seed, STREAM_OUTCROP, 0);
    bank.truncate(count);
    bank.sort_by_key(|&(x, y)| (y, x));
    for p in bank {
        let left = amount(STREAM_OUTCROP, p, rule.outcrop_flint);
        r.colony.nodes.insert(
            p,
            ResourceNode {
                kind: NodeKind::FlintOutcrop,
                left,
                material: FLINT,
            },
        );
    }
    r.colony.touch(false);
}

/// `gen.outer_bushes` bushes on Forest tiles with a Plain neighbour beyond
/// the camp ring, never on water, a building or another source.
fn outer_bushes(cfg: &WorldGenConfig, gen: &ColonyGen, r: &mut SimResources, camp: Tile) {
    let mut candidates: Vec<Tile> = tiles(r)
        .into_iter()
        .filter(|&p| {
            distance(p, camp) >= OUTER_BUSH_MIN
                && terrain(r, p) == TerrainType::Forest
                && !r.building_kinds.contains_key(&p)
                && !r.water_tiles.contains_key(&p)
                && !r.food_source_max.contains_key(&p)
                && !r.wood_source_max.contains_key(&p)
                && r.tile_grid
                    .neighbors_4(p.0, p.1)
                    .iter()
                    .any(|&q| terrain(r, q) == TerrainType::Plain)
        })
        .collect();
    shuffle(&mut candidates, cfg.seed, STREAM_COLONY_BUSH, 0);
    candidates.truncate(gen.outer_bushes as usize);
    candidates.sort_by_key(|&(x, y)| (y, x));
    for (x, y) in candidates {
        r.set_food_tile(x, y, cfg.berry_capacity);
        r.set_food_source_max(x, y, cfg.berry_capacity);
    }
}

/// `gen.outer_deadwood` deadwood spots beyond the camp band: Forest, then
/// Plain; never water, a building or a food source; two tiles from every spot.
fn outer_deadwood(cfg: &WorldGenConfig, gen: &ColonyGen, r: &mut SimResources, camp: Tile) {
    let mut chosen: Vec<Tile> = Vec::new();
    for (pass, kind) in [(0u32, TerrainType::Forest), (1, TerrainType::Plain)] {
        let mut candidates: Vec<Tile> = tiles(r)
            .into_iter()
            .filter(|&p| {
                distance(p, camp) >= OUTER_WOOD_MIN
                    && terrain(r, p) == kind
                    && !r.water_tiles.contains_key(&p)
                    && !r.building_kinds.contains_key(&p)
                    && !r.food_source_max.contains_key(&p)
                    && !r.food_tiles.contains_key(&p)
            })
            .collect();
        shuffle(&mut candidates, cfg.seed, STREAM_COLONY_WOOD, pass);
        for p in candidates {
            if chosen.len() >= gen.outer_deadwood as usize {
                break;
            }
            let near = |q: &Tile| distance(p, *q) < 2;
            if !chosen.iter().any(near) && !r.wood_source_max.keys().any(near) {
                chosen.push(p);
            }
        }
    }
    for p in chosen {
        r.wood_tiles.insert(p, cfg.deadwood_capacity);
        r.wood_source_max.insert(p, cfg.deadwood_capacity);
        r.wood_spot_area.insert(p, 0);
    }
}

/// Make at least `gen.min_adults` founders adults: while short, the youngest
/// non-adult founder (a `Child` or `Elder` marker) loses the marker and gets
/// an adult age keyed on `(seed, "colony_founder_fix", slot)`. Returns how
/// many founders were replaced (0 on every seed so far: the founder age range
/// lies wholly between childhood and old age).
pub fn ensure_adults(cfg: &WorldGenConfig, gen: &ColonyGen, engine: &mut SimEngine) -> u32 {
    let mut founders: Vec<(u64, hecs::Entity, bool, i64)> = engine
        .world
        .query::<(&Agent, Option<&Child>, Option<&Elder>, Option<&Life>)>()
        .iter()
        .map(|(entity, (agent, child, elder, life))| {
            (
                agent.id,
                entity,
                child.is_none() && elder.is_none(),
                life.map_or(0, |l| l.born_tick),
            )
        })
        .collect();
    founders.sort_by_key(|f| f.0);
    let adults = founders.iter().filter(|f| f.2).count() as u32;
    if adults >= gen.min_adults {
        return 0;
    }
    let mut young: Vec<_> = founders.iter().filter(|f| !f.2).copied().collect();
    // Youngest first: the latest birth tick, ties by AgentId.
    young.sort_by_key(|f| (std::cmp::Reverse(f.3), f.0));
    let (lo, hi) = engine.resources.rules.society.aging.map_or((0, 1), |a| {
        (
            a.founder_age_min_ticks,
            a.elder_age_ticks.max(a.founder_age_min_ticks + 1),
        )
    });
    let mut fixed = 0;
    for (slot, (_, entity, _, _)) in young.into_iter().enumerate() {
        if adults + fixed >= gen.min_adults {
            break;
        }
        let age = lo
            + keyed_below(
                cfg.seed,
                STREAM_COLONY_FOUNDER_FIX,
                slot as u64,
                0,
                0,
                0,
                hi - lo,
            );
        let _ = engine.world.remove_one::<Child>(entity);
        let _ = engine.world.remove_one::<Elder>(entity);
        if let Ok(mut life) = engine.world.get::<&mut Life>(entity) {
            life.born_tick = -(age as i64);
            life.lifespan_ticks = life.lifespan_ticks.max(age + 1);
        }
        fixed += 1;
    }
    fixed
}

/// One tree on every Forest tile outside the camp clearing that is not a
/// bush, a deadwood spot, a water source or a building (goal 6.2), with
/// `tree_logs_min + keyed_below(seed, STREAM_TREE_LOGS, x, y, 0, 0, span)` logs.
fn trees(cfg: &WorldGenConfig, gen: &ColonyGen, r: &mut SimResources, camp: Tile) {
    let Some(rule) = r.rules.colony else {
        return;
    };
    let lo = rule.tree_logs_min.min(rule.tree_logs_max);
    let span = u64::from(rule.tree_logs_max.max(lo) - lo) + 1;
    for p in tiles(r) {
        if terrain(r, p) != TerrainType::Forest
            || distance(p, camp) <= gen.clearing_radius
            || r.food_source_max.contains_key(&p)
            || r.wood_source_max.contains_key(&p)
            || r.water_tiles.contains_key(&p)
            || r.building_kinds.contains_key(&p)
        {
            continue;
        }
        let logs = lo
            + keyed_below(
                cfg.seed,
                STREAM_TREE_LOGS,
                u64::from(p.0),
                u64::from(p.1),
                0,
                0,
                span,
            ) as u8;
        r.colony.trees.insert(p, Tree { logs });
    }
    r.colony.touch(true);
}
