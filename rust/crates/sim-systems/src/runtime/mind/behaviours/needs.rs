//! Food, water and sleep lifecycle effects after authored policy selection.
use super::context::*;
use crate::runtime::agent_actions::consume;
use sim_core::components::{AgentState, Hunger, Sleep, TargetKind, Thirst};

fn start(context: &mut EffectContext<'_>, kind: TargetKind) -> EffectResult {
    // An earlier person's effect can exhaust the source advertised at tick start.
    // End this admission attempt; next tick rebuilds availability before selection.
    if context.target == ActionTarget::None {
        return finish(context);
    }
    let tile = context.tile_target()?;
    context.set_state(AgentState::Seeking { target: kind })?;
    context.attach(tile);
    Ok(EffectStatus::Running)
}
pub(super) fn eat_start(c: &mut EffectContext<'_>) -> EffectResult {
    if c.resources.rules.colony.is_some() {
        return super::food::start(c);
    }
    start(c, TargetKind::Food)
}
pub(super) fn drink_start(c: &mut EffectContext<'_>) -> EffectResult {
    start(c, TargetKind::Water)
}
pub(super) fn sleep_start(c: &mut EffectContext<'_>) -> EffectResult {
    start(c, TargetKind::Sleep)
}

fn advance(c: &mut EffectContext<'_>, kind: TargetKind) -> EffectResult {
    if c.interrupted {
        return finish(c);
    }
    let state = c.state()?;
    let key = (c.position.x, c.position.y);
    match state {
        AgentState::Seeking { target } if target == kind => {
            let present = match kind {
                TargetKind::Food => c.resources.food_tiles.get(&key).is_some_and(|n| *n > 0),
                TargetKind::Water => c.resources.water_tiles.get(&key).is_some_and(|n| *n > 0),
                TargetKind::Sleep => {
                    c.resources.sleep_tiles.get(&key).is_some_and(|n| *n > 0)
                        && (c.resources.rules.agent.night_rest.is_none()
                            || c.resources
                                .bed_of
                                .get(&c.agent)
                                .is_none_or(|bed| *bed == key))
                }
                TargetKind::GatherFood
                | TargetKind::GatherWood
                | TargetKind::ConstructionSite
                | TargetKind::Agent(_)
                | TargetKind::Activity
                | TargetKind::Job => return Err(EffectError::WrongTarget),
            };
            if present {
                c.set_state(AgentState::Consuming { target: kind })?;
            }
            Ok(EffectStatus::Running)
        }
        AgentState::Consuming { target } if target == kind => {
            let finished = match kind {
                TargetKind::Food => {
                    let (state, need) = c
                        .world
                        .query_one_mut::<(&mut AgentState, Option<&mut Hunger>)>(c.entity)
                        .map_err(|_| EffectError::MissingState)?;
                    let before_meals = c.resources.tallies.meals_eaten;
                    let before_missing = c.resources.tallies.meals_without_food;
                    consume::consume_food(
                        c.resources,
                        key,
                        state,
                        need,
                        &mut c.effects.food_version,
                    );
                    if c.resources.tallies.meals_eaten > before_meals
                        && c.resources.tallies.meals_without_food == before_missing
                    {
                        c.effects.completed_meals.push(c.agent);
                    }
                    c.effects.mark_dirty(EffectDomain::FoodTile(key));
                    *state == AgentState::Idle
                }
                TargetKind::Water => {
                    let (state, need) = c
                        .world
                        .query_one_mut::<(&mut AgentState, Option<&mut Thirst>)>(c.entity)
                        .map_err(|_| EffectError::MissingState)?;
                    consume::consume_water(c.resources, key, state, need);
                    *state == AgentState::Idle
                }
                TargetKind::Sleep => {
                    let (state, hunger, thirst, sleep) = c
                        .world
                        .query_one_mut::<(
                            &mut AgentState,
                            Option<&Hunger>,
                            Option<&Thirst>,
                            Option<&mut Sleep>,
                        )>(c.entity)
                        .map_err(|_| EffectError::MissingState)?;
                    consume::consume_sleep(
                        c.resources,
                        key,
                        state,
                        hunger,
                        thirst,
                        sleep,
                        c.night_rest,
                        c.availability,
                    );
                    *state == AgentState::Idle
                }
                TargetKind::GatherFood
                | TargetKind::GatherWood
                | TargetKind::ConstructionSite
                | TargetKind::Agent(_)
                | TargetKind::Activity
                | TargetKind::Job => return Err(EffectError::WrongTarget),
            };
            c.effects.mark_dirty(EffectDomain::Needs);
            c.effects.mark_dirty(EffectDomain::Targets);
            Ok(if finished {
                EffectStatus::Finished
            } else {
                EffectStatus::Running
            })
        }
        AgentState::Idle | AgentState::Seeking { .. } | AgentState::Consuming { .. } => {
            Ok(EffectStatus::Finished)
        }
    }
}
pub(super) fn eat_continue(c: &mut EffectContext<'_>) -> EffectResult {
    if c.resources.rules.colony.is_some() {
        return super::food::advance(c);
    }
    advance(c, TargetKind::Food)
}
pub(super) fn drink_continue(c: &mut EffectContext<'_>) -> EffectResult {
    if c.resources.rules.colony.is_some()
        && c.state()?
            == (AgentState::Seeking {
                target: TargetKind::Water,
            })
    {
        if c.target == ActionTarget::None {
            return finish(c);
        }
        c.attach(c.tile_target()?);
    }
    advance(c, TargetKind::Water)
}
pub(super) fn sleep_continue(c: &mut EffectContext<'_>) -> EffectResult {
    advance(c, TargetKind::Sleep)
}
pub(super) fn finish(c: &mut EffectContext<'_>) -> EffectResult {
    c.set_state(AgentState::Idle)?;
    c.clear();
    Ok(EffectStatus::Finished)
}
