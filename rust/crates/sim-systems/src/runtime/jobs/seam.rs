//! The decision seam (goal 4.3, ADR 0013): the only job code the decision
//! system calls. `decision/system.rs` gains about 25 lines that call these
//! three functions, plus the compile-forced `TargetKind::Job` arms.
//!
//! A job holder (an agent with a `Job` walking or working for it —
//! `Seeking { Job }` / `Consuming { Job }` — or a builder in
//! `Consuming { ConstructionSite }`) runs the decision's Idle cascade as a
//! temporary `Idle`, with leisure (the routine arm) skipped for it. After the
//! loop, `settle_after_decision` compares the choice:
//! - the decision chose nothing (idle wandering, the meal wait) — or chose to
//!   build the colony site the worker stands on — so the worker keeps its job
//!   and its previous state is restored;
//! - anything else (food, water, sleep, foraging, firewood, a V7 site,
//!   social, fire, flight) — the job is released (`CancelReason::Needs`) and
//!   the decision's choice stands; the job's `SeekTarget` is dropped so the
//!   post-pass re-resolves the new goal.
use hecs::{Entity, World};
use sim_core::colony::{CancelReason, Job};
use sim_core::components::{AgentState, Position, TargetKind};
use sim_engine::SimResources;
use std::collections::BTreeSet;

use super::cancel::release_job;

/// Every entity that holds a `Job` (sorted set: membership only).
pub fn holders(world: &World) -> BTreeSet<Entity> {
    world
        .query::<&Job>()
        .iter()
        .map(|(entity, _)| entity)
        .collect()
}

/// Whether the decision should run `entity` (in `state`) as a temporary Idle.
pub fn hold_for_decision(entity: Entity, state: AgentState, holders: &BTreeSet<Entity>) -> bool {
    match state {
        AgentState::Seeking {
            target: TargetKind::Job,
        }
        | AgentState::Consuming {
            target: TargetKind::Job,
        } => true,
        AgentState::Consuming {
            target: TargetKind::ConstructionSite,
        } => holders.contains(&entity),
        _ => false,
    }
}

/// Settle each held worker after the decision loop (see the module header).
/// `held` is in decision order (AgentId); structural changes follow it.
pub fn settle_after_decision(
    world: &mut World,
    res: &mut SimResources,
    held: &[(Entity, AgentState)],
    clear_seek: &mut Vec<Entity>,
) {
    for &(entity, previous) in held {
        if world.get::<&Job>(entity).is_err() {
            // A stale job state without a job: the decision's choice stands.
            continue;
        }
        let now = world
            .get::<&AgentState>(entity)
            .map(|state| *state)
            .unwrap_or_default();
        let own_site = matches!(
            now,
            AgentState::Seeking {
                target: TargetKind::ConstructionSite
            } | AgentState::Consuming {
                target: TargetKind::ConstructionSite
            }
        ) && world
            .get::<&Position>(entity)
            .is_ok_and(|pos| res.colony.occupied.contains_key(&(pos.x, pos.y)));
        if now == AgentState::Idle || own_site {
            if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                *state = previous;
            }
        } else {
            clear_seek.push(entity);
            release_job(world, res, entity, CancelReason::Needs);
        }
    }
}
