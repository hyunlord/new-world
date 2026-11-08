//! Room identity follows actual completed template interiors, including non-dwellings.
use sim_core::colony::building_template::ShelterKind;
use sim_core::colony::content::installed_or_bundled;
use sim_engine::SimResources;

/// Detection recreates Room values: derive dwelling_kind from saved completed sites each pass.
pub fn associate_rooms(res: &mut SimResources) {
    for room in res.colony.rooms.values_mut() {
        room.dwelling_kind = None;
    }
    let catalog = installed_or_bundled();
    for site in res.colony.buildings.sites.values() {
        if !super::is_intact(res, site) {
            continue;
        }
        let Some(template) = catalog.building_templates().get(&site.template) else {
            continue;
        };
        for room in res.colony.rooms.values_mut() {
            if !room.tiles.is_empty()
                && room.tiles.iter().all(|tile| {
                    tile.0 > site.anchor.0
                        && tile.1 > site.anchor.1
                        && tile.0
                            < site
                                .anchor
                                .0
                                .saturating_add(site.footprint.0)
                                .saturating_sub(1)
                        && tile.1
                            < site
                                .anchor
                                .1
                                .saturating_add(site.footprint.1)
                                .saturating_sub(1)
                })
            {
                room.dwelling_kind = Some(site.template.clone());
                if template.function.warmth == ShelterKind::Open {
                    room.roofed = false;
                }
            }
        }
    }
}
