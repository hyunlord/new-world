//! Existing systems retain flight, drafted combat, job progress and hearth feeding.
use super::context::*;
use sim_core::colony::people::{Drafted, Fleeing};
use sim_core::colony::Job;
use sim_core::components::AgentState;

pub(super) fn flee(c: &mut EffectContext<'_>) -> EffectResult {
    Ok(if c.world.get::<&Fleeing>(c.entity).is_ok() {
        EffectStatus::Running
    } else {
        EffectStatus::Finished
    })
}
pub(super) fn fight(c: &mut EffectContext<'_>) -> EffectResult {
    Ok(if c.world.get::<&Drafted>(c.entity).is_ok() {
        EffectStatus::Running
    } else {
        EffectStatus::Finished
    })
}
pub(super) fn job(c: &mut EffectContext<'_>) -> EffectResult {
    Ok(if c.world.get::<&Job>(c.entity).is_ok() {
        EffectStatus::Running
    } else {
        EffectStatus::Finished
    })
}
pub(super) fn owned_finish(_: &mut EffectContext<'_>) -> EffectResult {
    // Owner components and targets are never removed by a duty acknowledgement.
    Ok(EffectStatus::Finished)
}
pub(super) fn job_finish(c: &mut EffectContext<'_>) -> EffectResult {
    if c.finish_reason == FinishReason::Interrupted
        && c.effects.previous_jobs.contains_key(&c.entity)
        && c.world.get::<&Job>(c.entity).is_ok()
    {
        c.effects.job_releases.push(c.entity);
        c.effects.dirty.insert(EffectDomain::Jobs);
    }
    Ok(EffectStatus::Finished)
}
pub(super) fn idle_start(c: &mut EffectContext<'_>) -> EffectResult {
    c.set_state(AgentState::Idle)?;
    c.clear();
    Ok(EffectStatus::Running)
}
pub(super) fn idle_continue(c: &mut EffectContext<'_>) -> EffectResult {
    Ok(if c.interrupted {
        EffectStatus::Finished
    } else {
        EffectStatus::Running
    })
}
/// A targeted rest owns an Activity, so the existing movement system reaches the hearth.
pub(super) fn rest_start(c: &mut EffectContext<'_>) -> EffectResult {
    match c.target {
        ActionTarget::None => idle_start(c),
        ActionTarget::Tile(tile) => {
            let routine = c
                .routine
                .as_deref_mut()
                .ok_or(EffectError::MissingRoutine)?;
            if c.member.is_none() || c.lineage_key.is_none() {
                return Err(EffectError::MissingOwner);
            }
            if tile.0 >= c.resources.tile_grid.width || tile.1 >= c.resources.tile_grid.height {
                return Err(EffectError::StaleTarget);
            }
            if !routine.start_selected_hearth(
                c.agent,
                c.entity,
                c.lineage_key,
                c.member,
                tile,
                c.resources,
            ) {
                // The live transaction can deny a valid target after an interruption
                // or after another person's reservation fills the hearth.
                return super::needs::finish(c);
            }
            c.effects.dirty.insert(EffectDomain::RoutineOccupancy);
            c.set_state(AgentState::Seeking {
                target: sim_core::components::TargetKind::Activity,
            })?;
            c.attach(tile);
            Ok(EffectStatus::Running)
        }
        ActionTarget::Food { .. } | ActionTarget::Agent { .. } | ActionTarget::Routine { .. } => {
            Err(EffectError::WrongTarget)
        }
    }
}
pub(super) fn rest_continue(c: &mut EffectContext<'_>) -> EffectResult {
    if c.routine
        .as_deref()
        .is_some_and(|routine| routine.activity(c.agent).is_some())
    {
        super::routine::continue_running(c)
    } else {
        idle_continue(c)
    }
}
pub(super) fn rest_finish(c: &mut EffectContext<'_>) -> EffectResult {
    if c.routine
        .as_deref()
        .is_some_and(|routine| routine.activity(c.agent).is_some())
    {
        super::routine::finish(c)
    } else {
        super::needs::finish(c)
    }
}
