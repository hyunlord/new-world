//! Putting items down (7.4 merge and spill) and the colony site index.
//!
//! Logs are never destroyed: `put_items` always finds room, ring by ring.
use hecs::{Entity, World};
use sim_core::colony::{ColonySite, ItemKind, ItemStack, MaterialNeeds};
use sim_core::components::ConstructionSite;
use sim_core::TerrainType;
use sim_engine::SimResources;
use std::collections::BTreeMap;

/// Chebyshev distance.
pub fn chebyshev(a: (u32, u32), b: (u32, u32)) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}

/// Terrain of `tile` (`None` out of bounds).
pub fn terrain(res: &SimResources, tile: (u32, u32)) -> Option<TerrainType> {
    let grid = &res.tile_grid;
    (tile.0 < grid.width && tile.1 < grid.height)
        .then(|| grid.terrain_type[grid.idx(tile.0, tile.1)])
}

/// Whether `tile` is water (river).
pub fn is_water(res: &SimResources, tile: (u32, u32)) -> bool {
    terrain(res, tile) == Some(TerrainType::River)
}

/// Whether `tile` is covered by a building or a site: a V7 building tile
/// (`building_kinds`) or a colony footprint (`colony.occupied`).
pub fn under_building(res: &SimResources, tile: (u32, u32)) -> bool {
    res.building_kinds.contains_key(&tile) || res.colony.occupied.contains_key(&tile)
}

/// Whether items may lie on `tile` (7.4): in bounds, not water or mountain,
/// not under a building or site.
pub fn can_hold_items(res: &SimResources, tile: (u32, u32)) -> bool {
    matches!(
        terrain(res, tile),
        Some(t) if t != TerrainType::River && t != TerrainType::Mountain
    ) && !under_building(res, tile)
}

/// Room for `kind` on `tile` (0 when the tile cannot hold items or holds another kind).
pub fn room_on(res: &SimResources, tile: (u32, u32), kind: ItemKind, max: u16) -> u16 {
    if !can_hold_items(res, tile) {
        return 0;
    }
    match res.colony.ground_items.get(&tile) {
        None => max,
        Some(stack) if stack.kind == kind => max.saturating_sub(stack.count),
        Some(_) => 0,
    }
}

/// Put `n` items of `kind` down at `tile` (7.4): fill `tile` up to the stack
/// maximum, then the nearest tiles in Chebyshev ring order (within a ring
/// `(y, x)` ascending) that can hold items and are empty or hold a
/// same-kind stack below the maximum. Returns the tiles filled, in order.
/// Never destroys an item while any tile of the map has room.
pub fn put_items(
    res: &mut SimResources,
    tile: (u32, u32),
    kind: ItemKind,
    n: u16,
) -> Vec<(u32, u32)> {
    let max = res
        .rules
        .colony
        .map_or(25, |rule| rule.stack_max_logs)
        .max(1);
    let (w, h) = (res.tile_grid.width, res.tile_grid.height);
    let mut left = n;
    let mut filled = Vec::new();
    let max_ring = w.max(h);
    let mut ring = 0u32;
    while left > 0 && ring <= max_ring {
        let (cx, cy) = (i64::from(tile.0), i64::from(tile.1));
        let r = i64::from(ring);
        for y in (cy - r)..=(cy + r) {
            for x in (cx - r)..=(cx + r) {
                if left == 0 {
                    break;
                }
                if (x - cx).abs().max((y - cy).abs()) != r || x < 0 || y < 0 {
                    continue;
                }
                let t = (x as u32, y as u32);
                if t.0 >= w || t.1 >= h {
                    continue;
                }
                let room = room_on(res, t, kind, max);
                if room == 0 {
                    continue;
                }
                let add = room.min(left);
                let stack = res
                    .colony
                    .ground_items
                    .entry(t)
                    .or_insert(ItemStack { kind, count: 0 });
                stack.count += add;
                left -= add;
                filled.push(t);
            }
        }
        ring += 1;
    }
    debug_assert_eq!(
        left, 0,
        "put_items: no room left on the map for {left} items"
    );
    if !filled.is_empty() {
        res.colony.touch(false);
    }
    filled
}

