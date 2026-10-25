//! The colony's navigation map (M3, ADR 0021): what each tile costs to enter,
//! from the tile grid and the colony's 1×1 space sites, built once and kept
//! current tile by tile.
//!
//! [`tile_cost`] is the one definition of passability: river, mountain, undug
//! hill, a wall without a door and a wall's construction site are impassable;
//! a door (built or a site) costs 12; a ford (M4: a shallow river tile placed
//! by the colony worldgen) 30; anything else 10. [`ensure`] builds the
//! map when it is missing (a new colony world, or after a load: the map is not
//! saved) and [`refresh`] re-reads one tile after the code that changed it —
//! the only writers are the commands and the job layer, never a renderer.
//! Every function returns at once without `rules.colony`, so the baseline
//! world never has a map.
use sim_core::colony::content::PlannedNav;
use sim_core::nav::{CostMap, Tile, COST_BANK, COST_DOOR, COST_PLAIN, IMPASSABLE};
use sim_core::TerrainType;
use sim_engine::SimResources;

/// The cost byte of `tile` (0 = impassable; 0 off the map).
pub fn tile_cost(res: &SimResources, tile: Tile) -> u8 {
    let grid = &res.tile_grid;
    if tile.0 >= grid.width || tile.1 >= grid.height {
        return IMPASSABLE;
    }
    let i = grid.idx(tile.0, tile.1);
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
    match res
        .colony
        .space_sites
        .get(&tile)
        .map(|kind| kind.definition().planned_nav)
    {
        Some(PlannedNav::Blocked) => IMPASSABLE,
        Some(PlannedNav::Door) => COST_DOOR,
        Some(PlannedNav::Unchanged) | None => COST_PLAIN,
    }
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
