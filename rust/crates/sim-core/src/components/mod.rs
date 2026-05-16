//! V7 Phase 4-α — Canonical ECS components.
//!
//! First deliverable of the Agent Core phase (Week 7-8). Replaces the
//! Phase 2 local placeholder in `sim_systems::runtime::influence::agent_sample::Position`
//! per the self-documenting landmark at `agent_sample.rs:9-15`.
//!
//! Phase 5-α extends the surface with [`AgentId`] (re-exported here) and
//! the [`Hunger`] need component — the first daily-routine driver.
//!
//! Phase 5-β adds the second need ([`Thirst`]) and the agent-decision
//! FSM ([`AgentState`] + [`TargetKind`]) that consumes both needs and
//! emits the first agent-originated `CausalEvent::AgentDecision`.
//!
//! Phase 5-γ adds the third need ([`Sleep`]) alongside the day/night
//! clock substrate (see `sim_engine::SimResources::time_of_day` /
//! `ticks_per_day` and `sim_systems::runtime::needs::SleepDecaySystem`).
//!
//! Phase 6-α adds the construction data substrate ([`BlueprintId`],
//! [`BuildingBlueprint`], [`ConstructionSite`]) and extends
//! [`TargetKind`] with a 4th variant `ConstructionSite` — Phase 5-γ
//! Path (b) symmetry precedent. `AgentState` is intentionally unchanged.
//!
//! See `.harness/plans/phase4.md` §2, `.harness/plans/phase5.md`
//! §2.1 / §2.β / §2.γ, and `.harness/plans/phase6.md` §2.1 for the
//! full sub-stage decomposition.

pub mod agent;
pub mod agent_state;
pub mod construction;
pub mod hunger;
pub mod position;
pub mod sleep;
pub mod thirst;

pub use agent::{Agent, AgentId};
pub use agent_state::{AgentState, TargetKind};
pub use construction::{BlueprintId, BuildingBlueprint, ConstructionSite};
pub use hunger::Hunger;
pub use position::Position;
pub use sleep::Sleep;
pub use thirst::Thirst;
