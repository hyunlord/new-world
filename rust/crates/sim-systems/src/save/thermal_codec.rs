//! Format18 thermal tail; exact format17 prefix and baseline writer remain unchanged.
use super::{Body, BodyRef, SaveError};
use serde::{Deserialize, Serialize};
use sim_core::colony::thermal::{DoorThermal, RoomThermal, ThermalRules, ThermalState};
use sim_engine::SimResources;

#[derive(Serialize)]
pub(super) struct BodyRefV18<'a> {
    pub body: BodyRef<'a>,
    pub thermal: &'a Option<ThermalState>,
}
#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub(super) struct BodyV18 {
    pub body: Body,
    pub thermal: Option<ThermalState>,
}

/// Initialize new physics at the saved instant without inventing historic heat.
pub(super) fn migrate(res: &mut SimResources, tick: u64) -> Result<(), SaveError> {
    if res.rules.colony.is_none() {
        return Ok(());
    }
    let rules = ThermalRules::bundled().map_err(SaveError::Decode)?;
    res.thermal = Some(ThermalState::new(rules));
    let outdoor = crate::runtime::thermal::outdoor_temperature(res)
        .ok_or_else(|| SaveError::Decode("missing thermal migration rules".into()))?;
    let mut state = res
        .thermal
        .take()
        .ok_or_else(|| SaveError::Decode("missing thermal migration state".into()))?;
    state.last_integrated_tick = Some(tick);
    state.last_exposure_tick = Some(tick);
    for (id, room) in &res.colony.rooms {
        if room.roofed {
            state.rooms.insert(
                *id,
                RoomThermal {
                    last_integrated_tick: None,
                    temperature_d10: outdoor,
                    tiles: room.tiles.clone(),
                },
            );
        }
        for tile in &room.doors {
            state.doors.entry(*tile).or_insert(DoorThermal {
                accounted_until_tick: tick,
                ..Default::default()
            });
        }
    }
    res.thermal = Some(state);
    Ok(())
}

pub(super) fn validate(res: &SimResources, tick: u64) -> Result<(), String> {
    let Some(state) = &res.thermal else {
        return Ok(());
    };
    if res.rules.colony.is_none() {
        return Err("baseline cannot own colony thermal state".into());
    }
    state.rules.validate()?;
    let materials = sim_core::colony::materials::embedded_definitions()?;
    if state
        .rules
        .material_insulation_permille
        .keys()
        .any(|id| !materials.iter().any(|d| d.id == *id))
    {
        return Err("saved thermal material is not registered".into());
    }
    let catalog = sim_core::colony::content::installed_or_bundled();
    if state
        .rules
        .apparel_insulation_d10
        .keys()
        .any(|kind| catalog.item(*kind).is_none_or(|d| d.apparel.is_none()))
    {
        return Err("saved thermal apparel is not registered".into());
    }
    if [state.last_integrated_tick, state.last_exposure_tick]
        .into_iter()
        .flatten()
        .any(|at| at > tick)
    {
        return Err("thermal hour stamp is in the future".into());
    }
    let in_bounds =
        |tile: &(u32, u32)| tile.0 < res.tile_grid.width && tile.1 < res.tile_grid.height;
    if state.doors.keys().any(|tile| !in_bounds(tile))
        || state
            .rooms
            .values()
            .any(|room| room.tiles.iter().any(|tile| !in_bounds(tile)))
        || state
            .paid_hearths
            .values()
            .any(|receipt| !in_bounds(&receipt.tile))
        || state.dwellings.keys().any(|tile| !in_bounds(tile))
    {
        return Err("saved thermal coordinate is outside the world".into());
    }
    if state
        .rooms
        .values()
        .any(|room| room.last_integrated_tick.is_some_and(|at| at > tick))
    {
        return Err("thermal room stamp is in the future".into());
    }
    if state
        .rooms
        .values()
        .any(|room| !(-100_000..=100_000).contains(&room.temperature_d10) || room.tiles.is_empty())
    {
        return Err("invalid saved thermal room".into());
    }
    for door in state.doors.values() {
        if door.accounted_until_tick > tick
            || door.open_ticks_in_hour > 60
            || door.open_until_tick > tick.saturating_add(u64::from(state.rules.door_passage_ticks))
        {
            return Err("invalid saved door interval".into());
        }
    }
    for receipt in state.paid_hearths.values() {
        if receipt.tick > tick || receipt.debited_wood > receipt.requested_wood {
            return Err("invalid paid heat receipt".into());
        }
    }
    for dwelling in state.dwellings.values() {
        if dwelling.deficient_since_tick.is_some_and(|at| at > tick)
            || dwelling
                .stage
                .is_some_and(|id| !state.rules.stages.iter().any(|s| s.id == id))
        {
            return Err("invalid saved dwelling history".into());
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn decode_test(payload: &[u8]) -> Result<(Body, Option<Option<ThermalState>>), String> {
    match super::decode::<BodyV18>(payload) {
        Ok(value) => Ok((value.body, Some(value.thermal))),
        Err(_) => super::decode::<Body>(payload).map(|body| (body, None)),
    }
}
#[cfg(test)]
pub(super) fn encode_test(
    body: &Body,
    thermal: &Option<Option<ThermalState>>,
) -> Result<Vec<u8>, String> {
    match thermal {
        Some(state) => super::encode(&(body, state)),
        None => super::encode(body),
    }
}
