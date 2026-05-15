//! Decision-making runtime systems (V7 Phase 5-β / P5β-4).
//!
//! First agent-originated causal event milestone. The single system
//! shipped in β is [`AgentDecisionSystem`] (priority 125, every tick),
//! slotted between `AgentMovementSystem` (priority 120) and
//! `HungerDecaySystem` (priority 130). It owns the FSM transitions
//! that drive `Idle → Seeking → Consuming`, the `Consuming` consume
//! effect on `Hunger`/`Thirst`, and the emission of
//! [`CausalEvent::AgentDecision`].
//!
//! Later phases will extend this module with `combat_decision`,
//! `social_decision`, etc. — all priority 12x slots share the
//! "decision tier" of the schedule.

pub mod agent_decision;

pub use agent_decision::{
    AgentDecisionSystem, HUNGER_CONSUME_AMOUNT, HUNGER_THRESHOLD, THIRST_CONSUME_AMOUNT,
    THIRST_THRESHOLD,
};
