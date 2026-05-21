//! `CombatSystem` (priority 137) — Phase 9-β runtime combat resolution.
//!
//! Responsibility: advance `combat_progress` for every active pair in
//! `SimResources::combat_pairs`. When a pair reaches
//! `REQUIRED_COMBAT_PROGRESS`:
//!   1. Apply `DAMAGE_PER_COMBAT_TICK` to the defender's `BodyHealth`.
//!   2. Emit `CausalEvent::CombatCompleted` (parent = most recent same-pair
//!      `CombatStarted` on the shared tile).
//!   3. Bump `relationships[RelationshipKey::new(attacker, defender)].hostility`
//!      by `HOSTILITY_BUMP`.
//!   4. Remove pair from `combat_pairs` + `combat_progress`.
//!   5. If defender `is_dead()`: despawn immediately + resource-map cleanup
//!      (relationships, interaction_progress, combat_pairs, combat_progress).
//!      Reset attacker to `AgentState::Idle`.
//!   6. If defender alive: reset both attacker and defender to `Idle`.
//!
//! `AgentDecisionSystem` (priority 125) owns the `Idle → Consuming{Agent}`
//! transition and `CombatStarted` emission. `CombatSystem` never emits
//! `CombatStarted` — it only emits `CombatCompleted`.
//!
//! Defensive behavior:
//!   - Attacker entity already despawned: the pair is dropped silently,
//!     no event emitted, no panic.
//!   - Defender entity already despawned: attacker is reset to Idle,
//!     pair dropped, no event emitted, no panic.
//!   - Defender missing `BodyHealth` component: treated as hp_after = 0.0
//!     and the despawn pathway runs (so the malformed agent is removed
//!     cleanly rather than left half-tracked). No panic.

use std::collections::HashMap;

use hecs::World;
use sim_core::causal::CausalEvent;
use sim_core::components::{
    Agent, AgentId, AgentState, BodyHealth, Memory, MemoryEntry, Position, RelationshipKey,
    HOSTILITY_BUMP,
};
use sim_engine::{RuntimeSystem, SimResources};

use crate::runtime::combat::{DAMAGE_PER_COMBAT_TICK, REQUIRED_COMBAT_PROGRESS};

/// Phase 9-β combat resolution system.
#[derive(Debug, Default)]
pub struct CombatSystem;

impl CombatSystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for CombatSystem {
    fn name(&self) -> &str {
        "CombatSystem"
    }

