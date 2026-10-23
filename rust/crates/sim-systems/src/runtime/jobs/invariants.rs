//! The 7.6 invariants: item conservation per kind (M2 goal 3.1), reservations,
//! stacks. Asserted every tick by tests and, under debug assertions, by
//! `JobExecuteSystem`.
use hecs::World;
use sim_core::colony::{Carry, ItemKind, Job, ReserveKey, Tool};
use sim_core::components::{Agent, AgentId};
use sim_engine::SimResources;
use std::collections::{BTreeMap, BTreeSet};

use super::items::{can_hold_items, colony_sites, stack_max};

/// Where the items of one kind are (M2 goal 3.1): the terms of the per-kind
/// conservation `created == ground + carried + held + delivered + shelved + consumed`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LedgerTerms {
    /// Items ever created (`ItemLedger::created`).
    pub created: u64,
    /// On the ground (stacks).
    pub ground: u64,
    /// In agents' hands (`Carry`).
    pub carried: u64,
    /// Held as tools (`Tool`).
    pub held: u64,
    /// Delivered to construction sites (`MaterialNeeds`).
    pub delivered: u64,
    /// On workshop input shelves.
    pub shelved: u64,
    /// Used up (`ItemLedger::consumed`).
    pub consumed: u64,
}

impl LedgerTerms {
    /// Whether the conservation holds.
    pub const fn balanced(&self) -> bool {
        self.created
            == self.ground
                + self.carried
                + self.held
                + self.delivered
                + self.shelved
                + self.consumed
    }
}

/// The ledger terms of `kind`.
pub fn ledger_of(world: &World, res: &SimResources, kind: ItemKind) -> LedgerTerms {
    let carried = world
        .query::<&Carry>()
        .iter()
        .filter(|(_, carry)| carry.kind == kind)
        .map(|(_, carry)| u64::from(carry.count))
        .sum();
    // M5: a worn top counts as held (ADR 0029).
    let worn = world
        .query::<&sim_core::colony::people::Apparel>()
        .iter()
        .filter(|(_, a)| a.top.item() == kind)
        .count() as u64;
    let held = world
        .query::<&Tool>()
        .iter()
        .filter(|(_, tool)| tool.item.kind == kind)
        .count() as u64
        + worn;
    let delivered = colony_sites(world)
        .values()
        .map(|view| u64::from(view.needs.delivered_of(kind)))
        .sum();
    let shelved = res
        .colony
        .workshops
        .values()
        .flat_map(|w| w.shelf.iter())
        .filter(|stack| stack.kind == kind)
        .map(|stack| u64::from(stack.count))
        .sum();
    LedgerTerms {
        created: res.colony.item_ledger.created_of(kind),
        ground: res.colony.ground_total(kind),
        carried,
        held,
        delivered,
        shelved,
        consumed: res.colony.item_ledger.consumed_of(kind),
    }
}

/// The ledger's terms for logs: `(created, on the ground, carried, delivered, consumed)` (M1's form).
pub fn log_ledger(world: &World, res: &SimResources) -> (u64, u64, u64, u64, u64) {
    let t = ledger_of(world, res, ItemKind::Log);
    (
        t.created,
        t.ground,
        t.carried,
        t.delivered + t.shelved + t.held,
        t.consumed,
    )
}

/// Check every 7.6 invariant; the first broken one is returned as text.
/// The ledger message starts with `item ledger` (`log ledger` for logs, M1's form).
pub fn check(world: &World, res: &SimResources) -> Result<(), String> {
    for kind in ItemKind::ALL {
        let t = ledger_of(world, res, kind);
        if !t.balanced() {
            let label = if kind == ItemKind::Log {
                "log ledger".to_string()
            } else {
                format!("item ledger ({})", kind.name())
            };
            return Err(format!(
                "{label}: created {} != ground {} + carried {} + held {} + delivered {} + shelved {} + consumed {}",
                t.created, t.ground, t.carried, t.held, t.delivered, t.shelved, t.consumed
            ));
        }
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
    for (tile, stack) in &res.colony.ground_items {
        let max = stack_max(res, stack.kind);
        if stack.count == 0 || stack.count > max {
            return Err(format!("stack of {} at {tile:?} (max {max})", stack.count));
        }
        // M3: a stack may wait on a space site's tile until it is carried off.
        if !can_hold_items(res, *tile) && !res.colony.space_sites.contains_key(tile) {
            return Err(format!(
                "stack at {tile:?} on water, mountain or under a building"
            ));
        }
    }
    Ok(())
}
