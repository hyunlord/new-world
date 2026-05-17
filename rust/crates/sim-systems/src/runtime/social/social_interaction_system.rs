//! `SocialInteractionSystem` at priority 134 — the runtime tick of the
//! Phase 7-β agent-to-agent interaction loop (V7 Phase 7-β / P7β-1).
//!
//! # Tick ordering
//!
//! ```text
//! 120  AgentMovementSystem
//! 125  AgentDecisionSystem    (Idle→Seeking, Seeking→Consuming, SocialInteractionStarted emit)
//! 130  HungerDecaySystem
//! 131  ThirstDecaySystem
//! 132  SleepDecaySystem
//! 133  ConstructionSystem
//! 134  SocialInteractionSystem ← this file
//! 1000 InfluenceVisualizationSystem
//! ```
//!
//! # Responsibilities
//!
//! For each agent in [`AgentState::Consuming`]`{ target: TargetKind::Agent(partner_id) }`:
//!
//! 1. Verify mutuality: partner exists, is co-located, and its
//!    `AgentState` is `Consuming{Agent(self.id)}`.
//! 2. If mutual: advance `interaction_progress[RelationshipKey]` by 1
//!    (deduplicated via canonical key — one increment per pair per tick).
//! 3. On `progress >= REQUIRED_INTERACTION_PROGRESS`: emit
//!    [`CausalEvent::SocialInteractionCompleted`], bump
//!    [`RelationshipState::familiarity`] by `FAMILIARITY_BUMP` (saturating
//!    at 1.0), reset both agents to `Idle`, and saturating-subtract
//!    `SOCIAL_CONSUME_AMOUNT` from both agents' `Social.loneliness`.
//!    The `interaction_progress` entry is INTENTIONALLY LEFT at value
//!    `REQUIRED_INTERACTION_PROGRESS` on the completion tick (so a
//!    post-`engine.tick()` observation can witness the terminal progress
//!    value), and reaped by the step (g) stale-cleanup pass on the NEXT
//!    SIS tick — at which point both agents are no longer in mutual
//!    `Consuming` (they were reset to `Idle` above), so the key no longer
//!    appears in the live `mutual_pairs` snapshot and gets pruned.
//! 4. If NOT mutual (partner gone, on different tile, or pointing
//!    elsewhere): reset the agent to `Idle` WITHOUT panic, WITHOUT
//!    emitting a completion event, and remove any stale
//!    `interaction_progress` entry for this pair.
//! 5. Stale-progress cleanup: scan `interaction_progress` and remove any
//!    entry whose pair is not currently in mutual `Consuming`. Guards
//!    against unbounded growth from save/load mid-cycle or out-of-band
//!    mutation.
//!
//! # Causal chain closure
//!
//! ```text
//! AgentDecision { reason: SocialReason, agent: smaller } ← root
//!   ↑ (SocialInteractionStarted.parent)
//! SocialInteractionStarted { agents: (smaller, larger) }
//!   ↑ (SocialInteractionCompleted.parent)
//! SocialInteractionCompleted { agents, familiarity_after }
//! ```

use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};
use sim_core::causal::CausalEvent;
use sim_core::components::{
    Agent, AgentId, AgentState, Position, RelationshipKey, Social, TargetKind,
};
use sim_engine::{RuntimeSystem, SimResources};

use crate::runtime::decision::{
    FAMILIARITY_BUMP, REQUIRED_INTERACTION_PROGRESS, SOCIAL_CONSUME_AMOUNT,
};

/// Per-pair record used when ordering mutual `Consuming{Agent}` pairs for
/// deterministic side-effect application. Pairs are sorted by canonical
/// `RelationshipKey` (smaller-then-larger) before progress mutation,
/// completion-event push, familiarity bump, and state transition.
type OrderedPairRow = (RelationshipKey, usize, usize, (u32, u32));

/// Phase 7-β social interaction runtime system. Stateless — all per-pair
/// state lives in `SimResources::interaction_progress` and
/// `SimResources::relationships`.
#[derive(Debug, Default)]
pub struct SocialInteractionSystem;

impl SocialInteractionSystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

/// Snapshot row for one agent currently in `Consuming{Agent(_)}`.
struct ConsumingRow {
    entity: Entity,
    agent_id: AgentId,
    position: (u32, u32),
    partner_id: AgentId,
}

impl RuntimeSystem for SocialInteractionSystem {
    fn name(&self) -> &str {
        "SocialInteractionSystem"
    }

