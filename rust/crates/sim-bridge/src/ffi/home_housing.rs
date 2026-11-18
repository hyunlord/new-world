//! Read-only assigned homes, authored zone evidence and safe decorative yard placements.
use godot::prelude::*;
use sim_core::colony::content::{Catalog, CompletionCategory};
use sim_core::colony::Occupant;
use sim_engine::SimResources;

/// An intact template home associated with this person's assigned bed, never current position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateHome {
    /// Authored template id from the assigned bed's detected room.
    pub dwelling_kind: String,
    /// Localized template name key from the selected catalogue.
    pub name_key: String,
    /// Persistent site identity, not a catalogue ordinal or entity handle.
    pub site_id: u32,
}

/// The economy collector has already found this AgentId; zero is a valid live identity.
pub fn collect_template_home(
    res: &SimResources,
    catalog: &Catalog,
    agent: u64,
) -> Option<TemplateHome> {
    let bed = *res.bed_of.get(&agent)?;
    let room = sim_systems::runtime::rooms::room_at(res, bed);
    let site = if let Some(room) = room {
        let kind = room.dwelling_kind.as_deref()?;
        // Ordered site ids give a stable tie-break; validated state excludes overlapping homes.
        res.colony.buildings.sites.values().find(|site| {
            site.template == kind
                && sim_systems::runtime::building_templates::is_intact(res, site)
                && !room.tiles.is_empty()
                && room
                    .tiles
                    .iter()
                    .all(|tile| strictly_inside(*tile, site.anchor, site.footprint))
        })?
    } else {
        // Legacy huts have no detected interior room. Ownership follows the assigned bed's
        // exact occupied-Hut identity and a matching complete template piece, never distance.
        let Occupant::Hut(centre) = res.colony.occupied.get(&bed)? else {
            return None;
        };
        res.colony.buildings.sites.values().find(|site| {
            sim_systems::runtime::building_templates::is_intact(res, site)
                && site.pieces.iter().any(|piece| {
                    piece.centre() == *centre
                        && catalog.blueprint(piece.kind).is_some_and(|definition| {
                            definition.completion == CompletionCategory::Dwelling
                        })
                        && contains(bed, piece.tile, piece.size)
                })
        })?
    };
    let template = catalog.building_templates().get(&site.template)?;
    Some(TemplateHome {
        dwelling_kind: site.template.clone(),
        name_key: template.name_key.clone(),
        site_id: site.id,
    })
}

/// Add b economy fields; get_agent_detail remains its locked nine-key identity contract.
pub fn append_template_home(dict: &mut VarDictionary, home: Option<&TemplateHome>) {
    dict.set(
        "dwelling_kind",
        home.map_or("", |row| row.dwelling_kind.as_str()),
    );
    dict.set(
        "dwelling_name_key",
        home.map_or("", |row| row.name_key.as_str()),
    );
    dict.set(
        "dwelling_site_id",
        home.map_or(-1, |row| i64::from(row.site_id)),
    );
}

fn contains(tile: (u32, u32), anchor: (u32, u32), size: (u32, u32)) -> bool {
    let Some(right) = anchor.0.checked_add(size.0) else {
        return false;
    };
    let Some(bottom) = anchor.1.checked_add(size.1) else {
        return false;
    };
    tile.0 >= anchor.0 && tile.1 >= anchor.1 && tile.0 < right && tile.1 < bottom
}

fn strictly_inside(tile: (u32, u32), anchor: (u32, u32), size: (u32, u32)) -> bool {
    let Some(right) = anchor.0.checked_add(size.0) else {
        return false;
    };
    let Some(bottom) = anchor.1.checked_add(size.1) else {
        return false;
    };
    tile.0 > anchor.0
        && tile.1 > anchor.1
        && tile.0 < right.saturating_sub(1)
        && tile.1 < bottom.saturating_sub(1)
}

/// One real designated housing tile; independent from the stockpile tile index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HousingTile {
    /// Shared zone allocator identity, never narrowed to i32.
    pub id: u32,
    /// World tile x, checked at the Godot packed-i32 boundary.
    pub x: u32,
    /// World tile y, checked at the Godot packed-i32 boundary.
    pub y: u32,
}

/// Stable global row-major order even though underlying BTreeSet tuples order x before y.
pub fn collect_housing_tiles(res: &SimResources) -> Vec<HousingTile> {
    let mut rows: Vec<_> = res
        .colony
        .buildings
        .housing_zones
        .iter()
        .flat_map(|(&id, tiles)| tiles.iter().map(move |&(x, y)| HousingTile { id, x, y }))
        .collect();
    rows.sort_unstable_by_key(|row| (row.y, row.x, row.id));
    rows
}

