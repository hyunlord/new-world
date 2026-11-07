//! Validate additive state before save/restore; corruption never manufactures completion.
use hecs::World;
use sim_core::colony::building_site::{BuildingSite, BuildingStatus, PiecePhase};
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::content_state::CompletedLayer;
use sim_core::colony::knowledge::ContentStatus;
use sim_core::colony::{ColonySite, Designation, MaterialNeeds};
use sim_core::components::ConstructionSite;
use sim_engine::SimResources;
use std::collections::BTreeSet;

/// Check semantic shape, positive active ownership, and immutable completion provenance.
pub fn validate_state(world: &World, res: &SimResources) -> Result<(), String> {
    validate_state_at_boundary(world, res, res.current_tick)
}

/// Pending commands may reach the engine admission boundary; completed facts may not.
pub fn validate_state_at_boundary(
    world: &World,
    res: &SimResources,
    command_boundary: u64,
) -> Result<(), String> {
    let state = &res.colony.buildings;
    if !state.sites.is_empty() && res.rules.colony.is_none() {
        return Err("building sites outside colony".into());
    }
    let mut piece_ids = BTreeSet::new();
    for (&id, site) in &state.sites {
        if id != site.id || id >= state.next_site_id {
            return Err("building site id/counter mismatch".into());
        }
        validate_site(world, res, site, command_boundary)?;
        for piece in &site.pieces {
            if piece.id >= state.next_piece_id || !piece_ids.insert(piece.id) {
                return Err("building piece id/counter mismatch".into());
            }
        }
    }
    let mut housing = BTreeSet::new();
    for (&id, tiles) in &state.housing_zones {
        if id >= res.colony.next_zone_id || res.colony.zones.contains_key(&id) || tiles.is_empty() {
            return Err("invalid shared housing zone identity".into());
        }
        for &tile in tiles {
            if tile.0 >= res.tile_grid.width
                || tile.1 >= res.tile_grid.height
                || !housing.insert(tile)
            {
                return Err("invalid or overlapping housing zone tiles".into());
            }
        }
    }
    super::history_validation::validate(res, command_boundary)
}

