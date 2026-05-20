//! V7 Phase 5-β — `AgentDecisionSystem` (priority 125, every tick).
//!
//! First agent-originated decision system. Owns the `Idle → Seeking →
//! Consuming → Idle` FSM in [`AgentState`] and emits the first
//! agent-originated [`CausalEvent::AgentDecision`] record.
//!
//! # Tick ordering
//!
//! ```text
//! 90   BuildingStampSystem
//! 100  InfluenceUpdateSystem
//! 110  AgentInfluenceSampleSystem
//! 120  AgentMovementSystem          ← Brownian step (suppressed when Seeking)
//! 125  AgentDecisionSystem          ← reads pre-decay Hunger/Thirst
//! 130  HungerDecaySystem
//! 131  ThirstDecaySystem
//! 1000 InfluenceVisualizationSystem
//! ```
//!
//! Placing the decision system between movement (120) and decay
//! (130/131) means: the agent reads the same `Hunger`/`Thirst` value
//! that the previous tick's visualisation observed, then decay applies
//! after for the next tick.
//!
//! # FSM rules (locked by P5β-4)
//!
//! - **Idle → Seeking { Food }**  when `Hunger.value > HUNGER_THRESHOLD`.
//! - **Idle → Seeking { Water }** when `Thirst.value > THIRST_THRESHOLD`
//!   *and* Hunger is not already over its threshold (Hunger wins ties so
//!   the FSM is deterministic).
//! - **Seeking { T } → Consuming { T }** when the agent's current tile
//!   contains a matching resource (`SimResources::food_tiles` /
//!   `water_tiles`).
//! - **Consuming { T } → Idle**     after decrementing the resource
//!   counter by 1 (saturating) and reducing the matching need by
//!   `HUNGER_CONSUME_AMOUNT` / `THIRST_CONSUME_AMOUNT`.
//!
//! Phase β does NOT yet implement pathing toward a remote resource —
//! `Seeking { T }` blocks Brownian motion (via
//! [`AgentState::suppresses_movement`]) so agents wait on their tile
//! for a resource to arrive (or the harness to stage it). Path-toward-
//! target lands in γ.
//!
//! # Causal chain
//!
//! On every `Idle → Seeking` transition the system emits
//! [`CausalEvent::AgentDecision`] onto the agent's current tile. The
//! `parent` field links to the most recent same-tile
//! `CausalEvent::InfluenceChanged` (if present), closing the
//! `BuildingPlaced → StampDirty → InfluenceChanged → AgentDecision`
//! chain that the "왜?" UI walks backwards. When no influence event has
//! ever been recorded on the agent's tile the decision becomes a chain
//! root (`parent: None`).

use std::collections::HashMap;

use hecs::World;
use sim_core::causal::{CausalEvent, CausalLogStorage, DecisionReason, EventId, MemoryRecallTrigger};
use sim_core::components::{
    Agent, AgentId, AgentState, BuildingBlueprint, ConstructionSite, Hunger, Memory, Position,
    Sleep, Social, TargetKind, Thirst, SALIENCE_FLOOR,
};
use sim_engine::{RuntimeSystem, SimResources};

use crate::runtime::memory::MAX_RECENCY_TICKS;

/// Cascade arm identifier — the 5 natural drives whose eligibility is
/// computed independently. Memory is a bias SOURCE, not an arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CascadeArm {
    Hunger,
    Thirst,
    Fatigue,
    Construction,
    Social,
    /// Memory-driven negative combat trigger. A non-natural cascade arm
    /// activated by `memory_weight_delta` strictly below
    /// `-BIAS_FLIP_THRESHOLD`. Phase 9-β / P9β-5.
    Combat,
}

/// Linear recency factor: 1.0 at `elapsed == 0`, 0.0 at
/// `elapsed >= MAX_RECENCY_TICKS`.
fn recency_factor(encoded_tick: u64, current_tick: u64) -> f64 {
    let elapsed = current_tick.saturating_sub(encoded_tick);
    if elapsed >= MAX_RECENCY_TICKS {
        0.0
    } else {
        1.0 - (elapsed as f64 / MAX_RECENCY_TICKS as f64)
    }
}

/// Classify a memory entry's `event_id` against a cascade arm by looking
/// up the referenced [`CausalEvent`] in `causal_log`. Returns `false` on
/// lookup miss (Phase 3-β graceful eviction).
fn event_id_matches_arm(
    event_id: EventId,
    arm: CascadeArm,
    causal_log: &CausalLogStorage,
) -> bool {
    let Some(event) = causal_log.lookup(event_id) else {
        return false;
    };
    matches!(
        (arm, event),
        (CascadeArm::Hunger, CausalEvent::AgentDecision { reason: DecisionReason::HungerThresholdBreach, .. })
        | (CascadeArm::Thirst, CausalEvent::AgentDecision { reason: DecisionReason::ThirstThresholdBreach, .. })
        | (CascadeArm::Fatigue, CausalEvent::AgentDecision { reason: DecisionReason::FatigueThresholdBreach, .. })
        | (CascadeArm::Construction, CausalEvent::AgentDecision { reason: DecisionReason::ConstructionReason, .. })
        | (CascadeArm::Construction, CausalEvent::ConstructionStarted { .. })
        | (CascadeArm::Construction, CausalEvent::ConstructionCompleted { .. })
        | (CascadeArm::Social, CausalEvent::AgentDecision { reason: DecisionReason::SocialReason, .. })
        | (CascadeArm::Social, CausalEvent::SocialInteractionStarted { .. })
        | (CascadeArm::Social, CausalEvent::SocialInteractionCompleted { .. })
        | (CascadeArm::Combat, CausalEvent::CombatStarted { .. })
        | (CascadeArm::Combat, CausalEvent::CombatCompleted { .. })
        | (CascadeArm::Combat, CausalEvent::AgentDecision { reason: DecisionReason::CombatReason, .. })
    )
}

