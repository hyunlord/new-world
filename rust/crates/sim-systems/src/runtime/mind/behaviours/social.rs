//! Social chase lifecycle effects and missing-partner cleanup.
use super::context::*;
use sim_core::causal::CausalEvent;
use sim_core::components::{AgentState, TargetKind};

fn partner(c: &EffectContext<'_>) -> Result<(u64, (u32, u32)), EffectError> {
    match c.target {
        ActionTarget::Agent { agent, tile } if agent != c.agent => Ok((agent, tile)),
        ActionTarget::None => Err(EffectError::MissingTarget),
        ActionTarget::Tile(_) | ActionTarget::Routine { .. } | ActionTarget::Agent { .. } => {
            Err(EffectError::WrongTarget)
        }
    }
}
pub(super) fn start(c: &mut EffectContext<'_>) -> EffectResult {
    let (peer, tile) = partner(c)?;
    c.set_state(AgentState::Seeking {
        target: TargetKind::Agent(peer),
    })?;
    c.effects.social_seek_entities.insert(c.entity);
    if c.resources.rules.agent.social_seek.is_some() {
        c.resources
            .social_chase_since
            .insert(c.agent, c.resources.current_tick);
    }
    if let Some(routine) = c.routine.as_deref_mut() {
        routine.invite(peer, c.world);
        c.effects.dirty.insert(EffectDomain::Invitations);
        c.effects.dirty.insert(EffectDomain::RoutineOccupancy);
    }
    c.attach(tile);
    Ok(EffectStatus::Running)
}
pub(super) fn continue_running(c: &mut EffectContext<'_>) -> EffectResult {
    if c.interrupted {
        return finish(c);
    }
    let (peer, tile) = match partner(c) {
        Ok(value) => value,
        Err(EffectError::MissingTarget) => {
            if let Some(rule) = c.resources.rules.agent.social_seek {
                c.resources.social_retry_after.insert(
                    c.agent,
                    c.resources
                        .current_tick
                        .saturating_add(rule.retry_cooldown_ticks),
                );
            }
            return finish(c);
        }
        Err(error) => return Err(error),
    };
    match c.state()? {
        AgentState::Seeking {
            target: TargetKind::Agent(id),
        } if id == peer => {
            if let Some(rule) = c.resources.rules.agent.social_seek {
                let tick = c.resources.current_tick;
                let since = *c
                    .resources
                    .social_chase_since
                    .entry(c.agent)
                    .or_insert(tick);
                if tick.saturating_sub(since) >= rule.max_chase_ticks {
                    c.resources
                        .social_retry_after
                        .insert(c.agent, tick.saturating_add(rule.retry_cooldown_ticks));
                    return finish(c);
                }
            }
            if c.social_handshake {
                if c.agent < peer {
                    let id = c.resources.issue_event_id();
                    let tick = c.resources.current_tick;
                    c.resources.causal_log.push(
                        c.position.y * c.resources.tile_grid.width + c.position.x,
                        CausalEvent::SocialInteractionStarted {
                            id,
                            parent: c.cause,
                            agents: (c.agent, peer),
                            position: (c.position.x, c.position.y),
                            tick,
                        },
                    );
                }
                c.resources.social_chase_since.remove(&c.agent);
                c.set_state(AgentState::Consuming {
                    target: TargetKind::Agent(peer),
                })?;
                c.clear();
            } else {
                c.attach(tile);
            }
            Ok(EffectStatus::Running)
        }
        AgentState::Consuming {
            target: TargetKind::Agent(id),
        } if id == peer => Ok(EffectStatus::Running),
        AgentState::Idle | AgentState::Seeking { .. } | AgentState::Consuming { .. } => {
            Ok(EffectStatus::Finished)
        }
    }
}
pub(super) fn finish(c: &mut EffectContext<'_>) -> EffectResult {
    c.resources.social_chase_since.remove(&c.agent);
    c.effects.social_seek_entities.remove(&c.entity);
    super::needs::finish(c)
}
