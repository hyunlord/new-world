//! Field orders validate the complete rectangle before changing any designation.
use hecs::World;
use sim_core::colony::content::CropKind;
use sim_core::colony::cultivation::{FieldPhase, FieldTile, FieldWork};
use sim_core::colony::{CancelReason, CommandResult, Job, RejectReason};
use sim_engine::SimResources;

fn rectangle(res: &SimResources, min: (u32, u32), max: (u32, u32)) -> Option<Vec<(u32, u32)>> {
    if min.0 > max.0
        || min.1 > max.1
        || max.0 >= res.tile_grid.width
        || max.1 >= res.tile_grid.height
    {
        return None;
    }
    crate::commands::rect_tiles(res, min.0, min.1, max.0, max.1)
}
fn cancel_at(world: &mut World, res: &mut SimResources, tiles: &[(u32, u32)]) {
    let mut entities: Vec<_> = world
        .query::<(&sim_core::components::Agent, &Job)>()
        .iter()
        .filter_map(|(e, (agent, j))| {
            crate::runtime::jobs::field::tile(j.kind)
                .filter(|t| tiles.contains(t))
                .map(|_| (agent.id, e))
        })
        .collect();
    entities.sort_by_key(|(id, _)| *id);
    for (_, entity) in entities {
        crate::runtime::jobs::cancel::cancel(world, res, entity, CancelReason::TargetGone);
    }
}
/// Designate validated outdoor soil without replacing an established crop.
pub fn zone(
    world: &mut World,
    res: &mut SimResources,
    min: (u32, u32),
    max: (u32, u32),
    crop: CropKind,
) -> CommandResult {
    let Some(def) = sim_core::colony::content::installed_or_bundled().crop(crop) else {
        return CommandResult::rejected(RejectReason::Unknown);
    };
    let Some(tiles) = rectangle(res, min, max) else {
        return CommandResult::rejected(RejectReason::OutOfMap);
    };
    if tiles.iter().any(|t| !physical_soil(res, *t)) {
        return CommandResult::rejected(RejectReason::Blocked);
    }
    if tiles
        .iter()
        .any(|t| !crate::commands::camp_reaches(res, *t))
    {
        return CommandResult::rejected(RejectReason::Unreachable);
    }
    let Some(owner) = res
        .settlements
        .values()
        .min_by_key(|s| {
            (
                crate::runtime::jobs::items::chebyshev(min, s.formation_tile),
                s.settlement_id,
            )
        })
        .map(|s| s.settlement_id)
    else {
        return CommandResult::rejected(RejectReason::Unknown);
    };
    if tiles.iter().any(|t| {
        res.cultivation.fields.get(t).is_some_and(|f| {
            f.owner != owner
                || (f.crop != crop
                    && !matches!(
                        f.phase,
                        FieldPhase::Untilled
                            | FieldPhase::Tilled
                            | FieldPhase::Fallow
                            | FieldPhase::Harvested { .. }
                    ))
        })
    }) {
        return CommandResult::rejected(RejectReason::Blocked);
    }
    let changed: Vec<_> = tiles
        .into_iter()
        .filter(|t| {
            res.cultivation
                .fields
                .get(t)
                .is_none_or(|f| !f.active || f.crop != crop)
        })
        .collect();
    if changed.is_empty() {
        return CommandResult::rejected(RejectReason::Nothing);
    }
    let first = res.cultivation.fields.is_empty();
    cancel_at(world, res, &changed);
    for tile in &changed {
        let field = res.cultivation.fields.entry(*tile).or_insert(FieldTile {
            owner,
            crop,
            phase: FieldPhase::Untilled,
            active: true,
            work: FieldWork::Auto,
            tended: false,
        });
        field.active = true;
        field.crop = crop;
        field.work = FieldWork::Auto;
        res.cultivation
            .fertility
            .entry(*tile)
            .or_insert(def.initial_fertility);
    }
    if first {
        res.cultivation.last_update_day = Some(super::day(res));
    }
    CommandResult::ok(changed.len() as u32)
}
/// Remove work designations while retaining crops, soil and consumed seeds.
pub fn erase(
    world: &mut World,
    res: &mut SimResources,
    min: (u32, u32),
    max: (u32, u32),
) -> CommandResult {
    let Some(tiles) = rectangle(res, min, max) else {
        return CommandResult::rejected(RejectReason::OutOfMap);
    };
    let changed: Vec<_> = tiles
        .into_iter()
        .filter(|t| res.cultivation.fields.get(t).is_some_and(|f| f.active))
        .collect();
    if changed.is_empty() {
        return CommandResult::rejected(RejectReason::Nothing);
    }
    cancel_at(world, res, &changed);
    for tile in &changed {
        if let Some(field) = res.cultivation.fields.get_mut(tile) {
            field.active = false;
            field.work = FieldWork::Auto;
        }
    }
    CommandResult::ok(changed.len() as u32)
}
/// Set scheduling preference; pausing releases existing field jobs and cargo.
pub fn work(
    world: &mut World,
    res: &mut SimResources,
    tile: (u32, u32),
    mode: FieldWork,
) -> CommandResult {
    let Some(field) = res.cultivation.fields.get(&tile) else {
        return CommandResult::rejected(RejectReason::Unknown);
    };
    if !field.active || field.work == mode {
        return CommandResult::rejected(RejectReason::Nothing);
    }
    if mode == FieldWork::Paused {
        cancel_at(world, res, &[tile]);
    }
    if let Some(field) = res.cultivation.fields.get_mut(&tile) {
        field.work = mode;
    }
    CommandResult::ok(1)
}

/// Steward candidates use the same physical and reachability checks as zoning.
pub fn zoneable(res: &SimResources, tile: (u32, u32)) -> bool {
    physical_soil(res, tile)
        && crate::commands::camp_reaches(res, tile)
        && !res.cultivation.fields.contains_key(&tile)
}

fn physical_soil(res: &SimResources, tile: (u32, u32)) -> bool {
    crate::runtime::jobs::items::can_hold_items(res, tile)
        && !res.colony.trees.contains_key(&tile)
        && !res.food.plants.contains_key(&tile)
        && !res.colony.zone_of_tile.contains_key(&tile)
        && super::uncovered_soil(res, tile)
}