/// Compute the cascade-bias weight delta for `arm` from a snapshot of
/// `memory`. Linear product form per plan §2 P8β-MOD-2 step 4:
/// `sum(valence * salience * recency_factor)` over entries that
/// (a) match the arm, AND (b) have salience strictly above
/// `SALIENCE_FLOOR` (eligibility gate from plan A16).
fn memory_weight_delta(
    memory: &Memory,
    arm: CascadeArm,
    current_tick: u64,
    causal_log: &CausalLogStorage,
) -> f64 {
    memory
        .entries
        .iter()
        .filter(|entry| entry.salience > SALIENCE_FLOOR)
        .filter(|entry| event_id_matches_arm(entry.event_id, arm, causal_log))
        .map(|entry| {
            let recency = recency_factor(entry.encoded_tick, current_tick);
            entry.valence * entry.salience * recency
        })
        .sum()
}

/// Find the highest-magnitude contributing entry for `arm` — used as the
/// `recalled_event` field on `MemoryRecalled` emissions.
fn top_contributor_entry(
    memory: &Memory,
    arm: CascadeArm,
    current_tick: u64,
    causal_log: &CausalLogStorage,
) -> Option<EventId> {
    memory
        .entries
        .iter()
        .filter(|entry| entry.salience > SALIENCE_FLOOR)
        .filter(|entry| event_id_matches_arm(entry.event_id, arm, causal_log))
        .max_by(|a, b| {
            let wa = (a.valence * a.salience * recency_factor(a.encoded_tick, current_tick)).abs();
            let wb = (b.valence * b.salience * recency_factor(b.encoded_tick, current_tick)).abs();
            wa.partial_cmp(&wb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|e| e.event_id)
}

/// Per-arm natural eligibility score (binary 0.0/1.0 baseline). Memory
/// weight deltas are added on top. The arm with the highest adjusted
/// score wins; ties resolved by the original (Hunger > Thirst > Fatigue >
/// Construction > Social) priority order.
fn arm_priority_index(arm: CascadeArm) -> u8 {
    match arm {
        CascadeArm::Hunger => 0,
        CascadeArm::Thirst => 1,
        CascadeArm::Fatigue => 2,
        CascadeArm::Construction => 3,
        CascadeArm::Social => 4,
        CascadeArm::Combat => 5,
    }
}

/// V7 Phase 8-β cascade-bias flip threshold (P8β-MOD-2 step 5 refinement).
///
/// A non-natural-winner arm only flips the cascade when its
/// `memory_weight_delta` STRICTLY exceeds this threshold — i.e. the
/// memory-adjusted score must surpass the natural arm's binary
/// eligibility score (1.0). Below the threshold, the natural winner is
/// preserved and no `MemoryRecalled` event is emitted (plan §A20: bias
/// observed but non-load-bearing).
///
/// Locked at 1.0 (equal to the natural arm's binary eligibility).
/// Only a delta strictly GREATER than 1.0 can override the natural
/// winner, preventing weak positive bias from causing spurious flips.
pub const BIAS_FLIP_THRESHOLD: f64 = 1.0;


/// Hunger value strictly above which an Idle agent transitions to
/// `Seeking { Food }`. Phase 5-β scope: a flat constant. Per-agent
/// thresholds (temperament-driven) land in δ.
///
/// Typed `f32` to match [`Hunger::value`](sim_core::components::Hunger),
/// which itself remains `f32` per the locked α scope. Thirst-side
/// constants (and the rest of the new β math) use `f64` per the
/// project's "ALL f64 for simulation math" rule.
pub const HUNGER_THRESHOLD: f32 = 50.0;

/// Thirst value strictly above which an Idle agent transitions to
/// `Seeking { Water }` (only if Hunger has not already triggered).
///
/// `f64` to match the new β [`Thirst`] component and the project's
/// simulation-math determinism rule.
pub const THIRST_THRESHOLD: f64 = 50.0;

/// Amount subtracted from `Hunger.value` when a `Consuming { Food }`
/// step completes. Chosen so a single Consume cleanly drops a freshly
/// breached agent (value ≈ 51) back below `HUNGER_THRESHOLD`.
///
/// `f32` to match [`Hunger::value`](sim_core::components::Hunger).
pub const HUNGER_CONSUME_AMOUNT: f32 = 30.0;

/// Amount subtracted from `Thirst.value` when a `Consuming { Water }`
/// step completes. Mirrors `HUNGER_CONSUME_AMOUNT` (in `f64` to match
/// the new β [`Thirst`] component).
pub const THIRST_CONSUME_AMOUNT: f64 = 30.0;

/// Fatigue value strictly above which an Idle agent transitions to
/// `Seeking { Sleep }` (only if neither Hunger nor Thirst has already
/// triggered).
///
/// `f64` to match the new γ [`Sleep`] component (V7 Phase 5-γ / P5γ-8).
pub const FATIGUE_THRESHOLD: f64 = 50.0;

/// Amount subtracted from `Sleep.fatigue` when a `Consuming { Sleep }`
/// step completes. Mirrors `HUNGER_CONSUME_AMOUNT` / `THIRST_CONSUME_AMOUNT`
/// (in `f64` to match the new γ [`Sleep`] component, V7 Phase 5-γ / P5γ-8).
pub const FATIGUE_CONSUME_AMOUNT: f64 = 30.0;

/// Loneliness value strictly above which an Idle agent transitions to
/// `Seeking { Agent(other) }` (only if no higher-priority drive has
/// already triggered). V7 Phase 7-β / P7β-14.
///
/// `f64` to match the [`Social`] component.
pub const SOCIAL_THRESHOLD: f64 = 50.0;

/// Amount subtracted from `Social.loneliness` when a multi-tick social
/// interaction completes. Mirrors `HUNGER_CONSUME_AMOUNT` style.
/// V7 Phase 7-β / P7β-14.
pub const SOCIAL_CONSUME_AMOUNT: f64 = 30.0;

/// Number of ticks of mutual `Consuming { Agent(other) }` required before
/// a `SocialInteractionCompleted` is emitted. V7 Phase 7-β / P7β-14.
pub const REQUIRED_INTERACTION_PROGRESS: u32 = 3;

/// Amount added to `RelationshipState::familiarity` (saturating at 1.0)
/// on each completed interaction. V7 Phase 7-β / P7β-14.
pub const FAMILIARITY_BUMP: f64 = 0.1;

/// Phase 5-β decision system. Stateless — all per-agent state lives
/// in the [`AgentState`], [`Hunger`], [`Thirst`] components and the
/// sparse `food_tiles` / `water_tiles` maps on [`SimResources`].
#[derive(Debug, Default)]
pub struct AgentDecisionSystem;

impl AgentDecisionSystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for AgentDecisionSystem {
    fn name(&self) -> &str {
        "AgentDecisionSystem"
    }

    fn priority(&self) -> u32 {
        125
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

        // V7 Phase 6-β / P6β-7 — co-location map snapshot. Pre-built
        // here (BEFORE the agent query) so the agent loop can look up
        // `(pos.x, pos.y) → (Entity, BuildingBlueprint)` without
        // re-entering the world during a borrowed query. ConstructionSystem
        // at priority 133 owns Consuming-state exit; this map is only
        // consumed by the Idle branch's 4th cascade and the Seeking
        // branch's has_resource check.
        //
        // Attempt-3 fix: filter out sites with `is_complete()` so the Idle
        // cascade and the Seeking transition never treat a fully-built
        // (or `required_progress == 0`) site as an active construction
        // target. Otherwise an agent could enter `Seeking{ConstructionSite}`
        // (or even `Consuming{ConstructionSite}`) on a site that
        // `ConstructionSystem` never advances — the agent would get
        // stuck in Consuming with no progress and no completion edge.
        let construction_sites: HashMap<(u32, u32), (hecs::Entity, BuildingBlueprint)> = {
            let mut sites = HashMap::new();
            let mut q = world.query::<&ConstructionSite>();
            for (e, site) in q.iter() {
                if site.is_complete() {
                    continue;
                }
                sites.insert((site.position.x, site.position.y), (e, site.blueprint));
            }
            sites
        };

        // V7 Phase 7-β / P7β-6 — social-eligible snapshot. An Idle agent is
        // social-eligible iff its loneliness breaches `SOCIAL_THRESHOLD`
        // AND it has no higher-priority drive breached (Needs +
        // Construction). Computed BEFORE the agent mutation loop so the
        // social cascade does not pick partners that the cascade itself
        // routes elsewhere (e.g. partner has Hunger > threshold). This
        // closes A6a/A6b/A6c — without it, agent B could pick agent A as
        // a social partner even though A's cascade resolves to Hunger.
        let social_eligible_by_pos: HashMap<(u32, u32), Vec<AgentId>> = {
            let mut map: HashMap<(u32, u32), Vec<AgentId>> = HashMap::new();
            let mut q = world.query::<(
                &Agent,
                &Position,
                &AgentState,
                Option<&Hunger>,
                Option<&Thirst>,
                Option<&Sleep>,
                Option<&Social>,
            )>();
            for (_, (a, pos, state, h, t, s, social_opt)) in q.iter() {
                if !matches!(*state, AgentState::Idle) {
                    continue;
                }
                let lone_breach = social_opt.is_some_and(|x| x.loneliness > SOCIAL_THRESHOLD);
                let hunger_breach = h.is_some_and(|x| x.value > HUNGER_THRESHOLD);
                let thirst_breach = t.is_some_and(|x| x.value > THIRST_THRESHOLD);
                let fatigue_breach = s.is_some_and(|x| x.fatigue > FATIGUE_THRESHOLD);
                let construction_breach = construction_sites.contains_key(&(pos.x, pos.y));
                if lone_breach
                    && !hunger_breach
                    && !thirst_breach
                    && !fatigue_breach
                    && !construction_breach
                {
                    map.entry((pos.x, pos.y)).or_default().push(a.id);
                }
            }
            map
        };

        // V7 Phase 7-β / P7β-7 — Seeking{Agent(_)} snapshot. Records the
        // START-OF-TICK partner pointer for every agent currently in
        // `Seeking { Agent(partner_id) }`. The Seeking branch consults
        // this snapshot (NOT the live world state) so a mutual pair can
        // BOTH transition to Consuming this tick — without the snapshot,
        // the first agent processed transitions to Consuming and the
        // second observes a non-Seeking partner, breaking the handshake.
        let seeking_partners_by_pos: HashMap<(u32, u32), Vec<(AgentId, AgentId)>> = {
            let mut map: HashMap<(u32, u32), Vec<(AgentId, AgentId)>> = HashMap::new();
            let mut q = world.query::<(&Agent, &Position, &AgentState)>();
            for (_, (a, pos, state)) in q.iter() {
                if let AgentState::Seeking {
                    target: TargetKind::Agent(partner_id),
                } = *state
                {
                    map.entry((pos.x, pos.y))
                        .or_default()
                        .push((a.id, partner_id));
                }
            }
            map
        };

        // V7 Phase 8-β — Bias-path social-peer map. Lists co-located peers
        // whose loneliness breaches `SOCIAL_THRESHOLD`, regardless of
        // their other drives. The cascade-bias flip path needs this
        // wider eligibility so a Memory-driven Social win can target a
        // peer even when that peer's natural cascade routes to Hunger
        // or Construction. The strict-eligibility map
        // (`social_eligible_by_pos`) above is preserved for the natural
        // cascade.
        let social_partner_breached_by_pos: HashMap<(u32, u32), Vec<AgentId>> = {
            let mut map: HashMap<(u32, u32), Vec<AgentId>> = HashMap::new();
            let mut q = world.query::<(&Agent, &Position, &AgentState, Option<&Social>)>();
            for (_, (a, pos, state, social_opt)) in q.iter() {
                if !matches!(*state, AgentState::Idle) {
                    continue;
                }
                if social_opt.is_some_and(|s| s.loneliness > SOCIAL_THRESHOLD) {
                    map.entry((pos.x, pos.y)).or_default().push(a.id);
                }
            }
            map
        };

        // V7 Phase 8-β — all-idle-peer map for the bias-only Social eligibility
        // path (plan §A16 arm-symmetry fix). Captures ANY idle agent at the
        // same tile regardless of loneliness, unlike
        // `social_partner_breached_by_pos` which gates on
        // `loneliness > SOCIAL_THRESHOLD`.
        let all_idle_peers_by_pos: HashMap<(u32, u32), Vec<AgentId>> = {
            let mut map: HashMap<(u32, u32), Vec<AgentId>> = HashMap::new();
            let mut q = world.query::<(&Agent, &Position, &AgentState)>();
            for (_, (a, pos, state)) in q.iter() {
                if matches!(*state, AgentState::Idle) {
                    map.entry((pos.x, pos.y)).or_default().push(a.id);
                }
            }
            map
        };

        // Per-agent FSM evaluation. We pull `Hunger` / `Thirst` as
        // optional so non-need-carrying agents are still observed and
        // simply stay `Idle`. V7 Phase 7-β adds `Option<&Social>` for the
        // 5th cascade arm. V7 Phase 8-β adds `Option<&Memory>` for the
        // cascade-bias source.
        let mut query = world.query::<(
            &Position,
            &Agent,
            &mut AgentState,
            Option<&mut Hunger>,
            Option<&mut Thirst>,
            Option<&mut Sleep>,
            Option<&Social>,
            Option<&Memory>,
        )>();
        for (_entity, (pos, agent, state, hunger_opt, thirst_opt, sleep_opt, social_opt, memory_opt)) in
            query.iter()
        {
            let tile_idx = pos.y * width + pos.x;
            match *state {
                AgentState::Idle => {
                    // Deterministic FSM ordering (V7 Phase 7-β / P7β-6):
                    // Hunger > Thirst > Fatigue > Construction > Social.
                    // Social is the lowest-priority drive — Needs and
                    // Construction always win.
                    let breached = if hunger_opt
                        .as_ref()
                        .is_some_and(|h| h.value > HUNGER_THRESHOLD)
                    {
                        Some((TargetKind::Food, DecisionReason::HungerThresholdBreach))
                    } else if thirst_opt
                        .as_ref()
                        .is_some_and(|t| t.value > THIRST_THRESHOLD)
                    {
                        Some((TargetKind::Water, DecisionReason::ThirstThresholdBreach))
                    } else if sleep_opt
                        .as_ref()
                        .is_some_and(|s| s.fatigue > FATIGUE_THRESHOLD)
                    {
                        Some((TargetKind::Sleep, DecisionReason::FatigueThresholdBreach))
                    } else if construction_sites.contains_key(&(pos.x, pos.y)) {
                        // V7 Phase 6-β / P6β-6: 4th cascade step. Fires
                        // ONLY when no Need has breached AND a
                        // co-located active ConstructionSite exists.
                        Some((
                            TargetKind::ConstructionSite,
                            DecisionReason::ConstructionReason,
                        ))
                    } else if social_opt.is_some_and(|s| s.loneliness > SOCIAL_THRESHOLD) {
                        // V7 Phase 7-β / P7β-6 — 5th cascade step. Pick the
                        // lowest-AgentId co-located social-eligible peer
                        // (deterministic tie-break). Eligibility (computed
                        // up-top in `social_eligible_by_pos`) excludes any
                        // peer whose cascade routes elsewhere.
                        social_eligible_by_pos
                            .get(&(pos.x, pos.y))
                            .and_then(|peers| {
                                peers
                                    .iter()
                                    .filter(|id| **id != agent.id)
                                    .min()
                                    .copied()
                            })
                            .map(|other_id| {
                                (
                                    TargetKind::Agent(other_id),
                                    DecisionReason::SocialReason,
                                )
                            })
                    } else {
                        // 7th cascade arm: combat. Fires when no higher-
                        // priority drive is active AND the agent's
                        // memory_weight_delta for the Combat arm is
                        // strictly below -BIAS_FLIP_THRESHOLD.
                        // Requires a co-located idle peer (lowest-AgentId).
                        // Phase 9-β / P9β-2 / P9β-5.
                        if let Some(memory) = memory_opt {
                            let combat_delta = memory_weight_delta(
                                memory,
                                CascadeArm::Combat,
                                tick,
                                &resources.causal_log,
                            );
                            if combat_delta < -BIAS_FLIP_THRESHOLD {
                                all_idle_peers_by_pos
                                    .get(&(pos.x, pos.y))
                                    .and_then(|peers| {
                                        peers
                                            .iter()
                                            .filter(|id| **id != agent.id)
                                            .min()
                                            .copied()
                                    })
                                    .map(|enemy_id| {
                                        (
                                            TargetKind::Agent(enemy_id),
                                            DecisionReason::CombatReason,
                                        )
                                    })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    if let Some((natural_target, natural_reason)) = breached {
                        // Phase 9-β / P9β-1 — combat arm early-exit handler.
                        // Combat bypasses the memory-bias flip machinery
                        // (it IS a memory-driven trigger) and transitions
                        // directly to Consuming{Agent}, emitting its own
                        // MemoryRecalled + AgentDecision{CombatReason}
                        // + (smaller-id only) CombatStarted causal chain.
                        if natural_reason == DecisionReason::CombatReason {
                            if let TargetKind::Agent(enemy_id) = natural_target {
                                let Some(memory) = memory_opt else {
                                    continue; // defensive — combat arm required memory
                                };
                                let Some(recalled_event) = top_contributor_entry(
                                    memory,
                                    CascadeArm::Combat,
                                    tick,
                                    &resources.causal_log,
                                ) else {
                                    continue; // top contributor evicted between scoring and emission
                                };
                                let recall_parent = resources
                                    .causal_log
                                    .lookup(recalled_event)
                                    .and_then(|e| e.parent());
                                let recall_id = resources.issue_event_id();
                                resources.causal_log.push(
                                    tile_idx,
                                    CausalEvent::MemoryRecalled {
                                        id: recall_id,
                                        parent: recall_parent,
                                        agent: agent.id,
                                        recalled_event,
                                        triggered_by: MemoryRecallTrigger::CombatContext {
                                            agent_id: enemy_id,
                                        },
                                        tick,
                                    },
                                );
                                let decision_id = resources.issue_event_id();
                                resources.causal_log.push(
                                    tile_idx,
                                    CausalEvent::AgentDecision {
                                        id: decision_id,
                                        parent: Some(recall_id),
                                        agent: agent.id,
                                        position: (pos.x, pos.y),
                                        reason: DecisionReason::CombatReason,
                                        tick,
                                    },
                                );
                                // Emit CombatStarted once per pair — only
                                // from the smaller AgentId.
                                if agent.id < enemy_id {
                                    let started_id = resources.issue_event_id();
                                    resources.causal_log.push(
                                        tile_idx,
                                        CausalEvent::CombatStarted {
                                            id: started_id,
                                            parent: Some(decision_id),
                                            attacker: agent.id,
                                            defender: enemy_id,
                                            position: (pos.x, pos.y),
                                            tick,
                                        },
                                    );
                                }
                                // Direct Idle → Consuming{Agent} transition
                                // (no Seeking step). Insert canonical pair.
                                *state = AgentState::Consuming {
                                    target: TargetKind::Agent(enemy_id),
                                };
                                let canonical = if agent.id < enemy_id {
                                    (agent.id, enemy_id)
                                } else {
                                    (enemy_id, agent.id)
                                };
                                resources.combat_pairs.insert(canonical);
                            }
                            continue;
                        }
                        // V7 Phase 8-β cascade-bias check (P8β-MOD-2). Only
                        // performed when the agent has a Memory component
                        // AND at least one non-natural-winner arm is also
                        // eligible (otherwise there is no candidate to
                        // flip to).
                        let natural_arm = match natural_reason {
                            DecisionReason::HungerThresholdBreach => CascadeArm::Hunger,
                            DecisionReason::ThirstThresholdBreach => CascadeArm::Thirst,
                            DecisionReason::FatigueThresholdBreach => CascadeArm::Fatigue,
                            DecisionReason::ConstructionReason => CascadeArm::Construction,
                            DecisionReason::SocialReason => CascadeArm::Social,
                            DecisionReason::MemoryReason => CascadeArm::Construction,
                            // Unreachable in practice — CombatReason hits
                            // `continue` above. Added for exhaustiveness.
                            DecisionReason::CombatReason => CascadeArm::Combat,
                        };

                        // Build the (arm, target) eligibility set for the
                        // bias path. Eligibility uses RELAXED Social rules
                        // (peer's own breach state ignored) so the bias
                        // can shift to Social over Construction.
                        let mut eligible: Vec<(CascadeArm, TargetKind)> = Vec::new();
                        if hunger_opt.as_ref().is_some_and(|h| h.value > HUNGER_THRESHOLD) {
                            eligible.push((CascadeArm::Hunger, TargetKind::Food));
                        }
                        if thirst_opt.as_ref().is_some_and(|t| t.value > THIRST_THRESHOLD) {
                            eligible.push((CascadeArm::Thirst, TargetKind::Water));
                        }
                        if sleep_opt.as_ref().is_some_and(|s| s.fatigue > FATIGUE_THRESHOLD) {
                            eligible.push((CascadeArm::Fatigue, TargetKind::Sleep));
                        }
                        if construction_sites.contains_key(&(pos.x, pos.y)) {
                            eligible.push((
                                CascadeArm::Construction,
                                TargetKind::ConstructionSite,
                            ));
                        }
                        if social_opt.is_some_and(|s| s.loneliness > SOCIAL_THRESHOLD) {
                            if let Some(peer_id) = social_partner_breached_by_pos
                                .get(&(pos.x, pos.y))
                                .and_then(|peers| {
                                    peers.iter().filter(|id| **id != agent.id).min().copied()
                                })
                            {
                                eligible.push((CascadeArm::Social, TargetKind::Agent(peer_id)));
                            }
                        }
                        // Bias-only Construction eligibility: add Construction
                        // arm even when no active ECS ConstructionSite exists
                        // at the agent's tile, if the agent has Memory entries
                        // whose event_id matches the Construction arm. Mirrors
                        // the social_partner_breached_by_pos extension for the
                        // Social arm (plan §A17: Social natural winner →
                        // Construction flip via memory bias).
                        if !eligible.iter().any(|(a, _)| *a == CascadeArm::Construction)
                            && memory_opt.is_some_and(|m| {
                                m.entries.iter().any(|e| {
                                    e.salience > SALIENCE_FLOOR
                                        && event_id_matches_arm(
                                            e.event_id,
                                            CascadeArm::Construction,
                                            &resources.causal_log,
                                        )
                                })
                            })
                        {
                            eligible.push((
                                CascadeArm::Construction,
                                TargetKind::ConstructionSite,
                            ));
                        }
                        // Bias-only Social eligibility (plan §A16 arm-symmetry
                        // fix): add Social arm when Memory has Social-arm
                        // entries with salience > SALIENCE_FLOOR even when the
                        // focal agent's loneliness does not breach
                        // SOCIAL_THRESHOLD. Symmetric to bias-only Construction.
                        if !eligible.iter().any(|(a, _)| *a == CascadeArm::Social)
                            && memory_opt.is_some_and(|m| {
                                m.entries.iter().any(|e| {
                                    e.salience > SALIENCE_FLOOR
                                        && event_id_matches_arm(
                                            e.event_id,
                                            CascadeArm::Social,
                                            &resources.causal_log,
                                        )
                                })
                            })
                        {
                            if let Some(peer_id) = all_idle_peers_by_pos
                                .get(&(pos.x, pos.y))
                                .and_then(|peers| {
                                    peers.iter().filter(|id| **id != agent.id).min().copied()
                                })
                            {
                                eligible.push((CascadeArm::Social, TargetKind::Agent(peer_id)));
                            }
                        }

                        // V7 Phase 8-β threshold-based flip decision
                        // (P8β-MOD-2 step 5 refinement). A non-natural arm
                        // flips the cascade ONLY when its adjusted score
                        // (memory_weight_delta) strictly exceeds the
                        // natural arm's adjusted score
                        // (BIAS_FLIP_THRESHOLD + natural arm's own delta).
                        // BIAS_FLIP_THRESHOLD (= 1.0) is the natural arm's
                        // intrinsic binary eligibility; the natural arm's
                        // memory delta raises the bar further, so an
                        // overwhelmingly strong natural winner is preserved
                        // (plan §A19/P8β-MOD-2).
                        //
                        // Among eligible non-natural arms whose deltas pass
                        // the natural margin, the highest delta wins; ties
                        // resolved by the natural cascade priority order.
                        let mut flip_target_arm: Option<(CascadeArm, TargetKind, f64)> =
                            None;
                        if let Some(memory) = memory_opt.filter(|_| eligible.len() >= 2) {
                            let natural_delta = memory_weight_delta(
                                memory,
                                natural_arm,
                                tick,
                                &resources.causal_log,
                            );
                            let natural_margin = BIAS_FLIP_THRESHOLD + natural_delta;
                            for (arm, tk) in &eligible {
                                if *arm == natural_arm {
                                    continue;
                                }
                                let delta = memory_weight_delta(
                                    memory,
                                    *arm,
                                    tick,
                                    &resources.causal_log,
                                );
                                if delta <= natural_margin {
                                    continue;
                                }
                                match flip_target_arm {
                                    None => flip_target_arm = Some((*arm, *tk, delta)),
                                    Some((b_arm, _, b_delta)) => {
                                        if delta > b_delta
                                            || (delta == b_delta
                                                && arm_priority_index(*arm)
                                                    < arm_priority_index(b_arm))
                                        {
                                            flip_target_arm = Some((*arm, *tk, delta));
                                        }
                                    }
                                }
                            }
                        }

                        if let Some((bias_arm, bias_target, _)) = flip_target_arm {
                            // Cascade FLIP path — emit MemoryRecalled, then
                            // AgentDecision{MemoryReason} linked to it.
                            //
                            // `flip_target_arm` is only ever Some when the
                            // bias-scoring branch above ran, which required
                            // `memory_opt` to be Some. Use a safe match
                            // rather than `.unwrap()` (project rule: no
                            // unwrap() in production code).
                            let Some(memory) = memory_opt else {
                                // Unreachable in practice — defensive skip.
                                continue;
                            };
                            // Issue-5 fix: require a real load-bearing
                            // recalled_event. If no top contributor is
                            // resolvable (e.g. all matching entries were
                            // evicted between scoring and emission), skip
                            // the flip entirely — DO NOT emit a
                            // `MemoryRecalled` carrying a fabricated
                            // `recalled_event = 0`.
                            let Some(recalled_event) = top_contributor_entry(
                                memory,
                                bias_arm,
                                tick,
                                &resources.causal_log,
                            ) else {
                                continue;
                            };
                            let recall_parent = resources
                                .causal_log
                                .lookup(recalled_event)
                                .and_then(|e| e.parent());
                            let recall_id = resources.issue_event_id();
                            resources.causal_log.push(
                                tile_idx,
                                CausalEvent::MemoryRecalled {
                                    id: recall_id,
                                    parent: recall_parent,
                                    agent: agent.id,
                                    recalled_event,
                                    triggered_by: MemoryRecallTrigger::CascadeBias,
                                    tick,
                                },
                            );
                            let decision_id = resources.issue_event_id();
                            resources.causal_log.push(
                                tile_idx,
                                CausalEvent::AgentDecision {
                                    id: decision_id,
                                    parent: Some(recall_id),
                                    agent: agent.id,
                                    position: (pos.x, pos.y),
                                    reason: DecisionReason::MemoryReason,
                                    tick,
                                },
                            );
                            *state = AgentState::Seeking { target: bias_target };
                        } else {
                            // Natural-cascade path — original behaviour.
                            let parent = resources
                                .causal_log
                                .get(tile_idx)
                                .and_then(|log| {
                                    log.as_slice().iter().rev().find_map(|ev| match ev {
                                        CausalEvent::InfluenceChanged { id, .. } => Some(*id),
                                        CausalEvent::AgentDecision {
                                            id,
                                            agent: a,
                                            reason: DecisionReason::SocialReason,
                                            ..
                                        } if matches!(
                                            natural_reason,
                                            DecisionReason::SocialReason
                                        ) && *a != agent.id =>
                                        {
                                            Some(*id)
                                        }
                                        _ => None,
                                    })
                                });
                            let id = resources.issue_event_id();
                            resources.causal_log.push(
                                tile_idx,
                                CausalEvent::AgentDecision {
                                    id,
                                    parent,
                                    agent: agent.id,
                                    position: (pos.x, pos.y),
                                    reason: natural_reason,
                                    tick,
                                },
                            );
                            *state = AgentState::Seeking {
                                target: natural_target,
                            };
                        }
                    }
                }
                AgentState::Seeking { target } => {
                    let key = (pos.x, pos.y);
                    let has_resource = match target {
                        TargetKind::Food => resources
                            .food_tiles
                            .get(&key)
                            .copied()
                            .is_some_and(|v| v > 0),
                        TargetKind::Water => resources
                            .water_tiles
                            .get(&key)
                            .copied()
                            .is_some_and(|v| v > 0),
                        TargetKind::Sleep => resources
                            .sleep_tiles
                            .get(&key)
                            .copied()
                            .is_some_and(|v| v > 0),
                        // V7 Phase 6-β / P6β-8: ConstructionSite has a
                        // resource at this tile iff an active site exists
                        // co-located with the agent. Snapshot taken at
                        // the top of `tick` via `construction_sites`.
                        TargetKind::ConstructionSite => {
                            construction_sites.contains_key(&key)
                        }
                        // V7 Phase 7-β / P7β-7 — mutual handshake check.
                        // Uses the start-of-tick `seeking_partners_by_pos`
                        // snapshot so both agents in a mutual pair pass
                        // the check this tick (live state mutation would
                        // break the second-processed agent's check).
                        TargetKind::Agent(partner_id) => seeking_partners_by_pos
                            .get(&(pos.x, pos.y))
                            .map(|peers| {
                                peers.iter().any(|(peer_id, peer_target)| {
                                    *peer_id == partner_id && *peer_target == agent.id
                                })
                            })
                            .unwrap_or(false),
                    };
                    if has_resource {
                        // V7 Phase 6-β / P6β-8: emit ConstructionStarted on
                        // the Seeking{ConstructionSite} → Consuming{ConstructionSite}
                        // transition. Parent = most recent same-tile
                        // AgentDecision for THIS agent with reason
                        // ConstructionReason (the originating decision —
                        // not a HungerThresholdBreach decision, not a
                        // different agent's decision).
                        if matches!(target, TargetKind::ConstructionSite) {
                            if let Some((_site_entity, blueprint)) =
                                construction_sites.get(&key).copied()
                            {
                                let parent = resources.causal_log.get(tile_idx).and_then(|log| {
                                    log.as_slice().iter().rev().find_map(|ev| match ev {
                                        CausalEvent::AgentDecision {
                                            id,
                                            agent: a,
                                            reason: DecisionReason::ConstructionReason,
                                            ..
                                        } if *a == agent.id => Some(*id),
                                        _ => None,
                                    })
                                });
                                let id = resources.issue_event_id();
                                resources.causal_log.push(
                                    tile_idx,
                                    CausalEvent::ConstructionStarted {
                                        id,
                                        parent,
                                        blueprint,
                                        position: (pos.x, pos.y),
                                        tick,
                                    },
                                );
                            }
                        }
                        // V7 Phase 7-β / P7β-7/P7β-8 — emit
                        // SocialInteractionStarted exactly once per mutual
                        // handshake: only the smaller-AgentId agent emits.
                        // The partner observes the handshake silently
                        // (transitions to Consuming via its own
                        // snapshot-check path).
                        if let TargetKind::Agent(partner_id) = target {
                            if agent.id < partner_id {
                                let parent: Option<EventId> = resources
                                    .causal_log
                                    .get(tile_idx)
                                    .and_then(|log| {
                                        log.as_slice().iter().rev().find_map(|ev| match ev {
                                            CausalEvent::AgentDecision {
                                                id,
                                                agent: a,
                                                reason: DecisionReason::SocialReason,
                                                ..
                                            } if *a == agent.id => Some(*id),
                                            _ => None,
                                        })
                                    });
                                let id = resources.issue_event_id();
                                resources.causal_log.push(
                                    tile_idx,
                                    CausalEvent::SocialInteractionStarted {
                                        id,
                                        parent,
                                        agents: (agent.id, partner_id),
                                        position: (pos.x, pos.y),
                                        tick,
                                    },
                                );
                            }
                        }
                        *state = AgentState::Consuming { target };
                    }
                }
                AgentState::Consuming { target } => {
                    let key = (pos.x, pos.y);
                    // Section-3.10 conditional mutation rule (plan attempt 3):
                    //   - tile map mutation is conditional on Some
                    //   - need decrement is UNCONDITIONAL
                    //   - state → Idle is UNCONDITIONAL
                    //
                    // Rationale: when an agent enters Consuming but the
                    // tile is absent (e.g. another agent consumed it the
                    // prior tick, or the tile was never populated), the
                    // agent still completes its consume action — Hunger /
                    // Thirst drop, state returns to Idle. This is the
                    // locked behavior under Assertions 16 and 17.
                    // The agent must NOT synthesise a phantom 0-counter
                    // entry; `entry().or_insert(0)` is forbidden.
                    match target {
                        TargetKind::Food => {
                            if let Some(counter) = resources.food_tiles.get_mut(&key) {
                                *counter = counter.saturating_sub(1);
                                if *counter == 0 {
                                    resources.food_tiles.remove(&key);
                                }
                            }
                            if let Some(h) = hunger_opt {
                                h.value = (h.value - HUNGER_CONSUME_AMOUNT).max(0.0);
                            }
                            *state = AgentState::Idle;
                        }
                        TargetKind::Water => {
                            if let Some(counter) = resources.water_tiles.get_mut(&key) {
                                *counter = counter.saturating_sub(1);
                                if *counter == 0 {
                                    resources.water_tiles.remove(&key);
                                }
                            }
                            if let Some(t) = thirst_opt {
                                t.value = (t.value - THIRST_CONSUME_AMOUNT).max(0.0);
                            }
                            *state = AgentState::Idle;
                        }
                        TargetKind::Sleep => {
                            if let Some(counter) = resources.sleep_tiles.get_mut(&key) {
                                *counter = counter.saturating_sub(1);
                                if *counter == 0 {
                                    resources.sleep_tiles.remove(&key);
                                }
                            }
                            if let Some(s) = sleep_opt {
                                s.fatigue = (s.fatigue - FATIGUE_CONSUME_AMOUNT).max(0.0);
                            }
                            *state = AgentState::Idle;
                        }
                        // V7 Phase 6-α: ConstructionSite consume is β scope.
                        // The ConstructionSystem (β) is the sole owner of
                        // construction progress and completion behavior;
                        // α MUST NOT advance progress, mutate needs/resources,
                        // or transition the FSM state here. The match arm
                        // exists only to satisfy exhaustiveness — kept
                        // strictly inert and state-preserving so Phase 6-β
                        // remains the only place these semantics land.
                        TargetKind::ConstructionSite => {
                            // Intentional no-op: preserve state, do not touch
                            // tile maps, do not mutate needs, do not advance
                            // any construction progress. β scope.
                        }
                        // V7 Phase 7-α / P7α-12: Agent(_) consume is β scope.
                        // The SocialInteractionSystem (β, priority 134) is
                        // the sole owner of social-interaction progress,
                        // mutual-handshake validation, and FSM exit; α MUST
                        // NOT advance familiarity, mutate Social /
                        // RelationshipState, or transition the FSM state
                        // here. Named arm (not a wildcard) so a future 6th
                        // variant fails to compile here. Mirrors the Phase
                        // 6-α ConstructionSite inert pattern at lines
                        // ~362-366 above.
                        TargetKind::Agent(_) => {
                            // Intentional no-op: preserve state, do not touch
                            // relationships, do not bump familiarity, do not
                            // advance interaction_progress. β scope.
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    fn fresh_engine() -> SimEngine {
        SimEngine::new(32, 32, MaterialRegistry::new())
    }

    #[test]
    fn metadata() {
        let s = AgentDecisionSystem::new();
        assert_eq!(s.name(), "AgentDecisionSystem");
        assert_eq!(s.priority(), 125);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn idle_to_seeking_on_hunger_breach() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(3, 3);
        e.world
            .insert(entity, (AgentState::Idle, Hunger::new(60.0, 0.0)))
            .unwrap();

        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let state = *e.world.get::<&AgentState>(entity).unwrap();
        assert_eq!(state, AgentState::Seeking { target: TargetKind::Food });
    }

    #[test]
    fn idle_to_seeking_on_thirst_when_hunger_below() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(3, 3);
        e.world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(10.0, 0.0),
                    Thirst::new(60.0, 0.0),
                ),
            )
            .unwrap();

        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let state = *e.world.get::<&AgentState>(entity).unwrap();
        assert_eq!(
            state,
            AgentState::Seeking { target: TargetKind::Water }
        );
    }

    #[test]
    fn idle_stays_idle_when_both_below() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(3, 3);
        e.world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(10.0, 0.0),
                    Thirst::new(10.0, 0.0),
                ),
            )
            .unwrap();

        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        let state = *e.world.get::<&AgentState>(entity).unwrap();
        assert_eq!(state, AgentState::Idle);
    }

    #[test]
    fn seeking_transitions_to_consuming_on_food_tile() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(5, 7);
        e.world
            .insert(
                entity,
                (
                    AgentState::Seeking { target: TargetKind::Food },
                    Hunger::new(60.0, 0.0),
                ),
            )
            .unwrap();
        e.resources.food_tiles.insert((5, 7), 3);

        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let state = *e.world.get::<&AgentState>(entity).unwrap();
        assert_eq!(state, AgentState::Consuming { target: TargetKind::Food });
        // Counter is decremented only during the Consuming step.
        assert_eq!(e.resources.food_tiles.get(&(5, 7)), Some(&3));
    }

    #[test]
    fn consuming_decrements_food_and_hunger_then_idles() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(2, 2);
        e.world
            .insert(
                entity,
                (
                    AgentState::Consuming { target: TargetKind::Food },
                    Hunger::new(60.0, 0.0),
                ),
            )
            .unwrap();
        e.resources.food_tiles.insert((2, 2), 2);

        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let state = *e.world.get::<&AgentState>(entity).unwrap();
        let hunger = *e.world.get::<&Hunger>(entity).unwrap();
        assert_eq!(state, AgentState::Idle);
        assert_eq!(hunger.value, 60.0 - HUNGER_CONSUME_AMOUNT);
        assert_eq!(e.resources.food_tiles.get(&(2, 2)), Some(&1));
    }

    #[test]
    fn consuming_removes_tile_when_counter_hits_zero() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(1, 1);
        e.world
            .insert(
                entity,
                (
                    AgentState::Consuming { target: TargetKind::Water },
                    Thirst::new(60.0, 0.0),
                ),
            )
            .unwrap();
        e.resources.water_tiles.insert((1, 1), 1);

        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        assert!(!e.resources.water_tiles.contains_key(&(1, 1)));
    }

    #[test]
    fn breach_emits_agent_decision_event() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(4, 4);
        e.world
            .insert(entity, (AgentState::Idle, Hunger::new(60.0, 0.0)))
            .unwrap();

        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let width = e.resources.tile_grid.width;
        let tile_idx = 4 * width + 4;
        let log = e.resources.causal_log.get(tile_idx).expect("log present");
        let decision = log
            .as_slice()
            .iter()
            .find(|ev| matches!(ev, CausalEvent::AgentDecision { .. }))
            .expect("AgentDecision recorded");
        match decision {
            CausalEvent::AgentDecision {
                position, reason, ..
            } => {
                assert_eq!(*position, (4, 4));
                assert_eq!(*reason, DecisionReason::HungerThresholdBreach);
            }
            _ => unreachable!(),
        }
    }
}