/// Take up to `n` items of `kind` from the stack at `tile`; removes the
/// stack at 0. Returns the number taken.
pub fn take_items(res: &mut SimResources, tile: (u32, u32), kind: ItemKind, n: u16) -> u16 {
    let Some(stack) = res.colony.ground_items.get_mut(&tile) else {
        return 0;
    };
    if stack.kind != kind {
        return 0;
    }
    let taken = stack.count.min(n);
    stack.count -= taken;
    if stack.count == 0 {
        res.colony.ground_items.remove(&tile);
    }
    if taken > 0 {
        res.colony.touch(false);
    }
    taken
}

/// One colony construction site as the job systems see it.
#[derive(Debug, Clone, Copy)]
pub struct SiteView {
    /// The site entity.
    pub entity: Entity,
    /// Its blueprint, footprint and centre.
    pub site: ColonySite,
    /// Its materials.
    pub needs: MaterialNeeds,
    /// Its work progress.
    pub progress: u32,
    /// Work needed.
    pub required: u32,
}

/// Every colony site keyed by its centre tile (sorted, so iteration is `(x, y)` order).
pub fn colony_sites(world: &World) -> BTreeMap<(u32, u32), SiteView> {
    let mut out = BTreeMap::new();
    for (entity, (site, needs, construction)) in world
        .query::<(&ColonySite, &MaterialNeeds, &ConstructionSite)>()
        .iter()
    {
        out.insert(
            site.centre(),
            SiteView {
                entity,
                site: *site,
                needs: *needs,
                progress: construction.progress,
                required: construction.blueprint.required_progress,
            },
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::rules::{ColonyRule, WorldRules};
    use sim_core::MaterialRegistry;

    fn res() -> SimResources {
        let mut res = SimResources::new(8, 8, MaterialRegistry::new());
        res.rules = WorldRules {
            colony: Some(ColonyRule {
                stack_max_logs: 5,
                ..ColonyRule::M1
            }),
            ..WorldRules::default()
        };
        res
    }

    #[test]
    fn put_items_fills_the_tile_then_spills_in_ring_order_and_never_on_water() {
        let mut res = res();
        // (3, 2) is water: the ring after (3, 3) skips it.
        let i = res.tile_grid.idx(3, 2);
        res.tile_grid.terrain_type[i] = TerrainType::River;
        let filled = put_items(&mut res, (3, 3), ItemKind::Log, 12);
        // Ring 0: (3,3) gets 5; ring 1 in (y, x) order: (2,2) 5, then (4,2) 2 ((3,2) is water).
        assert_eq!(filled, vec![(3, 3), (2, 2), (4, 2)]);
        assert_eq!(res.colony.ground_items[&(3, 3)].count, 5);
        assert_eq!(res.colony.ground_items[&(2, 2)].count, 5);
        assert_eq!(res.colony.ground_items[&(4, 2)].count, 2);
        assert!(!res.colony.ground_items.contains_key(&(3, 2)));
        assert_eq!(res.colony.ground_total(ItemKind::Log), 12);
        assert_eq!(take_items(&mut res, (4, 2), ItemKind::Log, 9), 2);
        assert!(!res.colony.ground_items.contains_key(&(4, 2)));
    }

    #[test]
    fn put_items_merges_into_a_partial_stack_and_skips_buildings() {
        let mut res = res();
        res.colony.ground_items.insert(
            (1, 1),
            ItemStack {
                kind: ItemKind::Log,
                count: 3,
            },
        );
        res.building_kinds
            .insert((0, 0), sim_core::components::BuildingKind::Hearth);
        let filled = put_items(&mut res, (1, 1), ItemKind::Log, 4);
        assert_eq!(filled, vec![(1, 1), (1, 0)]);
        assert_eq!(res.colony.ground_items[&(1, 1)].count, 5);
        assert_eq!(res.colony.ground_items[&(1, 0)].count, 2);
    }
}
