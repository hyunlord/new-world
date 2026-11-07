//! Read-only upgrade admission, shared by execution and panel queries.
use hecs::World;
use sim_core::colony::building_site::{BuildingPiece, BuildingSite, BuildingStatus, PiecePhase};
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::content_state::CompletedLayer;
use sim_core::colony::knowledge::ContentStatus;
use sim_core::colony::RejectReason;
use sim_engine::SimResources;

pub(super) struct UpgradePlan {
    pub site: BuildingSite,
    pub target: String,
    pub station: Option<String>,
    pub pieces: Vec<BuildingPiece>,
    pub next_piece: u32,
    pub source_record: u32,
    pub walls: Vec<(u32, u32)>,
}

pub(super) fn prepare(
    world: &World,
    res: &SimResources,
    id: u32,
) -> Result<UpgradePlan, RejectReason> {
    let Some(site) = res.colony.buildings.sites.get(&id).cloned() else {
        return Err(RejectReason::Nothing);
    };
    if site.status != BuildingStatus::Complete {
        return Err(RejectReason::NotYet);
    }
    let catalog = installed_or_bundled();
    let Some(current) = catalog.building_templates().get(&site.template) else {
        return Err(RejectReason::Unknown);
    };
    let Some(target) = current
        .upgrade_to
        .as_deref()
        .and_then(|next| catalog.building_templates().get(next))
    else {
        return Err(RejectReason::NotYet);
    };
    if target.status != ContentStatus::Connected
        || !super::commands::known(world, &target.requires_knowledge, catalog)
        || target.rotated_footprint(site.rotation).ok() != Some(site.footprint)
    {
        return Err(RejectReason::NotYet);
    }
    let Ok(plan) = target.expand(catalog, site.station.as_deref(), site.rotation) else {
        return Err(RejectReason::Unknown);
    };
    let Ok(count) = u32::try_from(plan.len()) else {
        return Err(RejectReason::Unknown);
    };
    let Some(next_piece) = res.colony.buildings.next_piece_after(count) else {
        return Err(RejectReason::Unknown);
    };
    let Ok(source_record) = u32::try_from(res.colony.command_log.len()) else {
        return Err(RejectReason::Unknown);
    };
    let mut pieces = Vec::with_capacity(plan.len());
    for (index, next) in plan.into_iter().enumerate() {
        crate::commands::check_placement_capability(res, next.kind)?;
        let tile = (site.anchor.0 + next.offset.0, site.anchor.1 + next.offset.1);
        let Some(previous) = site.pieces.iter().find(|previous| {
            previous.tile == tile
                && previous.size == next.size
                && crate::runtime::jobs::space::completion_layer(previous.kind)
                    == crate::runtime::jobs::space::completion_layer(next.kind)
        }) else {
            return Err(RejectReason::NotYet);
        };
        // Upgrade never treats already-lost walls/furnishings as finished work.
        if !super::physically_complete(res, previous) {
            return Err(RejectReason::Blocked);
        }
        let phase = if previous.kind == next.kind {
            PiecePhase::Complete
        } else {
            if crate::runtime::jobs::space::completion_layer(next.kind) != CompletedLayer::Wall {
                return Err(RejectReason::NotYet);
            }
            PiecePhase::Replacing
        };
        let Ok(index) = u32::try_from(index) else {
            return Err(RejectReason::Unknown);
        };
        pieces.push(BuildingPiece {
            id: res.colony.buildings.next_piece_id + index,
            kind: next.kind,
            tile,
            size: next.size,
            phase,
        });
    }
    // Also reject plans that silently remove an existing floor, bed, or station.
    if site.pieces.iter().any(|old| {
        !pieces.iter().any(|next| {
            next.tile == old.tile
                && next.size == old.size
                && crate::runtime::jobs::space::completion_layer(next.kind)
                    == crate::runtime::jobs::space::completion_layer(old.kind)
        })
    }) {
        return Err(RejectReason::NotYet);
    }
    if !pieces
        .iter()
        .any(|piece| piece.phase == PiecePhase::Replacing)
    {
        return Err(RejectReason::Nothing);
    }
    if pieces.iter().any(|piece| {
        piece.phase == PiecePhase::Replacing && res.colony.designations.contains_key(&piece.tile)
    }) {
        return Err(RejectReason::Blocked);
    }
    let walls: Vec<_> = pieces
        .iter()
        .filter(|piece| piece.phase == PiecePhase::Replacing)
        .map(|piece| piece.tile)
        .collect();
    crate::commands::check_template_walls(res, &walls)?;
    let station = site
        .station
        .clone()
        .or_else(|| target.stations.first().cloned());
    Ok(UpgradePlan {
        site,
        target: target.id.clone(),
        station,
        pieces,
        next_piece,
        source_record,
        walls,
    })
}