/// Append aligned arrays to the existing overlay; version remains its existing overlay_version.
pub fn append_housing_tiles(dict: &mut VarDictionary, rows: &[HousingTile]) -> Result<(), String> {
    let mut ids = Vec::with_capacity(rows.len());
    let mut xs = Vec::with_capacity(rows.len());
    let mut ys = Vec::with_capacity(rows.len());
    for row in rows {
        ids.push(i64::from(row.id));
        xs.push(i32::try_from(row.x).map_err(|_| "housing x exceeds packed-i32".to_string())?);
        ys.push(i32::try_from(row.y).map_err(|_| "housing y exceeds packed-i32".to_string())?);
    }
    // All conversion succeeds before any dictionary mutation.
    dict.set("housing_zone_ids", PackedInt64Array::from(ids.as_slice()));
    dict.set("housing_zone_xs", PackedInt32Array::from(xs.as_slice()));
    dict.set("housing_zone_ys", PackedInt32Array::from(ys.as_slice()));
    Ok(())
}

/// Append layout envelopes separately from player stockpile/housing columns.
pub(super) fn append_layout_tiles(dict: &mut VarDictionary, res: &SimResources) {
    let catalog = sim_core::colony::content::installed_or_bundled();
    let mut rows: Array<VarDictionary> = Array::new();
    for (&id, cause) in &res.colony.buildings.zone_causes {
        let tiles = res
            .colony
            .buildings
            .housing_zones
            .get(&id)
            .or_else(|| res.colony.buildings.layout_zones.get(&id));
        let Some(tiles) = tiles else { continue };
        let mut row = zone_cause(res, id);
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        for &(x, y) in tiles {
            xs.push(i64::from(x));
            ys.push(i64::from(y));
        }
        row.set("xs", PackedInt64Array::from(xs.as_slice()));
        row.set("ys", PackedInt64Array::from(ys.as_slice()));
        let category = catalog
            .zones()
            .rule_for(&cause.kind)
            .map(|rule| rule.category);
        row.set(
            "category",
            match category {
                Some(sim_core::colony::zones::ZoneUse::Housing) => "housing",
                Some(sim_core::colony::zones::ZoneUse::Storage) => "storage",
                Some(sim_core::colony::zones::ZoneUse::Workshop) => "workshop",
                Some(sim_core::colony::zones::ZoneUse::Field) => "field",
                Some(sim_core::colony::zones::ZoneUse::Pen) => "pen",
                None => "default",
            },
        );
        rows.push(&row);
    }
    dict.set("layout_zones", rows);
}

/// Saved layout reason; provenance remains separate from the steward request origin.
pub(super) fn zone_cause(res: &SimResources, id: u32) -> VarDictionary {
    let mut row = VarDictionary::new();
    let Some(cause) = res.colony.buildings.zone_causes.get(&id) else {
        return row;
    };
    let catalog = sim_core::colony::content::installed_or_bundled();
    row.set("zone_id", i64::from(id));
    row.set("zone_kind", cause.kind.as_str());
    row.set(
        "zone_name_key",
        catalog
            .building_templates()
            .zone_kinds()
            .iter()
            .find(|kind| kind.id == cause.kind)
            .map_or("UI_UNKNOWN", |kind| kind.name_key.as_str()),
    );
    row.set("zone_trigger", cause.trigger.as_str());
    row.set("zone_people", i64::from(cause.people));
    row.set("zone_slots", i64::from(cause.slots));
    let tiles = res
        .colony
        .buildings
        .housing_zones
        .get(&id)
        .or_else(|| res.colony.buildings.layout_zones.get(&id));
    let centre = tiles.filter(|tiles| !tiles.is_empty()).map(|tiles| {
        let (sx, sy) = tiles.iter().fold((0u64, 0u64), |(sx, sy), &(x, y)| {
            (sx + u64::from(x), sy + u64::from(y))
        });
        let n = u64::try_from(tiles.len()).unwrap_or(1);
        (sx / n, sy / n)
    });
    let hearth = sim_systems::runtime::jobs::hearth::hearth_tiles(res)
        .first()
        .copied();
    let direction = match (centre, hearth) {
        (Some((x, y)), Some((hx, hy))) if x.abs_diff(u64::from(hx)) > y.abs_diff(u64::from(hy)) => {
            if x < u64::from(hx) {
                "M55_DIR_WEST"
            } else {
                "M55_DIR_EAST"
            }
        }
        (Some((_, y)), Some((_, hy))) => {
            if y < u64::from(hy) {
                "M55_DIR_NORTH"
            } else {
                "M55_DIR_SOUTH"
            }
        }
        _ => "UI_UNKNOWN",
    };
    row.set("zone_direction_key", direction);
    row
}

