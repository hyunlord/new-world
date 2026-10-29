//! Retire legacy food transport after its actual inventory became physical items.
use hecs::World;
use sim_core::components::{Agent, AgentState, PorterKind, PorterTrip, SeekTarget, TargetKind};
use sim_engine::SimResources;

pub(super) fn retire(world: &mut World, res: &mut SimResources) -> Result<(), String> {
    let mut people: Vec<_> = world
        .query::<(&Agent, &PorterTrip)>()
        .iter()
        .filter(|(_, (_, trip))| trip.kind == PorterKind::Food)
        .map(|(entity, (agent, _))| (agent.id, entity))
        .collect();
    people.sort_by_key(|(id, _)| *id);
    for (id, entity) in people {
        world
            .remove_one::<PorterTrip>(entity)
            .map_err(|error| error.to_string())?;
        let walking = world.get::<&AgentState>(entity).is_ok_and(|state| {
            matches!(*state, AgentState::Seeking { target: TargetKind::Agent(target) } if target == id)
        });
        if walking {
            world
                .insert_one(entity, AgentState::Idle)
                .map_err(|error| error.to_string())?;
            if world.get::<&SeekTarget>(entity).is_ok() {
                world
                    .remove_one::<SeekTarget>(entity)
                    .map_err(|error| error.to_string())?;
            }
        }
    }
    let ids: Vec<_> = res
        .settlement_relations
        .active
        .iter()
        .filter(|(_, record)| record.trip.kind == PorterKind::Food)
        .map(|(id, _)| *id)
        .collect();
    for id in ids {
        if let Some(mut record) = res.settlement_relations.active.remove(&id) {
            let remainder = record
                .trip
                .amount
                .saturating_sub(record.delivered)
                .saturating_sub(record.returned)
                .saturating_sub(record.lost);
            // This closes the old transport accounting, not physical food accounting:
            // Inventory conversion already retained every real piece in migrated units.
            record.lost += remainder;
            record.failed = true;
            record.end_tick = Some(res.current_tick);
            res.tallies.porter_units_lost += u64::from(remainder);
            res.tallies.porter_loads_lost += u64::from(remainder > 0);
            res.settlement_relations.trips.push(record);
        }
    }
    Ok(())
}
