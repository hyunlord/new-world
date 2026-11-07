//! Housing designations remain independent of the stockpile lookup index.
use sim_engine::SimResources;
use std::collections::BTreeSet;
type Tile = (u32, u32);

pub(super) fn add_housing_zone(res: &mut SimResources, tiles: &[Tile]) -> usize {
    let fresh: BTreeSet<_> = tiles
        .iter()
        .copied()
        .filter(|tile| {
            !res.colony
                .buildings
                .housing_zones
                .values()
                .any(|zone| zone.contains(tile))
        })
        .collect();
    if fresh.is_empty() {
        return 0;
    }
    let Some(next) = res.colony.next_zone_id.checked_add(1) else {
        return 0;
    };
    let id = res.colony.next_zone_id;
    res.colony.next_zone_id = next;
    let count = fresh.len();
    res.colony.buildings.housing_zones.insert(id, fresh);
    res.colony.touch(false);
    count
}

pub(super) fn remove_housing_area(res: &mut SimResources, tiles: &[Tile]) -> BTreeSet<Tile> {
    let mut removed = BTreeSet::new();
    for zone in res.colony.buildings.housing_zones.values_mut() {
        for tile in tiles {
            if zone.remove(tile) {
                removed.insert(*tile);
            }
        }
    }
    res.colony
        .buildings
        .housing_zones
        .retain(|_, zone| !zone.is_empty());
    if !removed.is_empty() {
        res.colony.touch(false);
    }
    removed
}
