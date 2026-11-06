//! Recovery rest keeps its real seat until authored health/needs guards release it.
use super::context::*;
use sim_core::components::{AgentState, TargetKind, Temperament};

pub(super) fn continue_running(c: &mut EffectContext<'_>) -> EffectResult {
    if c.interrupted {
        return super::owned::rest_finish(c);
    }
    let Some(activity) = c.routine.as_deref().and_then(|r| r.activity(c.agent)) else {
        return super::needs::finish(c);
    };
    // The pure target provider has rechecked heat, reachability and capacity.
    // A vanished/cooled destination ends through the existing reservation owner.
    if c.tile_target()? != activity.place {
        return super::owned::rest_finish(c);
    }
    if (c.position.x, c.position.y) == activity.place {
        if activity.arrived_tick.is_none() {
            let temperament = c.world.get::<&Temperament>(c.entity).ok().map(|t| *t);
            if let Some(routine) = c.routine.as_deref_mut() {
                routine.arrive(c.agent, c.entity, temperament.as_ref());
            }
            c.effects.mark_dirty(EffectDomain::RoutineOccupancy);
        }
        c.set_state(AgentState::Consuming {
            target: TargetKind::Activity,
        })?;
        c.clear();
    } else {
        c.set_state(AgentState::Seeking {
            target: TargetKind::Activity,
        })?;
        c.attach(activity.place);
    }
    // Leisure deadlines do not represent recovery. The authored policy supplies
    // full-health release and critical hunger/thirst interruption, without HP writes.
    Ok(EffectStatus::Running)
}
