//! Physical food lifecycle: arrive first, then consume an actual selected owner.
use super::context::*;
use sim_core::components::{AgentState, TargetKind};

pub(super) fn start(c: &mut EffectContext<'_>) -> EffectResult {
    let ActionTarget::Food { tile, owner } = c.target else {
        return finish(c);
    };
    c.resources.food.active_targets.insert(c.agent, owner);
    c.set_state(AgentState::Seeking {
        target: TargetKind::Food,
    })?;
    c.attach(tile);
    Ok(EffectStatus::Running)
}

pub(super) fn advance(c: &mut EffectContext<'_>) -> EffectResult {
    if c.interrupted {
        return finish(c);
    }
    let position = (c.position.x, c.position.y);
    let retained = c
        .resources
        .food
        .active_targets
        .get(&c.agent)
        .copied()
        .and_then(|owner| eligible(c, owner).map(|tile| (tile, owner)));
    let (tile, owner) = match retained {
        Some(target) => target,
        None => {
            c.resources.food.active_targets.remove(&c.agent);
            let ActionTarget::Food { owner, .. } = c.target else {
                return finish(c);
            };
            let Some(tile) = eligible(c, owner) else {
                return finish(c);
            };
            c.resources.food.active_targets.insert(c.agent, owner);
            (tile, owner)
        }
    };
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
                return finish(c);
            }
            c.effects.completed_meals.push(c.agent);
            c.effects.dirty.insert(EffectDomain::FoodSources);
            c.effects.dirty.insert(EffectDomain::Needs);
            c.effects.dirty.insert(EffectDomain::Targets);
            // One physical piece completes one action, preserving the Idle boundary.
            // Further hunger is reconsidered by authored policy on the next tick.
            finish(c)
        }
        AgentState::Idle | AgentState::Seeking { .. } | AgentState::Consuming { .. } => finish(c),
    }
}

/// Resolve the admitted owner from live physical state without reranking alternatives.
fn eligible(c: &EffectContext<'_>, owner: sim_core::colony::food::FoodOwner) -> Option<(u32, u32)> {
    use sim_core::colony::food::FoodOwner;
    use sim_core::colony::{Carry, ItemStack};
    let res = &*c.resources;
    let origin = (c.position.x, c.position.y);
    let tile = match owner {
        FoodOwner::Ground(tile) => tile,
        FoodOwner::Pot { id, .. } => res.food.pots.get(&id)?.tile,
        FoodOwner::Carried(id) if id == c.agent => origin,
        FoodOwner::Carried(_) | FoodOwner::Shelf { .. } => return None,
    };
    let stack = match owner {
        FoodOwner::Carried(_) => {
            let carry = c.world.get::<&Carry>(c.entity).ok()?;
            ItemStack::of(carry.spec(), carry.count)
        }
        _ => crate::runtime::food::items::stack(c.world, res, owner)?,
    };
    let food = stack.kind.definition().food.as_ref()?;
    let age = res.food.ages.get(&owner)?;
    let day = crate::runtime::food::day(res);
    if stack.count == 0
        || !food.raw_edible
        || age.born_day > day
        || age.remaining(
            day,
            food.lifetime_days(crate::runtime::food::storage::class(res, owner)),
        ) == 0
        || !res.nav.passable(tile)
        || !res.nav.reachable(origin, tile)
    {
        return None;
    }
    Some(tile)
}

pub(super) fn finish(c: &mut EffectContext<'_>) -> EffectResult {
    if c.resources.rules.colony.is_some() {
        c.resources.food.active_targets.remove(&c.agent);
    }
    super::needs::finish(c)
}
