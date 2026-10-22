//! Putting items down (7.4 merge and spill) and the colony site index.
//!
//! Items are never destroyed: `put_items` always finds room, ring by ring.
//! M2: a stack holds one [`ItemSpec`] bucket (kind, material, quality / 10),
//! and each kind has its own stack maximum (`ColonyRule::stack_max`).
use hecs::{Entity, World};
use sim_core::colony::{ColonySite, ItemKind, ItemSpec, ItemStack, MaterialNeeds};
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
/// not under a building or site; in the colony (M3) also passable and not a
/// wall or door, so a spill never lands where no one can walk.
pub fn can_hold_items(res: &SimResources, tile: (u32, u32)) -> bool {
    matches!(
        terrain(res, tile),
        Some(t) if t != TerrainType::River && t != TerrainType::Mountain
    ) && !under_building(res, tile)
        && (res.rules.colony.is_none()
            || (crate::runtime::nav::passable(res, tile) && !has_wall(res, tile)))
}

/// Whether `tile` carries a wall or a door (M3).
pub fn has_wall(res: &SimResources, tile: (u32, u32)) -> bool {
    let grid = &res.tile_grid;
    tile.0 < grid.width
        && tile.1 < grid.height
        && grid.wall_material[grid.idx(tile.0, tile.1)].is_some()
}

/// The stack maximum of `kind` under the colony rule (25 without one).
pub fn stack_max(res: &SimResources, kind: ItemKind) -> u16 {
    res.rules
        .colony
        .map_or(kind.default_stack_max(), |rule| rule.stack_max(kind))
        .max(1)
}

/// Room for `spec` on `tile` (0 when the tile cannot hold items or holds a
/// stack that `spec` does not merge with).
pub fn room_on(res: &SimResources, tile: (u32, u32), spec: ItemSpec) -> u16 {
    if !can_hold_items(res, tile) {
        return 0;
    }
    let max = stack_max(res, spec.kind);
    match res.colony.ground_items.get(&tile) {
        None => max,
        Some(stack) if stack.spec().merges_with(&spec) => max.saturating_sub(stack.count),
        Some(_) => 0,
    }
}

/// Put `n` items of `spec` down at `tile` (7.4): fill `tile` up to the stack
/// maximum, then the nearest tiles in Chebyshev ring order (within a ring
/// `(y, x)` ascending) that can hold items and are empty or hold a stack it
/// merges with below the maximum. Returns the tiles filled, in order.
/// Never destroys an item while any tile of the map has room. M3: in the
/// colony a spill stays in the nav region of `tile` (when `tile` is passable),
/// so items never land where the one who dropped them cannot walk; only when
/// that region has no room does it spread further.
pub fn put_items(
    res: &mut SimResources,
    tile: (u32, u32),
    spec: ItemSpec,
    n: u16,
) -> Vec<(u32, u32)> {
    let region = (res.rules.colony.is_some() && res.nav.is_built() && res.nav.passable(tile))
        .then(|| res.nav.region_of(tile))
        .filter(|r| *r != 0);
    let (mut filled, mut left) = put_items_in(res, tile, spec, n, region);
    if region.is_some() && left > 0 {
        let (more, rest) = put_items_in(res, tile, spec, left, None);
        filled.extend(more);
        left = rest;
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

/// One pass of [`put_items`], only onto tiles of `region` when given; returns
/// the tiles filled and the items still left.
fn put_items_in(
    res: &mut SimResources,
    tile: (u32, u32),
    spec: ItemSpec,
    n: u16,
    region: Option<u32>,
) -> (Vec<(u32, u32)>, u16) {
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
                let room = room_on(res, t, spec);
                if room == 0 || region.is_some_and(|r| res.nav.region_of(t) != r) {
                    continue;
                }
                let add = room.min(left);
                let stack = res
                    .colony
                    .ground_items
                    .entry(t)
                    .or_insert(ItemStack::of(spec, 0));
                stack.absorb(spec, add);
                left -= add;
                filled.push(t);
            }
        }
        ring += 1;
    }
    (filled, left)
}

/// Take up to `n` items of `kind` from the stack at `tile`; removes the
/// stack at 0. Returns the number taken and their spec.
pub fn take_items(
    res: &mut SimResources,
    tile: (u32, u32),
    kind: ItemKind,
    n: u16,
) -> (u16, ItemSpec) {
    let none = (0, ItemSpec::raw(kind, None));
    let Some(stack) = res.colony.ground_items.get_mut(&tile) else {
        return none;
    };
    if stack.kind != kind {
        return none;
    }
    let spec = stack.spec();
    let taken = stack.count.min(n);
    stack.count -= taken;
    if stack.count == 0 {
        res.colony.ground_items.remove(&tile);
    }
    if taken > 0 {
        res.colony.touch(false);
    }
    (taken, spec)
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
        let log = ItemSpec::raw(ItemKind::Log, None);
        let filled = put_items(&mut res, (3, 3), log, 12);
        // Ring 0: (3,3) gets 5; ring 1 in (y, x) order: (2,2) 5, then (4,2) 2 ((3,2) is water).
        assert_eq!(filled, vec![(3, 3), (2, 2), (4, 2)]);
        assert_eq!(res.colony.ground_items[&(3, 3)].count, 5);
        assert_eq!(res.colony.ground_items[&(2, 2)].count, 5);
        assert_eq!(res.colony.ground_items[&(4, 2)].count, 2);
        assert!(!res.colony.ground_items.contains_key(&(3, 2)));
        assert_eq!(res.colony.ground_total(ItemKind::Log), 12);
        assert_eq!(take_items(&mut res, (4, 2), ItemKind::Log, 9), (2, log));
        assert!(!res.colony.ground_items.contains_key(&(4, 2)));
    }

    #[test]
    fn put_items_merges_into_a_partial_stack_and_skips_buildings() {
        let mut res = res();
        let log = ItemSpec::raw(ItemKind::Log, None);
        res.colony
            .ground_items
            .insert((1, 1), ItemStack::of(log, 3));
        res.building_kinds
            .insert((0, 0), sim_core::components::BuildingKind::Hearth);
        let filled = put_items(&mut res, (1, 1), log, 4);
        assert_eq!(filled, vec![(1, 1), (1, 0)]);
        assert_eq!(res.colony.ground_items[&(1, 1)].count, 5);
        assert_eq!(res.colony.ground_items[&(1, 0)].count, 2);
    }
}
