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

mod forage;
mod memory_bias;
mod needs;
mod routine;
mod system;
mod targets;
mod wood;

pub use memory_bias::{
    arm_priority_index, cascade_arm_memory_tag, CascadeArm, BIAS_FLIP_THRESHOLD,
};
pub use needs::{
    actionable_need, actionable_need_with_fatigue_threshold, clock_in_window, fatigue_threshold_at,
    minute_in_window, person_night_rest, NeedAvailability, CRITICAL_NEED, FAMILIARITY_BUMP,
    FATIGUE_CONSUME_AMOUNT, FATIGUE_THRESHOLD, HUNGER_CONSUME_AMOUNT, HUNGER_THRESHOLD,
    REQUIRED_INTERACTION_PROGRESS, SOCIAL_CONSUME_AMOUNT, SOCIAL_THRESHOLD, THIRST_CONSUME_AMOUNT,
    THIRST_THRESHOLD,
};
pub use system::AgentDecisionSystem;
pub use targets::{
    nearest_food_tile_in_radius, nearest_gather_tile, nearest_local_food, nearest_resource_tile,
    GATHER_SCAN_RADIUS,
};
