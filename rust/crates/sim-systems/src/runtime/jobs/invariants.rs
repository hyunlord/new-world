//! The 7.6 invariants: log conservation, reservations, stacks. Asserted every
//! tick by tests and, under debug assertions, by `JobExecuteSystem`.
use hecs::World;
use sim_core::colony::{Carry, ItemKind, Job, ReserveKey};
use sim_core::components::{Agent, AgentId};
use sim_engine::SimResources;
use std::collections::{BTreeMap, BTreeSet};

use super::items::{can_hold_items, colony_sites};

/// The ledger's terms for logs: `(created, on the ground, carried, delivered, consumed)`.
pub fn log_ledger(world: &World, res: &SimResources) -> (u64, u64, u64, u64, u64) {
    let carried = world
        .query::<&Carry>()
        .iter()
        .filter(|(_, carry)| carry.kind == ItemKind::Log)
        .map(|(_, carry)| u64::from(carry.count))
        .sum();
    let delivered = colony_sites(world)
        .values()
        .filter(|view| view.needs.kind == ItemKind::Log)
        .map(|view| u64::from(view.needs.delivered))
        .sum();
    (
        res.colony.item_ledger.created_of(ItemKind::Log),
        res.colony.ground_total(ItemKind::Log),
        carried,
        delivered,
        res.colony.item_ledger.consumed_of(ItemKind::Log),
    )
}

/// Check every 7.6 invariant; the first broken one is returned as text.
pub fn check(world: &World, res: &SimResources) -> Result<(), String> {
    let (created, ground, carried, delivered, consumed) = log_ledger(world, res);
    if created != ground + carried + delivered + consumed {
        return Err(format!(
            "log ledger: created {created} != ground {ground} + carried {carried} + delivered {delivered} + consumed {consumed}"
        ));
    }
    let mut held: BTreeMap<AgentId, BTreeSet<ReserveKey>> = BTreeMap::new();
    for (_, (agent, job)) in world.query::<(&Agent, &Job)>().iter() {
        held.insert(agent.id, job.keys().into_iter().collect());
    }
    let mut reserved: BTreeMap<AgentId, BTreeSet<ReserveKey>> = BTreeMap::new();
    for (key, agent) in &res.colony.reservations {
        reserved.entry(*agent).or_default().insert(*key);
    }
    for (agent, keys) in &reserved {
        match held.get(agent) {
            None => {
                return Err(format!(
                    "reservations {keys:?} held by agent {agent}, who has no job"
                ))
            }
            Some(job_keys) if job_keys != keys => {
                return Err(format!(
                    "agent {agent} reserves {keys:?} but its job holds {job_keys:?}"
                ))
            }
            Some(_) => {}
        }
    }
    for (agent, job_keys) in &held {
        if !job_keys.is_empty() && !reserved.contains_key(agent) {
            return Err(format!(
                "agent {agent}'s job holds {job_keys:?} without reservations"
            ));
        }
    }
    let max = res
        .rules
        .colony
        .map_or(u16::MAX, |rule| rule.stack_max_logs);
    for (tile, stack) in &res.colony.ground_items {
        if stack.count == 0 || stack.count > max {
            return Err(format!("stack of {} at {tile:?} (max {max})", stack.count));
        }
        if !can_hold_items(res, *tile) {
            return Err(format!(
                "stack at {tile:?} on water, mountain or under a building"
            ));
        }
    }
    Ok(())
}
