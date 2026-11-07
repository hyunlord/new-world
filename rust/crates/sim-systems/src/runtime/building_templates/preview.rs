//! Read-only authoritative template admission, shared by queued commands and previews.
use hecs::World;
use sim_core::colony::building_site::{BuildingStatus, Tile};
use sim_core::colony::building_template::PlannedPiece;
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::knowledge::ContentStatus;
use sim_core::colony::RejectReason;
use sim_engine::SimResources;

pub(super) struct BuildPlan {
    pub plan: Vec<PlannedPiece>,
    pub footprint: Tile,
    pub next_site: u32,
    pub next_piece: u32,
    pub source_record: u32,
    pub station: Option<String>,
}

/// Preview every hard gate without allocating metadata or changing physical state.
pub fn preview(
    world: &World,
    res: &SimResources,
    id: &str,
    anchor: Tile,
    rotation: u8,
    station: Option<&str>,
) -> Result<Vec<PlannedPiece>, RejectReason> {
    prepare(world, res, id, anchor, rotation, station).map(|ready| ready.plan)
}

pub(super) fn prepare(
    world: &World,
    res: &SimResources,
    id: &str,
    anchor: Tile,
    rotation: u8,
    station: Option<&str>,
) -> Result<BuildPlan, RejectReason> {
    let catalog = installed_or_bundled();
    let Some(template) = catalog.building_templates().get(id) else {
        return Err(RejectReason::Unknown);
    };
    if template.status != ContentStatus::Connected
        || !super::commands::known(world, &template.requires_knowledge, catalog)
    {
        return Err(RejectReason::NotYet);
    }
    let Ok(footprint) = template.rotated_footprint(rotation) else {
        return Err(RejectReason::Unknown);
    };
    let Some(right) = anchor.0.checked_add(footprint.0) else {
        return Err(RejectReason::OutOfMap);
    };
    let Some(bottom) = anchor.1.checked_add(footprint.1) else {
        return Err(RejectReason::OutOfMap);
    };
    if right > res.tile_grid.width || bottom > res.tile_grid.height {
        return Err(RejectReason::OutOfMap);
    }
    if res.colony.buildings.sites.values().any(|site| {
        site.status == BuildingStatus::Building
            && anchor.0 < site.anchor.0.saturating_add(site.footprint.0)
            && site.anchor.0 < right
            && anchor.1 < site.anchor.1.saturating_add(site.footprint.1)
            && site.anchor.1 < bottom
    }) {
        return Err(RejectReason::Blocked);
    }
    let Ok(plan) = template.expand(catalog, station, rotation) else {
        return Err(RejectReason::Unknown);
    };
    super::commands::plan_materials(res, &plan)?;
    let station = station
        .map(str::to_owned)
        .or_else(|| template.stations.first().cloned());
    if plan.is_empty() {
        return Err(RejectReason::Unknown);
    }
    let Ok(count) = u32::try_from(plan.len()) else {
        return Err(RejectReason::Unknown);
    };
    let state = &res.colony.buildings;
    let Some((next_site, next_piece)) = state.next_ids(count) else {
        return Err(RejectReason::Unknown);
    };
    let Ok(source_record) = u32::try_from(res.colony.command_log.len()) else {
        return Err(RejectReason::Unknown);
    };
    let people = crate::commands::people_tiles(world);
    for piece in &plan {
        crate::commands::check_placement_capability(res, piece.kind)?;
        let tile = (anchor.0 + piece.offset.0, anchor.1 + piece.offset.1);
        if piece.kind.is_space() {
            crate::commands::check_space(res, &people, piece.kind, tile)?;
        } else {
            crate::commands::check_blueprint(res, piece.kind, tile.0, tile.1)?;
        }
    }
    Ok(BuildPlan {
        plan,
        footprint,
        next_site,
        next_piece,
        source_record,
        station,
    })
}
