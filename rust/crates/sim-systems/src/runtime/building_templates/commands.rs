//! Expand a template into persistent work; existing placement commands own physical pieces.
pub use super::preview::preview;
use hecs::World;
use sim_core::colony::building_site::{
    BuildCause, BuildingPiece, BuildingSite, BuildingStatus, PiecePhase, Tile,
};
use sim_core::colony::content::{installed_or_bundled, Catalog};
use sim_core::colony::knowledge::{self, ContentStatus};
use sim_core::colony::{CommandResult, RejectReason};
use sim_engine::SimResources;

/// Shared availability check for UI, steward, and the authoritative command.
pub fn known(world: &World, required: &[String], catalog: &Catalog) -> bool {
    required.iter().all(|name| {
        catalog.knowledge_defs().id_of(name).is_some_and(|id| {
            crate::runtime::knowledge::anyone_practises(world, knowledge::bit(id))
        })
    })
}

/// Exact rule-adjusted physical material needs, shared with steward and UI affordability.
pub fn plan_materials(
    res: &SimResources,
    plan: &[sim_core::colony::building_template::PlannedPiece],
) -> Result<std::collections::BTreeMap<sim_core::colony::ItemKind, u32>, RejectReason> {
    let rule = res.rules.colony.ok_or(RejectReason::NoColony)?;
    let mut materials = std::collections::BTreeMap::<sim_core::colony::ItemKind, u32>::new();
    for piece in plan {
        for need in piece.kind.plan(&rule).1.used() {
            let total = materials.entry(need.kind).or_default();
            *total = total
                .checked_add(u32::from(need.needed))
                .ok_or(RejectReason::Unknown)?;
        }
    }
    Ok(materials)
}

/// Exact selected-plan requirements; authored materials validate only default content costs.
pub fn material_requirements(
    res: &SimResources,
    id: &str,
    station: Option<&str>,
    rotation: u8,
) -> Result<std::collections::BTreeMap<sim_core::colony::ItemKind, u32>, RejectReason> {
    let catalog = installed_or_bundled();
    let template = catalog
        .building_templates()
        .get(id)
        .ok_or(RejectReason::Unknown)?;
    if template.status != ContentStatus::Connected {
        return Err(RejectReason::NotYet);
    }
    let plan = template
        .expand(catalog, station, rotation)
        .map_err(|_| RejectReason::Unknown)?;
    plan_materials(res, &plan)
}

/// Create the entire ordered plan atomically; activation uses existing physical jobs.
pub fn build(
    world: &mut World,
    res: &mut SimResources,
    id: &str,
    anchor: Tile,
    rotation: u8,
    station: Option<&str>,
    cause: BuildCause,
) -> CommandResult {
    let admission_tick = res.current_tick;
    build_at(
        world,
        res,
        id,
        anchor,
        rotation,
        station,
        cause,
        admission_tick,
    )
}

/// Use the logged admission tick when a command arrives between simulation ticks.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_at(
    world: &mut World,
    res: &mut SimResources,
    id: &str,
    anchor: Tile,
    rotation: u8,
    station: Option<&str>,
    cause: BuildCause,
    admission_tick: u64,
) -> CommandResult {
    let ready = match super::preview::prepare(world, res, id, anchor, rotation, station) {
        Ok(ready) => ready,
        Err(reason) => return CommandResult::rejected(reason),
    };
    let super::preview::BuildPlan {
        plan,
        footprint,
        next_site,
        next_piece,
        source_record,
        station,
    } = ready;
    let right = anchor.0 + footprint.0;
    let bottom = anchor.1 + footprint.1;
    let state = &res.colony.buildings;
    let mut pieces = Vec::with_capacity(plan.len());
    for (index, piece) in plan.into_iter().enumerate() {
        let Ok(index) = u32::try_from(index) else {
            return CommandResult::rejected(RejectReason::Unknown);
        };
        pieces.push(BuildingPiece {
            id: state.next_piece_id + index,
            kind: piece.kind,
            tile: (anchor.0 + piece.offset.0, anchor.1 + piece.offset.1),
            size: piece.size,
            phase: PiecePhase::Pending,
        });
    }
    let cause = match cause {
        BuildCause::Steward(_) => res
            .colony
            .buildings
            .housing_zones
            .iter()
            .find_map(|(&zone, tiles)| {
                (anchor.1..bottom)
                    .all(|y| (anchor.0..right).all(|x| tiles.contains(&(x, y))))
                    .then_some(BuildCause::Zone(zone))
            })
            .unwrap_or(cause),
        BuildCause::Command | BuildCause::Zone(_) => cause,
    };
    let site_id = state.next_site_id;
    let state = &mut res.colony.buildings;
    state.next_site_id = next_site;
    state.next_piece_id = next_piece;
    state.sites.insert(
        site_id,
        BuildingSite {
            id: site_id,
            template: id.to_owned(),
            station,
            anchor,
            rotation,
            footprint,
            pieces,
            cause,
            source_record,
            placed_tick: admission_tick,
            status: BuildingStatus::Building,
            completion_entry: None,
        },
    );
    res.colony.touch(false);
    super::advance(world, res);
    CommandResult::ok(1)
}

/// Cancel remaining work through the existing refund/cancellation path; finished pieces remain.
pub fn cancel(world: &mut World, res: &mut SimResources, id: u32) -> CommandResult {
    let Some(site) = res.colony.buildings.sites.get(&id).cloned() else {
        return CommandResult::rejected(RejectReason::Nothing);
    };
    if site.status == BuildingStatus::Complete
        || !site.pieces.iter().any(|piece| {
            matches!(
                piece.phase,
                PiecePhase::Pending | PiecePhase::Active | PiecePhase::Replacing
            )
        })
    {
        return CommandResult::rejected(RejectReason::Nothing);
    }
    let mut site = site;
    cancel_plan(world, res, &mut site);
    res.colony.buildings.sites.insert(id, site);
    CommandResult::ok(1)
}

/// Cancel this exact current generation, including pieces activated earlier in this same pass.
pub(super) fn cancel_plan(world: &mut World, res: &mut SimResources, site: &mut BuildingSite) {
    for piece in &site.pieces {
        if matches!(piece.phase, PiecePhase::Active | PiecePhase::Replacing) {
            crate::commands::apply_template_piece(
                world,
                res,
                sim_core::colony::Command::CancelArea {
                    x0: piece.tile.0,
                    y0: piece.tile.1,
                    x1: piece.tile.0 + piece.size.0 - 1,
                    y1: piece.tile.1 + piece.size.1 - 1,
                },
            );
        }
    }
    site.status = BuildingStatus::Cancelled;
    for piece in &mut site.pieces {
        if piece.phase != PiecePhase::Complete {
            piece.phase = PiecePhase::Cancelled;
        }
    }
    res.colony.touch(false);
}
