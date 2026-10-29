//! Physical food lifecycle: arrive first, then consume an actual selected owner.
use super::context::*;
use sim_core::components::{AgentState, TargetKind};

pub(super) fn start(c: &mut EffectContext<'_>) -> EffectResult {
    let ActionTarget::Food { tile, .. } = c.target else {
        return super::needs::finish(c);
    };
    c.set_state(AgentState::Seeking {
        target: TargetKind::Food,
    })?;
    c.attach(tile);
    Ok(EffectStatus::Running)
}

pub(super) fn advance(c: &mut EffectContext<'_>) -> EffectResult {
    if c.interrupted {
        return super::needs::finish(c);
    }
    let ActionTarget::Food { tile, owner } = c.target else {
        return super::needs::finish(c);
    };
    let position = (c.position.x, c.position.y);
    if position != tile {
        c.set_state(AgentState::Seeking {
            target: TargetKind::Food,
        })?;
        c.attach(tile);
        return Ok(EffectStatus::Running);
    }
    match c.state()? {
        AgentState::Seeking {
            target: TargetKind::Food,
        } => {
            c.set_state(AgentState::Consuming {
                target: TargetKind::Food,
            })?;
            Ok(EffectStatus::Running)
        }
        AgentState::Consuming {
            target: TargetKind::Food,
        } => {
            let day = crate::runtime::food::day(c.resources);
            if crate::runtime::food::meals::consume(c.world, c.resources, c.entity, owner, day)
                .is_err()
            {
                return super::needs::finish(c);
            }
            c.effects.completed_meals.push(c.agent);
            c.effects.dirty.insert(EffectDomain::FoodSources);
            c.effects.dirty.insert(EffectDomain::Needs);
            c.effects.dirty.insert(EffectDomain::Targets);
            // One physical piece completes one action, preserving the Idle boundary.
            // Further hunger is reconsidered by authored policy on the next tick.
            super::needs::finish(c)
        }
        AgentState::Idle | AgentState::Seeking { .. } | AgentState::Consuming { .. } => {
            super::needs::finish(c)
        }
    }
}
