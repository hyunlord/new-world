//! Validate the versioned field and bounded-origin tail before save or load.
use sim_core::colony::content::installed;
use sim_core::colony::cultivation::{FieldPhase, OriginShare};
use sim_engine::SimResources;

pub(super) fn validate(world: &hecs::World, res: &SimResources) -> Result<(), String> {
    let state = &res.cultivation;
    let crafting: std::collections::BTreeSet<_> = world
        .query::<(&sim_core::components::Agent, &sim_core::colony::Job)>()
        .iter()
        .filter_map(|(_, (agent, job))| {
            (matches!(job.kind, sim_core::colony::JobKind::Craft { .. })
                && matches!(job.stage, sim_core::colony::JobStage::Working { .. }))
            .then_some(agent.id)
        })
        .collect();
    if state
        .craft_origins
        .keys()
        .any(|agent| !crafting.contains(agent))
    {
        return Err("cultivated craft origin has no live working crafter".into());
    }

    let catalog = installed().map_err(|error| error.to_string())?;
    let inside = |tile: (u32, u32)| tile.0 < res.tile_grid.width && tile.1 < res.tile_grid.height;
    for (&tile, field) in &state.fields {
        if !inside(tile) || field.owner == 0 || catalog.crop(field.crop).is_none() {
            return Err(format!("invalid saved field at {tile:?}"));
        }
        let day = match field.phase {
            FieldPhase::Sown { planted_day } | FieldPhase::Growing { planted_day, .. } => {
                Some(planted_day)
            }
            FieldPhase::Ripe { ripe_day } => Some(ripe_day),
            FieldPhase::Harvested { harvest_day } => Some(harvest_day),
            _ => None,
        };
        if day.is_some_and(|day| state.last_update_day.is_some_and(|last| day > last)) {
            return Err(format!("field date is after calendar update at {tile:?}"));
        }
    }
    if state
        .fertility
        .iter()
        .any(|(&tile, &soil)| !inside(tile) || soil > 1000)
    {
        return Err("invalid saved field fertility".into());
    }
    let invalid_share =
        |share: &OriginShare| share.lower > share.upper || share.upper > OriginShare::SCALE;
    if state
        .food_origins
        .values()
        .chain(state.craft_origins.values())
        .chain(state.drying_origins.iter())
        .any(invalid_share)
    {
        return Err("invalid saved cultivated food fraction".into());
    }
    if state.drying_origins.len() != res.food.pending_drying.len() {
        return Err("cultivated drying origins do not align with pending batches".into());
    }
    for totals in state.eaten_by_year.values() {
        if totals.field_lower > totals.field_upper
            || u128::from(totals.field_upper)
                > u128::from(totals.units) * u128::from(OriginShare::SCALE)
        {
            return Err("invalid cultivated meal totals".into());
        }
    }
    Ok(())
}