/// Add a saved zone reason to a chronicle entry without a new getter.
pub(super) fn growth_evidence(res: &SimResources, index: Option<u32>) -> VarDictionary {
    let Some(entry) = index.and_then(|i| res.chronicle.entry(i)) else {
        return VarDictionary::new();
    };
    use sim_core::chronicle::ChronicleKind;
    if entry.kind == ChronicleKind::ZoneLaidOut {
        return zone_cause(res, entry.cause);
    }
    let mut row = VarDictionary::new();
    let catalog = sim_core::colony::content::installed_or_bundled();
    match entry.kind {
        ChronicleKind::HouseStage => {
            row.set("site_id", i64::from(entry.cause));
            row.set(
                "house_stage_name_key",
                catalog
                    .yards()
                    .stages
                    .iter()
                    .find(|stage| i64::from(stage.code) == entry.value)
                    .map_or("UI_UNKNOWN", |stage| stage.name_key.as_str()),
            );
        }
        ChronicleKind::ReputationChanged => {
            let keys: PackedStringArray = (0..2)
                .map(|slot| (entry.cause >> (8 * slot)) & 0xFF)
                .filter(|byte| *byte != 0)
                .map(|byte| {
                    GString::from(
                        super::neighbours::reputation_cause_key(
                            (byte >> 1) as usize - 1,
                            byte & 1 == 1,
                        )
                        .as_str(),
                    )
                })
                .collect();
            row.set("reputation_cause_keys", keys);
        }
        ChronicleKind::MaterialWave => {
            row.set("site_id", i64::from(entry.cause));
            row.set(
                "era_name_key",
                catalog
                    .knowledge_defs()
                    .eras
                    .eras
                    .iter()
                    .find(|era| i64::from(era.order) == entry.value)
                    .map_or("UI_UNKNOWN", |era| era.name_key.as_str()),
            );
        }
        _ => {}
    }
    row
}

/// One decorative prop admitted by the exact collector used by the 3D snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YardPropView {
    /// Authored prop identity, independent from its model.
    pub id: String,
    /// Selected 3D model identity; the renderer supplies its category fallback.
    pub model: String,
    /// Free footprint-adjacent world tile.
    pub tile: (u32, u32),
    /// Clockwise quarter-turns, including the house rotation.
    pub rotation: u8,
}

/// Project authored props without allocating Godot objects or mutating simulation space.
pub fn collect_yard_props(
    res: &SimResources,
    catalog: &Catalog,
    site: &sim_core::colony::building_site::BuildingSite,
) -> Vec<YardPropView> {
    let Some(progress) = res.colony.buildings.house_growth.get(&site.id) else {
        return Vec::new();
    };
    if !sim_systems::runtime::building_templates::is_intact(res, site) {
        return Vec::new();
    }
    let mut claimed = std::collections::BTreeSet::new();
    catalog
        .yards()
        .props
        .iter()
        .filter_map(|prop| {
            if progress.stage < prop.min_stage {
                return None;
            }
            let tile = prop_tile(site.anchor, site.footprint, site.rotation, prop.offset)?;
            if !yard_tile_free(res, tile) || !claimed.insert(tile) {
                return None;
            }
            Some(YardPropView {
                id: prop.id.clone(),
                model: prop.model.clone(),
                tile,
                rotation: (prop.rotation + site.rotation) % 4,
            })
        })
        .collect()
}