fn validate_site(
    world: &World,
    res: &SimResources,
    site: &BuildingSite,
    command_boundary: u64,
) -> Result<(), String> {
    let catalog = installed_or_bundled();
    let template = catalog
        .building_templates()
        .get(&site.template)
        .ok_or("unknown saved building template")?;
    if template.status != ContentStatus::Connected
        || site.rotation > 3
        || template.rotated_footprint(site.rotation).ok() != Some(site.footprint)
    {
        return Err("saved building template/rotation/footprint mismatch".into());
    }
    let station_valid = match site.station.as_deref() {
        Some(station) => template.stations.iter().any(|allowed| allowed == station),
        None => template.stations.is_empty(),
    };
    if !station_valid {
        return Err("saved building station is not normalized/authored".into());
    }
    let plan = template.expand(catalog, site.station.as_deref(), site.rotation)?;
    if plan.is_empty() || plan.len() != site.pieces.len() {
        return Err("empty/truncated building plan".into());
    }
    let right = site
        .anchor
        .0
        .checked_add(site.footprint.0)
        .ok_or("building footprint overflow")?;
    let bottom = site
        .anchor
        .1
        .checked_add(site.footprint.1)
        .ok_or("building footprint overflow")?;
    if right > res.tile_grid.width || bottom > res.tile_grid.height {
        return Err("building footprint outside map".into());
    }
    for (index, (piece, expected)) in site.pieces.iter().zip(&plan).enumerate() {
        let x = site
            .anchor
            .0
            .checked_add(expected.offset.0)
            .ok_or("building offset overflow")?;
        let y = site
            .anchor
            .1
            .checked_add(expected.offset.1)
            .ok_or("building offset overflow")?;
        let end_x = piece
            .tile
            .0
            .checked_add(piece.size.0)
            .ok_or("piece footprint overflow")?;
        let end_y = piece
            .tile
            .1
            .checked_add(piece.size.1)
            .ok_or("piece footprint overflow")?;
        if piece.tile != (x, y)
            || piece.kind != expected.kind
            || piece.size != expected.size
            || piece.size.0 == 0
            || piece.size.1 == 0
            || end_x > right
            || end_y > bottom
        {
            return Err("ordered physical building plan mismatch".into());
        }
        if site.status == BuildingStatus::Cancelled {
            if !matches!(piece.phase, PiecePhase::Complete | PiecePhase::Cancelled) {
                return Err("cancelled building retains live work".into());
            }
            continue;
        }
        if site.status == BuildingStatus::Complete {
            if piece.phase != PiecePhase::Complete {
                return Err("complete building has unfinished piece".into());
            }
            // Current physical loss is legal with an immutable historical completion.
            continue;
        }
        if matches!(piece.phase, PiecePhase::Active | PiecePhase::Complete)
            && site.pieces[..index]
                .iter()
                .any(|prior| prior.overlaps(piece) && prior.phase != PiecePhase::Complete)
        {
            return Err("building layer activated out of order".into());
        }
        match piece.phase {
            PiecePhase::Active => {
                let count = world
                    .query::<(&ColonySite, &MaterialNeeds, &ConstructionSite)>()
                    .iter()
                    .filter(|(_, (active, _, _))| {
                        active.kind == piece.kind
                            && active.top_left == piece.tile
                            && active.size == piece.size
                    })
                    .count();
                if count != 1 {
                    return Err("active building piece lacks exact physical owner".into());
                }
            }
            PiecePhase::Complete if !super::physically_complete(res, piece) => {
                return Err("partial building completion lacks physical witness".into())
            }
            PiecePhase::Cancelled => return Err("live building has cancelled piece".into()),
            PiecePhase::Replacing => {
                let old = crate::runtime::jobs::space::completed_at(
                    res,
                    piece.tile,
                    CompletedLayer::Wall,
                );
                if crate::runtime::jobs::space::completion_layer(piece.kind) != CompletedLayer::Wall
                    || old.is_none_or(|kind| kind == piece.kind)
                    || res.colony.designations.get(&piece.tile) != Some(&Designation::Deconstruct)
                {
                    return Err("replacement lacks old wall/deconstruction witness".into());
                }
            }
            PiecePhase::Pending
                if super::physically_complete(res, piece)
                    || world.query::<&ColonySite>().iter().any(|(_, active)| {
                        active.kind == piece.kind
                            && active.top_left == piece.tile
                            && active.size == piece.size
                    }) =>
            {
                return Err("pending piece already owns physical work/completion".into())
            }
            _ => {}
        }
    }
    let command = super::history_validation::source(
        res,
        site,
        site.source_record,
        site.cause,
        &site.template,
        command_boundary,
    )?;
    if res.colony.command_log[site.source_record as usize].tick != site.placed_tick
        || site.placed_tick > command_boundary
    {
        return Err("building generation tick/source mismatch".into());
    }
    if site
        .pieces
        .iter()
        .any(|piece| piece.phase == PiecePhase::Replacing)
        && !matches!(command, sim_core::colony::Command::UpgradeBuilding { .. })
    {
        return Err("replacement phase outside an upgrade".into());
    }
    if matches!(command, sim_core::colony::Command::UpgradeBuilding { .. }) {
        let prior = template
            .upgrade_from
            .as_deref()
            .ok_or("upgrade target lacks predecessor")?;
        let witnessed = res.chronicle.entries().iter().enumerate().any(|(index,entry)| {
            let code = usize::try_from(entry.value).ok();
            entry.kind == sim_core::chronicle::ChronicleKind::BuildingComplete
                && code.and_then(|code| catalog.building_templates().buildings.get(code)).is_some_and(|row| row.id == prior)
                && u32::try_from(index).ok().and_then(|index|res.chronicle.cause_of(index)).is_some_and(|cause| {
                    matches!(cause,sim_core::chronicle::CauseRef::Building {site:id,record,..} if id == site.id && record < site.source_record)
                })
        });
        if !witnessed {
            return Err("current upgrade lacks completed predecessor".into());
        }
    }
    match site.status {
        BuildingStatus::Complete if site.completion_entry.is_none() => {
            Err("forged completion without history".into())
        }
        BuildingStatus::Building if site.completion_entry.is_some() => {
            Err("active generation already has completion history".into())
        }
        _ => Ok(()),
    }
}
