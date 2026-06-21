//! Direction-2 slice 2-4 — `StockpileConsumeSystem` (priority 142, warm tier).
//!
//! Famine-fallback consumption of the communal [`Settlement::stockpile`]. Slice
//! 2-2 gathers ground Food into agent `Inventory`; 2-3a deposits that Food into
//! the settlement reserve; 2-4 (this pass) lets a critically-hungry member draw
//! on that reserve so it does not starve to death beside a full stockpile.
//!
//! Design (deliberately tight — slice 2-4):
//! - **Direct relief, settlement-side.** Withdraws Food from the `stockpile`
//!   and relieves the member's [`Hunger`] directly (mirroring the ground-eat
//!   per-meal amount). No cascade, no sim-core change, no `AgentState` write —
//!   structurally symmetric with 2-3a's `StockpileDepositSystem`.
//! - **Gated as a famine fallback (no double-dip).** Relief fires ONLY when the
//!   member is NOT already handling hunger via the normal ground-eat path — i.e.
//!   `AgentState::target() != Some(TargetKind::Food)` (skips `Seeking{Food}` /
//!   `Consuming{Food}`). Normal ground eating is untouched.
//! - **Mass balance.** Withdraw exactly one meal BEFORE relieving; relieve only
//!   if `got > 0`. Never relieve hunger without consuming reserve; never consume
//!   reserve without relieving.
//! - **Food only.** Other [`ResourceKind`]s are not relief targets.
//!
//! Determinism: settlements are iterated in sorted [`SettlementId`] order and
//! each settlement's `member_agents` (a `HashSet`) in sorted [`AgentId`] order;
//! `stockpile` is a `BTreeMap`. The relief sequence is independent of
//! `HashSet`/`HashMap` iteration order.

use std::collections::HashMap;

use hecs::{Entity, World};
use sim_core::components::{
    Agent, AgentId, AgentState, Hunger, Position, ResourceKind, SettlementId, TargetKind,
    SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_engine::{RuntimeSystem, SimResources};

// The relief amount mirrors the ground-tile eat amount; imported (not
// redefined) from the decision module per the slice-2-4 zero-sim-core rule.
use crate::runtime::decision::{HUNGER_CONSUME_AMOUNT, HUNGER_THRESHOLD};

/// Famine level at/above which a settlement member draws on the communal
/// reserve. Set to [`HUNGER_THRESHOLD`] (50.0) so reserve consumption is a
/// genuine last-resort net — the same level at which the agent would normally
/// start seeking ground food.
pub const STOCKPILE_RELIEF_HUNGER_THRESHOLD: f32 = HUNGER_THRESHOLD;

/// Food units withdrawn from the reserve per relief event. Mirrors the ground
/// tile's per-meal decrement of 1.
pub const STOCKPILE_RELIEF_FOOD_PER_MEAL: u32 = 1;

/// Chebyshev distance between two tile coordinates (matches the proximity
/// convention `SettlementSystem` / `StockpileDepositSystem` use).
fn chebyshev(a: (u32, u32), b: (u32, u32)) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}

/// Direction-2 slice 2-4 famine-fallback consume system.
///
/// Priority **142** — strictly after `HungerDecaySystem` (130),
/// `SettlementSystem` (138) so the roster is current, `StarvationSystem` (139)
/// so this tick's saturation damage is computed from PRE-relief hunger, and
/// `StockpileDepositSystem` (141) so the reserve already reflects this tick's
/// deposits. Warm tier (`tick_interval = 5`) — relief is a fallback, not
/// hot-path work.
#[derive(Debug, Default)]
pub struct StockpileConsumeSystem;

impl StockpileConsumeSystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for StockpileConsumeSystem {
    fn name(&self) -> &str {
        "StockpileConsumeSystem"
    }

    fn priority(&self) -> u32 {
        142
    }

    fn tick_interval(&self) -> u64 {
        5
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        // AgentId → Entity map for resolving roster member ids to live entities.
        // A member id with no live entity (dead/despawned/stale) is absent here
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
                // Skip a member whose entity is missing (dead/despawned/stale).
                let entity = match entity_by_id.get(&aid) {
                    Some(e) => *e,
                    None => continue,
                };

                // Proximity gate (copy out so the borrow drops before any mut).
                let pos = match world.get::<&Position>(entity) {
                    Ok(p) => (p.x, p.y),
                    Err(_) => continue,
                };
                if chebyshev(pos, home) > SETTLEMENT_PROXIMITY_RADIUS {
                    continue;
                }

                // Double-dip gate: skip members already on the ground-eat path
                // (Seeking{Food}/Consuming{Food}). A missing AgentState is
                // treated as "not eating ground" (target None).
                let eating_ground = world
                    .get::<&AgentState>(entity)
                    .map(|s| s.target() == Some(TargetKind::Food))
                    .unwrap_or(false);
                if eating_ground {
                    continue;
                }

                // Famine-threshold gate (inclusive `>=`).
                let hungry = match world.get::<&Hunger>(entity) {
                    Ok(h) => h.value >= STOCKPILE_RELIEF_HUNGER_THRESHOLD,
                    Err(_) => continue,
                };
                if !hungry {
                    continue;
                }

                // Withdraw EXACTLY one meal BEFORE relieving; relieve only if
                // we actually consumed reserve (got > 0).
                let got = match resources.settlements.get_mut(&sid) {
                    Some(settlement) => {
                        settlement.withdraw(ResourceKind::Food, STOCKPILE_RELIEF_FOOD_PER_MEAL)
                    }
                    None => 0,
                };
                if got > 0 {
                    if let Ok(mut h) = world.get::<&mut Hunger>(entity) {
                        h.value = (h.value - HUNGER_CONSUME_AMOUNT).max(0.0);
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
        let s = StockpileConsumeSystem::new();
        assert_eq!(s.name(), "StockpileConsumeSystem");
        assert_eq!(s.priority(), 142);
        assert_eq!(s.tick_interval(), 5);
    }

    #[test]
    fn chebyshev_is_max_axis_distance() {
        assert_eq!(chebyshev((10, 10), (15, 10)), 5);
        assert_eq!(chebyshev((10, 10), (10, 15)), 5);
        assert_eq!(chebyshev((10, 10), (13, 14)), 4);
        assert_eq!(chebyshev((10, 10), (10, 10)), 0);
    }

    #[test]
    fn relief_constants_mirror_ground_eat() {
        // Relief threshold == the seek threshold; meal == ground per-meal (1).
        assert_eq!(STOCKPILE_RELIEF_HUNGER_THRESHOLD, HUNGER_THRESHOLD);
        assert_eq!(STOCKPILE_RELIEF_FOOD_PER_MEAL, 1);
    }
}
