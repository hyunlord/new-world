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

use hecs::World;
use sim_core::causal::{CausalEvent, DecisionReason};
use sim_core::components::{Agent, AgentState, Hunger, Position, Sleep, TargetKind, Thirst};
use sim_engine::{RuntimeSystem, SimResources};

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

        // Per-agent FSM evaluation. We pull `Hunger` / `Thirst` as
        // optional so non-need-carrying agents are still observed and
        // simply stay `Idle`.
        let mut query = world.query::<(
            &Position,
            &Agent,
            &mut AgentState,
            Option<&mut Hunger>,
            Option<&mut Thirst>,
            Option<&mut Sleep>,
        )>();
        for (_entity, (pos, agent, state, hunger_opt, thirst_opt, sleep_opt)) in query.iter() {
            let tile_idx = pos.y * width + pos.x;
            match *state {
                AgentState::Idle => {
                    // Deterministic FSM ordering: Hunger > Thirst > Fatigue.
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
                    } else {
                        None
                    };

                    if let Some((target, reason)) = breached {
                        // Walk this tile's existing log for a parent.
                        let parent = resources
                            .causal_log
                            .get(tile_idx)
                            .and_then(|log| {
                                log.as_slice().iter().rev().find_map(|ev| match ev {
                                    CausalEvent::InfluenceChanged { id, .. } => Some(*id),
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
                                reason,
                                tick,
                            },
                        );
                        *state = AgentState::Seeking { target };
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
                    };
                    if has_resource {
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