    fn priority(&self) -> u32 {
        137
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let tick = resources.current_tick;
        let width = resources.tile_grid.width;

        // Build AgentId → Entity map (pattern from MemorySystem).
        let id_to_entity: HashMap<AgentId, hecs::Entity> = world
            .query::<&Agent>()
            .iter()
            .map(|(e, a)| (a.id, e))
            .collect();

        // Collect active pairs deterministically (sorted) so iteration
        // order is stable across HashSet hash randomisation — critical
        // for harness Assertion 27 (determinism).
        let mut active_pairs: Vec<(AgentId, AgentId)> =
            resources.combat_pairs.iter().copied().collect();
        active_pairs.sort();

        // Advance progress for all active pairs; collect completed.
        let mut completed: Vec<(AgentId, AgentId)> = Vec::new();
        for pair in &active_pairs {
            let prog = resources.combat_progress.entry(*pair).or_insert(0);
            *prog += 1;
            if *prog >= REQUIRED_COMBAT_PROGRESS {
                completed.push(*pair);
            }
        }

        // Process each completed pair.
        for (attacker_id, defender_id) in completed {
            // Remove tracking before any despawn path (idempotent cleanup).
            resources.combat_pairs.remove(&(attacker_id, defender_id));
            resources.combat_progress.remove(&(attacker_id, defender_id));

            let attacker_entity = match id_to_entity.get(&attacker_id).copied() {
                Some(e) => e,
                None => continue, // attacker already despawned
            };
            let defender_entity = match id_to_entity.get(&defender_id).copied() {
                Some(e) => e,
                None => {
                    // Defender already gone — reset attacker.
                    if let Ok(mut s) = world.get::<&mut AgentState>(attacker_entity) {
                        *s = AgentState::Idle;
                    }
                    continue;
                }
            };

            // Derive shared tile from attacker position.
            let position = world
                .get::<&Position>(attacker_entity)
                .map(|p| (p.x, p.y))
                .unwrap_or((0, 0));
            let tile_idx = position.1 * width + position.0;

            // Find parent CombatStarted id on the same tile (most recent
            // matching pair, reverse scan).
            let parent_id = resources.causal_log.get(tile_idx).and_then(|log| {
                log.as_slice().iter().rev().find_map(|ev| {
                    if let CausalEvent::CombatStarted {
                        id,
                        attacker,
                        defender,
                        ..
                    } = ev
                    {
                        if *attacker == attacker_id && *defender == defender_id {
                            return Some(*id);
                        }
                    }
                    None
                })
            });

            // Apply damage to defender. Missing BodyHealth treated as
            // hp_after = 0.0 to keep CombatSystem panic-free on malformed
            // entities (harness Assertion 28).
            let hp_after = match world.get::<&mut BodyHealth>(defender_entity) {
                Ok(mut bh) => {
                    bh.apply_damage(DAMAGE_PER_COMBAT_TICK);
                    bh.hp
                }
                Err(_) => 0.0,
            };

            // V7 Phase 10-γ / P10γ-A13: compute settlement link.
            // `Some(sid)` if either combatant is a member of any
            // settlement at emission time; lowest sid wins for
            // determinism. CombatSystem (priority 137) runs before
            // SettlementSystem (priority 138) in the same tick, so the
            // membership view is the pre-tick snapshot from the previous
            // SettlementSystem run — which is exactly the routing
            // predicate the chronicle harness asserts against.
            let settlement_link: Option<sim_core::components::SettlementId> = {
                let mut candidates: Vec<sim_core::components::SettlementId> = resources
                    .settlements
                    .iter()
                    .filter(|(_, s)| {
                        s.member_agents.contains(&attacker_id)
                            || s.member_agents.contains(&defender_id)
                    })
                    .map(|(id, _)| *id)
                    .collect();
                candidates.sort();
                candidates.first().copied()
            };

            // Emit CombatCompleted.
            let completed_id = resources.issue_event_id();
            resources.causal_log.push(
                tile_idx,
                CausalEvent::CombatCompleted {
                    id: completed_id,
                    parent: parent_id,
                    attacker: attacker_id,
                    defender: defender_id,
                    position,
                    hp_after,
                    settlement_link,
                    tick,
                },
            );

            // Encode CombatCompleted directly into Memory (MemorySystem priority 136
            // runs before CombatSystem priority 137 so it cannot see same-tick
            // CombatCompleted events; direct encoding here closes that gap).
            // Salience 0.9 / valence -0.8 mirror the MemorySystem mapping table.
            for enc_entity in [attacker_entity, defender_entity] {
                if let Ok(mut mem) = world.get::<&mut Memory>(enc_entity) {
                    if mem.find_by_event_id(completed_id).is_none() {
                        mem.insert(MemoryEntry::new(completed_id, tick, -0.8, 0.9));
                    }
                }
            }

            // Bump hostility (saturating, NaN-safe via `bump_hostility`).
            resources
                .relationships
                .entry(RelationshipKey::new(attacker_id, defender_id))
                .or_default()
                .bump_hostility(HOSTILITY_BUMP);

            // Check death via the post-damage component value (defensive:
            // missing BodyHealth ⇒ treat as dead so the malformed entity
            // is removed cleanly).
            let defender_dead = world
                .get::<&BodyHealth>(defender_entity)
                .map(|bh| bh.is_dead())
                .unwrap_or(true);

            if defender_dead {
                let dead_id = defender_id;
                let _ = world.despawn(defender_entity);

                // Cleanup all resource maps referencing the dead agent.
                resources
                    .relationships
                    .retain(|k, _| k.0 != dead_id && k.1 != dead_id);
                resources
                    .interaction_progress
                    .retain(|k, _| k.0 != dead_id && k.1 != dead_id);
                resources
                    .combat_pairs
                    .retain(|(a, d)| *a != dead_id && *d != dead_id);
                resources
                    .combat_progress
                    .retain(|(a, d), _| *a != dead_id && *d != dead_id);

                // Reset attacker to Idle.
                if let Ok(mut s) = world.get::<&mut AgentState>(attacker_entity) {
                    *s = AgentState::Idle;
                }
            } else {
                // Both survive — reset both to Idle.
                if let Ok(mut s) = world.get::<&mut AgentState>(attacker_entity) {
                    *s = AgentState::Idle;
                }
                if let Ok(mut s) = world.get::<&mut AgentState>(defender_entity) {
                    *s = AgentState::Idle;
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
        let s = CombatSystem::new();
        assert_eq!(s.name(), "CombatSystem");
        assert_eq!(s.priority(), 137);
        assert_eq!(s.tick_interval(), 1);
    }
}
