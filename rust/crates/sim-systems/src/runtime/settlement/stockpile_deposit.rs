//! Direction-2 slice 2-3a — `StockpileDepositSystem` (priority 141, warm tier).
//!
//! Passive Inventory→stockpile transfer. When a settlement member happens to be
//! within its settlement's [`SETTLEMENT_PROXIMITY_RADIUS`] of the settlement's
//! `formation_tile` while carrying Food, that Food drains entirely into the
//! settlement's uncapped [`Settlement::stockpile`]. This is the storage half of
//! the supply chain (slice 2-2 fills the agent `Inventory`; 2-3a empties it into
//! the settlement).
//!
//! Design (deliberately tight — slice 2-3a):
//! - **Passive / opportunistic.** It does NOT make agents walk home to deposit;
//!   it deposits when a member is already near home for any other reason. The
//!   active "return home to deposit" supply run is slice 2-3b.
//! - **No cascade, no sim-core variant.** Implemented as a standalone
//!   settlement-side pass reading positions + inventories + member rosters. This
//!   avoids the freeze origin (`agent_decision.rs`) entirely and adds no new
//!   `TargetKind`/`CascadeArm`.
//! - **Food only.** Slice 2-2 gathers only Food; other [`ResourceKind`]s are not
//!   produced yet and are left untouched in the member's `Inventory`.
//!
//! Determinism: settlements are iterated in sorted [`SettlementId`] order and
//! each settlement's `member_agents` (a `HashSet`) in sorted [`AgentId`] order,
//! so the deposit sequence is independent of `HashSet`/`HashMap` iteration order.

use std::collections::HashMap;

use hecs::{Entity, World};
use sim_core::components::{
    Agent, AgentId, Inventory, Position, ResourceKind, SettlementId, SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_engine::{RuntimeSystem, SimResources};

/// Chebyshev distance between two tile coordinates (matches the proximity
/// convention `SettlementSystem` uses for membership / formation).
fn chebyshev(a: (u32, u32), b: (u32, u32)) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}

/// Direction-2 slice 2-3a passive deposit system.
///
/// Priority **141** — strictly after `SettlementSystem` (138) so the
/// `member_agents` roster is current for the tick, and after `StarvationSystem`
/// (139) / `ResourceRegenSystem` (140) so a death's roster mutation already
/// settled. Warm tier (`tick_interval = 10`) — depositing is not hot-path work.
#[derive(Debug, Default)]
pub struct StockpileDepositSystem;

impl StockpileDepositSystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for StockpileDepositSystem {
    fn name(&self) -> &str {
        "StockpileDepositSystem"
    }

    fn priority(&self) -> u32 {
        141
    }

    fn tick_interval(&self) -> u64 {
        10
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        // AgentId → Entity map for resolving roster member ids to live entities.
        // A member id with no live entity (dead/despawned) is simply absent here
        // and is skipped below — no panic, no unwrap.
        let entity_by_id: HashMap<AgentId, Entity> = world
            .query::<&Agent>()
            .iter()
            .map(|(entity, agent)| (agent.id, entity))
            .collect();

        // Iterate settlements in deterministic (sorted) SettlementId order.
        let mut settlement_ids: Vec<SettlementId> =
            resources.settlements.keys().copied().collect();
        settlement_ids.sort_unstable();

        for sid in settlement_ids {
            // Snapshot home tile + sorted member roster for this settlement.
            let (home, members) = match resources.settlements.get(&sid) {
                Some(settlement) => {
                    let mut members: Vec<AgentId> =
                        settlement.member_agents.iter().copied().collect();
                    members.sort_unstable();
                    (settlement.formation_tile, members)
                }
                None => continue,
            };

            for aid in members {
                // Skip a member whose entity is missing (dead/despawned/stale id).
                let entity = match entity_by_id.get(&aid) {
                    Some(e) => *e,
                    None => continue,
                };

                // Read position (copy out so the borrow drops before the mut borrow).
                let pos = match world.get::<&Position>(entity) {
                    Ok(p) => (p.x, p.y),
                    Err(_) => continue,
                };
                if chebyshev(pos, home) > SETTLEMENT_PROXIMITY_RADIUS {
                    continue;
                }

                // Drain ALL carried Food (stockpile is uncapped). `remove`
                // returns the amount actually removed; `store` adds exactly that.
                let taken = match world.get::<&mut Inventory>(entity) {
                    Ok(mut inv) => {
                        let n = inv.get(ResourceKind::Food);
                        if n == 0 {
                            continue;
                        }
                        inv.remove(ResourceKind::Food, n)
                    }
                    Err(_) => continue,
                };

                if taken > 0 {
                    if let Some(settlement) = resources.settlements.get_mut(&sid) {
                        settlement.store(ResourceKind::Food, taken);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata() {
        let s = StockpileDepositSystem::new();
        assert_eq!(s.name(), "StockpileDepositSystem");
        assert_eq!(s.priority(), 141);
        assert_eq!(s.tick_interval(), 10);
    }

    #[test]
    fn chebyshev_is_max_axis_distance() {
        assert_eq!(chebyshev((10, 10), (15, 10)), 5);
        assert_eq!(chebyshev((10, 10), (10, 15)), 5);
        assert_eq!(chebyshev((10, 10), (13, 14)), 4);
        assert_eq!(chebyshev((10, 10), (10, 10)), 0);
    }
}
