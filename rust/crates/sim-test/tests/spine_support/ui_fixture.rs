//! Capture metadata is derived from the saved world; it never supplies simulation state.
use super::*;
use sim_core::components::{BodyHealth, Position};
use sim_systems::runtime::building_templates::{commands, is_intact, materials};

fn contains(tile: Tile, anchor: Tile, size: Tile) -> bool {
    tile.0 >= anchor.0
        && tile.1 >= anchor.1
        && tile.0 - anchor.0 < size.0
        && tile.1 - anchor.1 < size.1
}

fn candidates(engine: &SimEngine) -> Vec<Tile> {
    let centre = camp(engine);
    let mut tiles: Vec<_> = (4..engine.resources.tile_grid.height.saturating_sub(4))
        .flat_map(|y| (4..engine.resources.tile_grid.width.saturating_sub(4)).map(move |x| (x, y)))
        .filter(|&tile| items::chebyshev(tile, centre) <= 24)
        .collect();
    tiles.sort_by_key(|&tile| (items::chebyshev(tile, centre), tile.1, tile.0));
    tiles
}

fn clear(engine: &SimEngine, tile: Tile) -> bool {
    !engine.resources.colony.occupied.contains_key(&tile)
        && !engine.resources.colony.zone_of_tile.contains_key(&tile)
        && !engine
            .resources
            .colony
            .buildings
            .housing_zones
            .values()
            .any(|tiles| tiles.contains(&tile))
        && !engine
            .resources
            .colony
            .buildings
            .sites
            .values()
            .any(|site| contains(tile, site.anchor, site.footprint))
        && sim_systems::runtime::rooms::room_at(&engine.resources, tile).is_none()
        && engine
            .world
            .query::<(&Agent, &Position)>()
            .iter()
            .all(|(_, (_, position))| items::chebyshev(tile, (position.x, position.y)) > 3)
        && commands::preview(&engine.world, &engine.resources, "hut", tile, 0, None).is_ok()
}

pub fn free_tile(engine: &SimEngine) -> Tile {
    candidates(engine)
        .into_iter()
        .find(|&tile| clear(engine, tile))
        .expect("actual unmarked tile clear of templates, rooms, structures and person hitboxes")
}

pub fn home(engine: &SimEngine) -> Option<String> {
    let mut people: Vec<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(entity, agent)| (agent.id, entity))
        .collect();
    people.sort_by_key(|&(id, _)| id);
    people.into_iter().find_map(|(id, entity)| {
        if engine
            .world
            .get::<&BodyHealth>(entity)
            .is_ok_and(|health| health.is_dead())
        {
            return None;
        }
        let bed = *engine.resources.bed_of.get(&id)?;
        let room = sim_systems::runtime::rooms::room_at(&engine.resources, bed)?;
        let kind = room.dwelling_kind.as_deref()?;
        engine
            .resources
            .colony
            .buildings
            .sites
            .values()
            .any(|site| {
                site.template == kind
                    && is_intact(&engine.resources, site)
                    && contains(bed, site.anchor, site.footprint)
                    && room
                        .tiles
                        .iter()
                        .all(|&tile| contains(tile, site.anchor, site.footprint))
            })
            .then(|| entity.to_bits().get().to_string())
    })
}

fn placement(engine: &SimEngine) -> (String, Tile, Tile) {
    let catalog = sim_core::colony::content::installed_or_bundled();
    let have = materials::available_materials(&engine.world, &engine.resources);
    for id in [
        "log_house",
        "hut",
        "storehouse",
        "drying_shed",
        "workshop_shed",
    ] {
        let template = catalog
            .building_templates()
            .get(id)
            .expect("bundled template");
        let needs = commands::material_requirements(&engine.resources, id, None, 0)
            .expect("validated connected default plan");
        if !needs
            .iter()
            .all(|(item, count)| have.get(item).copied().unwrap_or(0) >= *count)
        {
            continue;
        }
        let size = template.rotated_footprint(0).expect("default rotation");
        if size.0 != size.1 {
            continue;
        }
        if let Some(anchor) = candidates(engine).into_iter().find(|&tile| {
            [0, 1].into_iter().all(|rotation| {
                commands::preview(&engine.world, &engine.resources, id, tile, rotation, None)
                    .is_ok()
            })
        }) {
            return (id.to_owned(), anchor, size);
        }
    }
    panic!(
        "02complete has no actually affordable, admitted square template; do not fabricate stock"
    );
}

pub fn manifest(engine: &mut SimEngine, label: &str) -> serde_json::Value {
    match label {
        "02complete" => {
            let (id, anchor, size) = placement(engine);
            let cache = free_tile(engine);
            let mut ui = serde_json::json!({
                "placement_template": id,
                "placement_cursor_tile": [anchor.0 + size.0 / 2, anchor.1 + size.1 / 2],
                "cache_housing_rect": [cache.0, cache.1, cache.0, cache.1]
            });
            super::ui_motion::attach(engine, &mut ui);
            ui
        }
        "03zonegrown" => {
            let (id, tile) = engine
                .resources
                .colony
                .buildings
                .housing_zones
                .iter()
                .flat_map(|(&id, tiles)| tiles.iter().map(move |&tile| (id, tile)))
                .find(|(_, tile)| {
                    !engine
                        .resources
                        .colony
                        .buildings
                        .sites
                        .values()
                        .any(|site| contains(*tile, site.anchor, site.footprint))
                        && engine
                            .world
                            .query::<(&Agent, &Position)>()
                            .iter()
                            .all(|(_, (_, pos))| items::chebyshev(*tile, (pos.x, pos.y)) > 3)
                })
                .expect("real detached housing selection tile");
            serde_json::json!({"housing_tile": [tile.0, tile.1], "housing_zone_id": id})
        }
        "03-ready-upgrade" => {
            serde_json::json!({"native_action": "upgrade", "upgrade_stock_checked": true})
        }
        "04upgraded" => {
            serde_json::json!({"home_entity_bits": home(engine).expect("real assigned template bed")})
        }
        "05cord-world" => {
            let cord = ItemKind::from_name("cord").expect("cord content");
            let (tile, stack) = engine
                .resources
                .colony
                .ground_items
                .iter()
                .find(|(_, stack)| stack.kind == cord && stack.count > 0)
                .expect("real cord pile");
            serde_json::json!({"cord_tile": [tile.0, tile.1], "cord_item_code": cord.code(),
                "cord_count": stack.count, "cord_quality": stack.quality,
                "cord_material": stack.material.map_or("", |id| engine.resources.colony.materials.name(id)),
                "cord_created_before": 0, "cord_created_after": engine.resources.colony.item_ledger.created_of(cord)})
        }
        _ => serde_json::json!({}),
    }
}
