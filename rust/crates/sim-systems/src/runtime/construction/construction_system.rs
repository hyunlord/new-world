//! `ConstructionSystem` at priority 133 — the runtime tick of the
//! Phase 6 agent-driven construction loop (V7 Phase 6-β / P6β-1).
//!
//! # Tick ordering
//!
//! ```text
//! 120  AgentMovementSystem
//! 125  AgentDecisionSystem        (Idle→Seeking, Seeking→Consuming, ConstructionStarted emit)
//! 130  HungerDecaySystem
//! 131  ThirstDecaySystem
//! 132  SleepDecaySystem
//! 133  ConstructionSystem         ← this file
//! 1000 InfluenceVisualizationSystem
//! ```
//!
//! Slotted strictly between `SleepDecaySystem` (132) and
//! `InfluenceVisualizationSystem` (1000), runs every tick.
//!
//! # Responsibilities
//!
//! For each agent in [`AgentState::Consuming`]`{ target: TargetKind::ConstructionSite }`:
//!
//! 1. Look up the co-located [`ConstructionSite`] entity by `(pos.x, pos.y)`.
//! 2. If no site exists at the tile — **absent-site fallback**: reset the
//!    agent to [`AgentState::Idle`] without panic, without progress
//!    mutation, without emitting completion events. This guards against
//!    the race where the site was despawned between the
//!    `AgentDecisionSystem` tick and this one.
//! 3. If a site exists — call [`ConstructionSite::advance`]. Each
//!    co-located [`AgentState::Consuming`]`{ConstructionSite}` agent
//!    advances the site **once** per tick (per-agent progress semantics
//!    per locked P6β-10 / attempt 2). The completion-edge is recorded
//!    at most once per site even if multiple agents are co-located; any
//!    subsequent agents in the same tick observe the saturated site
//!    and contribute no extra completion chain.
//! 4. On completion edge — emit
//!    [`CausalEvent::ConstructionCompleted`] followed by
//!    [`CausalEvent::BuildingPlaced`] (parent-chained,
//!    `radius = 0`), reset all co-located
//!    `Consuming { ConstructionSite }` agents to `Idle`, and despawn the
//!    site entity.
//!
//! # Causal chain closure
//!
//! ```text
//! AgentDecision { reason: ConstructionReason } ← parent
//!   ↑ (ConstructionStarted.parent)
//! ConstructionStarted                          ← parent
//!   ↑ (ConstructionCompleted.parent)
//! ConstructionCompleted                        ← parent
//!   ↑ (BuildingPlaced.parent)
//! BuildingPlaced { radius: 0 }
//! ```
//!
//! `radius = 0` distinguishes the agent-construction emission path from
//! the BSS (`BuildingStampSystem` priority 90) stamping path; the
//! agent-construction path does not stamp influence in Phase 6-β.

use std::collections::HashMap;

use hecs::{Entity, World};
use sim_core::causal::CausalEvent;
use sim_core::components::{AgentState, BuildingBlueprint, ConstructionSite, Position, TargetKind};
use sim_engine::{RuntimeSystem, SimResources};

/// Phase 6-β construction runtime system. Stateless — all per-site state
/// lives in the [`ConstructionSite`] component.
#[derive(Debug, Default)]
pub struct ConstructionSystem;

impl ConstructionSystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for ConstructionSystem {
    fn name(&self) -> &str {
        "ConstructionSystem"
    }

