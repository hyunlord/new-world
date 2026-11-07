//! External b bridge adapter draft; assigned beds and real housing designations only.
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
