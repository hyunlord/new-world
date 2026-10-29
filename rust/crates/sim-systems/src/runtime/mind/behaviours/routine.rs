//! Registered routine lifecycle effects and hearth admission.
use super::context::*;
use sim_core::components::{AgentState, TargetKind, Temperament};

pub(super) fn start(c: &mut EffectContext<'_>) -> EffectResult {
    let (kind, tile) = match c.target {
        ActionTarget::Routine { kind, tile } => (kind, tile),
        ActionTarget::None => return Err(EffectError::MissingTarget),
        ActionTarget::Food { .. } | ActionTarget::Tile(_) | ActionTarget::Agent { .. } => {
            return Err(EffectError::WrongTarget)
        }
    };
    let routine = c
        .routine
        .as_deref_mut()
        .ok_or(EffectError::MissingRoutine)?;
    let started = routine.start_selected_kind(
        c.agent,
        c.entity,
        c.lineage_key,
        c.member,
        kind,
        tile,
        c.resources,
    );
    c.effects.dirty.insert(EffectDomain::RoutineOccupancy);
    if started {
        c.set_state(AgentState::Seeking {
            target: TargetKind::Activity,
        })?;
        c.attach(tile);
        Ok(EffectStatus::Running)
    } else {
        Ok(EffectStatus::Finished)
    }
}
pub(super) fn continue_running(c: &mut EffectContext<'_>) -> EffectResult {
    if c.interrupted {
        return finish(c);
    }
    let state = c.state()?;
    let temperament = c.world.get::<&Temperament>(c.entity).ok().map(|t| *t);
    let routine = c
        .routine
        .as_deref_mut()
        .ok_or(EffectError::MissingRoutine)?;
    let Some(activity) = routine.activity(c.agent) else {
        return super::needs::finish(c);
    };
    let walking = matches!(
        state,
        AgentState::Seeking {
            target: TargetKind::Activity
        }
    ) && (c.position.x, c.position.y) != activity.place;
    if let Some(completed) = routine.deadline(
        c.agent,
        c.member,
        walking,
        c.lineage_key,
        temperament.as_ref(),
    ) {
        routine.end_selected_node(c.agent, c.entity, completed, c.node_random_key);
        c.effects.dirty.insert(EffectDomain::RoutineOccupancy);
        return super::needs::finish(c);
    }
    match state {
        AgentState::Seeking {
            target: TargetKind::Activity,
        } => {
            if (c.position.x, c.position.y) == activity.place {
                routine.arrive(c.agent, c.entity, temperament.as_ref());
                c.effects.dirty.insert(EffectDomain::RoutineOccupancy);
                c.set_state(AgentState::Consuming {
                    target: TargetKind::Activity,
                })?;
                c.clear();
            } else {
                c.attach(activity.place);
            }
            Ok(EffectStatus::Running)
        }
        AgentState::Consuming {
            target: TargetKind::Activity,
        } => Ok(EffectStatus::Running),
        AgentState::Idle | AgentState::Seeking { .. } | AgentState::Consuming { .. } => {
            Ok(EffectStatus::Finished)
        }
    }
}
pub(super) fn finish(c: &mut EffectContext<'_>) -> EffectResult {
    if let Some(routine) = c.routine.as_deref_mut() {
        match c.finish_reason {
            FinishReason::Completed => {
                routine.end_selected_node(c.agent, c.entity, true, c.node_random_key)
            }
            FinishReason::Interrupted => routine.interrupt(c.agent, c.entity),
        }
        c.effects.dirty.insert(EffectDomain::RoutineOccupancy);
    }
    super::needs::finish(c)
}

/// Failed entry owns the cooldown and observation but never an activity reservation.
pub(super) fn no_place(c: &mut EffectContext<'_>) -> EffectResult {
    let child = c
        .world
        .get::<&sim_core::components::Child>(c.entity)
        .is_ok();
    let elder = c
        .world
        .get::<&sim_core::components::Elder>(c.entity)
        .is_ok();
    let temperament = c
        .world
        .get::<&Temperament>(c.entity)
        .ok()
        .map(|value| *value);
    c.routine
        .as_deref_mut()
        .ok_or(EffectError::MissingRoutine)?
        .deny_selected_node(
            c.agent,
            c.entity,
            child,
            elder,
            temperament.as_ref(),
            c.node_random_key,
        );
    c.effects.dirty.insert(EffectDomain::RoutineOccupancy);
    super::needs::finish(c)
}
