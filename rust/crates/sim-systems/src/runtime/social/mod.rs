//! V7 Phase 7-β — Agent-to-agent social interaction runtime.
//!
//! Owns the progress-and-completion side of the social interaction loop.
//! `AgentDecisionSystem` (priority 125) handles all `AgentState`
//! transitions INTO and THROUGH `Seeking`/`Consuming` for
//! [`TargetKind::Agent`](sim_core::components::TargetKind::Agent); this
//! module's [`SocialInteractionSystem`] (priority 134) handles the OUT
//! transition (`Consuming → Idle`) plus per-pair progress, familiarity
//! bump, asymmetric-partner fallback, and stale-progress cleanup.

pub mod social_interaction_system;

pub use social_interaction_system::SocialInteractionSystem;
