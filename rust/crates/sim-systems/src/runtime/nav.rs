//! The colony's navigation map (M3, ADR 0021): what each tile costs to enter,
//! from the tile grid and the colony's 1×1 space sites, built once and kept
//! current tile by tile.
//!
//! [`tile_cost`] is the one definition of passability: river, mountain, undug
//! hill, a wall without a door and a wall's construction site are impassable;
//! a door (built or a site) costs 12; a ford (M4: a shallow river tile placed
//! by the colony worldgen) 30; other ground uses its authored path-stage cost. [`ensure`] builds the
//! map when it is missing (a new colony world, or after a load: the map is not
//! saved) and [`refresh`] re-reads one tile after the code that changed it —
//! the only writers are the commands and the job layer, never a renderer.
//! Every function returns at once without `rules.colony`, so the baseline
//! world never has a map.
use sim_core::colony::content::PlannedNav;
use sim_core::colony::paths;
use sim_core::nav::{CostMap, Tile, COST_BANK, COST_DOOR, COST_PLAIN, IMPASSABLE};
use sim_core::TerrainType;
use sim_engine::SimResources;

/// The cost byte of `tile` (0 = impassable; 0 off the map).
/// M-GROW 3.2: on plain ground the path stage's nav cost overrides `COST_PLAIN`.
pub fn tile_cost(res: &SimResources, tile: Tile) -> u8 {
    let grid = &res.tile_grid;
    if tile.0 >= grid.width || tile.1 >= grid.height {
        return IMPASSABLE;
    }
    cost_at(res, grid.idx(tile.0, tile.1), tile, |i| {
        path_stage_cost(res, i)
    })
}

/// [`tile_cost`] of the in-grid `tile` at index `i`, with the plain-ground cost from `stage_cost`.
fn cost_at(res: &SimResources, i: usize, tile: Tile, stage_cost: impl Fn(usize) -> u8) -> u8 {
    let grid = &res.tile_grid;
    match grid.terrain_type[i] {
        TerrainType::River if grid.is_ford(i) => return COST_BANK,
        TerrainType::River | TerrainType::Mountain => return IMPASSABLE,
        TerrainType::Hill if !grid.is_dug(i) => return IMPASSABLE,
        _ => {}
    }
    if grid.wall_material[i].is_some() {
        return if grid.is_door(i) {
            COST_DOOR
        } else {
            IMPASSABLE
        };
    }
    if res.colony.space_sites.is_empty() {
        return stage_cost(i);
    }
    match res
        .colony
        .space_sites
        .get(&tile)
        .map(|kind| kind.definition().planned_nav)
    {
        Some(PlannedNav::Blocked) => IMPASSABLE,
        Some(PlannedNav::Door) => COST_DOOR,
        Some(PlannedNav::Unchanged) | None => stage_cost(i),
    }
}

/// [`passable`] for every tile, indexed like the grid: one pass, the path-stage table read once
/// (D-SIMFIX1-24: whole-map planners call this instead of `passable` per tile).
pub fn passable_mask(res: &SimResources) -> Vec<bool> {
    let grid = &res.tile_grid;
    let stages = paths::installed();
    let stage_costs: Vec<u8> = (0..=u8::MAX)
        .map(|code| {
            stages
                .stage_by_code(code)
                .map_or(COST_PLAIN, |s| s.nav_cost)
        })
        .collect();
    let stage_cost =
        |i: usize| stage_costs[usize::from(res.colony.path_stage.get(i).copied().unwrap_or(0))];
    let width = grid.width as usize;
    (0..grid.len())
        .map(|i| {
            let tile = ((i % width.max(1)) as u32, (i / width.max(1)) as u32);
            cost_at(res, i, tile, stage_cost) != IMPASSABLE
        })
        .collect()
}

/// Plain-ground cost adjusted by the tile's path stage (M-GROW 3.2).
fn path_stage_cost(res: &SimResources, idx: usize) -> u8 {
    let stage_code = res.colony.path_stage.get(idx).copied().unwrap_or(0);
    paths::installed()
        .stage_by_code(stage_code)
        .map_or(COST_PLAIN, |s| s.nav_cost)
}

/// A fresh cost map of the whole grid.
pub fn build(res: &SimResources) -> CostMap {
    let (w, h) = (res.tile_grid.width, res.tile_grid.height);
    let mut map = CostMap::new(w, h);
    for y in 0..h {
        for x in 0..w {
            map.set((x, y), tile_cost(res, (x, y)));
        }
    }
    map
}

/// Build the map if the colony has none yet (or the grid changed size).
pub fn ensure(res: &mut SimResources) {
    if res.rules.colony.is_none() {
        return;
    }
    let map = res.nav.map();
    if map.is_empty() || map.width() != res.tile_grid.width || map.height() != res.tile_grid.height
    {
        let floors: Vec<_> = res
            .content_state
            .completed
            .keys()
            .filter(|(_, layer)| *layer == sim_core::colony::content_state::CompletedLayer::Floor)
            .map(|(tile, _)| *tile)
            .collect();
        for tile in floors {
            crate::runtime::paths::refresh_built_surface(res, tile);
        }
        let fresh = build(res);
        res.nav.install(fresh);
    }
}

