//! Same-footprint upgrades retain floors and beds and use ordinary wall deconstruction.
use hecs::World;
use sim_core::colony::building_site::{BuildCause, BuildingStatus, PiecePhase};
use sim_core::colony::building_template::PlannedPiece;
use sim_core::colony::{CommandResult, ItemKind, RejectReason};
use sim_engine::SimResources;
use std::collections::BTreeMap;

/// Query exact upgrade hard gates without deconstruction, allocator writes, or commands.
pub fn check(world: &World, res: &SimResources, id: u32) -> Result<(), RejectReason> {
    super::upgrade_plan::prepare(world, res, id).map(|_| ())
}

/// Price only changed pieces; retained floors, beds and doors consume no new supplies.
pub fn material_requirements(
    world: &World,
    res: &SimResources,
    id: u32,
) -> Result<BTreeMap<ItemKind, u32>, RejectReason> {
    let ready = super::upgrade_plan::prepare(world, res, id)?;
    let changed: Vec<_> = ready
        .pieces
        .iter()
        .filter(|piece| piece.phase == PiecePhase::Replacing)
        .map(|piece| PlannedPiece {
            kind: piece.kind,
            offset: (
                piece.tile.0 - ready.site.anchor.0,
                piece.tile.1 - ready.site.anchor.1,
            ),
            size: piece.size,
        })
        .collect();
    super::commands::plan_materials(res, &changed)
}

/// Execute the exact read-only prepared generation through existing physical commands.
pub fn upgrade(
    world: &mut World,
    res: &mut SimResources,
    id: u32,
    cause: BuildCause,
) -> CommandResult {
    let admission_tick = res.current_tick;
    upgrade_at(world, res, id, cause, admission_tick)
}

/// Stamp a new generation with the same boundary as its accepted command record.
pub(crate) fn upgrade_at(
    world: &mut World,
    res: &mut SimResources,
    id: u32,
    cause: BuildCause,
    admission_tick: u64,
) -> CommandResult {
    let ready = match super::upgrade_plan::prepare(world, res, id) {
        Ok(ready) => ready,
        Err(reason) => return CommandResult::rejected(reason),
    };
    let result = crate::commands::designate_template_walls(res, &ready.walls);
    if !result.accepted {
        return result;
    }
    res.colony.buildings.next_piece_id = ready.next_piece;
    let mut site = ready.site;
    site.template = ready.target;
    site.station = ready.station;
    site.pieces = ready.pieces;
    site.cause = cause;
    site.source_record = ready.source_record;
    site.placed_tick = admission_tick;
    site.completion_entry = None;
    site.status = BuildingStatus::Building;
    res.colony.buildings.sites.insert(id, site);
    res.colony.touch(false);
    super::advance(world, res);
    CommandResult::ok(1)
}
