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
            let mut state = c
                .world
                .get::<&mut AgentState>(c.entity)
                .map_err(|_| EffectError::MissingState)?;
            match kind {
                TargetKind::Food => {
                    let before_meals = c.resources.tallies.meals_eaten;
                    let before_missing = c.resources.tallies.meals_without_food;
                    let mut need = c.world.get::<&mut Hunger>(c.entity).ok();
                    consume::consume_food(
                        c.resources,
                        key,
                        &mut state,
                        need.as_deref_mut(),
                        &mut c.effects.food_version,
                    );
                    if c.resources.tallies.meals_eaten > before_meals
                        && c.resources.tallies.meals_without_food == before_missing
                    {
                        c.effects.completed_meals.push(c.agent);
                    }
                    c.effects.dirty.insert(EffectDomain::FoodTile(key));
                }
                TargetKind::Water => {
                    let mut need = c.world.get::<&mut Thirst>(c.entity).ok();
                    consume::consume_water(c.resources, key, &mut state, need.as_deref_mut());
                }
                TargetKind::Sleep => {
                    let hunger = c.world.get::<&Hunger>(c.entity).ok();
                    let thirst = c.world.get::<&Thirst>(c.entity).ok();
                    let mut sleep = c.world.get::<&mut Sleep>(c.entity).ok();
                    consume::consume_sleep(
                        c.resources,
                        key,
                        &mut state,
                        hunger.as_deref(),
                        thirst.as_deref(),
                        sleep.as_deref_mut(),
                        c.night_rest,
                        c.availability,
                    );
                }
                TargetKind::GatherFood
                | TargetKind::GatherWood
                | TargetKind::ConstructionSite
                | TargetKind::Agent(_)
                | TargetKind::Activity
                | TargetKind::Job => return Err(EffectError::WrongTarget),
            }
            c.effects.dirty.insert(EffectDomain::Needs);
            c.effects.dirty.insert(EffectDomain::Targets);
            Ok(if *state == AgentState::Idle {
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
    advance(c, TargetKind::Food)
}
pub(super) fn drink_continue(c: &mut EffectContext<'_>) -> EffectResult {
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
