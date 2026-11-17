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
    let mut changed_manual = BTreeSet::new();
    let state = &mut res.colony.buildings;
    for (&id, zone) in &mut state.housing_zones {
        // Automatic layouts are village planning facts, not player designations.
        if state.zone_causes.contains_key(&id) {
            continue;
        }
        for tile in tiles {
            if zone.remove(tile) {
                removed.insert(*tile);
                changed_manual.insert(id);
            }
        }
    }
    state.housing_zones.retain(|_, zone| !zone.is_empty());
    // Current membership can end without changing the building's historical cause.
    state.site_zones.retain(|site, zone| {
        if !changed_manual.contains(zone) {
            return true;
        }
        let Some(building) = state.sites.get(site) else {
            return false;
        };
        let Some(tiles) = state.housing_zones.get(zone) else {
            return false;
        };
        let (Some(right), Some(bottom)) = (
            building.anchor.0.checked_add(building.footprint.0),
            building.anchor.1.checked_add(building.footprint.1),
        ) else {
            return false;
        };
        (building.anchor.1..bottom)
            .all(|y| (building.anchor.0..right).all(|x| tiles.contains(&(x, y))))
    });
    if !removed.is_empty() {
        res.colony.touch(false);
    }
    removed
}
