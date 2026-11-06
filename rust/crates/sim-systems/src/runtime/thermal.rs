//! Colony thermal owner: after geometry152, hourly heat/exposure and per-tick Cold.
//! Reads physical rooms, paid hearth receipts and apparel; writes only colony thermal/Cold.
use hecs::World;
use sim_core::colony::people::Apparel;
use sim_core::colony::thermal::{PaidHeat, RoomThermal, ThermalState};
use sim_core::components::{Agent, Child, Cold, Position, SettlementId};
use sim_core::rooms::Tile;
use sim_engine::SimResources;
use std::collections::BTreeMap;
mod doors;
mod dwelling;
mod food;
mod heat;
pub use doors::note_door_passage;
pub use dwelling::{dwelling_view, DwellingView};

/// Activate authored thermal behavior only for the colony.
pub fn initialize(res: &mut SimResources) {
    if !res.thermal_enabled {
        res.thermal = None;
        return;
    }
    if res.rules.colony.is_some() && res.thermal.is_none() {
        res.thermal = Some(ThermalState::new(
            sim_core::colony::content::installed_or_bundled()
                .thermal_rules()
                .clone(),
        ));
    }
}

/// Actual fuel receipt, recorded after the existing single withdrawal.
pub fn record_paid_heat(res: &mut SimResources, owner: SettlementId, receipt: PaidHeat) {
    initialize(res);
    if let Some(state) = res.thermal.as_mut() {
        if state
            .paid_hearths
            .get(&owner)
            .is_none_or(|old| old.tick != receipt.tick)
        {
            state.paid_hearths.insert(owner, receipt);
            state.view_version = state.view_version.wrapping_add(1);
        }
    }
}

/// Current authored outdoor temperature, including the active cold pressure.
pub fn outdoor_temperature(res: &SimResources) -> Option<i32> {
    let rules = &res.thermal.as_ref()?.rules;
    let season = crate::runtime::resource_regen::season_at(res).map_or(0, |(s, _)| s);
    let minute = res
        .current_tick
        .saturating_add(res.rules.clock_offset_ticks)
        % res.ticks_per_day.max(1);
    let hour = usize::try_from(minute.saturating_mul(24) / res.ticks_per_day.max(1)).ok()?;
    let snap = crate::runtime::story::active_mechanic(
        res,
        sim_core::colony::content::PressureMechanic::Cold,
    )
    .is_some();
    Some(
        rules.season_outdoor_d10[season.min(3)]
            + rules.hour_offset_d10[hour.min(23)]
            + if snap { rules.cold_snap_offset_d10 } else { 0 },
    )
}

/// A roofed interior's committed temperature, otherwise actual outdoor.
pub fn place_temperature(res: &SimResources, tile: Tile) -> Option<i32> {
    let state = res.thermal.as_ref()?;
    if let Some(room) = crate::runtime::rooms::room_at(res, tile).filter(|room| room.roofed) {
        if let Some(value) = state.rooms.get(&room.id) {
            return Some(value.temperature_d10);
        }
    }
    outdoor_temperature(res)
}

/// Predict this hourly room integration without mutating state or charging fuel.
/// `excluded_owner` omits that owner's existing receipt when deciding its heat demand.
pub fn preview_room_temperature(
    world: &World,
    res: &SimResources,
    room: u32,
    excluded_owner: Option<SettlementId>,
) -> Option<i32> {
    heat::preview(world, res, room, excluded_owner)
}

/// Native felt temperature; worn condition remains owned by hourly apparel wear.
pub fn felt_temperature(res: &SimResources, apparel: Option<&Apparel>, tile: Tile) -> Option<i32> {
    let state = res.thermal.as_ref()?;
    let insulation = apparel
        .and_then(|a| state.rules.apparel_insulation_d10.get(&a.top.item()))
        .copied()
        .unwrap_or(0);
    place_temperature(res, tile).map(|temperature| temperature + insulation)
}

/// Whether this room has a fully paid positive primary hearth in the current hour.
pub fn room_paid(res: &SimResources, room: u32) -> bool {
    res.thermal.as_ref().is_some_and(|state| {
        state.paid_hearths.values().any(|paid| {
            paid.tick / 60 == res.current_tick / 60
                && paid.requested_wood > 0
                && paid.requested_wood == paid.debited_wood
                && res.building_kinds.get(&paid.tile)
                    == Some(&sim_core::components::BuildingKind::Hearth)
                && res.colony.room_of_tile.get(&paid.tile) == Some(&room)
        })
    })
}

/// Existing RoomSystem's post-geometry hook; no additional registered system slot.
pub fn update_after_rooms(world: &mut World, res: &mut SimResources) {
    initialize(res);
    let Some(outdoor) = outdoor_temperature(res) else {
        return;
    };
    let Some(mut state) = res.thermal.take() else {
        return;
    };
    let geometry_changed = state.rooms.len()
        != res.colony.rooms.values().filter(|r| r.roofed).count()
        || res.colony.rooms.values().filter(|r| r.roofed).any(|room| {
            state
                .rooms
                .get(&room.id)
                .is_none_or(|old| old.tiles != room.tiles)
        });
    if geometry_changed {
        state.rooms = heat::remap(res, &state, outdoor);
    }
    state
        .doors
        .retain(|&(x, y), _| res.tile_grid.is_door(res.tile_grid.idx(x, y)));
    state
        .paid_hearths
        .retain(|owner, _| res.settlements.contains_key(owner));
    doors::advance(&mut state, res.current_tick);
    let hourly =
        res.current_tick.is_multiple_of(60) && state.last_integrated_tick != Some(res.current_tick);
    if hourly {
        heat::integrate(world, res, &mut state, outdoor);
    }
    state.view_version = state.view_version.wrapping_add(1);
    res.thermal = Some(state);
    let threshold = res
        .thermal
        .as_ref()
        .map_or(0, |s| s.rules.cold_threshold_d10);
    let mut rows: Vec<_> = world
        .query::<(&Agent, &Position, Option<&Apparel>, Option<&Cold>)>()
        .iter()
        .map(|(e, (a, p, apparel, cold))| {
            (
                a.id,
                e,
                felt_temperature(res, apparel, (p.x, p.y)).is_some_and(|t| t < threshold),
                cold.is_some(),
            )
        })
        .collect();
    rows.sort_by_key(|row| row.0);
    let mut exposed = Vec::new();
    for (id, e, cold, had) in rows {
        if cold {
            exposed.push((id, e));
        }
        match (cold, had) {
            (true, false) => {
                let _ = world.insert_one(e, Cold);
            }
            (false, true) => {
                let _ = world.remove_one::<Cold>(e);
            }
            _ => {}
        }
    }
    if hourly
        && res
            .thermal
            .as_ref()
            .is_some_and(|s| s.last_exposure_tick != Some(res.current_tick))
    {
        if let Some(state) = res.thermal.as_mut() {
            state.last_exposure_tick = Some(res.current_tick);
        }
        res.tallies.cold_member_hours += u64::try_from(exposed.len()).unwrap_or(0);
        if let Some(rule) = res.rules.society.exposure {
            let snap = crate::runtime::story::active_mechanic(
                res,
                sim_core::colony::content::PressureMechanic::Cold,
            );
            crate::runtime::settlement::fire::expose(
                world,
                res,
                &rule,
                &exposed,
                res.current_tick,
                snap,
            );
        }
    }
    dwelling::update(world, res);
}