    fn priority(&self) -> u32 {
        134
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

        // (a) Snapshot every agent currently in Consuming{Agent(partner_id)}.
        let mut consuming: Vec<ConsumingRow> = Vec::new();
        {
            let mut q = world.query::<(&Agent, &Position, &AgentState)>();
            for (entity, (a, pos, state)) in q.iter() {
                if let AgentState::Consuming {
                    target: TargetKind::Agent(partner_id),
                } = *state
                {
                    consuming.push(ConsumingRow {
                        entity,
                        agent_id: a.id,
                        position: (pos.x, pos.y),
                        partner_id,
                    });
                }
            }
        }

        // (b) Index by AgentId for partner lookup.
        let id_to_idx: HashMap<AgentId, usize> = consuming
            .iter()
            .enumerate()
            .map(|(i, row)| (row.agent_id, i))
            .collect();

        // (c) Classify each consuming row as mutual or fallback.
        //     mutual_pairs is deduplicated by canonical RelationshipKey so
        //     progress is incremented at most once per pair per tick.
        let mut mutual_pairs: HashMap<RelationshipKey, (usize, usize, (u32, u32))> =
            HashMap::new();
        let mut fallback_entities: Vec<Entity> = Vec::new();
        let mut fallback_keys: Vec<RelationshipKey> = Vec::new();

        for row in &consuming {
            let mutual = id_to_idx
                .get(&row.partner_id)
                .map(|&pi| {
                    let partner = &consuming[pi];
                    partner.partner_id == row.agent_id && partner.position == row.position
                })
                .unwrap_or(false);

            if !mutual {
                fallback_entities.push(row.entity);
                fallback_keys.push(RelationshipKey::new(row.agent_id, row.partner_id));
                continue;
            }

            let key = RelationshipKey::new(row.agent_id, row.partner_id);
            if let std::collections::hash_map::Entry::Vacant(slot) = mutual_pairs.entry(key) {
                let self_idx = id_to_idx[&row.agent_id];
                let partner_idx = id_to_idx[&row.partner_id];
                slot.insert((self_idx, partner_idx, row.position));
            }
        }

        // (d) Apply fallback: reset agent → Idle and remove the (possibly
        //     stale) progress entry. No completion event.
        for entity in &fallback_entities {
            if let Ok(mut state) = world.get::<&mut AgentState>(*entity) {
                *state = AgentState::Idle;
            }
        }
        for key in &fallback_keys {
            resources.interaction_progress.remove(key);
        }

        // (e) Advance progress for each mutual pair. Detect completion.
        //
        // §β re-plan determinism lock: iterate `mutual_pairs` in sorted
        // canonical `RelationshipKey` order BEFORE applying any side
        // effect (progress mutation, event push, familiarity bump, state
        // transition). HashMap iteration order is non-deterministic;
        // sorting at the side-effect boundary keeps per-tick observable
        // behaviour identical across same-seed runs (A20).
        //
        // First-observation semantics (P7β-10 tick-delta lock):
        //   The vacant-entry branch must distinguish two cases:
        //
        //   (i) Handshake-tick (real cascade): on the SAME tick that
        //       AgentDecisionSystem emitted `SocialInteractionStarted` for
        //       this pair, this system runs at priority 134 and sees the
        //       pair for the first time. We SEED at 0 — no increment yet.
        //       This keeps `Completed.tick - Started.tick` exactly equal to
        //       `REQUIRED_INTERACTION_PROGRESS` (locked by A9).
        //
        //   (ii) Pre-placed (no Started this tick): the pair entered mutual
        //        Consuming through some other path (test setup, save/load,
        //        out-of-band mutation). No Started event was emitted at the
        //        current tick for this pair, so the "handshake tick is tick
        //        zero of the cycle" invariant does NOT apply. We INSERT 1
        //        (treat this tick as the first progress tick). This makes a
        //        fresh pre-placed pair complete after exactly
        //        `REQUIRED_INTERACTION_PROGRESS` direct ticks.
        //
        //   On every subsequent same-pair tick, we INCREMENT by 1.
        //   Completion fires when post-increment progress reaches
        //   `REQUIRED_INTERACTION_PROGRESS`.
        let mut completion_outputs: Vec<(RelationshipKey, (u32, u32), Entity, Entity)> =
            Vec::new();
        let mut ordered_pairs: Vec<OrderedPairRow> = mutual_pairs
            .iter()
            .map(|(k, (a, b, pos))| (*k, *a, *b, *pos))
            .collect();
        ordered_pairs.sort_by_key(|(k, _, _, _)| (k.smaller(), k.larger()));
        for (key, a_idx, b_idx, position) in &ordered_pairs {
            let key = *key;
            let a_idx = *a_idx;
            let b_idx = *b_idx;
            let position = *position;
            use std::collections::hash_map::Entry;
            // Detect the handshake tick: was a SocialInteractionStarted for
            // THIS pair emitted on THIS tile on the current tick?
            let canonical_agents = (key.smaller(), key.larger());
            let tile_idx = position.1 * width + position.0;
            let just_started = resources
                .causal_log
                .get(tile_idx)
                .map(|log| {
                    log.as_slice().iter().any(|ev| {
                        matches!(
                            ev,
                            CausalEvent::SocialInteractionStarted { tick: t, agents, .. }
                                if *t == tick && *agents == canonical_agents
                        )
                    })
                })
                .unwrap_or(false);

            let post_value: u32 = match resources.interaction_progress.entry(key) {
                Entry::Vacant(slot) => {
                    // Handshake tick → seed at 0 (no increment); pre-placed
                    // → seed at 1 (this tick counts as the first progress
                    // tick so 3 SIS ticks complete the cycle).
                    let initial = if just_started { 0 } else { 1 };
                    slot.insert(initial);
                    initial
                }
                Entry::Occupied(mut slot) => {
                    let v = slot.get_mut();
                    *v += 1;
                    *v
                }
            };
            if post_value >= REQUIRED_INTERACTION_PROGRESS {
                completion_outputs.push((
                    key,
                    position,
                    consuming[a_idx].entity,
                    consuming[b_idx].entity,
                ));
            }
        }

        // (f) Apply completions.
        for (key, position, ent_a, ent_b) in &completion_outputs {
            // Bump familiarity (saturating at 1.0 via RelationshipState::bump).
            let entry = resources.relationships.entry(*key).or_default();
            entry.bump(FAMILIARITY_BUMP);
            let familiarity_after = entry.familiarity;

            // Find parent: most recent same-tile SocialInteractionStarted
            // for this canonical pair.
            let tile_idx = position.1 * width + position.0;
            let parent = resources.causal_log.get(tile_idx).and_then(|log| {
                log.as_slice().iter().rev().find_map(|ev| match ev {
                    CausalEvent::SocialInteractionStarted { id, agents, .. }
                        if *agents == (key.smaller(), key.larger()) =>
                    {
                        Some(*id)
                    }
                    _ => None,
                })
            });

            let completed_id = resources.issue_event_id();
            resources.causal_log.push(
                tile_idx,
                CausalEvent::SocialInteractionCompleted {
                    id: completed_id,
                    parent,
                    agents: (key.smaller(), key.larger()),
                    position: *position,
                    familiarity_after,
                    tick,
                },
            );

            // Reset both agents → Idle + saturating-subtract loneliness.
            for entity in [*ent_a, *ent_b] {
                if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                    *state = AgentState::Idle;
                }
                if let Ok(mut social) = world.get::<&mut Social>(entity) {
                    social.loneliness = (social.loneliness - SOCIAL_CONSUME_AMOUNT).max(0.0);
                }
            }

            // NOTE: do NOT remove the `interaction_progress` entry here.
            // The plan §γ A4/A5 contract requires that the terminal value
            // `REQUIRED_INTERACTION_PROGRESS` be observable AFTER the
            // completion tick's `engine.tick()` returns. Step (g) below
            // will NOT prune this key on the completion tick because the
            // pair is still present in `mutual_pairs` (the pre-completion
            // snapshot drives step (g)). On the FOLLOWING SIS tick, the
            // agents are Idle, the live snapshot of mutual `Consuming`
            // pairs no longer contains this key, and step (g) reaps the
            // entry — fulfilling plan §γ A12 ("None or Some(0) by the
            // next post-completion observation").
        }

        // (g) Stale-progress cleanup. Any interaction_progress entry whose
        //     pair is NOT in `mutual_pairs` (the current active set) is
        //     stale — remove it. Completed pairs are also absent at this
        //     point because step (f) removed them above. This bounds the
        //     map to live mutual pairs only.
        let active_keys: HashSet<RelationshipKey> = mutual_pairs.keys().copied().collect();
        let stale: Vec<RelationshipKey> = resources
            .interaction_progress
            .keys()
            .copied()
            .filter(|k| !active_keys.contains(k))
            .collect();
        for key in stale {
            resources.interaction_progress.remove(&key);
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
        let s = SocialInteractionSystem::new();
        assert_eq!(s.name(), "SocialInteractionSystem");
        assert_eq!(s.priority(), 134);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn empty_world_tick_no_op() {
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        let mut sys = SocialInteractionSystem::new();
        sys.tick(&mut engine.world, &mut engine.resources);
        assert!(engine.resources.interaction_progress.is_empty());
        assert!(engine.resources.relationships.is_empty());
    }
}
