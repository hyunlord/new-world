//! Calendar growth and persistent soil; only jobs create or consume items.
pub mod commands;
use sim_core::colony::content::{installed_or_bundled, CropKind};
use sim_core::colony::cultivation::FieldPhase;
use sim_engine::SimResources;

/// Current colony display day, saturated to the persisted crop clock.
pub fn day(res: &SimResources) -> u32 {
    u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX)
}

/// Whether today lies inside any authored half-open sowing window.
pub fn sowing_window(res: &SimResources, crop: CropKind) -> bool {
    let (_, season, offset, _) = res.rules.calendar.date_of_day(u64::from(day(res)));
    installed_or_bundled().crop(crop).is_some_and(|def| {
        def.sow_windows.iter().any(|w| {
            u32::from(w.season) == season && offset >= w.start_day && offset < w.end_day_exclusive
        })
    })
}

/// Land retained by a field designation or an established crop, even after erasure.
pub(crate) fn occupies(res: &SimResources, tile: (u32, u32)) -> bool {
    res.cultivation.fields.get(&tile).is_some_and(|field| {
        field.active
            || matches!(
                field.phase,
                FieldPhase::Sown { .. } | FieldPhase::Growing { .. } | FieldPhase::Ripe { .. }
            )
    })
}

/// Runs once per elapsed calendar day, including erased and paused fields.
pub fn update(res: &mut SimResources) {
    if res.cultivation.fields.is_empty() {
        return;
    }
    let today = day(res);
    let previous = res.cultivation.last_update_day.unwrap_or(today);
    if res.cultivation.last_update_day == Some(today) {
        return;
    }
    let covered: std::collections::BTreeSet<_> = res
        .cultivation
        .fields
        .keys()
        .copied()
        .filter(|tile| !uncovered_soil(res, *tile))
        .collect();
    for at in previous.saturating_add(1)..=today {
        let (_, season, offset, _) = res.rules.calendar.date_of_day(u64::from(at));
        for (tile, field) in &mut res.cultivation.fields {
            let Some(def) = installed_or_bundled().crop(field.crop) else {
                continue;
            };
            let failure = if covered.contains(tile) {
                Some(1)
            } else if def.frost_sensitive && season == 3 && offset == 0 {
                Some(0)
            } else {
                None
            };
            if failure.is_some()
                && matches!(
                    field.phase,
                    FieldPhase::Sown { .. } | FieldPhase::Growing { .. } | FieldPhase::Ripe { .. }
                )
            {
                field.phase = FieldPhase::Fallow;
                res.chronicle.push(
                    sim_core::chronicle::ChronicleEntry::new(
                        res.current_tick,
                        sim_core::chronicle::ChronicleKind::CropFailure,
                    )
                    .settlement(Some(field.owner))
                    .cause(failure.unwrap_or(0))
                    .value(i64::from(field.crop.code())),
                );
            }
            field.phase = match field.phase {
                FieldPhase::Sown { planted_day } | FieldPhase::Growing { planted_day, .. } => {
                    let growth_days = at.saturating_sub(planted_day);
                    if growth_days >= def.growth_days {
                        FieldPhase::Ripe { ripe_day: at }
                    } else {
                        FieldPhase::Growing {
                            planted_day,
                            growth_days,
                        }
                    }
                }
                FieldPhase::Harvested { harvest_day } if at > harvest_day => FieldPhase::Fallow,
                phase => phase,
            };
            if matches!(field.phase, FieldPhase::Fallow) {
                let soil = res
                    .cultivation
                    .fertility
                    .entry(*tile)
                    .or_insert(def.initial_fertility);
                *soil = soil.saturating_add(def.fallow_recovery_per_day).min(1000);
            }
        }
    }
    res.cultivation.last_update_day = Some(today);
}

#[cfg(test)]
mod tests;

/// The exact practitioner gate used by field assignment and completion.
pub fn knowledge_mask(crop: CropKind) -> u32 {
    installed_or_bundled()
        .crop(crop)
        .and_then(|def| def.knowledge)
        .map_or(0, |code| 1u32.checked_shl(u32::from(code)).unwrap_or(0))
}
/// Ownership is the same nearest-home rule used when designating fields.
pub fn seed_belongs_to(
    res: &SimResources,
    owner: sim_core::components::SettlementId,
    tile: (u32, u32),
) -> bool {
    let Some(home) = res.settlements.get(&owner) else {
        return false;
    };
    res.nav
        .reaches(res.nav.region_of(home.formation_tile), tile)
        && res
            .settlements
            .values()
            .min_by_key(|s| {
                (
                    crate::runtime::jobs::items::chebyshev(tile, s.formation_tile),
                    s.settlement_id,
                )
            })
            .is_some_and(|s| s.settlement_id == owner)
}
/// Physical reachable ground units, including reserved seeds; not a job availability claim.
pub fn ground_seed_count(
    res: &SimResources,
    owner: sim_core::components::SettlementId,
    crop: CropKind,
) -> u64 {
    let Some(def) = installed_or_bundled().crop(crop) else {
        return 0;
    };
    res.colony
        .ground_items
        .iter()
        .filter(|(tile, stack)| stack.kind == def.seed_item && seed_belongs_to(res, owner, **tile))
        .map(|(_, stack)| u64::from(stack.count))
        .sum()
}

/// Existing construction can cover a field; this predicate is shared by admission and work.
pub fn uncovered_soil(res: &SimResources, tile: (u32, u32)) -> bool {
    tile.0 < res.tile_grid.width
        && tile.1 < res.tile_grid.height
        && res.tile_grid.outdoor[res.tile_grid.idx(tile.0, tile.1)]
        && res.tile_grid.floor_material[res.tile_grid.idx(tile.0, tile.1)].is_none()
        && !crate::runtime::jobs::items::under_building(res, tile)
        && !crate::runtime::rooms::roofed(res, tile)
}