/// Re-read `tile` after its terrain, wall, door, dug flag or space site changed
/// (bumps the nav version when its cost changed). Returns whether it changed.
pub fn refresh(res: &mut SimResources, tile: Tile) -> bool {
    if res.rules.colony.is_none() {
        return false;
    }
    ensure(res);
    let cost = tile_cost(res, tile);
    res.nav.set_cost(tile, cost)
}

/// Whether `tile` is passable in the colony (from the tile grid directly, so
/// it holds before the map is built; `true` in the baseline for every tile
/// that is not water or mountain).
pub fn passable(res: &SimResources, tile: Tile) -> bool {
    tile_cost(res, tile) != IMPASSABLE
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::colony::BlueprintKind;
    use sim_core::material::MaterialId;
    use sim_core::rules::{ColonyRule, WorldRules};
    use sim_core::MaterialRegistry;

    fn res() -> SimResources {
        let mut res = SimResources::new(6, 4, MaterialRegistry::new());
        res.rules = WorldRules {
            colony: Some(ColonyRule::M2),
            ..WorldRules::default()
        };
        res
    }

    #[test]
    fn passable_mask_equals_passable_tile_by_tile() {
        let mut res = res();
        let g = &mut res.tile_grid;
        let i = g.idx(0, 0);
        g.terrain_type[i] = TerrainType::River;
        let i = g.idx(1, 0);
        g.terrain_type[i] = TerrainType::River;
        g.set_ford(i, true);
        let i = g.idx(2, 0);
        g.terrain_type[i] = TerrainType::Hill;
        let i = g.idx(3, 0);
        g.terrain_type[i] = TerrainType::Hill;
        g.set_dug(i, true);
        let i = g.idx(4, 0);
        g.wall_material[i] = Some(MaterialId::from_str_hash("oak"));
        let i = g.idx(5, 0);
        g.wall_material[i] = Some(MaterialId::from_str_hash("oak"));
        g.set_door(i, true);
        let i = g.idx(0, 1);
        g.terrain_type[i] = TerrainType::Mountain;
        let check = |res: &SimResources| {
            let mask = passable_mask(res);
            for y in 0..res.tile_grid.height {
                for x in 0..res.tile_grid.width {
                    assert_eq!(
                        mask[res.tile_grid.idx(x, y)],
                        passable(res, (x, y)),
                        "{x},{y}"
                    );
                }
            }
        };
        check(&res);
        res.colony
            .space_sites
            .insert((1, 1), BlueprintKind::LogWall);
        res.colony.space_sites.insert((2, 1), BlueprintKind::Door);
        res.colony
            .space_sites
            .insert((3, 1), BlueprintKind::PackedFloor);
        check(&res);
    }

    #[test]
    fn tile_cost_follows_terrain_walls_doors_and_digging() {
        let mut res = res();
        let g = &mut res.tile_grid;
        let river = g.idx(0, 0);
        g.terrain_type[river] = TerrainType::River;
        let hill = g.idx(1, 0);
        g.terrain_type[hill] = TerrainType::Hill;
        let dug = g.idx(2, 0);
        g.terrain_type[dug] = TerrainType::Hill;
        g.set_dug(dug, true);
        let wall = g.idx(3, 0);
        g.wall_material[wall] = Some(MaterialId::from_str_hash("oak"));
        let door = g.idx(4, 0);
        g.wall_material[door] = Some(MaterialId::from_str_hash("oak"));
        g.set_door(door, true);
        res.colony
            .space_sites
            .insert((0, 1), BlueprintKind::LogWall);
        res.colony.space_sites.insert((1, 1), BlueprintKind::Door);
        res.colony
            .space_sites
            .insert((2, 1), BlueprintKind::PackedFloor);
        let got: Vec<u8> = [
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (4, 0),
            (5, 0),
            (0, 1),
            (1, 1),
            (2, 1),
        ]
        .iter()
        .map(|t| tile_cost(&res, *t))
        .collect();
        assert_eq!(got, vec![0, 0, 10, 0, 12, 10, 0, 12, 10]);
        // M4: a ford is a river tile people wade through at cost 30.
        let ford = res.tile_grid.idx(0, 2);
        res.tile_grid.terrain_type[ford] = TerrainType::River;
        assert_eq!(tile_cost(&res, (0, 2)), 0);
        res.tile_grid.set_ford(ford, true);
        assert_eq!(tile_cost(&res, (0, 2)), 30);
        assert_eq!(tile_cost(&res, (6, 0)), 0, "off the map");
    }

    #[test]
    fn refresh_matches_a_rebuild_and_bumps_the_version() {
        let mut res = res();
        ensure(&mut res);
        let v0 = res.nav.version;
        res.colony
            .space_sites
            .insert((2, 2), BlueprintKind::LogWall);
        assert!(refresh(&mut res, (2, 2)));
        assert!(!refresh(&mut res, (2, 2)), "unchanged");
        assert_eq!(res.nav.version, v0 + 1);
        assert_eq!(*res.nav.map(), build(&res));
        // The baseline world never builds one.
        res.rules.colony = None;
        let mut base = SimResources::new(6, 4, MaterialRegistry::new());
        ensure(&mut base);
        assert!(!res.nav.map().is_empty() && base.nav.map().is_empty());
    }
}