/// Authored stage and safe decorative placements for one real intact dwelling.
pub(super) fn append_house_stage(
    row: &mut VarDictionary,
    res: &SimResources,
    site: &sim_core::colony::building_site::BuildingSite,
) {
    let catalog = sim_core::colony::content::installed_or_bundled();
    let progress = res.colony.buildings.house_growth.get(&site.id);
    let stage = progress.map_or(0, |p| p.stage);
    row.set("house_stage", i64::from(stage));
    // M-GROW 3.5: people whose bed lies inside the footprint (chimney smoke).
    let inside = |(x, y): (u32, u32)| {
        x >= site.anchor.0
            && y >= site.anchor.1
            && x < site.anchor.0.saturating_add(site.footprint.0)
            && y < site.anchor.1.saturating_add(site.footprint.1)
    };
    row.set(
        "occupants",
        i64::try_from(res.bed_of.values().filter(|bed| inside(**bed)).count()).unwrap_or(0),
    );
    row.set(
        "occupied_days",
        i64::from(progress.map_or(0, |p| p.occupied_days)),
    );
    row.set(
        "house_stage_name_key",
        catalog
            .yards()
            .stages
            .iter()
            .find(|s| s.code == stage)
            .map_or("UI_UNKNOWN", |s| s.name_key.as_str()),
    );
    let mut props: Array<VarDictionary> = Array::new();
    for prop in collect_yard_props(res, catalog, site) {
        let mut detail = VarDictionary::new();
        detail.set("id", prop.id.as_str());
        detail.set("model", prop.model.as_str());
        detail.set("x", i64::from(prop.tile.0));
        detail.set("y", i64::from(prop.tile.1));
        detail.set("rotation", i64::from(prop.rotation));
        props.push(&detail);
    }
    row.set("yard_props", props);
}

fn prop_tile(
    anchor: (u32, u32),
    size: (u32, u32),
    rotation: u8,
    offset: (i32, i32),
) -> Option<(u32, u32)> {
    let (x, y) = (i64::from(offset.0), i64::from(offset.1));
    let (w, h) = (i64::from(size.0), i64::from(size.1));
    let (x, y) = match rotation {
        0 => (x, y),
        1 => (w - 1 - y, x),
        2 => (w - 1 - x, h - 1 - y),
        3 => (y, h - 1 - x),
        _ => return None,
    };
    // Only a footprint-adjacent outside tile is a yard; no remote decorative claims.
    if x < -1 || y < -1 || x > w || y > h || (x >= 0 && x < w && y >= 0 && y < h) {
        return None;
    }
    Some((
        u32::try_from(i64::from(anchor.0) + x).ok()?,
        u32::try_from(i64::from(anchor.1) + y).ok()?,
    ))
}

fn yard_tile_free(res: &SimResources, tile: (u32, u32)) -> bool {
    if tile.0 >= res.tile_grid.width
        || tile.1 >= res.tile_grid.height
        || !sim_systems::runtime::nav::passable(res, tile)
    {
        return false;
    }
    let i = res.tile_grid.idx(tile.0, tile.1);
    if res.colony.path_stage.get(i).copied().unwrap_or(0) > 0
        || res.colony.occupied.contains_key(&tile)
        || res.colony.space_sites.contains_key(&tile)
        || res.colony.trees.contains_key(&tile)
        || res.colony.nodes.contains_key(&tile)
        || res.colony.ground_items.contains_key(&tile)
        || res.cultivation.fields.contains_key(&tile)
        || res.ore.deposits.contains_key(&tile)
        || res.colony.zone_of_tile.contains_key(&tile)
        || res
            .colony
            .buildings
            .sites
            .values()
            .any(|site| contains(tile, site.anchor, site.footprint))
    {
        return false;
    }
    // A door and its immediate approach stay clear even before footfall wears a path.
    for dy in -1i64..=1 {
        for dx in -1i64..=1 {
            let (Ok(x), Ok(y)) = (
                u32::try_from(i64::from(tile.0) + dx),
                u32::try_from(i64::from(tile.1) + dy),
            ) else {
                continue;
            };
            if x < res.tile_grid.width
                && y < res.tile_grid.height
                && res.tile_grid.is_door(res.tile_grid.idx(x, y))
            {
                return false;
            }
            if res.colony.space_sites.get(&(x, y)).is_some_and(|kind| {
                kind.definition().planned_nav == sim_core::colony::content::PlannedNav::Door
            }) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod growth_tests {
    use super::prop_tile;
    #[test]
    fn yard_offsets_rotate_adjacent_to_the_actual_footprint() {
        assert_eq!(prop_tile((10, 10), (3, 4), 0, (-1, 2)), Some((9, 12)));
        assert_eq!(prop_tile((10, 10), (4, 3), 1, (-1, 2)), Some((11, 9)));
        assert_eq!(prop_tile((10, 10), (3, 4), 2, (-1, 2)), Some((13, 11)));
        assert_eq!(prop_tile((10, 10), (4, 3), 3, (-1, 2)), Some((12, 13)));
        assert_eq!(prop_tile((0, 0), (3, 3), 0, (-1, 0)), None);
        assert_eq!(prop_tile((10, 10), (3, 3), 0, (1, 1)), None);
        assert_eq!(prop_tile((10, 10), (3, 3), 0, (-2, 0)), None);
    }
}