    fn priority(&self) -> u32 {
        133
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let width = resources.tile_grid.width;
        if width == 0 || resources.tile_grid.height == 0 {
            return;
        }
        let tick = resources.current_tick;

        // (a) Snapshot site positions for fast agent-side lookup. We
        //     additionally track which sites are already complete so the
        //     agent loop below can treat already-complete sites as the
        //     same kind of "no work to do" condition as absent sites:
        //     the agent gets reset to Idle (no progress mutation, no
        //     duplicate completion chain).
        let mut sites_by_pos: HashMap<(u32, u32), Entity> = HashMap::new();
        let mut complete_sites_by_pos: HashMap<(u32, u32), Entity> = HashMap::new();
        {
            let mut q = world.query::<&ConstructionSite>();
            for (e, site) in q.iter() {
                let key = (site.position.x, site.position.y);
                if site.is_complete() {
                    complete_sites_by_pos.insert(key, e);
                } else {
                    sites_by_pos.insert(key, e);
                }
            }
        }

        // (b) Collect deferred operations from the agent loop. The borrow
        //     checker forbids mutating site components inside the agent
        //     query, so all writes happen below.
        //
        //     Attempt-3 fix: an agent in `Consuming{ConstructionSite}` on
        //     an already-complete site is routed to the same Idle-reset
        //     path as the absent-site fallback — without this, the agent
        //     stays stuck in Consuming forever (advance() returns false
        //     on a saturated site, so no completion edge fires and the
        //     agent never transitions out). No duplicate completion chain
        //     is emitted for the already-complete site.
        let mut absent_resets: Vec<Entity> = Vec::new();
        let mut progress_ops: Vec<Entity> = Vec::new();
        {
            let mut q = world.query::<(&Position, &AgentState)>();
            for (agent_entity, (pos, state)) in q.iter() {
                if !matches!(
                    *state,
                    AgentState::Consuming {
                        target: TargetKind::ConstructionSite
                    }
                ) {
                    continue;
                }
                let key = (pos.x, pos.y);
                if complete_sites_by_pos.contains_key(&key) {
                    // Already-complete site fallback — treat like absent.
                    absent_resets.push(agent_entity);
                    continue;
                }
                match sites_by_pos.get(&key).copied() {
                    None => absent_resets.push(agent_entity),
                    Some(site_entity) => progress_ops.push(site_entity),
                }
            }
        }

        // (c) Absent-site (or already-complete-site) fallback — reset
        //     agent FSM to Idle. Per the locked spec, no progress
        //     mutation, no completion events, no panic on a missing
        //     entity (the agent's site was despawned mid-tick or was
        //     already saturated by direct field write).
        for agent_entity in &absent_resets {
            if let Ok(mut state) = world.get::<&mut AgentState>(*agent_entity) {
                *state = AgentState::Idle;
            }
        }

        // (d) Per-agent progress: each co-located Consuming{ConstructionSite}
        //     agent advances the site once per tick. The completion-edge
        //     is recorded at most once per site — subsequent advances on a
        //     saturated site return `false` and contribute no extra chain.
        let mut completed: HashMap<Entity, (BuildingBlueprint, (u32, u32))> = HashMap::new();
        for site_entity in progress_ops {
            // Scope the mutable borrow tightly so the loop can re-enter
            // hecs queries below.
            let edge: Option<(BuildingBlueprint, (u32, u32))> = {
                let mut site_ref = match world.get::<&mut ConstructionSite>(site_entity) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let just_completed = site_ref.advance();
                if just_completed {
                    Some((
                        site_ref.blueprint,
                        (site_ref.position.x, site_ref.position.y),
                    ))
                } else {
                    None
                }
            };
            if let Some((blueprint, position)) = edge {
                // First completion edge wins. Later agents observe the
                // saturated site (advance returns false) and are silently
                // absorbed into the same tick — no duplicate chain.
                completed.entry(site_entity).or_insert((blueprint, position));
            }
        }

        // (e) For each completion edge: emit ConstructionCompleted then
        //     BuildingPlaced, reset all co-located Consuming agents to
        //     Idle, and despawn the site.
        for (site_entity, (blueprint, position)) in completed {
            let tile_idx = position.1 * width + position.0;

            // Parent of ConstructionCompleted: the most recent
            // ConstructionStarted on this tile (if any).
            let started_parent = resources
                .causal_log
                .get(tile_idx)
                .and_then(|log| {
                    log.as_slice().iter().rev().find_map(|ev| match ev {
                        CausalEvent::ConstructionStarted { id, .. } => Some(*id),
                        _ => None,
                    })
                });

            let completed_id = resources.issue_event_id();
            resources.causal_log.push(
                tile_idx,
                CausalEvent::ConstructionCompleted {
                    id: completed_id,
                    parent: started_parent,
                    blueprint,
                    position,
                    tick,
                },
            );

            // Parent of BuildingPlaced (agent-construction path): the
            // ConstructionCompleted we just emitted. radius = 0 —
            // agent-construction does not stamp influence in Phase 6-β.
            let placed_id = resources.issue_event_id();
            resources.causal_log.push(
                tile_idx,
                CausalEvent::BuildingPlaced {
                    id: placed_id,
                    parent: Some(completed_id),
                    position,
                    radius: 0,
                    tick,
                },
            );

            // Reset every co-located Consuming{ConstructionSite} agent
            // to Idle — owned by ConstructionSystem itself per the
            // locked Assertion 8 contract.
            {
                let mut q = world.query::<(&Position, &mut AgentState)>();
                for (_e, (pos, state)) in q.iter() {
                    if pos.x == position.0
                        && pos.y == position.1
                        && matches!(
                            *state,
                            AgentState::Consuming {
                                target: TargetKind::ConstructionSite
                            }
                        )
                    {
                        *state = AgentState::Idle;
                    }
                }
            }

            // Despawn the site after agent updates so the query above
            // can still observe co-located agents at this position.
            let _ = world.despawn(site_entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    #[test]
    fn metadata() {
        let s = ConstructionSystem::new();
        assert_eq!(s.name(), "ConstructionSystem");
        assert_eq!(s.priority(), 133);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn empty_world_tick_is_no_op() {
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        let mut sys = ConstructionSystem::new();
        // Must not panic on an empty world.
        sys.tick(&mut engine.world, &mut engine.resources);
    }
}
