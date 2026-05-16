//! V7 Phase 6-β — Agent-driven construction runtime systems.
//!
//! Owns the progress-and-completion side of the construction loop.
//! `AgentDecisionSystem` (priority 125) handles all `AgentState`
//! transitions INTO and THROUGH `Seeking`/`Consuming` for
//! [`TargetKind::ConstructionSite`](sim_core::components::TargetKind::ConstructionSite);
//! this module's [`ConstructionSystem`] (priority 133) handles the OUT
//! transition (`Consuming → Idle`) plus all `ConstructionSite` progress
//! mutation, absent-site fallback, and `ConstructionCompleted +
//! BuildingPlaced` emission on the completion tick.

pub mod construction_system;

pub use construction_system::ConstructionSystem;
